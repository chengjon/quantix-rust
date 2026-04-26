use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

use crate::core::{QuantixError, Result};
use crate::risk::LiveImportMirrorPosition;

pub(super) fn row_to_mirror_position(row: SqliteRow) -> Result<LiveImportMirrorPosition> {
    Ok(LiveImportMirrorPosition {
        code: row.try_get("code")?,
        volume: row.try_get("volume")?,
        avg_cost: parse_decimal(&row.try_get::<String, _>("avg_cost")?, "avg_cost")?,
        last_trade_at: parse_timestamp(&row.try_get::<String, _>("last_trade_at")?)?,
    })
}

pub(super) fn parse_timestamp(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .map_err(|err| QuantixError::DataParse(format!("invalid stored timestamp: {err}")))?
        .with_timezone(&Utc))
}

pub(super) fn parse_decimal(value: &str, field: &str) -> Result<rust_decimal::Decimal> {
    rust_decimal::Decimal::from_str_exact(value).map_err(|_| {
        QuantixError::DataParse(format!("invalid stored decimal for {field}: {value}"))
    })
}
