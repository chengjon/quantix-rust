//! 基本面数据提供商 Trait

use async_trait::async_trait;
use crate::core::Result;
use super::types::{
    FundamentalData, ValuationMetrics, EarningsReport,
    InstitutionHolding, DragonTigerItem, DividendInfo, CapitalFlow,
};

/// 基本面数据提供商 Trait
#[async_trait]
pub trait FundamentalProvider: Send + Sync {
    /// 提供商名称
    fn name(&self) -> &'static str;

    /// 获取完整基本面数据
    async fn get_fundamental(&self, code: &str) -> Result<FundamentalData> {
        let valuation = self.get_valuation(code).await.ok();
        let earnings = self.get_latest_earnings(code).await.ok();
        let holdings = self.get_institution_holdings(code).await.unwrap_or_default();

        Ok(FundamentalData {
            code: code.to_string(),
            name: String::new(),
            date: chrono::Utc::now().date_naive(),
            valuation,
            latest_earnings: earnings,
            institution_holdings: holdings,
            source: self.name().to_string(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// 获取估值指标
    async fn get_valuation(&self, code: &str) -> Result<ValuationMetrics>;

    /// 获取最新财报
    async fn get_latest_earnings(&self, code: &str) -> Result<EarningsReport>;

    /// 获取历史财报列表
    async fn get_earnings_history(&self, code: &str, years: u32) -> Result<Vec<EarningsReport>>;

    /// 获取机构持仓
    async fn get_institution_holdings(&self, code: &str) -> Result<Vec<InstitutionHolding>>;

    /// 获取龙虎榜数据
    async fn get_dragon_tiger(&self, code: &str, days: u32) -> Result<Vec<DragonTigerItem>>;

    /// 获取分红信息
    async fn get_dividend_history(&self, code: &str, years: u32) -> Result<Vec<DividendInfo>>;

    /// 获取资金流向
    async fn get_capital_flow(&self, code: &str, days: u32) -> Result<Vec<CapitalFlow>>;

    /// 检查是否可用
    fn is_available(&self) -> bool {
        true
    }
}
