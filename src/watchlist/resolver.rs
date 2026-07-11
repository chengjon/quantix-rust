use crate::bridge::client::BridgeHttpClient;
use crate::core::Result;
use crate::db::PostgresClient;
use crate::watchlist::WatchlistListItem;
use async_trait::async_trait;
use futures_util::future::join_all;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, timeout};

const NAME_LOOKUP_TIMEOUT: Duration = Duration::from_secs(1);

/// 自选股行情快照：latest_price 最新价、price_change_pct 可选涨跌幅（百分比）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistQuoteSnapshot {
    pub latest_price: Decimal,
    pub price_change_pct: Option<Decimal>,
}

/// 自选股展示行：code、name 可选名称、group 分组、tags 标签、latest_price/price_change_pct 可选行情。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistDisplayRow {
    pub code: String,
    pub name: Option<String>,
    pub group: String,
    pub tags: Vec<String>,
    pub latest_price: Option<Decimal>,
    pub price_change_pct: Option<Decimal>,
}

/// 自选股名称查询 trait：按 code 返回可选名称（异步）。实现可走 Postgres / 本地缓存 / 第三方接口。
#[async_trait]
pub trait WatchlistNameLookup: Send + Sync {
    async fn lookup_name(&self, code: &str) -> Result<Option<String>>;
}

/// 自选股行情批量查询 trait：codes 输入标的列表，返回 code → WatchlistQuoteSnapshot 映射（异步）。
#[async_trait]
pub trait WatchlistQuoteLookup: Send + Sync {
    async fn lookup_quotes(
        &self,
        codes: &[String],
    ) -> Result<HashMap<String, WatchlistQuoteSnapshot>>;
}

/// 自选股解析器：组合名称查询与行情查询，将 WatchlistListItem 列表解析为展示行。名称查询超时 1s 容错为 None。
pub struct WatchlistResolver {
    name_lookup: Arc<dyn WatchlistNameLookup>,
    quote_lookup: Arc<dyn WatchlistQuoteLookup>,
}

impl WatchlistResolver {
    /// 构造解析器：注入名称查询与行情查询实现（均以 trait object 形式持有）。
    pub fn new(
        name_lookup: Arc<dyn WatchlistNameLookup>,
        quote_lookup: Arc<dyn WatchlistQuoteLookup>,
    ) -> Self {
        Self {
            name_lookup,
            quote_lookup,
        }
    }

    /// 将自选股列表解析为展示行：with_price=false 跳过行情查询仅填充名称；名称查询并发执行（join_all），单标的 1s 超时容错。行情查询失败返回空 map（不阻断）。
    pub async fn resolve_rows(
        &self,
        items: &[WatchlistListItem],
        with_price: bool,
    ) -> Vec<WatchlistDisplayRow> {
        let quote_map = if with_price {
            let codes: Vec<String> = items.iter().map(|item| item.code.clone()).collect();
            self.quote_lookup
                .lookup_quotes(&codes)
                .await
                .unwrap_or_default()
        } else {
            HashMap::new()
        };
        let name_map: HashMap<String, Option<String>> = join_all(items.iter().map(|item| {
            let code = item.code.clone();
            let name_lookup = Arc::clone(&self.name_lookup);

            async move {
                let name = match timeout(NAME_LOOKUP_TIMEOUT, name_lookup.lookup_name(&code)).await
                {
                    Ok(Ok(name)) => name,
                    Ok(Err(_)) | Err(_) => None,
                };

                (code, name)
            }
        }))
        .await
        .into_iter()
        .collect();

        let mut rows = Vec::with_capacity(items.len());
        for item in items {
            let name = name_map.get(&item.code).cloned().flatten();
            let quote = quote_map.get(&item.code);

            rows.push(WatchlistDisplayRow {
                code: item.code.clone(),
                name,
                group: item.group.clone(),
                tags: item.tags.clone(),
                latest_price: quote.map(|snapshot| snapshot.latest_price),
                price_change_pct: quote.and_then(|snapshot| snapshot.price_change_pct),
            });
        }

        rows
    }
}

/// 基于 Postgres stock_info 表的名称查询实现：通过 POSTGRES_URL 环境变量连接；缺失时返回 None。
#[derive(Debug, Clone, Default)]
pub struct PostgresWatchlistNameLookup;

#[async_trait]
impl WatchlistNameLookup for PostgresWatchlistNameLookup {
    async fn lookup_name(&self, code: &str) -> Result<Option<String>> {
        let Some(database_url) = std::env::var("POSTGRES_URL").ok() else {
            return Ok(None);
        };

        let client = PostgresClient::new(&database_url).await?;
        Ok(client
            .query_stock_info(code)
            .await?
            .map(|stock_info| stock_info.name))
    }
}

/// 基于 TDX 直连的行情批量查询实现：空输入返回空 map；非空输入通过 TdxSource 抓取行情，无价格数据被跳过。
#[derive(Debug, Clone, Default)]
pub struct TdxWatchlistQuoteLookup;

#[async_trait]
impl WatchlistQuoteLookup for TdxWatchlistQuoteLookup {
    async fn lookup_quotes(
        &self,
        codes: &[String],
    ) -> Result<HashMap<String, WatchlistQuoteSnapshot>> {
        if codes.is_empty() {
            return Ok(HashMap::new());
        }

        let source = crate::sources::TdxSource::with_default_config()?;
        let code_refs: Vec<(u16, &str)> = codes
            .iter()
            .map(|code| (infer_market(code), code.as_str()))
            .collect();

        let quotes = source.fetch_quotes_batch(&code_refs).await?;
        let mut result = HashMap::with_capacity(quotes.len());

        for quote in quotes {
            let latest_price = Decimal::from_f64_retain(quote.price);
            let change_pct = Decimal::from_f64_retain(quote.change_percent);

            if let Some(price) = latest_price {
                result.insert(
                    quote.code,
                    WatchlistQuoteSnapshot {
                        latest_price: price,
                        price_change_pct: change_pct,
                    },
                );
            }
        }

        Ok(result)
    }
}

/// 基于 bridge HTTP 接口的 TDX 行情批量查询实现（与 TdxWatchlistQuoteLookup 对应，但走 bridge 而非直连）。
#[derive(Debug, Clone)]
pub struct BridgeTdxWatchlistQuoteLookup {
    client: BridgeHttpClient,
}

impl BridgeTdxWatchlistQuoteLookup {
    /// 构造 bridge 行情查询：注入 BridgeHttpClient。
    pub fn new(client: BridgeHttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl WatchlistQuoteLookup for BridgeTdxWatchlistQuoteLookup {
    async fn lookup_quotes(
        &self,
        codes: &[String],
    ) -> Result<HashMap<String, WatchlistQuoteSnapshot>> {
        if codes.is_empty() {
            return Ok(HashMap::new());
        }

        let source = crate::sources::BridgeTdxSource::new(self.client.clone());
        let code_refs: Vec<(u16, &str)> = codes
            .iter()
            .map(|code| (infer_market(code), code.as_str()))
            .collect();
        let quotes = source.fetch_quotes_batch(&code_refs).await?;
        let mut result = HashMap::with_capacity(quotes.len());

        for quote in quotes {
            let latest_price = Decimal::from_f64_retain(quote.price);
            let change_pct = Decimal::from_f64_retain(quote.change_percent);

            if let Some(price) = latest_price {
                result.insert(
                    quote.code,
                    WatchlistQuoteSnapshot {
                        latest_price: price,
                        price_change_pct: change_pct,
                    },
                );
            }
        }

        Ok(result)
    }
}

fn infer_market(code: &str) -> u16 {
    if code.starts_with('6') { 1 } else { 0 }
}
