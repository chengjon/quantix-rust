use super::quote_collector::StockInfo;
use crate::core::Result;
use crate::sources::tdx::StockQuote;
use std::future::Future;
use tokio::time::{Duration, timeout};
use tracing::warn;

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
}
