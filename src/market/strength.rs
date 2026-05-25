mod foundation;
mod report;

pub use foundation::{
    AShareIndustryRow, MarketAnalysisFoundation, MarketFoundationSummary,
    MarketIndustryClassificationRow, MarketSnapshotRow, SectorCoverageRow,
    build_market_analysis_foundation,
};
pub use report::{MarketStrengthReport, StrongSectorStockRow};

pub(crate) use report::{
    FundamentalSnapshot, FundamentalSnapshotBatch, build_market_strength_report,
};

#[cfg(test)]
use report::{
    compare_board_rows_asc, compare_board_rows_desc, compare_market_cap_desc, compare_profit_desc,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    use crate::market::{BoardRankRow, BoardType};

    fn sample_stock(
        code: &str,
        name: &str,
        price: f64,
        change_pct: f64,
        amount: f64,
    ) -> MarketSnapshotRow {
        MarketSnapshotRow {
            code: code.to_string(),
            name: name.to_string(),
            price,
            change_pct,
            volume: 1000.0,
            amount,
        }
    }

    fn sample_industry(code: &str, industry_name: &str) -> MarketIndustryClassificationRow {
        MarketIndustryClassificationRow {
            code: code.to_string(),
            industry_name: industry_name.to_string(),
        }
    }

    #[test]
    fn foundation_builds_coverage_summary() {
        let foundation = build_market_analysis_foundation(
            vec![
                sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0),
                sample_stock("601398", "工商银行", 7.0, 1.5, 800.0),
                sample_stock("300024", "机器人", 15.0, 4.2, 1200.0),
            ],
            vec![
                sample_industry("600000", "银行"),
                sample_industry("601398", "银行"),
                sample_industry("300024", "机械设备"),
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
    fn foundation_requires_existing_industry_data() {
        let err = build_market_analysis_foundation(
            vec![sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0)],
            Vec::new(),
        )
        .unwrap_err();

        assert!(err.to_string().contains("risk sync industry"));
    }

    #[test]
    fn ranking_prefers_larger_metric_values() {
        let mut rows = vec![
            StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "601398".to_string(),
                name: "工商银行".to_string(),
                latest_price: 7.0,
                latest_change_pct: 1.0,
                market_cap: Some(Decimal::new(300000, 2)),
                latest_report_profit: Some(Decimal::new(10000, 2)),
            },
            StrongSectorStockRow {
                sector_name: "银行".to_string(),
                code: "600000".to_string(),
                name: "浦发银行".to_string(),
                latest_price: 10.0,
                latest_change_pct: 2.0,
                market_cap: Some(Decimal::new(500000, 2)),
                latest_report_profit: Some(Decimal::new(8000, 2)),
            },
        ];

        rows.sort_by(compare_market_cap_desc);
        assert_eq!(rows[0].code, "600000");

        rows.sort_by(compare_profit_desc);
        assert_eq!(rows[0].code, "601398");
    }

    #[test]
    fn board_ordering_supports_strong_and_weak_views() {
        let mut rows = vec![
            BoardRankRow::new("BK001", "银行", BoardType::Sector, 1, 2.5),
            BoardRankRow::new("BK002", "有色金属", BoardType::Sector, 2, -1.8),
            BoardRankRow::new("BK003", "计算机", BoardType::Sector, 3, 4.1),
        ];

        rows.sort_by(compare_board_rows_desc);
        assert_eq!(rows[0].board_name, "计算机");

        rows.sort_by(compare_board_rows_asc);
        assert_eq!(rows[0].board_name, "有色金属");
    }

    #[test]
    fn strength_report_builds_strong_weak_and_ranked_stock_views() {
        let foundation = build_market_analysis_foundation(
            vec![
                sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0),
                sample_stock("601398", "工商银行", 7.0, 1.5, 800.0),
                sample_stock("300024", "机器人", 15.0, 4.2, 1200.0),
                sample_stock("000960", "锡业股份", 14.0, -1.0, 900.0),
            ],
            vec![
                sample_industry("600000", "银行"),
                sample_industry("601398", "银行"),
                sample_industry("300024", "计算机"),
                sample_industry("000960", "有色金属"),
            ],
        )
        .unwrap();

        let report = build_market_strength_report(
            foundation,
            vec![
                BoardRankRow::new("BK001", "计算机", BoardType::Sector, 1, 4.1),
                BoardRankRow::new("BK002", "银行", BoardType::Sector, 2, 2.5),
                BoardRankRow::new("BK003", "有色金属", BoardType::Sector, 3, -1.8),
            ],
            FundamentalSnapshotBatch {
                snapshots: vec![
                    FundamentalSnapshot {
                        code: "600000".to_string(),
                        market_cap: Some(Decimal::new(500000, 2)),
                        latest_report_profit: Some(Decimal::new(8000, 2)),
                    },
                    FundamentalSnapshot {
                        code: "601398".to_string(),
                        market_cap: Some(Decimal::new(700000, 2)),
                        latest_report_profit: Some(Decimal::new(10000, 2)),
                    },
                    FundamentalSnapshot {
                        code: "300024".to_string(),
                        market_cap: Some(Decimal::new(200000, 2)),
                        latest_report_profit: Some(Decimal::new(3000, 2)),
                    },
                ],
                valuation_error_count: 0,
                earnings_error_count: 0,
            },
            2,
            1,
            2,
        );

        assert_eq!(report.strong_sectors.len(), 2);
        assert_eq!(report.strong_sectors[0].board_name, "计算机");
        assert_eq!(report.weak_sectors.len(), 1);
        assert_eq!(report.weak_sectors[0].board_name, "有色金属");
        assert_eq!(report.candidate_stock_count, 3);
        assert_eq!(report.market_cap_coverage_count, 3);
        assert_eq!(report.profit_coverage_count, 3);
        assert_eq!(report.top_by_market_cap.len(), 2);
        assert_eq!(report.top_by_market_cap[0].code, "601398");
        assert_eq!(report.top_by_profit.len(), 2);
        assert_eq!(report.top_by_profit[0].code, "601398");
    }

    #[test]
    fn strength_report_falls_back_to_industry_derived_rankings_when_sector_names_do_not_match() {
        let foundation = build_market_analysis_foundation(
            vec![
                sample_stock("600000", "浦发银行", 10.0, 2.1, 1000.0),
                sample_stock("601398", "工商银行", 7.0, 1.5, 800.0),
                sample_stock("300024", "机器人", 15.0, 4.2, 1200.0),
                sample_stock("000960", "锡业股份", 14.0, -1.0, 900.0),
            ],
            vec![
                sample_industry("600000", "银行"),
                sample_industry("601398", "银行"),
                sample_industry("300024", "计算机"),
                sample_industry("000960", "有色金属"),
            ],
        )
        .unwrap();

        let report = build_market_strength_report(
            foundation,
            vec![
                BoardRankRow::new("BK0002", "白酒", BoardType::Sector, 1, 1.9),
                BoardRankRow::new("BK0003", "保险", BoardType::Sector, 2, 1.5),
            ],
            FundamentalSnapshotBatch {
                snapshots: vec![
                    FundamentalSnapshot {
                        code: "600000".to_string(),
                        market_cap: Some(Decimal::new(500000, 2)),
                        latest_report_profit: Some(Decimal::new(8000, 2)),
                    },
                    FundamentalSnapshot {
                        code: "601398".to_string(),
                        market_cap: Some(Decimal::new(700000, 2)),
                        latest_report_profit: Some(Decimal::new(10000, 2)),
                    },
                    FundamentalSnapshot {
                        code: "300024".to_string(),
                        market_cap: Some(Decimal::new(200000, 2)),
                        latest_report_profit: Some(Decimal::new(3000, 2)),
                    },
                ],
                valuation_error_count: 0,
                earnings_error_count: 0,
            },
            2,
            1,
            2,
        );

        assert_eq!(report.strong_sectors.len(), 2);
        assert_eq!(report.strong_sectors[0].board_name, "计算机");
        assert_eq!(report.strong_sectors[1].board_name, "银行");
        assert_eq!(report.weak_sectors[0].board_name, "有色金属");
        assert_eq!(report.candidate_stock_count, 3);
        assert_eq!(report.market_cap_coverage_count, 3);
        assert_eq!(report.profit_coverage_count, 3);
        assert_eq!(report.top_by_market_cap[0].code, "601398");
        assert_eq!(report.top_by_profit[0].code, "601398");
    }
}
