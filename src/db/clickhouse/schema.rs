use tracing::info;

use crate::core::{QuantixError, Result};

use super::ClickHouseClient;
use super::models::market_table_sqls;

impl ClickHouseClient {
    /// 初始化数据库和表
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
        self.create_minute_klines_table().await?;
        self.create_minute_shares_table().await?;

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

    async fn create_minute_klines_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS minute_klines ON CLUSTER '{cluster}' (
                timestamp DateTime,
                code String,
                period String,
                adjust String,
                open Float64,
                high Float64,
                low Float64,
                close Float64,
                volume Float64,
                amount Float64,
                date MATERIALIZED toDate(timestamp)
            )
            ENGINE = MergeTree()
            PARTITION BY (period, toYYYYMM(timestamp))
            ORDER BY (date, code, period, adjust, timestamp)
            SETTINGS index_granularity = 8192
        "#;
        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 minute_klines 表失败: {}", e))
            })?;
        info!("minute_klines 表创建成功");
        Ok(())
    }

    async fn create_minute_shares_table(&self) -> Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS minute_shares ON CLUSTER '{cluster}' (
                timestamp DateTime,
                code String,
                price Float64,
                volume Float64,
                amount Float64,
                avg_price Float64,
                date MATERIALIZED toDate(timestamp)
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (date, code, timestamp)
            SETTINGS index_granularity = 8192
        "#;
        self.client
            .query(sql.replace("'{cluster}'", "single_cluster").as_str())
            .execute()
            .await
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 minute_shares 表失败: {}", e))
            })?;
        info!("minute_shares 表创建成功");
        Ok(())
    }
}
