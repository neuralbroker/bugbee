//! Target / asset model for recon and AKG.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetId(pub String);

impl TargetId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn from_path(path: &str) -> Self {
        Self(format!("path:{}", path))
    }

    pub fn from_service(host: &str, port: u16) -> Self {
        Self(format!("svc:{host}:{port}"))
    }
}

impl Default for TargetId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMechanism {
    None,
    Jwt,
    OAuth,
    Cookie,
    ApiKey,
    MutualTls,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: TargetId,
    pub kind: TargetKind,
    pub label: String,
    pub auth: AuthMechanism,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    SourceFile { path: String },
    Endpoint { method: String, path: String },
    Service { host: String, port: u16 },
    Repo { root: String },
}

impl Target {
    pub fn source_file(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            id: TargetId::from_path(&path),
            kind: TargetKind::SourceFile { path: path.clone() },
            label: path,
            auth: AuthMechanism::None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn repo(root: impl Into<String>) -> Self {
        let root = root.into();
        Self {
            id: TargetId(format!("repo:{root}")),
            kind: TargetKind::Repo { root: root.clone() },
            label: root,
            auth: AuthMechanism::None,
            metadata: serde_json::json!({}),
        }
    }
}
