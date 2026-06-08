use chrono::{DateTime, Utc};
/// TDengine REST API 客户端
///
/// 通过 REST API 连接原 quantix 项目的 TDengine 数据库
/// 高频时序数据读取
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use tracing::debug;

use crate::core::error::{QuantixError, Result};

#[derive(Debug, Deserialize)]
pub struct TDengineRestResponse {
    pub status: String,
    pub data: Vec<TdengineRow>,
}

#[derive(Debug, Deserialize)]
pub struct TdengineRow {
    pub ts: i64,
    pub code: String,
    #[serde(rename = "open")]
    pub open: Option<f64>,
    #[serde(rename = "high")]
    pub high: Option<f64>,
    #[serde(rename = "low")]
    pub low: Option<f64>,
    #[serde(rename = "close")]
    pub close: Option<f64>,
    #[serde(rename = "volume")]
    pub volume: Option<i64>,
}

/// 分钟 K线数据
#[derive(Debug, Clone)]
pub struct MinuteKline {
    pub ts: DateTime<Utc>,
    pub code: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

/// TDengine REST 客户端
pub struct TDengineClient {
    base_url: String,
    username: String,
    password: String,
    database: Option<String>,
    client: Client,
}

impl TDengineClient {
    /// 创建新的 TDengine REST 客户端
    pub fn new(base_url: &str, token: &str) -> Result<Self> {
        Self::build(base_url, token, None)
    }

    /// 创建绑定数据库的 TDengine REST 客户端
    pub fn new_with_database(base_url: &str, token: &str, database: &str) -> Result<Self> {
        let database = normalize_identifier(database, "database")?;
        Self::build(base_url, token, Some(database))
    }

    fn build(base_url: &str, token: &str, database: Option<String>) -> Result<Self> {
        let (username, password) = parse_token(token)?;
        let client = Client::builder()
            .build()
            .map_err(|e| QuantixError::DatabaseConnection(e.to_string()))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username,
            password,
            database,
            client,
        })
    }

    /// 检查连接
    pub async fn check_connection(&self) -> Result<()> {
        self.execute_sql("show databases").await
    }

    /// 查询分钟线数据
    pub async fn query_minute_kline(
        &self,
        table: &str,
        code: &str,
        start: i64,
        end: i64,
        limit: usize,
    ) -> Result<Vec<MinuteKline>> {
        let sql = format!(
            "SELECT * FROM {} WHERE code='{}' AND ts > {} AND ts < {} ORDER BY ts DESC LIMIT {}",
            table, code, start, end, limit
        );

        let response = self.send_sql(&sql).await?;

        let resp: TDengineRestResponse = response
            .json()
            .await
            .map_err(|e| QuantixError::DataParse(e.to_string()))?;

        if resp.status != "succ" {
            return Err(QuantixError::DatabaseQuery(format!(
                "TDengine error: {}",
                resp.status
            )));
        }

        let klines = resp
            .data
            .into_iter()
            .map(|row| {
                let ts = chrono::DateTime::from_timestamp(row.ts, 0)
                    .unwrap_or_else(|| chrono::DateTime::from_timestamp_millis(row.ts).unwrap());
                MinuteKline {
                    ts,
                    code: row.code,
                    open: row.open.unwrap_or(0.0),
                    high: row.high.unwrap_or(0.0),
                    low: row.low.unwrap_or(0.0),
                    close: row.close.unwrap_or(0.0),
                    volume: row.volume.unwrap_or(0),
                }
            })
            .collect();

        Ok(klines)
    }

    /// 创建逐笔成交表
    pub async fn create_tick_table(&self) -> Result<()> {
        let tick_data = self.qualified_name("tick_data");
        let sql = format!(
            "CREATE STABLE IF NOT EXISTS {tick_data} ( \
            ts TIMESTAMP, \
            price DOUBLE, \
            volume INT, \
            amount DOUBLE, \
            direction TINYINT \
        ) TAGS (code BINARY(16))"
        );
        self.execute_sql(&sql).await
    }

    /// 批量插入逐笔成交数据
    pub async fn insert_ticks(
        &self,
        code: &str,
        ticks: &[(i64, f64, i32, f64, i32)],
    ) -> Result<()> {
        if ticks.is_empty() {
            return Ok(());
        }
        // TDengine REST SQL 批量插入
        let values: Vec<String> = ticks
            .iter()
            .map(|(ts, price, vol, amt, dir)| {
                format!("({}, {}, {}, {}, {})", ts, price, vol, amt, dir)
            })
            .collect();

        for chunk in values.chunks(5000) {
            let table = self.qualified_name(&format!("t_{code}"));
            let tick_data = self.qualified_name("tick_data");
            let sql = format!(
                "INSERT INTO {table} USING {tick_data} TAGS ('{code}') VALUES {}",
                chunk.join(" ")
            );
            self.execute_sql(&sql).await?;
            debug!("插入 {} 条逐笔数据: {}", chunk.len(), code);
        }
        Ok(())
    }

    /// 执行原始 SQL
    pub async fn execute_sql(&self, sql: &str) -> Result<()> {
        let response = self.send_sql(sql).await?;

        let resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| QuantixError::DataParse(e.to_string()))?;

        let status_ok = resp["status"].as_str() == Some("succ")
            || resp["code"].as_i64() == Some(0)
            || resp["code"].as_u64() == Some(0);
        if !status_ok {
            let desc = resp["desc"].as_str().unwrap_or("unknown error");
            // 忽略 "Table already exists" 等非致命错误
            if desc.contains("already exists") || desc.contains("Invalid table name") {
                return Ok(());
            }
            return Err(QuantixError::DatabaseQuery(format!("TDengine: {}", desc)));
        }
        Ok(())
    }

    async fn send_sql(&self, sql: &str) -> Result<reqwest::Response> {
        let url = format!("{}/rest/sql", self.base_url);
        let response = self
            .client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .header(CONTENT_TYPE, "text/plain")
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(QuantixError::DatabaseQuery(format!(
                "TDengine HTTP {}: {}",
                status, body
            )));
        }
        Ok(response)
    }

    fn qualified_name(&self, name: &str) -> String {
        match &self.database {
            Some(database) if !name.contains('.') => format!("{database}.{name}"),
            _ => name.to_string(),
        }
    }
}

fn parse_token(token: &str) -> Result<(String, String)> {
    let (username, password) = token
        .split_once(':')
        .ok_or_else(|| QuantixError::Config("TDengine token must be user:password".to_string()))?;
    if username.trim().is_empty() {
        return Err(QuantixError::Config(
            "TDengine username cannot be empty".to_string(),
        ));
    }
    Ok((username.to_string(), password.to_string()))
}

fn normalize_identifier(value: &str, label: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(QuantixError::Config(format!(
            "TDengine {label} cannot be empty"
        )));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return Err(QuantixError::Config(format!(
            "TDengine {label} must be an ASCII identifier"
        )));
    }
    Ok(value.to_string())
}
