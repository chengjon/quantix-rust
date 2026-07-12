//! 交易日历行为测试，从原生 `trading_calendar.rs` 内联 `mod tests` 拆出。

use super::calendar::TradingCalendar;
use super::types::TradingSession;
use chrono::NaiveDate;

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

#[tokio::test]
async fn test_load_holidays_for_year_reads_configured_year() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("holidays.json");
    tokio::fs::write(
        &config_path,
        r#"{
            "description": "test holidays",
            "source": "test",
            "years": {
                "2026": {
                    "holidays": ["2026-01-01"],
                    "early_close": [],
                    "workdays_on_weekend": ["2026-01-04"]
                }
            }
        }"#,
    )
    .await
    .unwrap();

    let mut calendar = TradingCalendar {
        holidays: std::collections::HashMap::new(),
        workdays_on_weekend: std::collections::HashMap::new(),
        config_path: Some(config_path),
    };

    calendar.load_holidays_for_year(2026).await.unwrap();

    assert!(
        calendar
            .holidays
            .get(&2026)
            .unwrap()
            .contains(&NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    );
    assert!(
        calendar
            .is_holiday(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
            .await
    );
    assert!(calendar.is_workday_on_weekend(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap()));
}
