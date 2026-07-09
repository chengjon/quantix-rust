use polars::prelude::*;

/// 截面 rank：按 date 分组对指定列做 rank（默认升序），返回与原行序一致的 Series。输入列：col_name + date。
pub fn cs_rank(df: &DataFrame, col_name: &str) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([col(col_name)
            .rank(Default::default(), None)
            .over([col("date")])
            .alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .map(|column| column.as_materialized_series().clone())
}

/// 时序延迟：按 symbol 分组，将 col_name 向下平移 periods 期（首 periods 行为 null），返回与原行序一致的 Series。
pub fn ts_delay(df: &DataFrame, col_name: &str, periods: usize) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([col(col_name)
            .shift(lit(periods as i64))
            .over([col("symbol")])
            .alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .map(|column| column.as_materialized_series().clone())
}

/// 时序差分：按 symbol 分组返回 col_name(t) - col_name(t - periods)；首 periods 行为 null，与原行序一致。
pub fn ts_delta(df: &DataFrame, col_name: &str, periods: usize) -> PolarsResult<Series> {
    df.clone()
        .lazy()
        .with_columns([(col(col_name)
            - col(col_name)
                .shift(lit(periods as i64))
                .over([col("symbol")]))
        .alias("__factor_value")])
        .collect()?
        .column("__factor_value")
        .map(|column| column.as_materialized_series().clone())
}

/// 时序滚动 rank：按 symbol 分组、对 col_name 做长度为 window 的滚动排名；为保证顺序，先按 (symbol, date) 排序，再恢复原行序返回 Series。
pub fn ts_rank(df: &DataFrame, col_name: &str, window: usize) -> PolarsResult<Series> {
    let mut sorted = df
        .clone()
        .with_row_index("__factor_row_nr".into(), None)?
        .sort(["symbol", "date"], Default::default())?;
    let mut values = crate::factor::rolling::rolling_rank_by_symbol(&sorted, col_name, window)?;
    values.rename("__factor_value".into());
    sorted.with_column(values.into())?;
    sorted
        .sort(["__factor_row_nr"], Default::default())?
        .column("__factor_value")
        .map(|column| column.as_materialized_series().clone())
}
