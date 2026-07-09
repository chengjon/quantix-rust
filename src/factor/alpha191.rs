use crate::factor::rolling::{
    rolling_corr_by_symbol, rolling_rank_by_symbol, rolling_std_by_symbol,
};
use polars::prelude::*;

fn collect_factor_value(df: &DataFrame, expr: Expr) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([expr.alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .map(|column| column.as_materialized_series().clone())
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

/// Alpha191 #101：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_101(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, intraday_position_expr())
}

/// Alpha191 #102：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_102(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(
        df,
        lit(-1.0)
            * cs_rank_expr(col("close") - col("open"))
            * cs_rank_expr(col("volume").cast(DataType::Float64)),
    )
}

/// Alpha191 #103：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_103(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(
        df,
        intraday_position_expr() * col("volume").cast(DataType::Float64),
    )
}

/// Alpha191 #104：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
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
    frame.with_column(corr.into())?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_104_corr")))
}

/// Alpha191 #105：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
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

/// Alpha191 #106：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
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

/// Alpha191 #107：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
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

/// Alpha191 #108：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_108(df: &DataFrame) -> PolarsResult<Series> {
    alpha191_103(df)
}

/// Alpha191 #109：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_109(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, lit(-1.0) * ts_delta_expr(col("close"), 5))
}

/// Alpha191 #110：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_110(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![(col("low") - ts_delay_expr(col("close"), 1)).alias("__alpha191_110_gap_down")],
    )?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_110_gap_down")))
}

/// Alpha191 #111：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_111(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            (((col("close") - ts_delay_expr(col("close"), 1))
                / (ts_delay_expr(col("close"), 1) + lit(1e-12)))
                / (((ts_delay_expr(col("close"), 1) - col("open")) / (col("open") + lit(1e-12)))
                    + lit(1e-12)))
            .alias("__alpha191_111_ratio"),
        ],
    )?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_111_ratio")))
}

/// Alpha191 #112：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_112(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![ts_delta_expr(col("close"), 1).alias("__alpha191_112_delta_close")],
    )?;
    collect_factor_value(
        &frame,
        lit(-1.0)
            * cs_rank_expr(col("__alpha191_112_delta_close"))
            * cs_rank_expr(col("volume").cast(DataType::Float64)),
    )
}

/// Alpha191 #113：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_113(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate_two_stage(
        df,
        vec![
            col("volume")
                .cast(DataType::Float64)
                .alias("__alpha191_113_volume"),
        ],
        vec![
            cs_rank_expr(col("open")).alias("__alpha191_113_rank_open"),
            cs_rank_expr(col("__alpha191_113_volume")).alias("__alpha191_113_rank_volume"),
        ],
    )?;
    let mut values = rolling_corr_by_symbol(
        &frame,
        "__alpha191_113_rank_open",
        "__alpha191_113_rank_volume",
        10,
    )?;
    values = &values * -1.0;
    Ok(values)
}

/// Alpha191 #114：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_114(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, cs_rank_expr(intraday_position_expr()))
}

/// Alpha191 #115：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_115(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, lit(-1.0) * ts_delta_expr(col("close"), 7))
}

/// Alpha191 #116：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_116(df: &DataFrame) -> PolarsResult<Series> {
    let frame = collect_intermediate(
        df,
        vec![
            (col("close") - ts_delay_expr(col("close"), 1))
                .abs()
                .alias("__alpha191_116_abs_change"),
        ],
    )?;
    let mut values = rolling_rank_by_symbol(&frame, "__alpha191_116_abs_change", 20)?;
    values = &values * -1.0;
    Ok(values)
}

/// Alpha191 #117：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_117(df: &DataFrame) -> PolarsResult<Series> {
    alpha191_103(df)
}

/// Alpha191 #118：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_118(df: &DataFrame) -> PolarsResult<Series> {
    let mut frame = collect_intermediate(
        df,
        vec![
            col("close")
                .cast(DataType::Float64)
                .alias("__alpha191_118_close"),
            col("volume")
                .cast(DataType::Float64)
                .alias("__alpha191_118_volume"),
        ],
    )?;
    let mut corr =
        rolling_corr_by_symbol(&frame, "__alpha191_118_close", "__alpha191_118_volume", 5)?;
    corr.rename("__alpha191_118_corr".into());
    frame.with_column(corr.into())?;
    collect_factor_value(&frame, cs_rank_expr(col("__alpha191_118_corr")))
}

/// Alpha191 #119：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_119(df: &DataFrame) -> PolarsResult<Series> {
    collect_factor_value(df, lit(-1.0) * ts_delta_expr(col("close"), 3))
}

/// Alpha191 #120：按 191Alpha 公式集计算的横截面因子，输入 DataFrame 至少包含 OHLCV 列，返回单 Series。
pub fn alpha191_120(df: &DataFrame) -> PolarsResult<Series> {
    let mut frame = collect_intermediate(
        df,
        vec![
            col("close")
                .cast(DataType::Float64)
                .alias("__alpha191_120_close"),
        ],
    )?;
    let mut stddev = rolling_std_by_symbol(&frame, "__alpha191_120_close", 10)?;
    stddev.rename("__alpha191_120_stddev".into());
    frame.with_column(stddev.into())?;
    collect_factor_value(
        &frame,
        lit(-1.0) * cs_rank_expr(col("__alpha191_120_stddev")),
    )
}
