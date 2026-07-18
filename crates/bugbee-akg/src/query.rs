use bugbee_core::{FindingId, Severity};
use petgraph::algo::all_simple_paths;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::graph::{AkgEdge, AkgNode, AttackKnowledgeGraph, NodeId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillChain {
    pub nodes: Vec<NodeId>,
    pub labels: Vec<String>,
    pub max_severity: Severity,
    pub difficulty: PathDifficulty,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PathDifficulty {
    /// 0–100 (higher = harder).
    pub score: u8,
    pub hops: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPath {
    pub id: NodeId,
    pub label: String,
    pub impact: String,
    pub likelihood: u8,
    pub prerequisites: Vec<FindingId>,
}

impl AttackKnowledgeGraph {
    /// All attack-path nodes with their required findings.
    pub fn list_attack_paths(&self) -> Vec<AttackPath> {
        let mut out = Vec::new();
        for idx in self.raw().node_indices() {
            let Some(id) = self.id_of(idx) else { continue };
            let AkgNode::AttackPath {
                label,
                impact,
                likelihood,
            } = &self.raw()[idx]
            else {
                continue;
            };
            let mut prereq = Vec::new();
            for e in self.raw().edges(idx) {
                if *e.weight() == AkgEdge::Requires {
                    if let AkgNode::Finding { finding_id, .. } = &self.raw()[e.target()] {
                        prereq.push(finding_id.clone());
                    }
                }
            }
            out.push(AttackPath {
                id,
                label: label.clone(),
                impact: impact.clone(),
                likelihood: *likelihood,
                prerequisites: prereq,
            });
        }
        out
    }

    /// Suggest next findings to pursue given current verified set.
    pub fn suggest_pivot(&self) -> Vec<FindingId> {
        let mut candidates = Vec::new();
        for idx in self.raw().node_indices() {
            if let AkgNode::Finding {
                finding_id,
                verified,
                severity,
                ..
            } = &self.raw()[idx]
            {
                if !verified && matches!(severity, Severity::Critical | Severity::High) {
                    candidates.push(finding_id.clone());
                }
            }
        }
        candidates
    }

    /// Estimate difficulty of a multi-hop chain.
    pub fn estimate_difficulty(&self, hops: usize, max_sev: Severity) -> PathDifficulty {
        let sev_w = match max_sev {
            Severity::Critical => 10,
            Severity::High => 20,
            Severity::Medium => 35,
            Severity::Low => 50,
            Severity::Info => 60,
        };
        let hop_w = (hops as u8).saturating_mul(15).min(50);
        PathDifficulty {
            score: (sev_w + hop_w).min(100),
            hops,
        }
    }

    /// Find kill chains: attack-path nodes + multi-hop finding paths.
    pub fn find_kill_chains(&self, max_len: usize) -> Vec<KillChain> {
        let mut chains = Vec::new();

        // Synthesized attack paths count as first-class kill chains
        for path in self.list_attack_paths() {
            let max_sev = Severity::High;
            let mut labels = vec![path.label.clone()];
            labels.push(path.impact.clone());
            let nodes: Vec<_> = std::iter::once(path.id.clone())
                .chain(
                    path.prerequisites
                        .iter()
                        .map(|f| NodeId(format!("finding:{}", f.0))),
                )
                .collect();
            let difficulty = self.estimate_difficulty(nodes.len().max(2), max_sev);
            chains.push(KillChain {
                nodes,
                labels,
                max_severity: max_sev,
                difficulty,
            });
        }

        let finding_nodes: Vec<_> = self
            .raw()
            .node_indices()
            .filter(|&i| matches!(self.raw()[i], AkgNode::Finding { .. }))
            .collect();

        for &start in &finding_nodes {
            for &end in &finding_nodes {
                if start == end {
                    continue;
                }
                let paths = all_simple_paths::<Vec<_>, _>(self.raw(), start, end, 0, Some(max_len));
                for path in paths.take(5) {
                    let mut labels = Vec::new();
                    let mut max_sev = Severity::Info;
                    let mut nodes = Vec::new();
                    for idx in &path {
                        if let Some(id) = self.id_of(*idx) {
                            nodes.push(id);
                        }
                        match &self.raw()[*idx] {
                            AkgNode::Finding {
                                title, severity, ..
                            } => {
                                labels.push(title.clone());
                                if *severity > max_sev {
                                    max_sev = *severity;
                                }
                            }
                            AkgNode::Asset { label, .. } => labels.push(label.clone()),
                            AkgNode::AttackPath { label, .. } => labels.push(label.clone()),
                            AkgNode::Credential { kind, .. } => labels.push(kind.clone()),
                        }
                    }
                    if nodes.len() >= 2 {
                        let difficulty = self.estimate_difficulty(nodes.len(), max_sev);
                        chains.push(KillChain {
                            nodes,
                            labels,
                            max_severity: max_sev,
                            difficulty,
                        });
                    }
                }
            }
        }
        chains.sort_by_key(|c| std::cmp::Reverse(c.max_severity));
        chains.truncate(20);
        chains
    }

    /// Auto-build simple chains: secrets → eval/cmdi on same file.
    pub fn synthesize_local_chains(&mut self, findings: &[bugbee_core::Finding]) {
        use std::collections::HashMap;
        let mut by_path: HashMap<String, Vec<&bugbee_core::Finding>> = HashMap::new();
        for f in findings {
            by_path.entry(f.location.path.clone()).or_default().push(f);
        }
        for (path, group) in by_path {
            if group.len() < 2 {
                continue;
            }
            let ids: Vec<_> = group.iter().map(|f| f.id.clone()).collect();
            let has_secret = group.iter().any(|f| {
                f.tags.iter().any(|t| t.contains("secret"))
                    || f.rule_id.contains("secret")
                    || f.rule_id.contains("password")
            });
            let has_rce = group.iter().any(|f| {
                matches!(f.severity, Severity::Critical | Severity::High)
                    && (f.rule_id.contains("eval")
                        || f.rule_id.contains("shell")
                        || f.rule_id.contains("exec"))
            });
            if has_secret && has_rce {
                self.add_attack_path(
                    format!("secret→rce:{path}"),
                    "credential exposure enables high-impact code sink on same file",
                    70,
                    &ids,
                );
            } else if group.len() >= 2 {
                self.add_attack_path(
                    format!("multi-find:{path}"),
                    "multiple findings share file — review combined impact",
                    40,
                    &ids,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bugbee_core::{Finding, Location, Severity, SourceKind};

    #[test]
    fn ingest_and_chain() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = Finding::new(
            "secrets.x",
            "Secret",
            "key",
            Severity::High,
            SourceKind::Secrets,
            Location::line("app.py", 1),
        );
        let f2 = Finding::new(
            "owasp.python.eval",
            "Eval",
            "eval",
            Severity::Critical,
            SourceKind::Rule,
            Location::line("app.py", 10),
        );
        g.ingest_findings(&[f1.clone(), f2.clone()]);
        g.synthesize_local_chains(&[f1, f2]);
        assert!(g.node_count() >= 2);
        assert!(!g.list_attack_paths().is_empty() || g.edge_count() > 0);
    }
}
