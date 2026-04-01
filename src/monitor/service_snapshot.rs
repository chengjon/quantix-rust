use std::collections::{HashMap, HashSet};

use crate::monitor::models::{
    MonitorQuoteRow, MonitorWatchlistSnapshot, PriceAlert, PriceAlertKind, TriggeredAlert,
};
use crate::watchlist::WatchlistListItem;

pub(super) fn build_watchlist_snapshot(
    items: Vec<WatchlistListItem>,
    quote_rows: Vec<MonitorQuoteRow>,
    alerts: Vec<PriceAlert>,
) -> MonitorWatchlistSnapshot {
    let quote_map = quote_rows
        .into_iter()
        .map(|row| (row.code.clone(), row))
        .collect::<HashMap<_, _>>();

    let mut warnings = Vec::new();
    let mut rows = Vec::with_capacity(items.len());

    for item in items {
        let code = item.code.clone();
        rows.push(build_snapshot_row(item, quote_map.get(&code), &mut warnings));
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

    MonitorWatchlistSnapshot {
        rows,
        triggered_alerts,
        warnings,
    }
}

pub(super) fn build_snapshot_row(
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

pub(super) fn is_triggered(alert: &PriceAlert, current_price: f64) -> bool {
    match alert.kind {
        PriceAlertKind::Above => current_price >= alert.target_price,
        PriceAlertKind::Below => current_price <= alert.target_price,
    }
}
