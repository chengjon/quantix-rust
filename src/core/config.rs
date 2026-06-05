/// 核心配置管理
///
/// 从原 quantix 项目的 config/ 目录读取共享配置
use config::{Config, Environment};
use serde::Deserialize;
use std::path::Path;

pub const CLICKHOUSE_URL_ENV: &str = "CLICKHOUSE_URL";
pub const CLICKHOUSE_DB_ENV: &str = "CLICKHOUSE_DB";
pub const CLICKHOUSE_USER_ENV: &str = "CLICKHOUSE_USER";
pub const CLICKHOUSE_PASSWORD_ENV: &str = "CLICKHOUSE_PASSWORD";
pub const DEFAULT_CLICKHOUSE_URL: &str = "http://localhost:8123";
pub const DEFAULT_CLICKHOUSE_DB: &str = "quantix";
pub const DEFAULT_CLICKHOUSE_USER: &str = "default";
pub const DEFAULT_CLICKHOUSE_PASSWORD: &str = "";
pub const UPSTREAM_MYSQL_URL_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_URL";
pub const UPSTREAM_MYSQL_DB_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_DB";
pub const UPSTREAM_MYSQL_USER_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_USER";
pub const UPSTREAM_MYSQL_PASSWORD_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_PASSWORD";
pub const DEFAULT_UPSTREAM_MYSQL_URL: &str = "mysql://127.0.0.1:3306";
pub const DEFAULT_UPSTREAM_MYSQL_DB: &str = "mystocks";
pub const DEFAULT_UPSTREAM_MYSQL_USER: &str = "root";
pub const DEFAULT_UPSTREAM_MYSQL_PASSWORD: &str = "";

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub tdengine: Option<TDengineConfig>,
    pub postgresql: Option<PostgresConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TDengineConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub mode: String, // "rest" or "websocket"
}

#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_max_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub tdx: Option<TdxConfig>,
    pub tdx_api: Option<TdxApiConfig>,
    pub akshare: Option<AkShareConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TdxConfig {
    pub hosts: Vec<String>,
    pub port: u16,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TdxApiConfig {
    #[serde(default = "default_tdx_api_url")]
    pub base_url: String,
    #[serde(default = "default_tdx_api_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_tdx_api_retries")]
    pub max_retries: u32,
    #[serde(default = "default_tdx_api_enabled")]
    pub enabled: bool,
    #[serde(default = "default_tdx_api_max_batch_quote_size")]
    pub max_batch_quote_size: usize,
    #[serde(default = "default_tdx_api_health_timeout")]
    pub health_timeout_secs: u64,
}

fn default_tdx_api_url() -> String {
    std::env::var("TDX_API_URL").unwrap_or_else(|_| "http://tdx-api:8080".to_string())
}

fn default_tdx_api_timeout() -> u64 {
    std::env::var("TDX_API_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn default_tdx_api_retries() -> u32 {
    3
}

fn default_tdx_api_enabled() -> bool {
    std::env::var("TDX_API_ENABLED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(true)
}

fn default_tdx_api_max_batch_quote_size() -> usize {
    std::env::var("TDX_API_MAX_BATCH_QUOTE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
}

fn default_tdx_api_health_timeout() -> u64 {
    std::env::var("TDX_API_HEALTH_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

#[derive(Debug, Deserialize, Clone)]
pub struct AkShareConfig {
    pub base_url: String,
    pub rate_limit: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub data_sources: DataSourceConfig,
}

impl AppConfig {
    /// 从配置文件加载（支持共享原 quantix 配置）
    pub fn load(config_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut builder = Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(Environment::default().separator("__"));

        // 如果指定了配置目录，尝试从该目录加载
        if Path::new(config_dir).exists() {
            let config_file = Path::new(config_dir).join("data_sources.toml");
            if config_file.exists() {
                builder = builder.add_source(config::File::from(config_file));
            }
        }

        let config = builder.build()?;
        config
            .try_deserialize()
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}
