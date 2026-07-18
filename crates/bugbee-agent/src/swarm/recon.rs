//! ReconAgent — maps attack surface (auth-aware stubs for code repos).

use std::collections::HashMap;
use std::path::Path;

use bugbee_core::{AuthMechanism, Target, TargetKind};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconReport {
    pub files_indexed: usize,
    pub by_language: HashMap<String, usize>,
    pub auth_hints: Vec<String>,
    pub api_specs: Vec<String>,
    pub targets: Vec<Target>,
}

/// Map repository attack surface (source-centric recon for MVP).
pub fn recon_repo(root: &Path) -> ReconReport {
    let mut by_language: HashMap<String, usize> = HashMap::new();
    let mut auth_hints = Vec::new();
    let mut api_specs = Vec::new();
    let mut targets = vec![Target::repo(root.display().to_string())];
    let mut files_indexed = 0usize;

    let walker = WalkBuilder::new(root)
        .git_ignore(true)
        .hidden(false)
        .build();

    for entry in walker.flatten() {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        // API specs
        if matches!(
            name,
            "openapi.yaml" | "openapi.yml" | "openapi.json" | "swagger.json" | "swagger.yaml"
        ) || rel.contains("openapi")
            || rel.contains("swagger")
        {
            api_specs.push(rel.clone());
        }
        if name.ends_with(".graphql") || name == "schema.graphql" {
            api_specs.push(rel.clone());
        }

        let lang = match ext.as_str() {
            "py" => "python",
            "js" | "jsx" | "mjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            "rs" => "rust",
            "java" => "java",
            "php" => "php",
            "rb" => "ruby",
            _ => continue,
        };
        files_indexed += 1;
        *by_language.entry(lang.into()).or_default() += 1;
        targets.push(Target::source_file(&rel));

        // Cheap auth-flow hints from path names
        let low = rel.to_ascii_lowercase();
        if low.contains("auth") || low.contains("login") || low.contains("oauth") {
            auth_hints.push(format!("auth_surface:{rel}"));
        }
        if low.contains("jwt") {
            auth_hints.push(format!("jwt_hint:{rel}"));
        }
        if low.contains("session") {
            auth_hints.push(format!("session_hint:{rel}"));
        }
    }

    // Dedupe auth hints
    auth_hints.sort();
    auth_hints.dedup();

    // Annotate primary target
    if let Some(t) = targets.first_mut() {
        if auth_hints.iter().any(|h| h.contains("jwt")) {
            t.auth = AuthMechanism::Jwt;
        } else if auth_hints.iter().any(|h| h.contains("oauth")) {
            t.auth = AuthMechanism::OAuth;
        } else if auth_hints.iter().any(|h| h.contains("session")) {
            t.auth = AuthMechanism::Cookie;
        }
        if let TargetKind::Repo { .. } = &t.kind {
            t.metadata = serde_json::json!({
                "files": files_indexed,
                "api_specs": api_specs.len(),
                "auth_hints": auth_hints.len(),
            });
        }
    }

    ReconReport {
        files_indexed,
        by_language,
        auth_hints,
        api_specs,
        targets,
    }
}
