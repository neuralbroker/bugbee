use async_trait::async_trait;
use bugbee_core::{BugbeeError, ProviderConfig, Redactor, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::types::{ChatMessage, ChatRequest, ChatResponse, Role, ToolCall};

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse>;
    async fn list_models(&self) -> Result<Vec<String>>;
}

pub struct OpenAiCompatProvider {
    pub base_url: String,
    pub api_key: String,
    pub headers: Vec<(String, String)>,
    pub redactor: Redactor,
    client: Client,
}

impl OpenAiCompatProvider {
    pub fn new(config: &ProviderConfig, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;
        let headers: Vec<_> = config
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Ok(Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key,
            headers,
            redactor: Redactor::enterprise(),
            client,
        })
    }

    fn redact_messages(&self, messages: &[ChatMessage]) -> Vec<ChatMessage> {
        messages
            .iter()
            .map(|m| {
                let mut c = m.clone();
                c.content = self.redactor.redact(&c.content);
                c
            })
            .collect()
    }
}

#[derive(Serialize)]
struct OaiMessage<'a> {
    role: &'a str,
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
}

#[derive(Deserialize)]
struct OaiChatResponse {
    model: Option<String>,
    choices: Vec<OaiChoice>,
    usage: Option<OaiUsage>,
}

#[derive(Deserialize)]
struct OaiChoice {
    message: OaiRespMessage,
}

#[derive(Deserialize)]
struct OaiRespMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OaiToolCall>>,
}

#[derive(Deserialize)]
struct OaiToolCall {
    id: String,
    function: OaiFunction,
}

#[derive(Deserialize)]
struct OaiFunction {
    name: String,
    arguments: String,
}

#[derive(Deserialize)]
struct OaiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct OaiModelsResponse {
    data: Vec<OaiModel>,
}

#[derive(Deserialize)]
struct OaiModel {
    id: String,
}

fn role_str(r: Role) -> &'static str {
    match r {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatProvider {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse> {
        let messages = self.redact_messages(&req.messages);
        let oai_msgs: Vec<OaiMessage> = messages
            .iter()
            .map(|m| OaiMessage {
                role: role_str(m.role),
                content: &m.content,
                tool_call_id: m.tool_call_id.as_deref(),
            })
            .collect();

        let mut body = json!({
            "model": req.model,
            "messages": oai_msgs,
            "temperature": req.temperature,
            "max_tokens": req.max_tokens,
        });

        if !req.tools.is_empty() {
            let tools: Vec<_> = req
                .tools
                .iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tools);
        }

        let url = format!("{}/chat/completions", self.base_url);
        let mut builder = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&body);

        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BugbeeError::Provider(format!(
                "HTTP {status}: {}",
                text.chars().take(500).collect::<String>()
            )));
        }

        let parsed: OaiChatResponse = resp
            .json()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;

        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| BugbeeError::Provider("empty choices".into()))?;

        let tool_calls = choice
            .message
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|t| ToolCall {
                id: t.id,
                name: t.function.name,
                arguments: t.function.arguments,
            })
            .collect();

        let (prompt_tokens, completion_tokens) = match parsed.usage {
            Some(u) => (
                u.prompt_tokens.unwrap_or(0),
                u.completion_tokens.unwrap_or(0),
            ),
            None => (0, 0),
        };

        Ok(ChatResponse {
            content: choice.message.content.unwrap_or_default(),
            tool_calls,
            model: parsed.model.unwrap_or(req.model),
            prompt_tokens,
            completion_tokens,
        })
    }

    async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);
        let mut builder = self.client.get(&url).bearer_auth(&self.api_key);
        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }
        let resp = builder
            .send()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;
        if !resp.status().is_success() {
            return Ok(vec![]);
        }
        let parsed: OaiModelsResponse = resp
            .json()
            .await
            .map_err(|e| BugbeeError::Provider(e.to_string()))?;
        Ok(parsed.data.into_iter().map(|m| m.id).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChatMessage;
    use bugbee_core::ProviderConfig;
    use std::collections::HashMap;

    fn provider() -> OpenAiCompatProvider {
        let cfg = ProviderConfig {
            name: Some("test".into()),
            base_url: "https://example.test/v1".into(),
            api_key_env: None,
            api_key: None,
            models: vec![],
            headers: HashMap::new(),
            protocol: "openai_compat".into(),
        };
        OpenAiCompatProvider::new(&cfg, "test-key".into()).expect("provider")
    }

    #[test]
    fn redacts_secrets_before_provider_payload() {
        let p = provider();
        let messages = vec![ChatMessage::user(
            "token=github_pat_abcdefghijklmnopqrstuvwxyz_1234567890",
        )];
        let redacted = p.redact_messages(&messages);
        assert!(!redacted[0].content.contains("github_pat_abc"));
        assert!(redacted[0].content.contains("REDACTED"));
    }
}
