use std::collections::BTreeMap;

use polars::prelude::*;

use crate::core::{QuantixError, Result};
use crate::factor::dataset::FactorDataset;
use crate::factor::types::FactorComputeResult;

/// 分层回测请求：groups 分层数（必须 ≥ 2）、horizon 未来收益率 horizon（天）。
#[derive(Debug, Clone)]
pub struct LayeredBacktestRequest {
    pub groups: usize,
    pub horizon: usize,
}

/// 分层回测汇总：factor_id 因子 id、groups/horizon/periods 元信息、long_short_mean 多空组累计收益均值（无可评估周期时为 None）。
#[derive(Debug, Clone)]
pub struct LayeredBacktestSummary {
    pub factor_id: String,
    pub groups: usize,
    pub horizon: usize,
    pub periods: usize,
    pub long_short_mean: Option<f64>,
}

/// 分层回测完整结果：summary 汇总、by_period 每期各层收益 DataFrame、long_short 多空收益时序 DataFrame。
#[derive(Debug, Clone)]
pub struct LayeredBacktestResult {
    pub summary: LayeredBacktestSummary,
    pub by_period: DataFrame,
    pub long_short: DataFrame,
}

/// 运行分层回测：按因子值将标的分 request.groups 组、计算 request.horizon 天未来收益、汇总各层均值与多空差。groups<2 或数据不足返回错误。
pub fn run_layered_factor_backtest(
    dataset: &FactorDataset,
    factor: &FactorComputeResult,
    request: &LayeredBacktestRequest,
) -> Result<LayeredBacktestResult> {
    if request.groups < 2 {
        return Err(QuantixError::DataParse(
            "layered backtest groups must be at least two".to_string(),
        ));
    }
    if request.horizon == 0 {
        return Err(QuantixError::DataParse(
            "layered backtest horizon must be greater than zero".to_string(),
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
            .shift(lit(-(request.horizon as i64)))
            .over([col("symbol")])
            / col("close"))
            - lit(1.0))
        .alias("__forward_return")])
        .collect()
        .map_err(|e| QuantixError::DataParse(format!("forward return compute failed: {}", e)))?;

    let dates = factor
        .frame
        .column("date")
        .map_err(|e| QuantixError::DataParse(format!("factor date column read failed: {}", e)))?;
    let factor_values = f64_column(&factor.frame, "value")?;
    let returns = f64_column(&forward_returns, "__forward_return")?;

    let mut rows_by_date: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for row in 0..factor.frame.height() {
        if let (Some(factor_value), Some(forward_return)) =
            (factor_values.get(row), returns.get(row))
        {
            let date = dates
                .get(row)
                .map_err(|e| QuantixError::DataParse(format!("factor date read failed: {}", e)))?
                .to_string();
            rows_by_date
                .entry(date)
                .or_default()
                .push((factor_value, forward_return));
        }
    }

    let mut output_dates = Vec::new();
    let mut output_groups = Vec::new();
    let mut output_returns = Vec::new();
    let mut output_counts = Vec::new();
    let mut long_short_dates = Vec::new();
    let mut long_short_values = Vec::new();

    for (date, mut rows) in rows_by_date {
        if rows.len() < request.groups {
            continue;
        }
        rows.sort_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let row_count = rows.len();
        let mut group_returns = vec![Vec::new(); request.groups];
        for (rank, (_, forward_return)) in rows.into_iter().enumerate() {
            let group_index = (rank * request.groups / row_count).min(request.groups - 1);
            group_returns[group_index].push(forward_return);
        }

        let mut period_group_means = Vec::with_capacity(request.groups);
        for (idx, returns) in group_returns.iter().enumerate() {
            let group_mean = mean(returns);
            output_dates.push(date.clone());
            output_groups.push((idx + 1) as i64);
            output_returns.push(group_mean);
            output_counts.push(returns.len() as i64);
            period_group_means.push(group_mean);
        }

        if let (Some(low), Some(high)) = (
            period_group_means.first().and_then(|value| *value),
            period_group_means.last().and_then(|value| *value),
        ) {
            long_short_dates.push(date);
            long_short_values.push(high - low);
        }
    }

    let long_short_mean = mean(&long_short_values);
    let by_period = df!(
        "date" => output_dates,
        "group" => output_groups,
        "return" => output_returns,
        "count" => output_counts,
    )
    .map_err(|e| {
        QuantixError::DataParse(format!("layered backtest period frame build failed: {}", e))
    })?;
    let long_short = df!(
        "date" => long_short_dates,
        "long_short" => long_short_values,
    )
    .map_err(|e| {
        QuantixError::DataParse(format!(
            "layered backtest long-short frame build failed: {}",
            e
        ))
    })?;

    Ok(LayeredBacktestResult {
        summary: LayeredBacktestSummary {
            factor_id: factor.factor_id.clone(),
            groups: request.groups,
            horizon: request.horizon,
            periods: long_short.height(),
            long_short_mean,
        },
        by_period,
        long_short,
    })
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

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
