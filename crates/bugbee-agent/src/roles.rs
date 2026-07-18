//! Multi-agent constitution (OpenCode build/plan → Bugbee hunt/review).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleKind {
    HuntLead,
    Scout,
    TaintAnalyst,
    AdversarialReviewer,
    Patchsmith,
    // Spec §2 swarm topology
    Recon,
    Hunter,
    Symbolic,
    Prover,
    Chain,
    Scribe,
}

impl RoleKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RoleKind::HuntLead => "hunt_lead",
            RoleKind::Scout => "scout",
            RoleKind::TaintAnalyst => "taint_analyst",
            RoleKind::AdversarialReviewer => "adversarial_reviewer",
            RoleKind::Patchsmith => "patchsmith",
            RoleKind::Recon => "recon",
            RoleKind::Hunter => "hunter",
            RoleKind::Symbolic => "symbolic",
            RoleKind::Prover => "prover",
            RoleKind::Chain => "chain",
            RoleKind::Scribe => "scribe",
        }
    }

    pub fn display(self) -> &'static str {
        match self {
            RoleKind::HuntLead => "Hunt Lead",
            RoleKind::Scout => "Scout",
            RoleKind::TaintAnalyst => "Taint Analyst",
            RoleKind::AdversarialReviewer => "Adversarial Reviewer",
            RoleKind::Patchsmith => "Patchsmith",
            RoleKind::Recon => "ReconAgent",
            RoleKind::Hunter => "HunterAgent",
            RoleKind::Symbolic => "SymbolicAgent",
            RoleKind::Prover => "ProverAgent",
            RoleKind::Chain => "ChainAgent",
            RoleKind::Scribe => "ScribeAgent",
        }
    }

    /// OpenCode analogue.
    pub fn opencode_analogue(self) -> &'static str {
        match self {
            RoleKind::HuntLead | RoleKind::Patchsmith | RoleKind::Hunter | RoleKind::Prover => {
                "build"
            }
            RoleKind::Scout
            | RoleKind::TaintAnalyst
            | RoleKind::AdversarialReviewer
            | RoleKind::Recon
            | RoleKind::Symbolic
            | RoleKind::Chain
            | RoleKind::Scribe => "plan",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentRole {
    pub kind: RoleKind,
    pub system_prompt: String,
    pub read_only: bool,
    /// Max tool-loop steps for this role.
    pub max_steps: u32,
}

impl AgentRole {
    pub fn builtin(kind: RoleKind) -> Self {
        let (read_only, max_steps, system_prompt) = match kind {
            RoleKind::HuntLead => (
                false,
                24u32,
                concat!(
                    "You are Bugbee Hunt Lead — the primary security agent (OpenCode `build` analogue).\n",
                    "Mission: maximize true-positive high-impact findings with proof.\n",
                    "Workflow (efficient):\n",
                    "1) call `hunt` first for deterministic candidates\n",
                    "2) `list_findings` then deep-dive with `read`/`grep`/`glob`\n",
                    "3) attach evidence via `add_evidence`; promote with `review_finding`\n",
                    "4) use `todo_write` to track coverage; never invent file paths\n",
                    "HARD RULES: defense-only; no live exploits; no weaponized payloads;\n",
                    "prefer false-positive over unproven critical. Secrets stay redacted.\n",
                    "When done, summarize confirmed risks with file:line and stop calling tools."
                ),
            ),
            RoleKind::Scout => (
                true,
                12,
                concat!(
                    "You are Bugbee Scout (read-only breadth agent).\n",
                    "Sweep configs, auth, secrets, dependency smells. Use grep/glob/read.\n",
                    "Report candidates with path:line. Do not edit. Defense only."
                ),
            ),
            RoleKind::TaintAnalyst => (
                true,
                16,
                concat!(
                    "You are Bugbee Taint Analyst.\n",
                    "Prove source→sink dataflow for high/critical findings.\n",
                    "Read code around each finding; add_evidence with concrete traces.\n",
                    "Reject claims without a path. Defense only."
                ),
            ),
            RoleKind::AdversarialReviewer => (
                true,
                12,
                concat!(
                    "You are Bugbee Adversarial Reviewer (OpenCode `plan` / read-only).\n",
                    "Your job is to KILL weak findings. Mark false_positive when evidence is thin.\n",
                    "Confirm only when a skilled attacker path is clear. Defense only."
                ),
            ),
            RoleKind::Patchsmith => (
                false,
                16,
                concat!(
                    "You are Bugbee Patchsmith.\n",
                    "Propose minimal safe fixes for confirmed findings. Prefer smallest diff.\n",
                    "Never introduce exploit code. Explain residual risk. Defense only."
                ),
            ),
            RoleKind::Recon => (
                true,
                10,
                "You are ReconAgent. Map attack surface, auth flows, API specs. Defense only.",
            ),
            RoleKind::Hunter => (
                false,
                20,
                "You are HunterAgent. Run Carlini loops: hypothesize → test → observe → refine. Defense only.",
            ),
            RoleKind::Symbolic => (
                true,
                16,
                "You are SymbolicAgent. Verifiable taint/paths only. No hallucinations.",
            ),
            RoleKind::Prover => (
                true,
                12,
                "You are ProverAgent. Verify claims with local evidence. Never attack unauthorized systems.",
            ),
            RoleKind::Chain => (
                true,
                12,
                "You are ChainAgent. Build kill chains on the Attack Knowledge Graph.",
            ),
            RoleKind::Scribe => (
                true,
                8,
                "You are ScribeAgent. Write bounty-quality reports with fix guidance.",
            ),
        };
        Self {
            kind,
            system_prompt: system_prompt.into(),
            read_only,
            max_steps,
        }
    }

    pub fn hunt_mode() -> Self {
        Self::builtin(RoleKind::HuntLead)
    }

    pub fn review_mode() -> Self {
        Self::builtin(RoleKind::AdversarialReviewer)
    }
}
