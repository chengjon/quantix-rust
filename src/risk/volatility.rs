use async_trait::async_trait;
use rust_decimal::Decimal;
use std::path::{Path, PathBuf};

use crate::analysis::indicators::atr;
use crate::core::{QuantixError, Result};
use crate::data::models::{AdjustType, Kline};
use crate::risk::models::{ProjectedBuyImpact, RiskRule, RiskRuleType, RuleValue};
use crate::sources::TdxDayFile;

pub const VOLATILITY_ATR_PERIOD: usize = 14;
pub const VOLATILITY_REQUIRED_BARS: usize = VOLATILITY_ATR_PERIOD + 1;
pub const RISK_TDX_ROOT_ENV: &str = "QUANTIX_TDX_ROOT";
pub const LEGACY_TDX_ROOT_ENV: &str = "TDX_ROOT";
pub const RISK_TDX_MARKET_ENV: &str = "QUANTIX_TDX_MARKET";
pub const LEGACY_TDX_MARKET_ENV: &str = "TDX_MARKET";

#[async_trait]
pub trait RiskBarLoader: Send + Sync + std::fmt::Debug {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>>;
}

#[async_trait]
impl<T> RiskBarLoader for &T
where
    T: RiskBarLoader + Send + Sync + std::fmt::Debug,
{
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        (*self).load_daily_bars(code, limit).await
    }
}

#[derive(Debug, Clone)]
pub struct DefaultRiskBarLoader {
    tdx_root: Option<PathBuf>,
    preferred_market: Option<String>,
}

impl DefaultRiskBarLoader {
    pub fn from_env() -> Self {
        let tdx_root = std::env::var_os(RISK_TDX_ROOT_ENV)
            .or_else(|| std::env::var_os(LEGACY_TDX_ROOT_ENV))
            .map(PathBuf::from);
        let preferred_market = std::env::var(RISK_TDX_MARKET_ENV)
            .ok()
            .or_else(|| std::env::var(LEGACY_TDX_MARKET_ENV).ok())
            .map(|market| market.to_ascii_lowercase());

        Self {
            tdx_root,
            preferred_market,
        }
    }

    fn load_from_tdx(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        let Some(root) = &self.tdx_root else {
            return Ok(Vec::new());
        };

        let code_num = parse_tdx_code(code)?;
        let path = resolve_tdx_day_file_path(root, code, self.preferred_market.as_deref())?;
        let mut rows = TdxDayFile::to_klines(code_num, path, AdjustType::None)?;
        if rows.len() > limit {
            rows = rows[rows.len() - limit..].to_vec();
        }
        Ok(rows)
    }
}

impl Default for DefaultRiskBarLoader {
    fn default() -> Self {
        Self::from_env()
    }
}

#[async_trait]
impl RiskBarLoader for DefaultRiskBarLoader {
    async fn load_daily_bars(&self, code: &str, limit: usize) -> Result<Vec<Kline>> {
        self.load_from_tdx(code, limit)
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
            code, RISK_TDX_ROOT_ENV, LEGACY_TDX_ROOT_ENV
        ))),
        many => Err(QuantixError::Other(format!(
            "代码 {} 在多个市场目录匹配到多个 day 文件: {}，请设置 {}",
            code,
            many.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            RISK_TDX_MARKET_ENV
        ))),
    }
}

pub async fn evaluate_volatility_limit<L>(
    rule: &RiskRule,
    projected_buy: &ProjectedBuyImpact,
    loader: &L,
) -> Result<()>
where
    L: RiskBarLoader + ?Sized,
{
    let RuleValue::Percentage(limit_pct) = rule.value.clone() else {
        return Err(QuantixError::Other(
            "risk rule volatility-limit 配置无效".to_string(),
        ));
    };

    if rule.rule_type != RiskRuleType::VolatilityLimit {
        return Err(QuantixError::Other(
            "risk rule volatility-limit 类型不匹配".to_string(),
        ));
    }

    let bars = loader
        .load_daily_bars(&projected_buy.code, VOLATILITY_REQUIRED_BARS)
        .await
        .map_err(|err| {
            QuantixError::Other(format!(
                "risk rule volatility-limit 检查失败: code={} 原因={}",
                projected_buy.code, err
            ))
        })?;

    if bars.len() < VOLATILITY_REQUIRED_BARS {
        return Err(QuantixError::Other(format!(
            "risk rule volatility-limit 检查失败: code={} 原因=可用日线不足，至少需要 {} 条",
            projected_buy.code, VOLATILITY_REQUIRED_BARS
        )));
    }

    let latest_close = bars.last().map(|bar| bar.close).ok_or_else(|| {
        QuantixError::Other(format!(
            "risk rule volatility-limit 检查失败: code={} 原因=未找到最新收盘价",
            projected_buy.code
        ))
    })?;
    if latest_close <= Decimal::ZERO {
        return Err(QuantixError::Other(format!(
            "risk rule volatility-limit 检查失败: code={} 原因=最新收盘价必须大于 0",
            projected_buy.code
        )));
    }

    let highs: Vec<Decimal> = bars.iter().map(|bar| bar.high).collect();
    let lows: Vec<Decimal> = bars.iter().map(|bar| bar.low).collect();
    let closes: Vec<Decimal> = bars.iter().map(|bar| bar.close).collect();
    let atr_value = atr(&highs, &lows, &closes, VOLATILITY_ATR_PERIOD)
        .last()
        .copied()
        .flatten()
        .ok_or_else(|| {
            QuantixError::Other(format!(
                "risk rule volatility-limit 检查失败: code={} 原因=ATR 计算失败",
                projected_buy.code
            ))
        })?;

    let actual_pct = (atr_value / latest_close * Decimal::from(100)).round_dp(2);
    if actual_pct > limit_pct {
        return Err(QuantixError::Other(format!(
            "risk rule volatility-limit 已超限: code={} threshold={} actual={}% period={}",
            projected_buy.code,
            rule.value.display(),
            actual_pct,
            VOLATILITY_ATR_PERIOD
        )));
    }

    Ok(())
}
