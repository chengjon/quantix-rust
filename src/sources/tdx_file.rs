/// 通达信文件解析器
///
/// 从 rustdx 项目迁移 - 支持 day 文件和 gbbq 文件解析
/// 用于本地通达信数据文件的读取和复权处理
use crate::core::Result;
use crate::data::models::{AdjustType, Kline};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// ============================================================================
// 字节解析辅助函数
// ============================================================================

mod bytes_helper {
    use chrono::NaiveDate;

    /// 将 slice 的 4 字节转为 u32 (little-endian)
    #[inline]
    pub fn u32_from_le_bytes(slice: &[u8], pos: usize) -> u32 {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&slice[pos..pos + 4]);
        u32::from_le_bytes(arr)
    }

    /// 将 slice 的 4 字节转为 f32 (little-endian)
    #[inline]
    pub fn f32_from_le_bytes(slice: &[u8], pos: usize) -> f32 {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(&slice[pos..pos + 4]);
        f32::from_le_bytes(arr)
    }

    /// 将 slice 的 1 字节转为 u8
    #[inline]
    pub fn u8_from_le_bytes(slice: &[u8], pos: usize) -> u8 {
        slice[pos]
    }

    /// 将 u32 日期转为字符串 (20210801 => "2021-08-01")
    #[inline]
    pub fn date_string(x: u32) -> String {
        let year = x / 10000;
        let month = (x % 10000) / 100;
        let day = x % 100;
        format!("{:04}-{:02}-{:02}", year, month, day)
    }

    /// 将 u32 日期转为 NaiveDate
    #[inline]
    pub fn to_naive_date(x: u32) -> Option<NaiveDate> {
        let year = (x / 10000) as i32;
        let month = (x % 10000) / 100;
        let day = x % 100;
        NaiveDate::from_ymd_opt(year, month, day)
    }
}

mod fuquan;

#[cfg(test)]
mod tests;

pub use self::fuquan::{FuquanCalculator, FuquanFactor, FuquanType, TdxDataImporter, TdxDayData};

use bytes_helper::*;

// ============================================================================
// Day 文件解析
// ============================================================================

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
            date: u32_from_le_bytes(data, 0),
            open: u32_from_le_bytes(data, 4) as f32 / 100.0,
            high: u32_from_le_bytes(data, 8) as f32 / 100.0,
            low: u32_from_le_bytes(data, 12) as f32 / 100.0,
            close: u32_from_le_bytes(data, 16) as f32 / 100.0,
            amount: f32_from_le_bytes(data, 20),
            volume: u32_from_le_bytes(data, 24),
        }
    }

    /// 获取日期字符串 (YYYY-MM-DD)
    pub fn date_string(&self) -> String {
        date_string(self.date)
    }

    /// 获取 NaiveDate
    pub fn naive_date(&self) -> Option<NaiveDate> {
        to_naive_date(self.date)
    }

    /// 获取股票代码字符串 (6位)
    pub fn code_string(&self) -> String {
        format!("{:06}", self.code)
    }

    /// 转换为 Kline
    pub fn to_kline(&self, adjust_type: AdjustType) -> Kline {
        Kline {
            code: self.code_string(),
            date: self
                .naive_date()
                .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
            open: Decimal::from_f32(self.open)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            high: Decimal::from_f32(self.high)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            low: Decimal::from_f32(self.low)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            close: Decimal::from_f32(self.close)
                .map(|d| d.round_dp(2))
                .unwrap_or(Decimal::ZERO),
            volume: self.volume as i64,
            amount: Decimal::from_f32(self.amount).map(|d| d.round_dp(2)),
            adjust_type,
        }
    }
}

/// Day 文件解析器
pub struct TdxDayFile;

impl TdxDayFile {
    /// 从文件读取所有日线数据
    pub fn from_file<P: AsRef<Path>>(code: u32, path: P) -> Result<Vec<TdxDayRecord>> {
        let mut file = File::open(path.as_ref()).map_err(|e| {
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
    pub fn to_klines<P: AsRef<Path>>(
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

// ============================================================================
// GBBQ 文件解析 (股本变迁)
// ============================================================================

/// 股本变迁记录 (除权除息)
#[derive(Debug, Clone)]
pub struct TdxGbbqRecord {
    /// 市场 (1=上海, 0=深圳)
    pub market: u8,
    /// 股票代码 (6字节)
    pub code: String,
    /// 日期 (u32)
    pub date: u32,
    /// 信息类型 (1=除权除息, 2=送配股上市, ...)
    pub category: u8,
    /// 分红（每10股派现金x元）/ 前流通盘
    pub fh_qltp: f32,
    /// 配股价 / 前总股本
    pub pgj_qzgb: f32,
    /// 送转股（每10股送转x股）/ 后流通盘
    pub sg_hltp: f32,
    /// 配股（每10股配x股）/ 后总股本
    pub pg_hzgb: f32,
}

impl TdxGbbqRecord {
    /// 从 29 字节解析
    /// | 位置 | 含义 |
    /// |------|------|
    /// | 0    | market |
    /// | 1-6  | code (6字节) |
    /// | 8-11 | date |
    /// | 12   | category |
    /// | 13-16| fh_qltp |
    /// | 17-20| pgj_qzgb |
    /// | 21-24| sg_hltp |
    /// | 25-28| pg_hzgb |
    pub fn from_chunk(chunk: &[u8]) -> Self {
        Self {
            market: u8_from_le_bytes(chunk, 0),
            code: std::str::from_utf8(&chunk[1..7]).unwrap_or("").to_string(),
            date: u32_from_le_bytes(chunk, 8),
            category: u8_from_le_bytes(chunk, 12),
            fh_qltp: f32_from_le_bytes(chunk, 13),
            pgj_qzgb: f32_from_le_bytes(chunk, 17),
            sg_hltp: f32_from_le_bytes(chunk, 21),
            pg_hzgb: f32_from_le_bytes(chunk, 25),
        }
    }

    /// 获取日期字符串
    pub fn date_string(&self) -> String {
        date_string(self.date)
    }

    /// 获取股票代码字符串
    pub fn code_string(&self) -> &str {
        &self.code
    }

    /// 计算除权后的前收盘价和涨跌幅
    /// close: 当日收盘价
    /// preclose: 原前收盘价
    /// flag: 是否为除权日
    /// 返回: [新前收盘, 收盘价, 涨跌幅]
    pub fn compute_pre_pct(&self, close: f32, mut preclose: f64, flag: bool) -> [f64; 3] {
        if flag {
            // 除权计算公式: (preclose * 10 - 分红 + 配股 * 配股价) / (10 + 配股 + 送股)
            preclose = (preclose * 10.0 - self.fh_qltp as f64
                + self.pg_hzgb as f64 * self.pgj_qzgb as f64)
                / (10.0 + self.pg_hzgb as f64 + self.sg_hltp as f64);
        }
        let close = close as f64;
        [preclose, close, close / preclose]
    }
}

/// GBBQ 文件解析器
pub struct TdxGbbqFile;

impl TdxGbbqFile {
    /// 从文件读取股本变迁数据（已解密）
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Vec<TdxGbbqRecord>> {
        let mut file = File::open(path.as_ref()).map_err(|e| {
            crate::core::QuantixError::DataSource(format!(
                "无法打开文件 {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| crate::core::QuantixError::DataSource(format!("读取文件失败: {}", e)))?;

        // 前 4 字节是记录数量
        let _count = u32_from_le_bytes(&data, 0) as usize;

        // 从第 5 字节开始，每条记录 29 字节
        Ok(data[4..]
            .chunks_exact(29)
            .map(TdxGbbqRecord::from_chunk)
            .collect())
    }

    /// 过滤出 A 股除权除息记录 (category = 1)
    pub fn filter_a_stock_dividend(records: &[TdxGbbqRecord]) -> Vec<TdxGbbqRecord> {
        records
            .iter()
            .filter(|r| {
                // A股代码: 6xxx, 0xxx, 3xxx
                let first_char = r.code.chars().next();
                let is_a_stock = matches!(first_char, Some('6') | Some('0') | Some('3'));
                is_a_stock && r.category == 1
            })
            .cloned()
            .collect()
    }

    /// 按股票代码分组
    pub fn group_by_code(records: Vec<TdxGbbqRecord>) -> HashMap<String, Vec<TdxGbbqRecord>> {
        let mut map: HashMap<String, Vec<TdxGbbqRecord>> = HashMap::new();
        for record in records {
            let code = record.code.clone();
            map.entry(code).or_default().push(record);
        }
        map
    }
}
