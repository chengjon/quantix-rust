use super::*;
use crate::core::QuantixError;
use crate::news::aggregator::AggregatorConfig;
use crate::news::providers::{BochaProvider, SerpApiProvider, TavilyProvider};
use crate::news::{NewsAggregator, NewsSearchRequest, NewsSearchResult};

const NEWS_TREND_DEFAULT_DAYS: u32 = 3;
const NEWS_TREND_DEFAULT_MAX_RESULTS: usize = 20;

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
    let request = build_news_search_request(query, code, days, max, provider);
    let aggregator = build_news_aggregator_from_env();
    ensure_news_provider_configured(&aggregator)?;

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

fn build_news_trend_search_request(date: Option<&str>, code: Option<&str>) -> NewsSearchRequest {
    let query = build_news_trend_query(date, code);
    build_news_search_request(
        &query,
        code,
        NEWS_TREND_DEFAULT_DAYS,
        NEWS_TREND_DEFAULT_MAX_RESULTS,
        None,
    )
}

fn build_news_trend_query(date: Option<&str>, code: Option<&str>) -> String {
    let mut query = match code {
        Some(code) => format!("{} 股票热点 新闻", code),
        None => "市场热点 新闻".to_string(),
    };
    if let Some(date) = date.map(str::trim).filter(|date| !date.is_empty()) {
        query.push(' ');
        query.push_str(date);
    }
    query
}

fn ensure_news_provider_configured(aggregator: &NewsAggregator) -> Result<()> {
    if aggregator.available_providers().is_empty() {
        Err(news_provider_unconfigured_error())
    } else {
        Ok(())
    }
}

fn news_provider_unconfigured_error() -> QuantixError {
    QuantixError::Unsupported(
        "news provider 尚未配置；请配置 TAVILY_API_KEY、SERPAPI_API_KEY 或 BOCHA_API_KEY 后再执行 news search/code/trend；可用 news providers 查看状态"
            .to_string(),
    )
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
    // 使用股票名称作为搜索关键词
    let query = format!("{} 股票", code);
    run_news_search(&query, Some(code), days, max, None).await
}

async fn run_news_trend(date: Option<&str>, code: Option<&str>) -> Result<()> {
    let request = build_news_trend_search_request(date, code);
    let aggregator = build_news_aggregator_from_env();
    ensure_news_provider_configured(&aggregator)?;

    println!("📊 新闻趋势分析");
    if let Some(d) = date {
        println!("   日期: {}", d);
    }
    if let Some(c) = code {
        println!("   股票代码: {}", c);
    }
    println!();

    println!("⏳ 正在搜索趋势相关新闻...");

    let result = execute_news_search(&aggregator, &request).await?;
    print_news_search_result(&result, request.max_results);

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

    #[test]
    fn news_trend_builds_market_hotspot_query() {
        let request = build_news_trend_search_request(None, None);

        assert_eq!(request.query, "市场热点 新闻");
        assert!(request.codes.is_empty());
        assert_eq!(request.days, 3);
        assert_eq!(request.max_results, 20);
        assert_eq!(request.provider, None);
    }

    #[test]
    fn news_trend_builds_code_and_date_query() {
        let request = build_news_trend_search_request(Some("20260531"), Some("600519"));

        assert_eq!(request.query, "600519 股票热点 新闻 20260531");
        assert_eq!(request.codes, vec!["600519"]);
        assert_eq!(request.days, 3);
        assert_eq!(request.max_results, 20);
        assert_eq!(request.provider, None);
    }
}
