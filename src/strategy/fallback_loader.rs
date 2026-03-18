use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};
use crate::sources::TdxDayFile;
use crate::strategy::runtime::StrategyBarLoader;

pub const STRATEGY_TDX_ROOT_ENV: &str = "QUANTIX_TDX_ROOT";
pub const LEGACY_TDX_ROOT_ENV: &str = "TDX_ROOT";

#[derive(Debug, Clone)]
pub struct FallbackStrategyBarLoader<P> {
    primary: P,
    tdx_root: Option<PathBuf>,
}

impl<P> FallbackStrategyBarLoader<P> {
    pub fn new(primary: P, tdx_root: Option<PathBuf>) -> Self {
        Self { primary, tdx_root }
    }

    pub fn from_env(primary: P) -> Self {
        let tdx_root = std::env::var_os(STRATEGY_TDX_ROOT_ENV)
            .or_else(|| std::env::var_os(LEGACY_TDX_ROOT_ENV))
            .map(PathBuf::from);
        Self::new(primary, tdx_root)
    }

    fn load_from_tdx(&self, code: &str, limit: usize) -> Result<Option<Vec<Kline>>> {
        let Some(root) = &self.tdx_root else {
            return Ok(None);
        };

        let code_num = parse_tdx_code(code)?;
        let path = resolve_tdx_day_file_path(root, code)?;
        let mut rows = TdxDayFile::to_klines(code_num, path, AdjustType::None)?;
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(Some(rows))
    }
}

#[async_trait]
impl<P> StrategyBarLoader for FallbackStrategyBarLoader<P>
where
    P: StrategyBarLoader,
{
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        match self.primary.load_daily_bars(code, limit).await {
            Ok(rows) if !rows.is_empty() => Ok(rows),
            Ok(_) => match self.load_from_tdx(code, limit)? {
                Some(rows) => Ok(rows),
                None => Ok(Vec::new()),
            },
            Err(primary_error) => match self.load_from_tdx(code, limit) {
                Ok(Some(rows)) => Ok(rows),
                Ok(None) => Err(primary_error),
                Err(fallback_error) => Err(QuantixError::Other(format!(
                    "主读取器失败: {}; TDX fallback 失败: {}",
                    primary_error, fallback_error
                ))),
            },
        }
    }
}

fn parse_tdx_code(code: &str) -> Result<u32> {
    let digits: String = code.chars().filter(|ch| ch.is_ascii_digit()).collect();
    if digits.is_empty() {
        return Err(QuantixError::Other(format!(
            "股票代码中未找到有效数字: {code}"
        )));
    }

    digits
        .parse::<u32>()
        .map_err(|e| QuantixError::Other(format!("股票代码解析失败: {}", e)))
}

fn resolve_tdx_day_file_path(root: impl AsRef<Path>, code: &str) -> Result<PathBuf> {
    let root = root.as_ref();
    let matches: Vec<PathBuf> = ["sh", "sz", "bj", "ds"]
        .iter()
        .map(|market| {
            root.join("vipdoc")
                .join(market)
                .join("lday")
                .join(format!("{}{}.day", market, code))
        })
        .filter(|path| path.exists())
        .collect();

    match matches.as_slice() {
        [single] => Ok(single.clone()),
        [] => Err(QuantixError::Other(format!(
            "未找到 {} 对应的 day 文件，请确认 {} 或 {}",
            code, STRATEGY_TDX_ROOT_ENV, LEGACY_TDX_ROOT_ENV
        ))),
        many => Err(QuantixError::Other(format!(
            "代码 {} 在多个市场目录匹配到多个 day 文件: {}",
            code,
            many.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}
