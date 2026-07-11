use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};
use crate::sources::TdxDayFile;
use crate::strategy::runtime::StrategyBarLoader;

/// TDX 根目录环境变量（新规范）：FallbackStrategyBarLoader 从此变量解析 TDX 数据根路径。
pub const STRATEGY_TDX_ROOT_ENV: &str = "QUANTIX_TDX_ROOT";
/// TDX 根目录环境变量（旧版兼容）：QUANTIX_TDX_ROOT 未设置时回退到此变量。
pub const LEGACY_TDX_ROOT_ENV: &str = "TDX_ROOT";
/// TDX 市场标识环境变量（新规范）：如 sh/sz，用于决定 day 文件子目录。
pub const STRATEGY_TDX_MARKET_ENV: &str = "QUANTIX_TDX_MARKET";
/// TDX 市场标识环境变量（旧版兼容）。
pub const LEGACY_TDX_MARKET_ENV: &str = "TDX_MARKET";

/// 单次 load_daily_bars 的数据来源标记：source_id 标识来源（"primary" / "tdx_fallback"）、fallback_used 是否走了 fallback 路径。供 daemon 遥测诊断使用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategyBarLoadSource {
    pub source_id: String,
    pub fallback_used: bool,
}

/// 双源 K 线 loader：primary 优先（如 ClickHouse），失败时按 tdx_root + preferred_market 走 TDX day 文件 fallback。last_source 记录最近一次来源供诊断。
#[derive(Debug, Clone)]
pub struct FallbackStrategyBarLoader<P> {
    primary: P,
    primary_source_id: &'static str,
    tdx_root: Option<PathBuf>,
    preferred_market: Option<String>,
    last_source: Arc<Mutex<Option<StrategyBarLoadSource>>>,
}

impl<P> FallbackStrategyBarLoader<P> {
    /// 以 primary 为首选、`tdx_root` 为 fallback 根目录构造 loader；primary_source_id 默认为 "primary"。
    pub fn new(primary: P, tdx_root: Option<PathBuf>) -> Self {
        Self::with_options(primary, "primary", tdx_root, None)
    }

    /// 同 new，但允许显式指定 primary_source_id（用于上报/审计中区分上游来源）。
    pub fn with_primary_source_id(
        primary: P,
        primary_source_id: &'static str,
        tdx_root: Option<PathBuf>,
    ) -> Self {
        Self::with_options(primary, primary_source_id, tdx_root, None)
    }

    /// 全参数构造：primary、primary_source_id、tdx_root、preferred_market（市场小写化）。
    pub fn with_options(
        primary: P,
        primary_source_id: &'static str,
        tdx_root: Option<PathBuf>,
        preferred_market: Option<String>,
    ) -> Self {
        Self {
            primary,
            primary_source_id,
            tdx_root,
            preferred_market: preferred_market.map(|value| value.to_ascii_lowercase()),
            last_source: Arc::new(Mutex::new(None)),
        }
    }

    /// 从 QUANTIX_TDX_ROOT/TDX_ROOT 与 QUANTIX_TDX_MARKET/TDX_MARKET 环境变量构造 loader。
    pub fn from_env(primary: P) -> Self {
        Self::from_env_with_primary_source_id(primary, "primary")
    }

    /// 同 from_env，但允许显式指定 primary_source_id。
    pub fn from_env_with_primary_source_id(primary: P, primary_source_id: &'static str) -> Self {
        let tdx_root = std::env::var_os(STRATEGY_TDX_ROOT_ENV)
            .or_else(|| std::env::var_os(LEGACY_TDX_ROOT_ENV))
            .map(PathBuf::from);
        let preferred_market = std::env::var(STRATEGY_TDX_MARKET_ENV)
            .ok()
            .or_else(|| std::env::var(LEGACY_TDX_MARKET_ENV).ok());
        Self::with_options(primary, primary_source_id, tdx_root, preferred_market)
    }

    /// 返回最近一次 load_daily_bars 实际命中的来源（primary 或 tdx-day-file）；尚未加载时为 `None`。
    pub fn last_source(&self) -> Option<StrategyBarLoadSource> {
        self.last_source_guard().clone()
    }

    fn last_source_guard(&self) -> MutexGuard<'_, Option<StrategyBarLoadSource>> {
        match self.last_source.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("strategy fallback loader source state mutex was poisoned");
                poisoned.into_inner()
            }
        }
    }

    fn set_last_source(&self, source: StrategyBarLoadSource) {
        *self.last_source_guard() = Some(source);
    }

    fn load_from_tdx(&self, code: &str, limit: usize) -> Result<Option<Vec<Kline>>> {
        let Some(root) = &self.tdx_root else {
            return Ok(None);
        };

        let code_num = parse_tdx_code(code)?;
        let path = resolve_tdx_day_file_path(root, code, self.preferred_market.as_deref())?;
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
            Ok(rows) if !rows.is_empty() => {
                self.set_last_source(StrategyBarLoadSource {
                    source_id: self.primary_source_id.to_string(),
                    fallback_used: false,
                });
                Ok(rows)
            }
            Ok(_) => match self.load_from_tdx(code, limit)? {
                Some(rows) => {
                    self.set_last_source(StrategyBarLoadSource {
                        source_id: "tdx-day-file".to_string(),
                        fallback_used: true,
                    });
                    Ok(rows)
                }
                None => Ok(Vec::new()),
            },
            Err(primary_error) => match self.load_from_tdx(code, limit) {
                Ok(Some(rows)) => {
                    self.set_last_source(StrategyBarLoadSource {
                        source_id: "tdx-day-file".to_string(),
                        fallback_used: true,
                    });
                    Ok(rows)
                }
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

fn resolve_tdx_day_file_path(
    root: impl AsRef<Path>,
    code: &str,
    preferred_market: Option<&str>,
) -> Result<PathBuf> {
    let root = root.as_ref();

    if let Some(market) = preferred_market {
        let market = market.to_ascii_lowercase();
        let path = root
            .join("vipdoc")
            .join(&market)
            .join("lday")
            .join(format!("{}{}.day", market, code));
        if path.exists() {
            return Ok(path);
        }
        return Err(QuantixError::Other(format!(
            "未找到指定市场的 day 文件: {}",
            path.display()
        )));
    }

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
            "代码 {} 在多个市场目录匹配到多个 day 文件: {}，请设置 {}",
            code,
            many.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            STRATEGY_TDX_MARKET_ENV
        ))),
    }
}
