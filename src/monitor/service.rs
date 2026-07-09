use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

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
    /// 构造 MonitorService：注入 watchlist reader / quote reader / alert store；三层职责清晰（read-only 数据读取、行情拉取、告警持久化）。
    pub fn new(watchlist_reader: RW, quote_reader: RQ, alert_store: RS) -> Self {
        Self {
            watchlist_reader,
            quote_reader,
            alert_store,
        }
    }

    /// 拉取当前 watchlist 快照：列出 watchlist items、按 code 批量查最新行情、合并为 MonitorWatchlistSnapshot。行情缺失的 code 会带 warning 但不阻断；任一 reader 失败透传。
    pub async fn load_watchlist_snapshot(&self) -> Result<MonitorWatchlistSnapshot> {
        let items = self.watchlist_reader.list_items().await?;
        if items.is_empty() {
            return Ok(MonitorWatchlistSnapshot::default());
        }

        let codes = items
            .iter()
            .map(|item| item.code.clone())
            .collect::<Vec<_>>();
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
            rows.push(build_snapshot_row(
                item,
                quote_map.get(&code),
                &mut warnings,
            ));
        }

        let mut seen_alert_ids = HashSet::new();
        let mut triggered_alerts = Vec::new();
        for row in &rows {
            let Some(current_price) = row.last_price else {
                continue;
            };

            for alert in &alerts {
                if alert.code != row.code || !is_triggered(alert, current_price) {
                    continue;
                }
                if !seen_alert_ids.insert(alert.id) {
                    continue;
                }

                triggered_alerts.push(TriggeredAlert {
                    alert_id: alert.id,
                    code: alert.code.clone(),
                    kind: alert.kind,
                    target_price: alert.target_price,
                    current_price,
                    triggered_at: row.quote_time,
                });
            }
        }

        Ok(MonitorWatchlistSnapshot {
            rows,
            triggered_alerts,
            warnings,
        })
    }

    /// 添加价格告警：透传到 alert_store.add_alert，传入 code / kind（向上/向下突破）/ target_price / now；返回持久化后的 PriceAlert（含 id）。
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

    /// 列出所有价格告警（无过滤）；透传到 alert_store.list_alerts，顺序与底层一致。
    pub async fn list_alerts(&self) -> Result<Vec<PriceAlert>> {
        self.alert_store.list_alerts().await
    }

    /// 按 id 删除价格告警；返回是否真实删除了一行（true=命中并删除，false=id 不存在）。SQL 失败透传。
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
