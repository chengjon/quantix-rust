pub use async_trait::async_trait;
pub use chrono::NaiveDate;
pub use polars::prelude::*;
pub use polars::prelude::{ParquetReader, SerReader};
pub use quantix_cli::cli::handlers::run_factor_command;
pub use quantix_cli::cli::{Cli, Commands, FactorCommands, FactorOutputFormat};
pub use quantix_cli::core::Result;
pub use quantix_cli::factor::{
    FactorCategory, FactorComputeRequest, FactorDataLoader, FactorDataset, FactorLoadRequest,
    FactorMeta, LayeredBacktestRequest, MissingPolicy, NeutralizationRequest,
    builtin_factor_catalog, cs_rank, evaluate_factor_ic, factor_ic_result_to_csv_string,
    factor_ic_result_to_parquet_file, factor_result_to_csv_string, factor_result_to_json_string,
    factor_result_to_parquet_file, factor_value_correlation, neutralize_factor_cross_sectional,
    run_layered_factor_backtest, ts_delay, ts_delta, ts_rank,
};
pub use std::collections::BTreeMap;
pub use std::fs::{File, write};

pub use clap::Parser;

pub(crate) struct MockFactorLoader {
    pub(crate) frame: DataFrame,
}

#[async_trait]
impl FactorDataLoader for MockFactorLoader {
    async fn load_bars(&self, _request: &FactorLoadRequest) -> Result<DataFrame> {
        Ok(self.frame.clone())
    }
}

pub(crate) fn mock_factor_frame() -> DataFrame {
    df!(
        "date" => &[
            "2026-01-01", "2026-01-01", "2026-01-01",
            "2026-01-02", "2026-01-02", "2026-01-02",
        ],
        "symbol" => &[
            "000001.SZ", "600000.SH", "000002.SZ",
            "000001.SZ", "600000.SH", "000002.SZ",
        ],
        "open" => &[10.0, 20.0, 30.0, 11.0, 21.0, 31.0],
        "high" => &[10.5, 20.5, 30.5, 11.5, 21.5, 31.5],
        "low" => &[9.5, 19.5, 29.5, 10.5, 20.5, 30.5],
        "close" => &[10.2, 20.2, 30.2, 11.2, 21.2, 31.2],
        "volume" => &[1000i64, 2000, 3000, 1100, 2100, 3100],
    )
    .unwrap()
}

pub(crate) fn mock_factor_frame_10d() -> DataFrame {
    let mut dates = Vec::new();
    let mut symbols = Vec::new();
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let universe = ["000001.SZ", "600000.SH", "000002.SZ"];

    for day in 1..=10 {
        for (idx, symbol) in universe.iter().enumerate() {
            let base = 10.0 + idx as f64 * 10.0 + day as f64;
            dates.push(format!("2026-01-{day:02}"));
            symbols.push((*symbol).to_string());
            open.push(base);
            high.push(base + 0.5);
            low.push(base - 0.5);
            close.push(if day == 5 && idx == 1 {
                None
            } else {
                Some(base + 0.2)
            });
            volume.push(1000i64 + day as i64 * 10 + idx as i64);
        }
    }

    df!(
        "date" => dates,
        "symbol" => symbols,
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume" => volume,
    )
    .unwrap()
}

pub(crate) fn mock_alpha101_frame() -> DataFrame {
    mock_alpha_frame_days(15)
}

pub(crate) fn mock_alpha_frame_days(days: usize) -> DataFrame {
    let mut dates = Vec::new();
    let mut symbols = Vec::new();
    let mut open = Vec::new();
    let mut high = Vec::new();
    let mut low = Vec::new();
    let mut close = Vec::new();
    let mut volume = Vec::new();
    let mut amount = Vec::new();
    let universe = ["000001.SZ", "600000.SH", "000002.SZ"];

    for day in 1..=days {
        for (idx, symbol) in universe.iter().enumerate() {
            let base = 20.0 + day as f64 * 0.15;
            let open_pattern = ((day * (idx + 2)) % 7) as f64;
            let close_pattern = (((day + idx) * (idx + 3)) % 9) as f64;
            dates.push(format!("2026-01-{day:02}"));
            symbols.push((*symbol).to_string());
            let open_value = base + open_pattern * 0.4 + idx as f64 * 0.05;
            let close_value = base + 0.2 + close_pattern * 0.35;
            open.push(open_value);
            high.push(open_value.max(close_value) + 0.8);
            low.push(open_value.min(close_value) - 0.6);
            close.push(close_value);
            let vol =
                1000i64 + ((day * day + idx * 37 + day * idx * 11 + (day % 4) * 29) % 500) as i64;
            volume.push(vol);
            amount.push(((open_value + close_value) / 2.0) * vol as f64);
        }
    }

    df!(
        "date" => dates,
        "symbol" => symbols,
        "open" => open,
        "high" => high,
        "low" => low,
        "close" => close,
        "volume" => volume,
        "amount" => amount,
    )
    .unwrap()
}
