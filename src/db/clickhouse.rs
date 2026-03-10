use crate::core::runtime::ClickHouseSettings;
/// ClickHouse 数据库客户端
///
/// 采用 MergeTree 引擎，针对 A股量化分析优化
use crate::core::{QuantixError, Result};
use crate::market::{
    BoardRankRow, BoardType, LeaderFilter, LeaderRow, MarketSentimentSnapshot, NorthFlowSnapshot,
};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use clickhouse::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info};

/// ClickHouse 客户端
pub struct ClickHouseClient {
    client: Client,
    database: String,
    /// 批量插入的批次大小
    batch_size: usize,
}

/// 默认批次大小
const DEFAULT_BATCH_SIZE: usize = 1000;

impl ClickHouseClient {
    /// 创建新的 ClickHouse 客户端
    ///
    /// ## 参数
    /// - `url`: ClickHouse HTTP 地址，如 "http://localhost:8123"
    /// - `database`: 数据库名称
    pub async fn new(url: &str, database: &str) -> Result<Self> {
        let client = Client::default().with_url(url).with_database(database);

        info!("ClickHouse 客户端初始化: {} -> {}", url, database);

        Ok(Self {
            client,
            database: database.to_string(),
            batch_size: DEFAULT_BATCH_SIZE,
        })
    }

    /// 使用共享设置创建
    pub async fn from_settings(settings: &ClickHouseSettings) -> Result<Self> {
        Self::new(&settings.url, &settings.database).await
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
        self.create_market_tables().await?;

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 stock_info 表失败: {}", e))
            })?;

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!(
                    "创建 stock_realtime_quotes 表失败: {}",
                    e
                ))
            })?;

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 kline_data 表失败: {}", e))
            })?;

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 limit_up_events 表失败: {}", e))
            })?;

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
            .map_err(|e| {
                QuantixError::DatabaseConnection(format!("创建 gbbq_events 表失败: {}", e))
            })?;

        info!("gbbq_events 表创建成功");
        Ok(())
    }

    /// 创建 Phase 23 市场分析表
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
            event
                .ex_price
                .map(|v| v.to_string())
                .unwrap_or("NULL".to_string()),
            event
                .record_date
                .map(|d| d.to_string())
                .unwrap_or("NULL".to_string()),
            if event.code.starts_with('6') || event.code.starts_with('5') {
                1u8
            } else {
                0u8
            }
        );

        self.client
            .query(&sql)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入 GBBQ 事件失败: {}", e)))?;

        Ok(())
    }

    /// 批量插入 GBBQ 事件 (优化版本)
    ///
    /// 使用 clickhouse crate 的 insert API 进行批量插入
    pub async fn insert_gbbq_events(
        &self,
        events: &[crate::data::models::GbbqEvent],
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条 GBBQ 事件", events.len());

        // 分批插入，避免单次请求过大
        for chunk in events.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("gbbq_events")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for event in chunk {
                let row = GbbqEventCH {
                    event_date: event.event_date,
                    code: event.code.clone(),
                    category: event.category,
                    dividend: event.dividend,
                    bonus_price: event.bonus_price,
                    bonus_share: event.bonus_share,
                    rights_share: event.rights_share,
                    ex_price: event.ex_price,
                    record_date: event.record_date,
                };
                insert.write(&row).await.map_err(|e| {
                    QuantixError::DatabaseQuery(format!("写入 GBBQ 事件失败: {}", e))
                })?;
            }

            insert.end().await.map_err(|e| {
                QuantixError::DatabaseQuery(format!("批量插入 GBBQ 事件失败: {}", e))
            })?;

            debug!("成功插入 {} 条 GBBQ 事件", chunk.len());
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

        let rows: Vec<GbbqEventCH> =
            self.client.query(&sql).fetch_all().await.map_err(|e| {
                QuantixError::DatabaseQuery(format!("查询最新 GBBQ 事件失败: {}", e))
            })?;

        Ok(rows
            .into_iter()
            .next()
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
            }))
    }

    /// 查询 K线数据 (支持多周期)
    pub async fn get_kline_data(
        &self,
        code: &str,
        period: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
        limit: Option<usize>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut where_clause = format!("code = '{}' AND period = '{}'", code, period);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND date <= '{}'", end));
        }

        let limit_str = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            r#"
            SELECT
                timestamp, code, name, period,
                open, high, low, close, volume, amount
            FROM kline_data
            WHERE {}
            ORDER BY timestamp ASC
            {}
            "#,
            where_clause, limit_str
        );

        let rows: Vec<KlineDataCH> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("查询 K线数据失败: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::data::models::Kline {
                code: r.code.clone(),
                date: r.timestamp.date_naive(),
                open: Decimal::from_str(&format!("{}", r.open)).unwrap_or_default(),
                high: Decimal::from_str(&format!("{}", r.high)).unwrap_or_default(),
                low: Decimal::from_str(&format!("{}", r.low)).unwrap_or_default(),
                close: Decimal::from_str(&format!("{}", r.close)).unwrap_or_default(),
                volume: r.volume as i64,
                amount: Decimal::from_str(&format!("{}", r.amount)).ok(),
                adjust_type: crate::data::models::AdjustType::None,
            })
            .collect())
    }

    /// 插入 K线数据
    pub async fn insert_kline_data(
        &self,
        kline: &crate::data::models::Kline,
        period: &str,
    ) -> Result<()> {
        let sql = format!(
            r#"
            INSERT INTO kline_data (
                timestamp, code, name, period,
                open, high, low, close, volume, amount, source
            ) VALUES
                (toDateTime64('{}'), '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, 'TDX')
            "#,
            kline.date.format("%Y-%m-%d %H:%M:%S"),
            kline.code,
            kline.code, // name 默认使用 code
            period,
            kline.open.to_string(),
            kline.high.to_string(),
            kline.low.to_string(),
            kline.close.to_string(),
            kline.volume,
            kline.amount.unwrap_or_default().to_string()
        );

        self.client
            .query(&sql)
            .execute()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入 K线数据失败: {}", e)))?;

        Ok(())
    }

    /// 批量插入 K线数据 (优化版本)
    ///
    /// 使用 clickhouse crate 的 insert API 进行批量插入
    pub async fn insert_kline_data_batch(
        &self,
        klines: &[crate::data::models::Kline],
        period: &str,
    ) -> Result<()> {
        if klines.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条 K线数据 (周期: {})", klines.len(), period);

        // 分批插入
        for chunk in klines.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("kline_data")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for kline in chunk {
                let timestamp = kline
                    .date
                    .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let row = KlineDataCH {
                    timestamp: DateTime::<Utc>::from_naive_utc_and_offset(timestamp, Utc),
                    code: kline.code.clone(),
                    name: kline.code.clone(), // name 默认使用 code
                    period: period.to_string(),
                    open: kline.open.to_f64().unwrap_or(0.0),
                    high: kline.high.to_f64().unwrap_or(0.0),
                    low: kline.low.to_f64().unwrap_or(0.0),
                    close: kline.close.to_f64().unwrap_or(0.0),
                    volume: kline.volume as f64,
                    amount: kline.amount.unwrap_or_default().to_f64().unwrap_or(0.0),
                    trade_count: 0,
                    source: "TDX".to_string(),
                };
                insert
                    .write(&row)
                    .await
                    .map_err(|e| QuantixError::DatabaseQuery(format!("写入 K线数据失败: {}", e)))?;
            }

            insert
                .end()
                .await
                .map_err(|e| QuantixError::DatabaseQuery(format!("批量插入 K线数据失败: {}", e)))?;

            debug!("成功插入 {} 条 K线数据", chunk.len());
        }

        Ok(())
    }

    /// 聚合查询：从分钟线聚合为日线
    pub async fn get_daily_from_minute(
        &self,
        code: &str,
        start_date: Option<chrono::NaiveDate>,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<Vec<crate::data::models::Kline>> {
        let mut where_clause = format!("code = '{}' AND period = '1m'", code);

        if let Some(start) = start_date {
            where_clause.push_str(&format!(" AND date >= '{}'", start));
        }
        if let Some(end) = end_date {
            where_clause.push_str(&format!(" AND date <= '{}'", end));
        }

        let sql = format!(
            r#"
            SELECT
                toStartOfDay(timestamp) as day,
                argMin(open, timestamp) as open,
                max(high) as high,
                min(low) as low,
                argMax(close, timestamp) as close,
                sum(volume) as volume,
                sum(amount) as amount
            FROM kline_data
            WHERE {}
            GROUP BY day
            ORDER BY day ASC
            "#,
            where_clause
        );

        // 使用原始查询结果
        let _result: Vec<u8> = self
            .client
            .query(&sql)
            .fetch_all()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("聚合查询失败: {}", e)))?;

        // 解析结果（手动解析因为结构特殊）
        // 这里简化处理，实际应用中可以使用更复杂的序列化
        // 暂时返回空 Vec，实际应该解析 ClickHouse 返回的格式
        Ok(vec![])
    }

    /// 插入单条实时行情
    pub async fn insert_stock_quote(&self, quote: &StockQuoteCH) -> Result<()> {
        let mut insert = self
            .client
            .insert("stock_realtime_quotes")
            .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?;
        insert
            .write(quote)
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入实时行情失败: {}", e)))?;
        insert
            .end()
            .await
            .map_err(|e| QuantixError::DatabaseQuery(format!("插入实时行情失败: {}", e)))?;
        Ok(())
    }

    /// 批量插入实时行情
    pub async fn insert_stock_quotes_batch(&self, quotes: &[StockQuoteCH]) -> Result<()> {
        if quotes.is_empty() {
            return Ok(());
        }

        debug!("批量插入 {} 条实时行情", quotes.len());

        for chunk in quotes.chunks(self.batch_size) {
            let mut insert = self
                .client
                .insert("stock_realtime_quotes")
                .map_err(|e| QuantixError::DatabaseQuery(format!("创建插入器失败: {}", e)))?
                .with_option("async_insert", "1")
                .with_option("wait_for_async_insert", "1");

            for quote in chunk {
                insert
                    .write(quote)
                    .await
                    .map_err(|e| QuantixError::DatabaseQuery(format!("写入实时行情失败: {}", e)))?;
            }

            insert
                .end()
                .await
                .map_err(|e| QuantixError::DatabaseQuery(format!("批量插入实时行情失败: {}", e)))?;

            debug!("成功插入 {} 条实时行情", chunk.len());
        }

        Ok(())
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

/// 板块日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct SectorDailyCH {
    pub sector_code: String,
    pub sector_name: String,
    pub sector_type: String,
    pub trade_date: chrono::NaiveDate,
    pub change_pct: f64,
    pub rank: u32,
    pub leader_code: Option<String>,
    pub leader_name: Option<String>,
    pub leader_change: Option<f64>,
    pub updated_at: DateTime<Utc>,
}

impl SectorDailyCH {
    pub fn try_into_board_rank(self) -> Result<BoardRankRow> {
        Ok(BoardRankRow::new(
            self.sector_code,
            self.sector_name,
            parse_board_type(&self.sector_type)?,
            self.rank as usize,
            self.change_pct,
        ))
    }

    pub fn try_into_leader(self, filter: LeaderFilter) -> Result<Option<LeaderRow>> {
        let board_type = parse_board_type(&self.sector_type)?;
        let leader_code = match self.leader_code {
            Some(code) if !code.trim().is_empty() => code,
            _ => return Ok(None),
        };
        let leader_name = match self.leader_name {
            Some(name) if !name.trim().is_empty() => name,
            _ => return Ok(None),
        };
        let change_pct = self.leader_change.unwrap_or_default();

        let (sector_name, concept_name) = match filter {
            LeaderFilter::Sector(name) => (Some(name), None),
            LeaderFilter::Concept(name) => (None, Some(name)),
            LeaderFilter::All => match board_type {
                BoardType::Sector => (Some(self.sector_name), None),
                BoardType::Concept => (None, Some(self.sector_name)),
            },
        };

        Ok(Some(LeaderRow::new(
            leader_code,
            leader_name,
            sector_name,
            concept_name,
            change_pct,
        )))
    }
}

/// 北向资金日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct NorthFlowDailyCH {
    pub trade_date: chrono::NaiveDate,
    pub sh_amount: f64,
    pub sz_amount: f64,
    pub total_amount: f64,
    pub balance: f64,
    pub updated_at: DateTime<Utc>,
}

impl NorthFlowDailyCH {
    pub fn into_snapshot(self) -> NorthFlowSnapshot {
        NorthFlowSnapshot::new(
            self.trade_date,
            self.sh_amount,
            self.sz_amount,
            self.total_amount,
            self.balance,
        )
    }
}

/// 市场情绪日线 (ClickHouse Row)
#[derive(Debug, Clone, Serialize, Deserialize, clickhouse::Row)]
pub struct MarketSentimentDailyCH {
    pub trade_date: chrono::NaiveDate,
    pub up_count: u32,
    pub down_count: u32,
    pub limit_up_count: u32,
    pub limit_down_count: u32,
    pub seal_rate: f64,
    pub break_rate: f64,
    pub consecutive_board_count: u32,
    pub updated_at: DateTime<Utc>,
}

impl MarketSentimentDailyCH {
    pub fn into_snapshot(self) -> MarketSentimentSnapshot {
        MarketSentimentSnapshot::new(
            self.trade_date,
            self.up_count as usize,
            self.down_count as usize,
            self.limit_up_count as usize,
            self.limit_down_count as usize,
            self.seal_rate,
            self.break_rate,
            self.consecutive_board_count as usize,
        )
    }
}

fn market_table_sqls() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "sector_daily",
            r#"
            CREATE TABLE IF NOT EXISTS sector_daily ON CLUSTER '{cluster}' (
                sector_code String,
                sector_name String,
                sector_type String,
                trade_date Date,
                change_pct Float64,
                rank UInt32,
                leader_code Nullable(String),
                leader_name Nullable(String),
                leader_change Nullable(Float64),
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY (trade_date, sector_type, rank, sector_code)
        "#,
        ),
        (
            "north_flow_daily",
            r#"
            CREATE TABLE IF NOT EXISTS north_flow_daily ON CLUSTER '{cluster}' (
                trade_date Date,
                sh_amount Float64,
                sz_amount Float64,
                total_amount Float64,
                balance Float64,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY trade_date
        "#,
        ),
        (
            "market_sentiment_daily",
            r#"
            CREATE TABLE IF NOT EXISTS market_sentiment_daily ON CLUSTER '{cluster}' (
                trade_date Date,
                up_count UInt32,
                down_count UInt32,
                limit_up_count UInt32,
                limit_down_count UInt32,
                seal_rate Float64,
                break_rate Float64,
                consecutive_board_count UInt32,
                updated_at DateTime DEFAULT now()
            )
            ENGINE = ReplacingMergeTree(updated_at)
            PARTITION BY toYYYYMM(trade_date)
            ORDER BY trade_date
        "#,
        ),
    ]
}

fn parse_board_type(sector_type: &str) -> Result<BoardType> {
    match sector_type.trim().to_ascii_lowercase().as_str() {
        "industry" | "sector" => Ok(BoardType::Sector),
        "concept" => Ok(BoardType::Concept),
        other => Err(QuantixError::DataParse(format!(
            "未知的板块类型: {}",
            other
        ))),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::{BoardType, LeaderFilter};

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

    #[test]
    fn test_market_table_sqls_include_phase23_tables() {
        let sql = market_table_sqls()
            .into_iter()
            .map(|(_, sql)| sql)
            .collect::<Vec<_>>()
            .join("\n");

        assert!(sql.contains("CREATE TABLE IF NOT EXISTS sector_daily"));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS north_flow_daily"));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS market_sentiment_daily"));
    }

    #[test]
    fn test_market_sector_row_maps_to_board_rank_and_leader() {
        let row = SectorDailyCH {
            sector_code: "BK001".to_string(),
            sector_name: "银行".to_string(),
            sector_type: "industry".to_string(),
            trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            change_pct: 2.35,
            rank: 1,
            leader_code: Some("600000".to_string()),
            leader_name: Some("浦发银行".to_string()),
            leader_change: Some(5.61),
            updated_at: Utc::now(),
        };

        let board = row.clone().try_into_board_rank().unwrap();
        let leader = row
            .try_into_leader(LeaderFilter::Sector("银行".to_string()))
            .unwrap()
            .unwrap();

        assert_eq!(board.board_type, BoardType::Sector);
        assert_eq!(board.board_name, "银行");
        assert_eq!(board.rank, 1);
        assert_eq!(leader.code, "600000");
        assert_eq!(leader.sector_name.as_deref(), Some("银行"));
        assert_eq!(leader.concept_name, None);
    }

    #[test]
    fn test_market_north_flow_row_maps_to_snapshot() {
        let row = NorthFlowDailyCH {
            trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            sh_amount: 12.3,
            sz_amount: 8.6,
            total_amount: 20.9,
            balance: 99.1,
            updated_at: Utc::now(),
        };

        let snapshot = row.into_snapshot();

        assert_eq!(
            snapshot.trade_date,
            chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap()
        );
        assert_eq!(snapshot.total_amount, 20.9);
        assert_eq!(snapshot.balance, 99.1);
    }

    #[test]
    fn test_market_sentiment_row_maps_to_snapshot() {
        let row = MarketSentimentDailyCH {
            trade_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            up_count: 3210,
            down_count: 1875,
            limit_up_count: 87,
            limit_down_count: 4,
            seal_rate: 0.81,
            break_rate: 0.19,
            consecutive_board_count: 23,
            updated_at: Utc::now(),
        };

        let snapshot = row.into_snapshot();

        assert_eq!(snapshot.limit_up_count, 87);
        assert_eq!(snapshot.consecutive_board_count, 23);
        assert_eq!(snapshot.seal_rate, 0.81);
    }
}
