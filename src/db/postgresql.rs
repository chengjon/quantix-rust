use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
/// PostgreSQL 数据库访问层
///
/// 连接原 quantix 项目的 PostgreSQL 数据库，实现只读访问
use sqlx::postgres::PgPoolOptions;
use sqlx::{FromRow, Pool, Postgres};

use crate::core::error::{QuantixError, Result};

/// K线数据模型（与 Python 项目共享）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KlineDaily {
    pub code: String,
    pub trade_date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: i32, // 1=前复权, 2=后复权, 0=不复权
    pub created_at: Option<NaiveDateTime>,
}

/// 股票信息模型（与 Python 项目共享）
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market: String, // SH/SZ
    pub list_date: Option<NaiveDate>,
    pub delist_date: Option<NaiveDate>,
}

/// PostgreSQL 客户端
pub struct PostgresClient {
    pool: Pool<Postgres>,
}

impl PostgresClient {
    /// 创建新的 PostgreSQL 客户端
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| QuantixError::DatabaseConnection(e.to_string()))?;

        Ok(Self { pool })
    }

    /// 检查连接
    pub async fn check_connection(&self) -> Result<()> {
        let _result: i32 = sqlx::query_scalar("SELECT 1::int4")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;
        Ok(())
    }

    /// 查询日线数据（从共享数据库）
    pub async fn query_kline_daily(
        &self,
        code: &str,
        start: Option<&str>,
        end: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KlineDaily>> {
        let query = r#"
            SELECT code, trade_date, open, high, low, close, volume, amount, adjust_type, created_at
            FROM kline_daily
            WHERE code = $1
              AND ($2::date IS NULL OR trade_date >= $2)
              AND ($3::date IS NULL OR trade_date <= $3)
            ORDER BY trade_date DESC
            LIMIT $4
        "#;

        let start_date = parse_compact_date(start);
        let end_date = parse_compact_date(end);

        let rows = sqlx::query_as::<_, KlineDaily>(query)
            .bind(code)
            .bind(&start_date)
            .bind(&end_date)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

        Ok(rows)
    }

    /// 查询股票信息
    pub async fn query_stock_info(&self, code: &str) -> Result<Option<StockInfo>> {
        let query = r#"
            SELECT code, name, market, list_date, delist_date
            FROM stock_info
            WHERE code = $1
        "#;

        let result = sqlx::query_as::<_, StockInfo>(query)
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

        Ok(result)
    }

    /// 列出所有股票
    pub async fn list_stocks(&self, market: Option<&str>) -> Result<Vec<StockInfo>> {
        let query = r#"
            SELECT code, name, market, list_date, delist_date
            FROM stock_info
            WHERE ($1::text IS NULL OR market = $1)
            ORDER BY code
        "#;

        let rows = sqlx::query_as::<_, StockInfo>(query)
            .bind(market)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

        Ok(rows)
    }
}

fn parse_compact_date(value: Option<&str>) -> Option<NaiveDate> {
    value.and_then(|value| NaiveDate::parse_from_str(value, "%Y%m%d").ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compact_date_returns_date_for_valid_compact_input() {
        assert_eq!(
            parse_compact_date(Some("20260403")),
            Some(NaiveDate::from_ymd_opt(2026, 4, 3).unwrap())
        );
    }

    #[test]
    fn parse_compact_date_returns_none_for_invalid_or_missing_input() {
        assert_eq!(parse_compact_date(Some("2026-04-03")), None);
        assert_eq!(parse_compact_date(None), None);
    }
}
