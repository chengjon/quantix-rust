use std::collections::HashMap;

use crate::core::Result;
use crate::sources::tdx_file::{FuquanCalculator, TdxDayData, TdxDayFile, TdxGbbqRecord};

pub(super) fn import_stock_day(
    data_dir: &str,
    code: &str,
    gbbqs: Option<&[TdxGbbqRecord]>,
) -> Result<Vec<TdxDayData>> {
    let code_num = code.parse::<u32>().map_err(|_| {
        crate::core::QuantixError::DataParse(format!("无效的股票代码: {}", code))
    })?;

    let day_path = format!("{}/{}.day", data_dir, code);
    let records = TdxDayFile::from_file(code_num, &day_path)?;
    let factors = FuquanCalculator::calculate(&records, gbbqs)?;

    Ok(records
        .iter()
        .zip(factors.iter())
        .map(|(record, factor)| TdxDayData::from_record(record, factor))
        .collect())
}

pub(super) fn import_batch(
    data_dir: &str,
    codes: &[String],
    gbbq_map: &HashMap<String, Vec<TdxGbbqRecord>>,
) -> Result<HashMap<String, Vec<TdxDayData>>> {
    let mut result = HashMap::new();

    for code in codes {
        let gbbqs = gbbq_map.get(code).map(|records| records.as_slice());
        match import_stock_day(data_dir, code, gbbqs) {
            Ok(data) => {
                if !data.is_empty() {
                    result.insert(code.clone(), data);
                }
            }
            Err(error) => {
                tracing::warn!("导入 {} 失败: {}", code, error);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use tempfile::tempdir;

    use super::*;

    #[derive(Clone, Copy)]
    struct DayRowFixtureConfig {
        date: u32,
        open: u32,
        high: u32,
        low: u32,
        close: u32,
        amount: f32,
        volume: u32,
    }

    impl Default for DayRowFixtureConfig {
        fn default() -> Self {
            Self {
                date: 20210801,
                open: 1000,
                high: 1050,
                low: 990,
                close: 1040,
                amount: 1_000_000.0,
                volume: 10_000,
            }
        }
    }

    struct DayFileFixtureConfig<'a> {
        code: &'a str,
        rows: Vec<DayRowFixtureConfig>,
    }

    impl Default for DayFileFixtureConfig<'static> {
        fn default() -> Self {
            Self {
                code: "600000",
                rows: vec![DayRowFixtureConfig::default()],
            }
        }
    }

    fn close_decimal(close: u32) -> Decimal {
        Decimal::from_f64(close as f64 / 100.0)
            .unwrap()
            .round_dp(2)
    }

    fn factor_decimal(rows: &[DayRowFixtureConfig], target_index: usize) -> Decimal {
        let mut factor = 1.0;
        let mut preclose = rows[0].close as f64 / 100.0;

        for row in rows.iter().take(target_index + 1) {
            let close = row.close as f64 / 100.0;
            factor *= close / preclose;
            preclose = close;
        }

        Decimal::from_f64(factor).unwrap().round_dp(6)
    }

    fn write_day_file(dir: &Path, fixture: &DayFileFixtureConfig<'_>) {
        let mut bytes = Vec::with_capacity(fixture.rows.len() * 32);
        for row in &fixture.rows {
            bytes.extend_from_slice(&row.date.to_le_bytes());
            bytes.extend_from_slice(&row.open.to_le_bytes());
            bytes.extend_from_slice(&row.high.to_le_bytes());
            bytes.extend_from_slice(&row.low.to_le_bytes());
            bytes.extend_from_slice(&row.close.to_le_bytes());
            bytes.extend_from_slice(&row.amount.to_le_bytes());
            bytes.extend_from_slice(&row.volume.to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        fs::write(dir.join(format!("{}.day", fixture.code)), bytes).unwrap();
    }

    #[test]
    fn import_stock_day_reads_day_file_and_builds_day_data() {
        let dir = tempdir().unwrap();
        let first_row = DayRowFixtureConfig::default();
        let second_row = DayRowFixtureConfig {
            date: 20210802,
            open: 1045,
            high: 1080,
            low: 1030,
            close: 1070,
            ..DayRowFixtureConfig::default()
        };
        let file_fixture = DayFileFixtureConfig {
            rows: vec![first_row, second_row],
            ..DayFileFixtureConfig::default()
        };
        write_day_file(dir.path(), &file_fixture);

        let imported =
            import_stock_day(dir.path().to_str().unwrap(), file_fixture.code, None).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].code, file_fixture.code);
        assert_eq!(imported[0].close, close_decimal(first_row.close));
        assert_eq!(imported[1].close, close_decimal(second_row.close));
        assert_eq!(imported[1].factor, factor_decimal(&file_fixture.rows, 1));
    }

    #[test]
    fn import_batch_skips_codes_that_fail_to_import() {
        let dir = tempdir().unwrap();
        let primary_fixture = DayFileFixtureConfig::default();
        let missing_code = "000001".to_string();
        write_day_file(dir.path(), &primary_fixture);

        let imported = import_batch(
            dir.path().to_str().unwrap(),
            &[primary_fixture.code.to_string(), missing_code.clone()],
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(imported.len(), 1);
        assert!(imported.contains_key(primary_fixture.code));
        assert!(!imported.contains_key(&missing_code));
    }
}
