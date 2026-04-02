use super::quote_collector::StockInfo;
use crate::core::Result;
use crate::sources::tdx::StockQuote;
use std::future::Future;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, warn};

pub(super) fn build_tdx_stock_codes(stocks: &[StockInfo]) -> Vec<(u16, String)> {
    stocks
        .iter()
        .map(|stock| (stock.market as u16, stock.code.clone()))
        .collect()
}

pub(super) fn build_tdx_stock_code_refs(
    stock_codes: &[(u16, String)],
) -> Vec<(u16, &str)> {
    stock_codes
        .iter()
        .map(|(market, code)| (*market, code.as_str()))
        .collect()
}

pub(super) fn stock_batches<'a>(
    stocks: &'a [StockInfo],
    batch_size: usize,
) -> Vec<&'a [StockInfo]> {
    stocks.chunks(batch_size).collect()
}

pub(super) async fn await_collect_quotes<F>(
    future: F,
    timeout_secs: u64,
) -> Result<Vec<StockQuote>>
where
    F: Future<Output = Result<Vec<StockQuote>>>,
{
    timeout(Duration::from_secs(timeout_secs), future)
        .await
        .map_err(|_| {
            warn!("采集行情超时（超过 {} 秒）", timeout_secs);
            crate::core::QuantixError::Timeout(format!("采集超时（超过 {} 秒）", timeout_secs))
        })?
        .map_err(|error| {
            warn!("采集行情失败: {}", error);
            error
        })
}

pub(super) async fn collect_batches<'a, F, Fut>(
    batches: Vec<&'a [StockInfo]>,
    inter_batch_delay: Duration,
    mut collect_batch: F,
) -> Vec<StockQuote>
where
    F: FnMut(&'a [StockInfo]) -> Fut,
    Fut: Future<Output = Result<Vec<StockQuote>>>,
{
    let total_batches = batches.len();
    let mut all_quotes = Vec::new();

    for (index, batch) in batches.into_iter().enumerate() {
        info!(
            "正在采集第 {}/{} 批（{} 只股票）",
            index + 1,
            total_batches,
            batch.len()
        );

        match collect_batch(batch).await {
            Ok(quotes) => {
                all_quotes.extend(quotes);
                debug!("第 {}/{} 批采集完成", index + 1, total_batches);
            }
            Err(error) => {
                warn!(
                    "第 {}/{} 批采集失败: {}, 跳过该批次",
                    index + 1,
                    total_batches,
                    error
                );
            }
        }

        if index + 1 < total_batches {
            tokio::time::sleep(inter_batch_delay).await;
        }
    }

    all_quotes
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[derive(Clone, Copy)]
    struct QuoteFixtureConfig<'a> {
        code: &'a str,
        name: &'a str,
        price: f64,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        volume: f64,
        amount: f64,
        market: u8,
    }

    impl Default for QuoteFixtureConfig<'_> {
        fn default() -> Self {
            Self {
                code: "000001",
                name: "平安银行",
                price: 10.5,
                preclose: 10.0,
                open: 10.2,
                high: 10.6,
                low: 10.1,
                volume: 1000.0,
                amount: 10500.0,
                market: 0,
            }
        }
    }

    fn build_quote(config: &QuoteFixtureConfig<'_>) -> StockQuote {
        StockQuote::from_tdx(
            config.code.to_string(),
            config.name.to_string(),
            config.price,
            config.preclose,
            config.open,
            config.high,
            config.low,
            config.volume,
            config.amount,
            config.market,
        )
    }

    fn build_quote_for_stock(stock: &StockInfo) -> StockQuote {
        build_quote(&QuoteFixtureConfig {
            code: stock.code.as_str(),
            name: stock.name.as_str(),
            market: stock.market,
            ..QuoteFixtureConfig::default()
        })
    }

    fn stock(code: &str, market: u8) -> StockInfo {
        StockInfo {
            code: code.to_string(),
            name: code.to_string(),
            market,
        }
    }

    #[test]
    fn build_tdx_stock_code_refs_preserves_market_and_code() {
        let owned = build_tdx_stock_codes(&[stock("000001", 0), stock("600000", 1)]);
        let refs = build_tdx_stock_code_refs(&owned);

        assert_eq!(refs, vec![(0, "000001"), (1, "600000")]);
    }

    #[test]
    fn stock_batches_respects_batch_size() {
        let stocks = vec![
            stock("000001", 0),
            stock("000002", 0),
            stock("000003", 0),
        ];

        let batches = stock_batches(&stocks, 2);

        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 2);
        assert_eq!(batches[1].len(), 1);
    }

    #[tokio::test]
    async fn await_collect_quotes_returns_quotes_when_future_completes() {
        let config = QuoteFixtureConfig::default();
        let future = async move {
            Ok::<Vec<StockQuote>, crate::core::QuantixError>(vec![build_quote(&config)])
        };

        let quotes = await_collect_quotes(future, 1).await.unwrap();

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].code, config.code);
        assert_eq!(quotes[0].market, config.market);
    }

    #[tokio::test]
    async fn await_collect_quotes_returns_timeout_error_when_future_takes_too_long() {
        let future = async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<Vec<StockQuote>, crate::core::QuantixError>(Vec::new())
        };

        let error = await_collect_quotes(future, 0).await.unwrap_err();

        assert!(matches!(error, crate::core::QuantixError::Timeout(_)));
    }

    #[tokio::test]
    async fn collect_batches_merges_quotes_from_successful_batches() {
        let stocks = vec![
            stock("000001", 0),
            stock("000002", 0),
            stock("600000", 1),
        ];
        let batches = stock_batches(&stocks, 2);

        let quotes = collect_batches(batches, Duration::from_millis(0), |batch| async move {
            Ok::<Vec<StockQuote>, crate::core::QuantixError>(
                batch.iter().map(build_quote_for_stock).collect(),
            )
        })
        .await;

        assert_eq!(quotes.len(), 3);
        assert_eq!(quotes[0].code, "000001");
        assert_eq!(quotes[2].code, "600000");
    }

    #[tokio::test]
    async fn collect_batches_skips_failed_batches_and_continues() {
        let stocks = vec![
            stock("000001", 0),
            stock("000002", 0),
            stock("600000", 1),
        ];
        let batches = stock_batches(&stocks, 1);

        let quotes = collect_batches(batches, Duration::from_millis(0), |batch| async move {
            if batch[0].code == "000002" {
                return Err(crate::core::QuantixError::DataSource(
                    "expected test failure".to_string(),
                ));
            }

            Ok::<Vec<StockQuote>, crate::core::QuantixError>(
                batch.iter().map(build_quote_for_stock).collect(),
            )
        })
        .await;

        let codes: Vec<_> = quotes.iter().map(|quote| quote.code.as_str()).collect();
        assert_eq!(codes, vec!["000001", "600000"]);
    }
}
