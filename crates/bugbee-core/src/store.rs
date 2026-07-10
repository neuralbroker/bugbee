use std::collections::BTreeMap;
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

    /// Record the latest observation of a detector finding while preserving
    /// analyst decisions and history from earlier scans.
    pub fn upsert_observation(&self, finding: &Finding) -> Result<()> {
        let mut merged = finding.clone();
        if let Ok(existing) = self.get(&finding.id) {
            merged.created_at = existing.created_at;
            merged.reviews = existing.reviews;
            merged.patch_diff = existing.patch_diff;
            merged.status = match existing.status {
                // A finding marked fixed that is still detected has regressed.
                FindingStatus::Fixed => FindingStatus::New,
                status => status,
            };
        }
        merged.updated_at = chrono::Utc::now();
        self.upsert(&merged)
    }

    /// Remove stale findings that were never given a durable analyst decision.
    /// Confirmed, false-positive, fixed, and won't-fix findings remain as history.
    pub fn prune_unreviewed_except(&self, observed_ids: &[Uuid]) -> Result<usize> {
        let statuses = "'new', 'triaged'";
        if observed_ids.is_empty() {
            return Ok(self.conn.execute(
                &format!("DELETE FROM findings WHERE status IN ({statuses})"),
                [],
            )?);
        }

        let placeholders = std::iter::repeat_n("?", observed_ids.len())
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!(
            "DELETE FROM findings WHERE status IN ({statuses}) AND id NOT IN ({placeholders})"
        );
        let ids = observed_ids.iter().map(Uuid::to_string).collect::<Vec<_>>();
        Ok(self.conn.execute(&query, rusqlite::params_from_iter(ids))?)
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
        let mut rules = BTreeMap::new();
        for f in &findings {
            let rule_id = f
                .evidence
                .rule_id
                .clone()
                .unwrap_or_else(|| f.category.clone());
            rules.entry(rule_id.clone()).or_insert_with(|| {
                serde_json::json!({
                    "id": rule_id,
                    "name": f.title,
                    "shortDescription": { "text": f.title },
                    "fullDescription": { "text": f.description },
                    "help": { "text": f.evidence.agent_notes.clone().unwrap_or_else(|| "Review the evidence and validate the finding before remediation.".into()) },
                    "properties": {
                        "category": f.category,
                        "cwe": f.cwe,
                        "owasp": f.owasp
                    }
                })
            });
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
                "ruleId": rule_id,
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
                        "informationUri": "https://github.com/neuralbroker/bugbee",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": rules.into_values().collect::<Vec<_>>()
                    }
                },
                "results": results
            }]
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Evidence, FindingLocation, LocationRole, Severity};
    use crate::scoring::BrsWeights;

    fn test_store() -> (FindingStore, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("bugbee-store-{}.db", Uuid::new_v4()));
        let store = FindingStore::open(&path).expect("open temporary finding store");
        (store, path)
    }

    fn candidate() -> Finding {
        let mut finding = Finding::new("Unsafe evaluation", "desc", Severity::High, "injection");
        finding.evidence = Evidence {
            rule_id: Some("python.eval".into()),
            ..Evidence::default()
        };
        finding.add_location(FindingLocation {
            file: "app.py".into(),
            start_line: 9,
            end_line: 9,
            start_col: None,
            end_col: None,
            role: LocationRole::Sink,
            snippet: None,
        });
        finding.recompute_scores(&BrsWeights::default());
        finding
    }

    #[test]
    fn observations_preserve_review_state_and_do_not_duplicate() {
        let (store, path) = test_store();
        let first = candidate();
        store.upsert_observation(&first).unwrap();
        store
            .update_status(&first.id, FindingStatus::Confirmed)
            .unwrap();

        store.upsert_observation(&candidate()).unwrap();
        assert_eq!(store.count().unwrap(), 1);
        assert_eq!(
            store.get(&first.id).unwrap().status,
            FindingStatus::Confirmed
        );

        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn pruning_only_removes_unreviewed_findings() {
        let (store, path) = test_store();
        let active = candidate();
        store.upsert_observation(&active).unwrap();

        let mut stale = candidate();
        stale.locations[0].start_line = 10;
        stale.locations[0].end_line = 10;
        stale.recompute_scores(&BrsWeights::default());
        store.upsert_observation(&stale).unwrap();
        store
            .update_status(&stale.id, FindingStatus::Confirmed)
            .unwrap();

        assert_eq!(store.prune_unreviewed_except(&[active.id]).unwrap(), 0);
        assert_eq!(store.count().unwrap(), 2);

        let mut disposable = candidate();
        disposable.locations[0].start_line = 11;
        disposable.locations[0].end_line = 11;
        disposable.recompute_scores(&BrsWeights::default());
        store.upsert_observation(&disposable).unwrap();
        assert_eq!(store.prune_unreviewed_except(&[active.id]).unwrap(), 1);
        assert_eq!(store.count().unwrap(), 2);

        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn sarif_includes_rule_metadata_and_locations() {
        let (store, path) = test_store();
        let finding = candidate();
        store.upsert_observation(&finding).unwrap();

        let sarif = store.export_sarif().unwrap();
        let run = &sarif["runs"][0];
        assert_eq!(sarif["version"], "2.1.0");
        assert_eq!(run["tool"]["driver"]["rules"].as_array().unwrap().len(), 1);
        assert_eq!(
            run["tool"]["driver"]["rules"][0]["shortDescription"]["text"],
            "Unsafe evaluation"
        );
        assert_eq!(run["results"][0]["ruleId"], "python.eval");
        assert_eq!(
            run["results"][0]["locations"][0]["physicalLocation"]["region"]["startLine"],
            9
        );

        drop(store);
        std::fs::remove_file(path).unwrap();
    }
}
