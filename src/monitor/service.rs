use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::core::Result;
use crate::monitor::models::{
    MonitorQuoteRow, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind, TriggeredAlert,
};
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
        let quote_map = self
            .quote_reader
            .load_quotes(&codes)
            .await?
            .into_iter()
            .map(|row| (row.code.clone(), row))
            .collect::<HashMap<_, _>>();
        let alerts = self.alert_store.list_alerts().await?;

        let mut warnings = Vec::new();
        let mut rows = Vec::with_capacity(items.len());

        for item in items {
            let code = item.code.clone();
            rows.push(build_snapshot_row(item, quote_map.get(&code), &mut warnings));
        }

        let triggered_alerts = rows
            .iter()
            .flat_map(|row| match row.last_price {
                Some(current_price) => alerts
                    .iter()
                    .filter(move |alert| alert.code == row.code && is_triggered(alert, current_price))
                    .map(|alert| TriggeredAlert {
                        alert_id: alert.id,
                        code: alert.code.clone(),
                        kind: alert.kind,
                        target_price: alert.target_price,
                        current_price,
                        triggered_at: row.quote_time.unwrap_or(alert.created_at),
                    })
                    .collect::<Vec<_>>(),
                None => Vec::new(),
            })
            .collect();

        Ok(MonitorWatchlistSnapshot {
            rows,
            triggered_alerts,
            warnings,
        })
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

fn build_snapshot_row(
    item: WatchlistListItem,
    quote: Option<&MonitorQuoteRow>,
    warnings: &mut Vec<String>,
) -> MonitorQuoteRow {
    match quote {
        Some(quote) => MonitorQuoteRow {
            code: item.code,
            group: item.group,
            tags: item.tags,
            last_price: quote.last_price,
            change_pct: quote.change_pct,
            quote_time: quote.quote_time,
            note: quote.note.clone(),
        },
        None => {
            warnings.push(format!("{}: quote unavailable", item.code));
            MonitorQuoteRow {
                code: item.code,
                group: item.group,
                tags: item.tags,
                last_price: None,
                change_pct: None,
                quote_time: None,
                note: Some("quote unavailable".to_string()),
            }
        }
    }
}

fn is_triggered(alert: &PriceAlert, current_price: f64) -> bool {
    match alert.kind {
        PriceAlertKind::Above => current_price >= alert.target_price,
        PriceAlertKind::Below => current_price <= alert.target_price,
    }
}
