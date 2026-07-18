//! Provider-agnostic language model clients.
//!
//! Network I/O lives here only. Callers must run [`bugbee_core::Redactor`]
//! on user/repo content before sending messages.

mod anthropic;
mod openai_compat;
mod types;

pub use anthropic::AnthropicClient;
pub use openai_compat::OpenAiCompatClient;
pub use types::{
    ChatMessage, ChatRequest, ChatResponse, Role, ToolCall, ToolCallFunction, ToolFunctionDef,
    ToolSpec,
};

use async_trait::async_trait;
use bugbee_core::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderProtocol {
    AnthropicMessages,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Copy)]
struct ProviderDefaults {
    canonical_name: &'static str,
    base_url: &'static str,
    api_key_env: &'static str,
    default_model: &'static str,
    protocol: ProviderProtocol,
}

/// Environment variable normally used by a provider. `None` means no key is
/// needed (for example, a local Ollama instance).
pub fn default_api_key_env(provider: &str) -> Option<&'static str> {
    let env = provider_defaults(provider).api_key_env;
    (!env.is_empty()).then_some(env)
}

/// Provider names which Bugbee configures without a custom base URL.
pub fn supported_providers() -> &'static [&'static str] {
    &[
        "openai",
        "anthropic",
        "xai",
        "grok",
        "kimi",
        "moonshot",
        "zai",
        "z-ai",
        "deepseek",
        "openrouter",
        "ollama",
        "local",
        "huggingface",
        "hf",
        "custom",
    ]
}

fn provider_defaults(provider: &str) -> ProviderDefaults {
    match provider.to_ascii_lowercase().as_str() {
        "anthropic" | "claude" => ProviderDefaults {
            canonical_name: "anthropic",
            base_url: "https://api.anthropic.com/v1",
            api_key_env: "ANTHROPIC_API_KEY",
            default_model: "claude-3-5-haiku-latest",
            protocol: ProviderProtocol::AnthropicMessages,
        },
        "xai" | "grok" => ProviderDefaults {
            canonical_name: "xai",
            base_url: "https://api.x.ai/v1",
            api_key_env: "XAI_API_KEY",
            default_model: "grok-3-mini",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "ollama" | "local" => ProviderDefaults {
            canonical_name: "ollama",
            base_url: "http://127.0.0.1:11434/v1",
            api_key_env: "",
            default_model: "qwen2.5-coder:7b",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "openrouter" => ProviderDefaults {
            canonical_name: "openrouter",
            base_url: "https://openrouter.ai/api/v1",
            api_key_env: "OPENROUTER_API_KEY",
            default_model: "openai/gpt-4o-mini",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "deepseek" => ProviderDefaults {
            canonical_name: "deepseek",
            base_url: "https://api.deepseek.com/v1",
            api_key_env: "DEEPSEEK_API_KEY",
            default_model: "deepseek-chat",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "kimi" | "moonshot" => ProviderDefaults {
            canonical_name: "kimi",
            base_url: "https://api.moonshot.ai/v1",
            api_key_env: "MOONSHOT_API_KEY",
            default_model: "kimi-k2.5",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "zai" | "z-ai" | "glm" => ProviderDefaults {
            canonical_name: "zai",
            base_url: "https://api.z.ai/api/paas/v4",
            api_key_env: "ZAI_API_KEY",
            default_model: "glm-4.7",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        "huggingface" | "hf" => ProviderDefaults {
            canonical_name: "huggingface",
            base_url: "https://router.huggingface.co/v1",
            api_key_env: "HF_TOKEN",
            default_model: "openai/gpt-oss-120b:fastest",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
        // `custom` and unknown values retain OpenAI compatibility so a
        // self-hosted gateway can be used with an explicit `base_url`.
        _ => ProviderDefaults {
            canonical_name: "openai",
            base_url: "https://api.openai.com/v1",
            api_key_env: "OPENAI_API_KEY",
            default_model: "gpt-4o-mini",
            protocol: ProviderProtocol::OpenAiCompatible,
        },
    }
}

/// Trait every model backend implements.
#[async_trait]
pub trait LlmClient: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
}

/// Build a client from environment / config snippets.
pub fn from_env(
    provider: Option<&str>,
    model: Option<&str>,
    base_url: Option<&str>,
    api_key_env: Option<&str>,
) -> Result<Box<dyn LlmClient>> {
    let requested_provider = provider.unwrap_or("openai");
    let defaults = provider_defaults(requested_provider);
    let model = model.unwrap_or(defaults.default_model).to_string();
    let base = base_url.unwrap_or(defaults.base_url).to_string();
    let key_env = api_key_env.unwrap_or(defaults.api_key_env);
    let api_key = if key_env.is_empty() {
        String::new()
    } else {
        std::env::var(key_env).unwrap_or_default()
    };

    match defaults.protocol {
        ProviderProtocol::AnthropicMessages => {
            Ok(Box::new(AnthropicClient::new(model, base, api_key)))
        }
        ProviderProtocol::OpenAiCompatible => Ok(Box::new(OpenAiCompatClient::new(
            defaults.canonical_name.to_string(),
            model,
            base,
            api_key,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_known_provider_defaults() {
        let kimi = provider_defaults("moonshot");
        assert_eq!(kimi.canonical_name, "kimi");
        assert_eq!(kimi.base_url, "https://api.moonshot.ai/v1");
        assert_eq!(default_api_key_env("hf"), Some("HF_TOKEN"));
        assert_eq!(default_api_key_env("ollama"), None);
    }

    #[test]
    fn chooses_native_anthropic_protocol() {
        assert_eq!(
            provider_defaults("anthropic").protocol,
            ProviderProtocol::AnthropicMessages
        );
    }
}
