use super::*;

// ============================================================
// 新闻搜索命令
// ============================================================

/// 处理新闻命令
pub async fn run_news_command(cmd: NewsCommands) -> Result<()> {
    match cmd {
        NewsCommands::Search {
            query,
            code,
            days,
            max,
            provider,
        } => run_news_search(&query, code.as_deref(), days, max, provider.as_deref()).await,
        NewsCommands::Code { code, days, max } => run_news_by_code(&code, days, max).await,
        NewsCommands::Trend { date, code } => {
            run_news_trend(date.as_deref(), code.as_deref()).await
        }
        NewsCommands::Providers => run_news_providers().await,
    }
}

async fn run_news_search(
    query: &str,
    code: Option<&str>,
    days: u32,
    max: usize,
    provider: Option<&str>,
) -> Result<()> {
    println!("📰 新闻搜索");
    println!("   关键词: {}", query);
    if let Some(c) = code {
        println!("   股票代码: {}", c);
    }
    println!("   时间范围: {} 天", days);
    println!("   最大结果: {}", max);
    if let Some(p) = provider {
        println!("   提供商: {}", p);
    }
    println!();

    // TODO: 实现实际的新闻搜索
    // 目前显示占位信息
    println!("⏳ 正在搜索...");

    // 检查可用的 API 密钥
    let tavily_key = std::env::var("TAVILY_API_KEY").ok();
    let serpapi_key = std::env::var("SERPAPI_API_KEY").ok();
    let bocha_key = std::env::var("BOCHA_API_KEY").ok();

    if tavily_key.is_none() && serpapi_key.is_none() && bocha_key.is_none() {
        println!("❌ 未配置任何新闻搜索 API");
        println!();
        println!("请配置以下环境变量之一:");
        println!("  TAVILY_API_KEY=your_key     (推荐，高质量 AI 友好)");
        println!("  SERPAPI_API_KEY=your_key    (Google 搜索)");
        println!("  BOCHA_API_KEY=your_key      (中文优化)");
        return Ok(());
    }

    // 显示可用的提供商
    let mut available = Vec::new();
    if tavily_key.is_some() {
        available.push("tavily");
    }
    if serpapi_key.is_some() {
        available.push("serpapi");
    }
    if bocha_key.is_some() {
        available.push("bocha");
    }
    println!("📋 可用提供商: {}", available.join(", "));
    println!();
    println!("💡 新闻搜索模块已加载，实际搜索功能需要完整实现");
    println!("   请参考 docs/MIGRATION_FROM_DAILY_STOCK_ANALYSIS.md");

    Ok(())
}

async fn run_news_by_code(code: &str, days: u32, max: usize) -> Result<()> {
    println!("📰 股票相关新闻");
    println!("   代码: {}", code);
    println!("   时间范围: {} 天", days);
    println!("   最大结果: {}", max);
    println!();

    // 使用股票名称作为搜索关键词
    let query = format!("{} 股票", code);
    run_news_search(&query, Some(code), days, max, None).await
}

async fn run_news_trend(date: Option<&str>, code: Option<&str>) -> Result<()> {
    println!("📊 新闻趋势分析");
    if let Some(d) = date {
        println!("   日期: {}", d);
    }
    if let Some(c) = code {
        println!("   股票代码: {}", c);
    }
    println!();
    println!("💡 趋势分析功能开发中...");

    Ok(())
}

async fn run_news_providers() -> Result<()> {
    println!("📰 可用的新闻搜索提供商");
    println!();

    let providers = vec![
        (
            "tavily",
            "Tavily",
            "高质量 AI 友好的搜索 API",
            "TAVILY_API_KEY",
        ),
        (
            "serpapi",
            "SerpAPI",
            "Google 搜索结果 API",
            "SERPAPI_API_KEY",
        ),
        ("bocha", "博查", "中文优化的新闻搜索", "BOCHA_API_KEY"),
    ];

    for (id, name, desc, env_var) in &providers {
        let configured = std::env::var(env_var).is_ok();
        let status = if configured {
            "✅ 已配置"
        } else {
            "❌ 未配置"
        };
        println!("  {} {} - {}", status, name, desc);
        println!("     ID: {} | 环境变量: {}", id, env_var);
        println!();
    }

    println!("💡 配置环境变量后即可使用对应的新闻搜索服务");

    Ok(())
}
