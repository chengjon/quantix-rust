//! OpenStock shadow-write handlers.
//!
//! Extracted from `openstock_handler.rs` to respect the 1200-line force-split
//! gate. Houses the shadow-persist / rollback / verify entry points and their
//! private helpers (`read_payload`, `shadow_client`, `shadow_env_confirmed`,
//! `map_shadow_write_error`).

use std::fs;
use std::io::Read;

use crate::core::runtime::ClickHouseSettings;
use crate::core::{QuantixError, Result};
use crate::db::clickhouse::ClickHouseClient;
use crate::sources::openstock::{
    LiveShadowRequest, live_shadow_error_into_quantix, validate_live_shadow_payload,
};
use crate::sources::openstock_shadow::{
    ShadowWriteError, new_batch_id, rollback_shadow_batch, verify_shadow_batch, write_shadow_klines,
};

const SHADOW_ENV_CONFIRM: &str = "QUANTIX_SHADOW_PERSIST_CONFIRM";
const SHADOW_INGESTED_BY: &str = "quantix-cli";

pub(crate) fn read_payload(payload_path: &str) -> Result<String> {
    if payload_path == "-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| QuantixError::Other(format!("读取 stdin 失败: {}", error)))?;
        Ok(buffer)
    } else {
        fs::read_to_string(payload_path).map_err(|error| {
            QuantixError::Other(format!(
                "读取 OpenStock 线上响应失败 ({}): {}",
                payload_path, error
            ))
        })
    }
}

async fn shadow_client() -> Result<ClickHouseClient> {
    let settings = ClickHouseSettings::from_env();
    ClickHouseClient::from_settings(&settings)
        .await
        .map_err(|e| QuantixError::Other(format!("创建 ClickHouse 客户端失败: {}", e)))
}

fn shadow_env_confirmed() -> bool {
    std::env::var(SHADOW_ENV_CONFIRM).ok().as_deref() == Some("yes")
}

pub(super) fn map_shadow_write_error(error: ShadowWriteError) -> QuantixError {
    let msg = match error {
        ShadowWriteError::ApplyFlagRequired => {
            "shadow 写入需要 --apply 标志（当前仅 dry-run）".to_string()
        }
        ShadowWriteError::EnvConfirmRequired => format!(
            "shadow 写入需要环境变量 {}=yes（双保险未通过）",
            SHADOW_ENV_CONFIRM
        ),
        ShadowWriteError::FailClosedNotEmpty { count } => {
            format!("shadow 拒绝写入：{} 条 fail-closed 解析错误", count)
        }
        ShadowWriteError::DriftNotEmpty { count } => {
            format!(
                "shadow 拒绝写入：{} 条 drift（请求与服务端返回不一致）",
                count
            )
        }
        ShadowWriteError::EmptyPayload => "shadow 拒绝写入：映射后 0 行".to_string(),
        ShadowWriteError::MappedCountMismatch {
            record_count,
            mapped_count,
        } => format!(
            "shadow 拒绝写入：record_count={} 与 mapped_count={} 不一致",
            record_count, mapped_count
        ),
        ShadowWriteError::DuplicateKeys { count } => {
            format!(
                "shadow 拒绝写入：{} 条重复 (source, period, code, date, adjust_type) 键",
                count
            )
        }
        ShadowWriteError::DbError(inner) => format!("shadow ClickHouse 错误：{}", inner),
    };
    QuantixError::Other(msg)
}

pub(crate) async fn persist_openstock_live(
    payload_path: &str,
    symbol: &str,
    period: &str,
    start: &str,
    end: &str,
    limit: Option<u32>,
    apply: bool,
) -> Result<()> {
    let payload = read_payload(payload_path)?;
    let request = LiveShadowRequest {
        symbol: symbol.to_string(),
        period: period.to_string(),
        start_date: start.to_string(),
        end_date: end.to_string(),
        limit,
    };
    let report =
        validate_live_shadow_payload(&payload, &request).map_err(live_shadow_error_into_quantix)?;
    let batch_id = new_batch_id();
    let env_confirmed = shadow_env_confirmed();

    let client = shadow_client().await?;
    let write_report = write_shadow_klines(
        &client,
        &report,
        &payload,
        &batch_id,
        SHADOW_INGESTED_BY,
        apply,
        env_confirmed,
    )
    .await
    .map_err(map_shadow_write_error)?;

    println!("OpenStock shadow persist");
    println!("  batch_id: {}", write_report.batch_id);
    println!("  artifact_hash: {}", write_report.artifact_hash);
    println!("  dry_run: {}", write_report.dry_run);
    println!("  applied: {}", write_report.applied);
    println!("  row_count: {}", write_report.row_count);
    if write_report.dry_run && apply {
        println!("  hint: 设 {}=yes 后再跑一次以真正写入", SHADOW_ENV_CONFIRM);
    }
    Ok(())
}

pub(crate) async fn shadow_rollback(batch_id: &str) -> Result<()> {
    let client = shadow_client().await?;
    let removed = rollback_shadow_batch(&client, batch_id)
        .await
        .map_err(map_shadow_write_error)?;
    println!("OpenStock shadow rollback");
    println!("  batch_id: {}", batch_id);
    println!("  rows_removed: {}", removed);
    Ok(())
}

pub(crate) async fn shadow_verify(batch_id: &str) -> Result<()> {
    let client = shadow_client().await?;
    let count = verify_shadow_batch(&client, batch_id)
        .await
        .map_err(map_shadow_write_error)?;
    println!("OpenStock shadow verify");
    println!("  batch_id: {}", batch_id);
    println!("  rows_present: {}", count);
    Ok(())
}
