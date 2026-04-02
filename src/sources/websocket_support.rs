use chrono::Utc;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::sources::websocket::RealtimeQuote;

pub(super) fn build_subscribe_message(codes: &[String]) -> Message {
    let subscribe_msg = serde_json::json!({
        "cmd": "sub",
        "data": codes
    });

    Message::Text(subscribe_msg.to_string())
}

pub(super) fn parse_realtime_quote_message(text: &str) -> Option<RealtimeQuote> {
    let data = serde_json::from_str::<serde_json::Value>(text).ok()?;
    let obj = data.as_object()?;
    let code = obj.get("code").and_then(|value| value.as_str())?;

    Some(RealtimeQuote {
        code: code.to_string(),
        name: obj
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        price: obj.get("price").and_then(|value| value.as_f64()).unwrap_or(0.0),
        preclose: obj
            .get("preclose")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        open: obj.get("open").and_then(|value| value.as_f64()).unwrap_or(0.0),
        high: obj.get("high").and_then(|value| value.as_f64()).unwrap_or(0.0),
        low: obj.get("low").and_then(|value| value.as_f64()).unwrap_or(0.0),
        volume: obj.get("volume").and_then(|value| value.as_i64()).unwrap_or(0),
        amount: obj
            .get("amount")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        change_percent: obj
            .get("change_percent")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        bid1: obj.get("bid1").and_then(|value| value.as_f64()),
        ask1: obj.get("ask1").and_then(|value| value.as_f64()),
        timestamp: Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SubscribeFixtureConfig<'a> {
        primary_code: &'a str,
        secondary_code: &'a str,
    }

    impl Default for SubscribeFixtureConfig<'_> {
        fn default() -> Self {
            Self {
                primary_code: "000001",
                secondary_code: "600000",
            }
        }
    }

    struct RealtimeQuotePayloadFixtureConfig<'a> {
        code: &'a str,
        name: &'a str,
        price: f64,
        preclose: f64,
        open: f64,
        high: f64,
        low: f64,
        volume: i64,
        amount: f64,
        change_percent: f64,
        bid1: f64,
        ask1: f64,
    }

    impl Default for RealtimeQuotePayloadFixtureConfig<'_> {
        fn default() -> Self {
            Self {
                code: "000001",
                name: "平安银行",
                price: 10.5,
                preclose: 10.0,
                open: 10.2,
                high: 10.6,
                low: 10.1,
                volume: 1000,
                amount: 10500.0,
                change_percent: 5.0,
                bid1: 10.49,
                ask1: 10.51,
            }
        }
    }

    fn build_codes(config: &SubscribeFixtureConfig<'_>) -> Vec<String> {
        vec![
            config.primary_code.to_string(),
            config.secondary_code.to_string(),
        ]
    }

    fn build_quote_payload(config: &RealtimeQuotePayloadFixtureConfig<'_>) -> String {
        serde_json::json!({
            "code": config.code,
            "name": config.name,
            "price": config.price,
            "preclose": config.preclose,
            "open": config.open,
            "high": config.high,
            "low": config.low,
            "volume": config.volume,
            "amount": config.amount,
            "change_percent": config.change_percent,
            "bid1": config.bid1,
            "ask1": config.ask1,
        })
        .to_string()
    }

    #[test]
    fn build_subscribe_message_encodes_codes_as_json_text() {
        let config = SubscribeFixtureConfig::default();
        let message = build_subscribe_message(&build_codes(&config));
        let Message::Text(text) = message else {
            panic!("expected text message");
        };

        assert!(text.contains("\"cmd\":\"sub\""));
        assert!(text.contains(config.primary_code));
        assert!(text.contains(config.secondary_code));
    }

    #[test]
    fn parse_realtime_quote_message_extracts_expected_fields() {
        let config = RealtimeQuotePayloadFixtureConfig::default();
        let text = build_quote_payload(&config);
        let quote = parse_realtime_quote_message(&text).expect("expected quote");

        assert_eq!(quote.code, config.code);
        assert_eq!(quote.name, config.name);
        assert_eq!(quote.price, config.price);
        assert_eq!(quote.bid1, Some(config.bid1));
        assert_eq!(quote.ask1, Some(config.ask1));
    }
}
