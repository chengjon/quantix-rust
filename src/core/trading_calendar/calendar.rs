//! A股交易日历管理器实现。
//!
//! 从原生 `trading_calendar.rs` 拆出，承载 [`TradingCalendar`] 结构体
//! 及其全部方法；类型定义见 [`super::types`]。

use super::types::YearHolidays;

use crate::core::{QuantixError, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveTime, TimeZone, Weekday};
use std::collections::{HashMap, HashSet};

/// A股交易日历管理器
#[derive(Default)]
pub struct TradingCalendar {
    /// 每年的节假日缓存，key为年份，value为节假日日期集合
    pub(crate) holidays: HashMap<i32, HashSet<NaiveDate>>,
    /// 调休工作日（周末补班）
    pub(crate) workdays_on_weekend: HashMap<i32, HashSet<NaiveDate>>,
    /// 配置文件路径
    pub(crate) config_path: Option<std::path::PathBuf>,
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
        let Some(config) = self.read_holiday_config().await? else {
            return Ok(());
        };

        for (year_str, year_data) in config.years {
            let year: i32 = year_str
                .parse()
                .map_err(|_| QuantixError::Other(format!("无效的年份: {}", year_str)))?;

            let (holiday_set, workday_set) = Self::parse_year_holidays(&year_data);
            self.holidays.insert(year, holiday_set);
            self.workdays_on_weekend.insert(year, workday_set);
        }

        tracing::info!("已加载 {} 年的节假日数据", self.holidays.len());

        Ok(())
    }

    async fn read_holiday_config(&self) -> Result<Option<super::types::HolidayConfig>> {
        let path = match &self.config_path {
            Some(path) => path,
            None => return Ok(None),
        };

        if !path.exists() {
            tracing::warn!("节假日配置文件不存在: {:?}", path);
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| QuantixError::Other(format!("读取节假日配置失败: {}", e)))?;

        let config = serde_json::from_str(&content)
            .map_err(|e| QuantixError::Other(format!("解析节假日配置失败: {}", e)))?;

        Ok(Some(config))
    }

    fn parse_year_holidays(year_data: &YearHolidays) -> (HashSet<NaiveDate>, HashSet<NaiveDate>) {
        let holiday_set = year_data.holiday_dates().collect();
        let workday_set = year_data.workday_dates().collect();
        (holiday_set, workday_set)
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
        let auction_start = Self::session_time(Self::AUCTION_START);
        let auction_end = Self::session_time(Self::AUCTION_END);
        let morning_start = Self::session_time(Self::MORNING_START);
        let morning_end = Self::session_time(Self::MORNING_END);
        let afternoon_start = Self::session_time(Self::AFTERNOON_START);
        let afternoon_end = Self::session_time(Self::AFTERNOON_END);

        (current_time >= auction_start && current_time <= auction_end)
            || (current_time >= morning_start && current_time <= morning_end)
            || (current_time >= afternoon_start && current_time <= afternoon_end)
    }

    /// 获取当前交易状态
    pub async fn get_current_status(&self) -> super::types::TradingStatus {
        let now = Local::now();
        let current_time = now.time();
        let date = now.date_naive();
        let is_trading_day = self.is_trading_day(date).await;

        let current_session = if !is_trading_day {
            super::types::TradingSession::Closed
        } else {
            let auction_start = Self::session_time(Self::AUCTION_START);
            let auction_end = Self::session_time(Self::AUCTION_END);
            let morning_start = Self::session_time(Self::MORNING_START);
            let morning_end = Self::session_time(Self::MORNING_END);
            let afternoon_start = Self::session_time(Self::AFTERNOON_START);
            let afternoon_end = Self::session_time(Self::AFTERNOON_END);

            if current_time >= auction_start && current_time <= auction_end {
                super::types::TradingSession::Auction
            } else if current_time >= morning_start && current_time <= morning_end {
                super::types::TradingSession::Morning
            } else if current_time >= afternoon_start && current_time <= afternoon_end {
                super::types::TradingSession::Afternoon
            } else {
                super::types::TradingSession::Closed
            }
        };

        // 计算下次开盘时间
        let next_open_time = self.calculate_next_open_time(now).await;

        super::types::TradingStatus {
            is_trading_day,
            current_session,
            next_open_time,
        }
    }

    /// 计算下次开盘时间
    async fn calculate_next_open_time(
        &self,
        now: chrono::DateTime<chrono::Local>,
    ) -> chrono::DateTime<chrono::Local> {
        let current_time = now.time();
        let date = now.date_naive();

        // 如果今天不是交易日，找到下一个交易日
        if !self.is_trading_day(date).await {
            let mut next_date = date + Duration::days(1);
            while !self.is_trading_day(next_date).await {
                next_date += Duration::days(1);
            }
            return Self::local_session_datetime(next_date, Self::AUCTION_START);
        }

        // 今天是交易日，判断当前时间
        let auction_start = Self::session_time(Self::AUCTION_START);
        let afternoon_end = Self::session_time(Self::AFTERNOON_END);

        if current_time < auction_start {
            // 还没到开盘，返回今天的开盘时间
            Self::local_session_datetime(date, Self::AUCTION_START)
        } else if current_time > afternoon_end {
            // 已收盘，返回明天的开盘时间
            let mut next_date = date + Duration::days(1);
            while !self.is_trading_day(next_date).await {
                next_date += Duration::days(1);
            }
            Self::local_session_datetime(next_date, Self::AUCTION_START)
        } else {
            // 交易时段内，返回下一个交易时段的开始时间
            let morning_start = Self::session_time(Self::MORNING_START);
            let morning_end = Self::session_time(Self::MORNING_END);

            if current_time >= morning_start && current_time < morning_end {
                // 上午交易时段，返回下午开盘
                Self::local_session_datetime(date, Self::AFTERNOON_START)
            } else {
                // 下午交易时段，返回明天的开盘
                let mut next_date = date + Duration::days(1);
                while !self.is_trading_day(next_date).await {
                    next_date += Duration::days(1);
                }
                Self::local_session_datetime(next_date, Self::AUCTION_START)
            }
        }
    }

    fn session_time((hour, minute, second): (u32, u32, u32)) -> NaiveTime {
        NaiveTime::from_hms_opt(hour, minute, second).unwrap_or_default()
    }

    fn local_datetime(date: NaiveDate, time: NaiveTime) -> chrono::DateTime<chrono::Local> {
        Local
            .from_local_datetime(&date.and_time(time))
            .earliest()
            .unwrap_or_else(|| {
                // 回退：把 naive 当 UTC 再贴 Local offset。理论上 from_local_datetime 的
                // earliest() 在合法 wall-clock 下不会返回 None；这里只在 DST 切换边缘兜底。
                let utc = date.and_time(time);
                chrono::TimeZone::from_local_datetime(&Local, &utc)
                    .earliest()
                    .unwrap_or_else(|| chrono::Local.from_utc_datetime(&utc))
            })
    }

    fn local_session_datetime(
        date: NaiveDate,
        session_time: (u32, u32, u32),
    ) -> chrono::DateTime<chrono::Local> {
        Self::local_datetime(date, Self::session_time(session_time))
    }

    /// 判断是否为周末（周六或周日）
    pub(crate) fn is_weekend(&self, date: NaiveDate) -> bool {
        matches!(date.weekday(), Weekday::Sat | Weekday::Sun)
    }

    /// 判断是否为调休工作日（周末补班）
    pub(crate) fn is_workday_on_weekend(&self, date: NaiveDate) -> bool {
        let year = date.year();
        if let Some(workdays) = self.workdays_on_weekend.get(&year) {
            workdays.contains(&date)
        } else {
            false
        }
    }

    /// 判断是否为节假日
    pub(crate) async fn is_holiday(&self, date: NaiveDate) -> bool {
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
        let year_key = year.to_string();
        let (holiday_set, workday_set) = self
            .read_holiday_config()
            .await?
            .and_then(|config| config.years.get(&year_key).map(Self::parse_year_holidays))
            .unwrap_or_default();

        self.holidays.insert(year, holiday_set);
        self.workdays_on_weekend.insert(year, workday_set);
        tracing::info!("已加载 {} 年节假日数据", year);

        Ok(())
    }

    /// 从交易日列表同步日历
    ///
    /// 遍历全年，推导出节假日（非交易日的平日）和调休日（交易日的周末）
    pub fn sync_trading_days(&mut self, year: i32, trading_days: Vec<NaiveDate>) {
        let trading_set: HashSet<NaiveDate> = trading_days.into_iter().collect();
        let mut holidays = HashSet::new();
        let mut workdays_on_weekend = HashSet::new();

        let start = NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(NaiveDate::MIN);
        let end = NaiveDate::from_ymd_opt(year, 12, 31).unwrap_or(NaiveDate::MAX);

        let mut d = start;
        while d <= end {
            let weekend = matches!(d.weekday(), Weekday::Sat | Weekday::Sun);
            let trading = trading_set.contains(&d);
            if weekend && trading {
                workdays_on_weekend.insert(d);
            } else if !weekend && !trading {
                holidays.insert(d);
            }
            d += Duration::days(1);
        }

        let h_count = holidays.len();
        let w_count = workdays_on_weekend.len();
        self.holidays.insert(year, holidays);
        self.workdays_on_weekend.insert(year, workdays_on_weekend);
        tracing::info!(
            "已同步 {} 年日历: {} 个节假日, {} 个调休日",
            year,
            h_count,
            w_count
        );
    }

    /// 获取指定年份的节假日列表
    pub fn holidays_for_year(&self, year: i32) -> Vec<NaiveDate> {
        self.holidays
            .get(&year)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// 获取指定年份的调休日列表
    pub fn workdays_on_weekend_for_year(&self, year: i32) -> Vec<NaiveDate> {
        self.workdays_on_weekend
            .get(&year)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// 获取建议的采集间隔（秒）
    /// 根据当前交易状态返回合理的采集间隔
    pub async fn get_recommended_interval(&self) -> u64 {
        let status = self.get_current_status().await;

        match status.current_session {
            super::types::TradingSession::Auction => 30, // 竞价期间 30秒
            super::types::TradingSession::Morning => 60, // 上午交易 60秒
            super::types::TradingSession::Afternoon => 60, // 下午交易 60秒
            super::types::TradingSession::Closed => {
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
