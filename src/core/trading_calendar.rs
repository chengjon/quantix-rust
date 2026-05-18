/// A股交易时间日历
///
/// 从短线侠项目迁移，支持交易时段检测和节假日判断
use crate::core::{QuantixError, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    pub fn as_str(&self) -> &'static str {
        match self {
            TradingSession::Morning => "morning",
            TradingSession::Afternoon => "afternoon",
            TradingSession::Auction => "auction",
            TradingSession::Closed => "closed",
        }
    }

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
    pub next_open_time: chrono::DateTime<Local>,
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
struct HolidayConfig {
    description: Option<String>,
    source: Option<String>,
    years: HashMap<String, YearHolidays>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct YearHolidays {
    holidays: Vec<String>,
    early_close: Vec<String>,
    workdays_on_weekend: Vec<String>,
}

/// A股交易日历管理器
pub struct TradingCalendar {
    /// 每年的节假日缓存，key为年份，value为节假日日期集合
    holidays: HashMap<i32, HashSet<NaiveDate>>,
    /// 调休工作日（周末补班）
    workdays_on_weekend: HashMap<i32, HashSet<NaiveDate>>,
    /// 配置文件路径
    config_path: Option<std::path::PathBuf>,
}

impl TradingCalendar {
    // 交易时段时间常量
    const AUCTION_START: (u32, u32, u32) = (9, 15, 0); // 集合竞价开始时间
    const AUCTION_END: (u32, u32, u32) = (9, 25, 0); // 集合竞价结束时间
    const MORNING_START: (u32, u32, u32) = (9, 30, 0); // 上午交易开始时间
    const MORNING_END: (u32, u32, u32) = (11, 30, 0); // 上午交易结束时间
    const AFTERNOON_START: (u32, u32, u32) = (13, 0, 0); // 下午交易开始时间
    const AFTERNOON_END: (u32, u32, u32) = (15, 0, 0); // 下午交易结束时间

    /// 创建新的交易日历实例
    pub async fn new() -> Result<Self> {
        tracing::info!("初始化交易日历");
        Ok(Self {
            holidays: HashMap::new(),
            workdays_on_weekend: HashMap::new(),
            config_path: None,
        })
    }

    /// 从配置文件创建交易日历
    pub async fn from_config(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let mut calendar = Self {
            holidays: HashMap::new(),
            workdays_on_weekend: HashMap::new(),
            config_path: Some(path.as_ref().to_path_buf()),
        };

        calendar.load_config().await?;
        Ok(calendar)
    }

    /// 从默认路径加载配置
    pub async fn from_default_config() -> Result<Self> {
        let default_paths = vec![
            std::path::PathBuf::from("config/holidays.json"),
            std::path::PathBuf::from("/etc/quantix/holidays.json"),
        ];

        for path in default_paths {
            if path.exists() {
                return Self::from_config(&path).await;
            }
        }

        tracing::warn!("未找到节假日配置文件，使用空节假日数据");
        Self::new().await
    }

    /// 加载配置文件
    async fn load_config(&mut self) -> Result<()> {
        let path = match &self.config_path {
            Some(p) => p.clone(),
            None => return Ok(()),
        };

        if !path.exists() {
            tracing::warn!("节假日配置文件不存在: {:?}", path);
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| QuantixError::Other(format!("读取节假日配置失败: {}", e)))?;

        let config: HolidayConfig = serde_json::from_str(&content)
            .map_err(|e| QuantixError::Other(format!("解析节假日配置失败: {}", e)))?;

        for (year_str, year_data) in config.years {
            let year: i32 = year_str
                .parse()
                .map_err(|_| QuantixError::Other(format!("无效的年份: {}", year_str)))?;

            // 解析节假日
            let mut holiday_set = HashSet::new();
            for date_str in &year_data.holidays {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    holiday_set.insert(date);
                }
            }
            self.holidays.insert(year, holiday_set);

            // 解析调休工作日
            let mut workday_set = HashSet::new();
            for date_str in &year_data.workdays_on_weekend {
                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    workday_set.insert(date);
                }
            }
            self.workdays_on_weekend.insert(year, workday_set);
        }

        tracing::info!("已加载 {} 年的节假日数据", self.holidays.len());

        Ok(())
    }

    /// 判断指定日期是否为交易日
    /// 交易日 = 工作日且非节假日，或调休工作日
    pub async fn is_trading_day(&self, date: NaiveDate) -> bool {
        // 1. 检查是否为调休工作日（周末补班）
        if self.is_workday_on_weekend(date) {
            return true;
        }

        // 2. 检查是否为周末
        if self.is_weekend(date) {
            return false;
        }

        // 3. 检查是否为节假日
        if self.is_holiday(date).await {
            return false;
        }

        true
    }

    /// 判断当前是否在交易时段内
    pub async fn is_in_trading_hours(&self) -> bool {
        let now = Local::now();
        let current_time = now.time();
        let date = now.date_naive();

        // 1. 检查是否为交易日
        if !self.is_trading_day(date).await {
            return false;
        }

        // 2. 检查是否在交易时段内
        let auction_start = NaiveTime::from_hms_opt(
            Self::AUCTION_START.0,
            Self::AUCTION_START.1,
            Self::AUCTION_START.2,
        )
        .unwrap();
        let auction_end = NaiveTime::from_hms_opt(
            Self::AUCTION_END.0,
            Self::AUCTION_END.1,
            Self::AUCTION_END.2,
        )
        .unwrap();
        let morning_start = NaiveTime::from_hms_opt(
            Self::MORNING_START.0,
            Self::MORNING_START.1,
            Self::MORNING_START.2,
        )
        .unwrap();
        let morning_end = NaiveTime::from_hms_opt(
            Self::MORNING_END.0,
            Self::MORNING_END.1,
            Self::MORNING_END.2,
        )
        .unwrap();
        let afternoon_start = NaiveTime::from_hms_opt(
            Self::AFTERNOON_START.0,
            Self::AFTERNOON_START.1,
            Self::AFTERNOON_START.2,
        )
        .unwrap();
        let afternoon_end = NaiveTime::from_hms_opt(
            Self::AFTERNOON_END.0,
            Self::AFTERNOON_END.1,
            Self::AFTERNOON_END.2,
        )
        .unwrap();

        (current_time >= auction_start && current_time <= auction_end)
            || (current_time >= morning_start && current_time <= morning_end)
            || (current_time >= afternoon_start && current_time <= afternoon_end)
    }

    /// 获取当前交易状态
    pub async fn get_current_status(&self) -> TradingStatus {
        let now = Local::now();
        let current_time = now.time();
        let date = now.date_naive();
        let is_trading_day = self.is_trading_day(date).await;

        let current_session = if !is_trading_day {
            TradingSession::Closed
        } else {
            let auction_start = NaiveTime::from_hms_opt(
                Self::AUCTION_START.0,
                Self::AUCTION_START.1,
                Self::AUCTION_START.2,
            )
            .unwrap();
            let auction_end = NaiveTime::from_hms_opt(
                Self::AUCTION_END.0,
                Self::AUCTION_END.1,
                Self::AUCTION_END.2,
            )
            .unwrap();
            let morning_start = NaiveTime::from_hms_opt(
                Self::MORNING_START.0,
                Self::MORNING_START.1,
                Self::MORNING_START.2,
            )
            .unwrap();
            let morning_end = NaiveTime::from_hms_opt(
                Self::MORNING_END.0,
                Self::MORNING_END.1,
                Self::MORNING_END.2,
            )
            .unwrap();
            let afternoon_start = NaiveTime::from_hms_opt(
                Self::AFTERNOON_START.0,
                Self::AFTERNOON_START.1,
                Self::AFTERNOON_START.2,
            )
            .unwrap();
            let afternoon_end = NaiveTime::from_hms_opt(
                Self::AFTERNOON_END.0,
                Self::AFTERNOON_END.1,
                Self::AFTERNOON_END.2,
            )
            .unwrap();

            if current_time >= auction_start && current_time <= auction_end {
                TradingSession::Auction
            } else if current_time >= morning_start && current_time <= morning_end {
                TradingSession::Morning
            } else if current_time >= afternoon_start && current_time <= afternoon_end {
                TradingSession::Afternoon
            } else {
                TradingSession::Closed
            }
        };

        // 计算下次开盘时间
        let next_open_time = self.calculate_next_open_time(now).await;

        TradingStatus {
            is_trading_day,
            current_session,
            next_open_time,
        }
    }

    /// 计算下次开盘时间
    async fn calculate_next_open_time(
        &self,
        now: chrono::DateTime<Local>,
    ) -> chrono::DateTime<Local> {
        let current_time = now.time();
        let date = now.date_naive();

        // 如果今天不是交易日，找到下一个交易日
        if !self.is_trading_day(date).await {
            let mut next_date = date + Duration::days(1);
            while !self.is_trading_day(next_date).await {
                next_date = next_date + Duration::days(1);
            }
            return Local
                .with_ymd_and_hms(
                    next_date.year(),
                    next_date.month(),
                    next_date.day(),
                    9,
                    15,
                    0,
                )
                .unwrap();
        }

        // 今天是交易日，判断当前时间
        let auction_start = NaiveTime::from_hms_opt(9, 15, 0).unwrap();
        let afternoon_end = NaiveTime::from_hms_opt(15, 0, 0).unwrap();

        if current_time < auction_start {
            // 还没到开盘，返回今天的开盘时间
            Local
                .with_ymd_and_hms(date.year(), date.month(), date.day(), 9, 15, 0)
                .unwrap()
        } else if current_time > afternoon_end {
            // 已收盘，返回明天的开盘时间
            let mut next_date = date + Duration::days(1);
            while !self.is_trading_day(next_date).await {
                next_date = next_date + Duration::days(1);
            }
            Local
                .with_ymd_and_hms(
                    next_date.year(),
                    next_date.month(),
                    next_date.day(),
                    9,
                    15,
                    0,
                )
                .unwrap()
        } else {
            // 交易时段内，返回下一个交易时段的开始时间
            let morning_start = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
            let morning_end = NaiveTime::from_hms_opt(11, 30, 0).unwrap();
            let afternoon_start = NaiveTime::from_hms_opt(13, 0, 0).unwrap();

            if current_time >= morning_start && current_time < morning_end {
                // 上午交易时段，返回下午开盘
                Local
                    .with_ymd_and_hms(date.year(), date.month(), date.day(), 13, 0, 0)
                    .unwrap()
            } else {
                // 下午交易时段，返回明天的开盘
                let mut next_date = date + Duration::days(1);
                while !self.is_trading_day(next_date).await {
                    next_date = next_date + Duration::days(1);
                }
                Local
                    .with_ymd_and_hms(
                        next_date.year(),
                        next_date.month(),
                        next_date.day(),
                        9,
                        15,
                        0,
                    )
                    .unwrap()
            }
        }
    }

    /// 判断是否为周末（周六或周日）
    fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// 判断是否为调休工作日（周末补班）
    fn is_workday_on_weekend(&self, date: NaiveDate) -> bool {
        let year = date.year();
        if let Some(workdays) = self.workdays_on_weekend.get(&year) {
            workdays.contains(&date)
        } else {
            false
        }
    }

    /// 判断是否为节假日
    async fn is_holiday(&self, date: NaiveDate) -> bool {
        let year = date.year();
        if let Some(year_holidays) = self.holidays.get(&year) {
            year_holidays.contains(&date)
        } else {
            // 如果没有该年的节假日数据，尝试加载
            // 简化实现：暂时返回false（没有节假日数据）
            false
        }
    }

    /// 加载指定年份的节假日数据
    pub async fn load_holidays_for_year(&mut self, year: i32) -> Result<()> {
        // TODO: 实现从文件或数据库加载节假日数据
        // 可以从 Python quantix 项目的节假日数据同步
        let holiday_set = HashSet::new();

        tracing::info!("加载 {} 年节假日数据 (当前为空，待实现)", year);

        self.holidays.insert(year, holiday_set);
        Ok(())
    }

    /// 获取建议的采集间隔（秒）
    /// 根据当前交易状态返回合理的采集间隔
    pub async fn get_recommended_interval(&self) -> u64 {
        let status = self.get_current_status().await;

        match status.current_session {
            TradingSession::Auction => 30,   // 竞价期间 30秒
            TradingSession::Morning => 60,   // 上午交易 60秒
            TradingSession::Afternoon => 60, // 下午交易 60秒
            TradingSession::Closed => {
                if status.is_trading_day {
                    // 交易日但休市（午休）: 5分钟
                    300
                } else {
                    // 非交易日: 30分钟
                    1800
                }
            }
        }
    }
}

impl Default for TradingCalendar {
    fn default() -> Self {
        Self {
            holidays: HashMap::new(),
            workdays_on_weekend: HashMap::new(),
            config_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trading_calendar_creation() {
        let calendar = TradingCalendar::new().await;
        assert!(calendar.is_ok());
    }

    #[test]
    fn test_is_weekend() {
        let calendar = TradingCalendar::default();
        // 2026-03-01 是周六
        let date = NaiveDate::from_ymd_opt(2026, 2, 1).unwrap();
        assert!(calendar.is_weekend(date));

        // 2026-03-03 是周一
        let date = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
        assert!(!calendar.is_weekend(date));
    }

    #[test]
    fn test_trading_session_display() {
        assert_eq!(TradingSession::Morning.as_str(), "morning");
        assert_eq!(TradingSession::Afternoon.as_str(), "afternoon");
        assert_eq!(TradingSession::Auction.as_str(), "auction");
        assert_eq!(TradingSession::Closed.as_str(), "closed");
    }
}
