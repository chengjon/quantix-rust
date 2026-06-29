//! Shadow-namespace insert/delete/count extensions for
//! [`ClickHouseClient`](super::ClickHouseClient).
//!
//! Lives in its own `impl` block so the existing block in `kline.rs`
//! and `mod.rs` is left untouched (governance: additive-only).
//!
//! All three methods target the `quantix_shadow.openstock_daily_kline_shadow`
//! table defined in `db/schema/quantix_shadow_init.sql`. No method
//! here touches the production `kline_data` table.

use chrono::{DateTime, Utc};
use clickhouse::Row;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};
use crate::sources::openstock_shadow::ShadowKlineRow;

/// Wire shape for `quantix_shadow.openstock_daily_kline_shadow`.
///
/// Field order matches the CREATE TABLE column order in
/// `db/schema/quantix_shadow_init.sql` to keep ClickHouse JSONEachRow
/// inserts stable.
#[derive(Debug, Clone, Serialize, Deserialize, Row)]
pub struct ShadowKlineRowCH {
    pub source: String,
    pub period: String,
    pub code: String,
    pub date: chrono::NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub adjust_type: String,
    pub batch_id: String,
    pub artifact_hash: String,
    pub ingested_by: String,
    pub ingested_at: DateTime<Utc>,
}

impl From<&ShadowKlineRow> for ShadowKlineRowCH {
    fn from(row: &ShadowKlineRow) -> Self {
        Self {
            source: row.source.to_string(),
            period: row.period.clone(),
            code: row.code.clone(),
            date: row.date,
            open: row.open.to_f64().unwrap_or(0.0),
            high: row.high.to_f64().unwrap_or(0.0),
            low: row.low.to_f64().unwrap_or(0.0),
            close: row.close.to_f64().unwrap_or(0.0),
            volume: row.volume as f64,
            amount: row.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
            adjust_type: row.adjust_type.to_string(),
            batch_id: row.batch_id.clone(),
            artifact_hash: row.artifact_hash.clone(),
            ingested_by: row.ingested_by.clone(),
            ingested_at: row.ingested_at,
        }
    }
}

impl super::ClickHouseClient {
    /// Insert a batch of shadow rows into
    /// `quantix_shadow.openstock_daily_kline_shadow`. Empty input is
    /// a no-op (the dry-run gate upstream already refuses empties,
    /// but we keep the guard here for direct callers).
    pub async fn insert_shadow_klines(&self, rows: &[ShadowKlineRow]) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let mut insert = self
            .client
            .insert("quantix_shadow.openstock_daily_kline_shadow")
            .map_err(|e| QuantixError::DatabaseQuery(format!("shadow insert 创建失败: {}", e)))?;
        for row in rows {
            let wire = ShadowKlineRowCH::from(row);
            insert
                .write(&wire)
                .await
                .map_err(|e| QuantixError::DatabaseQuery(format!("shadow 写入失败: {}", e)))?;
        }
        insert
            .end()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("shadow 提交失败: {}", e)))?;
        Ok(())
    }

    /// Delete every shadow row attributable to `batch_id`. Idempotent
    /// — a second call reports the same outcome (zero rows). Returns
    /// the number of rows ClickHouse reports as affected.
    pub async fn delete_shadow_batch(&self, batch_id: &str) -> Result<usize> {
        let sql = format!(
            "ALTER TABLE quantix_shadow.openstock_daily_kline_shadow \
             DELETE WHERE batch_id = '{batch_id}'"
        );
        #[derive(Debug, Deserialize, Row)]
        struct Affected {
            rows: usize,
        }
        let affected: Vec<Affected> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("shadow 删除失败: {}", e)))?;
        Ok(affected.first().map(|a| a.rows).unwrap_or(0))
    }

    /// Count shadow rows attributable to `batch_id`. Used by the
    /// `shadow-verify` CLI subcommand and by integration tests.
    pub async fn count_shadow_batch(&self, batch_id: &str) -> Result<u64> {
        let sql = format!(
            "SELECT count() as cnt FROM quantix_shadow.openstock_daily_kline_shadow \
             WHERE batch_id = '{batch_id}'"
        );
        #[derive(Debug, Deserialize, Row)]
        struct Count {
            cnt: u64,
        }
        let rows: Vec<Count> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("shadow 计数失败: {}", e)))?;
        Ok(rows.first().map(|r| r.cnt).unwrap_or(0))
    }
}
