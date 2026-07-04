use crate::market::service::{
    market_sentiment_daily_to_snapshot, north_flow_daily_to_snapshot, sector_daily_to_board_rank,
    sector_daily_to_leader,
};
use crate::market::{BoardType, LeaderFilter};

use super::models::market_table_sqls;
use super::*;

#[test]
fn test_stock_info_ch_derive() {
    let info = StockInfoCH {
        code: "000001".to_string(),
        name: "平安银行".to_string(),
        market: 0,
        list_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        status: "active".to_string(),
        updated_at: chrono::Utc::now().naive_utc(),
    };
    assert_eq!(info.code, "000001");
}

#[test]
fn test_market_table_sqls_include_phase23_tables() {
    let sql = market_table_sqls()
        .into_iter()
        .map(|(_, sql)| sql)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(sql.contains("CREATE TABLE IF NOT EXISTS sector_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS north_flow_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS market_sentiment_daily"));
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS market_fundamentals_daily"));
}

#[test]
fn test_market_sector_row_maps_to_board_rank_and_leader() {
    let row = SectorDailyCH {
        sector_code: "BK001".to_string(),
        sector_name: "银行".to_string(),
        sector_type: "industry".to_string(),
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        change_pct: 2.35,
        rank: 1,
        leader_code: Some("600000".to_string()),
        leader_name: Some("浦发银行".to_string()),
        leader_change: Some(5.61),
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let board = sector_daily_to_board_rank(row.clone()).unwrap();
    let leader = sector_daily_to_leader(row, LeaderFilter::Sector("银行".to_string()))
        .unwrap()
        .unwrap();

    assert_eq!(board.board_type, BoardType::Sector);
    assert_eq!(board.board_name, "银行");
    assert_eq!(board.rank, 1);
    assert_eq!(leader.code, "600000");
    assert_eq!(leader.sector_name.as_deref(), Some("银行"));
    assert_eq!(leader.concept_name, None);
}

#[test]
fn test_market_north_flow_row_maps_to_snapshot() {
    let row = NorthFlowDailyCH {
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        sh_amount: 12.3,
        sz_amount: 8.6,
        total_amount: 20.9,
        balance: 99.1,
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let snapshot = north_flow_daily_to_snapshot(row);

    assert_eq!(
        snapshot.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()
    );
    assert_eq!(snapshot.total_amount, 20.9);
    assert_eq!(snapshot.balance, 99.1);
}

#[test]
fn test_market_sentiment_row_maps_to_snapshot() {
    let row = MarketSentimentDailyCH {
        trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
        up_count: 3210,
        down_count: 1875,
        limit_up_count: 87,
        limit_down_count: 4,
        seal_rate: 0.81,
        break_rate: 0.19,
        consecutive_board_count: 23,
        updated_at: "2026-03-10 15:00:00".to_string(),
    };

    let snapshot = market_sentiment_daily_to_snapshot(row);

    assert_eq!(snapshot.limit_up_count, 87);
    assert_eq!(snapshot.consecutive_board_count, 23);
    assert_eq!(snapshot.seal_rate, 0.81);
}

#[test]
fn test_market_sector_row_deserializes_json_each_row_payload() {
    let row: SectorDailyCH = serde_json::from_str(
        r#"{"sector_code":"BK0001","sector_name":"银行","sector_type":"industry","trade_date":"2026-03-14","change_pct":2.35,"rank":1,"leader_code":null,"leader_name":null,"leader_change":null,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(row.sector_code, "BK0001");
    assert_eq!(row.sector_name, "银行");
    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
    assert!(row.leader_code.is_none());
}

#[test]
fn test_market_north_flow_row_deserializes_json_each_row_payload() {
    let row: NorthFlowDailyCH = serde_json::from_str(
        r#"{"trade_date":"2026-03-14","sh_amount":50.5,"sz_amount":35.2,"total_amount":85.7,"balance":12500.0,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.total_amount, 85.7);
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
}

#[test]
fn test_market_sentiment_row_deserializes_json_each_row_payload() {
    let row: MarketSentimentDailyCH = serde_json::from_str(
        r#"{"trade_date":"2026-03-14","up_count":2800,"down_count":2100,"limit_up_count":45,"limit_down_count":12,"seal_rate":0.78,"break_rate":0.15,"consecutive_board_count":120,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(
        row.trade_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.limit_up_count, 45);
    assert_eq!(row.updated_at, "2026-03-14 15:12:16");
}

#[test]
fn test_market_fundamental_snapshot_row_deserializes_json_each_row_payload() {
    let row: MarketFundamentalSnapshotCH = serde_json::from_str(
        r#"{"code":"600519","snapshot_date":"2026-03-14","market_cap":23000.5,"latest_report_profit":862.1,"profit_source":"report","pe_dynamic":27.4,"updated_at":"2026-03-14 15:12:16"}"#,
    )
    .unwrap();

    assert_eq!(row.code, "600519");
    assert_eq!(
        row.snapshot_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap()
    );
    assert_eq!(row.market_cap, Some(23000.5));
    assert_eq!(row.latest_report_profit, Some(862.1));
    assert_eq!(row.profit_source, "report");
    assert_eq!(row.pe_dynamic, Some(27.4));
}

#[test]
fn models_minute_kline_ch_has_expected_fields() {
    use crate::db::clickhouse::models::MinuteKlineCH;
    use chrono::{TimeZone, Utc};

    let row = MinuteKlineCH {
        timestamp: Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        period: "1m".into(),
        adjust: "none".into(),
        open: 12.34,
        high: 12.50,
        low: 12.20,
        close: 12.40,
        volume: 123456.0,
        amount: 1_530_000.0,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.period, "1m");
    assert_eq!(row.adjust, "none");
    assert_eq!(row.volume, 123456.0);
}

#[test]
fn models_minute_share_ch_has_expected_fields() {
    use crate::db::clickhouse::models::MinuteShareCH;
    use chrono::{TimeZone, Utc};

    let row = MinuteShareCH {
        timestamp: Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2026, 7, 4)
                .unwrap()
                .and_hms_opt(9, 30, 0)
                .unwrap(),
        ),
        code: "sh600000".into(),
        price: 12.34,
        volume: 1000.0,
        amount: 12340.0,
        avg_price: 12.34,
    };
    assert_eq!(row.code, "sh600000");
    assert_eq!(row.price, 12.34);
    assert_eq!(row.avg_price, 12.34);
}
