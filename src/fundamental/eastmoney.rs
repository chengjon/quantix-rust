//! 东方财富基本面数据提供商
//!
//! 实现 FundamentalProvider trait，委托给各个专门的获取器

use super::dragon_tiger::DragonTigerFetcher;
use super::earnings::EarningsFetcher;
use super::institution::InstitutionFetcher;
use super::provider::FundamentalProvider;
use super::types::{
    CapitalFlow, DividendInfo, DragonTigerItem, EarningsReport, InstitutionHolding,
    ValuationMetrics,
};
use super::valuation::ValuationFetcher;
use crate::core::{QuantixError, Result};
use async_trait::async_trait;

/// 东方财富基本面数据提供商
pub struct EastMoneyFundamentalProvider {
    valuation: ValuationFetcher,
    earnings: EarningsFetcher,
    institution: InstitutionFetcher,
    dragon_tiger: DragonTigerFetcher,
}

impl EastMoneyFundamentalProvider {
    /// 创建新的东方财富数据提供商实例
    pub fn new() -> Self {
        Self {
            valuation: ValuationFetcher::new(),
            earnings: EarningsFetcher::new(),
            institution: InstitutionFetcher::new(),
            dragon_tiger: DragonTigerFetcher::new(),
        }
    }
}

impl Default for EastMoneyFundamentalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FundamentalProvider for EastMoneyFundamentalProvider {
    fn name(&self) -> &'static str {
        "EastMoney"
    }

    async fn get_valuation(&self, code: &str) -> Result<ValuationMetrics> {
        self.valuation.fetch_from_eastmoney(code).await
    }

    async fn get_latest_earnings(&self, code: &str) -> Result<EarningsReport> {
        self.earnings.fetch_latest(code).await
    }

    async fn get_earnings_history(&self, code: &str, years: u32) -> Result<Vec<EarningsReport>> {
        self.earnings.fetch_history(code, years).await
    }

    async fn get_institution_holdings(&self, code: &str) -> Result<Vec<InstitutionHolding>> {
        self.institution.fetch_holdings(code).await
    }

    async fn get_dragon_tiger(&self, code: &str, days: u32) -> Result<Vec<DragonTigerItem>> {
        self.dragon_tiger.fetch(code, days).await
    }

    async fn get_dividend_history(&self, _code: &str, _years: u32) -> Result<Vec<DividendInfo>> {
        // 分红数据获取器尚未实现
        Err(QuantixError::Unsupported(
            "分红数据获取功能尚未实现".to_string(),
        ))
    }

    async fn get_capital_flow(&self, _code: &str, _days: u32) -> Result<Vec<CapitalFlow>> {
        // 资金流向获取器尚未实现
        Err(QuantixError::Unsupported(
            "资金流向获取功能尚未实现".to_string(),
        ))
    }
}
