/// 核心配置管理
///
/// 从原 quantix 项目的 config/ 目录读取共享配置
use config::{Config, Environment};
use serde::Deserialize;
use std::path::Path;

/// ClickHouse URL 环境变量（HTTP 端点）。
pub const CLICKHOUSE_URL_ENV: &str = "CLICKHOUSE_URL";
/// ClickHouse 数据库名环境变量。
pub const CLICKHOUSE_DB_ENV: &str = "CLICKHOUSE_DB";
/// ClickHouse 用户名环境变量。
pub const CLICKHOUSE_USER_ENV: &str = "CLICKHOUSE_USER";
/// ClickHouse 密码环境变量。
pub const CLICKHOUSE_PASSWORD_ENV: &str = "CLICKHOUSE_PASSWORD";
/// ClickHouse 默认 URL（本机 8123 端口）。
pub const DEFAULT_CLICKHOUSE_URL: &str = "http://localhost:8123";
/// ClickHouse 默认数据库名。
pub const DEFAULT_CLICKHOUSE_DB: &str = "quantix";
/// ClickHouse 默认用户名。
pub const DEFAULT_CLICKHOUSE_USER: &str = "default";
/// ClickHouse 默认密码（空串）。
pub const DEFAULT_CLICKHOUSE_PASSWORD: &str = "";
/// 上游 MySQL URL 环境变量（Tushare/proxy 层）。
pub const UPSTREAM_MYSQL_URL_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_URL";
/// 上游 MySQL 数据库名环境变量。
pub const UPSTREAM_MYSQL_DB_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_DB";
/// 上游 MySQL 用户名环境变量。
pub const UPSTREAM_MYSQL_USER_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_USER";
/// 上游 MySQL 密码环境变量。
pub const UPSTREAM_MYSQL_PASSWORD_ENV: &str = "QUANTIX_UPSTREAM_MYSQL_PASSWORD";
/// 上游 MySQL 默认 URL（本机 3306）。
pub const DEFAULT_UPSTREAM_MYSQL_URL: &str = "mysql://127.0.0.1:3306";
/// 上游 MySQL 默认数据库名。
pub const DEFAULT_UPSTREAM_MYSQL_DB: &str = "mystocks";
/// 上游 MySQL 默认用户名。
pub const DEFAULT_UPSTREAM_MYSQL_USER: &str = "root";
/// 上游 MySQL 默认密码（空串）。
pub const DEFAULT_UPSTREAM_MYSQL_PASSWORD: &str = "";

/// 数据库配置聚合：tdengine 时序库（可选）、postgresql 关系库（可选）。两者均可空，表示未启用对应数据源。
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub tdengine: Option<TDengineConfig>,
    pub postgresql: Option<PostgresConfig>,
}

/// TDengine 时序数据库配置：host/port/database/username/password 连接参数、mode 取值 "rest" 或 "websocket" 决定客户端协议。
#[derive(Debug, Deserialize, Clone)]
pub struct TDengineConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub mode: String, // "rest" or "websocket"
}

/// PostgreSQL 连接配置：host/port/database/username/password 连接参数、pool_max_size 连接池上限。
#[derive(Debug, Deserialize, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_max_size: u32,
}

/// 数据源配置聚合：tdx 通达信行情（可选）、akshare AkShare 数据源（可选）。
#[derive(Debug, Deserialize, Clone)]
pub struct DataSourceConfig {
    pub tdx: Option<TdxConfig>,
    pub akshare: Option<AkShareConfig>,
}

/// 通达信行情源配置：hosts 服务器列表（支持轮询）、port 端口、timeout 超时（毫秒）。
#[derive(Debug, Deserialize, Clone)]
pub struct TdxConfig {
    pub hosts: Vec<String>,
    pub port: u16,
    pub timeout: u64,
}

/// AkShare 数据源配置：base_url HTTP 入口、rate_limit 每秒请求数上限。
#[derive(Debug, Deserialize, Clone)]
pub struct AkShareConfig {
    pub base_url: String,
    pub rate_limit: u32,
}

/// 应用顶层配置：database 数据库层、data_sources 行情/数据源层。由 `config/` 目录下的 TOML 文件加载。
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
