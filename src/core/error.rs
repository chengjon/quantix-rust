/// 核心错误类型
///
/// 统一的错误处理，便于与 Python 端错误信息对应
use thiserror::Error;

/// 项目统一错误类型：覆盖配置/数据库/数据源/解析/网络/HTTP/序列化/算法等类别。所有 `?` 错误都汇聚到此类型，便于跨模块传递与 CLI 输出。
#[derive(Error, Debug)]
pub enum QuantixError {
    #[error("配置错误: {0}")]
    Config(String),

    #[error("数据库连接失败: {0}")]
    DatabaseConnection(String),

    #[error("数据库查询失败: {0}")]
    DatabaseQuery(String),

    #[error("数据源错误: {0}")]
    DataSource(String),

    #[error("数据解析错误: {0}")]
    DataParse(String),

    #[error("超时错误: {0}")]
    Timeout(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SQLx 错误: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("网络错误: {0}")]
    Network(String),

    #[error("功能暂不支持: {0}")]
    Unsupported(String),

    #[error("其他错误: {0}")]
    Other(String),

    #[error("算法错误: {0}")]
    Algo(String),
}

/// 项目统一 Result 类型别名：错误固定为 `QuantixError`，所有 fallible 公共 API 返回此类型。
pub type Result<T> = std::result::Result<T, QuantixError>;

impl From<crate::execution::algo::AlgoError> for QuantixError {
    fn from(err: crate::execution::algo::AlgoError) -> Self {
        QuantixError::Algo(err.to_string())
    }
}
