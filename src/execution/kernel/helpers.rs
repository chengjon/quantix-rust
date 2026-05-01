use crate::execution::models::FillDetails;
use crate::strategy::trait_def::Signal;

pub(super) fn signal_to_str(signal: Signal) -> &'static str {
    match signal {
        Signal::Buy => "buy",
        Signal::Sell => "sell",
        Signal::Hold => "hold",
    }
}

pub(super) fn fill_details_json(fill_details: Option<&FillDetails>) -> serde_json::Value {
    match fill_details {
        Some(fill) => serde_json::json!({
            "fill_id": fill.fill_id,
            "fill_quantity": fill.fill_quantity,
            "fill_price": fill.fill_price,
        }),
        None => serde_json::Value::Null,
    }
}
