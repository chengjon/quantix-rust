/// A股交易时间处理
///
/// 处理 A股特有的交易时段、节假日等
use chrono::{Datelike, NaiveDateTime, NaiveTime};

/// A股交易时段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingSession {
    Morning,
    Afternoon,
}

impl TradingSession {
    fn time(hour: u32, minute: u32, second: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, second).unwrap_or_default()
    }

    /// 返回该时段开盘时间：Morning=09:30:00、Afternoon=13:00:00。
    pub fn start_time(&self) -> NaiveTime {
        match self {
            TradingSession::Morning => Self::time(9, 30, 0),
            TradingSession::Afternoon => Self::time(13, 0, 0),
        }
    }

    /// 返回该时段收盘时间：Morning=11:30:00、Afternoon=15:00:00。
    pub fn end_time(&self) -> NaiveTime {
        match self {
            TradingSession::Morning => Self::time(11, 30, 0),
            TradingSession::Afternoon => Self::time(15, 0, 0),
        }
    }

    /// 返回所有交易时段（按时间顺序：Morning 在前、Afternoon 在后）。
    pub fn all_sessions() -> Vec<TradingSession> {
        vec![TradingSession::Morning, TradingSession::Afternoon]
    }
}

/// 检查给定时间是否在交易时段内
pub fn is_trading_time(dt: NaiveDateTime) -> bool {
    let time = dt.time();
    // weekday(): Mon=0, Sun=6
    let weekday = dt.weekday().num_days_from_monday();

    // 周末不交易 (5=Saturday, 6=Sunday)
    if weekday >= 5 {
        return false;
    }

    // 检查是否在任一交易时段内
    for session in TradingSession::all_sessions() {
        let start = session.start_time();
        let end = session.end_time();
        if time >= start && time <= end {
            return true;
        }
    }

    false
}

/// 获取下一个交易时间（简化版，未考虑节假日）
pub fn next_trading_time(dt: NaiveDateTime) -> NaiveDateTime {
    let mut current = dt;

    loop {
        // 尝试向后推移到下一个可能的交易时间
        current += chrono::Duration::minutes(1);

        // weekday(): Mon=0, Sun=6
        let weekday = current.weekday().num_days_from_monday();
        let time = current.time();

        // 如果是周一到周五 (0-4)
        if weekday < 5 {
            // 检查交易时段
            for session in TradingSession::all_sessions() {
                let start = session.start_time();
                if time == start {
                    return current;
                }
            }
        }
    }
}
