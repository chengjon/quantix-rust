use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn ignore_file_covers_repo_local_noise() {
    let ignore_path = repo_root().join(".ignore");
    let contents = fs::read_to_string(ignore_path).expect("expected .ignore to exist");

    for entry in [".worktrees/", ".gitnexus/", "target/"] {
        assert!(
            contents.lines().any(|line| line.trim() == entry),
            "expected .ignore to contain {entry}"
        );
    }
}

#[test]
fn readme_documents_foundation_p0_workspace_constraints() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    assert!(
        contents.contains(".worktrees/"),
        "expected README to mention repo-local worktrees"
    );
    assert!(
        contents.contains("Foundation P0"),
        "expected README to mention Foundation P0"
    );
    assert!(
        contents.contains("daemon"),
        "expected README to describe daemon support boundary"
    );
    assert!(
        contents.contains("quantix market sector"),
        "expected README to advertise market sector command"
    );
    assert!(
        contents.contains("quantix market overview"),
        "expected README to advertise market overview command"
    );
    assert!(
        contents.contains("历史/详情/实时功能延后"),
        "expected README to describe deferred market features"
    );
}

#[test]
fn readme_documents_phase24_monitor_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 24: 实时监控",
        "quantix monitor watchlist --once",
        "quantix monitor alert add 000001 --above 16.0",
        "quantix monitor alert add 000001 --below 15.0",
        "quantix monitor config show",
        "quantix monitor daemon run",
        "quantix monitor service install",
        "quantix monitor service-config show",
        "quantix monitor service-config set --quantix-bin",
        "quantix monitor event list",
        "QUANTIX_MONITOR_DB_PATH",
        "~/.quantix/monitor/alerts.db",
        "QUANTIX_MONITOR_CONFIG_PATH",
        "~/.quantix/monitor/config.json",
        "~/.quantix/monitor/service.json",
        "~/.local/bin/quantix-monitor-run",
        "systemd --user",
        "系统通知延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase25_stop_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 25: 止盈止损",
        "quantix stop set 000001 --loss 14.5",
        "quantix stop set 000001 --trailing 5 --profit 18.0",
        "quantix stop list",
        "quantix stop remove 000001",
        "仅允许对已在本地自选池中的股票设置规则",
        "quantix monitor watchlist --once 会在监控快照阶段继续评估止盈止损规则",
        "stop status / stop history / stop update / 百分比止损止盈参数延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase23_market_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### market - 市场分析",
        "quantix market sector [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market concept [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market north [--date <YYYY-MM-DD>]",
        "quantix market sentiment [--date <YYYY-MM-DD>]",
        "quantix market leader (--sector <NAME> | --concept <NAME> | --all) [--limit <N>] [--date <YYYY-MM-DD>]",
        "quantix market overview [--top <N>] [--date <YYYY-MM-DD>]",
        "历史/详情/实时能力延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase25_stop_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### stop - 止盈止损",
        "quantix stop set <CODE> [--loss <PRICE>] [--profit <PRICE>] [--trailing <PCT>]",
        "quantix stop list",
        "quantix stop remove <CODE>",
        "仅允许为当前本地自选池中的代码设置规则",
        "复用 `QUANTIX_MONITOR_DB_PATH` 指向的 SQLite 路径",
        "`quantix monitor watchlist --once` 会在输出监控快照后继续评估止盈止损规则",
        "`stop status`、`stop history`、`stop update`、`--loss-pct`、`--profit-pct` 延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase24_monitor_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### monitor - 实时监控",
        "quantix monitor watchlist --once",
        "quantix monitor alert add <CODE> (--above <PRICE> | --below <PRICE>)",
        "quantix monitor alert list",
        "quantix monitor alert remove <ID>",
        "quantix monitor config show",
        "quantix monitor daemon run",
        "quantix monitor service install",
        "quantix monitor service-config show",
        "quantix monitor service-config set --quantix-bin",
        "quantix monitor event list",
        "QUANTIX_MONITOR_DB_PATH",
        "~/.quantix/monitor/alerts.db",
        "QUANTIX_MONITOR_CONFIG_PATH",
        "~/.quantix/monitor/config.json",
        "~/.quantix/monitor/service.json",
        "~/.local/bin/quantix-monitor-run",
        "systemd --user",
        "系统通知延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase26_trade_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 26: 模拟交易",
        "quantix trade init",
        "quantix trade buy",
        "quantix trade sell",
        "quantix trade history",
        "quantix trade fees",
        "quantix trade overview",
        "quantix trade position --current",
        "quantix trade cash",
        "QUANTIX_TRADE_PATH",
        "实时价格获取失败时降级为空",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase26_trade_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### trade - 模拟交易",
        "quantix trade init [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]",
        "quantix trade buy <CODE> --price <PRICE> --volume <N>",
        "quantix trade sell <CODE> --price <PRICE> --volume <N>",
        "quantix trade history [--code <CODE>] [--limit <N>]",
        "quantix trade fees [--code <CODE>] [--limit <N>]",
        "quantix trade overview [--current]",
        "quantix trade position [--current]",
        "quantix trade cash",
        "QUANTIX_TRADE_PATH",
        "best-effort 实时行情",
        "拿不到价格时降级为空",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase27_risk_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 27: 风险管理",
        "quantix risk rule set --type position-limit --value 20%",
        "quantix risk rule set --type daily-loss-limit --value 50000",
        "quantix risk rule list",
        "quantix risk status",
        "quantix risk pnl",
        "quantix risk position",
        "quantix risk log",
        "quantix risk lock release",
        "QUANTIX_RISK_PATH",
        "trade buy` 会执行风控预检查，`trade sell` 仍然允许成交",
        "risk status` 会额外显示锁状态来源、作用交易日、触发原因、触发时间",
        "risk log` 仅记录规则变更、日亏损锁触发、手动释放、以及 rollover/reset 清锁事件",
        "当日内不再自动重新锁定",
        "risk log` 默认返回最近事件，当前支持按事件写入日 `--date` 与事件类型 `--type` 过滤",
        "实盘导入 / 波动率和行业规则 / 自动减仓",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase29_strategy_paper_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 29: 策略 Paper 执行骨架",
        "quantix strategy run -n ma_cross --mode paper --code 000001",
        "quantix strategy run -n ma_cross --mode mock_live --code 000001",
        "QUANTIX_STRATEGY_RUNTIME_DB_PATH",
        "~/.quantix/strategy/runtime.db",
        "执行前请先运行 `quantix trade init`",
        "`mock_live` 当前会返回非终态订单状态",
        "live 模式仍在开发中",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase29b_strategy_signal_daemon_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 29B: 策略信号守护进程",
        "quantix strategy config init",
        "quantix strategy daemon run --once",
        "quantix strategy signal list",
        "quantix strategy signal approve --signal-id",
        "quantix strategy request list",
        "quantix strategy service install",
        "quantix strategy service-config show",
        "quantix strategy service-config set --quantix-bin",
        "~/.quantix/strategy/config.json",
        "~/.quantix/strategy/service.json",
        "~/.quantix/strategy/service.env",
        "~/.local/bin/quantix-strategy-run",
        "QUANTIX_TDX_ROOT",
        "QUANTIX_TDX_MARKET",
        "strategy signal list` 输出包含 `source=<SOURCE> fallback=<BOOL>`",
        "strategy daemon run --once` 首次启动只 bootstrap 到最新 bar",
        "批准 signal 只会创建 `execution_request`，不会自动交易",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase27_risk_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### risk - 风险管理",
        "quantix risk rule set --type position-limit --value 20%",
        "quantix risk rule set --type daily-loss-limit --value 50000",
        "quantix risk rule enable --type position-limit",
        "quantix risk rule disable --type daily-loss-limit",
        "quantix risk status",
        "quantix risk pnl",
        "quantix risk position",
        "quantix risk log [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]",
        "quantix risk lock release",
        "QUANTIX_RISK_PATH",
        "`risk status`、`risk pnl`、`risk position` 依赖已初始化的 paper-trade 账户",
        "`risk status` 的 `状态来源` 当前只区分 `open`、`daily_loss_locked`、`manual_release_active`",
        "当日买入锁触发后，新的 `trade buy` 会被拒绝，但 `trade sell` 仍允许执行",
        "`risk log` 默认返回最近 20 条事件，可用 `--limit` 调整，并支持 `--date`、`--type` 单独或组合过滤",
        "`risk log --date` 匹配事件写入日，也就是 `event.ts` 所在日期",
        "`risk lock release` 在当日内抑制基于日亏损规则的自动重新加锁",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase29_strategy_paper_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "quantix strategy run -n <NAME> [--mode <MODE>] [-c|--code <CODE>]",
        "| `paper` | 模拟盘模式（当前支持 `ma_cross` 单次执行） |",
        "| `mock_live` | mock-live 模式（支持非终态订单生命周期模拟） |",
        "quantix strategy run -n ma_cross --mode paper -c 000001",
        "quantix strategy run -n ma_cross --mode mock_live -c 000001",
        "QUANTIX_STRATEGY_RUNTIME_DB_PATH",
        "~/.quantix/strategy/runtime.db",
        "首次使用前请先执行 `quantix trade init`",
        "`mock_live` 可能返回 `accepted`、`partially_filled`、`unknown` 等非终态状态",
        "`live` 模式仍在开发中",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase29b_strategy_daemon_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "Phase 29B: 策略信号守护进程",
        "quantix strategy config init",
        "quantix strategy daemon run --once",
        "quantix strategy signal list --approval-status pending",
        "quantix strategy signal approve --signal-id <ID> --target-mode paper --target-account default",
        "quantix strategy signal reject --signal-id <ID> --reason \"manual reject\"",
        "quantix strategy request list --status pending",
        "quantix strategy service install",
        "quantix strategy service-config show",
        "quantix strategy service-config set --quantix-bin /abs/path/to/quantix --env-file /abs/path/to/service.env",
        "~/.quantix/strategy/config.json",
        "~/.quantix/strategy/runtime.db",
        "~/.quantix/strategy/service.json",
        "~/.quantix/strategy/service.env",
        "~/.local/bin/quantix-strategy-run",
        "QUANTIX_TDX_ROOT",
        "QUANTIX_TDX_MARKET",
        "strategy signal list` 会输出 `source=<SOURCE> fallback=<BOOL>`",
        "strategy signal approve` 会输出 `request_id signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS>`",
        "strategy request list` 会输出 `request_id signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS>`",
        "不会自动交易，不会修改 paper 账户",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn quickstart_uses_current_strategy_cli_shape() {
    let quickstart_path = repo_root().join("docs").join("QUICKSTART.md");
    let contents = fs::read_to_string(quickstart_path).expect("expected QUICKSTART.md to exist");

    assert!(
        contents.contains("cargo run -- strategy run -n ma_cross --code 000001"),
        "expected QUICKSTART to use current strategy CLI syntax"
    );
}
