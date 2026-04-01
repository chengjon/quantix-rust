use chrono::Utc;
use std::collections::HashMap;

use super::eastmoney::{MoneyFlowData, Quote, StockInfo};

pub(super) fn stock_list_params() -> [(&'static str, &'static str); 9] {
    [
        ("pn", "1"),
        ("pz", "5000"),
        ("po", "1"),
        ("np", "1"),
        ("fltt", "2"),
        ("invt", "2"),
        ("fs", "m:0+t:6,m:0+t:80,m:1+t:2,m:1+t:23"),
        ("fid", "f3"),
        ("fields", "f12,f13,f14,f2,f3,f4,f5,f6"),
    ]
}

pub(super) fn build_realtime_quote_params(
    codes: &[String],
) -> Vec<(&'static str, String)> {
    vec![
        ("fltt", "2".to_string()),
        ("invt", "2".to_string()),
        ("secids", build_realtime_secids(codes)),
        ("fields", "f12,f13,f14,f2,f3,f4,f5,f6".to_string()),
    ]
}

pub(super) fn money_flow_params(code: &str) -> Vec<(&'static str, String)> {
    vec![
        ("lmt", "0".to_string()),
        ("klt", "1".to_string()),
        ("secid", format_eastmoney_secid(code)),
        ("fields1", "f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12,f13".to_string()),
        ("fields2", "f62,f63,f64,f65".to_string()),
        ("ut", "fa5fd1943c7b386f172d6893d2bcdbd".to_string()),
    ]
}

pub(super) fn parse_stock_list_placeholder() -> Vec<StockInfo> {
    Vec::new()
}

pub(super) fn parse_realtime_quotes_placeholder(
    codes: &[String],
) -> HashMap<String, Quote> {
    codes.iter()
        .map(|code| {
            (
                code.clone(),
                Quote {
                    code: code.clone(),
                    name: String::new(),
                    price: 0.0,
                    change: 0.0,
                    change_pct: 0.0,
                    volume: 0.0,
                    amount: 0.0,
                    high: 0.0,
                    low: 0.0,
                    open: 0.0,
                    preclose: 0.0,
                },
            )
        })
        .collect()
}

pub(super) fn parse_money_flow_placeholder() -> MoneyFlowData {
    MoneyFlowData {
        code: String::new(),
        date: Utc::now().date_naive(),
        main_in: 0.0,
        main_out: 0.0,
        retail_in: 0.0,
        retail_out: 0.0,
        main_net: 0.0,
    }
}

fn build_realtime_secids(codes: &[String]) -> String {
    codes
        .iter()
        .map(|code| format!("f{}", code))
        .collect::<Vec<_>>()
        .join(",")
}

fn format_eastmoney_secid(code: &str) -> String {
    format!("{}.{}", if code.starts_with('6') { "1" } else { "0" }, code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_realtime_quote_params_joins_codes_in_expected_format() {
        let params = build_realtime_quote_params(&["000001".to_string(), "600000".to_string()]);
        let secids = params
            .iter()
            .find(|(key, _)| *key == "secids")
            .map(|(_, value)| value.as_str())
            .unwrap();

        assert_eq!(secids, "f000001,f600000");
    }

    #[test]
    fn money_flow_params_formats_secid_by_market_prefix() {
        let sh = money_flow_params("600000");
        let sz = money_flow_params("000001");

        assert_eq!(sh[2], ("secid", "1.600000".to_string()));
        assert_eq!(sz[2], ("secid", "0.000001".to_string()));
    }
}
