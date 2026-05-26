use quantix_cli::market::{
    MarketIndustryClassificationRow, MarketSnapshotRow, build_market_analysis_foundation,
};

fn stock(code: &str, name: &str, change_pct: f64) -> MarketSnapshotRow {
    MarketSnapshotRow {
        code: code.to_string(),
        name: name.to_string(),
        price: 10.0,
        change_pct,
        volume: 1000.0,
        amount: 10000.0,
    }
}

fn industry(code: &str, industry_name: &str) -> MarketIndustryClassificationRow {
    MarketIndustryClassificationRow {
        code: code.to_string(),
        industry_name: industry_name.to_string(),
    }
}

#[test]
fn market_foundation_builds_from_deterministic_rows_without_external_adapters() {
    let foundation = build_market_analysis_foundation(
        vec![
            stock("600000", "浦发银行", 2.1),
            stock("601398", "工商银行", 1.5),
            stock("300024", "机器人", 4.2),
        ],
        vec![
            industry("600000", "银行"),
            industry("601398", "银行"),
            industry("300024", "机械设备"),
        ],
    )
    .unwrap();

    assert_eq!(foundation.summary.total_stocks, 3);
    assert_eq!(foundation.summary.classified_stocks, 3);
    assert_eq!(foundation.summary.unclassified_stocks, 0);
    assert_eq!(foundation.summary.sector_count, 2);
    assert_eq!(foundation.summary.top_sectors[0].industry_name, "银行");
    assert_eq!(foundation.summary.top_sectors[0].stock_count, 2);
}

#[test]
fn market_foundation_requires_industry_rows_from_the_runtime_layer() {
    let err = build_market_analysis_foundation(vec![stock("600000", "浦发银行", 2.1)], Vec::new())
        .unwrap_err();

    assert!(err.to_string().contains("risk sync industry"));
}
