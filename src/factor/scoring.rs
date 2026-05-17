use polars::prelude::{
    AnyValue, CsvWriter, DataFrame, NamedFrom, ParquetWriter, SerWriter, Series,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use crate::core::{QuantixError, Result};
use crate::factor::catalog::FactorCatalog;
use crate::factor::dataset::FactorDataset;

#[derive(Debug, Clone)]
pub struct FactorScoreResult {
    pub factors: Vec<String>,
    pub frame: DataFrame,
}

pub fn score_factors_latest(
    catalog: &FactorCatalog,
    dataset: &FactorDataset,
    factors: &[String],
    top: Option<usize>,
) -> Result<FactorScoreResult> {
    if factors.is_empty() {
        return Err(QuantixError::Config(
            "factor score requires at least one --factor".to_string(),
        ));
    }

    let latest_date = latest_dataset_date(dataset)?;
    let mut scores_by_symbol: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for factor in factors {
        let result = catalog.compute(factor, dataset)?;
        let values = latest_factor_values(&result.frame, &latest_date)?;
        for (symbol, score) in percentile_scores(values) {
            scores_by_symbol.entry(symbol).or_default().push(score);
        }
    }

    let mut rows = scores_by_symbol
        .into_iter()
        .filter_map(|(symbol, scores)| {
            if scores.is_empty() {
                return None;
            }
            let factor_count = scores.len() as i64;
            let score = scores.iter().sum::<f64>() / factor_count as f64;
            Some((symbol, score, factor_count))
        })
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return Err(QuantixError::DataParse(
            "factor score produced no valid latest-date rows".to_string(),
        ));
    }

    rows.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.cmp(&right.0))
    });
    if let Some(limit) = top {
        rows.truncate(limit);
    }

    let dates = vec![latest_date; rows.len()];
    let symbols = rows
        .iter()
        .map(|(symbol, _, _)| symbol.clone())
        .collect::<Vec<_>>();
    let scores = rows.iter().map(|(_, score, _)| *score).collect::<Vec<_>>();
    let counts = rows
        .iter()
        .map(|(_, _, factor_count)| *factor_count)
        .collect::<Vec<_>>();
    let frame = DataFrame::new(vec![
        Series::new("date".into(), dates),
        Series::new("symbol".into(), symbols),
        Series::new("score".into(), scores),
        Series::new("factor_count".into(), counts),
    ])
    .map_err(|e| QuantixError::DataParse(format!("factor score frame build failed: {}", e)))?;

    Ok(FactorScoreResult {
        factors: factors.to_vec(),
        frame,
    })
}

pub fn factor_score_result_to_csv_string(result: &FactorScoreResult) -> Result<String> {
    let mut bytes = Vec::new();
    CsvWriter::new(&mut bytes)
        .finish(&mut result.frame.clone())
        .map_err(|e| QuantixError::Other(format!("factor score csv export failed: {}", e)))?;
    String::from_utf8(bytes).map_err(|e| {
        QuantixError::Other(format!(
            "factor score csv export produced invalid utf8: {}",
            e
        ))
    })
}

pub fn factor_score_result_to_json_string(result: &FactorScoreResult) -> Result<String> {
    let dates = result.frame.column("date").map_err(|e| {
        QuantixError::DataParse(format!("factor score date column read failed: {}", e))
    })?;
    let symbols = result.frame.column("symbol").map_err(|e| {
        QuantixError::DataParse(format!("factor score symbol column read failed: {}", e))
    })?;
    let scores = result
        .frame
        .column("score")
        .and_then(|series| series.f64())
        .map_err(|e| {
            QuantixError::DataParse(format!("factor score score column read failed: {}", e))
        })?;
    let counts = result
        .frame
        .column("factor_count")
        .and_then(|series| series.i64())
        .map_err(|e| {
            QuantixError::DataParse(format!(
                "factor score factor_count column read failed: {}",
                e
            ))
        })?;

    let rows = (0..result.frame.height())
        .map(|idx| {
            Ok(serde_json::json!({
                "date": dates.get(idx).map_err(|e| {
                    QuantixError::DataParse(format!("factor score date read failed: {}", e))
                })?.to_string(),
                "symbol": symbols.get(idx).map_err(|e| {
                    QuantixError::DataParse(format!("factor score symbol read failed: {}", e))
                })?.to_string(),
                "score": scores.get(idx),
                "factor_count": counts.get(idx),
            }))
        })
        .collect::<Result<Vec<_>>>()?;

    serde_json::to_string(&serde_json::json!({
        "factors": result.factors,
        "rows": rows,
    }))
    .map_err(|e| QuantixError::Other(format!("factor score json export failed: {}", e)))
}

pub fn factor_score_result_to_parquet_file(
    result: &FactorScoreResult,
    path: impl AsRef<Path>,
) -> Result<()> {
    let file = File::create(path.as_ref())
        .map_err(|e| QuantixError::Other(format!("factor score parquet create failed: {}", e)))?;
    let mut frame = result.frame.clone();
    ParquetWriter::new(file)
        .finish(&mut frame)
        .map_err(|e| QuantixError::Other(format!("factor score parquet export failed: {}", e)))?;
    Ok(())
}

fn latest_dataset_date(dataset: &FactorDataset) -> Result<String> {
    let dates = dataset
        .frame()
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("factor score date column missing: {}", e)))?;
    let mut latest: Option<String> = None;
    for idx in 0..dataset.frame().height() {
        let date = dates
            .get(idx)
            .map_err(|e| QuantixError::DataParse(format!("factor score date read failed: {}", e)))?
            .to_plain_string();
        if latest.as_ref().is_none_or(|current| date > *current) {
            latest = Some(date);
        }
    }
    latest.ok_or_else(|| QuantixError::DataParse("factor score dataset is empty".to_string()))
}

fn latest_factor_values(frame: &DataFrame, latest_date: &str) -> Result<Vec<(String, f64)>> {
    let dates = frame.column("date").map_err(|e| {
        QuantixError::DataParse(format!("factor score factor date column missing: {}", e))
    })?;
    let symbols = frame.column("symbol").map_err(|e| {
        QuantixError::DataParse(format!("factor score factor symbol column missing: {}", e))
    })?;
    let values = frame.column("value").map_err(|e| {
        QuantixError::DataParse(format!("factor score factor value column missing: {}", e))
    })?;

    let mut rows = Vec::new();
    for idx in 0..frame.height() {
        let date = dates
            .get(idx)
            .map_err(|e| QuantixError::DataParse(format!("factor score date read failed: {}", e)))?
            .to_plain_string();
        if date != latest_date {
            continue;
        }
        let Some(value) = factor_value_to_f64(values.get(idx).map_err(|e| {
            QuantixError::DataParse(format!("factor score factor value read failed: {}", e))
        })?) else {
            continue;
        };
        let symbol = symbols
            .get(idx)
            .map_err(|e| {
                QuantixError::DataParse(format!("factor score symbol read failed: {}", e))
            })?
            .to_plain_string();
        rows.push((symbol, value));
    }
    Ok(rows)
}

trait AnyValuePlainString {
    fn to_plain_string(self) -> String;
}

impl AnyValuePlainString for AnyValue<'_> {
    fn to_plain_string(self) -> String {
        self.get_str()
            .map(str::to_owned)
            .unwrap_or_else(|| self.to_string())
    }
}

fn factor_value_to_f64(value: AnyValue<'_>) -> Option<f64> {
    match value {
        AnyValue::Float64(value) => Some(value),
        AnyValue::Float32(value) => Some(value as f64),
        AnyValue::Int64(value) => Some(value as f64),
        AnyValue::Int32(value) => Some(value as f64),
        AnyValue::Int16(value) => Some(value as f64),
        AnyValue::Int8(value) => Some(value as f64),
        AnyValue::UInt64(value) => Some(value as f64),
        AnyValue::UInt32(value) => Some(value as f64),
        AnyValue::UInt16(value) => Some(value as f64),
        AnyValue::UInt8(value) => Some(value as f64),
        _ => None,
    }
}

fn percentile_scores(values: Vec<(String, f64)>) -> Vec<(String, f64)> {
    if values.is_empty() {
        return Vec::new();
    }
    if values.len() == 1 {
        return vec![(values[0].0.clone(), 1.0)];
    }

    let valid = values
        .into_iter()
        .filter(|(_, value)| value.is_finite())
        .collect::<Vec<_>>();
    if has_single_distinct_value(&valid) {
        return valid.into_iter().map(|(symbol, _)| (symbol, 1.0)).collect();
    }

    let denominator = (valid.len() - 1) as f64;
    valid
        .iter()
        .map(|(symbol, value)| {
            let less = valid.iter().filter(|(_, other)| other < value).count();
            let equal = valid.iter().filter(|(_, other)| other == value).count();
            let average_rank = less as f64 + (equal as f64 + 1.0) / 2.0;
            let percentile = (average_rank - 1.0) / denominator;
            (symbol.clone(), percentile)
        })
        .collect()
}

fn has_single_distinct_value(values: &[(String, f64)]) -> bool {
    values
        .first()
        .is_none_or(|(_, first)| values.iter().all(|(_, value)| value == first))
}
