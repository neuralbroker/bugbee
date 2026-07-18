use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use bugbee_core::{Evidence, FindingId, FindingStatus, Location, Result, Store};
use bugbee_engine::{hunt, HuntOptions};
use parking_lot::Mutex;
use regex::Regex;
use serde_json::{json, Value};
use tracing::debug;

use crate::permissions::{Permission, PermissionGate};
use crate::session::{Session, TodoItem};
use crate::tools::registry::ToolName;
use crate::tools::truncate::{truncate_output, DEFAULT_MAX};
use crate::tools::ToolResult;

/// Shared mutable context for tool execution (OpenCode Tool.Context analogue).
pub struct ToolContext {
    pub root: PathBuf,
    pub gate: PermissionGate,
    pub store: Arc<Mutex<Store>>,
    pub session: Arc<Mutex<Session>>,
    pub hunt_config: bugbee_core::ProjectConfig,
}

pub struct ToolExecutor {
    pub ctx: ToolContext,
}

impl ToolExecutor {
    pub fn new(ctx: ToolContext) -> Self {
        Self { ctx }
    }

    pub fn execute(&self, name: &str, args: &Value) -> ToolResult {
        let Some(tool) = ToolName::parse(name) else {
            return ToolResult::err(name, format!("unknown tool: {name}"));
        };
        let res = match tool {
            ToolName::Read => self.tool_read(args),
            ToolName::Grep => self.tool_grep(args),
            ToolName::Glob => self.tool_glob(args),
            ToolName::ListDir => self.tool_list_dir(args),
            ToolName::Hunt => self.tool_hunt(),
            ToolName::ListFindings => self.tool_list_findings(args),
            ToolName::GetFinding => self.tool_get_finding(args),
            ToolName::ReviewFinding => self.tool_review(args),
            ToolName::AddEvidence => self.tool_add_evidence(args),
            ToolName::TodoWrite => self.tool_todo(args),
            ToolName::Shell => self.tool_shell(args),
            ToolName::Edit => self.tool_edit(args),
        };
        match res {
            Ok(mut r) => {
                r.output = truncate_output(&r.output, DEFAULT_MAX);
                r
            }
            Err(e) => ToolResult::err(name, e.to_string()),
        }
    }

    fn tool_read(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Read)?;
        let path = str_arg(args, "path")
            .ok_or_else(|| bugbee_core::Error::Invalid("path required".into()))?;
        let path = self.ctx.gate.resolve_path(path)?;
        if self.ctx.gate.is_sensitive(&path) {
            return Ok(ToolResult::err(
                "read",
                format!("blocked sensitive path: {}", path.display()),
            ));
        }
        if path.is_dir() {
            return self.list_path(&path);
        }
        let content = fs::read_to_string(&path)?;
        let lines: Vec<&str> = content.lines().collect();
        let offset = int_arg(args, "offset").unwrap_or(1).max(1) as usize;
        let limit = int_arg(args, "limit").unwrap_or(200) as usize;
        let start = offset.saturating_sub(1).min(lines.len());
        let end = (start + limit).min(lines.len());
        let mut body = String::new();
        for (i, line) in lines[start..end].iter().enumerate() {
            let mut l = (*line).to_string();
            if l.len() > 2000 {
                l.truncate(2000);
                l.push('…');
            }
            body.push_str(&format!("{:6}|{}\n", start + i + 1, l));
        }
        let rel = self.ctx.gate.rel_display(&path);
        Ok(ToolResult::ok(
            &rel,
            format!(
                "# {rel} lines {}-{} of {}\n{body}",
                start + 1,
                end,
                lines.len()
            ),
        )
        .with_meta(json!({
            "path": rel,
            "total_lines": lines.len(),
            "truncated": end < lines.len()
        })))
    }

    fn list_path(&self, path: &Path) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::ListDir)?;
        let mut names = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let mark = if entry.file_type()?.is_dir() { "/" } else { "" };
            names.push(format!("{}{mark}", entry.file_name().to_string_lossy()));
        }
        names.sort();
        let rel = self.ctx.gate.rel_display(path);
        Ok(ToolResult::ok(
            &rel,
            format!("# dir {rel}\n{}", names.join("\n")),
        ))
    }

    fn tool_list_dir(&self, args: &Value) -> Result<ToolResult> {
        let path = str_arg(args, "path").unwrap_or(".");
        let path = self.ctx.gate.resolve_path(path)?;
        self.list_path(&path)
    }

    fn tool_grep(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Grep)?;
        let pattern = str_arg(args, "pattern")
            .ok_or_else(|| bugbee_core::Error::Invalid("pattern required".into()))?;
        let re = Regex::new(pattern)
            .map_err(|e| bugbee_core::Error::Invalid(format!("bad regex: {e}")))?;
        let search_root = match str_arg(args, "path") {
            Some(p) => self.ctx.gate.resolve_path(p)?,
            None => self.ctx.root.clone(),
        };
        let include = str_arg(args, "include");
        let max = int_arg(args, "max_matches").unwrap_or(80) as usize;

        let mut matches = Vec::new();
        let walker = ignore::WalkBuilder::new(&search_root)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.flatten() {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if let Some(inc) = include {
                if !glob_match(inc, path) {
                    continue;
                }
            }
            if self.ctx.gate.is_sensitive(path) {
                continue;
            }
            let Ok(content) = fs::read_to_string(path) else {
                continue;
            };
            if content.len() > 1_500_000 {
                continue;
            }
            for (idx, line) in content.lines().enumerate() {
                if re.is_match(line) {
                    let rel = path.strip_prefix(&self.ctx.root).unwrap_or(path).display();
                    let snip = if line.len() > 200 {
                        format!("{}…", &line[..200])
                    } else {
                        line.to_string()
                    };
                    matches.push(format!("{rel}:{}:{snip}", idx + 1));
                    if matches.len() >= max {
                        break;
                    }
                }
            }
            if matches.len() >= max {
                break;
            }
        }

        let n = matches.len();
        let body = if matches.is_empty() {
            "No matches".into()
        } else {
            matches.join("\n")
        };
        Ok(ToolResult::ok(pattern, body).with_meta(json!({ "matches": n, "truncated": n >= max })))
    }

    fn tool_glob(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Glob)?;
        let pattern = str_arg(args, "pattern")
            .ok_or_else(|| bugbee_core::Error::Invalid("pattern required".into()))?;
        let mut hits = Vec::new();
        let walker = ignore::WalkBuilder::new(&self.ctx.root)
            .hidden(false)
            .git_ignore(true)
            .build();
        for entry in walker.flatten() {
            if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let rel = path
                .strip_prefix(&self.ctx.root)
                .unwrap_or(path)
                .to_string_lossy();
            if glob_match(pattern, Path::new(rel.as_ref())) || glob_match(pattern, path) {
                hits.push(rel.into_owned());
                if hits.len() >= 200 {
                    break;
                }
            }
        }
        hits.sort();
        Ok(ToolResult::ok(pattern, hits.join("\n")).with_meta(json!({ "count": hits.len() })))
    }

    fn tool_hunt(&self) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Hunt)?;
        let opts = HuntOptions::from_config(&self.ctx.root, &self.ctx.hunt_config);
        let summary = hunt(&opts)?;
        self.ctx.store.lock().upsert_many(&summary.findings)?;
        self.ctx.session.lock().log(
            "hunt",
            format!(
                "{} files / {} findings / {} rules",
                summary.files_scanned,
                summary.findings.len(),
                summary.rules_loaded
            ),
        );
        let mut top = String::new();
        for f in summary.findings.iter().take(25) {
            top.push_str(&format!(
                "- [{}] brs={} ecs={} {}:{}  {}  id={}\n",
                f.severity.as_str(),
                f.brs,
                f.ecs,
                f.location.path,
                f.location.start_line,
                f.title,
                f.id
            ));
        }
        Ok(ToolResult::ok(
            "hunt",
            format!(
                "Hunt complete: {} files, {} findings, {} rules.\n{top}",
                summary.files_scanned,
                summary.findings.len(),
                summary.rules_loaded
            ),
        )
        .with_meta(json!({
            "files": summary.files_scanned,
            "findings": summary.findings.len(),
            "rules": summary.rules_loaded
        })))
    }

    fn tool_list_findings(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Findings)?;
        let status = str_arg(args, "status").and_then(FindingStatus::parse);
        let limit = int_arg(args, "limit").unwrap_or(40) as usize;
        let findings = self.ctx.store.lock().list(status)?;
        let mut out = String::new();
        for f in findings.iter().take(limit) {
            out.push_str(&format!(
                "{}\t{:<8}\t{:<12}\tbrs={}\tecs={}\t{}:{}  {}\n",
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
        if findings.is_empty() {
            out = "no findings (run hunt first)".into();
        }
        Ok(ToolResult::ok("list_findings", out).with_meta(json!({ "count": findings.len() })))
    }

    fn tool_get_finding(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Findings)?;
        let id =
            str_arg(args, "id").ok_or_else(|| bugbee_core::Error::Invalid("id required".into()))?;
        let f = self
            .ctx
            .store
            .lock()
            .get(&FindingId(id.to_string()))?
            .ok_or_else(|| bugbee_core::Error::NotFound(format!("finding {id}")))?;
        let text = serde_json::to_string_pretty(&f)?;
        Ok(ToolResult::ok(id, text))
    }

    fn tool_review(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Review)?;
        let id =
            str_arg(args, "id").ok_or_else(|| bugbee_core::Error::Invalid("id required".into()))?;
        let status = str_arg(args, "status")
            .and_then(FindingStatus::parse)
            .ok_or_else(|| bugbee_core::Error::Invalid("status required".into()))?;
        let f = self
            .ctx
            .store
            .lock()
            .set_status(&FindingId(id.to_string()), status)?;
        Ok(ToolResult::ok(
            id,
            format!("{} → {}", f.id, f.status.as_str()),
        ))
    }

    fn tool_add_evidence(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Review)?;
        let id =
            str_arg(args, "id").ok_or_else(|| bugbee_core::Error::Invalid("id required".into()))?;
        let kind = str_arg(args, "kind")
            .ok_or_else(|| bugbee_core::Error::Invalid("kind required".into()))?;
        let detail = str_arg(args, "detail")
            .ok_or_else(|| bugbee_core::Error::Invalid("detail required".into()))?;
        let mut f = self
            .ctx
            .store
            .lock()
            .get(&FindingId(id.to_string()))?
            .ok_or_else(|| bugbee_core::Error::NotFound(format!("finding {id}")))?;
        let loc = match (str_arg(args, "path"), int_arg(args, "line")) {
            (Some(p), Some(l)) => Some(Location::line(p, l as u32)),
            (Some(p), None) => Some(Location::line(p, 1)),
            _ => None,
        };
        f.push_evidence(Evidence {
            kind: kind.into(),
            detail: detail.into(),
            location: loc,
        });
        let ecs = f.ecs;
        let brs = f.brs;
        self.ctx.store.lock().upsert_finding(&f)?;
        Ok(ToolResult::ok(
            id,
            format!("evidence added; ecs={ecs} brs={brs}"),
        ))
    }

    fn tool_todo(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Todo)?;
        let items = args
            .get("items")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let todos: Vec<TodoItem> = items
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let content = v.get("content")?.as_str()?.to_string();
                let done = v.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
                Some(TodoItem {
                    id: format!("t{i}"),
                    content,
                    done,
                })
            })
            .collect();
        let n = todos.len();
        self.ctx.session.lock().set_todos(todos.clone());
        let body = todos
            .iter()
            .map(|t| format!("[{}] {}", if t.done { "x" } else { " " }, t.content))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ToolResult::ok("todos", body).with_meta(json!({ "count": n })))
    }

    fn tool_shell(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Shell)?;
        let cmd = str_arg(args, "command")
            .ok_or_else(|| bugbee_core::Error::Invalid("command required".into()))?;
        let lower = cmd.to_ascii_lowercase();
        for bad in [
            "rm -rf", "mkfs", "dd if=", ":(){", "curl |", "wget |", "nc -", "ncat",
        ] {
            if lower.contains(bad) {
                return Ok(ToolResult::err(
                    "shell",
                    format!("blocked dangerous command pattern: {bad}"),
                ));
            }
        }
        debug!(cmd = %cmd, "shell tool");
        let output = Command::new("bash")
            .arg("-lc")
            .arg(cmd)
            .current_dir(&self.ctx.root)
            .output()?;
        let mut text = String::new();
        text.push_str(&String::from_utf8_lossy(&output.stdout));
        if !output.stderr.is_empty() {
            text.push_str("\n--- stderr ---\n");
            text.push_str(&String::from_utf8_lossy(&output.stderr));
        }
        text.push_str(&format!("\n(exit {})", output.status.code().unwrap_or(-1)));
        Ok(ToolResult::ok("shell", text))
    }

    fn tool_edit(&self, args: &Value) -> Result<ToolResult> {
        self.ctx.gate.check(Permission::Edit)?;
        let path = str_arg(args, "path")
            .ok_or_else(|| bugbee_core::Error::Invalid("path required".into()))?;
        let old = str_arg(args, "old_string")
            .ok_or_else(|| bugbee_core::Error::Invalid("old_string required".into()))?;
        let new = str_arg(args, "new_string")
            .ok_or_else(|| bugbee_core::Error::Invalid("new_string required".into()))?;

        let resolved = self.ctx.gate.resolve_path(path)?;
        if self.ctx.gate.is_sensitive(&resolved) {
            return Ok(ToolResult::err(
                "edit",
                format!("blocked sensitive path: {}", resolved.display()),
            ));
        }
        if !resolved.starts_with(&self.ctx.root) {
            return Ok(ToolResult::err(
                "edit",
                "path escapes project root".to_string(),
            ));
        }

        let content = fs::read_to_string(&resolved)?;
        if !content.contains(old) {
            return Ok(ToolResult::err(
                "edit",
                format!("old_string not found in {}", self.ctx.gate.rel_display(&resolved)),
            ));
        }
        let count = content.matches(old).count();
        if count > 1 {
            return Ok(ToolResult::err(
                "edit",
                format!("old_string matched {count} times — provide more context"),
            ));
        }

        let new_content = content.replace(old, new);
        fs::write(&resolved, &new_content)?;

        let rel = self.ctx.gate.rel_display(&resolved);
        let finding_id = str_arg(args, "finding_id").unwrap_or("-");
        if finding_id != "-" && !finding_id.is_empty() {
            if let Ok(Some(mut f)) = self.ctx.store.lock().get(&bugbee_core::FindingId(finding_id.to_string())) {
                f.push_evidence(bugbee_core::Evidence {
                    kind: "patch".into(),
                    detail: format!("edit {}: replaced pattern ({} chars → {} chars)", rel, old.len(), new.len()),
                    location: None,
                });
                let _ = self.ctx.store.lock().upsert_finding(&f);
            }
        }

        Ok(ToolResult::ok(
            "edit",
            format!(
                "applied edit to {rel} ({} replacement{}), {} → {} chars",
                count,
                if count == 1 { "" } else { "s" },
                old.len(),
                new.len()
            ),
        ).with_meta(json!({
            "path": rel,
            "old_len": old.len(),
            "new_len": new.len(),
            "finding_id": finding_id
        })))
    }
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str())
}

fn int_arg(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(|v| v.as_u64())
}

fn glob_match(pattern: &str, path: &Path) -> bool {
    let s = path.to_string_lossy();
    if glob::Pattern::new(pattern)
        .map(|p| {
            p.matches(&s)
                || path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| p.matches(n))
        })
        .unwrap_or(false)
    {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("**/*.") {
        return s.ends_with(&format!(".{ext}"));
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return s.ends_with(&format!(".{ext}"));
    }
    s.contains(pattern.trim_start_matches('*'))
}

/// Parallel evidence enrichment: read snippets around each finding (no LLM).
pub fn parallel_enrich(store: &Mutex<Store>, root: &Path, max: usize) -> Result<usize> {
    use rayon::prelude::*;

    let findings = store.lock().list(None)?;
    let targets: Vec<_> = findings
        .into_iter()
        .filter(|f| f.evidence.len() < 2)
        .take(max)
        .collect();

    let root = root.to_path_buf();
    let updates: Vec<_> = targets
        .into_par_iter()
        .filter_map(|mut f| {
            let path = root.join(&f.location.path);
            let content = fs::read_to_string(&path).ok()?;
            let lines: Vec<&str> = content.lines().collect();
            let idx = f.location.start_line.saturating_sub(1) as usize;
            let start = idx.saturating_sub(3);
            let end = (idx + 4).min(lines.len());
            let window = lines[start..end].join("\n");
            f.push_evidence(Evidence {
                kind: "context_window".into(),
                detail: format!("lines {}-{}:\n{window}", start + 1, end),
                location: Some(Location::line(&f.location.path, f.location.start_line)),
            });
            Some(f)
        })
        .collect();

    let n = updates.len();
    store.lock().upsert_many(&updates)?;
    Ok(n)
}
