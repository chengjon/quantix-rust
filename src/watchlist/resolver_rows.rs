use futures_util::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

use crate::watchlist::WatchlistListItem;

use super::resolver::{
    WatchlistDisplayRow, WatchlistNameLookup, WatchlistQuoteLookup, WatchlistQuoteSnapshot,
};

pub(super) async fn resolve_display_rows(
    items: &[WatchlistListItem],
    with_price: bool,
    name_lookup: Arc<dyn WatchlistNameLookup>,
    quote_lookup: Arc<dyn WatchlistQuoteLookup>,
    name_lookup_timeout: Duration,
) -> Vec<WatchlistDisplayRow> {
    let quote_map = if with_price {
        let codes: Vec<String> = items.iter().map(|item| item.code.clone()).collect();
        quote_lookup.lookup_quotes(&codes).await.unwrap_or_default()
    } else {
        HashMap::new()
    };

    let name_map = join_all(items.iter().map(|item| {
        let code = item.code.clone();
        let name_lookup = Arc::clone(&name_lookup);

        async move {
            let name = match timeout(name_lookup_timeout, name_lookup.lookup_name(&code)).await {
                Ok(Ok(name)) => name,
                Ok(Err(_)) | Err(_) => None,
            };

            (code, name)
        }
    }))
    .await
    .into_iter()
    .collect::<HashMap<String, Option<String>>>();

    build_display_rows(items, &name_map, &quote_map)
}

fn build_display_rows(
    items: &[WatchlistListItem],
    name_map: &HashMap<String, Option<String>>,
    quote_map: &HashMap<String, WatchlistQuoteSnapshot>,
) -> Vec<WatchlistDisplayRow> {
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
