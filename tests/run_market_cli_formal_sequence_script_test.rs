use std::fs;
use std::process::Command;

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

#[test]
fn formal_sequence_script_extracts_strength_stocks_fields_from_real_execution() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let summary_log = log_dir.join("market_cli_formal_sequence.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_env = tempdir.path().join("fake.env");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(
        &fake_quantix,
        r#"#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  "risk sync industry --standard shenwan")
    echo "sync ok"
    ;;
  "market foundation")
    cat <<'EOF'
== 市场基础数据 ==
A股总数: 5300
已匹配行业: 5200
未匹配行业: 100
行业数: 31

行业覆盖 Top10:
排名 行业 成分股数
----------------------------------------
1    银行 42
EOF
    ;;
  "market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10")
    cat <<'EOF'
== 强弱板块分析 ==
基础数据: A股=5300 行业覆盖=5200 未覆盖=100
强势板块候选股数: 12
基本面覆盖: 市值=8/12 利润=6/12

强势板块:
排名     代码         板块             涨跌幅
--------------------------------------------------------
1        BK001        银行             2.10%

弱势板块:
排名     代码         板块             涨跌幅
--------------------------------------------------------
1        BK999        有色金属         -1.80%

强势板块个股 Top10 总市值:
排名 行业 代码 名称 现价 总市值(亿)
------------------------------------------------------------------------------------
1    银行 601398 工商银行 7.00 7000.00

强势板块个股 Top10 推算净利润:
排名 行业 代码 名称 现价 推算净利润(亿)
------------------------------------------------------------------------------------
1    银行 601398 工商银行 7.00 100.00
EOF
    ;;
  "market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10")
    cat <<'EOF'
== 强势板块个股排行 ==
强势板块范围: Top3
行业过滤: 银行
候选股数: 1
上一会计周期净利润覆盖: 1/1

按上一会计周期净利润从大到小 Top10:
排名 行业 代码 名称 现价 上一会计周期净利润(亿)
------------------------------------------------------------------------------------
1    银行 601398 工商银行 7.00 100.00
EOF
    ;;
  *)
    echo "unexpected args: $*" >&2
    exit 64
    ;;
esac
"#,
    )
    .expect("should write fake quantix");
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n").expect("should write fake init");
    fs::write(&fake_env, "").expect("should write fake env");

    for path in [&fake_quantix, &fake_init] {
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        fs::set_permissions(path, perms).expect("set permissions");
    }

    let output = Command::new("bash")
        .arg("scripts/dev/run_market_cli_formal_sequence.sh")
        .env("LOG_DIR", &log_dir)
        .env("SUMMARY_LOG", &summary_log)
        .env("QUANTIX_BIN", &fake_quantix)
        .env("INIT_LOCAL_ENV_SCRIPT", &fake_init)
        .env("LOCAL_ENV_PATH", &fake_env)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = fs::read_to_string(&summary_log).expect("should read summary log");
    assert!(summary.contains("[RESULT] market_strength_stocks_exit=0"));
    assert!(summary.contains("[FIELD] market_strength_stocks_sector_filter=银行"));
    assert!(summary.contains("[FIELD] market_strength_stocks_metric=上一会计周期净利润"));
    assert!(summary.contains("[FIELD] market_strength_stocks_coverage=1/1"));
    assert!(summary.contains("[FIELD] market_strength_stocks_top_row=1 银行 601398 工商银行 7.00 100.00"));
}
