use super::*;

use crate::core::{QuantixError, Result};

// ============================================================
// 舆情分析命令
// ============================================================

/// 处理舆情命令
pub async fn run_sentiment_command(cmd: SentimentCommands) -> Result<()> {
    match cmd {
        SentimentCommands::Show { code } => run_sentiment_show(&code).await,
        SentimentCommands::History { code, days } => run_sentiment_history(&code, days).await,
        SentimentCommands::Mentions { code, max } => run_sentiment_mentions(&code, max).await,
    }
}

async fn run_sentiment_show(code: &str) -> Result<()> {
    Err(sentiment_provider_unwired_error("show", code))
}

async fn run_sentiment_history(code: &str, _days: u32) -> Result<()> {
    Err(sentiment_provider_unwired_error("history", code))
}

async fn run_sentiment_mentions(code: &str, _max: usize) -> Result<()> {
    Err(sentiment_provider_unwired_error("mentions", code))
}

fn sentiment_provider_unwired_error(command: &str, code: &str) -> QuantixError {
    QuantixError::Unsupported(format!(
        "sentiment provider 尚未接线，sentiment {command} 当前不可生成真实舆情数据；code={code}"
    ))
}
