use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub id: &'static str,
    pub description: &'static str,
    pub can_edit: bool,
}

pub const AGENTS: &[AgentSpec] = &[
    AgentSpec {
        id: "hunt_lead",
        description: "Plans campaigns and owns the finding queue",
        can_edit: false,
    },
    AgentSpec {
        id: "scout",
        description: "Fast read-only codebase exploration",
        can_edit: false,
    },
    AgentSpec {
        id: "taint_analyst",
        description: "Follows data flows and builds evidence",
        can_edit: false,
    },
    AgentSpec {
        id: "adversarial_reviewer",
        description: "Attempts to falsify findings (auto-review)",
        can_edit: false,
    },
    AgentSpec {
        id: "patchsmith",
        description: "Proposes minimal patches for human approval",
        can_edit: true,
    },
];
