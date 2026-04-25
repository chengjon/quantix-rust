use std::fs;

#[test]
fn acceptance_orchestrator_references_template_precheck_and_smoke() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_acceptance.sh")
        .expect("should read scripts/dev/run_market_cli_acceptance.sh");

    for expected in [
        "market_cli_env.example.sh",
        ".env.market.local",
        "init_market_cli_local_env.sh",
        "check_market_cli_prereqs.sh",
        "verify_market_cli_smoke.sh",
        "Suggested first step: source",
        "Environment precheck",
        "Smoke verification",
        "quantix risk sync industry --standard shenwan",
        "quantix market foundation",
        "quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10",
        "quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10",
        "Market CLI acceptance orchestration completed.",
    ] {
        assert!(
            script.contains(expected),
            "expected acceptance orchestrator to contain {expected}"
        );
    }
}
