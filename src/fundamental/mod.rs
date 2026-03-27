//! 基本面数据模块
//!
//! 提供股票基本面数据获取能力：估值、财报、龙虎榜等

pub mod types;
pub mod provider;
pub mod valuation;
pub mod earnings;
pub mod institution;
pub mod dragon_tiger;
pub mod eastmoney;

pub use types::{
    FundamentalData, ValuationMetrics, EarningsReport,
    InstitutionHolding, DragonTigerItem,
};
pub use provider::FundamentalProvider;
pub use eastmoney::EastMoneyFundamentalProvider;
