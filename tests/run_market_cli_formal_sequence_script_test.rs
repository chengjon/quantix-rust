use std::fs;
use std::process::Command;

#[test]
fn formal_sequence_script_covers_sync_foundation_strength_and_strength_stocks() {
    let script = fs::read_to_string("scripts/dev/run_market_cli_formal_sequence.sh")
        .expect("should read scripts/dev/run_market_cli_formal_sequence.sh");

    for expected in [
        "risk sync industry --standard shenwan",
        "MARKET_FUNDAMENTALS_INPUT",
        "REHEARSAL_SCRIPT",
        "QUANTIX_MARKET_SNAPSHOT_SOURCE",
        "data validate-fundamentals --input",
        "data import-fundamentals --input",
        "No market fundamentals input configured; skipping validate-fundamentals / import-fundamentals steps",
        "To validate a fundamentals JSON against scratch ClickHouse first, run:",
        "market foundation",
        "Market snapshot source mode:",
        "resolve_market_date",
        "MARKET_DATE_QUERY_CMD",
        "Using market date for formal sequence:",
        "market strength --date $MARKET_DATE --strong-top 3 --weak-top 3 --stock-top 10",
        "resolve_strength_stocks_sector_from_log",
        "Using dynamic strong sector for market strength-stocks",
        "market strength-stocks --date $MARKET_DATE --strong-top 3",
        "--metric profit --top 10",
        ".env.market.local",
        "init_market_cli_local_env.sh",
        "[RESULT] ${key}_exit=",
        "[LOG] ${key}_log=",
        "[SUMMARY] ${key}_summary=",
        "[FIELD] market_foundation_total_stocks=",
        "[FIELD] market_foundation_top_sector=",
        "[FIELD] market_strength_top_strong_sector=",
        "[FIELD] market_strength_top_market_cap_stock=",
        "[FIELD] market_strength_snapshot_source=",
        "[FIELD] market_strength_tdx_coverage=",
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
fn formal_sequence_script_optionally_runs_market_fundamentals_import() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let summary_log = log_dir.join("market_cli_formal_sequence.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_fundamentals = tempdir.path().join("market_fundamentals.json");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(&fake_fundamentals, "[]").expect("should write fake fundamentals file");
    fs::write(
        &fake_quantix,
        format!(
            r#"#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  "risk sync industry --standard shenwan")
    echo "sync ok"
    ;;
  "data validate-fundamentals --input {}")
    cat <<'EOF'
🔎 校验市场基础面快照
  文件: {}
  记录数: 2
  唯一股票: 2
  快照日期范围: 2026-03-14 ~ 2026-03-15
  总市值覆盖: 2/2
  净利润覆盖: 2/2

[FIELD] validation_total_records=2
[FIELD] validation_unique_codes=2
[FIELD] validation_snapshot_min=2026-03-14
[FIELD] validation_snapshot_max=2026-03-15
[FIELD] validation_market_cap_coverage=2/2
[FIELD] validation_latest_report_profit_coverage=2/2
[FIELD] validation_profit_sources=manual=1,report=1
[PASS] No blocking data-shape issues detected.
EOF
    ;;
  "data import-fundamentals --input {}")
    cat <<'EOF'
 导入市场基础面快照
  文件: {}
  记录数: 2
✅ 市场基础面快照导入完成
  已写入: 2
  耗时(秒): 1
EOF
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
  "market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10")
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
  "market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10")
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
            fake_fundamentals.display(),
            fake_fundamentals.display(),
            fake_fundamentals.display(),
            fake_fundamentals.display()
        ),
    )
    .expect("should write fake quantix");
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
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
        .env("MARKET_DATE", "2026-03-14")
        .env("MARKET_FUNDAMENTALS_INPUT", &fake_fundamentals)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = fs::read_to_string(&summary_log).expect("should read summary log");
    assert!(summary.contains("[STEP] Validate market fundamentals"));
    assert!(summary.contains("[RESULT] market_fundamentals_validate_exit=0"));
    assert!(summary.contains(&format!(
        "[FIELD] market_fundamentals_validate_input={}",
        fake_fundamentals.display()
    )));
    assert!(summary.contains("[FIELD] market_fundamentals_validate_total_records=2"));
    assert!(summary.contains("[STEP] Import market fundamentals"));
    assert!(summary.contains(&format!(
        "[FIELD] market_fundamentals_import_input={}",
        fake_fundamentals.display()
    )));
    assert!(summary.contains("[FIELD] market_fundamentals_import_records=2"));
    assert!(summary.contains("[FIELD] market_fundamentals_import_written=2"));
    assert!(summary.contains("[RESULT] market_fundamentals_import_exit=0"));
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
  "market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10")
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
  "market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10")
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
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
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
        .env("MARKET_DATE", "2026-03-14")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = fs::read_to_string(&summary_log).expect("should read summary log");
    assert!(summary.contains(
        "[INFO] To validate a fundamentals JSON against scratch ClickHouse first, run:"
    ));
    assert!(summary.contains("[INFO] Using market date for formal sequence: 2026-03-14"));
    assert!(summary.contains("[FIELD] market_strength_snapshot_source=primary"));
    assert!(summary.contains("[FIELD] market_strength_tdx_coverage=N/A"));
    assert!(summary.contains("[RESULT] market_strength_stocks_exit=0"));
    assert!(summary.contains("[FIELD] market_strength_stocks_sector_filter=银行"));
    assert!(summary.contains("[FIELD] market_strength_stocks_metric=上一会计周期净利润"));
    assert!(summary.contains("[FIELD] market_strength_stocks_coverage=1/1"));
    assert!(
        summary
            .contains("[FIELD] market_strength_stocks_top_row=1 银行 601398 工商银行 7.00 100.00")
    );
}

#[test]
fn formal_sequence_script_falls_back_to_repo_root_env_for_tdx_settings() {
    let tempdir = tempfile::tempdir().expect("should create tempdir");
    let log_dir = tempdir.path().join("logs");
    let summary_log = log_dir.join("market_cli_formal_sequence.log");
    let fake_quantix = tempdir.path().join("fake-quantix.sh");
    let fake_init = tempdir.path().join("fake-init.sh");
    let fake_env = tempdir.path().join("fake.env");
    let fake_root_env = tempdir.path().join("repo.env");

    fs::create_dir_all(&log_dir).expect("should create log dir");
    fs::write(
        &fake_quantix,
        r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "${QUANTIX_TDX_ROOT:-}" != "/mnt/d/mystocks/tdx/tdx-quant" ]]; then
  echo "missing QUANTIX_TDX_ROOT" >&2
  exit 71
fi
if [[ "${QUANTIX_TDX_MARKET:-}" != "sh" ]]; then
  echo "missing QUANTIX_TDX_MARKET" >&2
  exit 72
fi
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
  "market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10")
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
  "market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10")
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
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
    fs::write(&fake_env, "").expect("should write fake env");
    fs::write(
        &fake_root_env,
        "QUANTIX_TDX_ROOT=/mnt/d/mystocks/tdx/tdx-quant\nQUANTIX_TDX_MARKET=sh\n",
    )
    .expect("should write fake repo env");

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
        .env("ROOT_ENV_PATH", &fake_root_env)
        .env("MARKET_DATE", "2026-03-14")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn formal_sequence_script_extracts_strength_stocks_top_row_for_dynamic_top_count() {
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
  "market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10")
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
  "market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10")
    cat <<'EOF'
== 强势板块个股排行 ==
强势板块范围: Top3
行业过滤: 银行
候选股数: 2
上一会计周期净利润覆盖: 2/2

按上一会计周期净利润从大到小 Top2:
排名 行业 代码 名称 现价 上一会计周期净利润(亿)
------------------------------------------------------------------------------------
1    银行 603323 苏农银行 4.96 20.42
2    银行 601997 贵阳银行 6.12 14.77
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
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
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
        .env("MARKET_DATE", "2026-03-14")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = fs::read_to_string(&summary_log).expect("should read summary log");
    assert!(summary.contains("[FIELD] market_strength_stocks_coverage=2/2"));
    assert!(
        summary
            .contains("[FIELD] market_strength_stocks_top_row=1 银行 603323 苏农银行 4.96 20.42")
    );
}

#[test]
fn formal_sequence_script_passes_market_snapshot_source_mode_to_quantix() {
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
if [[ "${QUANTIX_MARKET_SNAPSHOT_SOURCE:-}" != "tdx" ]]; then
  echo "missing QUANTIX_MARKET_SNAPSHOT_SOURCE" >&2
  exit 73
fi
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
  "market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10")
    cat <<'EOF'
2026-04-28T05:35:05Z  INFO quantix_cli::market::strength: QUANTIX_MARKET_SNAPSHOT_SOURCE=tdx，跳过 EastMoney A股全市场快照，直接使用 TDX
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
  "market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10")
    cat <<'EOF'
2026-04-28T05:35:08Z  INFO quantix_cli::market::strength: QUANTIX_MARKET_SNAPSHOT_SOURCE=tdx，跳过 EastMoney A股全市场快照，直接使用 TDX
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
    fs::write(&fake_init, "#!/usr/bin/env bash\nset -euo pipefail\n")
        .expect("should write fake init");
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
        .env("MARKET_DATE", "2026-03-14")
        .env("QUANTIX_MARKET_SNAPSHOT_SOURCE", "tdx")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run formal sequence script");

    assert!(
        output.status.success(),
        "expected success, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let summary = fs::read_to_string(&summary_log).expect("should read summary log");
    assert!(summary.contains("[INFO] Market snapshot source mode: tdx"));
    assert!(summary.contains("[FIELD] market_strength_snapshot_source=tdx_configured"));
}
