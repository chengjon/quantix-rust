//! Core types for LLM interactions
//!
//! Unified types that work across all LLM providers (OpenAI, DeepSeek, Gemini, etc.)

use serde::{Deserialize, Serialize};

/// A single tool call requested by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Name of the tool to invoke
    pub name: String,
    /// Arguments to pass to the tool
    pub arguments: serde_json::Value,
    /// Thought signature (for models with reasoning)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

/// Normalized response from any LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// Text response (final answer)
    pub content: Option<String>,
    /// Tool calls to execute
    pub tool_calls: Vec<ToolCall>,
    /// Chain-of-thought from thinking models (DeepSeek-R1, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Token usage information
    pub usage: TokenUsage,
    /// Provider name (openai, deepseek, gemini, etc.)
    pub provider: String,
    /// Full model name used (e.g., deepseek/deepseek-chat)
    pub model: String,
}

impl Default for LLMResponse {
    fn default() -> Self {
        Self {
            content: None,
            tool_calls: Vec::new(),
            reasoning_content: None,
            usage: TokenUsage::default(),
            provider: String::new(),
            model: String::new(),
        }
    }
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Message role in conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Tool calls (for assistant messages)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// Tool call ID (for tool response messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Reasoning content (for thinking models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

impl Message {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Create an assistant message with tool calls
    pub fn assistant_with_tools(content: Option<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.unwrap_or_default(),
            tool_calls,
            tool_call_id: None,
            reasoning_content: None,
        }
    }

    /// Create a tool response message
    pub fn tool_response(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: Some(tool_call_id.into()),
            reasoning_content: None,
        }
    }
}

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

/// Configuration for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMCallOptions {
    /// Temperature (0.0 - 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Enable thinking mode (for supported models)
    #[serde(default)]
    pub enable_thinking: bool,
}

fn default_temperature() -> f32 {
    0.7
}

fn default_timeout() -> u64 {
    60
}

impl Default for LLMCallOptions {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            max_tokens: None,
            timeout_secs: default_timeout(),
            enable_thinking: false,
        }
    }
}

/// Supported LLM providers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LLMProvider {
    OpenAI,
    DeepSeek,
    Gemini,
    Anthropic,
    Ollama,
    Custom,
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::DeepSeek => write!(f, "deepseek"),
            Self::Gemini => write!(f, "gemini"),
            Self::Anthropic => write!(f, "anthropic"),
            Self::Ollama => write!(f, "ollama"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for LLMProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" | "gpt" => Ok(Self::OpenAI),
            "deepseek" => Ok(Self::DeepSeek),
            "gemini" | "google" => Ok(Self::Gemini),
            "anthropic" | "claude" => Ok(Self::Anthropic),
            "ollama" | "local" => Ok(Self::Ollama),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Unknown LLM provider: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_message_creation() {
        let sys = Message::system("You are a helpful assistant.");
        assert_eq!(sys.role, MessageRole::System);

        let user = Message::user("Hello!");
        assert_eq!(user.role, MessageRole::User);

        let assistant = Message::assistant("Hi there!");
        assert_eq!(assistant.role, MessageRole::Assistant);
    }

    #[test]
    fn test_provider_parsing() {
        assert_eq!(
            LLMProvider::from_str("openai").unwrap(),
            LLMProvider::OpenAI
        );
        assert_eq!(
            LLMProvider::from_str("DeepSeek").unwrap(),
            LLMProvider::DeepSeek
        );
        assert_eq!(
            LLMProvider::from_str("gemini").unwrap(),
            LLMProvider::Gemini
        );
    }
}
