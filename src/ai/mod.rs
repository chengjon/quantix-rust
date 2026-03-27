//! AI Decision Module
//!
//! Phase 2: LLM-based decision support for quantitative trading
//!
//! # Architecture
//!
//! - `types` - Core data types (ToolCall, LLMResponse, Message)
//! - `adapter` - LLM adapter trait and provider implementations
//! - `providers` - Individual provider implementations (OpenAI, DeepSeek, etc.)
//! - `prompt` - Prompt template system
//! - `decision` - Decision engine for trading analysis

pub mod types;
pub mod adapter;
pub mod providers;
pub mod prompt;
pub mod decision;

pub use types::{ToolCall, LLMResponse, Message, MessageRole};
pub use adapter::{LlmAdapter, LlmConfig};
pub use decision::{DecisionEngine, DecisionResult};
