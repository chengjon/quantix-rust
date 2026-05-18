use chrono::{DateTime, Utc};
use csv::Trim;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::core::{QuantixError, Result};
use crate::risk::{
    LiveImportCashBusinessType, LiveImportRecord, LiveImportRecordType, LiveImportTradeSide,
};

#[derive(Debug, Clone, Deserialize)]
struct RawLiveImportRow {
    record_type: String,
    account_id: String,
    external_id: String,
    code: Option<String>,
    side: Option<String>,
    price: Option<String>,
    volume: Option<i64>,
    fee_total: Option<String>,
    business_type: Option<String>,
    amount: Option<String>,
    executed_at: Option<String>,
    occurred_at: Option<String>,
}

pub fn parse_live_import_csv(input: &str) -> Result<Vec<LiveImportRecord>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(input.as_bytes());

    reader
        .deserialize::<RawLiveImportRow>()
        .map(|row| {
            row.map_err(|err| QuantixError::Other(format!("risk import csv parse failed: {err}")))
                .and_then(normalize_row)
        })
        .collect()
}

pub fn parse_live_import_json(input: &str) -> Result<Vec<LiveImportRecord>> {
    let rows: Vec<RawLiveImportRow> = serde_json::from_str(input)?;
    rows.into_iter().map(normalize_row).collect()
}

fn normalize_row(row: RawLiveImportRow) -> Result<LiveImportRecord> {
    let record_type = LiveImportRecordType::from_str(row.record_type.trim()).ok_or_else(|| {
        QuantixError::Other(format!(
            "risk import 不支持的 record_type: {}",
            row.record_type
        ))
    })?;
    let account_id = required_text("account_id", &row.account_id)?;
    let external_id = required_text("external_id", &row.external_id)?;

    match record_type {
        LiveImportRecordType::Trade => {
            let code = required_optional_text("code", row.code)?;
            let side =
                LiveImportTradeSide::from_str(required_optional_text("side", row.side)?.as_str())
                    .ok_or_else(|| {
                    QuantixError::Other("risk import trade side 仅支持 buy|sell".to_string())
                })?;
            let price = parse_positive_decimal("price", row.price)?;
            let volume = row.volume.ok_or_else(|| {
                QuantixError::Other("risk import trade volume 不能为空".to_string())
            })?;
            if volume <= 0 {
                return Err(QuantixError::Other(
                    "risk import trade volume 必须是正整数".to_string(),
                ));
            }
            let fee_total = parse_non_negative_decimal("fee_total", row.fee_total)?;
            let executed_at = parse_required_timestamp("executed_at", row.executed_at)?;

            Ok(LiveImportRecord {
                record_type,
                account_id,
                external_id,
                code: Some(code),
                side: Some(side),
                price: Some(price),
                volume: Some(volume),
                fee_total: Some(fee_total),
                business_type: None,
                amount: None,
                executed_at: Some(executed_at),
                occurred_at: None,
            })
        }
        LiveImportRecordType::Cash => {
            let business_type = LiveImportCashBusinessType::from_str(
                required_optional_text("business_type", row.business_type)?.as_str(),
            )
            .ok_or_else(|| {
                QuantixError::Other(
                    "risk import cash business_type 仅支持 deposit|withdraw".to_string(),
                )
            })?;
            let amount = parse_required_decimal("amount", row.amount)?;
            match business_type {
                LiveImportCashBusinessType::Deposit if amount <= Decimal::ZERO => {
                    return Err(QuantixError::Other(
                        "risk import deposit amount 必须大于 0".to_string(),
                    ));
                }
                LiveImportCashBusinessType::Withdraw if amount >= Decimal::ZERO => {
                    return Err(QuantixError::Other(
                        "risk import withdraw amount 必须小于 0".to_string(),
                    ));
                }
                _ => {}
            }
            let occurred_at = parse_required_timestamp("occurred_at", row.occurred_at)?;

            Ok(LiveImportRecord {
                record_type,
                account_id,
                external_id,
                code: None,
                side: None,
                price: None,
                volume: None,
                fee_total: None,
                business_type: Some(business_type),
                amount: Some(amount),
                executed_at: None,
                occurred_at: Some(occurred_at),
            })
        }
    }
}

fn required_text(field: &str, value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(QuantixError::Other(format!("risk import {field} 不能为空")));
    }
    Ok(value.to_string())
}

fn required_optional_text(field: &str, value: Option<String>) -> Result<String> {
    value
        .map(|raw| required_text(field, &raw))
        .transpose()?
        .ok_or_else(|| QuantixError::Other(format!("risk import {field} 不能为空")))
}

fn parse_required_decimal(field: &str, value: Option<String>) -> Result<Decimal> {
    let raw = required_optional_text(field, value)?;
    Decimal::from_str_exact(raw.trim())
        .map_err(|_| QuantixError::Other(format!("risk import {field} 无法解析: {raw}")))
}

fn parse_positive_decimal(field: &str, value: Option<String>) -> Result<Decimal> {
    let decimal = parse_required_decimal(field, value)?;
    if decimal <= Decimal::ZERO {
        return Err(QuantixError::Other(format!(
            "risk import {field} 必须大于 0"
        )));
    }
    Ok(decimal)
}

fn parse_non_negative_decimal(field: &str, value: Option<String>) -> Result<Decimal> {
    let decimal = parse_required_decimal(field, value)?;
    if decimal < Decimal::ZERO {
        return Err(QuantixError::Other(format!(
            "risk import {field} 不能小于 0"
        )));
    }
    Ok(decimal)
}

fn parse_required_timestamp(field: &str, value: Option<String>) -> Result<DateTime<Utc>> {
    let raw = required_optional_text(field, value)?;
    DateTime::parse_from_rfc3339(raw.trim())
        .map(|value| value.with_timezone(&Utc))
        .map_err(|_| QuantixError::Other(format!("risk import {field} 无法解析: {raw}")))
}
