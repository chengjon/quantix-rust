use chrono::{DateTime, NaiveDateTime, Utc};
/// TDengine REST API 客户端
///
/// 通过 REST API 连接原 quantix 项目的 TDengine 数据库
/// 高频时序数据读取
use reqwest::Client;
use serde::Deserialize;

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
    token: String,
    client: Client,
}

impl TDengineClient {
    /// 创建新的 TDengine REST 客户端
    pub fn new(base_url: &str, token: &str) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| QuantixError::DatabaseConnection(e.to_string()))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            client,
        })
    }

    /// 检查连接
    pub async fn check_connection(&self) -> Result<()> {
        let url = format!("{}/rest/login/{}", self.base_url, self.token);
        self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(e.to_string()))?;

        Ok(())
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

        let url = format!("{}/rest/sql/{}", self.base_url, self.token);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "sql": sql }))
            .send()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(e.to_string()))?;

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
                let ts = NaiveDateTime::from_timestamp_opt(row.ts, 0)
                    .unwrap_or_else(|| NaiveDateTime::from_timestamp_millis(row.ts).unwrap())
                    .and_utc();
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
}
