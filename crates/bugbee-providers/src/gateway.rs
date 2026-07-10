use std::collections::HashMap;
use std::sync::Arc;

use bugbee_core::{BugbeeConfig, BugbeeError, Result};

use crate::openai_compat::{LlmProvider, OpenAiCompatProvider};
use crate::types::{ChatMessage, ChatRequest, ChatResponse};

/// Routes chat to any configured provider. Model-agnostic: any provider/model the user configures.
pub struct InferenceGateway {
    config: BugbeeConfig,
    clients: HashMap<String, Arc<dyn LlmProvider>>,
}

impl InferenceGateway {
    pub fn from_config(config: BugbeeConfig) -> Result<Self> {
        let mut clients: HashMap<String, Arc<dyn LlmProvider>> = HashMap::new();
        for (id, pcfg) in &config.providers {
            if !config.provider_allowlist.is_empty()
                && !config.provider_allowlist.iter().any(|a| a == id)
            {
                continue;
            }
            // Prefer resolving key; skip providers without keys (except local-style)
            let key = match config.resolve_api_key(id) {
                Ok(k) => k,
                Err(_) => {
                    if pcfg.api_key.as_deref() == Some("ollama")
                        || pcfg.base_url.contains("127.0.0.1")
                        || pcfg.base_url.contains("localhost")
                    {
                        pcfg.api_key.clone().unwrap_or_else(|| "local".into())
                    } else {
                        continue;
                    }
                }
            };
            if pcfg.protocol != "openai_compat" {
                tracing::warn!(
                    provider = %id,
                    protocol = %pcfg.protocol,
                    "provider skipped: native protocol adapter is not implemented"
                );
                continue;
            }

            // Covers xAI, DeepSeek, Qwen, Kimi, GLM, Ollama, OpenRouter, and custom gateways.
            let client = OpenAiCompatProvider::new(pcfg, key)?;
            clients.insert(id.clone(), Arc::new(client));
        }
        Ok(Self { config, clients })
    }

    pub fn available_providers(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub async fn chat_role(&self, role: &str, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        let model_ref = match role {
            "hunt" => self.config.inference.hunt.as_deref(),
            "scout" => self.config.inference.scout.as_deref(),
            "review" => self.config.inference.review.as_deref(),
            "patch" => self.config.inference.patch.as_deref(),
            _ => None,
        }
        .or(self.config.inference.hunt.as_deref())
        .ok_or_else(|| {
            BugbeeError::Config(
                "no model configured for inference — set inference.hunt = \"provider/model\" or use any model via /connect"
                    .into(),
            )
        })?;

        self.chat_model(model_ref, messages).await
    }

    /// Call any model: "provider/model_id". Platform does not restrict which model ids are valid.
    pub async fn chat_model(
        &self,
        model_ref: &str,
        messages: Vec<ChatMessage>,
    ) -> Result<ChatResponse> {
        let (provider_id, model) = self.config.parse_model_ref(model_ref)?;
        let client = self.clients.get(&provider_id).ok_or_else(|| {
            BugbeeError::Provider(format!(
                "provider '{provider_id}' not connected (set API key / base_url). Available: {:?}",
                self.available_providers()
            ))
        })?;

        let req = ChatRequest {
            model,
            messages,
            temperature: self.config.inference.temperature,
            max_tokens: self.config.inference.max_tokens,
            tools: vec![],
        };
        client.chat(req).await
    }

    pub async fn list_models(&self, provider_id: &str) -> Result<Vec<String>> {
        let client = self
            .clients
            .get(provider_id)
            .ok_or_else(|| BugbeeError::NotFound(format!("provider '{provider_id}'")))?;
        client.list_models().await
    }

    pub fn config(&self) -> &BugbeeConfig {
        &self.config
    }
}
