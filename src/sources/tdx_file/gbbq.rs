use super::bytes_helper;
use crate::core::Result;
use std::collections::HashMap;
use std::io::Read;

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
            market: bytes_helper::u8_from_le_bytes(chunk, 0),
            code: std::str::from_utf8(&chunk[1..7]).unwrap_or("").to_string(),
            date: bytes_helper::u32_from_le_bytes(chunk, 8),
            category: bytes_helper::u8_from_le_bytes(chunk, 12),
            fh_qltp: bytes_helper::f32_from_le_bytes(chunk, 13),
            pgj_qzgb: bytes_helper::f32_from_le_bytes(chunk, 17),
            sg_hltp: bytes_helper::f32_from_le_bytes(chunk, 21),
            pg_hzgb: bytes_helper::f32_from_le_bytes(chunk, 25),
        }
    }

    /// 获取日期字符串
    pub fn date_string(&self) -> String {
        bytes_helper::date_string(self.date)
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
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<TdxGbbqRecord>> {
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

        // 前 4 字节是记录数量
        let _count = bytes_helper::u32_from_le_bytes(&data, 0) as usize;

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
