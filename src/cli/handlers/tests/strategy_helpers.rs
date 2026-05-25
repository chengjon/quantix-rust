use super::*;

pub(super) fn make_kline(code: &str, day: u32, close: Decimal, volume: i64) -> Kline {
    Kline {
        code: code.to_string(),
        date: NaiveDate::from_ymd_opt(2024, 1, day).unwrap(),
        open: close,
        high: close + dec!(1),
        low: close - dec!(1),
        close,
        volume,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

pub(super) fn fixed_ts() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 17, 9, 30, 0).unwrap()
}

pub(super) fn sample_run(
    symbol: &str,
    bar_end: DateTime<Utc>,
) -> crate::execution::models::StrategyRunRecord {
    crate::execution::models::StrategyRunRecord {
        run_id: uuid::Uuid::new_v4().to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: crate::execution::models::StrategyRunStatus::Running,
        symbol: symbol.to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: serde_json::json!({}),
    }
}

pub(super) fn sample_signal(
    run_id: &str,
    signal_id: &str,
    bar_end: DateTime<Utc>,
) -> crate::execution::models::StrategySignalRecord {
    crate::execution::models::StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        signal_value: "buy".to_string(),
        signal_status: crate::execution::models::SignalStatus::New,
        approval_status: crate::execution::models::ApprovalStatus::Pending,
        run_id: run_id.to_string(),
        metadata_json: json!({
            "fast": 5,
            "slow": 20,
            "market_price": "12.34",
            "signal_value": "buy",
            "execution_policy": {
                "fixed_cash_per_buy": "10000",
                "slippage_bps": 0
            },
            "bar_source_id": "test-primary",
            "bar_source_fallback": false
        }),
        created_at: fixed_ts(),
        updated_at: fixed_ts(),
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FakeLoader {
    pub(super) data: HashMap<String, Vec<Kline>>,
}

impl StrategyBarLoadTelemetry for FakeLoader {
    fn last_source(&self) -> Option<crate::strategy::StrategyBarLoadSource> {
        Some(crate::strategy::StrategyBarLoadSource {
            source_id: "test-primary".to_string(),
            fallback_used: false,
        })
    }
}

#[async_trait]
impl DailyKlineLoader for FakeLoader {
    async fn load_daily_klines(
        &self,
        code: &str,
        lookback: usize,
    ) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > lookback {
            rows = rows[rows.len() - lookback..].to_vec();
        }
        Ok(rows)
    }
}

#[async_trait]
impl StrategyBarLoader for FakeLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

#[async_trait]
impl crate::risk::RiskBarLoader for FakeLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> crate::core::Result<Vec<Kline>> {
        let mut rows = self.data.get(code).cloned().unwrap_or_default();
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}
