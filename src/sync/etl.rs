/// 数据同步程序（ETL）
///
/// 实现 Python quantix ↔ quantix-rust 数据同步
/// 方向：PostgreSQL/TDengine → ClickHouse
use crate::core::Result;
use crate::db::clickhouse::{ClickHouseClient, KlineDataCH, MarketFundamentalSnapshotCH};
use crate::sources::kline_aggregator::KlineData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

/// 同步配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// PostgreSQL 连接字符串
    pub postgres_url: String,
    /// ClickHouse 连接字符串
    pub clickhouse_url: String,
    /// ClickHouse 数据库名
    pub clickhouse_db: String,
    /// ClickHouse 用户名
    pub clickhouse_user: String,
    /// ClickHouse 密码
    pub clickhouse_password: String,
    /// 批量大小
    pub batch_size: usize,
    /// 同步延迟（秒）
    pub sync_interval: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            postgres_url: std::env::var("POSTGRES_URL")
                .unwrap_or_else(|_| "postgresql://localhost:5432/quantix".to_string()),
            clickhouse_url: std::env::var("CLICKHOUSE_URL")
                .unwrap_or_else(|_| "http://localhost:8123".to_string()),
            clickhouse_db: std::env::var("CLICKHOUSE_DB").unwrap_or_else(|_| "quantix".to_string()),
            clickhouse_user: std::env::var("CLICKHOUSE_USER")
                .unwrap_or_else(|_| "default".to_string()),
            clickhouse_password: std::env::var("CLICKHOUSE_PASSWORD")
                .unwrap_or_else(|_| "".to_string()),
            batch_size: 1000,
            sync_interval: 300, // 5分钟
        }
    }
}

/// 同步统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    /// 同步开始时间
    pub start_time: DateTime<Utc>,
    /// 同步结束时间
    pub end_time: DateTime<Utc>,
    /// 同步的记录数
    pub records_synced: usize,
    /// 失败的记录数
    pub records_failed: usize,
    /// 耗时（秒）
    pub elapsed_seconds: i64,
}

/// 市场基础面快照同步记录。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketFundamentalSyncRecord {
    pub code: String,
    pub snapshot_date: chrono::NaiveDate,
    pub market_cap: Option<f64>,
    pub latest_report_profit: Option<f64>,
    pub profit_source: String,
    pub pe_dynamic: Option<f64>,
}

/// 数据同步器
pub struct DataSync {
    config: SyncConfig,
    clickhouse_client: ClickHouseClient,
}

impl DataSync {
    /// 创建新的同步器
    pub async fn new(config: SyncConfig) -> Result<Self> {
        let clickhouse_client = ClickHouseClient::new(
            &config.clickhouse_url,
            &config.clickhouse_db,
            &config.clickhouse_user,
            &config.clickhouse_password,
        )
        .await?;

        info!("数据同步器初始化完成");

        Ok(Self {
            config,
            clickhouse_client,
        })
    }

    /// 使用默认配置创建
    pub async fn with_default_config() -> Result<Self> {
        let config = SyncConfig::default();
        Self::new(config).await
    }

    /// 同步日线数据（PostgreSQL → ClickHouse）
    pub async fn sync_daily_klines(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<SyncStats> {
        info!("开始同步日线数据: {} 到 {}", start_date, end_date);

        let start_time = Utc::now();

        let daily_data = Self::fetch_daily_source_data(&self.config, start_date, end_date).await?;

        // 写入 ClickHouse
        let records_synced = self.write_klines_to_clickhouse(&daily_data).await?;

        let end_time = Utc::now();
        let elapsed = end_time.signed_duration_since(start_time).num_seconds();

        let stats = SyncStats {
            start_time,
            end_time,
            records_synced,
            records_failed: 0,
            elapsed_seconds: elapsed,
        };

        info!(
            "日线数据同步完成：{} 条记录，耗时 {} 秒",
            records_synced, elapsed
        );

        Ok(stats)
    }

    /// 同步分钟线数据（TDengine → ClickHouse）
    pub async fn sync_minute_klines(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<SyncStats> {
        info!("开始同步分钟线数据");

        let start = Utc::now();

        let minute_data =
            Self::fetch_minute_source_data(&self.config, start_time, end_time).await?;

        let records_synced = self.write_klines_to_clickhouse(&minute_data).await?;

        let end = Utc::now();
        let elapsed = end.signed_duration_since(start).num_seconds();

        let stats = SyncStats {
            start_time: start,
            end_time: end,
            records_synced,
            records_failed: 0,
            elapsed_seconds: elapsed,
        };

        info!(
            "分钟线数据同步完成：{} 条记录，耗时 {} 秒",
            records_synced, elapsed
        );

        Ok(stats)
    }

    /// 将上游已准备好的市场基础面记录落盘到 ClickHouse。
    ///
    /// 该入口不负责抓取外部数据，只负责本地 ETL 写入，便于后续替换不同来源。
    pub async fn sync_market_fundamentals(
        &self,
        records: &[MarketFundamentalSyncRecord],
    ) -> Result<SyncStats> {
        info!("开始同步市场基础面快照: {} 条记录", records.len());

        let start_time = Utc::now();
        let records_synced = self
            .write_market_fundamentals_to_clickhouse(records)
            .await?;
        let end_time = Utc::now();
        let elapsed = end_time.signed_duration_since(start_time).num_seconds();

        Ok(SyncStats {
            start_time,
            end_time,
            records_synced,
            records_failed: 0,
            elapsed_seconds: elapsed,
        })
    }

    /// 写入 K线数据到 ClickHouse
    async fn write_klines_to_clickhouse(&self, klines: &[KlineData]) -> Result<usize> {
        if klines.is_empty() {
            return Ok(0);
        }

        let client = self.clickhouse_client.client();

        // 准备插入数据
        for chunk in klines.chunks(self.config.batch_size) {
            let mut insert = client.insert("kline_data").map_err(|e| {
                crate::core::QuantixError::DatabaseConnection(format!(
                    "ClickHouse insert error: {:?}",
                    e
                ))
            })?;

            for kline in chunk {
                let kline_ch = KlineDataCH {
                    timestamp: crate::db::clickhouse::datetime_utc_to_offsetdatetime(
                        kline.timestamp,
                    ),
                    code: kline.code.clone(),
                    name: kline.name.clone(),
                    period: kline.period.as_str().to_string(),
                    open: kline.open,
                    high: kline.high,
                    low: kline.low,
                    close: kline.close,
                    volume: kline.volume,
                    amount: kline.amount,
                    trade_count: kline.trade_count,
                    source: kline.source.clone(),
                };

                insert.write(&kline_ch).await.map_err(|e| {
                    crate::core::QuantixError::DatabaseConnection(format!(
                        "ClickHouse write error: {:?}",
                        e
                    ))
                })?;
            }

            insert.end().await.map_err(|e| {
                crate::core::QuantixError::DatabaseConnection(format!(
                    "ClickHouse end error: {:?}",
                    e
                ))
            })?;

            debug!("ClickHouse 写入完成：{} 条记录", chunk.len());
        }

        Ok(klines.len())
    }

    async fn fetch_daily_source_data(
        _config: &SyncConfig,
        _start_date: chrono::NaiveDate,
        _end_date: chrono::NaiveDate,
    ) -> Result<Vec<KlineData>> {
        Err(crate::core::QuantixError::Unsupported(
            "DataSync::fetch_daily_source_data 尚未接入 PostgreSQL 日线来源".to_string(),
        ))
    }

    async fn fetch_minute_source_data(
        _config: &SyncConfig,
        _start_time: DateTime<Utc>,
        _end_time: DateTime<Utc>,
    ) -> Result<Vec<KlineData>> {
        Err(crate::core::QuantixError::Unsupported(
            "DataSync::fetch_minute_source_data 尚未接入分钟线来源".to_string(),
        ))
    }

    async fn write_market_fundamentals_to_clickhouse(
        &self,
        records: &[MarketFundamentalSyncRecord],
    ) -> Result<usize> {
        if records.is_empty() {
            return Ok(0);
        }

        let snapshots = records
            .iter()
            .map(|record| MarketFundamentalSnapshotCH {
                code: record.code.clone(),
                snapshot_date: record.snapshot_date,
                market_cap: record.market_cap,
                latest_report_profit: record.latest_report_profit,
                profit_source: record.profit_source.clone(),
                pe_dynamic: record.pe_dynamic,
                updated_at: Utc::now()
                    .naive_utc()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            })
            .collect::<Vec<_>>();

        self.clickhouse_client
            .insert_market_fundamental_snapshots(&snapshots)
            .await?;

        Ok(snapshots.len())
    }

    /// 运行定时同步
    pub async fn run_sync_schedule(&self) -> Result<()> {
        info!("启动定时同步任务");

        loop {
            let today = Utc::now().date_naive();

            // 同步最近30天的日线数据
            let start_date = today - chrono::Duration::days(30);
            let end_date = today;

            match self.sync_daily_klines(start_date, end_date).await {
                Ok(stats) => {
                    info!("日线同步成功: {} 条记录", stats.records_synced);
                }
                Err(e) => {
                    error!("日线同步失败: {}", e);
                }
            }

            // 等待下次同步
            tokio::time::sleep(Duration::from_secs(self.config.sync_interval)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::QuantixError;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }

    struct SyncEnvGuard {
        clickhouse_user: Option<String>,
        clickhouse_password: Option<String>,
    }

    impl SyncEnvGuard {
        fn capture() -> Self {
            Self {
                clickhouse_user: std::env::var("CLICKHOUSE_USER").ok(),
                clickhouse_password: std::env::var("CLICKHOUSE_PASSWORD").ok(),
            }
        }
    }

    impl Drop for SyncEnvGuard {
        fn drop(&mut self) {
            match &self.clickhouse_user {
                Some(value) => unsafe { std::env::set_var("CLICKHOUSE_USER", value) },
                None => unsafe { std::env::remove_var("CLICKHOUSE_USER") },
            }

            match &self.clickhouse_password {
                Some(value) => unsafe { std::env::set_var("CLICKHOUSE_PASSWORD", value) },
                None => unsafe { std::env::remove_var("CLICKHOUSE_PASSWORD") },
            }
        }
    }

    #[test]
    fn test_sync_config_default() {
        let _lock = env_lock();
        let _guard = SyncEnvGuard::capture();
        unsafe {
            std::env::remove_var("CLICKHOUSE_USER");
            std::env::remove_var("CLICKHOUSE_PASSWORD");
        }

        let config = SyncConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.sync_interval, 300);
        assert_eq!(config.clickhouse_user, "default");
        assert_eq!(config.clickhouse_password, "");
    }

    #[test]
    fn test_sync_config_reads_clickhouse_auth_from_env() {
        let _lock = env_lock();
        let _guard = SyncEnvGuard::capture();
        unsafe {
            std::env::set_var("CLICKHOUSE_USER", "sync_user");
            std::env::set_var("CLICKHOUSE_PASSWORD", "sync_password");
        }

        let config = SyncConfig::default();
        assert_eq!(config.clickhouse_user, "sync_user");
        assert_eq!(config.clickhouse_password, "sync_password");
    }

    #[tokio::test]
    async fn test_fetch_daily_source_data_returns_unsupported() {
        let config = SyncConfig::default();
        let err = DataSync::fetch_daily_source_data(
            &config,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[tokio::test]
    async fn test_fetch_minute_source_data_returns_unsupported() {
        let config = SyncConfig::default();
        let err = DataSync::fetch_minute_source_data(
            &config,
            Utc::now() - chrono::Duration::days(1),
            Utc::now(),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[test]
    fn test_market_fundamental_sync_record_serialization_roundtrip() {
        let record = MarketFundamentalSyncRecord {
            code: "600519".to_string(),
            snapshot_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 14).unwrap(),
            market_cap: Some(23000.5),
            latest_report_profit: Some(862.1),
            profit_source: "report".to_string(),
            pe_dynamic: Some(27.4),
        };

        let json = serde_json::to_string(&record).unwrap();
        let restored: MarketFundamentalSyncRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, record);
    }
}
