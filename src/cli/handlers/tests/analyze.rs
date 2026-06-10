use super::*;
use crate::analysis::candle_patterns::{CandleInput, ReferencePricePolicy};
use crate::core::QuantixError;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

#[tokio::test]
async fn test_create_clickhouse_client_uses_runtime_settings() {
    let _lock = env_lock();
    let _guard = ClickHouseDbEnvGuard::capture();
    unsafe {
        std::env::set_var(CLICKHOUSE_URL_ENV, "http://runtime-host:8123");
        std::env::set_var(CLICKHOUSE_DB_ENV, "quantix_runtime_test");
        std::env::set_var(CLICKHOUSE_USER_ENV, "handler_user");
        std::env::set_var(CLICKHOUSE_PASSWORD_ENV, "handler_password");
    }

    let client = create_clickhouse_client().await.unwrap();
    assert_eq!(client.database(), "quantix_runtime_test");
    assert_eq!(client.http_auth_for_test().0, "handler_user");
    assert_eq!(client.http_auth_for_test().1, "handler_password");
}

#[test]
fn test_parse_candle_spec_parses_ohlc_values() {
    let candle = parse_candle_spec("10,12,8,10").unwrap();

    assert_eq!(candle.open, dec!(10));
    assert_eq!(candle.high, dec!(12));
    assert_eq!(candle.low, dec!(8));
    assert_eq!(candle.close, dec!(10));
}

#[test]
fn test_pattern_rows_from_klines_preserve_dates_and_ohlc() {
    let rows = pattern_rows_from_klines(&[Kline {
        code: "000001".to_string(),
        date: NaiveDate::from_ymd_opt(2026, 3, 17).unwrap(),
        open: dec!(10),
        high: dec!(12),
        low: dec!(8),
        close: dec!(10),
        volume: 100,
        amount: None,
        adjust_type: AdjustType::None,
    }]);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "2026-03-17");
    assert_eq!(rows[0].candle.open, dec!(10));
    assert_eq!(rows[0].candle.high, dec!(12));
    assert_eq!(rows[0].candle.low, dec!(8));
    assert_eq!(rows[0].candle.close, dec!(10));
}

#[test]
fn test_sequence_references_uses_previous_close_values() {
    let candles = vec![
        CandleInput {
            open: dec!(10),
            high: dec!(10),
            low: dec!(10),
            close: dec!(10),
        },
        CandleInput {
            open: dec!(10),
            high: dec!(12),
            low: dec!(10),
            close: dec!(12),
        },
        CandleInput {
            open: dec!(12),
            high: dec!(12),
            low: dec!(8),
            close: dec!(10),
        },
    ];

    let refs = sequence_references(&candles, &ReferencePricePolicy::PreviousClose).unwrap();

    assert_eq!(refs, vec![dec!(10), dec!(12)]);
}

#[test]
fn test_infer_tdx_code_from_day_file_path_extracts_six_digit_code() {
    assert_eq!(
        infer_tdx_code_from_day_file_path(
            "/mnt/d/ProgramData/tdx_20251231/vipdoc/sh/lday/sh000001.day"
        )
        .unwrap(),
        1
    );
    assert_eq!(
        infer_tdx_code_from_day_file_path("/tmp/sz300750.day").unwrap(),
        300750
    );
}

#[test]
fn test_pattern_rows_from_day_file_reads_and_limits_rows() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("sh000001.day");
    let mut bytes = Vec::new();

    bytes.extend(build_day_record_bytes(
        20260315, 1000, 1100, 900, 1050, 1000.0, 200, 980,
    ));
    bytes.extend(build_day_record_bytes(
        20260316, 1050, 1200, 1000, 1180, 1200.0, 220, 1050,
    ));
    std::fs::write(&path, bytes).unwrap();

    let rows = pattern_rows_from_day_file(&path, None, None, 1).unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, "2026-03-16");
    assert_eq!(rows[0].candle.open, dec!(10.5));
    assert_eq!(rows[0].candle.high, dec!(12));
    assert_eq!(rows[0].candle.low, dec!(10));
    assert_eq!(rows[0].candle.close, dec!(11.8));
}

#[test]
fn test_resolve_tdx_day_file_path_prefers_explicit_market() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let sh_dir = root.join("vipdoc").join("sh").join("lday");
    let sz_dir = root.join("vipdoc").join("sz").join("lday");
    std::fs::create_dir_all(&sh_dir).unwrap();
    std::fs::create_dir_all(&sz_dir).unwrap();
    std::fs::write(sh_dir.join("sh000001.day"), []).unwrap();
    std::fs::write(sz_dir.join("sz000001.day"), []).unwrap();

    let resolved = resolve_tdx_day_file_path(root, "000001", Some("sz")).unwrap();

    assert_eq!(resolved, sz_dir.join("sz000001.day"));
}

#[test]
fn test_resolve_tdx_day_file_path_rejects_ambiguous_market() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let sh_dir = root.join("vipdoc").join("sh").join("lday");
    let sz_dir = root.join("vipdoc").join("sz").join("lday");
    std::fs::create_dir_all(&sh_dir).unwrap();
    std::fs::create_dir_all(&sz_dir).unwrap();
    std::fs::write(sh_dir.join("sh000001.day"), []).unwrap();
    std::fs::write(sz_dir.join("sz000001.day"), []).unwrap();

    let error = resolve_tdx_day_file_path(root, "000001", None).unwrap_err();

    assert!(error.to_string().contains("匹配到多个"));
}

#[allow(clippy::too_many_arguments)]
fn build_day_record_bytes(
    date: u32,
    open: u32,
    high: u32,
    low: u32,
    close: u32,
    amount: f32,
    volume: u32,
    prev_close: u32,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);
    buf.extend(date.to_le_bytes());
    buf.extend(open.to_le_bytes());
    buf.extend(high.to_le_bytes());
    buf.extend(low.to_le_bytes());
    buf.extend(close.to_le_bytes());
    buf.extend(amount.to_le_bytes());
    buf.extend(volume.to_le_bytes());
    buf.extend(prev_close.to_le_bytes());
    buf
}

#[test]
fn test_task_add_is_explicitly_unsupported() {
    let err = ensure_task_command_supported_for_p0(&TaskCommands::Add {
        name: "demo".to_string(),
        cron: "0 * * * *".to_string(),
        command: "echo demo".to_string(),
    })
    .unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
}

#[test]
fn test_task_start_daemon_is_explicitly_unsupported() {
    let err =
        ensure_task_command_supported_for_p0(&TaskCommands::Start { daemon: true }).unwrap_err();

    assert!(matches!(err, QuantixError::Unsupported(_)));
}

#[test]
fn test_foundation_p0_task_templates_match_scheduler_templates() {
    let templates = foundation_p0_task_template_descriptions();

    assert_eq!(
        templates,
        vec![
            (
                "pre_market_check".to_string(),
                "检查盘前数据".to_string(),
                "0 8 * * 1-5".to_string()
            ),
            (
                "auction_collection".to_string(),
                "竞价数据采集".to_string(),
                "30,0 9 * * 1-5".to_string()
            ),
            (
                "market_open".to_string(),
                "开盘检查".to_string(),
                "30 9 * * 1-5".to_string()
            ),
            (
                "market_close".to_string(),
                "收盘检查".to_string(),
                "0 15 * * 1-5".to_string()
            ),
            (
                "post_market_process".to_string(),
                "盘后数据处理".to_string(),
                "30 15 * * 1-5".to_string()
            ),
            (
                "data_sync".to_string(),
                "数据同步".to_string(),
                "0 16 * * *".to_string()
            ),
        ]
    );
}
