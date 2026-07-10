use std::path::PathBuf;

/// 自选股清单 JSON 路径环境变量（覆盖默认 `$HOME/.quantix/watchlist.json`）。
pub const WATCHLIST_PATH_ENV: &str = "QUANTIX_WATCHLIST_PATH";
/// paper 交易账本 JSON 路径环境变量（覆盖默认 `$HOME/.quantix/trade.json`）。
pub const TRADE_PATH_ENV: &str = "QUANTIX_TRADE_PATH";
/// 风控状态 JSON 路径环境变量（覆盖默认 `$HOME/.quantix/risk.json`）。
pub const RISK_PATH_ENV: &str = "QUANTIX_RISK_PATH";
/// monitor SQLite 数据库路径环境变量。
pub const MONITOR_DB_PATH_ENV: &str = "QUANTIX_MONITOR_DB_PATH";
/// monitor YAML 配置路径环境变量。
pub const MONITOR_CONFIG_PATH_ENV: &str = "QUANTIX_MONITOR_CONFIG_PATH";
/// 策略配置 YAML 路径环境变量。
pub const STRATEGY_CONFIG_PATH_ENV: &str = "QUANTIX_STRATEGY_CONFIG_PATH";
/// 策略运行时 SQLite 数据库路径环境变量。
pub const STRATEGY_RUNTIME_DB_PATH_ENV: &str = "QUANTIX_STRATEGY_RUNTIME_DB_PATH";
/// 执行配置 JSON 路径环境变量。
pub const EXECUTION_CONFIG_PATH_ENV: &str = "QUANTIX_EXECUTION_CONFIG_PATH";
/// bridge base URL 环境变量（miniQMT bridge 服务地址）。
pub const BRIDGE_BASE_URL_ENV: &str = "QUANTIX_BRIDGE_BASE_URL";
/// bridge API key 环境变量（X-API-Key header 值）。
pub const BRIDGE_API_KEY_ENV: &str = "QUANTIX_BRIDGE_API_KEY";
/// bridge bearer token 环境变量（Authorization: Bearer，与 API key 二选一）。
pub const BRIDGE_BEARER_TOKEN_ENV: &str = "QUANTIX_BRIDGE_BEARER_TOKEN";
/// bridge 契约版本环境变量（默认 `miniqmt.v1`）。
pub const BRIDGE_CONTRACT_VERSION_ENV: &str = "QUANTIX_BRIDGE_CONTRACT_VERSION";
/// bridge HTTP 调用超时（毫秒）环境变量。
pub const BRIDGE_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_TIMEOUT_MS";
/// bridge 轮询间隔（毫秒）环境变量（用于异步任务查询）。
pub const BRIDGE_POLL_INTERVAL_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_INTERVAL_MS";
/// bridge 轮询总超时（毫秒）环境变量（超出则判定为任务超时）。
pub const BRIDGE_POLL_TIMEOUT_MS_ENV: &str = "QUANTIX_BRIDGE_POLL_TIMEOUT_MS";
/// bridge 默认 base URL（本机 miniQMT 服务端口）。
pub const DEFAULT_BRIDGE_BASE_URL: &str = "http://127.0.0.1:17580";
/// bridge 默认契约版本。
pub const DEFAULT_BRIDGE_CONTRACT_VERSION: &str = "miniqmt.v1";
/// bridge 默认 HTTP 调用超时（30 秒）。
pub const DEFAULT_BRIDGE_TIMEOUT_MS: u64 = 30_000;
/// bridge 默认轮询间隔（1 秒）。
pub const DEFAULT_BRIDGE_POLL_INTERVAL_MS: u64 = 1_000;
/// bridge 默认轮询总超时（30 秒）。
pub const DEFAULT_BRIDGE_POLL_TIMEOUT_MS: u64 = 30_000;
/// OpenStock base URL 环境变量（OpenStock 历史数据服务地址）。
pub const OPENSTOCK_BASE_URL_ENV: &str = "OPENSTOCK_BASE_URL";
/// OpenStock API key 环境变量。
pub const OPENSTOCK_API_KEY_ENV: &str = "OPENSTOCK_API_KEY";
/// OpenStock 默认 HTTP 调用超时（30 秒）。
pub const DEFAULT_OPENSTOCK_TIMEOUT_SECS: u64 = 30;

/// ClickHouse 连接配置：url host、database 名、user/password 凭据。从环境变量构造，用于行情/历史数据查询。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

/// 上游 MySQL（Tushare/proxy 层）连接配置：url host、database 名、user/password 凭据。用于基本面/历史数据回填。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpstreamMySqlSettings {
    pub url: String,
    pub database: String,
    pub user: String,
    pub password: String,
}

/// CLI 运行时聚合配置：clickhouse 行情库、bridge miniQMT 网关、upstream_mysql 上游数据源、openstock 历史数据源，以及 watchlist/trade/risk/monitor/strategy/execution 各子系统的存储路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliRuntime {
    pub clickhouse: ClickHouseSettings,
    pub bridge: BridgeRuntimeSettings,
    pub upstream_mysql: UpstreamMySqlSettings,
    pub openstock: OpenStockSettings,
    pub watchlist_path: PathBuf,
    pub trade_path: PathBuf,
    pub risk_path: PathBuf,
    pub monitor_db_path: PathBuf,
    pub monitor_config_path: PathBuf,
    pub strategy_config_path: PathBuf,
    pub strategy_runtime_db_path: PathBuf,
    pub execution_config_path: PathBuf,
}

/// bridge 运行时配置：base_url 服务地址、api_key/bearer_token/api_key_fallback 认证凭据（三选一）、contract_version 契约、timeout_ms HTTP 超时、poll_interval_ms/poll_timeout_ms 异步轮询参数、tdx_enabled 是否启用 TDX 桥接、qmt_preview_enabled 是否启用 QMT 预演。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeRuntimeSettings {
    pub base_url: String,
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub api_key_fallback: Option<String>,
    pub contract_version: String,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub poll_timeout_ms: u64,
    pub tdx_enabled: bool,
    pub qmt_preview_enabled: bool,
}

/// OpenStock 客户端配置：base_url 服务地址、api_key 认证凭据、timeout_secs HTTP 超时。base_url/api_key 可空（未配置则禁用 OpenStock 数据源）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenStockSettings {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}
