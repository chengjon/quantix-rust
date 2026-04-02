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

    fn write_day_file(
        dir: &Path,
        code: &str,
        rows: &[(u32, u32, u32, u32, u32, f32, u32)],
    ) {
        let mut bytes = Vec::with_capacity(rows.len() * 32);
        for (date, open, high, low, close, amount, volume) in rows {
            bytes.extend_from_slice(&date.to_le_bytes());
            bytes.extend_from_slice(&open.to_le_bytes());
            bytes.extend_from_slice(&high.to_le_bytes());
            bytes.extend_from_slice(&low.to_le_bytes());
            bytes.extend_from_slice(&close.to_le_bytes());
            bytes.extend_from_slice(&amount.to_le_bytes());
            bytes.extend_from_slice(&volume.to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }

        fs::write(dir.join(format!("{code}.day")), bytes).unwrap();
    }

    #[test]
    fn import_stock_day_reads_day_file_and_builds_day_data() {
        let dir = tempdir().unwrap();
        write_day_file(
            dir.path(),
            "600000",
            &[
                (20210801, 1000, 1050, 990, 1040, 1000000.0, 10000),
                (20210802, 1045, 1080, 1030, 1070, 1000000.0, 10000),
            ],
        );

        let imported = import_stock_day(dir.path().to_str().unwrap(), "600000", None).unwrap();

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].code, "600000");
        assert_eq!(imported[0].close, Decimal::from_f64(10.4).unwrap());
        assert_eq!(imported[1].close, Decimal::from_f64(10.7).unwrap());
        assert!(imported[1].factor > Decimal::from_f64(1.02).unwrap());
    }

    #[test]
    fn import_batch_skips_codes_that_fail_to_import() {
        let dir = tempdir().unwrap();
        write_day_file(
            dir.path(),
            "600000",
            &[(20210801, 1000, 1050, 990, 1040, 1000000.0, 10000)],
        );

        let imported = import_batch(
            dir.path().to_str().unwrap(),
            &["600000".to_string(), "000001".to_string()],
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(imported.len(), 1);
        assert!(imported.contains_key("600000"));
        assert!(!imported.contains_key("000001"));
    }
}
