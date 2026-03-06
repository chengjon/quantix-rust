/// ClickHouse 数据库客户端
///
/// 采用 MergeTree 引擎，针对 A股量化分析优化

use crate::core::{QuantixError, Result};
use clickhouse::Client;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

/// ClickHouse 客户端
pub struct ClickHouseClient {
    client: Client,
    database: String,
}

impl ClickHouseClient {
    /// 创建新的 ClickHouse 客户端
    ///
    /// ## 参数
    /// - `url`: ClickHouse HTTP 地址，如 "http://localhost:8123"
    /// - `database`: 数据库名称
    pub async fn new(url: &str, database: &str) -> Result<Self> {
        let client = Client::default()
            .with_url(url)
            .with_database(database);

        info!("ClickHouse 客户端初始化: {} -> {}", url, database);

        Ok(Self {
            client,
            database: database.to_string(),
        })
    }

    /// 使用默认配置创建
    pub async fn with_default_config() -> Result<Self> {
        let url = std::env::var("CLICKHOUSE_URL")
            .unwrap_or_else(|_| "http://localhost:8123".to_string());
        let database = std::env::var("CLICKHOUSE_DB")
            .unwrap_or_else(|_| "quantix".to_string());

        Self::new(&url, &database).await
    }

    /// 获取底层客户端
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// 初始化数据库和表
    pub async fn init_database(&self) -> Result<()> {
        info!("初始化 ClickHouse 数据库和表...");

        // 创建数据库
        let create_db = format!("CREATE DATABASE IF NOT EXISTS {}", self.database);
        self.client
            .query(&create_db)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建数据库失败: {}", e)))?;

        info!("数据库 {} 创建成功", self.database);

        // 创建表
        self.create_stock_info_table().await?;
        self.create_stock_quotes_table().await?;
        self.create_kline_data_table().await?;
        self.create_limit_up_events_table().await?;
        self.create_gbbq_events_table().await?;

        info!("所有 ClickHouse 表创建成功");
        Ok(())
    }

    /// 创建股票基本信息表
    async fn create_stock_info_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS stock_info ON CLUSTER '{cluster}' (
                code String,
                name String,
                market UInt8,
                list_date Date,
                status String,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            ORDER BY (market, code)
        "#;

        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建 stock_info 表失败: {}", e)))?;

        info!("stock_info 表创建成功");
        Ok(())
    }

    /// 创建股票实时行情表
    async fn create_stock_quotes_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS stock_realtime_quotes ON CLUSTER '{cluster}' (
                timestamp UInt64,
                code String,
                name String,
                price Float64,
                preclose Float64,
                open Float64,
                high Float64,
                low Float64,
                volume Float64,
                amount Float64,
                change_percent Float64,
                market UInt8,
                date MATERIALIZED toDate(toDateTime(timestamp))
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(toDateTime(timestamp))
            ORDER BY (date, code, timestamp)
            SETTINGS index_granularity = 8192
        "#;

        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建 stock_realtime_quotes 表失败: {}", e)))?;

        info!("stock_realtime_quotes 表创建成功");
        Ok(())
    }

    /// 创建 K线数据表
    async fn create_kline_data_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS kline_data ON CLUSTER '{cluster}' (
                timestamp DateTime,
                code String,
                name String,
                period String,
                open Float64,
                high Float64,
                low Float64,
                close Float64,
                volume Float64,
                amount Float64,
                trade_count UInt32,
                source String,
                date MATERIALIZED toDate(timestamp)
            )
            ENGINE = MergeTree()
            PARTITION BY (period, toYYYYMM(timestamp))
            ORDER BY (date, code, period, timestamp)
            SETTINGS index_granularity = 8192
        "#;

        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建 kline_data 表失败: {}", e)))?;

        info!("kline_data 表创建成功");
        Ok(())
    }

    /// 创建涨停事件表
    async fn create_limit_up_events_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS limit_up_events ON CLUSTER '{cluster}' (
                limit_time DateTime,
                code String,
                name String,
                limit_type String,
                open_price Float64,
                limit_price Float64,
                sealed_amount Float64,
                sealed_volume Float64,
                buy1_volume Float64,
                volume Float64,
                amount Float64,
                turnover_rate Float32,
                sector_name String,
                is_first_board UInt8,
                preclose Float64,
                date MATERIALIZED toDate(limit_time)
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(limit_time)
            ORDER BY (date, limit_time, code)
            SETTINGS index_granularity = 8192
        "#;

        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建 limit_up_events 表失败: {}", e)))?;

        info!("limit_up_events 表创建成功");
        Ok(())
    }

    /// 创建股本变迁事件表 (除权除息)
    async fn create_gbbq_events_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS gbbq_events ON CLUSTER '{cluster}' (
                event_date Date,
                code String,
                category UInt8,
                dividend Float32,
                bonus_price Float32,
                bonus_share Float32,
                rights_share Float32,
                ex_price Nullable(Float64),
                record_date Nullable(Date),
                market UInt8,
                created_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(created_at)
            PARTITION BY toYYYYMM(event_date)
            ORDER BY (event_date, code, category)
            SETTINGS index_granularity = 8192
        "#;

        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建 gbbq_events 表失败: {}", e)))?;

        info!("gbbq_events 表创建成功");
        Ok(())
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

    /// 插入 GBBQ 事件
    pub async fn insert_gbbq_event(&self, event: &crate::data::models::GbbqEvent) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO gbbq_events (
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date, market
            ) VALUES
            ('{}', '{}', {}, {}, {}, {}, {}, {}, {}, {})
            "#,
            event.event_date,
            event.code,
            event.category,
            event.dividend,
            event.bonus_price,
            event.bonus_share,
            event.rights_share,
            event.ex_price.map(|v| v.to_string()).unwrap_or("NULL".to_string()),
            event.record_date.map(|d| d.to_string()).unwrap_or("NULL".to_string()),
            if event.code.starts_with('6') || event.code.starts_with('5') { 1u8 } else { 0u8 }
        );

        self.client
            .query(&sql)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入 GBBQ 事件失败: {}", e)))?;

        Ok(())
    }

    /// 批量插入 GBBQ 事件
    pub async fn insert_gbbq_events(
        &self,
        events: &[crate::data::models::GbbqEvent],
    ) -> Result<()> {
        for event in events {
            self.insert_gbbq_event(event).await?;
        }
        Ok(())
    }

    /// 查询股票的 GBBQ 事件
    pub async fn get_gbbq_events(
        &self,
        code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<crate::data::models::GbbqEvent>> {
        let mut where_clause = format!("code = '{}'", code);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND event_date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND event_date <= '{}'", end));
        }

        let sql = format!(
            r#"
            SELECT
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date
            FROM gbbq_events
            WHERE {}
            ORDER BY event_date ASC
            "#,
            where_clause
        );

        let rows: Vec<GbbqEventCH> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询 GBBQ 事件失败: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::data::models::GbbqEvent {
                code: r.code,
                event_date: r.event_date,
                category: r.category,
                dividend: r.dividend,
                bonus_price: r.bonus_price,
                bonus_share: r.bonus_share,
                rights_share: r.rights_share,
                ex_price: r.ex_price,
                record_date: r.record_date,
            })
            .collect())
    }

    /// 获取股票的最新除权除息事件
    pub async fn get_latest_gbbq_event(
        &self,
        code: &str,
    ) -> Result<Option<crate::data::models::GbbqEvent>> {
        let sql = format!(
            r#"
            SELECT
                event_date, code, category, dividend, bonus_price,
                bonus_share, rights_share, ex_price, record_date
            FROM gbbq_events
            WHERE code = '{}' AND category = 1
            ORDER BY event_date DESC
            LIMIT 1
            "#,
            code
        );

        let rows: Vec<GbbqEventCH> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询最新 GBBQ 事件失败: {}", e)))?;

        Ok(rows.into_iter().next().map(|r| crate::data::models::GbbqEvent {
            code: r.code,
            event_date: r.event_date,
            category: r.category,
            dividend: r.dividend,
            bonus_price: r.bonus_price,
            bonus_share: r.bonus_share,
            rights_share: r.rights_share,
            ex_price: r.ex_price,
            record_date: r.record_date,
        }))
    }
}

/// 股票基本信息 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct StockInfoCH {
    pub code: String,
    pub name: String,
    pub market: u8,
    pub list_date: chrono::NaiveDate,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

/// 股票实时行情 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct StockQuoteCH {
    pub timestamp: u64,
    pub code: String,
    pub name: String,
    pub price: f64,
    pub preclose: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub amount: f64,
    pub change_percent: f64,
    pub market: u8,
}

/// K线数据 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct KlineDataCH {
    pub timestamp: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub period: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub trade_count: u32,
    pub source: String,
}

/// 涨停事件 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct LimitUpEventCH {
    pub limit_time: DateTime<Utc>,
    pub code: String,
    pub name: String,
    pub limit_type: String,
    pub open_price: f64,
    pub limit_price: f64,
    pub sealed_amount: f64,
    pub sealed_volume: f64,
    pub buy1_volume: f64,
    pub volume: f64,
    pub amount: f64,
    pub turnover_rate: f32,
    pub sector_name: String,
    pub is_first_board: u8,
    pub preclose: f64,
}

/// GBBQ 事件 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct GbbqEventCH {
    pub event_date: chrono::NaiveDate,
    pub code: String,
    pub category: u8,
    pub dividend: f32,
    pub bonus_price: f32,
    pub bonus_share: f32,
    pub rights_share: f32,
    pub ex_price: Option<f64>,
    pub record_date: Option<chrono::NaiveDate>,
}

impl Default for ClickHouseClient {
    fn default() -> Self {
        Self {
            client: Client::default(),
            database: "quantix".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_info_ch_derive() {
        // 测试 Row derive 是否正确
        let info = StockInfoCH {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            market: 0,
            list_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            status: "active".to_string(),
            updated_at: Utc::now(),
        };
        assert_eq!(info.code, "000001");
    }
}
