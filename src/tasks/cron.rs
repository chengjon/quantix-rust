/// Cron 表达式解析
///
/// 从短线侠项目迁移 - 解析和验证 cron 表达式
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};

/// Cron 表达式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExpression {
    /// 分钟 (0-59)
    pub minutes: CronField,
    /// 小时 (0-23)
    pub hours: CronField,
    /// 日 (1-31)
    pub days: CronField,
    /// 月 (1-12)
    pub months: CronField,
    /// 星期 (0-6, 0=周日)
    pub weekdays: CronField,
    /// 原始表达式
    raw: String,
}

/// Cron 字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronField {
    /// 所有值
    All,
    /// 特定值
    Specific(Vec<u32>),
    /// 范围
    Range(u32, u32),
    /// 步长
    Step { base: Box<CronField>, step: u32 },
}

impl CronExpression {
    /// 解析 cron 表达式
    ///
    /// 格式: 分 时 日 月 周
    /// 例如: "0 9 * * 1-5" 表示工作日早上9点
    pub fn new(expression: &str) -> Result<Self, String> {
        let parts: Vec<&str> = expression.split_whitespace().collect();

        if parts.len() != 5 {
            return Err(format!("无效的 cron 表达式: {}, 需要 5 个字段", expression));
        }

        let minutes = Self::parse_field(parts[0], 0, 59)?;
        let hours = Self::parse_field(parts[1], 0, 23)?;
        let days = Self::parse_field(parts[2], 1, 31)?;
        let months = Self::parse_field(parts[3], 1, 12)?;
        let weekdays = Self::parse_field(parts[4], 0, 6)?;

        Ok(Self {
            minutes,
            hours,
            days,
            months,
            weekdays,
            raw: expression.to_string(),
        })
    }

    /// 解析单个字段
    fn parse_field(expr: &str, min: u32, max: u32) -> Result<CronField, String> {
        if expr == "*" {
            return Ok(CronField::All);
        }

        // 处理步长 (e.g., */5, 1-10/2)
        if let Some(idx) = expr.find('/') {
            let base_expr = &expr[..idx];
            let step: u32 = expr[idx + 1..]
                .parse()
                .map_err(|_| format!("无效的步长: {}", expr))?;

            if step == 0 {
                return Err("步长不能为0".to_string());
            }

            let base = if base_expr == "*" {
                CronField::All
            } else {
                Self::parse_field(base_expr, min, max)?
            };

            return Ok(CronField::Step {
                base: Box::new(base),
                step,
            });
        }

        // 处理范围 (e.g., 1-5)
        if let Some(idx) = expr.find('-') {
            let start: u32 = expr[..idx]
                .parse()
                .map_err(|_| format!("无效的范围起始: {}", expr))?;
            let end: u32 = expr[idx + 1..]
                .parse()
                .map_err(|_| format!("无效的范围结束: {}", expr))?;

            if start < min || start > max || end < min || end > max {
                return Err(format!("范围超出界限: {} ({}-{})", expr, min, max));
            }

            if start > end {
                return Err(format!("范围起始大于结束: {}", expr));
            }

            return Ok(CronField::Range(start, end));
        }

        // 处理列表 (e.g., 1,2,3)
        if expr.contains(',') {
            let values: Result<Vec<u32>, _> = expr
                .split(',')
                .map(|s| {
                    let v: u32 = s.trim().parse().map_err(|_| format!("无效的值: {}", s))?;
                    if v < min || v > max {
                        Err(format!("值超出界限: {} ({}-{})", v, min, max))
                    } else {
                        Ok(v)
                    }
                })
                .collect();

            return Ok(CronField::Specific(values?));
        }

        // 单个值
        let value: u32 = expr
            .parse()
            .map_err(|_| format!("无效的字段值: {}", expr))?;

        if value < min || value > max {
            return Err(format!("值超出界限: {} ({}-{})", value, min, max));
        }

        Ok(CronField::Specific(vec![value]))
    }

    /// 检查给定时间是否匹配 cron 表达式
    pub fn should_run(&self, dt: &NaiveDateTime) -> bool {
        self.matches_field(&self.minutes, dt.minute())
            && self.matches_field(&self.hours, dt.hour())
            && self.matches_field(&self.days, dt.day() as u32)
            && self.matches_field(&self.months, dt.month() as u32)
            && self.matches_field(&self.weekdays, dt.weekday().num_days_from_sunday())
    }

    /// 检查字段是否匹配
    fn matches_field(&self, field: &CronField, value: u32) -> bool {
        match field {
            CronField::All => true,
            CronField::Specific(values) => values.contains(&value),
            CronField::Range(start, end) => value >= *start && value <= *end,
            CronField::Step { base, step } => self.matches_step_field(base.as_ref(), value, *step),
        }
    }

    fn matches_step_field(&self, base: &CronField, value: u32, step: u32) -> bool {
        match base {
            CronField::All => value % step == 0,
            CronField::Range(start, end) => {
                if value >= *start && value <= *end {
                    (value - start) % step == 0
                } else {
                    false
                }
            }
            CronField::Specific(values) => {
                // 对于特定值列表，检查是否有值匹配步长模式
                values
                    .iter()
                    .any(|&v| v >= value && (v - value) % step == 0)
            }
            CronField::Step { .. } => false, // 不支持嵌套步长
        }
    }

    /// 获取原始表达式
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// 获取下次执行时间
    pub fn next_run_after(&self, after: NaiveDateTime) -> NaiveDateTime {
        let mut current = after + chrono::Duration::seconds(60); // 至少加1分钟

        // 最多尝试 365 天
        for _ in 0..365 * 24 * 60 {
            if self.should_run(&current) {
                return current;
            }
            current = current + chrono::Duration::minutes(1);
        }

        // 如果找不到，返回一年后
        after + chrono::Duration::days(365)
    }
}

impl std::fmt::Display for CronExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}

/// 常用 cron 表达式
impl CronExpression {
    /// 每分钟
    pub fn every_minute() -> Self {
        Self::new("* * * * *").unwrap()
    }

    /// 每小时
    pub fn every_hour() -> Self {
        Self::new("0 * * * *").unwrap()
    }

    /// 每天 9:30
    pub fn daily_9_30() -> Self {
        Self::new("30 9 * * *").unwrap()
    }

    /// 每天 15:00
    pub fn daily_15_00() -> Self {
        Self::new("0 15 * * *").unwrap()
    }

    /// 工作日 9:30
    pub fn weekday_morning() -> Self {
        Self::new("30 9 * * 1-5").unwrap()
    }

    /// 每周一早上
    pub fn weekly_monday_morning() -> Self {
        Self::new("0 9 * * 1").unwrap()
    }

    /// 每月1号
    pub fn monthly_first_day() -> Self {
        Self::new("0 0 1 * *").unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_every_minute() {
        let cron = CronExpression::every_minute();
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 30, 45).unwrap(),
        );
        assert!(cron.should_run(&dt));
    }

    #[test]
    fn test_cron_daily_9_30() {
        let cron = CronExpression::daily_9_30();

        let should_match = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
        );
        assert!(cron.should_run(&should_match));

        let should_not_match = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(9, 31, 0).unwrap(),
        );
        assert!(!cron.should_run(&should_not_match));
    }

    #[test]
    fn test_cron_weekday() {
        let cron = CronExpression::weekday_morning();

        // 周一 9:30
        let monday = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), // 2024-01-01 是周一
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
        );
        assert!(cron.should_run(&monday));

        // 周六 9:30
        let saturday = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(), // 2024-01-06 是周六
            NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
        );
        assert!(!cron.should_run(&saturday));
    }

    #[test]
    fn test_cron_range() {
        let cron = CronExpression::new("0 9-17 * * *").unwrap(); // 工作时间每小时

        let within_range = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
        );
        assert!(cron.should_run(&within_range));

        let outside_range = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        );
        assert!(!cron.should_run(&outside_range));
    }

    #[test]
    fn test_cron_step() {
        let cron = CronExpression::new("*/5 * * * *").unwrap(); // 每5分钟

        let should_match = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 10, 0).unwrap(),
        );
        assert!(cron.should_run(&should_match));

        let should_not_match = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 13, 0).unwrap(),
        );
        assert!(!cron.should_run(&should_not_match));
    }

    #[test]
    fn test_next_run_after_finds_next_matching_minute() {
        let cron = CronExpression::new("*/5 * * * *").unwrap();
        let after = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveTime::from_hms_opt(12, 13, 0).unwrap(),
        );

        let next = cron.next_run_after(after);

        assert_eq!(
            next,
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(12, 15, 0).unwrap(),
            )
        );
    }
}
