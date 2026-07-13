//! 数据源类型枚举及稳定字符串标识。
//!
//! 从原生 `commands/data.rs` 拆出，承载 [`DataSourceKind`] 与
//! [`DataSourceKind::as_str`]。

use clap::ValueEnum;

/// 数据源类型枚举：Tdx 通达信直连、Akshare AkShare HTTP 接口。入库用 as_str() 字符串形式存储。
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum DataSourceKind {
    Tdx,
    Akshare,
}

impl DataSourceKind {
    /// 返回该数据源类型的稳定字符串标识（"tdx"/"akshare"），用于入库与日志。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tdx => "tdx",
            Self::Akshare => "akshare",
        }
    }
}
