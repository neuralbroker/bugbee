//! Crash recovery: resume swarm state from SQLite/JSON checkpoint in <2s.

use std::fs;
use std::path::{Path, PathBuf};

use bugbee_akg::{AkgSnapshot, AttackKnowledgeGraph};
use bugbee_core::{Error, Finding, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub version: u32,
    pub root: String,
    pub phase: String,
    pub findings: Vec<Finding>,
    pub akg: AkgSnapshot,
    pub notes: Vec<String>,
}

pub struct CheckpointStore {
    path: PathBuf,
}

impl CheckpointStore {
    pub fn in_project(root: &Path) -> Self {
        Self {
            path: root.join(".bugbee").join("checkpoint.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn save(&self, ckpt: &Checkpoint) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(ckpt)?;
        // Atomic-ish write
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, text)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn load(&self) -> Result<Option<Checkpoint>> {
        if !self.path.is_file() {
            return Ok(None);
        }
        let text = fs::read_to_string(&self.path)?;
        let ckpt: Checkpoint = serde_json::from_str(&text)
            .map_err(|e| Error::Other(format!("checkpoint parse: {e}")))?;
        Ok(Some(ckpt))
    }

    pub fn clear(&self) -> Result<()> {
        if self.path.is_file() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

impl Checkpoint {
    pub fn new(
        root: &Path,
        phase: impl Into<String>,
        findings: Vec<Finding>,
        akg: &AttackKnowledgeGraph,
    ) -> Self {
        Self {
            version: 1,
            root: root.display().to_string(),
            phase: phase.into(),
            findings,
            akg: akg.snapshot(),
            notes: Vec::new(),
        }
    }

    pub fn restore_akg(&self) -> AttackKnowledgeGraph {
        AttackKnowledgeGraph::restore(self.akg.clone())
    }
}
