use super::quote_collector::StockInfo;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
