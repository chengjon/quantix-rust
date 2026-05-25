use polars::prelude::*;

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
