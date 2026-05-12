use polars::prelude::*;

fn collect_factor_value(df: &DataFrame, expr: Expr) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([expr.alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .cloned()
}

fn cs_rank_expr(expr: Expr) -> Expr {
    expr.rank(Default::default(), None)
        .cast(DataType::Float64)
        .over([col("date")])
}

fn intraday_position_expr() -> Expr {
    (col("close") - col("open")) / ((col("high") - col("low")) + lit(1e-12))
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
