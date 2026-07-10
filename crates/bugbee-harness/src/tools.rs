use std::fs;
use std::path::Path;
use std::process::Command;

use bugbee_core::{BugbeeError, Redactor, Result};
use bugbee_index::RepoIndex;
use serde_json::json;

use crate::permissions::{Action, PermissionDecision, PermissionPolicy};

pub struct ToolContext<'a> {
    pub index: &'a RepoIndex,
    pub policy: &'a PermissionPolicy,
    pub auto_approve: bool,
}

pub fn tool_defs() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "read_file",
            "description": "Read a source file from the project (sensitive paths blocked)",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "grep",
            "description": "Search code for a regex pattern",
            "parameters": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "glob": {"type": "string"}
                },
                "required": ["pattern"]
            }
        }),
        json!({
            "name": "list_files",
            "description": "List indexed source files",
            "parameters": {
                "type": "object",
                "properties": {
                    "lang": {"type": "string"}
                }
            }
        }),
        json!({
            "name": "bash",
            "description": "Run a limited safe shell command (tests/git/search only)",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }
        }),
        json!({
            "name": "repomap",
            "description": "Get ranked important files for context",
            "parameters": {
                "type": "object",
                "properties": {
                    "limit": {"type": "integer"}
                }
            }
        }),
    ]
}

pub fn execute_tool(ctx: &ToolContext, name: &str, args: &serde_json::Value) -> Result<String> {
    match name {
        "read_file" => {
            ensure(ctx, Action::Read)?;
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BugbeeError::Other("path required".into()))?;
            if Redactor::is_sensitive_path(path) {
                return Err(BugbeeError::Other(format!(
                    "blocked sensitive path: {path}"
                )));
            }
            let content = ctx.index.read_file(path)?;
            let redacted = Redactor::enterprise().redact(&content);
            // Cap size for context
            Ok(truncate(&redacted, 80_000))
        }
        "grep" => {
            ensure(ctx, Action::Read)?;
            let pattern = args
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BugbeeError::Other("pattern required".into()))?;
            let re = regex::Regex::new(pattern)
                .map_err(|e| BugbeeError::Other(format!("bad regex: {e}")))?;
            let mut hits = Vec::new();
            for f in &ctx.index.files {
                if let Ok(content) = ctx.index.read_file(&f.path) {
                    for (i, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            let snippet = Redactor::enterprise().redact(line.trim());
                            hits.push(format!("{}:{}: {snippet}", f.path, i + 1));
                            if hits.len() >= 50 {
                                break;
                            }
                        }
                    }
                }
                if hits.len() >= 50 {
                    break;
                }
            }
            Ok(if hits.is_empty() {
                "no matches".into()
            } else {
                hits.join("\n")
            })
        }
        "list_files" => {
            ensure(ctx, Action::Read)?;
            let lang = args.get("lang").and_then(|v| v.as_str());
            let list: Vec<_> = ctx
                .index
                .files
                .iter()
                .filter(|f| lang.map(|l| f.lang.as_str() == l).unwrap_or(true))
                .map(|f| format!("{} ({}, {} lines)", f.path, f.lang.as_str(), f.lines))
                .collect();
            Ok(list.join("\n"))
        }
        "repomap" => {
            ensure(ctx, Action::Read)?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
            let map = ctx.index.repomap(limit);
            let lines: Vec<_> = map
                .iter()
                .map(|f| {
                    format!(
                        "{} — {} symbols, {} imports",
                        f.path,
                        f.symbols.len(),
                        f.imports.len()
                    )
                })
                .collect();
            Ok(lines.join("\n"))
        }
        "bash" => {
            ensure(ctx, Action::Bash)?;
            let cmd = args
                .get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BugbeeError::Other("command required".into()))?;
            if !PermissionPolicy::bash_allowed_prefix(cmd) {
                return Err(BugbeeError::Other(format!(
                    "bash command denied by policy (not in safe allowlist): {cmd}"
                )));
            }
            let root = Path::new(&ctx.index.root);
            let output = Command::new("bash")
                .arg("-c")
                .arg(cmd)
                .current_dir(root)
                .output()
                .map_err(|e| BugbeeError::Other(e.to_string()))?;
            let mut text = String::from_utf8_lossy(&output.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            Ok(truncate(&text, 40_000))
        }
        _ => Err(BugbeeError::Other(format!("unknown tool: {name}"))),
    }
}

fn ensure(ctx: &ToolContext, action: Action) -> Result<()> {
    match ctx.policy.check(action) {
        PermissionDecision::Allow => Ok(()),
        PermissionDecision::Deny => Err(BugbeeError::Other(format!(
            "permission denied for {action:?}"
        ))),
        PermissionDecision::Ask => {
            if ctx.auto_approve {
                Ok(())
            } else {
                Err(BugbeeError::Other(format!(
                    "permission ask required for {action:?} (use --auto to approve safe ops)"
                )))
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let boundary = s
            .char_indices()
            .take_while(|(index, _)| *index <= max)
            .map(|(index, _)| index)
            .last()
            .unwrap_or(0);
        format!(
            "{}…\n[truncated {} bytes]",
            &s[..boundary],
            s.len() - boundary
        )
    }
}

/// Write a patch proposal file for human review (never auto-apply in hunt).
pub fn write_patch_proposal(
    root: &Path,
    finding_id: &str,
    diff: &str,
) -> Result<std::path::PathBuf> {
    if finding_id.is_empty()
        || !finding_id
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
    {
        return Err(BugbeeError::Other(
            "patch proposal id must be a UUID-like identifier".into(),
        ));
    }
    let dir = root.join(".bugbee").join("patches");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{finding_id}.diff"));
    fs::write(&path, diff)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{execute_tool, truncate, write_patch_proposal, ToolContext};
    use crate::permissions::PermissionPolicy;
    use bugbee_index::Indexer;
    use serde_json::json;

    #[test]
    fn truncation_preserves_utf8_boundaries() {
        let output = truncate("αβγ", 3);
        assert!(output.starts_with('α'));
        assert!(output.contains("truncated"));
    }

    #[test]
    fn patch_proposal_rejects_path_traversal() {
        let root = std::env::temp_dir().join("bugbee-patch-test");
        assert!(write_patch_proposal(&root, "../../escape", "diff").is_err());
    }

    #[test]
    fn grep_redacts_secret_like_matches_before_returning_them() {
        let root = std::env::temp_dir().join(format!("bugbee-grep-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let token = "github_pat_abcdefghijklmnopqrstuvwxyz_1234567890";
        std::fs::write(root.join("app.py"), format!("token = '{token}'\n")).unwrap();
        let index = Indexer::new(&root).build().unwrap();
        let policy = PermissionPolicy::hunt_default();
        let context = ToolContext {
            index: &index,
            policy: &policy,
            auto_approve: false,
        };

        let output = execute_tool(&context, "grep", &json!({"pattern": "github_pat"}))
            .expect("grep succeeds");
        assert!(!output.contains(token));
        assert!(output.contains("REDACTED"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
