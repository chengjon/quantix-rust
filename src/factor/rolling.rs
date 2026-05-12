use polars::prelude::*;

pub(crate) fn rolling_corr_by_symbol(
    df: &DataFrame,
    left_col: &str,
    right_col: &str,
    window: usize,
) -> PolarsResult<Series> {
    let frame = df.group_by_stable(["symbol"])?.apply(|group| {
        let left = group.column(left_col)?.f64()?;
        let right = group.column(right_col)?.f64()?;
        let values = rolling_corr_values(left, right, window);
        DataFrame::new(vec![Series::new("__factor_value".into(), values)])
    })?;
    frame.column("__factor_value").cloned()
}

fn rolling_corr_values(
    left: &Float64Chunked,
    right: &Float64Chunked,
    window: usize,
) -> Vec<Option<f64>> {
    let mut out = Vec::with_capacity(left.len());
    for idx in 0..left.len() {
        if idx + 1 < window {
            out.push(None);
            continue;
        }

        let start = idx + 1 - window;
        let mut xs = Vec::with_capacity(window);
        let mut ys = Vec::with_capacity(window);
        for offset in start..=idx {
            match (left.get(offset), right.get(offset)) {
                (Some(x), Some(y)) => {
                    xs.push(x);
                    ys.push(y);
                }
                _ => {
                    xs.clear();
                    break;
                }
            }
        }

        if xs.len() != window {
            out.push(None);
            continue;
        }

        let mean_x = xs.iter().sum::<f64>() / window as f64;
        let mean_y = ys.iter().sum::<f64>() / window as f64;
        let mut numerator = 0.0;
        let mut x_var = 0.0;
        let mut y_var = 0.0;
        for (x, y) in xs.iter().zip(ys.iter()) {
            let dx = x - mean_x;
            let dy = y - mean_y;
            numerator += dx * dy;
            x_var += dx * dx;
            y_var += dy * dy;
        }

        let denominator = (x_var * y_var).sqrt();
        if denominator == 0.0 {
            out.push(None);
        } else {
            out.push(Some(numerator / denominator));
        }
    }
    out
}

pub(crate) fn rolling_rank_by_symbol(
    df: &DataFrame,
    col_name: &str,
    window: usize,
) -> PolarsResult<Series> {
    let frame = df.group_by_stable(["symbol"])?.apply(|group| {
        let values = group.column(col_name)?.f64()?;
        let ranks = rolling_rank_values(values, window);
        DataFrame::new(vec![Series::new("__factor_value".into(), ranks)])
    })?;
    frame.column("__factor_value").cloned()
}

fn rolling_rank_values(values: &Float64Chunked, window: usize) -> Vec<Option<f64>> {
    let mut out = Vec::with_capacity(values.len());
    for idx in 0..values.len() {
        if idx + 1 < window {
            out.push(None);
            continue;
        }

        let start = idx + 1 - window;
        let Some(current) = values.get(idx) else {
            out.push(None);
            continue;
        };

        let mut less = 0usize;
        let mut equal = 0usize;
        let mut valid = 0usize;
        for offset in start..=idx {
            match values.get(offset) {
                Some(value) if value < current => {
                    less += 1;
                    valid += 1;
                }
                Some(value) if value == current => {
                    equal += 1;
                    valid += 1;
                }
                Some(_) => valid += 1,
                None => break,
            }
        }

        if valid != window || equal == 0 {
            out.push(None);
        } else {
            let average_rank = less as f64 + (equal as f64 + 1.0) / 2.0;
            out.push(Some(average_rank));
        }
    }
    out
}

pub(crate) fn rolling_std_by_symbol(
    df: &DataFrame,
    col_name: &str,
    window: usize,
) -> PolarsResult<Series> {
    let frame = df.group_by_stable(["symbol"])?.apply(|group| {
        let values = group.column(col_name)?.f64()?;
        let stddev = rolling_std_values(values, window);
        DataFrame::new(vec![Series::new("__factor_value".into(), stddev)])
    })?;
    frame.column("__factor_value").cloned()
}

fn rolling_std_values(values: &Float64Chunked, window: usize) -> Vec<Option<f64>> {
    let mut out = Vec::with_capacity(values.len());
    for idx in 0..values.len() {
        if idx + 1 < window {
            out.push(None);
            continue;
        }

        let start = idx + 1 - window;
        let mut xs = Vec::with_capacity(window);
        for offset in start..=idx {
            match values.get(offset) {
                Some(value) => xs.push(value),
                None => {
                    xs.clear();
                    break;
                }
            }
        }

        if xs.len() != window {
            out.push(None);
            continue;
        }

        let mean = xs.iter().sum::<f64>() / window as f64;
        let variance = xs
            .iter()
            .map(|value| {
                let diff = value - mean;
                diff * diff
            })
            .sum::<f64>()
            / window as f64;
        out.push(Some(variance.sqrt()));
    }
    out
}
