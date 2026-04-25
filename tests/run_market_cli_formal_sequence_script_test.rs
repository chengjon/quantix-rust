use std::fs;

#[test]
fn formal_sequence_script_covers_sync_foundation_strength_and_strength_stocks() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_formal_sequence.sh")
        .expect("should read scripts/dev/run_market_cli_formal_sequence.sh");

    for expected in [
        "risk sync industry --standard shenwan",
        "market foundation",
        "market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10",
        "market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10",
        ".env.market.local",
        "init_market_cli_local_env.sh",
        "[RESULT] ${key}_exit=",
        "[LOG] ${key}_log=",
        "[SUMMARY] ${key}_summary=",
        "[FIELD] market_foundation_total_stocks=",
        "[FIELD] market_foundation_top_sector=",
        "[FIELD] market_strength_top_strong_sector=",
        "[FIELD] market_strength_top_market_cap_stock=",
        "[FIELD] market_strength_stocks_sector_filter=",
        "[FIELD] market_strength_stocks_top_row=",
        "强势板块个股 Top10 推算净利润:",
        "基础数据=",
        "A股总数=",
        "Market CLI formal sequence completed.",
    ] {
        assert!(
            script.contains(expected),
            "expected formal sequence script to contain {expected}"
        );
    }
}
