use bugbee_llm::ToolSpec;
use serde_json::json;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolName {
    Read,
    Grep,
    Glob,
    ListDir,
    Hunt,
    ListFindings,
    GetFinding,
    ReviewFinding,
    AddEvidence,
    TodoWrite,
    Shell,
}

impl ToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            ToolName::Read => "read",
            ToolName::Grep => "grep",
            ToolName::Glob => "glob",
            ToolName::ListDir => "list_dir",
            ToolName::Hunt => "hunt",
            ToolName::ListFindings => "list_findings",
            ToolName::GetFinding => "get_finding",
            ToolName::ReviewFinding => "review_finding",
            ToolName::AddEvidence => "add_evidence",
            ToolName::TodoWrite => "todo_write",
            ToolName::Shell => "shell",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "grep" => Some(Self::Grep),
            "glob" => Some(Self::Glob),
            "list_dir" => Some(Self::ListDir),
            "hunt" => Some(Self::Hunt),
            "list_findings" => Some(Self::ListFindings),
            "get_finding" => Some(Self::GetFinding),
            "review_finding" => Some(Self::ReviewFinding),
            "add_evidence" => Some(Self::AddEvidence),
            "todo_write" => Some(Self::TodoWrite),
            "shell" => Some(Self::Shell),
            _ => None,
        }
    }
}

/// Tool specs exposed to the model (OpenCode registry analogue).
pub fn tool_specs(include_shell: bool, include_edit_review: bool) -> Vec<ToolSpec> {
    let mut tools = vec![
        ToolSpec::function(
            "hunt",
            "Run deterministic local vulnerability engines (rules + secrets). Call early. Free, offline, fast.",
            json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        ),
        ToolSpec::function(
            "list_findings",
            "List stored findings with severity, scores, path, status. Optional status filter.",
            json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "description": "draft|confirmed|false_positive|fixed"
                    },
                    "limit": { "type": "integer", "description": "max rows (default 40)" }
                }
            }),
        ),
        ToolSpec::function(
            "get_finding",
            "Get full finding detail including evidence by id.",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" }
                },
                "required": ["id"]
            }),
        ),
        ToolSpec::function(
            "read",
            "Read a file under the project root. Use offset/limit for large files (1-indexed lines).",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "offset": { "type": "integer", "description": "start line 1-indexed" },
                    "limit": { "type": "integer", "description": "max lines (default 200)" }
                },
                "required": ["path"]
            }),
        ),
        ToolSpec::function(
            "grep",
            "Regex search file contents (ripgrep-style). Prefer over reading whole trees.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "path": { "type": "string", "description": "subdir or file, default root" },
                    "include": { "type": "string", "description": "glob like *.py" },
                    "max_matches": { "type": "integer" }
                },
                "required": ["pattern"]
            }),
        ),
        ToolSpec::function(
            "glob",
            "Find files by glob pattern under the project.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "e.g. **/*.{py,js}" }
                },
                "required": ["pattern"]
            }),
        ),
        ToolSpec::function(
            "list_dir",
            "List directory entries under the project root.",
            json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "relative path, default ." }
                }
            }),
        ),
        ToolSpec::function(
            "todo_write",
            "Replace the hunt plan todos. Track coverage of attack surfaces.",
            json!({
                "type": "object",
                "properties": {
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "content": { "type": "string" },
                                "done": { "type": "boolean" }
                            },
                            "required": ["content"]
                        }
                    }
                },
                "required": ["items"]
            }),
        ),
        ToolSpec::function(
            "add_evidence",
            "Attach re-verifiable evidence to a finding (increases ECS).",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "kind": { "type": "string", "description": "e.g. dataflow, sink, config" },
                    "detail": { "type": "string" },
                    "path": { "type": "string" },
                    "line": { "type": "integer" }
                },
                "required": ["id", "kind", "detail"]
            }),
        ),
    ];

    if include_edit_review {
        tools.push(ToolSpec::function(
            "review_finding",
            "Set finding status: confirm|fp|fixed. Use after evidence review.",
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string" },
                    "status": { "type": "string", "description": "confirm|fp|fixed|draft" }
                },
                "required": ["id", "status"]
            }),
        ));
    }

    if include_shell {
        tools.push(ToolSpec::function(
            "shell",
            "Run a short read-only diagnostic command (policy gated). Prefer dedicated tools.",
            json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
            }),
        ));
    }

    tools
}
