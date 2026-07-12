//! Lightweight taint heuristics: sources and sinks co-located in the same scope.

use bugbee_core::{
    BrsWeights, Evidence, Finding, FindingLocation, LocationRole, Redactor, Severity,
};
use bugbee_index::{Lang, RepoIndex};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
struct CodeScope {
    start_line: u32,
    end_line: u32,
}

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

static PAIRS: Lazy<Vec<TaintPair>> = Lazy::new(|| {
    vec![
        TaintPair {
            id: "py-sql-injection",
            title: "Possible SQL injection (user input → query)",
            source: Regex::new(
                r"request\.(args|form|json|get_json|values)|input\(|os\.environ|sys\.argv",
            )
            .expect("valid taint regex"),
            sink: Regex::new(
                r#"(execute|executemany|raw)\s*\(|cursor\.execute|f["'].*SELECT|["'].*SELECT.*\+|query\s*=\s*f["']"#,
            )
            .expect("valid taint regex"),
            languages: &["python"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "py-command-injection",
            title: "Possible command injection",
            source: Regex::new(r"request\.(args|form|json)|input\(|sys\.argv")
                .expect("valid taint regex"),
            sink: Regex::new(r"os\.system|subprocess\.(call|run|Popen)|os\.popen")
                .expect("valid taint regex"),
            languages: &["python"],
            cwe: "CWE-78",
            owasp: "A05:2025",
            severity: Severity::Critical,
        },
        TaintPair {
            id: "js-sql-injection",
            title: "Possible SQL injection (JS/TS)",
            source: Regex::new(r"req\.(query|body|params)|request\.(query|body)")
                .expect("valid taint regex"),
            sink: Regex::new(r"query\s*\(|\.query\s*\(|execute\s*\(.*\+|`.*SELECT.*\$\{")
                .expect("valid taint regex"),
            languages: &["javascript", "typescript"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "js-xss",
            title: "Possible XSS sink",
            source: Regex::new(r"req\.(query|body|params)").expect("valid taint regex"),
            sink: Regex::new(r"innerHTML\s*=|dangerouslySetInnerHTML|document\.write")
                .expect("valid taint regex"),
            languages: &["javascript", "typescript"],
            cwe: "CWE-79",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "go-sql-injection",
            title: "Possible SQL injection (Go string concat/format)",
            source: Regex::new(r"r\.(URL\.Query|FormValue|PathValue)|os\.Args")
                .expect("valid taint regex"),
            sink: Regex::new(r"Query\s*\(.*\+|Exec\s*\(.*\+|fmt\.Sprintf\s*\(.*SELECT")
                .expect("valid taint regex"),
            languages: &["go"],
            cwe: "CWE-89",
            owasp: "A05:2025",
            severity: Severity::High,
        },
        TaintPair {
            id: "go-command-injection",
            title: "Possible command injection (Go)",
            source: Regex::new(r"r\.(URL\.Query|FormValue)|os\.Args").expect("valid taint regex"),
            sink: Regex::new(r"exec\.Command\(|syscall\.Exec").expect("valid taint regex"),
            languages: &["go"],
            cwe: "CWE-78",
            owasp: "A05:2025",
            severity: Severity::Critical,
        },
    ]
});

pub fn scan_taint_heuristics(
    index: &RepoIndex,
    weights: &BrsWeights,
) -> anyhow::Result<Vec<Finding>> {
    let redactor = Redactor::enterprise();
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

        for pair in PAIRS.iter() {
            if !pair.languages.contains(&lang) {
                continue;
            }
            for scope in scopes_for(file.lang, &content) {
                let mut sources = Vec::new();
                let mut sinks = Vec::new();
                for (line_index, line) in content.lines().enumerate() {
                    let line_number = (line_index + 1) as u32;
                    if line_number < scope.start_line || line_number > scope.end_line {
                        continue;
                    }
                    if pair.source.is_match(line) {
                        sources.push((line_number, redactor.redact(line.trim())));
                    }
                    if pair.sink.is_match(line) {
                        sinks.push((line_number, redactor.redact(line.trim())));
                    }
                }
                if sources.is_empty() || sinks.is_empty() {
                    continue;
                }

                let mut f = Finding::new(
                    pair.title,
                    format!(
                        "Scope-limited taint heuristic: {} source(s) and {} sink(s) in {}:{}-{}",
                        sources.len(),
                        sinks.len(),
                        file.path,
                        scope.start_line,
                        scope.end_line
                    ),
                    pair.severity,
                    "taint",
                );
                f.cwe = vec![pair.cwe.into()];
                f.owasp = vec![pair.owasp.into()];
                f.confidence = 0.6;
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
                        .map(|(line, snippet)| format!("source {}:{line}: {snippet}", file.path))
                        .chain(
                            sinks.iter().map(|(line, snippet)| {
                                format!("sink {}:{line}: {snippet}", file.path)
                            }),
                        )
                        .collect(),
                    dataflow: Some(format!(
                        "scope-level heuristic path in {}:{}-{} (not full IFDS)",
                        file.path, scope.start_line, scope.end_line
                    )),
                    agent_notes: Some(
                        "Promote with agent review + human confirmation before auto-confirm".into(),
                    ),
                };
                if let Some((line, snippet)) = sources.first() {
                    f.add_location(FindingLocation {
                        file: file.path.clone(),
                        start_line: *line,
                        end_line: *line,
                        start_col: None,
                        end_col: None,
                        role: LocationRole::Source,
                        snippet: Some(snippet.clone()),
                    });
                }
                if let Some((line, snippet)) = sinks.first() {
                    f.add_location(FindingLocation {
                        file: file.path.clone(),
                        start_line: *line,
                        end_line: *line,
                        start_col: None,
                        end_col: None,
                        role: LocationRole::Sink,
                        snippet: Some(snippet.clone()),
                    });
                }
                f.recompute_scores(weights);
                out.push(f);
            }
        }
    }
    Ok(out)
}

fn scopes_for(lang: Lang, content: &str) -> Vec<CodeScope> {
    let declaration = match lang {
        Lang::Python => Some(Regex::new(r"^(?:async\s+)?def\s+\w+\s*\(").expect("valid regex")),
        Lang::Go => Some(Regex::new(r"^func\s+").expect("valid regex")),
        Lang::JavaScript | Lang::TypeScript => Some(
            Regex::new(
                r"^(?:\s*app\.(?:get|post|put|patch|delete|use)\s*\(|\s*(?:export\s+)?(?:async\s+)?function\s+\w+|\s*(?:export\s+)?(?:const|let)\s+\w+\s*=\s*(?:async\s*)?(?:\([^)]*\)|\w+)\s*=>)",
            )
            .expect("valid regex"),
        ),
        Lang::Other => None,
    };
    let Some(declaration) = declaration else {
        return vec![CodeScope {
            start_line: 1,
            end_line: content.lines().count() as u32,
        }];
    };

    let starts = content
        .lines()
        .enumerate()
        .filter_map(|(index, line)| declaration.is_match(line).then_some((index + 1) as u32))
        .collect::<Vec<_>>();
    if starts.is_empty() {
        return vec![CodeScope {
            start_line: 1,
            end_line: content.lines().count() as u32,
        }];
    }

    let total_lines = content.lines().count() as u32;
    starts
        .iter()
        .enumerate()
        .map(|(index, start_line)| CodeScope {
            start_line: *start_line,
            end_line: starts
                .get(index + 1)
                .map_or(total_lines, |next| next.saturating_sub(1)),
        })
        .collect()
}
