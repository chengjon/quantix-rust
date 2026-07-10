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

fn fallback_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(1970, 1, 1).unwrap_or_default()
}

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
            date: self.naive_date().unwrap_or_else(fallback_date),
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

        let mut current_xdxr = gbbq_iter.peek().copied();

        for day in days.iter() {
            let close = day.close as f64;

            // 检查是否有除权事件
            let mut xdxr = false;
            if let Some(xdxr_record) = current_xdxr {
                if day.date == xdxr_record.date {
                    // 除权日
                    let [new_preclose, _, _] =
                        xdxr_record.compute_pre_pct(day.close, preclose, true);
                    preclose = new_preclose;
                    xdxr = true;

                    // 移动到下一个除权记录
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                } else if day.date > xdxr_record.date {
                    // 跳过已经过的除权日（非交易日）
                    gbbq_iter.next();
                    current_xdxr = gbbq_iter.peek().copied();
                }
            }

            // 计算复权因子
            factor *= close / preclose;
            preclose = close;

            factors.push(FuquanFactor {
                date: day.naive_date().unwrap_or_else(fallback_date),
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
        factors.last().map(|f| (f.close, f.factor))
    }

    /// 应用前复权
    pub fn apply_qfq(kline: &Kline, factor: f64, latest_factor: f64) -> Kline {
        let adj_factor = latest_factor / factor;
        let adj_dec = Decimal::from_f64(adj_factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::QFQ,
            ..kline.clone()
        }
    }

    /// 应用后复权
    pub fn apply_hfq(kline: &Kline, factor: f64) -> Kline {
        let adj_dec = Decimal::from_f64(factor).unwrap_or(Decimal::ONE);
        Kline {
            open: (kline.open * adj_dec).round_dp(2),
            high: (kline.high * adj_dec).round_dp(2),
            low: (kline.low * adj_dec).round_dp(2),
            close: (kline.close * adj_dec).round_dp(2),
            adjust_type: AdjustType::HFQ,
            ..kline.clone()
        }
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
    pub fn to_kline(&self, adjust_type: AdjustType) -> Kline {
        Kline {
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

#[cfg(test)]
mod tests {
    use super::*;

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
            code: 600000,
            date: 20210801,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 105.0,
            amount: 1000000.0,
            volume: 10000,
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
                code: 600000,
                date: 20210801,
                open: 100.0,
                high: 105.0,
                low: 99.0,
                close: 104.0,
                amount: 1000000.0,
                volume: 10000,
            },
            TdxDayRecord {
                code: 600000,
                date: 20210802,
                open: 104.5,
                high: 108.0,
                low: 103.0,
                close: 107.0,
                amount: 1000000.0,
                volume: 10000,
            },
        ];

        let result = FuquanCalculator::calculate(&days, None).unwrap();
        assert_eq!(result.len(), 2);

        // 第一天因子应该约为 1.0 * (104/104) = 1.0
        assert!((result[0].factor - 1.0).abs() < 0.01);

        // 第二天因子应该约为 1.0 * (107/104) ≈ 1.029
        assert!((result[1].factor - 1.029).abs() < 0.01);
    }
}
