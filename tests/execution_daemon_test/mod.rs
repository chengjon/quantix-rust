//! Execution-daemon integration tests, split by concern.
//!
//! Historical note: `execution_daemon_test.rs` was 668 lines and was split:
//! - [`config`] — default-execution-config persistence smoke test,
//! - [`daemon_run`] — the seven `daemon_run_once_*` consumption/diagnostic cases,
//! - [`execute_by_id`] — the two `execute_request_by_id_*` kill-switch blocking cases.
//!
//! Shared helpers (`fixed_ts`, `sample_run`, `sample_signal`, `seed_current_industry`,
//! `invalid_runtime_risk_state_path`) live here at module level.

use chrono::{TimeZone, Utc};
use quantix_cli::execution::config::{AutoApprovalMode, JsonExecutionConfigStore};
use quantix_cli::execution::daemon::{
    consume_next_pending_request_with_components,
    execute_request_by_id_with_components_and_kill_switch,
};
use quantix_cli::execution::models::{
    ApprovalStatus, ExecutionRequestStatus, SignalStatus, StrategyRunRecord, StrategyRunStatus,
    StrategySignalRecord,
};
use quantix_cli::execution::runtime_store::StrategyRuntimeStore;
use quantix_cli::risk::{JsonRiskStore, ShenwanCurrentSeedRow, SqliteIndustryStore};
use quantix_cli::safety::{JsonKillSwitchStore, KillSwitchState};
use quantix_cli::trade::{InitAccountRequest, JsonPaperTradeStore, PaperTradeStore, TradeService};
use serde_json::json;
use tempfile::tempdir;

pub(super) fn fixed_ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 23, 9, 30, 0).unwrap()
}

pub(super) fn sample_run(bar_end: chrono::DateTime<Utc>) -> StrategyRunRecord {
    StrategyRunRecord {
        run_id: "execution-daemon-run".to_string(),
        strategy_name: "ma_cross".to_string(),
        mode: "signal".to_string(),
        trigger: "daemon".to_string(),
        status: StrategyRunStatus::Running,
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        started_at: fixed_ts(),
        finished_at: None,
        metadata_json: json!({}),
    }
}

pub(super) fn sample_signal(
    run_id: &str,
    signal_id: &str,
    bar_end: chrono::DateTime<Utc>,
) -> StrategySignalRecord {
    StrategySignalRecord {
        signal_id: signal_id.to_string(),
        strategy_instance_id: "ma_fast_5_slow_20".to_string(),
        strategy_name: "ma_cross".to_string(),
        symbol: "000001".to_string(),
        timeframe: "1d".to_string(),
        bar_end,
        signal_value: "buy".to_string(),
        signal_status: SignalStatus::New,
        approval_status: ApprovalStatus::Pending,
        run_id: run_id.to_string(),
        metadata_json: json!({
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

pub(super) async fn seed_current_industry(
    risk_state_path: &std::path::Path,
    code: &str,
    industry_name: &str,
) {
    let store = SqliteIndustryStore::from_risk_state_path(risk_state_path)
        .await
        .unwrap();
    store
        .upsert_shenwan_current_rows(
            &[ShenwanCurrentSeedRow {
                security_code: code.to_string(),
                industry_name: industry_name.to_string(),
                source: "test_seed".to_string(),
            }],
            fixed_ts(),
        )
        .await
        .unwrap();
}

pub(super) fn invalid_runtime_risk_state_path(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let risk_dir = dir.path().join("invalid-runtime-risk");
    std::fs::create_dir_all(&risk_dir).unwrap();
    std::fs::create_dir_all(risk_dir.join("industry_reference.db")).unwrap();
    risk_dir.join("risk_state.json")
}

mod config;
mod daemon_run;
mod execute_by_id;
