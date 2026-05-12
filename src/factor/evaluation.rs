use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::{QuantixError, Result};
use crate::factor::dataset::FactorDataset;
use crate::factor::types::FactorComputeResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorIcSummary {
    pub factor_id: String,
    pub horizon: usize,
    pub ic_mean: Option<f64>,
    pub ic_std: Option<f64>,
    pub ir: Option<f64>,
    pub observations: usize,
}

#[derive(Debug, Clone)]
pub struct FactorIcResult {
    pub summary: FactorIcSummary,
    pub by_date: DataFrame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FactorIcJsonRow {
    date: String,
    ic: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FactorIcJson {
    summary: FactorIcSummary,
    by_date: Vec<FactorIcJsonRow>,
}

pub fn evaluate_factor_ic(
    dataset: &FactorDataset,
    factor: &FactorComputeResult,
    horizon: usize,
) -> Result<FactorIcResult> {
    if horizon == 0 {
        return Err(QuantixError::DataParse(
            "factor evaluation horizon must be greater than zero".to_string(),
        ));
    }
    if factor.frame.height() != dataset.frame().height() {
        return Err(QuantixError::DataParse(format!(
            "factor `{}` row count does not match dataset row count",
            factor.factor_id
        )));
    }

    let forward_returns = dataset
        .frame()
        .clone()
        .lazy()
        .with_columns([((col("close")
            .shift(lit(-(horizon as i64)))
            .over([col("symbol")])
            / col("close"))
            - lit(1.0))
        .alias("__forward_return")])
        .collect()
        .map_err(|e| QuantixError::DataParse(format!("forward return compute failed: {}", e)))?;

    let dates = dataset
        .frame()
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("factor date column read failed: {}", e)))?;
    let factor_values = factor_values_as_f64(factor)?;
    let returns = forward_returns
        .column("__forward_return")
        .and_then(|s| s.cast(&DataType::Float64))
        .map_err(|e| QuantixError::DataParse(format!("forward return cast failed: {}", e)))?;
    let returns = returns
        .f64()
        .map_err(|e| QuantixError::DataParse(format!("forward return read failed: {}", e)))?;

    let mut by_date_pairs: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for row in 0..dataset.frame().height() {
        if let (Some(factor_value), Some(forward_return)) =
            (factor_values.get(row), returns.get(row))
        {
            let date = dates
                .get(row)
                .map_err(|e| QuantixError::DataParse(format!("factor date read failed: {}", e)))?
                .to_string();
            by_date_pairs
                .entry(date)
                .or_default()
                .push((factor_value, forward_return));
        }
    }

    let mut output_dates = Vec::with_capacity(by_date_pairs.len());
    let mut output_ics = Vec::with_capacity(by_date_pairs.len());
    for (date, pairs) in by_date_pairs {
        output_dates.push(date);
        output_ics.push(pearson_from_pairs(&pairs));
    }

    let ic_values = output_ics.iter().flatten().copied().collect::<Vec<_>>();
    let ic_mean = mean(&ic_values);
    let ic_std = sample_std(&ic_values, ic_mean);
    let ir = match (ic_mean, ic_std) {
        (Some(mean), Some(std)) if std > 0.0 => Some(mean / std),
        _ => None,
    };

    let by_date = df!(
        "date" => output_dates,
        "ic" => output_ics,
    )
    .map_err(|e| QuantixError::DataParse(format!("IC output frame build failed: {}", e)))?;

    Ok(FactorIcResult {
        summary: FactorIcSummary {
            factor_id: factor.factor_id.clone(),
            horizon,
            ic_mean,
            ic_std,
            ir,
            observations: ic_values.len(),
        },
        by_date,
    })
}

pub fn factor_value_correlation(
    left: &FactorComputeResult,
    right: &FactorComputeResult,
) -> Result<f64> {
    if left.frame.height() != right.frame.height() {
        return Err(QuantixError::DataParse(format!(
            "factor correlation row count mismatch: `{}` has {}, `{}` has {}",
            left.factor_id,
            left.frame.height(),
            right.factor_id,
            right.frame.height()
        )));
    }

    let left_values = factor_values_as_f64(left)?;
    let right_values = factor_values_as_f64(right)?;
    let mut pairs = Vec::new();
    for row in 0..left.frame.height() {
        if let (Some(left_value), Some(right_value)) = (left_values.get(row), right_values.get(row))
        {
            pairs.push((left_value, right_value));
        }
    }

    pearson_from_pairs(&pairs).ok_or_else(|| {
        QuantixError::DataParse(format!(
            "factor correlation between `{}` and `{}` has no valid observations",
            left.factor_id, right.factor_id
        ))
    })
}

pub fn factor_ic_result_to_json_string(result: &FactorIcResult) -> Result<String> {
    let dates = result
        .by_date
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("IC date column read failed: {}", e)))?;
    let ics = result
        .by_date
        .column("ic")
        .and_then(|s| s.cast(&DataType::Float64))
        .map_err(|e| QuantixError::DataParse(format!("IC value cast failed: {}", e)))?;
    let ics = ics
        .f64()
        .map_err(|e| QuantixError::DataParse(format!("IC value read failed: {}", e)))?;

    let mut rows = Vec::with_capacity(result.by_date.height());
    for row in 0..result.by_date.height() {
        rows.push(FactorIcJsonRow {
            date: dates
                .get(row)
                .map_err(|e| QuantixError::DataParse(format!("IC date read failed: {}", e)))?
                .to_string(),
            ic: ics.get(row),
        });
    }

    serde_json::to_string_pretty(&FactorIcJson {
        summary: result.summary.clone(),
        by_date: rows,
    })
    .map_err(|e| QuantixError::DataParse(format!("IC JSON export failed: {}", e)))
}

pub fn factor_ic_result_to_csv_string(result: &FactorIcResult) -> Result<String> {
    let dates = result
        .by_date
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("IC date column read failed: {}", e)))?;
    let ics = result
        .by_date
        .column("ic")
        .and_then(|s| s.cast(&DataType::Float64))
        .map_err(|e| QuantixError::DataParse(format!("IC value cast failed: {}", e)))?;
    let ics = ics
        .f64()
        .map_err(|e| QuantixError::DataParse(format!("IC value read failed: {}", e)))?;

    let mut output = String::from("date,ic\n");
    for row in 0..result.by_date.height() {
        let date = dates
            .get(row)
            .map_err(|e| QuantixError::DataParse(format!("IC date read failed: {}", e)))?
            .to_string();
        let ic = ics
            .get(row)
            .map(|value| value.to_string())
            .unwrap_or_default();
        output.push_str(&format!("{},{}\n", date, ic));
    }
    Ok(output)
}

pub fn factor_ic_result_to_parquet_file(
    result: &FactorIcResult,
    path: impl AsRef<Path>,
) -> Result<()> {
    let file = File::create(path.as_ref())
        .map_err(|e| QuantixError::Other(format!("IC parquet create failed: {}", e)))?;
    let mut frame = result.by_date.clone();
    ParquetWriter::new(file)
        .finish(&mut frame)
        .map_err(|e| QuantixError::Other(format!("IC parquet export failed: {}", e)))?;
    Ok(())
}

fn factor_values_as_f64(result: &FactorComputeResult) -> Result<Float64Chunked> {
    result
        .frame
        .column("value")
        .and_then(|s| s.cast(&DataType::Float64))
        .map_err(|e| {
            QuantixError::DataParse(format!(
                "factor `{}` value cast failed: {}",
                result.factor_id, e
            ))
        })?
        .f64()
        .map_err(|e| {
            QuantixError::DataParse(format!(
                "factor `{}` value read failed: {}",
                result.factor_id, e
            ))
        })
        .cloned()
}

fn pearson_from_pairs(pairs: &[(f64, f64)]) -> Option<f64> {
    if pairs.len() < 2 {
        return None;
    }

    let mean_x = pairs.iter().map(|(x, _)| *x).sum::<f64>() / pairs.len() as f64;
    let mean_y = pairs.iter().map(|(_, y)| *y).sum::<f64>() / pairs.len() as f64;
    let mut numerator = 0.0;
    let mut x_var = 0.0;
    let mut y_var = 0.0;
    for (x, y) in pairs {
        let dx = x - mean_x;
        let dy = y - mean_y;
        numerator += dx * dy;
        x_var += dx * dx;
        y_var += dy * dy;
    }

    let denominator = (x_var * y_var).sqrt();
    if denominator == 0.0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn sample_std(values: &[f64], mean: Option<f64>) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let mean = mean?;
    let variance = values
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum::<f64>()
        / (values.len() - 1) as f64;
    Some(variance.sqrt())
}
