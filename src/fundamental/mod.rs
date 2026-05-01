//! 基本面数据模块
//!
//! 提供股票基本面数据获取能力：估值、财报、龙虎榜等

pub mod dragon_tiger;
pub mod earnings;
pub mod eastmoney;
pub mod institution;
pub mod provider;
pub mod types;
pub mod valuation;

pub use eastmoney::EastMoneyFundamentalProvider;
pub use provider::FundamentalProvider;
pub use types::{
    DragonTigerItem, EarningsReport, FundamentalData, InstitutionHolding, ValuationMetrics,
};
