#![allow(clippy::too_many_arguments, clippy::type_complexity)]

/// TDX (通达信) 数据源
///
/// 从短线侠项目迁移，使用 rustdx-complete 连接通达信服务器
/// 支持实时行情采集
use crate::core::Result;
use crate::data::models::{Kline, StockInfo};
use async_trait::async_trait;
use chrono::Utc;
use rustdx_complete::tcp::stock::SecurityQuotes;
use rustdx_complete::tcp::{Tcp, Tdx};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::data::fetcher::Fetcher;

/// 股票实时行情数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StockQuote {
    /// Unix 时间戳（秒）
    pub timestamp: u64,
    /// 股票代码
    pub code: String,
    /// 股票名称
    pub name: String,
    /// 当前价
    pub price: f64,
    /// 昨收价
    pub preclose: f64,
    /// 今开价
    pub open: f64,
    /// 最高价
    pub high: f64,
    /// 最低价
    pub low: f64,
    /// 成交量（手）
    pub volume: f64,
    /// 成交额（元）
    pub amount: f64,
    /// 涨跌幅(%)
    pub change_percent: f64,
    /// 市场标识 (0=深圳, 1=上海)
    pub market: u8,
}

impl StockQuote {
    /// 计算涨跌幅
    pub fn calculate_change_percent(price: f64, preclose: f64) -> f64 {
        if preclose > 0.0 {
            ((price - preclose) / preclose) * 100.0
        } else {
            0.0
        }
    }

    /// 从 TDX 行情数据创建
    pub fn from_tdx(
        code: String,
        name: String,
        price: f64,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount: f64,
        market: u8,
    ) -> Self {
        let change_percent = Self::calculate_change_percent(price, preclose);

        Self {
            timestamp: Utc::now().timestamp() as u64,
            code,
            name,
            price,
            preclose,
            open,
            high,
            low,
            volume,
            amount,
            change_percent,
            market,
        }
    }
}

/// 通达信数据源（完整版，支持实时行情采集）
pub struct TdxSource {
    /// TCP 连接池
    tcp_pool: Vec<Arc<std::sync::Mutex<Tcp>>>,
    /// 连接池索引（轮询选择）
    connection_index: Arc<AtomicUsize>,
    /// 配置参数
    _hosts: Vec<String>,
    _port: u16,
    timeout: u64,
}

impl TdxSource {
    /// 创建新的 TDX 数据源
    ///
    /// ## 参数
    /// - `pool_size`: TCP 连接池大小（建议 3-5）
    /// - `hosts`: TDX 服务器主机列表（可选，默认使用标准服务器）
    /// - `port`: TDX 端口（默认 7709）
    /// - `timeout`: 超时时间（秒）
    pub fn new(pool_size: usize, hosts: Vec<String>, port: u16, timeout: u64) -> Result<Self> {
        let mut tcp_pool = Vec::new();

        for i in 0..pool_size {
            match Tcp::new() {
                Ok(tcp) => {
                    tcp_pool.push(Arc::new(std::sync::Mutex::new(tcp)));
                    debug!("TDX TCP 连接 #{} 创建成功", i);
                }
                Err(e) => {
                    warn!("TDX TCP 连接 #{} 创建失败: {}", i, e);
                    // 至少需要一个连接
                    if tcp_pool.is_empty() {
                        return Err(crate::core::QuantixError::DataSource(format!(
                            "无法创建任何 TCP 连接: {}",
                            e
                        )));
                    }
                }
            }
        }

        if tcp_pool.is_empty() {
            return Err(crate::core::QuantixError::DataSource(
                "无法创建任何 TCP 连接".to_string(),
            ));
        }

        info!("TDX 数据源初始化成功：{} 个 TCP 连接", tcp_pool.len());

        Ok(Self {
            tcp_pool,
            connection_index: Arc::new(AtomicUsize::new(0)),
            _hosts: hosts,
            _port: port,
            timeout,
        })
    }

    /// 使用默认配置创建 TDX 数据源
    pub fn with_default_config() -> Result<Self> {
        Self::new(3, vec![], 7709, 10)
    }

    /// 从连接池获取连接（轮询方式）
    fn get_connection(&self) -> Arc<std::sync::Mutex<Tcp>> {
        let index = self
            .connection_index
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_rem(self.tcp_pool.len());

        self.tcp_pool[index].clone()
    }

    /// 采集单批股票的实时行情
    ///
    /// ## 参数
    /// - `codes`: 股票代码列表，格式：[(market, code), ...]
    ///   market: 0=深圳, 1=上海
    ///
    /// ## 返回
    /// 返回采集到的行情数据列表
    pub async fn fetch_quotes_batch(&self, codes: &[(u16, &str)]) -> Result<Vec<StockQuote>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        debug!("开始采集 {} 只股票的实时行情", codes.len());

        // 转换为 Owned String 以避免生命周期问题
        let codes_owned: Vec<(u16, String)> =
            codes.iter().map(|(m, c)| (*m, c.to_string())).collect();

        // 从连接池获取连接
        let tcp = self.get_connection();

        // 使用 spawn_blocking 执行阻塞的 TDX I/O
        let handle: JoinHandle<Result<Vec<(String, String, f64, f64, f64, f64, f64, f64, f64)>>> =
            tokio::task::spawn_blocking(move || {
                // 转换为引用
                let codes_ref: Vec<(u16, &str)> =
                    codes_owned.iter().map(|(m, c)| (*m, c.as_str())).collect();

                let mut tcp_guard = tcp.lock().map_err(|e| {
                    crate::core::QuantixError::DataSource(format!("无法获取 TCP 锁: {}", e))
                })?;

                let mut quotes = SecurityQuotes::new(codes_ref);
                quotes.recv_parsed(&mut tcp_guard).map_err(|e| {
                    crate::core::QuantixError::DataSource(format!("TDX 接收失败: {}", e))
                })?;

                // 提取行情数据
                let result: Vec<(String, String, f64, f64, f64, f64, f64, f64, f64)> = quotes
                    .result()
                    .iter()
                    .map(|q| {
                        (
                            q.code.clone(),
                            q.name.clone(),
                            q.price,
                            q.preclose,
                            q.open,
                            q.high,
                            q.low,
                            q.vol,
                            q.amount,
                        )
                    })
                    .collect();

                Ok(result)
            });

        // 等待任务完成，带超时
        let timeout_result = tokio::time::timeout(Duration::from_secs(self.timeout), handle)
            .await
            .map_err(|_| {
                crate::core::QuantixError::Timeout(format!("采集超时（超过 {} 秒）", self.timeout))
            })?
            .map_err(|e| crate::core::QuantixError::DataSource(format!("任务执行失败: {}", e)))??;

        // 转换为 StockQuote
        let quotes: Vec<StockQuote> = timeout_result
            .into_iter()
            .map(
                |(code, name, price, preclose, open, high, low, volume, amount)| {
                    // 判断市场：6开头是上海，其他是深圳
                    let market = if code.starts_with('6') { 1 } else { 0 };
                    StockQuote::from_tdx(
                        code, name, price, preclose, open, high, low, volume, amount, market,
                    )
                },
            )
            .collect();

        debug!("成功采集 {} 只股票的实时行情", quotes.len());
        Ok(quotes)
    }
}

#[async_trait]
impl Fetcher for TdxSource {
    async fn get_stock_info(&self, _code: &str) -> Result<Option<StockInfo>> {
        Err(crate::core::QuantixError::Unsupported(
            "TdxSource::get_stock_info 尚未接入真实股票信息来源".to_string(),
        ))
    }

    async fn get_kline(
        &self,
        _code: &str,
        _start: chrono::NaiveDate,
        _end: chrono::NaiveDate,
    ) -> Result<Vec<Kline>> {
        Err(crate::core::QuantixError::Unsupported(
            "TdxSource::get_kline 尚未接入真实 K 线来源".to_string(),
        ))
    }

    async fn check_connection(&self) -> Result<()> {
        // 尝试创建一个测试连接
        match Tcp::new() {
            Ok(_) => {
                info!("TDX 连接检查成功");
                Ok(())
            }
            Err(e) => Err(crate::core::QuantixError::DataSource(format!(
                "TDX 连接失败: {}",
                e
            ))),
        }
    }
}

/// 默认实现：使用标准配置
impl Default for TdxSource {
    fn default() -> Self {
        Self::with_default_config().expect("无法创建默认 TDX 数据源")
    }
}

#[cfg(test)]
pub(crate) fn offline_tdx_source() -> TdxSource {
    TdxSource::new(1, vec![], 7709, 10).expect("offline TDX source config should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::QuantixError;

    #[test]
    fn test_tdx_source_creation() {
        let source = TdxSource::with_default_config();
        assert!(source.is_ok());
    }

    #[test]
    fn test_stock_quote_calculate_change_percent() {
        let change = StockQuote::calculate_change_percent(10.5, 10.0);
        assert!((change - 5.0).abs() < 0.01);

        let change = StockQuote::calculate_change_percent(9.5, 10.0);
        assert!((change - (-5.0)).abs() < 0.01);

        // 昨收价为0的情况
        let change = StockQuote::calculate_change_percent(10.0, 0.0);
        assert_eq!(change, 0.0);
    }

    #[test]
    fn test_stock_quote_from_tdx() {
        let quote = StockQuote::from_tdx(
            "000001".to_string(),
            "平安银行".to_string(),
            10.5,
            10.0,
            10.2,
            10.6,
            10.1,
            100000.0,
            1050000.0,
            0,
        );

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.name, "平安银行");
        assert_eq!(quote.price, 10.5);
        assert_eq!(quote.preclose, 10.0);
        assert!((quote.change_percent - 5.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_tdx_get_stock_info_returns_unsupported() {
        let source = offline_tdx_source();
        let err = source.get_stock_info("000001").await.unwrap_err();
        assert!(matches!(err, QuantixError::Unsupported(_)));
    }

    #[tokio::test]
    async fn test_tdx_get_kline_returns_unsupported() {
        let source = offline_tdx_source();
        let err = source
            .get_kline(
                "000001",
                chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, QuantixError::Unsupported(_)));
    }
}
