use async_trait::async_trait;
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use quantix_cli::core::Result;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::risk::{
    ProjectedBuyImpact, RiskAccountSnapshot, RiskService, RiskState, RiskStore,
    VOLATILITY_REQUIRED_BARS,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
struct FakeRiskStore {
    state: Arc<Mutex<Option<RiskState>>>,
}

impl FakeRiskStore {
    fn snapshot(&self) -> Option<RiskState> {
        self.state.lock().unwrap().clone()
    }
}

#[async_trait]
impl RiskStore for FakeRiskStore {
    async fn load_state(&self) -> Result<Option<RiskState>> {
        Ok(self.snapshot())
    }

    async fn save_state(&self, state: &RiskState) -> Result<()> {
        *self.state.lock().unwrap() = Some(state.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct FakeRiskBarLoader {
    bars: Vec<Kline>,
}

#[async_trait]
impl quantix_cli::risk::RiskBarLoader for FakeRiskBarLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        let mut rows: Vec<Kline> = self
            .bars
            .iter()
            .filter(|bar| bar.code == code)
            .cloned()
            .collect();
        if rows.len() > limit {
            rows = rows.split_off(rows.len() - limit);
        }
        Ok(rows)
    }
}

fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 24, 9, 35, 0).unwrap()
}

fn snapshot(total_assets: Decimal, positions: &[(&str, Decimal)]) -> RiskAccountSnapshot {
    RiskAccountSnapshot::new(
        "default",
        total_assets,
        positions
            .iter()
            .map(|(code, market_value)| ((*code).to_string(), *market_value))
            .collect(),
    )
}

fn bar(date: NaiveDate, close: Decimal, spread: Decimal) -> Kline {
    Kline {
        code: "000001".to_string(),
        date,
        open: close,
        high: close + spread,
        low: close - spread,
        close,
        volume: 1_000_000,
        amount: Some(close * Decimal::from(1_000_000)),
        adjust_type: AdjustType::None,
    }
}

fn bars_with_spread(spread: Decimal, count: usize) -> Vec<Kline> {
    (0..count)
        .map(|offset| {
            bar(
                NaiveDate::from_ymd_opt(2026, 3, 1)
                    .unwrap()
                    .checked_add_signed(Duration::days(offset as i64))
                    .unwrap(),
                dec!(100),
                spread,
            )
        })
        .collect()
}

#[tokio::test]
async fn volatility_limit_allows_buy_when_atr_ratio_is_below_threshold() {
    let store = FakeRiskStore::default();
    let service = RiskService::with_bar_loader(
        store,
        FakeRiskBarLoader {
            bars: bars_with_spread(dec!(1), 15),
        },
    );

    service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();

    service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(999500)),
            fixed_ts() + Duration::minutes(1),
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn volatility_limit_rejects_buy_when_atr_ratio_exceeds_threshold() {
    let store = FakeRiskStore::default();
    let service = RiskService::with_bar_loader(
        store.clone(),
        FakeRiskBarLoader {
            bars: bars_with_spread(dec!(6), 15),
        },
    );

    service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();
    let before = store.snapshot().unwrap();
    let before_event_count = before.events.len();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(999500)),
            fixed_ts() + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("volatility-limit"));
    assert!(err.to_string().contains("000001"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before_event_count);
}

#[tokio::test]
async fn volatility_limit_rejects_buy_when_available_bars_are_insufficient() {
    let store = FakeRiskStore::default();
    let service = RiskService::with_bar_loader(
        store.clone(),
        FakeRiskBarLoader {
            bars: bars_with_spread(dec!(1), 14),
        },
    );

    service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();
    let before = store.snapshot().unwrap();
    let before_event_count = before.events.len();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(999500)),
            fixed_ts() + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("volatility-limit"));
    assert!(err.to_string().contains("检查失败"));

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before_event_count);
}

#[tokio::test]
async fn volatility_limit_rejects_buy_when_loader_returns_no_bars() {
    let store = FakeRiskStore::default();
    let service =
        RiskService::with_bar_loader(store.clone(), FakeRiskBarLoader { bars: Vec::new() });

    service
        .set_rule("volatility-limit", "4%", fixed_ts())
        .await
        .unwrap();
    let before = store.snapshot().unwrap();
    let before_event_count = before.events.len();

    let err = service
        .check_buy(
            &snapshot(dec!(1000000), &[("000001", dec!(100000))]),
            &ProjectedBuyImpact::new("000001", dec!(150000), dec!(999500)),
            fixed_ts() + Duration::minutes(1),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("volatility-limit"));
    assert!(err.to_string().contains("可用日线不足"));
    assert!(
        err.to_string()
            .contains(&format!("至少需要 {} 条", VOLATILITY_REQUIRED_BARS))
    );

    let state = store.snapshot().unwrap();
    assert!(!state.buy_lock.locked);
    assert_eq!(state.events.len(), before_event_count);
}
