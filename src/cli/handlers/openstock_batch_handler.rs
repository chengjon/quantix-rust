//! P0.15b batch import handlers.
//!
//! Extracted from `openstock_handler.rs` to satisfy the 1200-line
//! force-split threshold in CLAUDE.md. These are the batch-related
//! symbols added by P0.15b:
//!
//! - `import_minute_klines_inner` — inner single-code kline import
//! - `import_minute_share_inner` — inner single-code share import
//! - `import_openstock_minute_all` — iterate all active codes
//! - `query_import_status` — read back `quantix.import_state`
//! - `print_summary_text` — text renderer for `BatchSummary`
//!
//! Pure mechanical move; no behavior change.

use crate::core::runtime::OpenStockSettings;
use crate::core::{QuantixError, Result};
use crate::sources::openstock_client::OpenStockClient;

/// Resolve the PostgreSQL connection URL for OpenStock batch commands.
///
/// Prefers `QUANTIX_POSTGRES_URL` if set (full URL override); otherwise
/// assembles one from the individual `POSTGRESQL_*` env vars with sensible
/// defaults (port 5438, database `quantix`). Returns `QuantixError::Config`
/// if any required var is missing.
pub(super) fn resolve_pg_url() -> Result<String> {
    if let Ok(url) = std::env::var("QUANTIX_POSTGRES_URL") {
        return Ok(url);
    }
    let host = std::env::var("POSTGRESQL_HOST")
        .map_err(|_| QuantixError::Config("POSTGRESQL_HOST not set".into()))?;
    let port = std::env::var("POSTGRESQL_PORT").unwrap_or_else(|_| "5438".into());
    let user = std::env::var("POSTGRESQL_USER")
        .map_err(|_| QuantixError::Config("POSTGRESQL_USER not set".into()))?;
    let pass = std::env::var("POSTGRESQL_PASSWORD")
        .map_err(|_| QuantixError::Config("POSTGRESQL_PASSWORD not set".into()))?;
    let db = std::env::var("POSTGRESQL_DATABASE").unwrap_or_else(|_| "quantix".into());
    Ok(format!("postgres://{user}:{pass}@{host}:{port}/{db}"))
}

/// Inner import logic — accepts already-constructed client + sink.
///
/// Used by both the CLI handler (which builds clients per invocation)
/// and the BatchScheduler (which builds clients once and reuses across
/// all codes). Bypasses all CLI output (`println!`) — the caller is
/// responsible for surfacing results.
///
/// `will_apply=true` triggers real ClickHouse writes; `false` is a
/// dry-run that streams through but writes nothing.
#[allow(dead_code, clippy::too_many_arguments)] // consumed by BatchScheduler in Task 7
pub(crate) async fn import_minute_klines_inner(
    client: &OpenStockClient,
    sink: &crate::db::clickhouse::ClickHouseMinuteKlineSink<'_>,
    code: &str,
    period: crate::data::models::MinutePeriod,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
    adjust: crate::data::models::AdjustType,
    will_apply: bool,
) -> Result<crate::db::clickhouse::StreamStats> {
    use crate::data::models::DateOrRange;
    use crate::db::clickhouse::stream_minute_klines_to_clickhouse;

    if !will_apply {
        // Dry-run path: stream and count, do not call the sink.
        // Mirrors the CLI dry-run branch but returns stats instead of printing.
        use futures::StreamExt;
        let dor = DateOrRange::Range { start, end };
        let s = client.fetch_minute_klines_stream(code, period, dor, adjust);
        futures::pin_mut!(s);
        let mut batches = 0u64;
        let mut total = 0u64;
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len() as u64;
        }
        return Ok(crate::db::clickhouse::StreamStats {
            batches,
            input_records: total,
            inserted_records: 0,
        });
    }

    let stats =
        stream_minute_klines_to_clickhouse(client, sink, code, period, start, end, adjust).await?;
    Ok(stats)
}

/// Inner import logic for minute share — accepts pre-built client + sink.
///
/// Mirrors `import_minute_klines_inner`. Used by both the CLI handler
/// and the BatchScheduler.
#[allow(dead_code)] // consumed by BatchScheduler in Task 7
pub(crate) async fn import_minute_share_inner(
    client: &OpenStockClient,
    sink: &crate::db::clickhouse::ClickHouseMinuteShareSink<'_>,
    code: &str,
    start: chrono::NaiveDate,
    end: chrono::NaiveDate,
    will_apply: bool,
) -> Result<crate::db::clickhouse::StreamStats> {
    use crate::data::models::DateOrRange;
    use crate::db::clickhouse::stream_minute_shares_to_clickhouse;

    if !will_apply {
        use futures::StreamExt;
        let dor = DateOrRange::Range { start, end };
        let s = client.fetch_minute_share_stream(code, dor);
        futures::pin_mut!(s);
        let mut batches = 0u64;
        let mut total = 0u64;
        while let Some(result) = s.next().await {
            let batch = result?;
            batches += 1;
            total += batch.len() as u64;
        }
        return Ok(crate::db::clickhouse::StreamStats {
            batches,
            input_records: total,
            inserted_records: 0,
        });
    }

    let stats = stream_minute_shares_to_clickhouse(client, sink, code, start, end).await?;
    Ok(stats)
}

/// P0.15b: `quantix data openstock import-minute-all`.
///
/// Iterates active codes from `quantix.stock_info`, runs P0.15a import
/// logic per code, tracks outcome in `quantix.import_state`. Default
/// behavior matches `import-minute-klines` re: env var
/// QUANTIX_OPENSTOCK_MINUTE_APPLY=yes.
pub(crate) async fn import_openstock_minute_all(
    settings: &OpenStockSettings,
    pg_url: &str,
    date: Option<String>,
    format: crate::cli::command_types::OutputFormat,
    dry_run: bool,
) -> Result<()> {
    use crate::cli::command_types::OutputFormat;
    use crate::data::models::{AdjustType, MinutePeriod};
    use crate::db::PostgresClient;
    use crate::tasks::openstock_import::fetcher::StockListFetcher;
    use crate::tasks::openstock_import::scheduler::BatchScheduler;
    use crate::tasks::openstock_import::state::ImportStateStore;
    use chrono::{Local, NaiveDate};
    use std::str::FromStr;

    let trade_date = match date.as_deref() {
        Some("today") | None => Local::now().date_naive(),
        Some(s) => {
            NaiveDate::from_str(s).map_err(|e| QuantixError::Config(format!("--date: {}", e)))?
        }
    };

    let will_apply = super::openstock_handler::compute_apply(true);
    let period = MinutePeriod::Minute5;
    let adjust = AdjustType::QFQ;

    let pg = PostgresClient::new(pg_url).await?;
    let fetcher = StockListFetcher::new(&pg);
    let state = ImportStateStore::new(&pg);

    let sched = BatchScheduler::new(&fetcher, &state, settings, period, adjust, will_apply);

    println!(
        "OpenStock import-minute-all ({})",
        if dry_run {
            "dry-run"
        } else if will_apply {
            "apply"
        } else {
            "no-env-apply"
        }
    );
    println!("  date: {}", trade_date);
    println!("  will_apply: {}", will_apply);

    let summary = sched.run(trade_date, dry_run).await?;

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&summary)
                .map_err(|e| QuantixError::Other(format!("json serialize: {}", e)))?;
            println!("{}", json);
        }
        OutputFormat::Text => {
            print_summary_text(&summary);
        }
    }
    Ok(())
}

/// P0.15b: `quantix data openstock import-status`.
///
/// Queries `quantix.import_state` for the given date, prints the latest
/// batch summary plus failure detail (code, kind, reason).
pub async fn query_import_status(
    pg_url: &str,
    date: String,
    format: crate::cli::command_types::OutputFormat,
) -> Result<()> {
    use crate::cli::command_types::OutputFormat;
    use crate::db::PostgresClient;
    use chrono::NaiveDate;
    use std::str::FromStr;

    let trade_date =
        NaiveDate::from_str(&date).map_err(|e| QuantixError::Config(format!("--date: {}", e)))?;

    let pg = PostgresClient::new(pg_url).await?;

    // Latest batch_id for this date (most recent imported_at).
    let batch_row: Option<(String,)> = sqlx::query_as(
        "SELECT batch_id FROM quantix.import_state \
         WHERE trade_date = $1 \
         GROUP BY batch_id \
         ORDER BY MAX(imported_at) DESC LIMIT 1",
    )
    .bind(trade_date)
    .fetch_optional(pg.pool())
    .await
    .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

    let batch_id = match batch_row {
        Some((id,)) => id,
        None => {
            let msg = format!("No import_state records for {}", trade_date);
            match format {
                OutputFormat::Json => {
                    println!(
                        "{{\"date\":\"{}\",\"found\":false,\"message\":\"{}\"}}",
                        trade_date, msg
                    );
                }
                OutputFormat::Text => println!("{}", msg),
            }
            return Ok(());
        }
    };

    // All rows for that batch.
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
        "SELECT code, kind, status, reason FROM quantix.import_state \
         WHERE trade_date = $1 AND batch_id = $2",
    )
    .bind(trade_date)
    .bind(&batch_id)
    .fetch_all(pg.pool())
    .await
    .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

    let mut success_klines = 0u32;
    let mut success_share = 0u32;
    let mut failed_klines = 0u32;
    let mut failed_share = 0u32;
    let mut failures: Vec<(String, String, String)> = Vec::new();
    for (code, kind, status, reason) in &rows {
        if status == "success" {
            if kind == "klines" {
                success_klines += 1;
            } else {
                success_share += 1;
            }
        } else {
            let reason_str = reason.clone().unwrap_or_else(|| "unknown".into());
            if kind == "klines" {
                failed_klines += 1;
            } else {
                failed_share += 1;
            }
            failures.push((code.clone(), kind.clone(), reason_str));
        }
    }

    match format {
        OutputFormat::Json => {
            let failures_json: Vec<serde_json::Value> = failures
                .into_iter()
                .map(|(code, kind, reason)| {
                    serde_json::json!({"code": code, "kind": kind, "reason": reason})
                })
                .collect();
            let payload = serde_json::json!({
                "date": trade_date.to_string(),
                "batch_id": batch_id,
                "success_count": {"klines": success_klines, "share": success_share},
                "failed_count": {"klines": failed_klines, "share": failed_share},
                "failures": failures_json,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&payload)
                    .map_err(|e| QuantixError::Other(format!("json: {}", e)))?
            );
        }
        OutputFormat::Text => {
            println!("Import status for {}", trade_date);
            println!();
            println!("  batch_id: {}", batch_id);
            println!(
                "    success: klines={} share={}",
                success_klines, success_share
            );
            println!(
                "    failed:  klines={} share={}",
                failed_klines, failed_share
            );
            if !failures.is_empty() {
                println!("  ── failed ──");
                for (code, kind, reason) in &failures {
                    println!("    {} {}: {}", code, kind, reason);
                }
            }
        }
    }
    Ok(())
}

/// Render a `BatchSummary` as human-readable text.
fn print_summary_text(summary: &crate::tasks::openstock_import::scheduler::BatchSummary) {
    println!("BatchSummary");
    println!("  batch_id: {}", summary.batch_id);
    println!("  date: {}", summary.date);
    println!("  started_at: {}", summary.started_at);
    if let Some(fin) = summary.finished_at {
        println!("  finished_at: {}", fin);
        let elapsed = fin.signed_duration_since(summary.started_at);
        println!("  elapsed: {:?}", elapsed);
    }
    println!("  total_codes: {}", summary.total_codes);
    println!(
        "  success: klines={} share={}",
        summary.success_count.klines, summary.success_count.share
    );
    println!(
        "  failed:  klines={} share={}",
        summary.failed_count.klines, summary.failed_count.share
    );
    if !summary.failures.is_empty() {
        println!("  ── failed detail ──");
        for f in &summary.failures {
            println!("  {} ({}): {}", f.code, f.kind, f.reason);
        }
    }
}
