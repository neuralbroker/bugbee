//! Neuro-symbolic adjudication types (NSAE Stage 4 inputs/outputs).
//!
//! Defense-only: verdicts describe authorized code assessment claims.

use serde::{Deserialize, Serialize};

/// Symbolic engine strength on a candidate path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SymbolicVerdict {
    /// Clear source→sink / structural proof.
    Strong,
    /// Partial signal; noisy match.
    Noisy,
    /// No symbolic support.
    #[default]
    None,
}

impl SymbolicVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            SymbolicVerdict::Strong => "strong",
            SymbolicVerdict::Noisy => "noisy",
            SymbolicVerdict::None => "none",
        }
    }
}

/// Final adjudicated state after NSAE matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdjudicationState {
    /// Not yet adjudicated.
    #[default]
    Pending,
    /// Matrix says vulnerable (surface to human/queue).
    Vulnerable,
    /// Matrix says safe / drop as noise.
    Safe,
    /// Needs more evidence (Carlini / human).
    Inconclusive,
}

impl AdjudicationState {
    pub fn as_str(self) -> &'static str {
        match self {
            AdjudicationState::Pending => "pending",
            AdjudicationState::Vulnerable => "vulnerable",
            AdjudicationState::Safe => "safe",
            AdjudicationState::Inconclusive => "inconclusive",
        }
    }
}

/// Neural hypothesis strength (from model confidence or engine proxy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NeuralSignal {
    Strong,
    Weak,
    #[default]
    None,
}

impl NeuralSignal {
    pub fn from_confidence(c: f32) -> Self {
        if c >= 0.75 {
            NeuralSignal::Strong
        } else if c >= 0.4 {
            NeuralSignal::Weak
        } else {
            NeuralSignal::None
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            NeuralSignal::Strong => "strong",
            NeuralSignal::Weak => "weak",
            NeuralSignal::None => "none",
        }
    }
}

/// LeanGuard-style asymmetric adjudication matrix (adapted for AppSec).
///
/// Research basis: pure LLM recall is precision-biased; symbolic can be noisy;
/// the matrix recovers high recall without mutual hallucination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjudicationResult {
    pub state: AdjudicationState,
    pub priority: u8,
    pub reason: String,
    pub kill_chain_candidate: bool,
}

pub fn adjudicate(symbolic: SymbolicVerdict, neural: NeuralSignal) -> AdjudicationResult {
    use AdjudicationState::*;
    use NeuralSignal as N;
    use SymbolicVerdict as S;

    match (symbolic, neural) {
        // Symbolic STRONG + neural NONE → VULNERABLE (override neural FN)
        (S::Strong, N::None) => AdjudicationResult {
            state: Vulnerable,
            priority: 90,
            reason: "symbolic strong overrides neural silence (FN protection)".into(),
            kill_chain_candidate: true,
        },
        // Symbolic STRONG + any neural → VULNERABLE high priority
        (S::Strong, N::Strong) | (S::Strong, N::Weak) => AdjudicationResult {
            state: Vulnerable,
            priority: 95,
            reason: "symbolic strong with neural support".into(),
            kill_chain_candidate: true,
        },
        // Symbolic NOISY + neural NONE → SAFE (avoid mutual hallucination)
        (S::Noisy, N::None) => AdjudicationResult {
            state: Safe,
            priority: 10,
            reason: "noisy symbolic without neural → drop".into(),
            kill_chain_candidate: false,
        },
        // Symbolic NOISY + neural STRONG → INCONCLUSIVE (needs prove)
        (S::Noisy, N::Strong) => AdjudicationResult {
            state: Inconclusive,
            priority: 60,
            reason: "noisy symbolic + strong neural → prove".into(),
            kill_chain_candidate: false,
        },
        (S::Noisy, N::Weak) => AdjudicationResult {
            state: Inconclusive,
            priority: 40,
            reason: "noisy dual-weak signal".into(),
            kill_chain_candidate: false,
        },
        // Both NONE → SAFE
        (S::None, N::None) => AdjudicationResult {
            state: Safe,
            priority: 0,
            reason: "no signal".into(),
            kill_chain_candidate: false,
        },
        // Neural only
        (S::None, N::Strong) => AdjudicationResult {
            state: Inconclusive,
            priority: 55,
            reason: "neural-only strong → needs symbolic prove".into(),
            kill_chain_candidate: false,
        },
        (S::None, N::Weak) => AdjudicationResult {
            state: Safe,
            priority: 15,
            reason: "neural-only weak → suppress".into(),
            kill_chain_candidate: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbolic_strong_beats_neural_silence() {
        let r = adjudicate(SymbolicVerdict::Strong, NeuralSignal::None);
        assert_eq!(r.state, AdjudicationState::Vulnerable);
        assert!(r.kill_chain_candidate);
    }

    #[test]
    fn noisy_without_neural_is_safe() {
        let r = adjudicate(SymbolicVerdict::Noisy, NeuralSignal::None);
        assert_eq!(r.state, AdjudicationState::Safe);
    }

    #[test]
    fn dual_none_safe() {
        let r = adjudicate(SymbolicVerdict::None, NeuralSignal::None);
        assert_eq!(r.state, AdjudicationState::Safe);
    }
}
