//! Active-code list fetcher (P0.15b).
//!
//! Reads `quantix.stock_info` for codes with `trade_status='1'` (active).
//! The trait abstraction lets `BatchScheduler` be unit-tested with an
//! in-memory fetcher.

use async_trait::async_trait;

use crate::core::error::{QuantixError, Result};
use crate::db::PostgresClient;

/// Read interface for the active-code list.
#[async_trait]
pub trait StockListFetchTrait: Send + Sync {
    async fn list_active_codes(&self) -> Result<Vec<String>>;
}

/// Production impl backed by PostgreSQL. Filters on `trade_status='1'`.
pub struct StockListFetcher<'a> {
    pg: &'a PostgresClient,
}

impl<'a> StockListFetcher<'a> {
    pub fn new(pg: &'a PostgresClient) -> Self {
        Self { pg }
    }
}

#[async_trait]
impl<'a> StockListFetchTrait for StockListFetcher<'a> {
    async fn list_active_codes(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT code FROM quantix.stock_info \
             WHERE trade_status = '1' \
             ORDER BY code",
        )
        .fetch_all(self.pg.pool())
        .await
        .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;
        Ok(rows.into_iter().map(|(c,)| c).collect())
    }
}

/// In-memory `StockListFetchTrait` impl for unit tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockFetcher {
    codes: Vec<String>,
}

#[cfg(test)]
impl MockFetcher {
    pub fn new(codes: Vec<String>) -> Self {
        Self { codes }
    }
}

#[cfg(test)]
#[async_trait]
impl StockListFetchTrait for MockFetcher {
    async fn list_active_codes(&self) -> Result<Vec<String>> {
        Ok(self.codes.clone())
    }
}
