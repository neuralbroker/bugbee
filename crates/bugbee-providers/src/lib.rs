//! Model-agnostic LLM providers.
//! Users can use ANY model via OpenAI-compatible endpoints or native adapters.

pub mod gateway;
pub mod openai_compat;
pub mod types;

pub use gateway::InferenceGateway;
pub use openai_compat::OpenAiCompatProvider;
pub use types::{ChatMessage, ChatRequest, ChatResponse, Role, ToolCall, ToolDef};
