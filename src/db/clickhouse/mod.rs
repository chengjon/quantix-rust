mod fundamentals;
mod gbbq;
mod kline;
mod minute;
mod models;
mod schema;
mod shadow_kline;

#[cfg(test)]
mod tests;

pub use self::minute::{
    StreamStats, stream_minute_klines_to_clickhouse, stream_minute_shares_to_clickhouse,
};
pub use self::models::{
    GbbqEventCH, KlineDataCH, LimitUpEventCH, MarketFundamentalSnapshotCH, MarketSentimentDailyCH,
    MinuteKlineCH, MinuteShareCH, NorthFlowDailyCH, SectorDailyCH, StockInfoCH, StockQuoteCH,
};

use crate::core::runtime::ClickHouseSettings;
use crate::core::{QuantixError, Result};
use chrono::{DateTime, Utc};
use clickhouse::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{debug, info};

/// ClickHouse 客户端
pub struct ClickHouseClient {
    client: Client,
    database: String,
    /// 批量插入的批次大小
    batch_size: usize,
    /// HTTP URL for direct queries (bypasses RowBinary)
    http_url: String,
    /// HTTP user
    http_user: String,
    /// HTTP password
    http_password: String,
}

/// 默认批次大小
const DEFAULT_BATCH_SIZE: usize = 1000;

impl ClickHouseClient {
    /// 创建新的 ClickHouse 客户端
    ///
    /// ## 参数
    /// - `url`: ClickHouse HTTP 地址，如 "http://localhost:8123"
    /// - `database`: 数据库名称
    /// - `user`: 用户名
    /// - `password`: 密码
    pub async fn new(url: &str, database: &str, user: &str, password: &str) -> Result<Self> {
        let client = Client::default()
            .with_url(url)
            .with_database(database)
            .with_user(user)
            .with_password(password);

        info!(
            "ClickHouse 客户端初始化: {} -> {} (user: {})",
            url, database, user
        );

        Ok(Self {
            client,
            database: database.to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
            http_url: url.to_string(),
            http_user: user.to_string(),
            http_password: password.to_string(),
        })
    }

    /// 使用共享设置创建
    pub async fn from_settings(settings: &ClickHouseSettings) -> Result<Self> {
        Self::new(
            &settings.url,
            &settings.database,
            &settings.user,
            &settings.password,
        )
        .await
    }

    /// 使用默认配置创建
    pub async fn with_default_config() -> Result<Self> {
        Self::from_settings(&ClickHouseSettings::from_env()).await
    }

    /// 获取底层客户端
    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    #[cfg(test)]
    pub(crate) fn http_auth_for_test(&self) -> (&str, &str) {
        (&self.http_user, &self.http_password)
    }

    /// Execute a query using HTTP with JSON format (bypasses RowBinary encoding issues)
    pub async fn query_json<T: for<'de> Deserialize<'de>>(&self, sql: &str) -> Result<Vec<T>> {
        let client = reqwest::Client::new();

        let url = format!(
            "{}/?user={}&password={}&database={}",
            self.http_url,
            urlencoding::encode(&self.http_user),
            urlencoding::encode(&self.http_password),
            self.database,
        );

        let query_with_format = format!("{}\nFORMAT JSONEachRow", sql);

        let response = client
            .post(&url)
            .body(query_with_format)
            .send()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("HTTP query failed: {}", e)))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            QuantixError::DatabaseQuery(format!("HTTP response read failed: {}", e))
        })?;

        if !status.is_success() {
            return Err(QuantixError::DatabaseQuery(format!(
                "Query failed ({}): {}",
                status, body
            )));
        }

        if body.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        for line in body.lines() {
            if line.is_empty() {
                continue;
            }
            let item: T = serde_json::from_str(line).map_err(|e| {
                QuantixError::DatabaseQuery(format!("JSON parse failed: {} in {}", e, line))
            })?;
            results.push(item);
        }

        Ok(results)
    }

    /// 检查连接
    pub async fn check_connection(&self) -> Result<()> {
        let result: Vec<u8> = self
            .client
            .query("SELECT 1")
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("连接检查失败: {}", e)))?;

        if !result.is_empty() && result[0] == 1 {
            info!("ClickHouse 连接正常");
            Ok(())
        } else {
            Err(QuantixError::DatabaseConnection("连接检查失败".to_string()))
        }
    }
}

impl Default for ClickHouseClient {
    fn default() -> Self {
        Self {
            client: Client::default(),
            database: "quantix".to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
            http_url: "http://localhost:8123".to_string(),
            http_user: "default".to_string(),
            http_password: "".to_string(),
        }
    }
}

impl ClickHouseClient {
    /// 设置批量插入批次大小
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// 获取当前批次大小
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}
