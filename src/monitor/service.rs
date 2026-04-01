use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::core::Result;
use crate::monitor::models::{MonitorQuoteRow, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind};
use crate::watchlist::WatchlistListItem;

#[async_trait]
pub trait MonitorWatchlistReader: Send + Sync {
    async fn list_items(&self) -> Result<Vec<WatchlistListItem>>;
}

#[async_trait]
pub trait MonitorQuoteReader: Send + Sync {
    async fn load_quotes(&self, codes: &[String]) -> Result<Vec<MonitorQuoteRow>>;
}

#[async_trait]
pub trait MonitorAlertStore: Send + Sync {
    async fn add_alert(
        &self,
        code: &str,
        kind: PriceAlertKind,
        target_price: f64,
        now: DateTime<Utc>,
    ) -> Result<PriceAlert>;

    async fn list_alerts(&self) -> Result<Vec<PriceAlert>>;

    async fn remove_alert(&self, id: i64) -> Result<bool>;

    async fn mark_triggered(&self, id: i64, triggered_at: DateTime<Utc>) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub struct MonitorService<RW, RQ, RS> {
    watchlist_reader: RW,
    quote_reader: RQ,
    alert_store: RS,
}

impl<RW, RQ, RS> MonitorService<RW, RQ, RS>
where
    RW: MonitorWatchlistReader,
    RQ: MonitorQuoteReader,
    RS: MonitorAlertStore,
{
    pub fn new(watchlist_reader: RW, quote_reader: RQ, alert_store: RS) -> Self {
        Self {
            watchlist_reader,
            quote_reader,
            alert_store,
        }
    }

    pub async fn load_watchlist_snapshot(&self) -> Result<MonitorWatchlistSnapshot> {
        let items = self.watchlist_reader.list_items().await?;
        if items.is_empty() {
            return Ok(MonitorWatchlistSnapshot::default());
        }

        let codes = items.iter().map(|item| item.code.clone()).collect::<Vec<_>>();
        let quote_rows = self
            .quote_reader
            .load_quotes(&codes)
            .await?;
        let alerts = self.alert_store.list_alerts().await?;

        Ok(super::service_snapshot::build_watchlist_snapshot(
            items,
            quote_rows,
            alerts,
        ))
    }

    pub async fn add_alert(
        &self,
        code: &str,
        kind: PriceAlertKind,
        target_price: f64,
        now: DateTime<Utc>,
    ) -> Result<PriceAlert> {
        self.alert_store
            .add_alert(code, kind, target_price, now)
            .await
    }

    pub async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
        self.alert_store.list_alerts().await
    }

    pub async fn remove_alert(&self, id: i64) -> Result<bool> {
        self.alert_store.remove_alert(id).await
    }
}
