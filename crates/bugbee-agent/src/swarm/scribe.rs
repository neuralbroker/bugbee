//! ScribeAgent — bounty-formatted reports (HackerOne / Bugcrowd compatible).

use bugbee_core::{Finding, Severity};

/// Render a single finding as a bounty-style markdown report.
pub fn render_bounty_report(finding: &Finding) -> String {
    let severity = match finding.severity {
        Severity::Critical => "Critical",
        Severity::High => "High",
        Severity::Medium => "Medium",
        Severity::Low => "Low",
        Severity::Info => "Informational",
    };

    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", finding.title));
    out.push_str(&format!("**Severity:** {severity}  \n"));
    out.push_str(&format!(
        "**CWE:** {}  \n",
        finding.cwe.as_deref().unwrap_or("n/a")
    ));
    out.push_str(&format!("**Rule:** `{}`  \n", finding.rule_id));
    out.push_str(&format!("**Finding ID:** `{}`  \n", finding.id));
    out.push_str(&format!(
        "**Adjudication:** {}  \n",
        finding.adjudicated_state.as_str()
    ));
    out.push_str(&format!(
        "**Verified:** {}  \n",
        if finding.verified { "yes" } else { "no" }
    ));
    out.push_str(&format!(
        "**BRS / ECS:** {} / {}  \n\n",
        finding.brs, finding.ecs
    ));

    out.push_str("## Description\n\n");
    out.push_str(&finding.message);
    out.push_str("\n\n");
    if let Some(ref reason) = finding.adjudication_reason {
        out.push_str(&format!("**NSAE:** {reason}\n\n"));
    }

    out.push_str("## Reproduction\n\n");
    out.push_str(&format!(
        "1. Open `{}` at line {}.\n",
        finding.location.path, finding.location.start_line
    ));
    if let Some(ref snip) = finding.location.snippet {
        out.push_str("2. Observe the following code:\n\n```\n");
        out.push_str(snip);
        out.push_str("\n```\n\n");
    }
    if let Some(ref poc) = finding.poc {
        out.push_str("### PoC steps (authorized / local only)\n\n");
        for (i, step) in poc.steps.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, step));
        }
        out.push('\n');
        if let Some(ref curl) = poc.curl_template {
            out.push_str("```bash\n");
            out.push_str(curl);
            out.push_str("\n```\n\n");
        }
    }

    out.push_str("## Impact\n\n");
    out.push_str(impact_blurb(finding));
    out.push_str("\n\n");

    out.push_str("## Fix\n\n");
    out.push_str(fix_blurb(finding));
    out.push_str("\n\n");

    out.push_str("## Evidence\n\n");
    for e in &finding.evidence {
        out.push_str(&format!(
            "- **{}:** {}\n",
            e.kind,
            e.detail.lines().next().unwrap_or("")
        ));
    }
    out.push('\n');
    out
}

pub fn render_bounty_reports(findings: &[Finding]) -> String {
    let mut parts = Vec::new();
    for f in findings {
        if f.adjudicated_state == bugbee_core::AdjudicationState::Safe {
            continue;
        }
        if f.status == bugbee_core::FindingStatus::FalsePositive {
            continue;
        }
        parts.push(render_bounty_report(f));
    }
    if parts.is_empty() {
        "# No reportable findings\n".into()
    } else {
        parts.join("\n---\n\n")
    }
}

fn impact_blurb(f: &Finding) -> &'static str {
    if f.rule_id.contains("eval") || f.rule_id.contains("shell") || f.rule_id.contains("exec") {
        "An attacker who can influence the tainted input may achieve remote code execution or command injection, leading to full host compromise in the worst case."
    } else if f.rule_id.contains("sql") {
        "SQL injection can lead to data exfiltration, authentication bypass, or data destruction depending on database privileges."
    } else if f.rule_id.contains("secret") || f.rule_id.contains("password") {
        "Hardcoded credentials can be extracted from source or builds and reused for unauthorized access."
    } else if f.rule_id.contains("ssrf") {
        "SSRF may allow access to internal services, cloud metadata, or network pivots."
    } else {
        "This issue may impact confidentiality, integrity, or availability depending on exploitability in the deployment context."
    }
}

fn fix_blurb(f: &Finding) -> &'static str {
    if f.rule_id.contains("eval") || f.rule_id.contains("exec") {
        "Remove dynamic code execution. Use safe parsers or explicit allowlisted operations."
    } else if f.rule_id.contains("shell") {
        "Avoid shell=True. Pass argument vectors to subprocess without a shell; validate inputs."
    } else if f.rule_id.contains("sql") {
        "Use parameterized queries / prepared statements. Never concatenate untrusted input into SQL."
    } else if f.rule_id.contains("secret") || f.rule_id.contains("password") {
        "Remove secrets from source. Load from a secret manager or environment; rotate exposed credentials."
    } else {
        "Apply least privilege, validate inputs, and add regression tests covering this sink."
    }
}
