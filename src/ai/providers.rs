//! LLM Provider implementations
//!
//! OpenAI-compatible providers (OpenAI, DeepSeek, local Ollama, etc.)

pub mod openai_compat;

pub use openai_compat::OpenAICompatAdapter;
