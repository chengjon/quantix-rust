use std::fs;

#[test]
fn acceptance_report_generator_references_expected_logs_and_sections() {
    let script = fs::read_to_string("scripts/dev/generate_market_cli_acceptance_report.sh")
        .expect("should read scripts/dev/generate_market_cli_acceptance_report.sh");

    for expected in [
        "run_market_cli_acceptance_*.log",
        "check_market_cli_prereqs_*.log",
        "verify_market_cli_smoke_*.log",
        "market_cli_formal_sequence_*.log",
        "Market CLI Acceptance Report",
        "precheck: PASS=",
        "smoke: PASS=",
        "sync industry exit=",
        "market foundation exit=",
        "market strength exit=",
        "summary:",
        "total_stocks:",
        "top_sector:",
        "candidate_stock_count:",
        "top_market_cap_stock:",
        "market strength-stocks exit=",
        "sector_filter:",
        "metric:",
        "coverage:",
        "top_row:",
        "quantix risk sync industry --standard shenwan",
        "quantix market foundation",
        "quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10",
        "quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10",
        "Generated report:",
    ] {
        assert!(
            script.contains(expected),
            "expected acceptance report generator to contain {expected}"
        );
    }
}
