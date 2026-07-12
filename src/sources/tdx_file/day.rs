use super::bytes_helper;
use crate::core::Result;
use crate::data::models::{AdjustType, Kline};
use rust_decimal::prelude::*;
use std::io::Read;

/// 通达信 day 文件中的单条日线记录（32字节）
#[derive(Debug, Clone, Copy)]
pub struct TdxDayRecord {
    /// 股票代码 (u32 格式)
    pub code: u32,
    /// 日期 (u32 格式，如 20210801)
    pub date: u32,
    /// 开盘价
    pub open: f32,
    /// 最高价
    pub high: f32,
    /// 最低价
    pub low: f32,
    /// 收盘价
    pub close: f32,
    /// 成交额（元）
    pub amount: f32,
    /// 成交量（股）
    pub volume: u32,
}

fn fallback_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap_or_default()
}

impl TdxDayRecord {
    /// 字节布局:
    /// | 位置 | 含义 | 类型 | 处理 |
    /// |------|------|------|------|
    /// | 0-3  | 年月日 | u32 | 20210801 格式 |
    /// | 4-7  | 开盘价 | u32 | /100 |
    /// | 8-11 | 最高价 | u32 | /100 |
    /// | 12-15| 最低价 | u32 | /100 |
    /// | 16-19| 收盘价 | u32 | /100 |
    /// | 20-23| 成交额 | f32 | - |
    /// | 24-27| 成交量 | u32 | - |
    /// | 28-31| 保留 | - | - |
    pub fn from_bytes(code: u32, data: &[u8]) -> Self {
        Self {
            code,
            date: bytes_helper::u32_from_le_bytes(data, 0),
            open: bytes_helper::u32_from_le_bytes(data, 4) as f32 / 100.0,
            high: bytes_helper::u32_from_le_bytes(data, 8) as f32 / 100.0,
            low: bytes_helper::u32_from_le_bytes(data, 12) as f32 / 100.0,
            close: bytes_helper::u32_from_le_bytes(data, 16) as f32 / 100.0,
            amount: bytes_helper::f32_from_le_bytes(data, 20),
            volume: bytes_helper::u32_from_le_bytes(data, 24),
        }
    }

    /// 获取日期字符串 (YYYY-MM-DD)
    pub fn date_string(&self) -> String {
        bytes_helper::date_string(self.date)
    }

    /// 获取 NaiveDate
    pub fn naive_date(&self) -> Option<chrono::NaiveDate> {
        bytes_helper::to_naive_date(self.date)
    }

    /// 获取股票代码字符串 (6位)
    pub fn code_string(&self) -> String {
        format!("{:06}", self.code)
    }

    /// 转换为 Kline
    pub fn to_kline(&self, adjust_type: AdjustType) -> Kline {
        Kline {
            code: self.code_string(),
            date: self.naive_date().unwrap_or_else(fallback_date),
            open: rust_decimal::Decimal::from_f32(self.open)
                .map(|d| d.round_dp(2))
                .unwrap_or(rust_decimal::Decimal::ZERO),
            high: rust_decimal::Decimal::from_f32(self.high)
                .map(|d| d.round_dp(2))
                .unwrap_or(rust_decimal::Decimal::ZERO),
            low: rust_decimal::Decimal::from_f32(self.low)
                .map(|d| d.round_dp(2))
                .unwrap_or(rust_decimal::Decimal::ZERO),
            close: rust_decimal::Decimal::from_f32(self.close)
                .map(|d| d.round_dp(2))
                .unwrap_or(rust_decimal::Decimal::ZERO),
            volume: self.volume as i64,
            amount: rust_decimal::Decimal::from_f32(self.amount).map(|d| d.round_dp(2)),
            adjust_type,
        }
    }
}

/// Day 文件解析器
pub struct TdxDayFile;

impl TdxDayFile {
    /// 从文件读取所有日线数据
    pub fn from_file<P: AsRef<std::path::Path>>(code: u32, path: P) -> Result<Vec<TdxDayRecord>> {
        let mut file = std::fs::File::open(path.as_ref()).map_err(|e| {
            crate::core::QuantixError::DataSource(format!(
                "无法打开文件 {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| crate::core::QuantixError::DataSource(format!("读取文件失败: {}", e)))?;

        Ok(data
            .chunks_exact(32)
            .map(|chunk| TdxDayRecord::from_bytes(code, chunk))
            .collect())
    }

    /// 从文件读取并转换为 Kline
    pub fn to_klines<P: AsRef<std::path::Path>>(
        code: u32,
        path: P,
        adjust_type: AdjustType,
    ) -> Result<Vec<Kline>> {
        let records = Self::from_file(code, path)?;
        Ok(records
            .into_iter()
            .map(|r| r.to_kline(adjust_type))
            .collect())
    }
}
