//! 交易日历数据类型。
//!
//! 从原生 `trading_calendar.rs` 拆出，承载 `TradingSession`、
//! `TradingStatus`、`HolidayData` 以及反序列化专用的
//! `HolidayConfig` / `YearHolidays`。

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 交易时段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradingSession {
    /// 上午交易 9:30-11:30
    Morning,
    /// 下午交易 13:00-15:00
    Afternoon,
    /// 集合竞价 9:15-9:25
    Auction,
    /// 休市
    Closed,
}

impl TradingSession {
    /// 返回稳定字符串标识（"morning"/"afternoon"/"auction"/"closed"），用于入库与跨模块传递。
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingSession::Morning => "morning",
            TradingSession::Afternoon => "afternoon",
            TradingSession::Auction => "auction",
            TradingSession::Closed => "closed",
        }
    }

    /// 返回中文展示名（"上午交易"/"下午交易"/"集合竞价"/"休市"），用于 UI 与报告。
    pub fn display_name(&self) -> &'static str {
        match self {
            TradingSession::Morning => "上午交易",
            TradingSession::Afternoon => "下午交易",
            TradingSession::Auction => "集合竞价",
            TradingSession::Closed => "休市",
        }
    }
}

/// 交易状态
#[derive(Debug, Clone)]
pub struct TradingStatus {
    /// 是否为交易日
    pub is_trading_day: bool,
    /// 当前交易时段
    pub current_session: TradingSession,
    /// 下次开盘时间
    pub next_open_time: chrono::DateTime<chrono::Local>,
}

/// 节假日数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayData {
    pub year: i32,
    pub holidays: Vec<String>,    // YYYY-MM-DD 格式
    pub early_close: Vec<String>, // 提前收盘时间
}

/// 节假日配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HolidayConfig {
    pub(crate) description: Option<String>,
    pub(crate) source: Option<String>,
    pub(crate) years: std::collections::HashMap<String, YearHolidays>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct YearHolidays {
    pub(crate) holidays: Vec<String>,
    pub(crate) early_close: Vec<String>,
    pub(crate) workdays_on_weekend: Vec<String>,
}

impl YearHolidays {
    pub(crate) fn holiday_dates(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        self.holidays
            .iter()
            .filter_map(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
    }

    pub(crate) fn workday_dates(&self) -> impl Iterator<Item = NaiveDate> + '_ {
        self.workdays_on_weekend
            .iter()
            .filter_map(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok())
    }
}
