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
