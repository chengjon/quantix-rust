/// 通达信文件解析器
///
/// 从 rustdx 项目迁移 - 支持 day 文件和 gbbq 文件解析
/// 用于本地通达信数据文件的读取和复权处理
///
/// Historical note: this file was once 683 lines and was split into
/// thematic submodules per the file-size limits in CLAUDE.md.
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

mod day;
mod fuquan;
mod gbbq;
mod importer;

pub use day::{TdxDayFile, TdxDayRecord};
pub use fuquan::{FuquanCalculator, FuquanFactor, FuquanType};
pub use gbbq::{TdxGbbqFile, TdxGbbqRecord};
pub use importer::{TdxDataImporter, TdxDayData};

#[cfg(test)]
mod tests {
    use super::*;
    use bytes_helper::{date_string, u32_from_le_bytes};

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
            amount: 1_000_000.0,
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
                amount: 1_000_000.0,
                volume: 10000,
            },
            TdxDayRecord {
                code: 600000,
                date: 20210802,
                open: 104.5,
                high: 108.0,
                low: 103.0,
                close: 107.0,
                amount: 1_000_000.0,
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

// Re-export bytes_helper for submodules that need it (day.rs, gbbq.rs).
// The private `mod bytes_helper` above is not directly visible to child
// modules defined via `mod day;`, so each submodule imports it via
// `super::bytes_helper` and uses `bytes_helper::function_name(...)`.
#[allow(unused_imports)]
use bytes_helper::*;
