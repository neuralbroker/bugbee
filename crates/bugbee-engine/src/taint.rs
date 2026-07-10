//! Lightweight taint heuristics: sources and sinks co-located or same function body.

use bugbee_core::{BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Severity};
use bugbee_index::{Lang, RepoIndex};
use regex::Regex;

struct TaintPair {
    id: &'static str,
    title: &'static str,
    source: Regex,
    sink: Regex,
    languages: &'static [&'static str],
    cwe: &'static str,
    owasp: &'static str,
    severity: Severity,
}

pub fn scan_taint_heuristics(
    index: &RepoIndex,
    weights: &BrsWeights,
) -> anyhow::Result<Vec<Finding>> {
    let pairs = vec![
        TaintPair {
            id: "py-sql-injection",
            title: "Possible SQL injection (user input → query)",
            source: Regex::new(
                r"request\.(args|form|json|get_json|values)|input\(|os\.environ|sys\.argv",
            )
            .unwrap(),
            sink: Regex::new(
                r#"(execute|executemany|raw)\s*\(|cursor\.execute|f["'].*SELECT|["'].*SELECT.*\+|query\s*=\s*f["']"#,
            )
            .unwrap(),
            languages: &["python"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "py-command-injection",
            title: "Possible command injection",
            source: Regex::new(r"request\.(args|form|json)|input\(|sys\.argv").unwrap(),
            sink: Regex::new(r"os\.system|subprocess\.(call|run|Popen)|os\.popen").unwrap(),
            languages: &["python"],
            cwe: "CWE-78",
            owasp: "A05:2025",
            severity: Severity::Critical,
        },
        TaintPair {
            id: "js-sql-injection",
            title: "Possible SQL injection (JS/TS)",
            source: Regex::new(r"req\.(query|body|params)|request\.(query|body)").unwrap(),
            sink: Regex::new(r"query\s*\(|\.query\s*\(|execute\s*\(.*\+|`.*SELECT.*\$\{").unwrap(),
            languages: &["javascript", "typescript"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "js-xss",
            title: "Possible XSS sink",
            source: Regex::new(r"req\.(query|body|params)").unwrap(),
            sink: Regex::new(r"innerHTML\s*=|dangerouslySetInnerHTML|document\.write").unwrap(),
            languages: &["javascript", "typescript"],
            cwe: "CWE-79",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "go-sql-injection",
            title: "Possible SQL injection (Go string concat/format)",
            source: Regex::new(r"r\.(URL\.Query|FormValue|PathValue)|os\.Args").unwrap(),
            sink: Regex::new(r"Query\s*\(.*\+|Exec\s*\(.*\+|fmt\.Sprintf\s*\(.*SELECT").unwrap(),
            languages: &["go"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "go-command-injection",
            title: "Possible command injection (Go)",
            source: Regex::new(r"r\.(URL\.Query|FormValue)|os\.Args").unwrap(),
            sink: Regex::new(r"exec\.Command\(|syscall\.Exec").unwrap(),
            languages: &["go"],
            cwe: "CWE-78",
            owasp: "A05:2025",
            severity: Severity::Critical,
        },
    ];

    let mut out = Vec::new();
    for file in &index.files {
        if file.lang == Lang::Other {
            continue;
        }
        let lang = file.lang.as_str();
        let content = match index.read_file(&file.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for pair in &pairs {
            if !pair.languages.contains(&lang) {
                continue;
            }
            let mut sources = Vec::new();
            let mut sinks = Vec::new();
            for (i, line) in content.lines().enumerate() {
                let ln = (i + 1) as u32;
                if pair.source.is_match(line) {
                    sources.push((ln, line.trim().to_string()));
                }
                if pair.sink.is_match(line) {
                    sinks.push((ln, line.trim().to_string()));
                }
            }
            if sources.is_empty() || sinks.is_empty() {
                continue;
            }
            // Same-file heuristic: both present → candidate taint flow
            let mut f = Finding::new(
                pair.title,
                format!(
                    "Heuristic taint: {} source(s) and {} sink(s) in {}",
                    sources.len(),
                    sinks.len(),
                    file.path
                ),
                pair.severity,
                "taint",
            );
            f.cwe = vec![pair.cwe.into()];
            f.owasp = vec![pair.owasp.into()];
            f.confidence = 0.55;
            f.exploitability = 0.6;
            f.blast_radius = 0.5;
            f.evidence = Evidence {
                rule_id: Some(format!("taint.{}", pair.id)),
                has_source: true,
                has_sink: true,
                has_path: true,
                has_repro: false,
                missing_sanitizer_check: true,
                traces: sources
                    .iter()
                    .map(|(l, s)| format!("source {}:{}: {s}", file.path, l))
                    .chain(
                        sinks
                            .iter()
                            .map(|(l, s)| format!("sink {}:{}: {s}", file.path, l)),
                    )
                    .collect(),
                dataflow: Some(format!(
                    "file-level heuristic path in {} (not full IFDS)",
                    file.path
                )),
                agent_notes: Some(
                    "Promote with agent review + human confirmation before auto-confirm".into(),
                ),
            };
            if let Some((ln, snip)) = sources.first() {
                f.add_location(FindingLocation {
                    file: file.path.clone(),
                    start_line: *ln,
                    end_line: *ln,
                    start_col: None,
                    end_col: None,
                    role: LocationRole::Source,
                    snippet: Some(snip.clone()),
                });
            }
            if let Some((ln, snip)) = sinks.first() {
                f.add_location(FindingLocation {
                    file: file.path.clone(),
                    start_line: *ln,
                    end_line: *ln,
                    start_col: None,
                    end_col: None,
                    role: LocationRole::Sink,
                    snippet: Some(snip.clone()),
                });
            }
            f.recompute_scores(weights);
            out.push(f);
        }
    }
    Ok(out)
}
