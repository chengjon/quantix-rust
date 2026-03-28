//! Decision engine for trading analysis
//!
//! High-level interface for making AI-powered trading decisions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::adapter::LlmAdapter;
use super::prompt::PromptRegistry;
use super::types::{LLMCallOptions, Message};
use crate::core::Result;

/// Decision engine configuration
#[derive(Debug, Clone)]
pub struct DecisionEngineConfig {
    /// Maximum retries for LLM calls
    pub max_retries: u32,
    /// Timeout for LLM calls in seconds
    pub timeout_secs: u64,
    /// Enable thinking mode for supported models
    pub enable_thinking: bool,
}

impl Default for DecisionEngineConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout_secs: 120,
            enable_thinking: true,
        }
    }
}

/// Decision engine
pub struct DecisionEngine {
    adapter: Box<dyn LlmAdapter>,
    prompts: PromptRegistry,
    config: DecisionEngineConfig,
}

impl DecisionEngine {
    /// Create a new decision engine with the given adapter
    pub fn new(adapter: Box<dyn LlmAdapter>) -> Self {
        Self {
            adapter,
            prompts: PromptRegistry::new(),
            config: DecisionEngineConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(adapter: Box<dyn LlmAdapter>, config: DecisionEngineConfig) -> Self {
        Self {
            adapter,
            prompts: PromptRegistry::new(),
            config,
        }
    }

    /// Check if the engine is available
    pub fn is_available(&self) -> bool {
        self.adapter.is_available()
    }

    /// Analyze a stock
    pub async fn analyze_stock(
        &self,
        code: &str,
        name: &str,
        price_data: &str,
        indicators: &str,
        news: Option<&str>,
    ) -> Result<DecisionResult> {
        let mut variables = HashMap::new();
        variables.insert("code".to_string(), code.to_string());
        variables.insert("name".to_string(), name.to_string());
        variables.insert("price_data".to_string(), price_data.to_string());
        variables.insert("indicators".to_string(), indicators.to_string());
        variables.insert("news".to_string(), news.unwrap_or("暂无新闻").to_string());

        let (system, user) = self
            .prompts
            .render("stock_analysis", &variables)
            .unwrap_or_else(|| {
                // Fallback template
                (
                    "你是一个专业的股票分析师。".to_string(),
                    format!(
                        "请分析股票 {} ({}):\n\n价格数据:\n{}\n\n技术指标:\n{}",
                        code, name, price_data, indicators
                    ),
                )
            });

        let messages = vec![Message::system(&system), Message::user(&user)];

        let options = LLMCallOptions {
            timeout_secs: self.config.timeout_secs,
            enable_thinking: self.config.enable_thinking,
            ..Default::default()
        };

        let response = self.adapter.complete_text(&messages, &options).await?;

        Ok(DecisionResult {
            analysis: response.content.unwrap_or_default(),
            reasoning: response.reasoning_content,
            model: response.model,
            usage: response.usage,
        })
    }

    /// Make a trading decision
    pub async fn make_decision(
        &self,
        code: &str,
        current_position: &str,
        analysis: &str,
        risk_level: &str,
    ) -> Result<TradingDecision> {
        let mut variables = HashMap::new();
        variables.insert("code".to_string(), code.to_string());
        variables.insert("current_position".to_string(), current_position.to_string());
        variables.insert("analysis".to_string(), analysis.to_string());
        variables.insert("risk_level".to_string(), risk_level.to_string());

        let (system, user) = self
            .prompts
            .render("trading_decision", &variables)
            .unwrap_or_else(|| {
                (
                    "你是一个专业的交易决策助手。请基于分析结果给出明确的交易建议。".to_string(),
                    format!(
                        "股票代码: {}\n当前持仓: {}\n分析结果: {}\n风险等级: {}\n\n请给出交易建议 (买入/卖出/持有) 并说明理由。",
                        code, current_position, analysis, risk_level
                    ),
                )
            });

        let messages = vec![Message::system(&system), Message::user(&user)];

        let options = LLMCallOptions {
            timeout_secs: self.config.timeout_secs,
            enable_thinking: self.config.enable_thinking,
            ..Default::default()
        };

        let response = self.adapter.complete_text(&messages, &options).await?;

        // Parse decision from response
        let content = response.content.unwrap_or_default();
        let decision = parse_trading_decision(&content);

        Ok(TradingDecision {
            action: decision.action,
            confidence: decision.confidence,
            reasoning: content,
            model: response.model,
        })
    }

    /// Simple chat completion
    pub async fn chat(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let mut messages = Vec::new();

        if let Some(sys) = system {
            messages.push(Message::system(sys));
        }
        messages.push(Message::user(prompt));

        let options = LLMCallOptions {
            timeout_secs: self.config.timeout_secs,
            ..Default::default()
        };

        let response = self.adapter.complete_text(&messages, &options).await?;

        Ok(response.content.unwrap_or_default())
    }
}

/// Result of an analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    /// Analysis content
    pub analysis: String,
    /// Reasoning content (for thinking models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Model used
    pub model: String,
    /// Token usage
    pub usage: super::types::TokenUsage,
}

/// Trading decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    /// Recommended action
    pub action: TradeAction,
    /// Confidence level (0-100)
    pub confidence: u8,
    /// Reasoning
    pub reasoning: String,
    /// Model used
    pub model: String,
}

/// Trade action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeAction {
    Buy,
    Sell,
    Hold,
}

impl std::fmt::Display for TradeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "买入"),
            Self::Sell => write!(f, "卖出"),
            Self::Hold => write!(f, "持有"),
        }
    }
}

/// Parse trading decision from LLM response
fn parse_trading_decision(content: &str) -> TradingDecision {
    let content_lower = content.to_lowercase();

    // Simple keyword-based parsing
    let action = if content_lower.contains("买入") || content_lower.contains("buy") {
        TradeAction::Buy
    } else if content_lower.contains("卖出") || content_lower.contains("sell") {
        TradeAction::Sell
    } else {
        TradeAction::Hold
    };

    // Estimate confidence based on certainty words
    let confidence = if content_lower.contains("强烈") || content_lower.contains("非常") || content_lower.contains("highly") {
        85
    } else if content_lower.contains("建议") || content_lower.contains("recommend") {
        70
    } else if content_lower.contains("可能") || content_lower.contains("might") || content_lower.contains("perhaps") {
        50
    } else {
        60
    };

    TradingDecision {
        action,
        confidence,
        reasoning: content.to_string(),
        model: String::new(),
    }
}
