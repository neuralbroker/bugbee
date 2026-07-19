use async_trait::async_trait;
use bugbee_core::{Error, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

use crate::types::{ChatMessage, ChatRequest, ChatResponse, Role, ToolCall};
use crate::LlmClient;

/// OpenAI-compatible Chat Completions client (OpenAI, xAI, Ollama, OpenRouter, …).
pub struct OpenAiCompatClient {
    name: String,
    model: String,
    base_url: String,
    api_key: String,
    http: Client,
}

impl OpenAiCompatClient {
    pub fn new(name: String, model: String, base_url: String, api_key: String) -> Self {
        Self {
            name,
            model,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(180))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }
}

#[derive(Serialize)]
struct ApiRequest {
    model: String,
    messages: Vec<Value>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<ApiChoice>,
    model: Option<String>,
}

#[derive(Deserialize)]
struct ApiChoice {
    message: ApiMessageOwned,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ApiMessageOwned {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCall>,
}

fn message_to_json(m: &ChatMessage) -> Value {
    let mut map = serde_json::Map::new();
    let role = match m.role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    };
    map.insert("role".into(), Value::String(role.into()));
    if let Some(ref c) = m.content {
        map.insert("content".into(), Value::String(c.clone()));
    } else if m.role != Role::Assistant || m.tool_calls.is_none() {
        map.insert("content".into(), Value::String(String::new()));
    }
    if let Some(ref calls) = m.tool_calls {
        map.insert(
            "tool_calls".into(),
            serde_json::to_value(calls).unwrap_or(Value::Array(vec![])),
        );
    }
    if let Some(ref id) = m.tool_call_id {
        map.insert("tool_call_id".into(), Value::String(id.clone()));
    }
    if let Some(ref name) = m.name {
        map.insert("name".into(), Value::String(name.clone()));
    }
    Value::Object(map)
}

#[async_trait]
impl LlmClient for OpenAiCompatClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let messages: Vec<Value> = request.messages.iter().map(message_to_json).collect();

        let tools = request
            .tools
            .as_ref()
            .map(|t| serde_json::to_value(t).unwrap_or(Value::Null));

        let body = ApiRequest {
            model: self.model.clone(),
            messages,
            temperature: request.temperature.unwrap_or(0.2),
            max_tokens: request.max_tokens.unwrap_or(4096),
            tools,
            tool_choice: request.tool_choice.clone(),
        };

        debug!(
            url = %url,
            model = %self.model,
            tools = request.tools.as_ref().map(|t| t.len()).unwrap_or(0),
            "llm chat request"
        );

        let mut req = self.http.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| Error::Provider(format!("request failed: {e}")))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| Error::Provider(format!("read body: {e}")))?;

        if !status.is_success() {
            let preview: String = text.chars().take(200).collect();
            return Err(Error::Provider(format!(
                "provider returned HTTP {status} — check your API key, model name, and account balance. Body: {preview}",
            )));
        }

        let parsed: ApiResponse = serde_json::from_str(&text).map_err(|e| {
            let preview: String = text.chars().take(200).collect();
            Error::Provider(format!(
                "failed to parse provider response: {e}. Raw response: {preview}",
            ))
        })?;

        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| Error::Provider("empty choices".into()))?;

        Ok(ChatResponse {
            content: choice.message.content.unwrap_or_default(),
            model: parsed.model.unwrap_or_else(|| self.model.clone()),
            finish_reason: choice.finish_reason,
            tool_calls: choice.message.tool_calls,
        })
    }
}
