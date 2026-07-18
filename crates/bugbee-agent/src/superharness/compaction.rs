//! Context compaction — OpenCode prune + Pi keepRecentTokens.

use bugbee_llm::{ChatMessage, Role};

#[derive(Debug, Clone)]
pub struct CompactionConfig {
    pub enabled: bool,
    /// Max estimated chars in message history before compacting.
    pub max_chars: usize,
    /// Keep this many recent messages after compaction (tail).
    pub keep_recent: usize,
    /// Always preserve system message.
    pub protect_system: bool,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chars: 120_000,
            keep_recent: 24,
            protect_system: true,
        }
    }
}

fn estimate_chars(messages: &[ChatMessage]) -> usize {
    messages
        .iter()
        .map(|m| {
            m.content.as_deref().map(|c| c.len()).unwrap_or(0)
                + m.tool_calls
                    .as_ref()
                    .map(|t| {
                        t.iter()
                            .map(|c| c.function.name.len() + c.function.arguments.len())
                            .sum::<usize>()
                    })
                    .unwrap_or(0)
        })
        .sum()
}

/// Compact message history if over budget.
///
/// Strategy (OpenCode-inspired):
/// 1. Keep system message(s)
/// 2. Insert a synthetic summary user message for dropped middle
/// 3. Keep last `keep_recent` messages
pub fn compact_messages(
    messages: &[ChatMessage],
    cfg: &CompactionConfig,
) -> (Vec<ChatMessage>, bool) {
    if !cfg.enabled {
        return (messages.to_vec(), false);
    }
    let total = estimate_chars(messages);
    if total <= cfg.max_chars {
        return (messages.to_vec(), false);
    }

    let mut system: Vec<ChatMessage> = Vec::new();
    let mut rest: Vec<ChatMessage> = Vec::new();
    for m in messages {
        if cfg.protect_system && m.role == Role::System {
            system.push(m.clone());
        } else {
            rest.push(m.clone());
        }
    }

    if rest.len() <= cfg.keep_recent {
        return (messages.to_vec(), false);
    }

    let drop_count = rest.len().saturating_sub(cfg.keep_recent);
    let dropped = &rest[..drop_count];
    let kept = rest[drop_count..].to_vec();

    // Lightweight extractive summary (no extra LLM required)
    let mut summary = String::from(
        "[superharness compaction] Older turns summarized to free context.\n\
         Key fragments retained:\n",
    );
    let mut n = 0;
    for m in dropped.iter().rev() {
        if n >= 8 {
            break;
        }
        let text = m.text();
        if text.is_empty() {
            continue;
        }
        let snip: String = text.chars().take(200).collect();
        summary.push_str(&format!("- ({:?}) {snip}\n", m.role));
        n += 1;
    }
    summary.push_str(&format!(
        "… compacted {drop_count} messages (was ~{total} chars).\n"
    ));

    let mut out = system;
    out.push(ChatMessage::user(summary));
    out.extend(kept);
    (out, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compacts_when_over_budget() {
        let mut msgs = vec![ChatMessage::system("sys")];
        for i in 0..40 {
            msgs.push(ChatMessage::user(format!("msg {i} {}", "x".repeat(4000))));
        }
        let cfg = CompactionConfig {
            max_chars: 50_000,
            keep_recent: 6,
            ..Default::default()
        };
        let (out, did) = compact_messages(&msgs, &cfg);
        assert!(did);
        assert!(out.len() < msgs.len());
        assert_eq!(out[0].role, Role::System);
    }
}
