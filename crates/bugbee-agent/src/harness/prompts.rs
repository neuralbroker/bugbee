use bugbee_core::Finding;

pub fn project_brief(name: &str, root: &str, finding_count: usize) -> String {
    format!(
        "Project: {name}\nRoot: {root}\nFindings in store: {finding_count}\n\
         Defense-only. Prefer tools over speculation. Call hunt if store is empty."
    )
}

pub fn findings_digest(findings: &[Finding], limit: usize) -> String {
    let mut s = String::from("Current findings:\n");
    for f in findings.iter().take(limit) {
        s.push_str(&format!(
            "- id={} [{}] status={} brs={} ecs={} {}:{} — {}\n",
            f.id,
            f.severity.as_str(),
            f.status.as_str(),
            f.brs,
            f.ecs,
            f.location.path,
            f.location.start_line,
            f.title
        ));
    }
    if findings.len() > limit {
        s.push_str(&format!("… {} more\n", findings.len() - limit));
    }
    s
}

pub fn godmode_user_goal() -> &'static str {
    "Run a maximum-efficiency security hunt:\n\
     1) hunt\n\
     2) prioritize critical/high\n\
     3) read/grep for evidence; add_evidence\n\
     4) adversarial pass: review_finding fp or confirm\n\
     5) final summary of confirmed issues with file:line\n\
     Be ruthless about false positives. Stay offline-tools first."
}
