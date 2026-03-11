use crate::core::runtime::ClickHouseSettings;
use crate::core::{QuantixError, Result};
use clickhouse::Client;
use tracing::info;

mod data_ops;
mod rows;

pub use rows::{
    GbbqEventCH, KlineDataCH, LimitUpEventCH, MarketSentimentDailyCH, NorthFlowDailyCH,
    SectorDailyCH, StockInfoCH, StockQuoteCH,
};
use rows::market_table_sqls;

pub struct ClickHouseClient {
    client: Client,
    database: String,
    batch_size: usize,
}

const DEFAULT_BATCH_SIZE: usize = 1000;

impl ClickHouseClient {
    pub async fn new(url: &str, database: &str) -> Result<Self> {
        let client = Client::default().with_url(url).with_database(database);

        info!("ClickHouse 客户端初始化: {} -> {}", url, database);

        Ok(Self {
            client,
            database: database.to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
        })
    }

    pub async fn from_settings(settings: &ClickHouseSettings) -> Result<Self> {
        Self::new(&settings.url, &settings.database).await
    }

    pub async fn with_default_config() -> Result<Self> {
        Self::from_settings(&ClickHouseSettings::from_env()).await
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub async fn init_database(&self) -> Result<()> {
        info!("初始化 ClickHouse 数据库和表...");

        let create_db = format!("CREATE DATABASE IF NOT EXISTS {}", self.database);
        self.client
            .query(&create_db)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseConnection(format!("创建数据库失败: {}", e)))?;

        info!("数据库 {} 创建成功", self.database);

        self.create_stock_info_table().await?;
        self.create_stock_quotes_table().await?;
        self.create_kline_data_table().await?;
        self.create_limit_up_events_table().await?;
        self.create_gbbq_events_table().await?;
        self.create_market_tables().await?;

        info!("所有 ClickHouse 表创建成功");
        Ok(())
    }

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 stock_info 表失败: {}", e))
            })?;

        info!("stock_info 表创建成功");
        Ok(())
    }

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!(
                    "创建 stock_realtime_quotes 表失败: {}",
                    e
                ))
            })?;

        info!("stock_realtime_quotes 表创建成功");
        Ok(())
    }

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 kline_data 表失败: {}", e))
            })?;

        info!("kline_data 表创建成功");
        Ok(())
    }

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 limit_up_events 表失败: {}", e))
            })?;

        info!("limit_up_events 表创建成功");
        Ok(())
    }

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 gbbq_events 表失败: {}", e))
            })?;

        info!("gbbq_events 表创建成功");
        Ok(())
    }

    async fn create_market_tables(&self) -> Result<()> {
        for (table_name, sql) in market_table_sqls() {
            self.client
                .query(sql.replace("'{cluster}'", "single_cluster").as_str())
                .execute()
                .await
                .map_err(|e| {
                    QuantixError::DatabaseConnection(format!("创建 {} 表失败: {}", table_name, e))
                })?;

            info!("{} 表创建成功", table_name);
        }

        Ok(())
    }

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
        }
    }
}

impl ClickHouseClient {
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }
}
