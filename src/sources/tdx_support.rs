use crate::sources::tdx::StockQuote;

pub(super) type OwnedCodes = Vec<(u16, String)>;
pub(super) type RawQuoteTuple = (String, String, f64, f64, f64, f64, f64, f64, f64);

pub(super) fn build_owned_codes(codes: &[(u16, &str)]) -> OwnedCodes {
    codes.iter().map(|(market, code)| (*market, code.to_string())).collect()
}

pub(super) fn build_code_refs(owned_codes: &OwnedCodes) -> Vec<(u16, &str)> {
    owned_codes
        .iter()
        .map(|(market, code)| (*market, code.as_str()))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
