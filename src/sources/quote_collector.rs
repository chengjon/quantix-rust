/// 行情采集器
///
/// 从短线侠项目迁移，支持批量采集全市场实时行情

use crate::core::Result;
use crate::sources::tdx::{StockQuote, TdxSource};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{debug, info, warn};

/// 股票基本信息（用于采集）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub market: u8, // 0=深圳, 1=上海
}

/// 并发行情采集器
pub struct QuoteCollector {
    /// TDX 数据源
    tdx_source: Arc<TdxSource>,
    /// 每批采集的股票数量
    batch_size: usize,
    /// 采集超时时间（秒）
    collect_timeout: u64,
}

impl QuoteCollector {
    /// 创建新的行情采集器
    ///
    /// ## 参数
    /// - `tdx_source`: TDX 数据源
    /// - `batch_size`: 每批采集的股票数量（建议 800）
    /// - `collect_timeout`: 每批采集超时时间（秒，建议 10）
    pub fn new(tdx_source: TdxSource, batch_size: usize, collect_timeout: u64) -> Self {
        info!(
            "行情采集器初始化：每批 {} 只股票，超时 {} 秒",
            batch_size,
            collect_timeout
        );

        Self {
            tdx_source: Arc::new(tdx_source),
            batch_size,
            collect_timeout,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Result<Self> {
        let tdx_source = TdxSource::with_default_config()?;
        Ok(Self::new(tdx_source, 800, 10))
    }

    /// 采集单批股票的实时行情
    ///
    /// ## 参数
    /// - `stocks`: 股票列表
    ///
    /// ## 返回
    /// 返回采集到的行情数据列表
    pub async fn collect_batch(&self, stocks: &[StockInfo]) -> Result<Vec<StockQuote>> {
        if stocks.is_empty() {
            return Ok(Vec::new());
        }

        debug!("开始采集 {} 只股票的实时行情", stocks.len());

        // 将股票代码转换为 TDX 格式
        let stock_codes: Vec<(u16, String)> = stocks
            .iter()
            .map(|s| (s.market as u16, s.code.clone()))
            .collect();

        // 转换为引用用于 TDX 调用
        let stock_codes_ref: Vec<(u16, &str)> = stock_codes
            .iter()
            .map(|(m, c)| (*m, c.as_str()))
            .collect();

        // 使用超时包装整个操作
        let result = timeout(
            Duration::from_secs(self.collect_timeout),
            self.tdx_source.fetch_quotes_batch(&stock_codes_ref),
        )
        .await
        .map_err(|_| {
            warn!("采集行情超时（超过 {} 秒）", self.collect_timeout);
            crate::core::QuantixError::Timeout(format!(
                "采集超时（超过 {} 秒）",
                self.collect_timeout
            ))
        })?
        .map_err(|e| {
            warn!("采集行情失败: {}", e);
            e
        })?;

        debug!("成功采集 {} 只股票的实时行情", result.len());
        Ok(result)
    }

    /// 将股票列表分批采集
    ///
    /// ## 参数
    /// - `stocks`: 所有股票列表
    ///
    /// ## 返回
    /// 返回所有批次采集的行情数据
    pub async fn collect_all(&self, stocks: &[StockInfo]) -> Result<Vec<StockQuote>> {
        if stocks.is_empty() {
            return Ok(Vec::new());
        }

        info!("开始分批采集全市场 {} 只股票的实时行情", stocks.len());

        // 将股票分批
        let batches: Vec<&[StockInfo]> = stocks.chunks(self.batch_size).collect();
        let total_batches = batches.len();
        let mut all_quotes = Vec::new();

        for (i, batch) in batches.iter().enumerate() {
            info!(
                "正在采集第 {}/{} 批（{} 只股票）",
                i + 1,
                total_batches,
                batch.len()
            );

            match self.collect_batch(batch).await {
                Ok(quotes) => {
                    all_quotes.extend(quotes);
                    debug!("第 {}/{} 批采集完成", i + 1, total_batches);
                }
                Err(e) => {
                    warn!(
                        "第 {}/{} 批采集失败: {}, 跳过该批次",
                        i + 1,
                        total_batches,
                        e
                    );
                    // 继续采集下一批，不中断整个流程
                }
            }

            // 避免请求过快被封 IP
            if i < total_batches - 1 {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        info!(
            "全市场行情采集完成：共获取 {} 只股票的行情数据",
            all_quotes.len()
        );

        Ok(all_quotes)
    }

    /// 获取批量大小
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// 设置批量大小
    pub fn set_batch_size(&mut self, size: usize) {
        self.batch_size = size;
        info!("批量大小已更新为: {}", size);
    }
}

/// 默认实现
impl Default for QuoteCollector {
    fn default() -> Self {
        Self::with_default_config().expect("无法创建默认行情采集器")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_collector_creation() {
        let collector = QuoteCollector::with_default_config();
        assert!(collector.is_ok());
    }

    #[test]
    fn test_quote_collector_batch_size() {
        let tdx_source = TdxSource::with_default_config().unwrap();
        let collector = QuoteCollector::new(tdx_source, 100, 5);
        assert_eq!(collector.batch_size(), 100);
    }

    #[tokio::test]
    async fn test_collect_empty_batch() {
        let collector = QuoteCollector::with_default_config().unwrap();
        let result = collector.collect_batch(&[]).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
