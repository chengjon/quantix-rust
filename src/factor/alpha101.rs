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

fn rolling_options(window: usize) -> RollingOptionsFixedWindow {
    RollingOptionsFixedWindow {
        window_size: window,
        min_periods: window,
        ..Default::default()
    }
}

fn cs_rank_expr(expr: Expr) -> Expr {
    expr.rank(Default::default(), None)
        .cast(DataType::Float64)
        .over([col("date")])
}

fn ts_delta_expr(expr: Expr, periods: usize) -> Expr {
    expr.clone() - expr.shift(lit(periods as i64)).over([col("symbol")])
}

fn vwap_expr() -> Expr {
    col("amount") / col("volume").cast(DataType::Float64)
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

pub fn alpha101_002(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate_two_stage(
        df,
        vec![
            ts_delta_expr(col("volume").cast(DataType::Float64).log(10.0), 2)
                .alias("__alpha101_002_volume_delta"),
            ((col("close") - col("open")) / col("open")).alias("__alpha101_002_intraday_return"),
        ],
        vec![
            cs_rank_expr(col("__alpha101_002_volume_delta"))
                .alias("__alpha101_002_rank_volume_delta"),
            cs_rank_expr(col("__alpha101_002_intraday_return"))
                .alias("__alpha101_002_rank_intraday_return"),
        ],
    )?;
    let mut values = rolling_corr_by_symbol(
        &frame,
        "__alpha101_002_rank_volume_delta",
        "__alpha101_002_rank_intraday_return",
        6,
    )?;
    values = &values * -1.0;
    Ok(values)
}

pub fn alpha101_003(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            cs_rank_expr(col("open")).alias("__alpha101_003_rank_open"),
            cs_rank_expr(col("volume").cast(DataType::Float64)).alias("__alpha101_003_rank_volume"),
        ],
    )?;
    let mut values = rolling_corr_by_symbol(
        &frame,
        "__alpha101_003_rank_open",
        "__alpha101_003_rank_volume",
        10,
    )?;
    values = &values * -1.0;
    Ok(values)
}

pub fn alpha101_005(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate_two_stage(
        df,
        vec![
            vwap_expr()
                .rolling_mean(rolling_options(10))
                .over([col("symbol")])
                .alias("__alpha101_005_mean_vwap"),
        ],
        vec![
            cs_rank_expr(col("open") - col("__alpha101_005_mean_vwap"))
                .alias("__alpha101_005_rank_open_vwap_mean"),
            cs_rank_expr(col("close") - vwap_expr()).alias("__alpha101_005_rank_close_vwap"),
        ],
    )?;
    collect_factor_value(
        &frame,
        col("__alpha101_005_rank_open_vwap_mean")
            * (lit(-1.0) * col("__alpha101_005_rank_close_vwap").abs()),
    )
}

pub fn alpha101_006(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            col("volume")
                .cast(DataType::Float64)
                .alias("__alpha101_006_volume"),
        ],
    )?;
    let mut values = rolling_corr_by_symbol(&frame, "open", "__alpha101_006_volume", 10)?;
    values = &values * -1.0;
    Ok(values)
}

pub fn alpha101_012(df: &DataFrame) -> PolarsResult<Series> {
    let volume_delta = ts_delta_expr(col("volume").cast(DataType::Float64), 1);
    let volume_sign = when(volume_delta.clone().gt(lit(0.0)))
        .then(lit(1.0))
        .when(volume_delta.lt(lit(0.0)))
        .then(lit(-1.0))
        .otherwise(lit(0.0));
    collect_factor_value(df, volume_sign * lit(-1.0) * ts_delta_expr(col("close"), 1))
}
