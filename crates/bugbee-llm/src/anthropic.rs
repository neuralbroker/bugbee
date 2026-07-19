//! Native Anthropic Messages API client.
//!
//! Anthropic's API is not an OpenAI Chat Completions endpoint: system prompts,
//! tool definitions, tool calls, and tool results all have different shapes.
//! Keeping this adapter separate avoids silently sending invalid requests when a
//! user selects `provider = "anthropic"`.

use async_trait::async_trait;
use bugbee_core::{Error, Result};
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use tracing::debug;

use crate::types::{ChatMessage, ChatRequest, ChatResponse, Role, ToolCall, ToolCallFunction};
use crate::LlmClient;

const API_VERSION: &str = "2023-06-01";

/// Client for Anthropic's native `POST /v1/messages` API.
pub struct AnthropicClient {
    model: String,
    base_url: String,
    api_key: String,
    http: Client,
}

impl AnthropicClient {
    pub fn new(model: String, base_url: String, api_key: String) -> Self {
        Self {
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
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Value>>,
}

fn to_api_request(model: &str, request: &ChatRequest) -> ApiRequest {
    let system = request
        .messages
        .iter()
        .filter(|message| message.role == Role::System)
        .filter_map(|message| message.content.as_deref())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    let messages = request
        .messages
        .iter()
        .filter(|message| message.role != Role::System)
        .map(message_to_json)
        .collect();

    let tools = request.tools.as_ref().map(|tools| {
        tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.function.name,
                    "description": tool.function.description,
                    "input_schema": tool.function.parameters,
                })
            })
            .collect()
    });

    ApiRequest {
        model: model.to_string(),
        max_tokens: request.max_tokens.unwrap_or(4096),
        temperature: request.temperature.unwrap_or(0.2),
        messages,
        system: (!system.is_empty()).then_some(system),
        tools,
    }
}

fn message_to_json(message: &ChatMessage) -> Value {
    let content = message.content.clone().unwrap_or_default();
    match message.role {
        Role::User => json!({ "role": "user", "content": content }),
        Role::Assistant => {
            let mut blocks = Vec::new();
            if !content.is_empty() {
                blocks.push(json!({ "type": "text", "text": content }));
            }
            if let Some(calls) = &message.tool_calls {
                for call in calls {
                    let input = serde_json::from_str(&call.function.arguments)
                        .unwrap_or_else(|_| json!({ "raw_arguments": call.function.arguments }));
                    blocks.push(json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.function.name,
                        "input": input,
                    }));
                }
            }
            json!({ "role": "assistant", "content": blocks })
        }
        Role::Tool => json!({
            "role": "user",
            "content": [{
                "type": "tool_result",
                "tool_use_id": message.tool_call_id.clone().unwrap_or_else(|| "unknown".into()),
                "content": content,
                "is_error": false,
            }],
        }),
        // System messages are lifted to the top-level `system` field.
        Role::System => json!({ "role": "user", "content": content }),
    }
}

fn parse_response(value: Value, fallback_model: &str) -> Result<ChatResponse> {
    let model = value
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or(fallback_model)
        .to_string();
    let finish_reason = value
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let mut content = String::new();
    let mut tool_calls = Vec::new();
    for block in value
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Provider("Anthropic response missing content blocks".into()))?
    {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    content.push_str(text);
                }
            }
            Some("tool_use") => {
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::Provider("Anthropic tool call missing id".into()))?;
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::Provider("Anthropic tool call missing name".into()))?;
                let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
                tool_calls.push(ToolCall {
                    id: id.to_string(),
                    kind: "function".into(),
                    function: ToolCallFunction {
                        name: name.to_string(),
                        arguments: input.to_string(),
                    },
                });
            }
            _ => {}
        }
    }

    Ok(ChatResponse {
        content,
        model,
        finish_reason,
        tool_calls,
    })
}

#[async_trait]
impl LlmClient for AnthropicClient {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        if self.api_key.is_empty() {
            return Err(Error::Provider(
                "ANTHROPIC_API_KEY is not set (or configure provider.api_key_env)".into(),
            ));
        }
        let url = format!("{}/messages", self.base_url);
        let body = to_api_request(&self.model, &request);
        debug!(url = %url, model = %self.model, "Anthropic Messages request");

        let response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|error| Error::Provider(format!("request failed: {error}")))?;
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|error| Error::Provider(format!("read body: {error}")))?;
        if !status.is_success() {
            let preview: String = text.chars().take(200).collect();
            return Err(Error::Provider(format!(
                "provider returned HTTP {status} — check your API key, model name, and account balance. Body: {preview}",
            )));
        }

        let value: Value = serde_json::from_str(&text).map_err(|error| {
            let preview: String = text.chars().take(200).collect();
            Error::Provider(format!(
                "failed to parse provider response: {error}. Raw response: {preview}",
            ))
        })?;
        parse_response(value, &self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChatMessage, ChatRequest, ToolCall, ToolCallFunction};

    #[test]
    fn maps_system_tools_and_tool_results_to_messages_api() {
        let call = ToolCall {
            id: "call_1".into(),
            kind: "function".into(),
            function: ToolCallFunction {
                name: "grep".into(),
                arguments: r#"{"pattern":"eval"}"#.into(),
            },
        };
        let req = ChatRequest::new(vec![
            ChatMessage::system("defensive only"),
            ChatMessage::user("inspect"),
            ChatMessage::assistant_tools(None, vec![call]),
            ChatMessage::tool_result("call_1", "grep", "app.py:10"),
        ]);

        let body = serde_json::to_value(to_api_request("claude-test", &req)).unwrap();
        assert_eq!(body["system"], "defensive only");
        assert_eq!(body["messages"][1]["content"][0]["type"], "tool_use");
        assert_eq!(body["messages"][2]["content"][0]["type"], "tool_result");
    }

    #[test]
    fn parses_text_and_tool_blocks() {
        let response = json!({
            "model": "claude-test",
            "stop_reason": "tool_use",
            "content": [
                {"type": "text", "text": "I will inspect."},
                {"type": "tool_use", "id": "tool_1", "name": "read", "input": {"path": "src/main.rs"}}
            ]
        });
        let parsed = parse_response(response, "fallback").unwrap();
        assert_eq!(parsed.content, "I will inspect.");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].function.name, "read");
    }
}
