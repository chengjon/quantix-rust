use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::analysis::indicators::atr;
use crate::core::{QuantixError, Result};
use crate::data::models::Kline;
use crate::risk::models::{ProjectedBuyImpact, RiskRule, RiskRuleType, RuleValue};
use crate::strategy::fallback_loader::FallbackStrategyBarLoader;
use crate::strategy::runtime::StrategyBarLoader;

pub const VOLATILITY_ATR_PERIOD: usize = 14;
pub const VOLATILITY_REQUIRED_BARS: usize = VOLATILITY_ATR_PERIOD + 1;

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

#[derive(Debug, Clone, Copy, Default)]
struct EmptyPrimaryBarLoader;

#[async_trait]
impl StrategyBarLoader for EmptyPrimaryBarLoader {
    async fn load_daily_bars(&self, _code: &str, _limit: usize) -> Result<Vec<Kline>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct DefaultRiskBarLoader {
    inner: FallbackStrategyBarLoader<EmptyPrimaryBarLoader>,
}

impl DefaultRiskBarLoader {
    pub fn from_env() -> Self {
        Self {
            inner: FallbackStrategyBarLoader::from_env_with_primary_source_id(
                EmptyPrimaryBarLoader,
                "risk-primary",
            ),
        }
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
        self.inner.load_daily_bars(code, limit).await
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
