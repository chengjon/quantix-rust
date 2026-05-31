use super::*;

use crate::ai::providers::OpenAICompatAdapter;
use crate::ai::{DecisionEngine, LlmConfig};
use crate::core::{CliRuntime, QuantixError, Result};

/// 处理 AI 命令
pub async fn run_ai_command(cmd: AiCommands) -> Result<()> {
    match cmd {
        AiCommands::Analyze {
            code,
            model,
            with_news,
        } => run_ai_analyze(&code, Some(model), with_news).await,
        AiCommands::Decide {
            code,
            position,
            risk,
        } => run_ai_decide(&code, position, &risk).await,
        AiCommands::Ask {
            question,
            code,
            model,
        } => run_ai_ask(&question, code.as_deref(), Some(model)).await,
        AiCommands::Market { date } => run_ai_market(date.as_deref()).await,
        AiCommands::Config { show, test } => run_ai_config(show, test).await,
    }
}

async fn run_ai_analyze(code: &str, model: Option<String>, with_news: bool) -> Result<()> {
    println!("🔍 AI 股票分析");
    println!("   代码: {}", code);
    if let Some(ref m) = model {
        println!("   模型: {}", m);
    }
    if with_news {
        println!("   包含新闻: 是");
    }
    println!("{}", ai_analyze_context_warning(with_news));
    println!();

    let config = LlmConfig::from_env();

    if !config.has_any_provider() {
        println!("❌ 未配置任何 LLM 提供商");
        println!();
        println!("请配置以下环境变量之一:");
        println!("  DEEPSEEK_API_KEY=your_key");
        println!("  OPENAI_API_KEY=your_key");
        println!("  GEMINI_API_KEY=your_key");
        println!("  ANTHROPIC_API_KEY=your_key");
        return Ok(());
    }

    // Create adapter based on available provider
    let adapter = if config.get_provider("deepseek").is_some() {
        Box::new(OpenAICompatAdapter::deepseek(&config)) as Box<dyn crate::ai::adapter::LlmAdapter>
    } else if config.get_provider("openai").is_some() {
        Box::new(OpenAICompatAdapter::openai(&config))
    } else if config.get_provider("ollama").is_some() {
        Box::new(OpenAICompatAdapter::ollama(&config))
    } else {
        println!("❌ 不支持的 LLM 提供商配置");
        return Ok(());
    };

    let engine = DecisionEngine::new(adapter);

    // Placeholder data - in real implementation would fetch from data sources
    let price_data = "近期价格数据 (模拟)";
    let indicators = "技术指标数据 (模拟)";

    match engine
        .analyze_stock(code, code, price_data, indicators, None)
        .await
    {
        Ok(result) => {
            println!("📊 分析结果:");
            println!();
            println!("{}", result.analysis);
            println!();
            println!(
                "📈 Token 使用: {} (提示) + {} (完成) = {} (总计)",
                result.usage.prompt_tokens,
                result.usage.completion_tokens,
                result.usage.total_tokens
            );
            println!("🤖 模型: {}", result.model);
            Ok(())
        }
        Err(e) => {
            println!("❌ 分析失败: {}", e);
            Err(e)
        }
    }
}

async fn run_ai_decide(code: &str, position: Option<i64>, risk: &str) -> Result<()> {
    println!("💡 AI 交易决策");
    println!("   代码: {}", code);
    println!("   持仓: {}", position.unwrap_or(0));
    println!("   风险: {}", risk);
    println!("{}", ai_decide_context_warning());
    println!();

    let config = LlmConfig::from_env();

    if !config.has_any_provider() {
        println!("❌ 未配置任何 LLM 提供商");
        return Ok(());
    }

    let adapter = if config.get_provider("deepseek").is_some() {
        Box::new(OpenAICompatAdapter::deepseek(&config)) as Box<dyn crate::ai::adapter::LlmAdapter>
    } else if config.get_provider("openai").is_some() {
        Box::new(OpenAICompatAdapter::openai(&config))
    } else {
        Box::new(OpenAICompatAdapter::ollama(&config))
    };

    let engine = DecisionEngine::new(adapter);

    let position_str = format!("{} 股", position.unwrap_or(0));
    let analysis = "技术面分析结果 (模拟)";

    match engine
        .make_decision(code, &position_str, analysis, risk)
        .await
    {
        Ok(decision) => {
            println!("📋 交易决策:");
            println!();
            println!("   动作: {}", decision.action);
            println!("   置信度: {}%", decision.confidence);
            println!();
            println!("📝 理由:");
            println!("{}", decision.reasoning);
            Ok(())
        }
        Err(e) => {
            println!("❌ 决策失败: {}", e);
            Err(e)
        }
    }
}

async fn run_ai_ask(question: &str, code: Option<&str>, _model: Option<String>) -> Result<()> {
    println!("💬 AI 问答");
    println!("   问题: {}", question);
    if let Some(c) = code {
        println!("   相关股票: {}", c);
    }
    println!();

    let config = LlmConfig::from_env();

    if !config.has_any_provider() {
        println!("❌ 未配置任何 LLM 提供商");
        return Ok(());
    }

    let adapter = if config.get_provider("deepseek").is_some() {
        Box::new(OpenAICompatAdapter::deepseek(&config)) as Box<dyn crate::ai::adapter::LlmAdapter>
    } else if config.get_provider("openai").is_some() {
        Box::new(OpenAICompatAdapter::openai(&config))
    } else {
        Box::new(OpenAICompatAdapter::ollama(&config))
    };

    let engine = DecisionEngine::new(adapter);

    let system = Some(
        "你是一个专业的A股投资顾问，请基于你的知识回答用户的问题。注意：不构成投资建议，仅供参考。",
    );

    match engine.chat(question, system).await {
        Ok(response) => {
            println!("🤖 回答:");
            println!();
            println!("{}", response);
            Ok(())
        }
        Err(e) => {
            println!("❌ 回答失败: {}", e);
            Err(e)
        }
    }
}

async fn run_ai_market(date: Option<&str>) -> Result<()> {
    println!("📈 AI 市场复盘");
    if let Some(d) = date {
        println!("   日期: {}", d);
    }
    println!("{}", ai_market_context_warning());
    println!();

    let config = LlmConfig::from_env();

    if !config.has_any_provider() {
        println!("❌ 未配置任何 LLM 提供商");
        return Ok(());
    }

    let adapter = if config.get_provider("deepseek").is_some() {
        Box::new(OpenAICompatAdapter::deepseek(&config)) as Box<dyn crate::ai::adapter::LlmAdapter>
    } else if config.get_provider("openai").is_some() {
        Box::new(OpenAICompatAdapter::openai(&config))
    } else {
        Box::new(OpenAICompatAdapter::ollama(&config))
    };

    let engine = DecisionEngine::new(adapter);

    let prompt = "请分析今日A股市场整体表现，包括主要指数走势、板块轮动、资金流向等方面，并给出明日市场展望。";

    let system = Some("你是一个专业的A股市场分析师，请基于市场数据进行分析。");

    match engine.chat(prompt, system).await {
        Ok(response) => {
            println!("📊 市场分析:");
            println!();
            println!("{}", response);
            Ok(())
        }
        Err(e) => {
            println!("❌ 分析失败: {}", e);
            Err(e)
        }
    }
}

async fn run_ai_config(show: bool, test: bool) -> Result<()> {
    let config = LlmConfig::from_env();

    if show {
        println!("📋 AI 配置信息:");
        println!();
        println!("   默认模型: {}", config.default_model);
        println!("   温度: {}", config.temperature);
        println!("   最大Token: {}", config.max_tokens);
        println!("   超时: {}秒", config.timeout_secs);
        println!();
        println!("   已配置提供商:");
        for (name, provider_config) in &config.providers {
            let has_key = provider_config.api_key.is_some();
            let key_status = if has_key {
                "✅ 已配置"
            } else {
                "❌ 未配置"
            };
            println!(
                "     - {}: {} (模型: {:?})",
                name, key_status, provider_config.models
            );
        }
    }

    if test {
        println!();
        println!("🔄 测试 LLM 连通性...");
        println!();

        for (name, provider_config) in &config.providers {
            print!("   测试 {}... ", name);
            println!("{}", ai_config_test_status(name, provider_config));
        }
    }

    if !show && !test {
        println!("📋 AI 配置状态:");
        println!();
        if config.has_any_provider() {
            println!("✅ 已配置 {} 个 LLM 提供商", config.providers.len());
            println!();
            println!("使用 'quantix ai config --show' 查看详细配置");
            println!("使用 'quantix ai config --test' 检查已配置 provider 状态");
        } else {
            println!("❌ 未配置任何 LLM 提供商");
            println!();
            println!("请配置以下环境变量之一:");
            println!("  DEEPSEEK_API_KEY=your_key");
            println!("  OPENAI_API_KEY=your_key");
            println!("  GEMINI_API_KEY=your_key");
            println!("  ANTHROPIC_API_KEY=your_key");
            println!("  OLLAMA_API_BASE=http://localhost:11434");
        }
    }

    Ok(())
}

fn ai_config_test_status(
    name: &str,
    provider_config: &crate::ai::adapter::ProviderConfig,
) -> &'static str {
    if provider_config.api_key.is_some() || name == "ollama" {
        "⚠️ 已配置（未发起真实连通性测试）"
    } else {
        "❌ 缺少认证配置"
    }
}

fn ai_analyze_context_warning(with_news: bool) -> String {
    let mut warning =
        "⚠️ 当前使用模拟价格/指标上下文；不会读取实时行情、财务或技术指标流水".to_string();
    if with_news {
        warning.push_str("；--with-news 仅回显，不注入真实新闻数据");
    }
    warning
}

fn ai_decide_context_warning() -> &'static str {
    "⚠️ 当前使用模拟技术面分析上下文；不会读取实仓、行情或信号流水"
}

fn ai_market_context_warning() -> &'static str {
    "⚠️ 当前使用固定提示词；不会抓取实时市场数据"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::adapter::ProviderConfig;

    #[test]
    fn ai_config_test_status_does_not_claim_connectivity() {
        let provider = ProviderConfig {
            api_key: Some("test-key".to_string()),
            base_url: Some("https://example.test/v1".to_string()),
            models: vec!["test-model".to_string()],
        };

        let status = ai_config_test_status("openai", &provider);

        assert_eq!(status, "⚠️ 已配置（未发起真实连通性测试）");
        assert!(!status.contains("可用"));
    }

    #[test]
    fn ai_analyze_context_warning_discloses_simulated_inputs() {
        let warning = ai_analyze_context_warning(true);

        assert!(warning.contains("模拟价格/指标上下文"));
        assert!(warning.contains("--with-news 仅回显"));
        assert!(warning.contains("不注入真实新闻数据"));
    }

    #[test]
    fn ai_decide_context_warning_discloses_simulated_analysis() {
        let warning = ai_decide_context_warning();

        assert!(warning.contains("模拟技术面分析上下文"));
        assert!(warning.contains("不会读取实仓、行情或信号流水"));
    }

    #[test]
    fn ai_market_context_warning_discloses_fixed_prompt_boundary() {
        let warning = ai_market_context_warning();

        assert!(warning.contains("固定提示词"));
        assert!(warning.contains("不会抓取实时市场数据"));
    }
}
