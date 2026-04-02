use crate::core::Result;
use crate::sources::tdx::StockQuote;
use rustdx_complete::tcp::Tcp;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{debug, warn};

pub(super) type OwnedCodes = Vec<(u16, String)>;
pub(super) type RawQuoteTuple = (String, String, f64, f64, f64, f64, f64, f64, f64);
pub(super) type SharedTcp = Arc<Mutex<Tcp>>;

pub(super) fn build_tcp_pool(pool_size: usize) -> Result<Vec<SharedTcp>> {
    let mut tcp_pool = Vec::new();

    for index in 0..pool_size {
        match Tcp::new() {
            Ok(tcp) => {
                tcp_pool.push(Arc::new(Mutex::new(tcp)));
                debug!("TDX TCP 连接 #{} 创建成功", index);
            }
            Err(error) => {
                warn!("TDX TCP 连接 #{} 创建失败: {}", index, error);
                if tcp_pool.is_empty() {
                    return Err(crate::core::QuantixError::DataSource(format!(
                        "无法创建任何 TCP 连接: {}",
                        error
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

    Ok(tcp_pool)
}

pub(super) fn build_owned_codes(codes: &[(u16, &str)]) -> OwnedCodes {
    codes.iter().map(|(market, code)| (*market, code.to_string())).collect()
}

pub(super) fn build_code_refs(owned_codes: &OwnedCodes) -> Vec<(u16, &str)> {
    owned_codes
        .iter()
        .map(|(market, code)| (*market, code.as_str()))
        .collect()
}

pub(super) fn next_pool_index(counter: &AtomicUsize, pool_len: usize) -> usize {
    counter.fetch_add(1, Ordering::Relaxed).wrapping_rem(pool_len)
}

pub(super) fn map_raw_quotes(rows: Vec<RawQuoteTuple>) -> Vec<StockQuote> {
    rows.into_iter()
        .map(
            |(code, name, price, preclose, open, high, low, volume, amount)| {
                let market = if code.starts_with('6') { 1 } else { 0 };
                StockQuote::from_tdx(
                    code, name, price, preclose, open, high, low, volume, amount, market,
                )
            },
        )
        .collect()
}

pub(super) async fn await_quote_batch(
    handle: tokio::task::JoinHandle<Result<Vec<RawQuoteTuple>>>,
    timeout_secs: u64,
) -> Result<Vec<StockQuote>> {
    let rows = tokio::time::timeout(Duration::from_secs(timeout_secs), handle)
        .await
        .map_err(|_| {
            crate::core::QuantixError::Timeout(format!("采集超时（超过 {} 秒）", timeout_secs))
        })?
        .map_err(|error| {
            crate::core::QuantixError::DataSource(format!("任务执行失败: {}", error))
        })??;

    Ok(map_raw_quotes(rows))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::core::QuantixError;

    #[test]
    fn build_tcp_pool_creates_requested_connections() {
        let pool = build_tcp_pool(1).unwrap();
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn build_tcp_pool_rejects_zero_size_pool() {
        let error = build_tcp_pool(0).unwrap_err();
        assert!(matches!(error, QuantixError::DataSource(_)));
    }

    #[test]
    fn build_code_refs_preserves_market_and_order() {
        let owned = build_owned_codes(&[(0, "000001"), (1, "600000")]);
        let refs = build_code_refs(&owned);
        assert_eq!(refs, vec![(0, "000001"), (1, "600000")]);
    }

    #[test]
    fn map_raw_quotes_derives_market_from_code_prefix() {
        let rows = vec![
            (
                "600000".to_string(),
                "浦发银行".to_string(),
                10.5,
                10.0,
                10.2,
                10.6,
                10.1,
                1000.0,
                10500.0,
            ),
            (
                "000001".to_string(),
                "平安银行".to_string(),
                12.0,
                11.5,
                11.8,
                12.1,
                11.7,
                800.0,
                9600.0,
            ),
        ];

        let quotes = map_raw_quotes(rows);
        assert_eq!(quotes[0].market, 1);
        assert_eq!(quotes[1].market, 0);
    }

    #[tokio::test]
    async fn await_quote_batch_maps_rows_after_join_completion() {
        let rows = vec![(
            "600000".to_string(),
            "浦发银行".to_string(),
            10.5,
            10.0,
            10.2,
            10.6,
            10.1,
            1000.0,
            10500.0,
        )];
        let handle = tokio::spawn(async move { Ok(rows) });

        let quotes = await_quote_batch(handle, 1).await.unwrap();

        assert_eq!(quotes.len(), 1);
        assert_eq!(quotes[0].code, "600000");
        assert_eq!(quotes[0].market, 1);
    }

    #[tokio::test]
    async fn await_quote_batch_returns_timeout_error_when_task_exceeds_limit() {
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            Ok::<Vec<RawQuoteTuple>, crate::core::QuantixError>(Vec::new())
        });

        let error = await_quote_batch(handle, 0).await.unwrap_err();

        assert!(matches!(error, QuantixError::Timeout(_)));
    }

    #[tokio::test]
    async fn await_quote_batch_wraps_join_failure_as_data_source_error() {
        let handle = tokio::spawn(async move {
            panic!("join failure");
            #[allow(unreachable_code)]
            Ok::<Vec<RawQuoteTuple>, crate::core::QuantixError>(Vec::new())
        });

        let error = await_quote_batch(handle, 1).await.unwrap_err();

        assert!(matches!(error, QuantixError::DataSource(_)));
    }

    #[test]
    fn next_pool_index_round_robins_across_pool_length() {
        let counter = AtomicUsize::new(0);

        let indexes = [
            next_pool_index(&counter, 3),
            next_pool_index(&counter, 3),
            next_pool_index(&counter, 3),
            next_pool_index(&counter, 3),
        ];

        assert_eq!(indexes, [0, 1, 2, 0]);
    }

    #[test]
    fn next_pool_index_preserves_wrapping_behavior_from_existing_counter_state() {
        let counter = AtomicUsize::new(usize::MAX);

        let index = next_pool_index(&counter, 4);

        assert_eq!(index, 3);
    }
}
