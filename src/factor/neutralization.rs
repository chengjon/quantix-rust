use std::collections::BTreeMap;

use polars::prelude::*;

use crate::core::{QuantixError, Result};
use crate::factor::dataset::FactorDataset;
use crate::factor::types::FactorComputeResult;

#[derive(Debug, Clone)]
pub struct NeutralizationRequest {
    pub exposures: Vec<String>,
    pub add_intercept: bool,
}

pub fn neutralize_factor_cross_sectional(
    dataset: &FactorDataset,
    factor: &FactorComputeResult,
    request: &NeutralizationRequest,
) -> Result<FactorComputeResult> {
    if request.exposures.is_empty() {
        return Err(QuantixError::DataParse(
            "neutralization requires at least one exposure column".to_string(),
        ));
    }
    if factor.frame.height() != dataset.frame().height() {
        return Err(QuantixError::DataParse(format!(
            "factor `{}` row count does not match dataset row count",
            factor.factor_id
        )));
    }

    let dates = factor
        .frame
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("factor date column read failed: {}", e)))?;
    let factor_values = f64_column(&factor.frame, "value")?;
    let exposure_values = request
        .exposures
        .iter()
        .map(|column| f64_column(dataset.frame(), column))
        .collect::<Result<Vec<_>>>()?;

    let mut rows_by_date: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for row in 0..factor.frame.height() {
        let date = dates
            .get(row)
            .map_err(|e| QuantixError::DataParse(format!("factor date read failed: {}", e)))?
            .to_string();
        rows_by_date.entry(date).or_default().push(row);
    }

    let mut neutralized_values = vec![None; factor.frame.height()];
    for row_indexes in rows_by_date.values() {
        neutralize_group(
            row_indexes,
            &factor_values,
            &exposure_values,
            request.add_intercept,
            &mut neutralized_values,
        );
    }

    let output_dates = string_column_values(&factor.frame, "date")?;
    let output_symbols = string_column_values(&factor.frame, "symbol")?;
    let frame = df!(
        "date" => output_dates,
        "symbol" => output_symbols,
        "value" => neutralized_values,
    )
    .map_err(|e| {
        QuantixError::DataParse(format!("neutralized factor frame build failed: {}", e))
    })?;

    Ok(FactorComputeResult {
        factor_id: format!("{}_neutralized", factor.factor_id),
        frame,
    })
}

fn neutralize_group(
    row_indexes: &[usize],
    factor_values: &Float64Chunked,
    exposure_values: &[Float64Chunked],
    add_intercept: bool,
    output: &mut [Option<f64>],
) {
    let parameter_count = exposure_values.len() + usize::from(add_intercept);
    if parameter_count == 0 {
        return;
    }

    let mut valid_rows = Vec::new();
    for &row in row_indexes {
        let Some(y) = factor_values.get(row) else {
            continue;
        };
        let exposures = exposure_values
            .iter()
            .map(|values| values.get(row))
            .collect::<Option<Vec<_>>>();
        if let Some(exposures) = exposures {
            valid_rows.push((row, y, exposures));
        }
    }

    if valid_rows.len() <= parameter_count {
        return;
    }

    let mut xtx = vec![vec![0.0; parameter_count]; parameter_count];
    let mut xty = vec![0.0; parameter_count];
    for (_, y, exposures) in &valid_rows {
        let x = design_row(exposures, add_intercept);
        for i in 0..parameter_count {
            xty[i] += x[i] * y;
            for j in 0..parameter_count {
                xtx[i][j] += x[i] * x[j];
            }
        }
    }

    let Some(beta) = solve_linear_system(xtx, xty) else {
        return;
    };

    for (row, y, exposures) in valid_rows {
        let x = design_row(&exposures, add_intercept);
        let fitted = x
            .iter()
            .zip(beta.iter())
            .map(|(feature, coefficient)| feature * coefficient)
            .sum::<f64>();
        output[row] = Some(y - fitted);
    }
}

fn design_row(exposures: &[f64], add_intercept: bool) -> Vec<f64> {
    let mut row = Vec::with_capacity(exposures.len() + usize::from(add_intercept));
    if add_intercept {
        row.push(1.0);
    }
    row.extend_from_slice(exposures);
    row
}

fn solve_linear_system(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    for pivot in 0..n {
        let mut best = pivot;
        for row in (pivot + 1)..n {
            if matrix[row][pivot].abs() > matrix[best][pivot].abs() {
                best = row;
            }
        }
        if matrix[best][pivot].abs() < 1e-12 {
            return None;
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);

        let pivot_value = matrix[pivot][pivot];
        for col in pivot..n {
            matrix[pivot][col] /= pivot_value;
        }
        rhs[pivot] /= pivot_value;

        for row in 0..n {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for col in pivot..n {
                matrix[row][col] -= factor * matrix[pivot][col];
            }
            rhs[row] -= factor * rhs[pivot];
        }
    }

    Some(rhs)
}

fn f64_column(frame: &DataFrame, column: &str) -> Result<Float64Chunked> {
    frame
        .column(column)
        .and_then(|series| series.cast(&DataType::Float64))
        .map_err(|e| QuantixError::DataParse(format!("column `{}` cast failed: {}", column, e)))?
        .f64()
        .map_err(|e| QuantixError::DataParse(format!("column `{}` read failed: {}", column, e)))
        .cloned()
}

fn string_column_values(frame: &DataFrame, name: &str) -> Result<Vec<String>> {
    let column = frame
        .column(name)
        .map_err(|e| QuantixError::DataParse(format!("{} column read failed: {}", name, e)))?;
    if let Ok(values) = column.str() {
        return Ok(values
            .into_iter()
            .map(|value| value.unwrap_or_default().to_string())
            .collect());
    }

    (0..column.len())
        .map(|row| {
            column
                .get(row)
                .map(|value| value.to_string())
                .map_err(|e| QuantixError::DataParse(format!("{} read failed: {}", name, e)))
        })
        .collect()
}
