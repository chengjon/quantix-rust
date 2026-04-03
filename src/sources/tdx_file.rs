/// 通达信文件解析器
///
/// 从 rustdx 项目迁移 - 支持 day 文件和 gbbq 文件解析
/// 用于本地通达信数据文件的读取和复权处理
use crate::core::Result;
use crate::data::models::{AdjustType, Kline};
use chrono::NaiveDate;
use rust_decimal::Decimal;
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
        let month = ((x % 10000) / 100) as u32;
        let day = (x % 100) as u32;
        NaiveDate::from_ymd_opt(year, month, day)
    }
}

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
            open: super::tdx_file_daydata_support::rounded_decimal_or_zero(self.open),
            high: super::tdx_file_daydata_support::rounded_decimal_or_zero(self.high),
            low: super::tdx_file_daydata_support::rounded_decimal_or_zero(self.low),
            close: super::tdx_file_daydata_support::rounded_decimal_or_zero(self.close),
            volume: self.volume as i64,
            amount: super::tdx_file_daydata_support::rounded_optional_decimal(self.amount),
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
    pub fn compute_pre_pct(&self, close: f32, preclose: f64, flag: bool) -> [f64; 3] {
        super::tdx_file_gbbq_support::compute_pre_pct(self, close, preclose, flag)
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
        super::tdx_file_gbbq_support::filter_a_stock_dividend(records)
    }

    /// 按股票代码分组
    pub fn group_by_code(
        records: Vec<TdxGbbqRecord>,
    ) -> std::collections::HashMap<String, Vec<TdxGbbqRecord>> {
        super::tdx_file_gbbq_support::group_by_code(records)
    }
}

// ============================================================================
// 复权计算
// ============================================================================

/// 复权因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuquanFactor {
    /// 日期
    pub date: NaiveDate,
    /// 复权因子
    pub factor: f64,
    /// 前收盘价
    pub preclose: f64,
    /// 收盘价
    pub close: f64,
    /// 是否为交易日
    pub trading: bool,
    /// 是否为除权日
    pub xdxr: bool,
}

/// 复权类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuquanType {
    /// 不复权
    None,
    /// 前复权
    QFQ,
    /// 后复权
    HFQ,
}

/// 复权计算器
pub struct FuquanCalculator;

impl FuquanCalculator {
    /// 跳过当前交易日前的除权记录，返回当前可用的除权记录。
    fn next_applicable_xdxr<'a, I>(
        gbbq_iter: &mut std::iter::Peekable<I>,
        day_date: u32,
    ) -> Option<&'a TdxGbbqRecord>
    where
        I: Iterator<Item = &'a TdxGbbqRecord>,
    {
        while let Some(xdxr_record) = gbbq_iter.peek().copied() {
            if day_date > xdxr_record.date {
                gbbq_iter.next();
                continue;
            }

            return Some(xdxr_record);
        }

        None
    }

    fn consume_same_day_xdxr<'a, I>(
        gbbq_iter: &mut std::iter::Peekable<I>,
        day: &TdxDayRecord,
        preclose: f64,
    ) -> (f64, bool)
    where
        I: Iterator<Item = &'a TdxGbbqRecord>,
    {
        let has_current_xdxr = matches!(
            Self::next_applicable_xdxr(gbbq_iter, day.date),
            Some(xdxr_record) if day.date == xdxr_record.date
        );

        if !has_current_xdxr {
            return (preclose, false);
        }

        let xdxr_record = gbbq_iter
            .next()
            .expect("peeked ex-right record should still be available");
        let [new_preclose, _, _] = xdxr_record.compute_pre_pct(day.close, preclose, true);
        (new_preclose, true)
    }

    /// 计算复权因子（使用涨跌幅算法）
    ///
    /// 算法说明:
    /// - 基于每日涨跌幅连续计算复权因子
    /// - factor = factor * (close / preclose)
    /// - 除权日需要调整前收盘价
    pub fn calculate(
        days: &[TdxDayRecord],
        gbbqs: Option<&[TdxGbbqRecord]>,
    ) -> Result<Vec<FuquanFactor>> {
        if days.is_empty() {
            return Ok(Vec::new());
        }

        let mut factors = Vec::with_capacity(days.len());
        let mut preclose = days[0].close as f64;
        let mut factor = 1.0;

        let mut gbbq_iter = gbbqs.map(|g| g.iter()).unwrap_or([].iter()).peekable();

        for day in days {
            let close = day.close as f64;
            let (current_preclose, xdxr) =
                Self::consume_same_day_xdxr(&mut gbbq_iter, day, preclose);
            preclose = current_preclose;

            // 计算复权因子
            factor *= close / preclose;
            preclose = close;

            factors.push(FuquanFactor {
                date: day
                    .naive_date()
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                factor,
                preclose: close,
                close,
                trading: true,
                xdxr,
            });
        }

        Ok(factors)
    }

    /// 获取最新的复权因子状态（用于增量更新）
    pub fn get_latest_factor(factors: &[FuquanFactor]) -> Option<(f64, f64)> {
        super::tdx_file_fuquan_support::get_latest_factor(factors)
    }

    /// 应用前复权
    pub fn apply_qfq(kline: &Kline, factor: f64, latest_factor: f64) -> Kline {
        super::tdx_file_fuquan_support::apply_qfq(kline, factor, latest_factor)
    }

    /// 应用后复权
    pub fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
        super::tdx_file_fuquan_support::apply_hfq(kline, factor)
    }
}

// ============================================================================
// 集成类型 - 完整的日线数据（含复权）
// ============================================================================

/// 完整的日线数据（含复权信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TdxDayData {
    /// 股票代码
    pub code: String,
    /// 日期
    pub date: NaiveDate,
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
        super::tdx_file_daydata_support::from_record(record, factor)
    }

    /// 转换为 Kline
    pub fn to_kline(&self, adjust_type: AdjustType) -> Kline {
        super::tdx_file_daydata_support::to_kline(self, adjust_type)
    }
}

// ============================================================================
// 批量导入
// ============================================================================

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
        super::tdx_file_import_support::import_stock_day(&self.data_dir, code, gbbqs)
    }

    /// 批量导入多只股票
    pub fn import_batch(
        &self,
        codes: &[String],
        gbbq_map: &HashMap<String, Vec<TdxGbbqRecord>>,
    ) -> Result<HashMap<String, Vec<TdxDayData>>> {
        super::tdx_file_import_support::import_batch(&self.data_dir, codes, gbbq_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_day_record(date: u32, close: f32) -> TdxDayRecord {
        TdxDayRecord {
            code: 600000,
            date,
            open: close,
            high: close,
            low: close,
            close,
            amount: 1000000.0,
            volume: 10000,
        }
    }

    fn build_gbbq_record(
        date: u32,
        fh_qltp: f32,
        pgj_qzgb: f32,
        sg_hltp: f32,
        pg_hzgb: f32,
    ) -> TdxGbbqRecord {
        TdxGbbqRecord {
            market: 1,
            code: "600000".to_string(),
            date,
            category: 1,
            fh_qltp,
            pgj_qzgb,
            sg_hltp,
            pg_hzgb,
        }
    }

    #[test]
    fn test_tdx_day_record_size() {
        // TdxDayRecord 应该是 32 字节 (与原始 day 文件记录大小一致)
        assert_eq!(std::mem::size_of::<TdxDayRecord>(), 32);
    }

    #[test]
    fn test_date_string_conversion() {
        assert_eq!(date_string(20210801), "2021-08-01");
        assert_eq!(date_string(19900101), "1990-01-01");
    }

    #[test]
    fn test_code_string_conversion() {
        let record = TdxDayRecord {
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            ..build_day_record(20210801, 105.0)
        };
        assert_eq!(record.code_string(), "600000");
    }

    #[test]
    fn test_fuquan_calculator_empty() {
        let result = FuquanCalculator::calculate(&[], None);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_fuquan_calculator_no_gbbq() {
        let days = vec![
            TdxDayRecord {
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 104.0,
                ..build_day_record(20210801, 104.0)
            },
            TdxDayRecord {
                open: 104.5,
                high: 108.0,
                low: 103.0,
                close: 107.0,
                ..build_day_record(20210802, 107.0)
            },
        ];

        let result = FuquanCalculator::calculate(&days, None).unwrap();
        assert_eq!(result.len(), 2);

        // 第一天因子应该约为 1.0 * (104/104) = 1.0
        assert!((result[0].factor - 1.0).abs() < 0.01);

        // 第二天因子应该约为 1.0 * (107/104) ≈ 1.029
        assert!((result[1].factor - 1.029).abs() < 0.01);
    }

    #[test]
    fn test_fuquan_calculator_skips_stale_gbbq_and_applies_same_day_record() {
        let days = vec![build_day_record(20210801, 104.0)];
        let stale_record = build_gbbq_record(20210731, 0.0, 0.0, 0.0, 0.0);
        let matching_record = build_gbbq_record(20210801, 1.0, 2.0, 1.0, 1.0);
        let [expected_preclose, _, expected_factor] =
            matching_record.compute_pre_pct(days[0].close, days[0].close as f64, true);

        let result =
            FuquanCalculator::calculate(&days, Some(&[stale_record, matching_record])).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].xdxr);
        assert!((result[0].factor - expected_factor).abs() < 1e-9);
        assert!((result[0].preclose - days[0].close as f64).abs() < 1e-9);
        assert!((expected_preclose - days[0].close as f64).abs() > 1.0);
    }

    #[test]
    fn test_consume_same_day_xdxr_skips_stale_record_and_consumes_matching_one() {
        let day = build_day_record(20210801, 104.0);
        let stale_record = build_gbbq_record(20210731, 0.0, 0.0, 0.0, 0.0);
        let matching_record = build_gbbq_record(20210801, 1.0, 2.0, 1.0, 1.0);
        let [expected_preclose, _, _] =
            matching_record.compute_pre_pct(day.close, day.close as f64, true);
        let records = [stale_record, matching_record];
        let mut iter = records.iter().peekable();

        let (preclose, xdxr) = FuquanCalculator::consume_same_day_xdxr(
            &mut iter,
            &day,
            day.close as f64,
        );

        assert!(xdxr);
        assert!((preclose - expected_preclose).abs() < 1e-9);
        assert!(iter.next().is_none());
    }
}
