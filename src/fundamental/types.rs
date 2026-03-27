//! 基本面数据类型定义

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// 基本面数据汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundamentalData {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 数据日期
    pub date: NaiveDate,
    /// 估值指标
    pub valuation: Option<ValuationMetrics>,
    /// 最新财报
    pub latest_earnings: Option<EarningsReport>,
    /// 机构持仓
    pub institution_holdings: Vec<InstitutionHolding>,
    /// 数据来源
    pub source: String,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 估值指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationMetrics {
    /// 股票代码
    pub code: String,
    /// 数据日期
    pub date: NaiveDate,
    /// 市盈率 (TTM)
    pub pe_ttm: Option<Decimal>,
    /// 市盈率 (静态)
    pub pe_static: Option<Decimal>,
    /// 市净率
    pub pb: Option<Decimal>,
    /// 市销率
    pub ps: Option<Decimal>,
    /// 总市值 (亿元)
    pub market_cap: Option<Decimal>,
    /// 流通市值 (亿元)
    pub float_market_cap: Option<Decimal>,
    /// 股息率 (%)
    pub dividend_yield: Option<Decimal>,
    /// 每股收益
    pub eps: Option<Decimal>,
    /// 每股净资产
    pub bvps: Option<Decimal>,
    /// 净资产收益率 (%)
    pub roe: Option<Decimal>,
    /// 毛利率 (%)
    pub gross_margin: Option<Decimal>,
    /// 净利率 (%)
    pub net_margin: Option<Decimal>,
}

impl ValuationMetrics {
    pub fn new(code: String, date: NaiveDate) -> Self {
        Self {
            code,
            date,
            pe_ttm: None,
            pe_static: None,
            pb: None,
            ps: None,
            market_cap: None,
            float_market_cap: None,
            dividend_yield: None,
            eps: None,
            bvps: None,
            roe: None,
            gross_margin: None,
            net_margin: None,
        }
    }
}

/// 财报数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsReport {
    /// 股票代码
    pub code: String,
    /// 报告期
    pub report_date: NaiveDate,
    /// 报告类型 (一季报/半年报/三季报/年报)
    pub report_type: String,
    /// 营业收入 (亿元)
    pub revenue: Option<Decimal>,
    /// 营业收入同比增长 (%)
    pub revenue_yoy: Option<Decimal>,
    /// 净利润 (亿元)
    pub net_profit: Option<Decimal>,
    /// 净利润同比增长 (%)
    pub net_profit_yoy: Option<Decimal>,
    /// 扣非净利润 (亿元)
    pub net_profit_deducted: Option<Decimal>,
    /// 经营活动现金流 (亿元)
    pub operating_cash_flow: Option<Decimal>,
    /// 总资产 (亿元)
    pub total_assets: Option<Decimal>,
    /// 净资产 (亿元)
    pub net_assets: Option<Decimal>,
    /// 资产负债率 (%)
    pub debt_ratio: Option<Decimal>,
    /// 毛利率 (%)
    pub gross_margin: Option<Decimal>,
    /// 净利率 (%)
    pub net_margin: Option<Decimal>,
    /// 公告日期
    pub announce_date: Option<NaiveDate>,
}

impl EarningsReport {
    pub fn new(code: String, report_date: NaiveDate, report_type: String) -> Self {
        Self {
            code,
            report_date,
            report_type,
            revenue: None,
            revenue_yoy: None,
            net_profit: None,
            net_profit_yoy: None,
            net_profit_deducted: None,
            operating_cash_flow: None,
            total_assets: None,
            net_assets: None,
            debt_ratio: None,
            gross_margin: None,
            net_margin: None,
            announce_date: None,
        }
    }
}

/// 机构持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionHolding {
    /// 股票代码
    pub code: String,
    /// 机构名称
    pub institution_name: String,
    /// 机构类型 (基金/社保/券商/险资/QFII)
    pub institution_type: String,
    /// 持股数量 (万股)
    pub shares: Decimal,
    /// 持股市值 (万元)
    pub market_value: Option<Decimal>,
    /// 占流通股比例 (%)
    pub float_ratio: Option<Decimal>,
    /// 变动方向 (新进/增持/减持/不变)
    pub change_direction: String,
    /// 变动数量 (万股)
    pub change_shares: Option<Decimal>,
    /// 报告期
    pub report_date: NaiveDate,
}

/// 龙虎榜数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragonTigerItem {
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 交易日期
    pub trade_date: NaiveDate,
    /// 收盘价
    pub close_price: Decimal,
    /// 涨跌幅 (%)
    pub change_pct: Decimal,
    /// 上榜原因
    pub reason: String,
    /// 买入金额 (万元)
    pub buy_amount: Decimal,
    /// 卖出金额 (万元)
    pub sell_amount: Decimal,
    /// 净买入金额 (万元)
    pub net_buy: Decimal,
    /// 买入前5营业部
    pub top_buyers: Vec<BrokerActivity>,
    /// 卖出前5营业部
    pub top_sellers: Vec<BrokerActivity>,
}

/// 营业部交易明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerActivity {
    /// 营业部名称
    pub broker_name: String,
    /// 买入金额 (万元)
    pub buy_amount: Option<Decimal>,
    /// 卖出金额 (万元)
    pub sell_amount: Option<Decimal>,
    /// 净买入 (万元)
    pub net_buy: Option<Decimal>,
}

/// 分红信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DividendInfo {
    /// 股票代码
    pub code: String,
    /// 报告年度
    pub fiscal_year: i32,
    /// 分红方案
    pub dividend_plan: String,
    /// 每股股利 (元)
    pub dividend_per_share: Decimal,
    /// 股息率 (%)
    pub dividend_yield: Option<Decimal>,
    /// 除息日
    pub ex_dividend_date: Option<NaiveDate>,
    /// 派息日
    pub pay_date: Option<NaiveDate>,
    /// 公告日期
    pub announce_date: Option<NaiveDate>,
}

/// 资金流向
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapitalFlow {
    /// 股票代码
    pub code: String,
    /// 交易日期
    pub date: NaiveDate,
    /// 主力净流入 (万元)
    pub main_net_inflow: Decimal,
    /// 超大单净流入 (万元)
    pub super_large_net_inflow: Decimal,
    /// 大单净流入 (万元)
    pub large_net_inflow: Decimal,
    /// 中单净流入 (万元)
    pub medium_net_inflow: Decimal,
    /// 小单净流入 (万元)
    pub small_net_inflow: Decimal,
    /// 成交额 (万元)
    pub turnover: Decimal,
}
