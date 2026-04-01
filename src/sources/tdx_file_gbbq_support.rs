use std::collections::HashMap;

use crate::sources::tdx_file::TdxGbbqRecord;

pub(super) fn compute_pre_pct(
    record: &TdxGbbqRecord,
    close: f32,
    mut preclose: f64,
    flag: bool,
) -> [f64; 3] {
    if flag {
        // 除权计算公式: (preclose * 10 - 分红 + 配股 * 配股价) / (10 + 配股 + 送股)
        preclose = (preclose * 10.0 - record.fh_qltp as f64
            + record.pg_hzgb as f64 * record.pgj_qzgb as f64)
            / (10.0 + record.pg_hzgb as f64 + record.sg_hltp as f64);
    }
    let close = close as f64;
    [preclose, close, close / preclose]
}

pub(super) fn filter_a_stock_dividend(records: &[TdxGbbqRecord]) -> Vec<TdxGbbqRecord> {
    records
        .iter()
        .filter(|record| {
            let first_char = record.code.chars().next();
            let is_a_stock = matches!(first_char, Some('6') | Some('0') | Some('3'));
            is_a_stock && record.category == 1
        })
        .cloned()
        .collect()
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

    fn sample_record(code: &str, category: u8) -> TdxGbbqRecord {
        TdxGbbqRecord {
            market: 0,
            code: code.to_string(),
            date: 20240101,
            category,
            fh_qltp: 1.0,
            pgj_qzgb: 2.0,
            sg_hltp: 1.0,
            pg_hzgb: 1.0,
        }
    }

    #[test]
    fn compute_pre_pct_applies_adjustment_formula_on_xdxr_day() {
        let record = sample_record("000001", 1);
        let [adjusted_preclose, close, pct] = compute_pre_pct(&record, 11.0, 12.0, true);

        assert!((adjusted_preclose - (121.0 / 12.0)).abs() < 1e-9);
        assert_eq!(close, 11.0);
        assert!((pct - (11.0 / (121.0 / 12.0))).abs() < 1e-9);
    }

    #[test]
    fn filter_a_stock_dividend_keeps_only_a_share_dividend_records() {
        let records = vec![
            sample_record("600000", 1),
            sample_record("000001", 1),
            sample_record("300001", 2),
            sample_record("200001", 1),
        ];

        let filtered = filter_a_stock_dividend(&records);
        let codes: Vec<_> = filtered.iter().map(|record| record.code.as_str()).collect();

        assert_eq!(codes, vec!["600000", "000001"]);
    }

    #[test]
    fn group_by_code_collects_records_under_the_same_stock_code() {
        let grouped = group_by_code(vec![
            sample_record("000001", 1),
            sample_record("600000", 1),
            sample_record("000001", 2),
        ]);

        assert_eq!(grouped["000001"].len(), 2);
        assert_eq!(grouped["600000"].len(), 1);
    }
}
