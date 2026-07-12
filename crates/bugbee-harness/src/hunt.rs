use std::path::{Path, PathBuf};

use bugbee_core::scoring::dual_review_decision;
use bugbee_core::{
    BugbeeConfig, DualReviewDecision, Finding, FindingStatus, FindingStore, Review, ReviewBy,
};
use bugbee_engine::HuntEngine;
use bugbee_index::Indexer;
use bugbee_providers::types::ChatMessage;
use bugbee_providers::InferenceGateway;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::permissions::PermissionPolicy;
use crate::prompts::{system_for_agent, CONSTITUTION};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuntReport {
    pub root: String,
    pub files_indexed: usize,
    pub findings: usize,
    pub auto_confirmed: usize,
    pub human_queue: usize,
    pub dropped: usize,
    pub duration_ms: u128,
}

pub struct HuntCampaign {
    pub root: PathBuf,
    pub config: BugbeeConfig,
    pub rules_dirs: Vec<PathBuf>,
    pub auto_approve: bool,
    pub use_llm_review: bool,
}

impl HuntCampaign {
    pub fn new(root: impl AsRef<Path>, config: BugbeeConfig) -> Self {
        let root = root.as_ref().to_path_buf();
        let mut rules_dirs = vec![root.join("rules")];
        // Prefer configured packs under rules/<pack> when present.
        for pack in &config.hunt.packs {
            let dir = root.join("rules").join(pack);
            if dir.exists() {
                rules_dirs.push(dir);
            }
        }
        // Workspace checkout packs (development / monorepo use).
        if let Ok(cwd) = std::env::current_dir() {
            rules_dirs.push(cwd.join("rules"));
            for pack in &config.hunt.packs {
                let dir = cwd.join("rules").join(pack);
                if dir.exists() {
                    rules_dirs.push(dir);
                }
            }
        }
        Self {
            root,
            config,
            rules_dirs,
            auto_approve: false,
            use_llm_review: false,
        }
    }

    pub async fn run(&self, store: &FindingStore) -> anyhow::Result<HuntReport> {
        let started = std::time::Instant::now();
        info!(root = %self.root.display(), "indexing");
        let index = Indexer::new(&self.root).build()?;
        info!(files = index.file_count(), "indexed");

        let engine =
            HuntEngine::with_rules_dirs(&self.rules_dirs, self.config.brs_weights.clone())?;
        let mut findings = engine.run(&index)?;

        let mut auto_confirmed = 0usize;
        let mut human_queue = 0usize;
        let mut dropped = 0usize;
        let mut observed_ids = Vec::new();

        // Optional LLM adversarial review when a model is configured
        let gateway = if self.use_llm_review {
            InferenceGateway::from_config(self.config.clone()).ok()
        } else {
            None
        };

        let (theta_high, theta_low) = self.config.hunt.effective_thresholds();

        for f in &mut findings {
            let mut auto_ok = f.ecs >= 0.75 && f.confidence >= 0.8 && f.brs >= theta_high;

            if let Some(gw) = &gateway {
                if let Ok(review_text) = self.llm_adversarial_review(gw, f).await {
                    let stands = review_text.to_lowercase().contains("stands")
                        && !review_text.to_lowercase().contains("false positive");
                    f.reviews.push(Review {
                        by: ReviewBy::Auto,
                        verdict: if stands {
                            "confirm".into()
                        } else {
                            "needs_human".into()
                        },
                        rationale: review_text.chars().take(2000).collect(),
                        ts: Utc::now(),
                    });
                    auto_ok = auto_ok && stands;
                }
            }

            let decision = dual_review_decision(
                auto_ok && !self.config.hunt.require_human_for_auto,
                f.brs,
                f.ecs,
                theta_high,
                theta_low,
            );

            match decision {
                DualReviewDecision::AutoConfirm => {
                    f.status = FindingStatus::Confirmed;
                    auto_confirmed += 1;
                }
                DualReviewDecision::Drop => {
                    dropped += 1;
                    continue;
                }
                DualReviewDecision::HumanQueue => {
                    f.status = FindingStatus::New;
                    human_queue += 1;
                }
            }

            f.recompute_scores(&self.config.brs_weights);
            store.upsert_observation(f)?;
            observed_ids.push(f.id);
        }

        store.prune_unreviewed_except(&observed_ids)?;

        let report = HuntReport {
            root: self.root.display().to_string(),
            files_indexed: index.file_count(),
            findings: observed_ids.len(),
            auto_confirmed,
            human_queue,
            dropped,
            duration_ms: started.elapsed().as_millis(),
        };
        Ok(report)
    }

    async fn llm_adversarial_review(
        &self,
        gw: &InferenceGateway,
        f: &Finding,
    ) -> bugbee_core::Result<String> {
        let sys = system_for_agent("review");
        // Locations/snippets are already redacted at detection time; keep the
        // payload minimal so model-bound content never includes raw secrets.
        let locations: Vec<String> = f
            .locations
            .iter()
            .map(|l| format!("{}:{}-{} ({:?})", l.file, l.start_line, l.end_line, l.role))
            .collect();
        let user = format!(
            "Finding to review:\nTitle: {}\nSeverity: {:?}\nBRS: {:.1} ECS: {:.2}\nDesc: {}\nRule: {}\nTraces: {}\nLocations: {}\n\nFalsify or say stands.",
            f.title,
            f.severity,
            f.brs,
            f.ecs,
            f.description,
            f.evidence.rule_id.as_deref().unwrap_or("-"),
            f.evidence.traces.join(" | "),
            locations.join("; ")
        );
        let resp = gw
            .chat_role(
                "review",
                vec![ChatMessage::system(sys), ChatMessage::user(user)],
            )
            .await?;
        Ok(resp.content)
    }

    pub async fn ask(
        &self,
        gw: &InferenceGateway,
        question: &str,
        role: &str,
    ) -> bugbee_core::Result<String> {
        let _policy = PermissionPolicy::from_config(&self.config.permissions);
        let index = Indexer::new(&self.root).build()?;
        let map = index.repomap(15);
        let map_txt: String = map
            .iter()
            .map(|f| format!("- {} ({})", f.path, f.lang.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        let sys = format!(
            "{}\n\nProject root: {}\nRepo map:\n{}\n{}",
            CONSTITUTION,
            self.root.display(),
            map_txt,
            system_for_agent(role)
        );
        let resp = gw
            .chat_role(
                role,
                vec![ChatMessage::system(sys), ChatMessage::user(question)],
            )
            .await?;
        Ok(resp.content)
    }
}
