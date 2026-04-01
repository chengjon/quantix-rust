use super::auction_collector::{AuctionQuote, WatchlistStock};

pub(super) fn calculate_sealed_amount(
    buy1_price: f64,
    buy1_volume: u64,
    sell1_price: f64,
    sell1_volume: u64,
) -> (f64, f64) {
    let sealed_buy = buy1_price * buy1_volume as f64;
    let sealed_sell = sell1_price * sell1_volume as f64;
    (sealed_buy, sealed_sell)
}

pub(super) fn calculate_strength_score(quote: &AuctionQuote) -> f32 {
    let price_rise = quote.change_percent.max(0.0) as f32;

    let buy_ratio = if quote.buy1_volume + quote.sell1_volume > 0 {
        (quote.buy1_volume as f32) / ((quote.buy1_volume + quote.sell1_volume) as f32)
    } else {
        0.5
    };

    let volume_ratio = (quote.volume as f32 / 1_000_000.0).min(1.0);

    let score = (price_rise * 40.0) + (buy_ratio * 30.0) + (volume_ratio * 30.0);

    score.clamp(0.0, 100.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_auction_quote(
    stock: &WatchlistStock,
    time: String,
    price: f64,
    pre_close: f64,
    volume: u64,
    amount: f64,
    buy1_price: f64,
    buy1_volume: u64,
    sell1_price: f64,
    sell1_volume: u64,
) -> AuctionQuote {
    let (sealed_buy, sealed_sell) =
        calculate_sealed_amount(buy1_price, buy1_volume, sell1_price, sell1_volume);

    let change_percent = if pre_close > 0.0 {
        ((price - pre_close) / pre_close) * 100.0
    } else {
        0.0
    };

    let mut auction_quote = AuctionQuote {
        code: stock.code.clone(),
        name: stock.name.clone(),
        time,
        price,
        pre_close,
        volume,
        amount,
        buy1_price,
        buy1_volume,
        sell1_price,
        sell1_volume,
        change_percent,
        sealed_amount_buy: sealed_buy,
        sealed_amount_sell: sealed_sell,
        strength_score: 0.0,
    };

    auction_quote.strength_score = calculate_strength_score(&auction_quote);
    auction_quote
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_stock() -> WatchlistStock {
        WatchlistStock {
            code: "000001".to_string(),
            name: "平安银行".to_string(),
            market: 0,
        }
    }

    #[test]
    fn calculate_sealed_amount_matches_expected_values() {
        let (buy, sell) = calculate_sealed_amount(10.0, 1000, 10.5, 500);
        assert_eq!(buy, 10000.0);
        assert_eq!(sell, 5250.0);
    }

    #[test]
    fn build_auction_quote_computes_change_and_strength() {
        let quote = build_auction_quote(
            &sample_stock(),
            "2026-04-01 09:20:00".to_string(),
            10.5,
            10.0,
            200_000,
            2_100_000.0,
            10.48,
            150_000,
            10.5,
            50_000,
        );

        assert_eq!(quote.code, "000001");
        assert!((quote.change_percent - 5.0).abs() < f64::EPSILON);
        assert!(quote.strength_score > 0.0);
    }
}
