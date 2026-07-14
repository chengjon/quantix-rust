//! Strategy-daemon integration tests, split by concern.
//!
//! Historical note: `strategy_daemon_test.rs` was 631 lines and was split:
//! - [`config_registry`] — `JsonStrategyConfigStore` persistence plus the
//!   `StrategyRegistry` build/evaluator cases,
//! - [`fallback`] — the four `FallbackStrategyBarLoader` fallback cases,
//! - [`daemon`] — the `StrategySignalDaemon` run/checkpoint/hot-reload cases and
//!   the two `auto_approval_*` execution-config cases.
//!
//! Shared helpers (`kline`, `configured_ma_cross`, `FakeBarLoader`,
//! `ErrorBarLoader`, `write_day_file`) live here.

use async_trait::async_trait;
use chrono::NaiveDate;
use quantix_cli::data::models::{AdjustType, Kline};
use quantix_cli::execution::config::{AutoApprovalMode, JsonExecutionConfigStore};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::strategy::daemon::StrategyBarLoadTelemetry;
use quantix_cli::strategy::runtime::StrategyBarLoader;
use quantix_cli::strategy::{
    BootstrapPolicy, JsonStrategyConfigStore, StrategyBarLoadSource, StrategyRegistry,
};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::tempdir;

pub(super) fn kline(day: u32, close: i64) -> Kline {
    Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2026, 3, day).unwrap(),
        open: dec!(1),
        high: dec!(1),
        low: dec!(1),
        close: rust_decimal::Decimal::from(close),
        volume: 100,
        amount: None,
        adjust_type: AdjustType::None,
    }
}

pub(super) fn configured_ma_cross(
    params: serde_json::Value,
) -> quantix_cli::strategy::ConfiguredStrategyInstance {
    quantix_cli::strategy::ConfiguredStrategyInstance {
        id: "ma_cross_test".to_string(),
        name: "ma_cross".to_string(),
        enabled: true,
        params,
    }
}

#[derive(Clone, Default)]
pub(super) struct FakeBarLoader {
    bars: Arc<Mutex<HashMap<String, Vec<Kline>>>>,
}

#[async_trait]
impl StrategyBarLoader for FakeBarLoader {
    async fn load_daily_bars(
        &self,
        code: &str,
        limit: usize,
    ) -> quantix_cli::core::Result<Vec<Kline>> {
        let bars = self
            .bars
            .lock()
            .unwrap()
            .get(code)
            .cloned()
            .unwrap_or_default();
        if bars.len() > limit {
            Ok(bars[bars.len() - limit..].to_vec())
        } else {
            Ok(bars)
        }
    }
}

impl FakeBarLoader {
    pub(super) fn set_bars(&self, code: &str, bars: Vec<Kline>) {
        self.bars.lock().unwrap().insert(code.to_string(), bars);
    }
}

impl StrategyBarLoadTelemetry for FakeBarLoader {
    fn last_source(&self) -> Option<StrategyBarLoadSource> {
        Some(StrategyBarLoadSource {
            source_id: "test-primary".to_string(),
            fallback_used: false,
        })
    }
}

#[derive(Clone)]
pub(super) struct ErrorBarLoader;

#[async_trait]
impl StrategyBarLoader for ErrorBarLoader {
    async fn load_daily_bars(
        &self,
        _code: &str,
        _limit: usize,
    ) -> quantix_cli::core::Result<Vec<Kline>> {
        Err(quantix_cli::core::QuantixError::DataSource(
            "primary loader failed".to_string(),
        ))
    }
}

pub(super) fn write_day_file(root: &std::path::Path, market: &str, code: &str, closes: &[u32]) {
    let dir = root.join("vipdoc").join(market).join("lday");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{}{}.day", market, code));
    let mut bytes = Vec::new();

    for (index, close) in closes.iter().enumerate() {
        let date = 20260301u32 + index as u32;
        bytes.extend_from_slice(&date.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&close.to_le_bytes());
        bytes.extend_from_slice(&(1000.0f32).to_le_bytes());
        bytes.extend_from_slice(&(100u32).to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }

    fs::write(path, bytes).unwrap();
}

mod config_registry;
mod daemon;
mod fallback;
