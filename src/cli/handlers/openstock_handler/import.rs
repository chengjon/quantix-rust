//! Auto-extracted child module of openstock_handler.rs
use super::*;

/// P0.15a: `quantix data openstock import-minute-klines`.
///
/// Persists minute klines to ClickHouse `minute_klines` (P0.14 table) for a
/// single code + date range. Default is dry-run (stream + count, no
/// ClickHouse). Writes occur iff `--apply == true` AND
/// `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (see `compute_apply`).
///
/// Stream-only (P0.13d weekly chunking); never uses the batch API.
/// Partial failure leaves committed batches in place (INV-FLOW-1).
pub(crate) async fn import_openstock_minute_klines(
    settings: &OpenStockSettings,
    code: String,
    period: String,
    adjust: String,
    start: Option<String>,
    end: Option<String>,
    apply: bool,
) -> Result<()> {
    use crate::data::models::{AdjustType, DateOrRange, MinutePeriod};
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;
    use std::str::FromStr;

    let period_enum = MinutePeriod::from_str(&period)
        .map_err(|e| QuantixError::Config(format!("--period: {}", e)))?;
    let adjust_enum = AdjustType::from_str(&adjust)
        .map_err(|e| QuantixError::Config(format!("--adjust: {}", e)))?;
    let dor = DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?;
    let (start_date, end_date) = match dor {
        DateOrRange::Range { start, end } => (start, end),
        DateOrRange::Date(_) => {
            return Err(QuantixError::Config(
                "internal: DateOrRange unexpectedly Date".into(),
            ));
        }
    };

    let client = OpenStockClient::from_settings(settings)?;
    let will_apply = compute_apply(apply);

    println!(
        "OpenStock import-minute-klines ({})",
        if will_apply { "apply" } else { "dry-run" }
    );
    println!(
        "  code: {}, period: {}, adjust: {}",
        code,
        period_enum.as_str(),
        adjust_enum.as_str()
    );
    println!("  range: {} .. {}", start_date, end_date);

    if !will_apply {
        eprintln!("  Streaming weekly chunks (counting only, no ClickHouse writes):");
        let s = client.fetch_minute_klines_stream(&code, period_enum, dor.clone(), adjust_enum);
        futures::pin_mut!(s);
        let mut batches = 0usize;
        let mut total = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] would_insert: +{} (cumulative: {})",
                batches,
                batch.len(),
                total
            );
        }
        println!("  dry_run: true, applied: false");
        println!("  would_insert_total: {}", total);
        println!("  batches: {}, elapsed: {:?}", batches, started.elapsed());
        if apply {
            // --apply was set but env var was not "yes" — give the operator a hint.
            println!("  hint: set {}=yes to actually insert", MINUTE_APPLY_ENV);
        }
        return Ok(());
    }

    // Apply branch — construct ClickHouse client + sink, call P0.14 consumer.
    use crate::db::ClickHouseClient;
    use crate::db::clickhouse::{ClickHouseMinuteKlineSink, stream_minute_klines_to_clickhouse};

    let ch = ClickHouseClient::with_default_config().await?;
    // Lifetime is inferred: ClickHouseMinuteKlineSink<'a> borrows from `ch`.
    // `ch` and `sink` both live in this scope, outliving the await below.
    let sink = ClickHouseMinuteKlineSink {
        client: ch.client(),
    };
    let stats = stream_minute_klines_to_clickhouse(
        &client,
        &sink,
        &code,
        period_enum,
        start_date,
        end_date,
        adjust_enum,
    )
    .await?;
    println!("  batches: {}", stats.batches);
    println!("  input_records: {}", stats.input_records);
    println!("  inserted_records: {}", stats.inserted_records);
    println!("  dry_run: false, applied: true");
    Ok(())
}

/// P0.15a: `quantix data openstock import-minute-share`.
///
/// Persists minute shares (time-share ticks) to ClickHouse `minute_shares`
/// (P0.14 table) for a single code + date range. Default is dry-run.
/// Writes occur iff `--apply == true` AND
/// `env QUANTIX_OPENSTOCK_MINUTE_APPLY == "yes"` (see `compute_apply`).
///
/// Stream-only (P0.13d weekly chunking). Partial failure leaves committed
/// batches in place (INV-FLOW-1).
pub(crate) async fn import_openstock_minute_share(
    settings: &OpenStockSettings,
    code: String,
    start: Option<String>,
    end: Option<String>,
    apply: bool,
) -> Result<()> {
    use crate::data::models::DateOrRange;
    use crate::sources::openstock_client::OpenStockClient;
    use futures::StreamExt;

    let dor = DateOrRange::from_cli(None, start.as_deref(), end.as_deref())?;
    let (start_date, end_date) = match dor {
        DateOrRange::Range { start, end } => (start, end),
        DateOrRange::Date(_) => {
            return Err(QuantixError::Config(
                "internal: DateOrRange unexpectedly Date".into(),
            ));
        }
    };

    let client = OpenStockClient::from_settings(settings)?;
    let will_apply = compute_apply(apply);

    println!(
        "OpenStock import-minute-share ({})",
        if will_apply { "apply" } else { "dry-run" }
    );
    println!("  code: {}", code);
    println!("  range: {} .. {}", start_date, end_date);

    if !will_apply {
        eprintln!("  Streaming weekly chunks (counting only, no ClickHouse writes):");
        let s = client.fetch_minute_share_stream(&code, dor.clone());
        futures::pin_mut!(s);
        let mut batches = 0usize;
        let mut total = 0usize;
        let started = std::time::Instant::now();
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len();
            eprintln!(
                "  [batch {}] would_insert: +{} (cumulative: {})",
                batches,
                batch.len(),
                total
            );
        }
        println!("  dry_run: true, applied: false");
        println!("  would_insert_total: {}", total);
        println!("  batches: {}, elapsed: {:?}", batches, started.elapsed());
        if apply {
            println!("  hint: set {}=yes to actually insert", MINUTE_APPLY_ENV);
        }
        return Ok(());
    }

    use crate::db::ClickHouseClient;
    use crate::db::clickhouse::{ClickHouseMinuteShareSink, stream_minute_shares_to_clickhouse};

    let ch = ClickHouseClient::with_default_config().await?;
    let sink = ClickHouseMinuteShareSink {
        client: ch.client(),
    };
    let stats =
        stream_minute_shares_to_clickhouse(&client, &sink, &code, start_date, end_date).await?;
    println!("  batches: {}", stats.batches);
    println!("  input_records: {}", stats.input_records);
    println!("  inserted_records: {}", stats.inserted_records);
    println!("  dry_run: false, applied: true");
    Ok(())
}

// ============================================================================
// OpenStock import-* canonical paths.
// Reachable via DataCommands::ImportTicks / ImportKlines.
// ============================================================================

/// `quantix data import-ticks` (OpenStock only).
///
/// Writes to TDengine
/// gated by `--apply` + `QUANTIX_OPENSTOCK_TICK_APPLY=yes`; default dry-run.
pub(crate) async fn import_openstock_ticks(
    code: &str,
    date: Option<&str>,
    apply: bool,
) -> Result<()> {
    use crate::core::config::AppConfig;
    use crate::db::TDengineClient;
    use crate::sources::openstock_ticks::parse_tick_data;

    let osc = OpenStockClient::from_env()?;
    let resp = osc
        .fetch_tick_data(code, date)
        .await
        .map_err(|e| QuantixError::Other(format!("fetch_tick_data: {e}")))?;
    let envelope = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some("TICK_DATA".to_string()),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let (meta, ticks) = parse_tick_data(envelope)
        .map_err(|e| QuantixError::DataParse(format!("parse_tick_data: {e}")))?;

    println!("OpenStock import-ticks dry-run (category=TICK_DATA)");
    println!("  代码:    {}", code);
    println!("  日期:    {}", date.unwrap_or("(latest)"));
    println!("  来源:    {}", resp.source);
    println!("  Tick 数: {}", ticks.len());
    if let Some(trading_date) = meta.trading_date.as_deref() {
        println!("  交易日:  {}", trading_date);
    }
    if let Some(first) = ticks.first() {
        println!(
            "  首条:    {} price={} vol={} amount={} dir={:?}",
            first.timestamp, first.price, first.volume, first.amount, first.direction
        );
    }
    if let Some(last) = ticks.last() {
        println!(
            "  末条:    {} price={} vol={} amount={} dir={:?}",
            last.timestamp, last.price, last.volume, last.amount, last.direction
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    if let Some(ms) = resp.latency_ms {
        println!("  latency_ms:    {}", ms);
    }

    if ticks.is_empty() {
        println!("  → 无 tick 数据; 跳过写入");
        return Ok(());
    }

    if !apply {
        println!("  → dry-run; 加 --apply 实际写入 (需 QUANTIX_OPENSTOCK_TICK_APPLY=yes)");
        return Ok(());
    }
    if std::env::var("QUANTIX_OPENSTOCK_TICK_APPLY")
        .ok()
        .as_deref()
        != Some("yes")
    {
        return Err(QuantixError::Other(
            "已 --apply 但 QUANTIX_OPENSTOCK_TICK_APPLY != yes; 拒绝写入 TDengine".to_string(),
        ));
    }

    let config =
        AppConfig::load("config").map_err(|e| QuantixError::Other(format!("加载配置失败: {e}")))?;
    let td = config
        .database
        .tdengine
        .ok_or_else(|| QuantixError::Config("缺少 TDengine 配置".to_string()))?;
    let token = format!("{}:{}", td.username, td.password);
    let tde = TDengineClient::new_with_database(
        &format!("http://{}:{}", td.host, td.port),
        &token,
        &td.database,
    )?;
    tde.check_connection().await?;
    tde.create_tick_table().await?;

    let rows: Vec<(i64, f64, i32, f64, i32)> = ticks
        .iter()
        .map(|t| {
            let ts_ms = t.timestamp.and_utc().timestamp_millis();
            let price_f = super::decimal_to_f64(t.price, "import-ticks")?;
            let amount_f = super::decimal_to_f64(t.amount, "import-ticks")?;
            // Maps TradeDirection → direction TINYINT byte (TDengine schema).
            let status_i = match t.direction {
                crate::data::models::TradeDirection::Buy => 1,
                crate::data::models::TradeDirection::Sell => -1,
                crate::data::models::TradeDirection::Neutral => 0,
            };
            Ok::<(i64, f64, i32, f64, i32), QuantixError>((
                ts_ms,
                price_f,
                t.volume as i32,
                amount_f,
                status_i,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    tde.insert_ticks(code, &rows).await?;
    println!(
        "  → 已写入 TDengine ({} 条 tick, source=OPENSTOCK)",
        rows.len()
    );
    Ok(())
}

/// `quantix data import-klines` (OpenStock only).
///
/// Writes to ClickHouse
/// `kline_data` table gated by `--apply` + `QUANTIX_OPENSTOCK_KLINE_APPLY=yes`;
/// default dry-run.
pub(crate) async fn import_openstock_klines(
    code: &str,
    kline_type: &str,
    start: Option<&str>,
    end: Option<&str>,
    apply: bool,
) -> Result<()> {
    use crate::db::ClickHouseClient;
    use crate::sources::openstock_index::parse_index_klines;

    // 选择 category: 指数代码 (sh/sz/cn 前缀) 用 INDEX_KLINES,
    // 其余股票代码用 HISTORICAL_KLINES。
    let is_index = code.starts_with("sh.") || code.starts_with("sz.") || code.starts_with("cn.");
    let osc = OpenStockClient::from_env()?;
    let resp = if is_index {
        osc.fetch_index_klines(code, start, end).await?
    } else {
        osc.fetch_historical_klines(code, start, end).await?
    };

    let envelope = OpenStockEnvelope {
        data: resp.records,
        source: Some(resp.source.clone()),
        data_category: Some(
            if is_index {
                "INDEX_KLINES"
            } else {
                "HISTORICAL_KLINES"
            }
            .to_string(),
        ),
        request_id: None,
        route_decision_id: None,
        quality_flags: Vec::new(),
        cache_state: None,
        circuit_state: None,
        latency_ms: resp.latency_ms,
        received_at: resp.received_at.clone(),
    };
    let klines =
        parse_index_klines(envelope).map_err(|e| QuantixError::DataParse(e.to_string()))?;

    println!(
        "OpenStock import-klines dry-run (category={})",
        if is_index {
            "INDEX_KLINES"
        } else {
            "HISTORICAL_KLINES"
        }
    );
    println!("  代码:    {}", code);
    println!("  来源:    {}", resp.source);
    println!("  记录数:  {}", klines.len());
    if let Some(first) = klines.first() {
        println!(
            "  首条:    {} O={} H={} L={} C={}",
            first.date, first.open, first.high, first.low, first.close
        );
    }
    if let Some(last) = klines.last() {
        println!(
            "  末条:    {} O={} H={} L={} C={}",
            last.date, last.open, last.high, last.low, last.close
        );
    }
    println!("  artifact_hash: {}", resp.artifact_hash);
    if let Some(ms) = resp.latency_ms {
        println!("  latency_ms:    {}", ms);
    }

    if !apply {
        println!("  → dry-run; 加 --apply 实际写入 (需 QUANTIX_OPENSTOCK_KLINE_APPLY=yes)");
        return Ok(());
    }
    if std::env::var("QUANTIX_OPENSTOCK_KLINE_APPLY")
        .ok()
        .as_deref()
        != Some("yes")
    {
        return Err(QuantixError::Other(
            "已 --apply 但 QUANTIX_OPENSTOCK_KLINE_APPLY != yes; 拒绝写入 kline_data 主表"
                .to_string(),
        ));
    }

    let ch = ClickHouseClient::with_default_config().await?;
    ch.check_connection().await?;
    ch.insert_kline_data_batch_with_source(&klines, kline_type, "OPENSTOCK")
        .await?;
    println!(
        "  → 已写入 ClickHouse kline_data ({} 条, source=OPENSTOCK)",
        klines.len()
    );
    Ok(())
}
