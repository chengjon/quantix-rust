//! Test helpers for PG-backed tests.
//!
//! Uses the `quantix_test` database (separate from production
//! `quantix`). Tests connect via `QUANTIX_POSTGRES_URL_TEST` env var
//! or fall back to the standard POSTGRESQL_* env vars with database
//! name overridden to `quantix_test`.

use quantix_cli::core::error::Result;

pub fn quantix_test_url() -> String {
    if let Ok(url) = std::env::var("QUANTIX_POSTGRES_URL_TEST") {
        return url;
    }
    let host = std::env::var("POSTGRESQL_HOST").unwrap_or_else(|_| "192.168.123.104".into());
    let port = std::env::var("POSTGRESQL_PORT").unwrap_or_else(|_| "5438".into());
    let user = std::env::var("POSTGRESQL_USER").unwrap_or_else(|_| "postgres".into());
    let pass = std::env::var("POSTGRESQL_PASSWORD").unwrap_or_else(|_| "".into());
    format!("postgres://{user}:{pass}@{host}:{port}/quantix_test")
}

/// Truncate `import_state` for the given date, leaving other dates intact.
/// Used by live tests to ensure a clean slate per test.
pub async fn truncate_state_for_date(date: chrono::NaiveDate) -> Result<()> {
    use quantix_cli::db::PostgresClient;
    let pg = PostgresClient::new(&quantix_test_url()).await?;
    sqlx::query("DELETE FROM import_state WHERE trade_date = $1")
        .bind(date)
        .execute(pg.pool())
        .await
        .map_err(|e| quantix_cli::core::error::QuantixError::DatabaseQuery(e.to_string()))?;
    Ok(())
}
