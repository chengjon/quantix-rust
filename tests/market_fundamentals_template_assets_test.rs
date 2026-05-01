use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn market_fundamentals_template_json_is_a_valid_array_of_records() {
    let path = repo_root()
        .join("examples")
        .join("market_fundamentals")
        .join("market_fundamentals.template.json");
    let raw = fs::read_to_string(path).expect("expected market fundamentals template json");
    let value: Value = serde_json::from_str(&raw).expect("template json should be valid json");
    let rows = value
        .as_array()
        .expect("template json top level should be an array");

    assert!(
        rows.len() >= 2,
        "expected at least two example fundamentals rows"
    );

    for row in rows {
        let obj = row
            .as_object()
            .expect("each template row should be a json object");
        assert!(obj.contains_key("code"));
        assert!(obj.contains_key("snapshot_date"));
        assert!(obj.contains_key("market_cap"));
        assert!(obj.contains_key("latest_report_profit"));
        assert!(obj.contains_key("profit_source"));
        assert!(obj.contains_key("pe_dynamic"));
        assert!(
            obj.get("code").and_then(Value::as_str).is_some(),
            "code should be a string"
        );
        assert!(
            obj.get("snapshot_date").and_then(Value::as_str).is_some(),
            "snapshot_date should be a string"
        );
        assert!(
            obj.get("profit_source").and_then(Value::as_str).is_some(),
            "profit_source should be a string"
        );
    }
}

#[test]
fn market_fundamentals_template_readme_documents_mapping_and_rehearsal_flow() {
    let path = repo_root()
        .join("examples")
        .join("market_fundamentals")
        .join("README.md");
    let contents = fs::read_to_string(path).expect("expected market fundamentals template readme");

    for expected in [
        "Market Fundamentals JSON Template",
        "quantix data import-fundamentals --input <json>",
        "market_fundamentals.template.json",
        "snapshot_date",
        "latest_report_profit",
        "scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh",
        "cargo run --bin quantix -- data import-fundamentals --input /abs/path/market_fundamentals.json",
        "code,snapshot_date,market_cap,latest_report_profit,profit_source,pe_dynamic",
    ] {
        assert!(
            contents.contains(expected),
            "expected market fundamentals template README to contain {expected}"
        );
    }
}
