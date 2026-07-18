use std::path::PathBuf;
use std::sync::Arc;

use bugbee_core::{ProjectConfig, Store};
use bugbee_llm::LlmClient;
use tokio::sync::mpsc;

use crate::harness::{run_godmode, GodmodeOptions};
use crate::swarm::{run_swarm, SwarmOptions};

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Phase { name: String },
    Step { message: String },
    Finding { count: usize },
    ToolCall { name: String, args: String },
    ToolResult { name: String, ok: bool, preview: String },
    Warn { message: String },
    Done { summary: String, elapsed_ms: u128 },
    Error { message: String },
}

pub async fn run_swarm_streaming(
    root: PathBuf,
    config: ProjectConfig,
    store: Store,
    opts: SwarmOptions,
    tx: mpsc::Sender<StreamEvent>,
) {
    let _ = tx.send(StreamEvent::Phase { name: "swarm".into() }).await;

    match run_swarm(root, config, store, opts).await {
        Ok(report) => {
            let _ = tx
                .send(StreamEvent::Done {
                    summary: report.summary,
                    elapsed_ms: report.elapsed_ms,
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(StreamEvent::Error {
                    message: format!("swarm error: {e}"),
                })
                .await;
        }
    }
}

pub async fn run_godmode_streaming(
    root: PathBuf,
    config: ProjectConfig,
    store: Store,
    client: Option<Arc<dyn LlmClient>>,
    opts: GodmodeOptions,
    tx: mpsc::Sender<StreamEvent>,
) {
    let _ = tx.send(StreamEvent::Phase { name: "godmode".into() }).await;

    match run_godmode(root, config, store, client, opts).await {
        Ok(report) => {
            for ev in &report.events {
                let _ = tx
                    .send(StreamEvent::Step {
                        message: format!("{:?}: {}", ev.kind, ev.message),
                    })
                    .await;
            }
            let _ = tx
                .send(StreamEvent::Done {
                    summary: report.summary,
                    elapsed_ms: report.elapsed_ms,
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(StreamEvent::Error {
                    message: format!("godmode error: {e}"),
                })
                .await;
        }
    }
}
