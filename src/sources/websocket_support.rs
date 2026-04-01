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

    #[test]
    fn build_subscribe_message_encodes_codes_as_json_text() {
        let message = build_subscribe_message(&["000001".to_string(), "600000".to_string()]);
        let Message::Text(text) = message else {
            panic!("expected text message");
        };

        assert!(text.contains("\"cmd\":\"sub\""));
        assert!(text.contains("000001"));
        assert!(text.contains("600000"));
    }

    #[test]
    fn parse_realtime_quote_message_extracts_expected_fields() {
        let text = r#"{"code":"000001","name":"平安银行","price":10.5,"preclose":10.0,"open":10.2,"high":10.6,"low":10.1,"volume":1000,"amount":10500.0,"change_percent":5.0,"bid1":10.49,"ask1":10.51}"#;
        let quote = parse_realtime_quote_message(text).expect("expected quote");

        assert_eq!(quote.code, "000001");
        assert_eq!(quote.name, "平安银行");
        assert_eq!(quote.price, 10.5);
        assert_eq!(quote.bid1, Some(10.49));
        assert_eq!(quote.ask1, Some(10.51));
    }
}
