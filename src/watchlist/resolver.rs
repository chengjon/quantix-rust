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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistQuoteSnapshot {
    pub latest_price: Decimal,
    pub price_change_pct: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchlistDisplayRow {
    pub code: String,
    pub name: Option<String>,
    pub group: String,
    pub tags: Vec<String>,
    pub latest_price: Option<Decimal>,
    pub price_change_pct: Option<Decimal>,
}

#[async_trait]
pub trait WatchlistNameLookup: Send + Sync {
    async fn lookup_name(&self, code: &str) -> Result<Option<String>>;
}

#[async_trait]
pub trait WatchlistQuoteLookup: Send + Sync {
    async fn lookup_quotes(
        &self,
        codes: &[String],
    ) -> Result<HashMap<String, WatchlistQuoteSnapshot>>;
}

pub struct WatchlistResolver {
    name_lookup: Arc<dyn WatchlistNameLookup>,
    quote_lookup: Arc<dyn WatchlistQuoteLookup>,
}

impl WatchlistResolver {
    pub fn new(
        name_lookup: Arc<dyn WatchlistNameLookup>,
        quote_lookup: Arc<dyn WatchlistQuoteLookup>,
    ) -> Self {
        Self {
            name_lookup,
            quote_lookup,
        }
    }

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

#[derive(Debug, Clone)]
pub struct BridgeTdxWatchlistQuoteLookup {
    client: BridgeHttpClient,
}

impl BridgeTdxWatchlistQuoteLookup {
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
