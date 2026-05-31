use super::*;
use crate::news::aggregator::AggregatorConfig;
use crate::news::providers::{BochaProvider, SerpApiProvider, TavilyProvider};
use crate::news::{NewsAggregator, NewsSearchRequest, NewsSearchResult};

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

    println!("⏳ 正在搜索...");

    let request = build_news_search_request(query, code, days, max, provider);
    let aggregator = build_news_aggregator_from_env();
    if aggregator.available_providers().is_empty() {
        print_missing_news_provider_config();
        return Ok(());
    }

    let result = execute_news_search(&aggregator, &request).await?;
    print_news_search_result(&result, max);

    Ok(())
}

fn build_news_search_request(
    query: &str,
    code: Option<&str>,
    days: u32,
    max: usize,
    provider: Option<&str>,
) -> NewsSearchRequest {
    let mut request = NewsSearchRequest::new(query)
        .with_days(days)
        .with_max_results(max);
    if let Some(code) = code {
        request = request.with_code(code);
    }
    request.provider = provider.map(str::to_string);
    request
}

fn build_news_aggregator_from_env() -> NewsAggregator {
    let mut providers = Vec::new();
    if let Ok(provider) = TavilyProvider::from_env() {
        providers.push(Box::new(provider) as Box<dyn crate::news::provider::NewsProvider>);
    }
    if let Ok(provider) = SerpApiProvider::from_env() {
        providers.push(Box::new(provider) as Box<dyn crate::news::provider::NewsProvider>);
    }
    if let Ok(provider) = BochaProvider::from_env() {
        providers.push(Box::new(provider) as Box<dyn crate::news::provider::NewsProvider>);
    }

    NewsAggregator::new(
        providers,
        AggregatorConfig {
            enable_cache: false,
            ..Default::default()
        },
    )
}

async fn execute_news_search(
    aggregator: &NewsAggregator,
    request: &NewsSearchRequest,
) -> Result<NewsSearchResult> {
    aggregator.search(request).await
}

fn print_missing_news_provider_config() {
    println!("❌ 未配置任何新闻搜索 API");
    println!();
    println!("请配置以下环境变量之一:");
    println!("  TAVILY_API_KEY=your_key     (推荐，高质量 AI 友好)");
    println!("  SERPAPI_API_KEY=your_key    (Google 搜索)");
    println!("  BOCHA_API_KEY=your_key      (中文优化)");
}

fn print_news_search_result(result: &NewsSearchResult, max: usize) {
    let cache_suffix = if result.from_cache {
        "，来自缓存"
    } else {
        ""
    };
    println!(
        "✅ 搜索完成: {} 条结果 | 提供商: {} | 耗时: {} ms{}",
        result.total, result.provider, result.elapsed_ms, cache_suffix
    );

    if result.articles.is_empty() {
        println!("📭 未找到匹配新闻");
        return;
    }

    for (index, article) in result.articles.iter().take(max).enumerate() {
        println!();
        println!("{}. {}", index + 1, article.title);
        println!("   来源: {}", article.source);
        if let Some(published_at) = article.published_at {
            println!("   时间: {}", published_at.format("%Y-%m-%d %H:%M:%S"));
        }
        if let Some(summary) = &article.summary
            && !summary.trim().is_empty()
        {
            println!("   摘要: {}", summary.trim());
        }
        if !article.related_codes.is_empty() {
            println!("   股票: {}", article.related_codes.join(", "));
        }
        println!("   链接: {}", article.url);
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::news::aggregator::AggregatorConfig;
    use crate::news::provider::NewsProvider;
    use crate::news::{
        NewsAggregator, NewsArticle, NewsProviderConfig, NewsSearchRequest, NewsSearchResult,
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct RecordingNewsProvider {
        seen_request: Arc<Mutex<Option<NewsSearchRequest>>>,
        config: NewsProviderConfig,
    }

    #[async_trait]
    impl NewsProvider for RecordingNewsProvider {
        fn name(&self) -> &'static str {
            "fake"
        }

        async fn search(&self, request: &NewsSearchRequest) -> Result<NewsSearchResult> {
            *self.seen_request.lock().unwrap() = Some(request.clone());
            Ok(NewsSearchResult::new(
                vec![NewsArticle::new(
                    format!("{} result", request.query),
                    "https://example.test/news".to_string(),
                    "fake".to_string(),
                )],
                "fake".to_string(),
            ))
        }

        fn is_available(&self) -> bool {
            true
        }

        fn config(&self) -> &NewsProviderConfig {
            &self.config
        }
    }

    #[tokio::test]
    async fn news_search_uses_aggregator_provider_results() {
        let seen_request = Arc::new(Mutex::new(None));
        let provider = RecordingNewsProvider {
            seen_request: Arc::clone(&seen_request),
            config: NewsProviderConfig::default(),
        };
        let aggregator = NewsAggregator::new(
            vec![Box::new(provider)],
            AggregatorConfig {
                enabled_providers: vec!["fake".to_string()],
                enable_cache: false,
                ..Default::default()
            },
        );
        let request = build_news_search_request("半导体", Some("600000"), 7, 3, Some("fake"));

        let result = execute_news_search(&aggregator, &request).await.unwrap();

        assert_eq!(result.provider, "fake");
        assert_eq!(result.total, 1);
        assert_eq!(result.articles[0].title, "半导体 result");

        let seen = seen_request.lock().unwrap().clone().unwrap();
        assert_eq!(seen.query, "半导体");
        assert_eq!(seen.codes, vec!["600000"]);
        assert_eq!(seen.days, 7);
        assert_eq!(seen.max_results, 3);
        assert_eq!(seen.provider.as_deref(), Some("fake"));
    }
}
