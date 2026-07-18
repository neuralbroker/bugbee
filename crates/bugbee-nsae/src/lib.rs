//! Neuro-Symbolic Adjudication Engine (NSAE) — the moat.
//!
//! Every finding must pass structural + matrix adjudication before surfacing.
//! Defense-only: analyzes authorized source; does not attack live systems.

mod ast;
mod hypothesis;
mod matrix;
mod prover;
mod translator;

pub use ast::{extract_structural_signals, StructuralSlice};
pub use hypothesis::{hypothesize, NeuralHypothesis};
pub use matrix::{adjudicate_all, adjudicate_finding};
pub use prover::{verify_finding, StaticProver};
pub use translator::{translate_hypothesis, FormalScript};
