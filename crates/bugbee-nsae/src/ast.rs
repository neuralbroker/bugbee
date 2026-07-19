//! Stage 1: AST / structural extractor (lightweight, zero-copy-ish line analysis).
//! Full tree-sitter multi-lang is Horizon B; this ships high-signal heuristics now.

use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSlice {
    pub path: String,
    pub language: String,
    pub taint_sources: Vec<Signal>,
    pub taint_sinks: Vec<Signal>,
    pub guards: Vec<Signal>,
    pub dangerous_apis: Vec<Signal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub line: u32,
    pub kind: String,
    pub text: String,
}

pub fn detect_language(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "py" => "python",
        "js" | "jsx" | "mjs" => "javascript",
        "ts" | "tsx" => "typescript",
        "go" => "go",
        "rs" => "rust",
        "java" => "java",
        "php" => "php",
        _ => "unknown",
    }
}

/// Extract structural security signals from file content.
pub fn extract_structural_signals(path: &Path, content: &str) -> StructuralSlice {
    let lang = detect_language(path);
    let mut slice = StructuralSlice {
        path: path.to_string_lossy().into_owned(),
        language: lang.into(),
        taint_sources: Vec::new(),
        taint_sinks: Vec::new(),
        guards: Vec::new(),
        dangerous_apis: Vec::new(),
    };

    let source_re = Regex::new(
        r"(?i)(request\.(args|form|json|get_json|query)|req\.(query|body|params)|os\.environ|getenv|input\s*\(|readline|params\[|@RequestParam|HttpServletRequest)",
    )
    .expect("re");
    let sink_re = Regex::new(
        r"(?i)(\beval\s*\(|\bexec\s*\(|subprocess\.|shell\s*=\s*True|os\.system|Runtime\.exec|mysqli_query|->query\s*\(|innerHTML|document\.write|pickle\.loads|yaml\.load\s*\(|child_process)",
    )
    .expect("re");
    let guard_re = Regex::new(
        r"(?i)(escape|sanitize|parametrize|prepared|allowlist|whitelist|htmlspecialchars|DOMPurify|validator\.|zod\.|pydantic)",
    )
    .expect("re");
    let danger_re = Regex::new(
        r"(?i)(md5|sha1\b|DES\b|ECB|verify\s*=\s*False|insecure|TODO\s*security|hardcoded)",
    )
    .expect("re");

    for (i, line) in content.lines().enumerate() {
        let line_no = (i + 1) as u32;
        if source_re.is_match(line) {
            slice.taint_sources.push(Signal {
                line: line_no,
                kind: "source".into(),
                text: trim(line),
            });
        }
        if sink_re.is_match(line) {
            slice.taint_sinks.push(Signal {
                line: line_no,
                kind: "sink".into(),
                text: trim(line),
            });
        }
        if guard_re.is_match(line) {
            slice.guards.push(Signal {
                line: line_no,
                kind: "guard".into(),
                text: trim(line),
            });
        }
        if danger_re.is_match(line) {
            slice.dangerous_apis.push(Signal {
                line: line_no,
                kind: "dangerous".into(),
                text: trim(line),
            });
        }
    }
    slice
}

fn trim(s: &str) -> String {
    let t = s.trim();
    if t.len() > 160 {
        format!("{}…", &t[..160])
    } else {
        t.into()
    }
}

/// Heuristic: finding line is near a sink with a source earlier in file → strong symbolic.
pub fn symbolic_strength_for_line(
    slice: &StructuralSlice,
    line: u32,
) -> bugbee_core::SymbolicVerdict {
    let near_sink = slice
        .taint_sinks
        .iter()
        .any(|s| s.line.abs_diff(line) <= 2 || s.line == line);

    if near_sink {
        // Source-before + no guard would strengthen confidence,
        // but sink-on-line alone is sufficient for Strong.
        bugbee_core::SymbolicVerdict::Strong
    } else if slice
        .dangerous_apis
        .iter()
        .any(|d| d.line.abs_diff(line) <= 1)
    {
        bugbee_core::SymbolicVerdict::Noisy
    } else if slice.taint_sinks.is_empty() && slice.dangerous_apis.is_empty() {
        bugbee_core::SymbolicVerdict::None
    } else {
        bugbee_core::SymbolicVerdict::Noisy
    }
}
