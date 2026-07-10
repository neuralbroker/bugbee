use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{BugbeeError, Result};
use crate::finding::{Finding, FindingStatus};

pub struct FindingStore {
    path: PathBuf,
    conn: Connection,
}

impl FindingStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        let store = Self { path, conn };
        store.init()?;
        Ok(store)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn init(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                json TEXT NOT NULL,
                brs REAL NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_findings_brs ON findings(brs DESC);
            CREATE INDEX IF NOT EXISTS idx_findings_status ON findings(status);
            "#,
        )?;
        Ok(())
    }

    pub fn upsert(&self, finding: &Finding) -> Result<()> {
        let json = serde_json::to_string(finding)?;
        self.conn.execute(
            r#"
            INSERT INTO findings (id, json, brs, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                json=excluded.json,
                brs=excluded.brs,
                status=excluded.status,
                updated_at=excluded.updated_at
            "#,
            params![
                finding.id.to_string(),
                json,
                finding.brs,
                format!("{:?}", finding.status).to_lowercase(),
                finding.created_at.to_rfc3339(),
                finding.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, id: &Uuid) -> Result<Finding> {
        let json: String = self
            .conn
            .query_row(
                "SELECT json FROM findings WHERE id = ?1",
                params![id.to_string()],
                |row| row.get(0),
            )
            .map_err(|_| BugbeeError::NotFound(format!("finding {id}")))?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn list_by_brs(&self, limit: usize) -> Result<Vec<Finding>> {
        let mut stmt = self
            .conn
            .prepare("SELECT json FROM findings ORDER BY brs DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;
        let mut out = Vec::new();
        for r in rows {
            let json = r?;
            out.push(serde_json::from_str(&json)?);
        }
        Ok(out)
    }

    pub fn list_all(&self) -> Result<Vec<Finding>> {
        self.list_by_brs(10_000)
    }

    pub fn update_status(&self, id: &Uuid, status: FindingStatus) -> Result<()> {
        let mut f = self.get(id)?;
        f.status = status;
        f.updated_at = chrono::Utc::now();
        self.upsert(&f)
    }

    pub fn count(&self) -> Result<usize> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM findings", [], |row| row.get(0))?;
        Ok(n as usize)
    }

    pub fn export_sarif(&self) -> Result<serde_json::Value> {
        let findings = self.list_all()?;
        let mut results = Vec::new();
        for f in &findings {
            let level = match f.severity {
                crate::finding::Severity::Critical | crate::finding::Severity::High => "error",
                crate::finding::Severity::Medium => "warning",
                _ => "note",
            };
            let loc = f.locations.first();
            let physical = loc.map(|l| {
                serde_json::json!({
                    "artifactLocation": { "uri": l.file },
                    "region": {
                        "startLine": l.start_line,
                        "endLine": l.end_line
                    }
                })
            });
            results.push(serde_json::json!({
                "ruleId": f.evidence.rule_id.clone().unwrap_or_else(|| f.category.clone()),
                "level": level,
                "message": { "text": format!("{} (BRS={:.1}, ECS={:.2})", f.title, f.brs, f.ecs) },
                "locations": physical.map(|p| vec![serde_json::json!({"physicalLocation": p})]).unwrap_or_default(),
                "properties": {
                    "brs": f.brs,
                    "ecs": f.ecs,
                    "cwe": f.cwe,
                    "owasp": f.owasp,
                    "status": format!("{:?}", f.status),
                    "id": f.id.to_string()
                }
            }));
        }

        Ok(serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "Bugbee",
                        "informationUri": "https://github.com/bugbee-dev/bugbee",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": []
                    }
                },
                "results": results
            }]
        }))
    }
}
