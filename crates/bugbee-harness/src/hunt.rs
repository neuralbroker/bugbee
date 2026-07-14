use std::path::{Path, PathBuf};

use bugbee_core::scoring::dual_review_decision;
use bugbee_core::{
    AiReview, BugbeeConfig, DualReviewDecision, Finding, FindingStatus, FindingStore, Review,
    ReviewBy,
};
use bugbee_engine::HuntEngine;
use bugbee_index::Indexer;
use bugbee_providers::types::ChatMessage;
use bugbee_providers::InferenceGateway;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::permissions::PermissionPolicy;
use crate::prompts::system_for_agent;

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
                    // Enrichment is deliberately gated behind adversarial
                    // review. This keeps a model from decorating a weak
                    // candidate with speculative remediation advice.
                    if stands {
                        if let Ok(ai_review) = self.llm_deep_review(gw, f).await {
                            f.ai_review = ai_review;
                        }
                    }
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

    async fn llm_deep_review(
        &self,
        gw: &InferenceGateway,
        f: &Finding,
    ) -> bugbee_core::Result<AiReview> {
        let locations = finding_context(f);
        let user = format!(
            "Produce a concise, evidence-bound security review for this confirmed candidate.\n\
             Use these exact labels, one short paragraph each: Summary:, Root cause:, Attack scenario:, Why it matters:, Suggested fix:, Alternatives:, Confidence:.\n\
             Only cite evidence provided below. Do not invent paths, sanitizers, exploit results, or code. Confidence must be 0.0 to 1.0.\n\n\
             Title: {}\nSeverity: {:?}\nBRS: {:.1}; ECS: {:.2}\nDescription: {}\nEvidence: {}\nLocations: {}",
            f.title,
            f.severity,
            f.brs,
            f.ecs,
            f.description,
            f.evidence.traces.join(" | "),
            locations,
        );
        let response = gw
            .chat_role(
                "review",
                vec![
                    ChatMessage::system(system_for_agent("review")),
                    ChatMessage::user(user),
                ],
            )
            .await?;
        Ok(parse_ai_review(&response.content))
    }

    /// Ask about a specific finding, with repository evidence and prior human
    /// decisions included. This is intentionally different from generic chat.
    pub async fn ask_finding(
        &self,
        gw: &InferenceGateway,
        finding: &Finding,
        question: &str,
    ) -> bugbee_core::Result<String> {
        let decisions = finding
            .reviews
            .iter()
            .map(|r| format!("{}: {}", r.verdict, r.rationale))
            .collect::<Vec<_>>()
            .join(" | ");
        let user = format!(
            "Answer the engineer's question about one Bugbee finding. Be concise, technical, and evidence-bound. \
             If the evidence is incomplete, say so. Never invent repository facts.\n\nQuestion: {question}\n\n\
             Finding: {}\nDescription: {}\nLocations: {}\nEvidence: {}\nPrior review memory: {}",
            finding.title,
            finding.description,
            finding_context(finding),
            finding.evidence.traces.join(" | "),
            decisions
        );
        Ok(gw
            .chat_role(
                "review",
                vec![
                    ChatMessage::system(system_for_agent("review")),
                    ChatMessage::user(user),
                ],
            )
            .await?
            .content)
    }

    /// Generate a proposal only. The caller stores it for review; this method
    /// has no write capability and never applies code.
    pub async fn propose_patch(
        &self,
        gw: &InferenceGateway,
        finding: &Finding,
    ) -> bugbee_core::Result<String> {
        let user = format!(
            "Propose the smallest safe remediation for this finding. Return a unified diff only when the evidence includes enough code context; otherwise explain exactly what must be changed. \
             Do not change unrelated files. Never claim the patch has been applied.\n\nFinding: {}\nDescription: {}\nLocations: {}\nEvidence: {}",
            finding.title,
            finding.description,
            finding_context(finding),
            finding.evidence.traces.join(" | ")
        );
        Ok(gw
            .chat_role(
                "patch",
                vec![
                    ChatMessage::system(system_for_agent("patch")),
                    ChatMessage::user(user),
                ],
            )
            .await?
            .content)
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
            "{}\n\nProject root: {}\nRepo map:\n{}",
            system_for_agent(role),
            self.root.display(),
            map_txt
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

fn finding_context(finding: &Finding) -> String {
    finding
        .locations
        .iter()
        .map(|location| {
            format!(
                "{}:{}-{} ({:?}){}",
                location.file,
                location.start_line,
                location.end_line,
                location.role,
                location
                    .snippet
                    .as_deref()
                    .map(|snippet| format!(": {snippet}"))
                    .unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn parse_ai_review(text: &str) -> AiReview {
    fn field(text: &str, label: &str) -> Option<String> {
        text.lines()
            .find_map(|line| line.trim().strip_prefix(label).map(str::trim))
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }
    let confidence = field(text, "Confidence:")
        .and_then(|value| value.trim_end_matches('%').parse::<f64>().ok())
        .map(|value| if value > 1.0 { value / 100.0 } else { value })
        .filter(|value| (0.0..=1.0).contains(value));
    AiReview {
        summary: field(text, "Summary:"),
        root_cause: field(text, "Root cause:"),
        attack_scenario: field(text, "Attack scenario:"),
        why_it_matters: field(text, "Why it matters:"),
        suggested_fix: field(text, "Suggested fix:"),
        alternative_fixes: field(text, "Alternatives:")
            .map(|value| {
                value
                    .split(';')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        related_finding_ids: Vec::new(),
        model_confidence: confidence,
        model: None,
        reviewed_at: Some(Utc::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_ai_review;

    #[test]
    fn parses_labeled_evidence_bound_review() {
        let review = parse_ai_review(
            "Summary: Raw input reaches a query builder.\n\
             Root cause: String concatenation.\n\
             Attack scenario: A request reaches the sink.\n\
             Why it matters: Query manipulation is possible.\n\
             Suggested fix: Use parameters.\n\
             Alternatives: Validate input; use an ORM.\n\
             Confidence: 98%",
        );
        assert_eq!(review.root_cause.as_deref(), Some("String concatenation."));
        assert_eq!(review.alternative_fixes.len(), 2);
        assert_eq!(review.model_confidence, Some(0.98));
    }
}
