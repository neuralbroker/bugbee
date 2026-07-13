use async_trait::async_trait;
use bugbee_core::{BugbeeError, ProviderConfig, Redactor, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::openai_compat::LlmProvider;
use crate::types::{ChatMessage, ChatRequest, ChatResponse, Role};

/// Native adapter for Anthropic's Messages API. Claude Code itself is a
/// separate client product; Bugbee talks directly to the supported API.
pub struct AnthropicProvider {
    base_url: String,
    api_key: String,
    headers: Vec<(String, String)>,
    redactor: Redactor,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: &ProviderConfig, api_key: String) -> Result<Self> {
        Ok(Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key,
            headers: config
                .headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            redactor: Redactor::enterprise(),
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .map_err(|e| BugbeeError::Provider(e.to_string()))?,
        })
    }

    fn redact_messages(&self, messages: &[ChatMessage]) -> Vec<ChatMessage> {
        messages
            .iter()
            .map(|message| {
                let mut message = message.clone();
                message.content = self.redactor.redact(&message.content);
                message
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct AnthropicResponse {
    id: Option<String>,
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    kind: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        let messages = self.redact_messages(&req.messages);
        let system = messages
            .iter()
            .filter(|m| matches!(m.role, Role::System))
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");
        let body_messages: Vec<_> = messages.iter().filter(|m| !matches!(m.role, Role::System)).map(|m| {
            json!({ "role": if matches!(m.role, Role::Assistant) { "assistant" } else { "user" }, "content": m.content })
        }).collect();
        let mut body = json!({ "model": req.model, "max_tokens": req.max_tokens, "temperature": req.temperature, "messages": body_messages });
        if !system.is_empty() {
            body["system"] = json!(system);
        }
        if !req.tools.is_empty() {
            body["tools"] = json!(req.tools.iter().map(|tool| json!({
                "name": tool.name, "description": tool.description, "input_schema": tool.parameters
            })).collect::<Vec<_>>());
        }
        let mut request = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body);
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        let response = request
            .send()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(BugbeeError::Provider(format!(
                "HTTP {status}: {}",
                text.chars().take(500).collect::<String>()
            )));
        }
        let response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;
        let content = response
            .content
            .into_iter()
            .filter(|block| block.kind == "text")
            .filter_map(|block| block.text)
            .collect::<Vec<_>>()
            .join("\n");
        let usage = response.usage;
        Ok(ChatResponse {
            content,
            tool_calls: vec![],
            model: response.id.unwrap_or(req.model),
            prompt_tokens: usage.as_ref().and_then(|u| u.input_tokens).unwrap_or(0),
            completion_tokens: usage.and_then(|u| u.output_tokens).unwrap_or(0),
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn redacts_messages() {
        let config = ProviderConfig {
            name: None,
            base_url: "https://api.anthropic.com".into(),
            api_key_env: None,
            api_key: None,
            models: vec![],
            headers: HashMap::new(),
            protocol: "anthropic".into(),
        };
        let provider = AnthropicProvider::new(&config, "key".into()).unwrap();
        assert!(
            !provider.redact_messages(&[ChatMessage::user("api_key=abcdefghijk")])[0]
                .content
                .contains("abcdefghijk")
        );
    }
}
