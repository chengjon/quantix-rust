use crate::core::Result;
use crate::data::models::AdjustType;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::day::{TdxDayFile, TdxDayRecord};
use super::fuquan::{FuquanCalculator, FuquanFactor};
use super::gbbq::TdxGbbqRecord;

fn fallback_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap_or_default()
}

/// 完整的日线数据（含复权信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdxDayData {
    /// 股票代码
    pub code: String,
    /// 日期
    pub date: chrono::NaiveDate,
    /// 开盘价
    pub open: Decimal,
    /// 最高价
    pub high: Decimal,
    /// 最低价
    pub low: Decimal,
    /// 收盘价
    pub close: Decimal,
    /// 成交量
    pub volume: i64,
    /// 成交额
    pub amount: Decimal,
    /// 前收盘价
    pub preclose: Decimal,
    /// 复权因子
    pub factor: Decimal,
    /// 涨跌幅
    pub change_pct: Decimal,
}

impl TdxDayData {
    /// 从 day 记录和复权因子创建
    pub fn from_record(record: &TdxDayRecord, factor: &FuquanFactor) -> Self {
        let change_pct = if factor.preclose > 0.0 {
            Decimal::from_f64((factor.close - factor.preclose) / factor.preclose * 100.0)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO)
        } else {
            Decimal::ZERO
        };

        Self {
            code: record.code_string(),
            date: record.naive_date().unwrap_or_else(fallback_date),
            open: Decimal::from_f32(record.open)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            high: Decimal::from_f32(record.high)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            low: Decimal::from_f32(record.low)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            close: Decimal::from_f32(record.close)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            volume: record.volume as i64,
            amount: Decimal::from_f32(record.amount)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            preclose: Decimal::from_f64(factor.preclose)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            factor: Decimal::from_f64(factor.factor)
                .map(|d| d.round_dp(6))
                .unwrap_or(Decimal::ONE),
            change_pct,
        }
    }

    /// 转换为 Kline
    pub fn to_kline(&self, adjust_type: AdjustType) -> crate::data::models::Kline {
        crate::data::models::Kline {
            code: self.code.clone(),
            date: self.date,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.volume,
            amount: Some(self.amount),
            adjust_type,
        }
    }
}

/// TDX 数据批量导入器
pub struct TdxDataImporter {
    /// 数据目录
    data_dir: String,
}

impl TdxDataImporter {
    /// 创建导入器
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// 导入单个股票的日线数据
    pub fn import_stock_day(
        &self,
        code: &str,
        gbbqs: Option<&[TdxGbbqRecord]>,
    ) -> Result<Vec<TdxDayData>> {
        let code_num = code.parse::<u32>().map_err(|_| {
            crate::core::QuantixError::DataParse(format!("无效的股票代码: {}", code))
        })?;

        let day_path = format!("{}/{}.day", self.data_dir, code);
        let records = TdxDayFile::from_file(code_num, &day_path)?;

        let factors = FuquanCalculator::calculate(&records, gbbqs)?;

        Ok(records
            .iter()
            .zip(factors.iter())
            .map(|(r, f)| TdxDayData::from_record(r, f))
            .collect())
    }

    /// 批量导入多只股票
    pub fn import_batch(
        &self,
        codes: &[String],
        gbbq_map: &HashMap<String, Vec<TdxGbbqRecord>>,
    ) -> Result<HashMap<String, Vec<TdxDayData>>> {
        let mut result = HashMap::new();

        for code in codes {
            let gbbqs = gbbq_map.get(code).map(|v| v.as_slice());
            match self.import_stock_day(code, gbbqs) {
                Ok(data) => {
                    if !data.is_empty() {
                        result.insert(code.clone(), data);
                    }
                }
                Err(e) => {
                    tracing::warn!("导入 {} 失败: {}", code, e);
                }
            }
        }

        Ok(result)
    }
}
