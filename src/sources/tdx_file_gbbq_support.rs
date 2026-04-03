use std::collections::HashMap;

use crate::sources::tdx_file::TdxGbbqRecord;

pub(super) fn adjusted_preclose(record: &TdxGbbqRecord, preclose: f64) -> f64 {
    (preclose * 10.0 - record.fh_qltp as f64
        + record.pg_hzgb as f64 * record.pgj_qzgb as f64)
        / (10.0 + record.pg_hzgb as f64 + record.sg_hltp as f64)
}

pub(super) fn compute_pre_pct(
    record: &TdxGbbqRecord,
    close: f32,
    mut preclose: f64,
    flag: bool,
) -> [f64; 3] {
    if flag {
        // 除权计算公式: (preclose * 10 - 分红 + 配股 * 配股价) / (10 + 配股 + 送股)
        preclose = adjusted_preclose(record, preclose);
    }
    let close = close as f64;
    [preclose, close, close / preclose]
}

pub(super) fn filter_a_stock_dividend(records: &[TdxGbbqRecord]) -> Vec<TdxGbbqRecord> {
    records
        .iter()
        .filter(|record| is_a_stock_dividend_record(record))
        .cloned()
        .collect()
}

pub(super) fn is_a_stock_dividend_record(record: &TdxGbbqRecord) -> bool {
    let first_char = record.code.chars().next();
    let is_a_stock = matches!(first_char, Some('6') | Some('0') | Some('3'));
    is_a_stock && record.category == 1
}

pub(super) fn group_by_code(records: Vec<TdxGbbqRecord>) -> HashMap<String, Vec<TdxGbbqRecord>> {
    let mut grouped: HashMap<String, Vec<TdxGbbqRecord>> = HashMap::new();
    for record in records {
        let code = record.code.clone();
        grouped.entry(code).or_default().push(record);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GbbqRecordFixtureConfig<'a> {
        code: &'a str,
        category: u8,
        market: u8,
        date: u32,
        fh_qltp: f32,
        pgj_qzgb: f32,
        sg_hltp: f32,
        pg_hzgb: f32,
    }

    impl Default for GbbqRecordFixtureConfig<'_> {
        fn default() -> Self {
            Self {
                code: "000001",
                category: 1,
                market: 0,
                date: 20240101,
                fh_qltp: 1.0,
                pgj_qzgb: 2.0,
                sg_hltp: 1.0,
                pg_hzgb: 1.0,
            }
        }
    }

    fn build_record(config: &GbbqRecordFixtureConfig<'_>) -> TdxGbbqRecord {
        TdxGbbqRecord {
            market: config.market,
            code: config.code.to_string(),
            date: config.date,
            category: config.category,
            fh_qltp: config.fh_qltp,
            pgj_qzgb: config.pgj_qzgb,
            sg_hltp: config.sg_hltp,
            pg_hzgb: config.pg_hzgb,
        }
    }

    #[test]
    fn is_a_stock_dividend_record_matches_a_share_dividend_entries() {
        assert!(is_a_stock_dividend_record(&build_record(
            &GbbqRecordFixtureConfig::default()
        )));
        assert!(!is_a_stock_dividend_record(&build_record(
            &GbbqRecordFixtureConfig {
                code: "200001",
                ..Default::default()
            }
        )));
        assert!(!is_a_stock_dividend_record(&build_record(
            &GbbqRecordFixtureConfig {
                category: 2,
                ..Default::default()
            }
        )));
    }

    #[test]
    fn adjusted_preclose_applies_dividend_formula() {
        let config = GbbqRecordFixtureConfig::default();
        let record = build_record(&config);
        let preclose = 12.0;
        let expected = (preclose * 10.0 - config.fh_qltp as f64
            + config.pg_hzgb as f64 * config.pgj_qzgb as f64)
            / (10.0 + config.pg_hzgb as f64 + config.sg_hltp as f64);

        let adjusted_preclose = adjusted_preclose(&record, preclose);

        assert!((adjusted_preclose - expected).abs() < 1e-9);
    }

    #[test]
    fn compute_pre_pct_applies_adjustment_formula_on_xdxr_day() {
        let config = GbbqRecordFixtureConfig::default();
        let record = build_record(&config);
        let close = 11.0;
        let preclose = 12.0;
        let expected_preclose = (preclose * 10.0 - config.fh_qltp as f64
            + config.pg_hzgb as f64 * config.pgj_qzgb as f64)
            / (10.0 + config.pg_hzgb as f64 + config.sg_hltp as f64);
        let [adjusted_preclose, adjusted_close, pct] =
            compute_pre_pct(&record, close, preclose, true);

        assert!((adjusted_preclose - expected_preclose).abs() < 1e-9);
        assert_eq!(adjusted_close, close as f64);
        assert!((pct - (close as f64 / expected_preclose)).abs() < 1e-9);
    }

    #[test]
    fn filter_a_stock_dividend_keeps_only_a_share_dividend_records() {
        let records = vec![
            build_record(&GbbqRecordFixtureConfig {
                code: "600000",
                ..Default::default()
            }),
            build_record(&GbbqRecordFixtureConfig::default()),
            build_record(&GbbqRecordFixtureConfig {
                code: "300001",
                category: 2,
                ..Default::default()
            }),
            build_record(&GbbqRecordFixtureConfig {
                code: "200001",
                ..Default::default()
            }),
        ];

        let filtered = filter_a_stock_dividend(&records);
        let codes: Vec<_> = filtered.iter().map(|record| record.code.as_str()).collect();

        assert_eq!(codes, vec!["600000", "000001"]);
    }

    #[test]
    fn group_by_code_collects_records_under_the_same_stock_code() {
        let grouped = group_by_code(vec![
            build_record(&GbbqRecordFixtureConfig::default()),
            build_record(&GbbqRecordFixtureConfig {
                code: "600000",
                ..Default::default()
            }),
            build_record(&GbbqRecordFixtureConfig {
                category: 2,
                ..Default::default()
            }),
        ]);

        assert_eq!(grouped["000001"].len(), 2);
        assert_eq!(grouped["600000"].len(), 1);
    }
}
