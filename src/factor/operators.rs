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
        .cloned()
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
        .cloned()
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
        .cloned()
}
