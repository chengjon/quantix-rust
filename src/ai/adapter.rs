//! LLM Adapter trait and configuration
//!
//! Unified interface for all LLM providers

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{LLMCallOptions, LLMResponse, Message, ToolDefinition, LLMProvider};

/// Main LLM adapter trait
#[async_trait]
pub trait LlmAdapter: Send + Sync {
    /// Get the provider name
    fn provider(&self) -> LLMProvider;

    /// Check if the adapter is available (has valid credentials)
    fn is_available(&self) -> bool;

    /// Send a completion request with optional tools
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        options: &LLMCallOptions,
    ) -> crate::core::Result<LLMResponse>;

    /// Send a simple text completion (no tools)
    async fn complete_text(
        &self,
        messages: &[Message],
        options: &LLMCallOptions,
    ) -> crate::core::Result<LLMResponse> {
        self.complete(messages, &[], options).await
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default model to use (e.g., "deepseek-chat", "gpt-4o")
    pub default_model: String,
    /// Fallback models to try if primary fails
    #[serde(default)]
    pub fallback_models: Vec<String>,
    /// Default temperature
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Default max tokens
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
}

fn default_temperature() -> f32 { 0.7 }
fn default_max_tokens() -> u32 { 4096 }
fn default_timeout() -> u64 { 60 }

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_model: "deepseek-chat".to_string(),
            fallback_models: Vec::new(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
            timeout_secs: default_timeout(),
            providers: HashMap::new(),
        }
    }
}

impl LlmConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Load default model from environment
        if let Ok(model) = std::env::var("LLM_DEFAULT_MODEL") {
            config.default_model = model;
        }

        // Load fallback models
        if let Ok(fallbacks) = std::env::var("LLM_FALLBACK_MODELS") {
            config.fallback_models = fallbacks
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Load temperature
        if let Ok(temp) = std::env::var("LLM_TEMPERATURE") {
            if let Ok(t) = temp.parse() {
                config.temperature = t;
            }
        }

        // Load max tokens
        if let Ok(tokens) = std::env::var("LLM_MAX_TOKENS") {
            if let Ok(t) = tokens.parse() {
                config.max_tokens = t;
            }
        }

        // Load provider configurations
        config.load_provider_configs();

        config
    }

    fn load_provider_configs(&mut self) {
        // DeepSeek
        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let base_url = std::env::var("DEEPSEEK_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com/v1".to_string());
            self.providers.insert(
                "deepseek".to_string(),
                ProviderConfig {
                    api_key: Some(api_key),
                    base_url: Some(base_url),
                    models: vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()],
                },
            );
        }

        // OpenAI
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let base_url = std::env::var("OPENAI_BASE_URL");
            self.providers.insert(
                "openai".to_string(),
                ProviderConfig {
                    api_key: Some(api_key),
                    base_url: base_url.ok(),
                    models: vec!["gpt-4o".to_string(), "gpt-4o-mini".to_string()],
                },
            );
        }

        // Gemini
        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            self.providers.insert(
                "gemini".to_string(),
                ProviderConfig {
                    api_key: Some(api_key),
                    base_url: None,
                    models: vec!["gemini-2.5-flash".to_string(), "gemini-2.5-pro".to_string()],
                },
            );
        }

        // Anthropic
        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            self.providers.insert(
                "anthropic".to_string(),
                ProviderConfig {
                    api_key: Some(api_key),
                    base_url: None,
                    models: vec!["claude-3-5-sonnet-latest".to_string()],
                },
            );
        }

        // Ollama (local)
        if std::env::var("OLLAMA_API_BASE").is_ok() || std::env::var("OLLAMA_HOST").is_ok() {
            let base_url = std::env::var("OLLAMA_API_BASE")
                .or_else(|_| std::env::var("OLLAMA_HOST"))
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            self.providers.insert(
                "ollama".to_string(),
                ProviderConfig {
                    api_key: None,
                    base_url: Some(base_url),
                    models: vec!["llama3".to_string(), "qwen2".to_string()],
                },
            );
        }
    }

    /// Get configuration for a specific provider
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Check if any provider is configured
    pub fn has_any_provider(&self) -> bool {
        !self.providers.is_empty()
    }
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key (if required)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Available models for this provider
    #[serde(default)]
    pub models: Vec<String>,
}
