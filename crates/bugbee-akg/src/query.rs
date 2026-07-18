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

    fn make_finding(rule_id: &str, title: &str, sev: Severity, path: &str, line: u32) -> Finding {
        Finding::new(rule_id, title, "", sev, SourceKind::Rule, Location::line(path, line))
    }

    fn make_secret(rule_id: &str, path: &str) -> Finding {
        let mut f = Finding::new(
            rule_id,
            "Hardcoded Secret",
            "API key in source",
            Severity::High,
            SourceKind::Secrets,
            Location::line(path, 5),
        );
        f.tags = vec!["secret".into(), "hardcoded".into()];
        f.rule_id = "secret.hardcoded".into();
        f
    }

    fn make_rce(rule_id: &str, path: &str) -> Finding {
        let mut f = Finding::new(
            rule_id,
            "Command Injection",
            "eval on user input",
            Severity::Critical,
            SourceKind::Rule,
            Location::line(path, 10),
        );
        f.rule_id = "rce.command.exec".into();
        f
    }

    fn nid(f: &Finding) -> NodeId {
        NodeId(format!("finding:{}", f.id.0))
    }

    // ── Topology fixture: 3-chain kill path ───────────────────────

    #[test]
    fn ingest_and_chain() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("secrets.x", "Secret", Severity::High, "app.py", 1);
        let f2 = make_finding("owasp.python.eval", "Eval", Severity::Critical, "app.py", 10);
        g.ingest_findings(&[f1.clone(), f2.clone()]);
        g.synthesize_local_chains(&[f1, f2]);
        assert!(g.node_count() >= 2);
        assert!(!g.list_attack_paths().is_empty() || g.edge_count() > 0);
    }

    #[test]
    fn kill_chain_topology_three_hop() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("sql-injection", "SQLi", Severity::Critical, "db.rs", 42);
        let f2 = make_finding("xss", "XSS", Severity::High, "ui.rs", 15);
        let f3 = make_finding("cmd-injection", "CMDi", Severity::Critical, "exec.rs", 7);

        g.ingest_findings(&[f1.clone(), f2.clone(), f3.clone()]);

        let a = nid(&f1);
        let b = nid(&f2);
        let c = nid(&f3);
        g.link(&a, &b, AkgEdge::DependsOn);
        g.link(&b, &c, AkgEdge::DependsOn);

        let chains = g.find_kill_chains(5);
        assert!(!chains.is_empty(), "should find at least one chain");

        let has_three_hop = chains.iter().any(|c| c.nodes.len() >= 3);
        assert!(has_three_hop, "expected a 3-hop kill chain");
    }

    #[test]
    fn kill_chain_with_explicit_attack_path() {
        let mut g = AttackKnowledgeGraph::new();

        let f1 = make_finding("cred-1", "AWS Key", Severity::High, "config.py", 1);
        let f2 = make_finding("rce-1", "RCE via eval", Severity::Critical, "exec.py", 20);
        g.ingest_findings(&[f1.clone(), f2.clone()]);

        g.add_attack_path(
            "cred-to-rce",
            "AWS key exposure enables authenticated RCE",
            85,
            &[f1.id.clone(), f2.id.clone()],
        );

        let chains = g.find_kill_chains(5);
        let has_path = chains.iter().any(|c| c.labels.iter().any(|l| l.contains("cred-to-rce")));
        assert!(has_path, "attack path should appear in kill chains");
    }

    // ── Path difficulty scoring ───────────────────────────────────

    #[test]
    fn path_difficulty_critical() {
        let g = AttackKnowledgeGraph::new();
        let d = g.estimate_difficulty(2, Severity::Critical);
        assert_eq!(d.score, 40);
        assert_eq!(d.hops, 2);
    }

    #[test]
    fn path_difficulty_info() {
        let g = AttackKnowledgeGraph::new();
        let d = g.estimate_difficulty(1, Severity::Info);
        assert_eq!(d.score, 75);
        assert_eq!(d.hops, 1);
    }

    #[test]
    fn path_difficulty_capped_hops() {
        let g = AttackKnowledgeGraph::new();
        let d = g.estimate_difficulty(10, Severity::Medium);
        assert_eq!(d.score, 85);
        assert_eq!(d.hops, 10);
    }

    // ── Empty graph / edge cases ──────────────────────────────────

    #[test]
    fn empty_graph_no_chains() {
        let g = AttackKnowledgeGraph::new();
        let chains = g.find_kill_chains(5);
        assert!(chains.is_empty());
    }

    #[test]
    fn empty_graph_no_attack_paths() {
        let g = AttackKnowledgeGraph::new();
        let paths = g.list_attack_paths();
        assert!(paths.is_empty());
    }

    #[test]
    fn empty_graph_no_pivot() {
        let g = AttackKnowledgeGraph::new();
        let pivots = g.suggest_pivot();
        assert!(pivots.is_empty());
    }

    // ── Pivot suggestions ─────────────────────────────────────────

    #[test]
    fn suggest_pivot_returns_unverified_critical() {
        let mut g = AttackKnowledgeGraph::new();
        let f = make_finding("sqli", "SQLi", Severity::Critical, "db.rs", 1);
        let id = f.id.clone();
        g.ingest_findings(&[f]);
        let pivots = g.suggest_pivot();
        assert_eq!(pivots.len(), 1);
        assert_eq!(pivots[0], id);
    }

    #[test]
    fn suggest_pivot_skips_verified() {
        let mut g = AttackKnowledgeGraph::new();
        let mut f = make_finding("sqli", "SQLi", Severity::Critical, "db.rs", 1);
        f.verified = true;
        g.ingest_findings(&[f]);
        let pivots = g.suggest_pivot();
        assert!(pivots.is_empty());
    }

    #[test]
    fn suggest_pivot_skips_info() {
        let mut g = AttackKnowledgeGraph::new();
        let f = make_finding("style", "Style", Severity::Info, "db.rs", 1);
        g.ingest_findings(&[f]);
        let pivots = g.suggest_pivot();
        assert!(pivots.is_empty());
    }

    // ── Attack path missing prerequisites ─────────────────────────

    #[test]
    fn attack_path_missing_prereqs() {
        let mut g = AttackKnowledgeGraph::new();
        g.add_attack_path("chain-1", "impact", 50, &[FindingId("nonexistent".into())]);
        let paths = g.list_attack_paths();
        assert!(paths.is_empty() || paths[0].prerequisites.is_empty());
    }

    // ── DependsOn multi-hop ───────────────────────────────────────

    #[test]
    fn depends_on_creates_dependency_chain() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("a", "Finding A", Severity::Medium, "f1.py", 1);
        let f2 = make_finding("b", "Finding B", Severity::High, "f2.py", 2);
        g.ingest_findings(&[f1.clone(), f2.clone()]);
        g.link(&nid(&f1), &nid(&f2), AkgEdge::DependsOn);
        let chains = g.find_kill_chains(5);
        let has_dep_chain = chains.iter().any(|c| c.nodes.len() >= 2);
        assert!(has_dep_chain, "expected depends-on chain");
    }

    // ── Secret→RCE synthesis ──────────────────────────────────────

    #[test]
    fn synthesize_secret_to_rce() {
        let mut g = AttackKnowledgeGraph::new();
        let secret = make_secret("sec-1", "config.py");
        let rce = make_rce("rce-1", "config.py");
        g.ingest_findings(&[secret.clone(), rce.clone()]);
        g.synthesize_local_chains(&[secret, rce]);
        let paths = g.list_attack_paths();
        let has = paths.iter().any(|p| p.label.contains("secret→rce"));
        assert!(has, "expected secret→rce path");
    }

    #[test]
    fn synthesize_skips_single() {
        let mut g = AttackKnowledgeGraph::new();
        let secret = make_secret("sec-1", "config.py");
        g.ingest_findings(&[secret.clone()]);
        g.synthesize_local_chains(&[secret]);
        assert!(g.list_attack_paths().is_empty());
    }

    // ── Snapshot round-trip ───────────────────────────────────────

    #[test]
    fn snapshot_round_trip() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("sql", "SQLi", Severity::Critical, "app.py", 1);
        let f2 = make_finding("xss", "XSS", Severity::High, "app.py", 2);
        g.ingest_findings(&[f1.clone(), f2.clone()]);
        g.add_attack_path("tp", "impact", 50, &[f1.id.clone(), f2.id.clone()]);

        let snap = g.snapshot();
        assert_eq!(snap.nodes.len(), g.node_count());
        assert_eq!(snap.edges.len(), g.edge_count());

        let restored = AttackKnowledgeGraph::restore(snap);
        assert_eq!(restored.node_count(), g.node_count());
        assert_eq!(restored.edge_count(), g.edge_count());
    }

    // ── Duplicate edge prevention ─────────────────────────────────

    #[test]
    fn duplicate_edges_not_added() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("fa", "A", Severity::High, "a.py", 1);
        let f2 = make_finding("fb", "B", Severity::High, "b.py", 2);
        g.ingest_findings(&[f1.clone(), f2.clone()]);

        let a = nid(&f1);
        let b = nid(&f2);
        g.link(&a, &b, AkgEdge::DependsOn);
        let count = g.edge_count();
        g.link(&a, &b, AkgEdge::DependsOn);
        assert_eq!(g.edge_count(), count);
    }

    // ── Multiple attack paths ─────────────────────────────────────

    #[test]
    fn multiple_attack_paths_listed() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("v1", "SQLi", Severity::Critical, "db.rs", 1);
        let f2 = make_finding("v2", "XSS", Severity::High, "ui.rs", 2);
        let f3 = make_finding("v3", "RCE", Severity::Critical, "exec.rs", 3);
        g.ingest_findings(&[f1, f2, f3]);
        g.add_attack_path("path-a", "data exfil", 70, &[FindingId("v1".into())]);
        g.add_attack_path("path-b", "full compromise", 90, &[FindingId("v1".into()), FindingId("v3".into())]);

        assert_eq!(g.list_attack_paths().len(), 2);
    }

    // ── Ingest with chain_dependencies ────────────────────────────

    #[test]
    fn ingest_with_chain_deps() {
        let mut g = AttackKnowledgeGraph::new();
        let f1 = make_finding("sql", "SQLi", Severity::Critical, "app.rs", 1);
        let mut f2 = make_finding("auth", "Auth Bypass", Severity::Critical, "app.rs", 5);
        f2.chain_dependencies.push(f1.id.clone());
        g.ingest_findings(&[f1, f2]);
        assert!(g.edge_count() >= 1, "expected DependsOn edges from chain_dependencies");
    }
}
