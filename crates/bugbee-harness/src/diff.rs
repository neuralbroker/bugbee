use std::collections::HashSet;

use tracing::debug;

use crate::proto::{
    signal, DiffResponse, HttpResponse, Signal, VerificationStatus,
};

/// Differential oracle — analyzes baseline vs. exploit responses to detect
/// indicators of successful exploitation.
#[derive(Default)]
pub struct DiffOracle;

impl DiffOracle {
    /// Compare baseline and exploit responses and return detected signals.
    pub fn analyze(&self, baseline: &HttpResponse, exploit: &HttpResponse) -> DiffResponse {
        let mut signals = Vec::new();

        // 1. Status code change
        if baseline.status_code != exploit.status_code {
            signals.push(Signal {
                r#type: signal::SignalType::StatusChange as i32,
                evidence: format!(
                    "status code changed: {} → {}",
                    baseline.status_code, exploit.status_code
                ),
                confidence: 0.6,
            });
        }

        // 2. Time delay detection (threshold: 4s)
        let time_diff = exploit.duration_ms - baseline.duration_ms;
        if time_diff > 4000 {
            signals.push(Signal {
                r#type: signal::SignalType::TimeDelay as i32,
                evidence: format!("response time increased by {}ms", time_diff),
                confidence: 0.8,
            });
        }

        // 3. Error leakage detection
        let baseline_errors = extract_error_patterns(&baseline.body);
        let exploit_errors = extract_error_patterns(&exploit.body);
        let new_errors: Vec<String> = exploit_errors
            .difference(&baseline_errors)
            .cloned()
            .collect();
        if !new_errors.is_empty() {
            signals.push(Signal {
                r#type: signal::SignalType::ErrorLeak as i32,
                evidence: format!("new error patterns: {}", new_errors.join(", ")),
                confidence: 0.7,
            });
        }

        // 4. Content length / structural change
        let body_diff = (exploit.body.len() as i64 - baseline.body.len() as i64).abs();
        if body_diff > 0 {
            let change_ratio = if baseline.body.is_empty() {
                1.0
            } else {
                body_diff as f64 / baseline.body.len() as f64
            };
            if change_ratio > 0.2 {
                signals.push(Signal {
                    r#type: signal::SignalType::ContentChange as i32,
                    evidence: format!(
                        "body size changed: {} bytes (delta={}, ratio={:.2})",
                        exploit.body.len(),
                        body_diff,
                        change_ratio
                    ),
                    confidence: 0.5,
                });
            }
        }

        // 5. Header change detection
        let baseline_headers: HashSet<String> = baseline
            .headers
            .keys()
            .map(|k| k.to_ascii_lowercase())
            .collect();
        let exploit_headers: HashSet<String> = exploit
            .headers
            .keys()
            .map(|k| k.to_ascii_lowercase())
            .collect();
        let new_headers: Vec<String> = exploit_headers
            .difference(&baseline_headers)
            .cloned()
            .collect();
        let missing_headers: Vec<String> = baseline_headers
            .difference(&exploit_headers)
            .cloned()
            .collect();
        if !new_headers.is_empty() || !missing_headers.is_empty() {
            let mut evidence = Vec::new();
            if !new_headers.is_empty() {
                evidence.push(format!("new headers: {:?}", new_headers));
            }
            if !missing_headers.is_empty() {
                evidence.push(format!("missing headers: {:?}", missing_headers));
            }
            signals.push(Signal {
                r#type: signal::SignalType::HeaderChange as i32,
                evidence: evidence.join("; "),
                confidence: 0.4,
            });
        }

        let significant = !signals.is_empty();
        let diff_bytes = body_diff;

        debug!(
            "diff: significant={} signals={} diff_bytes={}",
            significant,
            signals.len(),
            diff_bytes
        );

        DiffResponse {
            significant,
            signals,
            diff_bytes,
        }
    }

    /// Classify the overall verification status based on signals.
    pub fn classify(
        &self,
        replay_success: bool,
        diff: &DiffResponse,
        causal_valid: bool,
    ) -> VerificationStatus {
        match (replay_success, diff.significant, causal_valid) {
            (true, true, true) => VerificationStatus::Confirmed,
            (true, true, false) => VerificationStatus::Inconclusive,
            (true, false, _) => VerificationStatus::FalsePositive,
            (false, _, _) => VerificationStatus::Unreproducible,
        }
    }
}

/// Extract error-like patterns from response body text.
fn extract_error_patterns(body: &[u8]) -> HashSet<String> {
    let text = String::from_utf8_lossy(body);
    let patterns = [
        "sql", "syntax", "error", "exception", "warning", "fatal",
        "unexpected", "invalid", "failed", "cannot", "denied",
        "traceback", "stack", "nullpointer", "undefined",
        "Fatal error", "Warning", "Parse error", "exception",
    ];
    let mut found = HashSet::new();
    let lower = text.to_ascii_lowercase();
    for p in &patterns {
        if lower.contains(p) {
            found.insert(p.to_string());
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::HttpResponse as HttpResp;
    use std::collections::HashMap;

    fn make_response(status: i32, body: &str, ms: i32) -> HttpResponse {
        HttpResp {
            status_code: status,
            headers: HashMap::new(),
            body: body.as_bytes().to_vec(),
            duration_ms: ms,
        }
    }

    fn make_detailed_response(
        status: i32,
        body: &str,
        ms: i32,
        headers: Vec<(&str, &str)>,
    ) -> HttpResponse {
        let mut h = HashMap::new();
        for (k, v) in headers {
            h.insert(k.to_string(), v.to_string());
        }
        HttpResp {
            status_code: status,
            headers: h,
            body: body.as_bytes().to_vec(),
            duration_ms: ms,
        }
    }

    // ── Time delay detection ──────────────────────────────────────

    #[test]
    fn test_detects_time_delay() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(200, "ok", 5000);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert!(diff.significant, "should detect time delay");
        assert!(diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::TimeDelay as i32));
    }

    #[test]
    fn test_no_false_time_delay() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(200, "ok", 200);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        let has_time = diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::TimeDelay as i32);
        assert!(!has_time, "should not detect time delay for 100ms diff");
    }

    // ── Error leak detection ──────────────────────────────────────

    #[test]
    fn test_detects_new_errors() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(500, "Internal Server Error: SQL syntax error", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert!(diff.significant, "should detect error leak");
        assert!(diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::ErrorLeak as i32));
    }

    #[test]
    fn test_no_error_leak_when_same() {
        let baseline = make_response(500, "Internal Server Error", 100);
        let exploit = make_response(500, "Internal Server Error", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        let has_error = diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::ErrorLeak as i32);
        assert!(!has_error, "same errors should not be flagged");
    }

    // ── Status code diff ──────────────────────────────────────────

    #[test]
    fn test_detects_status_change() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(403, "forbidden", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert!(diff.significant, "should detect status change");
        assert!(diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::StatusChange as i32));
    }

    #[test]
    fn test_same_status_no_signal() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(200, "different body", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        let has_status = diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::StatusChange as i32);
        assert!(!has_status, "same status should not fire status signal");
    }

    // ── Edge cases ────────────────────────────────────────────────

    #[test]
    fn test_identical_responses() {
        let baseline = make_response(200, "ok", 100);
        let exploit = make_response(200, "ok", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert!(!diff.significant, "identical responses should not be significant");
        assert_eq!(diff.signals.len(), 0);
    }

    #[test]
    fn test_empty_bodies() {
        let baseline = make_response(200, "", 0);
        let exploit = make_response(200, "", 0);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert!(!diff.significant, "empty same bodies should not fire");
    }

    #[test]
    fn test_huge_bodies() {
        let big = "A".repeat(1_000_000);
        let bigger = "A".repeat(1_000_100);
        let baseline = make_response(200, &big, 100);
        let exploit = make_response(200, &bigger, 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        let has_content = diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::ContentChange as i32);
        assert!(!has_content, "tiny relative change should not fire");
    }

    #[test]
    fn test_non_utf8_bodies() {
        let baseline = make_response(200, "\u{fffd}\u{fffd}\0\u{1}", 100);
        let exploit = make_response(200, "\u{fffd}\u{fffd}\0\u{2}", 100);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        assert_eq!(diff.diff_bytes, 0);
    }

    #[test]
    fn test_new_header_detected() {
        let baseline = make_detailed_response(200, "ok", 100, vec![]);
        let exploit = make_detailed_response(200, "ok", 100, vec![("X-Debug", "true")]);
        let diff = DiffOracle {}.analyze(&baseline, &exploit);
        let has_header = diff
            .signals
            .iter()
            .any(|s| s.r#type == signal::SignalType::HeaderChange as i32);
        assert!(has_header, "new header should be detected");
    }

    // ── Classification ────────────────────────────────────────────

    #[test]
    fn test_classify_confirmed() {
        let oracle = DiffOracle {};
        let diff = DiffResponse {
            significant: true,
            signals: vec![],
            diff_bytes: 100,
        };
        let status = oracle.classify(true, &diff, true);
        assert_eq!(status, VerificationStatus::Confirmed);
    }

    #[test]
    fn test_classify_false_positive() {
        let oracle = DiffOracle {};
        let diff = DiffResponse {
            significant: false,
            signals: vec![],
            diff_bytes: 0,
        };
        let status = oracle.classify(true, &diff, true);
        assert_eq!(status, VerificationStatus::FalsePositive);
    }

    #[test]
    fn test_classify_unreproducible() {
        let oracle = DiffOracle {};
        let diff = DiffResponse {
            significant: false,
            signals: vec![],
            diff_bytes: 0,
        };
        let status = oracle.classify(false, &diff, false);
        assert_eq!(status, VerificationStatus::Unreproducible);
    }
}
