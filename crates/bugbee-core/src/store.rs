use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{Error, Result};
use crate::finding::{Finding, FindingId, FindingStatus};

/// Local SQLite store for findings and session metadata.
pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path.as_ref())?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS findings (
                id TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                status TEXT NOT NULL,
                severity TEXT NOT NULL,
                path TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_findings_status ON findings(status);
            CREATE INDEX IF NOT EXISTS idx_findings_path ON findings(path);
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_finding(&self, finding: &Finding) -> Result<()> {
        let payload = serde_json::to_string(finding)?;
        self.conn.execute(
            r#"
            INSERT INTO findings (id, payload, status, severity, path, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                payload = excluded.payload,
                status = excluded.status,
                severity = excluded.severity,
                path = excluded.path,
                updated_at = excluded.updated_at
            "#,
            params![
                finding.id.0,
                payload,
                finding.status.as_str(),
                finding.severity.as_str(),
                finding.location.path,
                finding.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn upsert_many(&self, findings: &[Finding]) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO findings (id, payload, status, severity, path, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(id) DO UPDATE SET
                    payload = excluded.payload,
                    status = excluded.status,
                    severity = excluded.severity,
                    path = excluded.path,
                    updated_at = excluded.updated_at
                "#,
            )?;
            for finding in findings {
                let payload = serde_json::to_string(finding)?;
                stmt.execute(params![
                    finding.id.0,
                    payload,
                    finding.status.as_str(),
                    finding.severity.as_str(),
                    finding.location.path,
                    finding.updated_at.to_rfc3339(),
                ])?;
            }
        }
        tx.commit()?;
        Ok(findings.len())
    }

    pub fn get(&self, id: &FindingId) -> Result<Option<Finding>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload FROM findings WHERE id = ?1")?;
        let row: Option<String> = stmt.query_row(params![id.0], |r| r.get(0)).optional()?;
        match row {
            Some(payload) => Ok(Some(serde_json::from_str(&payload)?)),
            None => Ok(None),
        }
    }

    pub fn list(&self, status: Option<FindingStatus>) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();
        if let Some(status) = status {
            let mut stmt = self.conn.prepare(
                "SELECT payload FROM findings WHERE status = ?1 ORDER BY updated_at DESC",
            )?;
            let rows = stmt.query_map(params![status.as_str()], |r| r.get::<_, String>(0))?;
            for row in rows {
                findings.push(serde_json::from_str(&row?)?);
            }
        } else {
            let mut stmt = self
                .conn
                .prepare("SELECT payload FROM findings ORDER BY updated_at DESC")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            for row in rows {
                findings.push(serde_json::from_str(&row?)?);
            }
        }
        Ok(findings)
    }

    pub fn set_status(&self, id: &FindingId, status: FindingStatus) -> Result<Finding> {
        let mut finding = self
            .get(id)?
            .ok_or_else(|| Error::NotFound(format!("finding {id}")))?;
        finding.set_status(status);
        self.upsert_finding(&finding)?;
        Ok(finding)
    }

    pub fn count(&self) -> Result<usize> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM findings", [], |r| r.get(0))?;
        Ok(n as usize)
    }

    pub fn clear_drafts(&self) -> Result<usize> {
        let n = self
            .conn
            .execute("DELETE FROM findings WHERE status = 'draft'", [])?;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Location, Severity, SourceKind};

    #[test]
    fn roundtrip() {
        let store = Store::open_in_memory().unwrap();
        let f = Finding::new(
            "sec.hardcoded",
            "Hardcoded secret",
            "possible api key",
            Severity::High,
            SourceKind::Secrets,
            Location::line("cfg.env", 1),
        );
        store.upsert_finding(&f).unwrap();
        let got = store.get(&f.id).unwrap().unwrap();
        assert_eq!(got.rule_id, "sec.hardcoded");
        assert_eq!(store.count().unwrap(), 1);
    }
}
