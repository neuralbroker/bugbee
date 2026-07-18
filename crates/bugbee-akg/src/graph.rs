use std::collections::HashMap;

use bugbee_core::{Finding, FindingId, Severity, Target, TargetId};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AkgNode {
    Asset {
        target_id: TargetId,
        label: String,
        auth_required: bool,
    },
    Finding {
        finding_id: FindingId,
        cwe: Option<String>,
        severity: Severity,
        verified: bool,
        title: String,
    },
    Credential {
        kind: String,
        privilege: String,
    },
    AttackPath {
        label: String,
        impact: String,
        likelihood: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AkgEdge {
    /// Finding unlocks access to asset.
    Enables,
    /// Attack path requires finding.
    Requires,
    /// Path exfiltrates data asset.
    Exfiltrates,
    /// Path escalates privilege.
    Escalates,
    /// Finding depends on another finding (chain).
    DependsOn,
    /// Asset contains finding location.
    Hosts,
}

/// In-memory directed attack knowledge graph.
pub struct AttackKnowledgeGraph {
    graph: DiGraph<AkgNode, AkgEdge>,
    index: HashMap<NodeId, NodeIndex>,
}

impl Default for AttackKnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AttackKnowledgeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    fn upsert(&mut self, id: NodeId, node: AkgNode) -> NodeIndex {
        if let Some(&idx) = self.index.get(&id) {
            self.graph[idx] = node;
            return idx;
        }
        let idx = self.graph.add_node(node);
        self.index.insert(id, idx);
        idx
    }

    pub fn add_asset(&mut self, target: &Target, auth_required: bool) -> NodeId {
        let id = NodeId(format!("asset:{}", target.id.0));
        self.upsert(
            id.clone(),
            AkgNode::Asset {
                target_id: target.id.clone(),
                label: target.label.clone(),
                auth_required,
            },
        );
        id
    }

    pub fn add_finding(&mut self, f: &Finding) -> NodeId {
        let id = NodeId(format!("finding:{}", f.id.0));
        self.upsert(
            id.clone(),
            AkgNode::Finding {
                finding_id: f.id.clone(),
                cwe: f.cwe.clone(),
                severity: f.severity,
                verified: f.verified,
                title: f.title.clone(),
            },
        );
        // Host edge to path asset
        let asset = Target::source_file(&f.location.path);
        let asset_id = self.add_asset(&asset, false);
        self.link(&id, &asset_id, AkgEdge::Hosts);
        id
    }

    pub fn add_attack_path(
        &mut self,
        label: impl Into<String>,
        impact: impl Into<String>,
        likelihood: u8,
        required_findings: &[FindingId],
    ) -> NodeId {
        let label = label.into();
        let id = NodeId(format!("path:{}", label));
        self.upsert(
            id.clone(),
            AkgNode::AttackPath {
                label: label.clone(),
                impact: impact.into(),
                likelihood,
            },
        );
        for fid in required_findings {
            let fid_node = NodeId(format!("finding:{}", fid.0));
            if self.index.contains_key(&fid_node) {
                self.link(&id, &fid_node, AkgEdge::Requires);
            }
        }
        id
    }

    pub fn link(&mut self, from: &NodeId, to: &NodeId, edge: AkgEdge) {
        let (Some(&a), Some(&b)) = (self.index.get(from), self.index.get(to)) else {
            return;
        };
        // Avoid duplicate edges of same type
        for e in self.graph.edges_directed(a, Direction::Outgoing) {
            if e.target() == b && *e.weight() == edge {
                return;
            }
        }
        self.graph.add_edge(a, b, edge);
    }

    pub fn get(&self, id: &NodeId) -> Option<&AkgNode> {
        self.index.get(id).map(|&i| &self.graph[i])
    }

    pub fn findings(&self) -> Vec<&AkgNode> {
        self.graph
            .node_weights()
            .filter(|n| matches!(n, AkgNode::Finding { .. }))
            .collect()
    }

    /// Ingest store findings into the graph.
    pub fn ingest_findings(&mut self, findings: &[Finding]) {
        for f in findings {
            self.add_finding(f);
            for dep in &f.chain_dependencies {
                let a = NodeId(format!("finding:{}", f.id.0));
                let b = NodeId(format!("finding:{}", dep.0));
                if self.index.contains_key(&b) {
                    self.link(&a, &b, AkgEdge::DependsOn);
                }
            }
        }
    }

    /// Serialize graph snapshot for checkpointing.
    pub fn snapshot(&self) -> AkgSnapshot {
        let mut nodes = Vec::new();
        for (id, &idx) in &self.index {
            nodes.push((id.clone(), self.graph[idx].clone()));
        }
        let mut edges = Vec::new();
        for e in self.graph.edge_references() {
            let from = self
                .index
                .iter()
                .find(|(_, &i)| i == e.source())
                .map(|(id, _)| id.clone());
            let to = self
                .index
                .iter()
                .find(|(_, &i)| i == e.target())
                .map(|(id, _)| id.clone());
            if let (Some(f), Some(t)) = (from, to) {
                edges.push((f, t, *e.weight()));
            }
        }
        AkgSnapshot { nodes, edges }
    }

    pub fn restore(snapshot: AkgSnapshot) -> Self {
        let mut g = Self::new();
        for (id, node) in snapshot.nodes {
            g.upsert(id, node);
        }
        for (from, to, edge) in snapshot.edges {
            g.link(&from, &to, edge);
        }
        g
    }

    pub(crate) fn raw(&self) -> &DiGraph<AkgNode, AkgEdge> {
        &self.graph
    }

    pub(crate) fn id_of(&self, idx: NodeIndex) -> Option<NodeId> {
        self.index
            .iter()
            .find(|(_, &i)| i == idx)
            .map(|(id, _)| id.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AkgSnapshot {
    pub nodes: Vec<(NodeId, AkgNode)>,
    pub edges: Vec<(NodeId, NodeId, AkgEdge)>,
}
