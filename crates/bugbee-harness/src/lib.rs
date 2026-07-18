//! Super Harness — gRPC server for sandboxed verification, replay, and differential analysis.
//!
//! Corresponds to `cmd/harness/` in the architecture.
//! Communicates with the Go/CLI side via Unix-domain gRPC at `/tmp/bugbee-harness.sock`.

pub mod proto {
    tonic::include_proto!("harness");
}

pub mod server;
pub mod client;
pub mod diff;
pub mod types;

pub use server::SuperHarness;
pub use client::HarnessClient;
pub use diff::DiffOracle;
pub use types::*;
