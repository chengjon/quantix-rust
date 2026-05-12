use polars::prelude::*;

fn collect_factor_value(df: &DataFrame, expr: Expr) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([expr.alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .cloned()
}

fn collect_intermediate(df: &DataFrame, exprs: Vec<Expr>) -> PolarsResult<DataFrame> {
    df.clone().lazy().with_columns(exprs).collect()
}

fn collect_intermediate_two_stage(
    df: &DataFrame,
    first: Vec<Expr>,
    second: Vec<Expr>,
) -> PolarsResult<DataFrame> {
    df.clone()
        .lazy()
        .with_columns(first)
        .with_columns(second)
        .collect()
}

fn cs_rank_expr(expr: Expr) -> Expr {
    expr.rank(Default::default(), None)
        .cast(DataType::Float64)
        .over([col("date")])
}

fn ts_delay_expr(expr: Expr, periods: usize) -> Expr {
    expr.shift(lit(periods as i64)).over([col("symbol")])
}

fn ts_delta_expr(expr: Expr, periods: usize) -> Expr {
    expr.clone() - ts_delay_expr(expr, periods)
}

fn intraday_position_expr() -> Expr {
    (col("close") - col("open")) / ((col("high") - col("low")) + lit(1e-12))
}

fn rolling_corr_by_symbol(
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

fn rolling_rank_by_symbol(df: &DataFrame, col_name: &str, window: usize) -> PolarsResult<Series> {
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

pub fn alpha191_101(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, intraday_position_expr())
}

pub fn alpha191_102(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(
        df,
        lit(-1.0)
            * cs_rank_expr(col("close") - col("open"))
            * cs_rank_expr(col("volume").cast(DataType::Float64)),
    )
}

pub fn alpha191_103(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(
        df,
        intraday_position_expr() * col("volume").cast(DataType::Float64),
    )
}

pub fn alpha191_104(df: &DataFrame) -> PolarsResult<Series> {
    let mut frame = collect_intermediate(
        df,
        vec![
            col("close")
                .cast(DataType::Float64)
                .alias("__alpha191_104_close"),
            col("volume")
                .cast(DataType::Float64)
                .alias("__alpha191_104_volume"),
        ],
    )?;
    let mut corr =
        rolling_corr_by_symbol(&frame, "__alpha191_104_close", "__alpha191_104_volume", 10)?;
    corr.rename("__alpha191_104_corr".into());
    frame.with_column(corr)?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_104_corr")))
}

pub fn alpha191_105(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate_two_stage(
        df,
        vec![
            col("volume")
                .cast(DataType::Float64)
                .alias("__alpha191_105_volume"),
        ],
        vec![
            cs_rank_expr(col("high")).alias("__alpha191_105_rank_high"),
            cs_rank_expr(col("__alpha191_105_volume")).alias("__alpha191_105_rank_volume"),
        ],
    )?;
    let mut values = rolling_corr_by_symbol(
        &frame,
        "__alpha191_105_rank_high",
        "__alpha191_105_rank_volume",
        5,
    )?;
    values = &values * -1.0;
    Ok(values)
}

pub fn alpha191_106(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            (col("close") - col("open"))
                .abs()
                .alias("__alpha191_106_abs_change"),
        ],
    )?;
    let mut values = rolling_rank_by_symbol(&frame, "__alpha191_106_abs_change", 10)?;
    values = &values * -1.0;
    Ok(values)
}

pub fn alpha191_107(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            ((col("close") - col("open"))
                / (ts_delay_expr(col("close"), 1) - col("open") + lit(1e-12)))
            .alias("__alpha191_107_ratio"),
        ],
    )?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_107_ratio")))
}

pub fn alpha191_108(df: &DataFrame) -> PolarsResult<Series> {
    alpha191_103(df)
}

pub fn alpha191_109(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, lit(-1.0) * ts_delta_expr(col("close"), 5))
}

pub fn alpha191_110(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![(col("low") - ts_delay_expr(col("close"), 1)).alias("__alpha191_110_gap_down")],
    )?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_110_gap_down")))
}
