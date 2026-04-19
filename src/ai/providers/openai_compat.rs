//! OpenAI-compatible API adapter
//!
//! Works with OpenAI, DeepSeek, and any OpenAI-compatible API (including local Ollama)

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::ai::adapter::{LlmAdapter, LlmConfig};
use crate::ai::types::{
    LLMCallOptions, LLMProvider, LLMResponse, Message, TokenUsage, ToolCall, ToolDefinition,
};
use crate::core::{QuantixError, Result};

/// OpenAI-compatible adapter
pub struct OpenAICompatAdapter {
    provider: LLMProvider,
    api_key: Option<String>,
    base_url: String,
    default_model: String,
    client: Client,
}

impl OpenAICompatAdapter {
    /// Create a new adapter for DeepSeek
    pub fn deepseek(config: &LlmConfig) -> Self {
        let provider_config = config.get_provider("deepseek");
        Self {
            provider: LLMProvider::DeepSeek,
            api_key: provider_config.and_then(|c| c.api_key.clone()),
            base_url: provider_config
                .and_then(|c| c.base_url.clone())
                .unwrap_or_else(|| "https://api.deepseek.com/v1".to_string()),
            default_model: "deepseek-chat".to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .unwrap(),
        }
    }

    /// Create a new adapter for OpenAI
    pub fn openai(config: &LlmConfig) -> Self {
        let provider_config = config.get_provider("openai");
        Self {
            provider: LLMProvider::OpenAI,
            api_key: provider_config.and_then(|c| c.api_key.clone()),
            base_url: provider_config
                .and_then(|c| c.base_url.clone())
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            default_model: "gpt-4o".to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .unwrap(),
        }
    }

    /// Create a new adapter for Ollama (local)
    pub fn ollama(config: &LlmConfig) -> Self {
        let provider_config = config.get_provider("ollama");
        Self {
            provider: LLMProvider::Ollama,
            api_key: None, // Ollama doesn't need API key
            base_url: provider_config
                .and_then(|c| c.base_url.clone())
                .unwrap_or_else(|| "http://localhost:11434/v1".to_string()),
            default_model: "llama3".to_string(),
            client: Client::builder()
                .timeout(Duration::from_secs(config.timeout_secs))
                .build()
                .unwrap(),
        }
    }

    /// Create a custom adapter with specified configuration
    pub fn custom(
        provider: LLMProvider,
        base_url: String,
        api_key: Option<String>,
        default_model: String,
        timeout_secs: u64,
    ) -> Self {
        Self {
            provider,
            api_key,
            base_url,
            default_model,
            client: Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .build()
                .unwrap(),
        }
    }

    /// Convert messages to OpenAI format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    crate::ai::types::MessageRole::System => "system",
                    crate::ai::types::MessageRole::User => "user",
                    crate::ai::types::MessageRole::Assistant => "assistant",
                    crate::ai::types::MessageRole::Tool => "tool",
                };

                if !msg.tool_calls.is_empty() {
                    // Assistant message with tool calls
                    OpenAIMessage {
                        role: role.to_string(),
                        content: if msg.content.is_empty() {
                            None
                        } else {
                            Some(msg.content.clone())
                        },
                        tool_calls: Some(
                            msg.tool_calls
                                .iter()
                                .map(|tc| OpenAIToolCall {
                                    id: tc.id.clone(),
                                    typ: "function".to_string(),
                                    function: OpenAIFunctionCall {
                                        name: tc.name.clone(),
                                        arguments: serde_json::to_string(&tc.arguments)
                                            .unwrap_or_default(),
                                    },
                                })
                                .collect(),
                        ),
                        tool_call_id: None,
                    }
                } else if msg.role == crate::ai::types::MessageRole::Tool {
                    // Tool response message
                    OpenAIMessage {
                        role: role.to_string(),
                        content: Some(msg.content.clone()),
                        tool_calls: None,
                        tool_call_id: msg.tool_call_id.clone(),
                    }
                } else {
                    // Regular message
                    OpenAIMessage {
                        role: role.to_string(),
                        content: Some(msg.content.clone()),
                        tool_calls: None,
                        tool_call_id: None,
                    }
                }
            })
            .collect()
    }

    /// Convert tool definitions to OpenAI format
    fn convert_tools(&self, tools: &[ToolDefinition]) -> Vec<OpenAITool> {
        tools
            .iter()
            .map(|tool| OpenAITool {
                typ: "function".to_string(),
                function: OpenAIFunctionDef {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.parameters.clone(),
                },
            })
            .collect()
    }

    /// Get thinking mode extra body for supported models
    fn get_thinking_extra(&self, model: &str) -> Option<serde_json::Value> {
        let model_lower = model.to_lowercase();

        // Models that auto-enable thinking (don't send extra body)
        let auto_thinking = ["deepseek-reasoner", "deepseek-r1", "qwq"];
        if auto_thinking.iter().any(|m| model_lower.contains(m)) {
            return None;
        }

        // Models that need opt-in
        if model_lower.contains("deepseek-chat") {
            return Some(serde_json::json!({
                "thinking": {"type": "enabled"}
            }));
        }

        None
    }
}

#[async_trait]
impl LlmAdapter for OpenAICompatAdapter {
    fn provider(&self) -> LLMProvider {
        self.provider.clone()
    }

    fn is_available(&self) -> bool {
        // Ollama doesn't need API key
        self.api_key.is_some() || self.provider == LLMProvider::Ollama
    }

    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        options: &LLMCallOptions,
    ) -> Result<LLMResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        let openai_messages = self.convert_messages(messages);
        let openai_tools = if tools.is_empty() {
            None
        } else {
            Some(self.convert_tools(tools))
        };

        let model = &self.default_model;
        let extra_body = self.get_thinking_extra(model);

        let mut body = serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "temperature": options.temperature,
        });

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(tools) = openai_tools {
            body["tools"] = serde_json::json!(tools);
        }

        // Merge extra body for thinking mode
        if let Some(extra) = extra_body
            && let Some(obj) = body.as_object_mut()
            && let Some(extra_obj) = extra.as_object()
        {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }

        let mut request = self.client.post(&url).json(&body);

        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| QuantixError::Other(format!("LLM API request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(QuantixError::Other(format!(
                "LLM API error ({}): {}",
                status, error_body
            )));
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::Other(format!("Failed to parse LLM response: {}", e)))?;

        // Parse response
        let choice = api_response.choices.first();
        match choice {
            Some(choice) => {
                let content = choice.message.content.clone();

                let tool_calls: Vec<ToolCall> = choice
                    .message
                    .tool_calls
                    .as_ref()
                    .map(|tcs| {
                        tcs.iter()
                            .map(|tc| {
                                let args: serde_json::Value = serde_json::from_str(
                                    &tc.function.arguments,
                                )
                                .unwrap_or(serde_json::json!({"raw": tc.function.arguments}));
                                ToolCall {
                                    id: tc.id.clone(),
                                    name: tc.function.name.clone(),
                                    arguments: args,
                                    thought_signature: None,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let usage = TokenUsage {
                    prompt_tokens: api_response.usage.prompt_tokens,
                    completion_tokens: api_response.usage.completion_tokens,
                    total_tokens: api_response.usage.total_tokens,
                };

                Ok(LLMResponse {
                    content,
                    tool_calls,
                    reasoning_content: None, // Would need special handling
                    usage,
                    provider: self.provider.to_string(),
                    model: api_response.model,
                })
            }
            None => Err(QuantixError::Other("LLM returned no choices".to_string())),
        }
    }
}

// ============================================================
// OpenAI API types
// ============================================================

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    typ: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    typ: String,
    function: OpenAIFunctionDef,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionDef {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    #[serde(default)]
    #[serde(rename = "finish_reason")]
    _finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAIToolCallResponse>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCallResponse {
    id: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    typ: String,
    function: OpenAIFunctionCallResponse,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionCallResponse {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u64,
    completion_tokens: u64,
    total_tokens: u64,
}
