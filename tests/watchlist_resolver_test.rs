use async_trait::async_trait;
use quantix_cli::watchlist::{
    WatchlistDisplayRow, WatchlistListItem, WatchlistNameLookup, WatchlistQuoteLookup,
    WatchlistQuoteSnapshot, WatchlistResolver,
};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, sleep, timeout};

#[derive(Clone)]
struct FakeNameLookup {
    names: HashMap<String, String>,
}

#[async_trait]
impl WatchlistNameLookup for FakeNameLookup {
    async fn lookup_name(&self, code: &str) -> quantix_cli::Result<Option<String>> {
        Ok(self.names.get(code).cloned())
    }
}

#[derive(Clone)]
struct FakeQuoteLookup {
    quotes: HashMap<String, WatchlistQuoteSnapshot>,
    should_fail: bool,
}

#[async_trait]
impl WatchlistQuoteLookup for FakeQuoteLookup {
    async fn lookup_quotes(
        &self,
        _codes: &[String],
    ) -> quantix_cli::Result<HashMap<String, WatchlistQuoteSnapshot>> {
        if self.should_fail {
            Err(quantix_cli::QuantixError::DataSource(
                "tdx unavailable".to_string(),
            ))
        } else {
            Ok(self.quotes.clone())
        }
    }
}

#[derive(Clone)]
struct SlowNameLookup {
    delay: Duration,
}

#[async_trait]
impl WatchlistNameLookup for SlowNameLookup {
    async fn lookup_name(&self, _code: &str) -> quantix_cli::Result<Option<String>> {
        sleep(self.delay).await;
        Ok(Some("slow-name".to_string()))
    }
}

fn item(code: &str, group: &str, tags: &[&str]) -> WatchlistListItem {
    WatchlistListItem {
        code: code.to_string(),
        group: group.to_string(),
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    }
}

#[tokio::test]
async fn resolver_returns_row_without_price_when_source_is_unavailable() {
    let resolver = WatchlistResolver::new(
        Arc::new(FakeNameLookup {
            names: HashMap::new(),
        }),
        Arc::new(FakeQuoteLookup {
            quotes: HashMap::new(),
            should_fail: true,
        }),
    );

    let rows = resolver
        .resolve_rows(&[item("000001", "default", &[])], true)
        .await;

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0],
        WatchlistDisplayRow {
            code: "000001".to_string(),
            name: None,
            group: "default".to_string(),
            tags: Vec::new(),
            latest_price: None,
            price_change_pct: None,
        }
    );
}

#[tokio::test]
async fn resolver_merges_group_tags_name_and_price_correctly() {
    let mut names = HashMap::new();
    names.insert("000001".to_string(), "平安银行".to_string());

    let mut quotes = HashMap::new();
    quotes.insert(
        "000001".to_string(),
        WatchlistQuoteSnapshot {
            latest_price: dec!(12.34),
            price_change_pct: Some(dec!(1.23)),
        },
    );

    let resolver = WatchlistResolver::new(
        Arc::new(FakeNameLookup { names }),
        Arc::new(FakeQuoteLookup {
            quotes,
            should_fail: false,
        }),
    );

    let rows = resolver
        .resolve_rows(&[item("000001", "core", &["bank", "longterm"])], true)
        .await;

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].name.as_deref(), Some("平安银行"));
    assert_eq!(rows[0].group, "core");
    assert_eq!(
        rows[0].tags,
        vec!["bank".to_string(), "longterm".to_string()]
    );
    assert_eq!(rows[0].latest_price, Some(dec!(12.34)));
    assert_eq!(rows[0].price_change_pct, Some(dec!(1.23)));
}

#[tokio::test]
async fn resolver_keeps_rows_when_some_quotes_are_missing() {
    let mut quotes = HashMap::new();
    quotes.insert(
        "000001".to_string(),
        WatchlistQuoteSnapshot {
            latest_price: dec!(10.01),
            price_change_pct: Some(dec!(0.11)),
        },
    );

    let resolver = WatchlistResolver::new(
        Arc::new(FakeNameLookup {
            names: HashMap::new(),
        }),
        Arc::new(FakeQuoteLookup {
            quotes,
            should_fail: false,
        }),
    );

    let rows = resolver
        .resolve_rows(
            &[
                item("000001", "default", &[]),
                item("600519", "core", &["whitehorse"]),
            ],
            true,
        )
        .await;

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].latest_price, Some(dec!(10.01)));
    assert_eq!(rows[1].code, "600519");
    assert_eq!(rows[1].latest_price, None);
    assert_eq!(rows[1].tags, vec!["whitehorse".to_string()]);
}

#[tokio::test]
async fn resolver_degrades_when_name_lookup_is_slow() {
    let resolver = WatchlistResolver::new(
        Arc::new(SlowNameLookup {
            delay: Duration::from_secs(3),
        }),
        Arc::new(FakeQuoteLookup {
            quotes: HashMap::new(),
            should_fail: false,
        }),
    );

    let rows = timeout(
        Duration::from_millis(1500),
        resolver.resolve_rows(&[item("000001", "default", &[])], true),
    )
    .await
    .expect("resolver should degrade instead of hanging");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].code, "000001");
    assert_eq!(rows[0].name, None);
}
