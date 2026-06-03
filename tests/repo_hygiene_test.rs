use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn is_repo_doc_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("md" | "html")
    )
}

fn collect_main_workspace_doc_paths(dir: &Path, docs: &mut Vec<PathBuf>) {
    let skip_dirs = [
        ".git",
        ".gitnexus",
        ".idea",
        ".vscode",
        ".worktrees",
        ".zread",
        "archive",
        "node_modules",
        "openspec",
        "plans",
        "reports",
        "superpowers",
        "target",
    ];

    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("expected {dir:?} to be readable")) {
        let entry = entry.unwrap_or_else(|_| panic!("expected readable entry under {dir:?}"));
        let path = entry.path();

        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !skip_dirs.contains(&name) {
                collect_main_workspace_doc_paths(&path, docs);
            }
            continue;
        }

        if is_repo_doc_path(&path) {
            docs.push(path);
        }
    }
}

fn collect_rust_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("expected {dir:?} to be readable")) {
        let entry = entry.unwrap_or_else(|_| panic!("expected readable entry under {dir:?}"));
        let path = entry.path();

        if path.is_dir() {
            collect_rust_paths(&path, paths);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
}

#[test]
fn market_output_renderer_does_not_panic_on_string_writes() {
    let path = repo_root().join("src/cli/handlers/market_output.rs");
    let contents = fs::read_to_string(&path).expect("expected market output renderer to exist");

    assert!(
        !contents.contains(".unwrap(") && !contents.contains(".expect("),
        "market output renderer should not panic while formatting CLI output"
    );
}

#[test]
fn cron_presets_do_not_panic_on_runtime_parse() {
    let path = repo_root().join("src/tasks/cron.rs");
    let contents = fs::read_to_string(&path).expect("expected cron scheduler to exist");
    let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);

    assert!(
        !production.contains(".unwrap(") && !production.contains(".expect("),
        "cron preset constructors should use validated constants instead of runtime unwraps"
    );
}

#[test]
fn anomaly_detector_does_not_write_directly_to_stdio() {
    let path = repo_root().join("src/anomaly/detector.rs");
    let contents = fs::read_to_string(&path).expect("expected anomaly detector to exist");
    let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);

    assert!(
        !production.contains("println!(") && !production.contains("eprintln!("),
        "anomaly detector should render results or use tracing instead of writing to stdio"
    );
}

#[test]
fn non_cli_production_modules_do_not_write_directly_to_stdio() {
    let root = repo_root();
    let src_dir = root.join("src");
    let mut rust_paths = Vec::new();
    collect_rust_paths(&src_dir, &mut rust_paths);

    let mut offenders = Vec::new();
    for path in rust_paths {
        let relative_path = path.strip_prefix(&root).unwrap_or(&path);
        let relative = relative_path.to_string_lossy();

        if relative.starts_with("src/cli/")
            || relative.starts_with("src/bin/")
            || relative.starts_with("src/tui/")
            || relative == "src/main.rs"
            || relative.ends_with("/tests.rs")
            || relative.contains("/tests/")
        {
            continue;
        }

        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("expected {relative} to be readable"));
        let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);
        if production.contains("println!(") || production.contains("eprintln!(") {
            offenders.push(relative.into_owned());
        }
    }

    assert!(
        offenders.is_empty(),
        "non-CLI production modules should use tracing or returned renderable data instead of direct stdio writes: {}",
        offenders.join(", ")
    );
}

#[test]
fn metrics_and_batch_production_paths_do_not_use_panic_prone_shortcuts() {
    for relative_path in ["src/monitoring/metrics.rs", "src/io/batch.rs"] {
        let path = repo_root().join(relative_path);
        let contents = fs::read_to_string(&path).expect("expected hygiene target to exist");
        let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);

        assert!(
            !production.contains(".unwrap(")
                && !production.contains(".expect(")
                && !production.contains("panic!("),
            "{relative_path} production code should not use panic-prone shortcuts"
        );
    }
}

#[test]
fn grid_initialize_range_does_not_unwrap_bounds() {
    let source_path = "src/strategy/grid.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    let initialize_start = source
        .find("fn initialize_grid")
        .unwrap_or_else(|| panic!("expected {source_path} to define initialize_grid"));
    let initialize_end = source[initialize_start..]
        .find("fn generate_grid_orders")
        .map(|offset| initialize_start + offset)
        .unwrap_or(source.len());
    let initialize_body = &source[initialize_start..initialize_end];

    assert!(
        initialize_body.contains("let upper_bound = current_price + half_range;"),
        "expected initialize_grid in {source_path} to calculate upper_bound as a local value"
    );
    assert!(
        initialize_body.contains("let lower_bound = current_price - half_range;"),
        "expected initialize_grid in {source_path} to calculate lower_bound as a local value"
    );
    assert!(
        !initialize_body.contains("upper_bound.unwrap()")
            && !initialize_body.contains("lower_bound.unwrap()"),
        "expected initialize_grid in {source_path} not to unwrap stored bounds while calculating range"
    );
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
fn archive_index_describes_retention_buckets() {
    let contents = fs::read_to_string(repo_root().join("docs").join("archive").join("README.md"))
        .expect("expected docs/archive/README.md to exist");

    for expected in [
        "FUNCTION_TREE.md",
        "docs/USER_MANUAL.md",
        "docs/archive/plans/",
        "docs/archive/reports/",
        "docs/archive/ad-hoc/",
        "current evidence report",
    ] {
        assert!(
            contents.contains(expected),
            "expected archive index to mention {expected}"
        );
    }
}

#[test]
fn archive_bucket_indexes_exist() {
    for relative_path in [
        "docs/archive/plans/README.md",
        "docs/archive/reports/README.md",
        "docs/archive/ad-hoc/README.md",
    ] {
        assert!(
            repo_root().join(relative_path).exists(),
            "expected archive bucket index {relative_path} to exist"
        );
    }
}

#[test]
fn legacy_docs_are_archived_without_moving_current_reports() {
    for relative_path in [
        "docs/archive/ad-hoc/AUDIT_OPTIMIZATION_PLAN_2026-03-29.md",
        "docs/archive/plans/2026-03-10-phase21-watchlist-implementation.md",
        "docs/archive/reports/CODE_RULE_SUMMARY.md",
        "docs/archive/reports/PHASE18_BENCHMARK_RESULTS.md",
    ] {
        assert!(
            repo_root().join(relative_path).exists(),
            "expected archived legacy document {relative_path} to exist"
        );
    }

    for relative_path in [
        "docs/AUDIT_OPTIMIZATION_PLAN_2026-03-29.md",
        "docs/plans/2026-03-10-phase21-watchlist-implementation.md",
        "docs/reports/CODE_RULE_SUMMARY.md",
        "docs/reports/PHASE18_BENCHMARK_RESULTS.md",
    ] {
        assert!(
            !repo_root().join(relative_path).exists(),
            "expected legacy document {relative_path} to move into archive"
        );
    }

    for relative_path in [
        "docs/reports/ARCHITECTURE_AUDIT_2026-05-23.md",
        "docs/reports/DIRTY_WORKTREE_RECHECK_DISPOSITION_2026-05-27.md",
    ] {
        assert!(
            repo_root().join(relative_path).exists(),
            "expected current evidence report {relative_path} to remain active"
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
        contents.contains("quantix market foundation"),
        "expected README to advertise market foundation command"
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
        contents.contains("quantix market strength"),
        "expected README to advertise market strength command"
    );
    assert!(
        contents.contains("历史/详情/实时功能延后"),
        "expected README to describe deferred market features"
    );
}

#[test]
fn readme_documents_foundation_p0_task_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "`quantix task` CLI 当前只支持查看预置任务模板、以前台模式启动它们，以及输出能力边界说明",
        "`task add` 与 `task start --daemon` 仍未开放",
        "CLI 层当前只开放预置模板查看与前台启动；`task add` / `task start --daemon` 仍是保留入口",
        "`task list/start/stop/status` - 实验性 Foundation P0 预置任务模板入口",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase24_monitor_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 24: 实时监控",
        "quantix monitor watchlist --once",
        "quantix monitor watchlist --repeat",
        "quantix monitor alert add 000001 --above 16.0",
        "quantix monitor alert add 000001 --below 15.0",
        "quantix monitor config show",
        "quantix monitor config set --notify true",
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
        "QUANTIX_MONITOR_NOTIFY=1",
        "NOTIFICATION_LOG_PATH",
        "推荐通过 `quantix monitor config set --notify true` 显式开启",
        "系统通知当前支持 `quantix monitor watchlist --repeat` / `quantix monitor daemon run` 对新增监控事件做自动通知桥接",
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
        "quantix stop set 000001 --loss-pct 5",
        "quantix stop update 000001 --profit-pct 12 --clear-profit",
        "quantix stop list",
        "quantix stop status --code 000001",
        "quantix stop history --code 000001 --limit 10",
        "quantix stop remove 000001",
        "仅允许对已在本地自选池中的股票设置规则",
        "quantix monitor watchlist --once 会在监控快照阶段继续评估止盈止损规则",
        "百分比规则优先锚定本地 paper 持仓均价",
        "无持仓时退回到规则的 reference_price",
        "stop_history 会记录规则变更和 trigger 审计事件",
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
        "quantix market foundation",
        "quantix market sector [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market concept [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market north [--date <YYYY-MM-DD>]",
        "quantix market sentiment [--date <YYYY-MM-DD>]",
        "quantix market leader (--sector <NAME> | --concept <NAME> | --all) [--limit <N>] [--date <YYYY-MM-DD>]",
        "quantix market overview [--top <N>] [--date <YYYY-MM-DD>]",
        "quantix market strength [--date <YYYY-MM-DD>] [--strong-top <N>] [--weak-top <N>] [--stock-top <N>]",
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
        "quantix stop set <CODE> [--loss <PRICE>] [--profit <PRICE>] [--loss-pct <PCT>] [--profit-pct <PCT>] [--trailing <PCT>]",
        "quantix stop update <CODE>",
        "quantix stop list",
        "quantix stop status [--code <CODE>]",
        "quantix stop history [--code <CODE>] [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]",
        "quantix stop remove <CODE>",
        "仅允许为当前本地自选池中的代码设置规则",
        "复用 `QUANTIX_MONITOR_DB_PATH` 指向的 SQLite 路径",
        "`quantix monitor watchlist --once` 会在输出监控快照后继续评估止盈止损规则",
        "`--loss` 与 `--loss-pct` 互斥，`--profit` 与 `--profit-pct` 互斥",
        "百分比阈值优先使用当前本地 `paper` 持仓 `avg_cost` 作为锚点",
        "没有持仓时退回到规则持久化的 `reference_price`",
        "`stop status` 会显示 `anchor_source`、有效阈值和 `eval_state`",
        "`stop history` 会记录 `set`、`update`、`remove` 和 `trigger` 事件",
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
        "quantix monitor watchlist --repeat",
        "quantix monitor alert add <CODE> (--above <PRICE> | --below <PRICE>)",
        "quantix monitor alert list",
        "quantix monitor alert remove <ID>",
        "quantix monitor config show",
        "quantix monitor config set --notify true",
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
        "QUANTIX_MONITOR_NOTIFY=1",
        "NOTIFICATION_LOG_PATH",
        "推荐通过 `quantix monitor config set --notify true` 显式开启",
        "系统通知当前支持 `quantix monitor watchlist --repeat` / `quantix monitor daemon run` 对新增监控事件做自动通知桥接",
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
fn readme_and_user_manual_document_bridge_v1_commands() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let manual = fs::read_to_string(repo_root().join("docs").join("USER_MANUAL.md"))
        .expect("expected USER_MANUAL.md");

    for expected in [
        "Windows Bridge v1",
        "QUANTIX_BRIDGE_BASE_URL",
        "QUANTIX_BRIDGE_API_KEY",
        "quantix execution qmt status",
        "quantix execution qmt preview --request-id <ID>",
        "quantix execution bridge status",
        "quantix execution bridge qmt-preview --request-id <ID>",
        "QMT preview-only",
        "qmt.mode=live",
        "execution_request",
        "TDX bridge source",
    ] {
        assert!(
            readme.contains(expected),
            "expected README to contain {expected}"
        );
        assert!(
            manual.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn readme_and_user_manual_document_qmt_live_boundary() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let manual = fs::read_to_string(repo_root().join("docs").join("USER_MANUAL.md"))
        .expect("expected USER_MANUAL.md");

    for expected in [
        "QMT preview path",
        "qmt-preview",
        "quantix execution qmt live --request-id <ID>",
        "quantix execution bridge qmt-live --request-id <ID>",
        "target_mode=live",
        "qmt_live",
        "qmt.enabled=true",
        "qmt.supports",
        "order_submit",
        "不代表整个 QMT 能力仍然只有预览",
        "输入 `YES` 确认",
        "quantix execution qmt query --order-id <ORDER_ID>",
        "quantix execution bridge qmt-query --order-id <ORDER_ID>",
    ] {
        assert!(
            readme.contains(expected),
            "expected README to contain {expected}"
        );
        assert!(
            manual.contains(expected),
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
        "quantix risk rule set --type volatility-limit --value 4%",
        "quantix risk sync industry --standard shenwan",
        "quantix risk rule set --type industry-blocklist --value 银行,地产",
        "quantix risk import live-trades --account live-001 --input /tmp/live.csv",
        "quantix risk rebuild live-account --account live-001",
        "quantix risk rule list",
        "quantix risk status",
        "quantix risk pnl",
        "quantix risk position",
        "quantix risk status --source live_import --account live-001",
        "quantix risk pnl --source live_import --account live-001",
        "quantix risk position --source live_import --account live-001",
        "quantix risk log",
        "quantix risk lock release",
        "QUANTIX_RISK_PATH",
        "QUANTIX_UPSTREAM_MYSQL_URL",
        "industry_reference.db",
        "trade buy` 会执行风控预检查，`trade sell` 仍然允许成交",
        "risk status` 会额外显示锁状态来源、作用交易日、触发原因、触发时间",
        "risk log` 仅记录规则变更、日亏损锁触发、手动释放、以及 rollover/reset 清锁事件",
        "当日内不再自动重新锁定",
        "risk log` 默认返回最近事件，当前支持按事件写入日 `--date` 与事件类型 `--type` 过滤",
        "live_import 镜像账户与 paper_trade.json 严格隔离",
        "`volatility-limit` 使用 `ATR(14) / latest_close * 100`",
        "`volatility-limit` 缺少日线时会拒绝买单而不是静默跳过",
        "`volatility-limit` 只拦截新的买单，不影响卖出",
        "`industry-blocklist` 现已成为受支持的风险规则",
        "Phase 27D v1 使用 `SW 一级行业` 作为运行时生效标准",
        "`security_class_2024` / CSRC 2024 仍保留在系统中作为并行分类标准，但不是该 v1 规则的运行时生效标准",
        "运行时风控评估只读取本地 SQLite 参考/快照表",
        "MySQL 仅作为上游同步源，不是运行时查询依赖",
        "启用 `industry-blocklist` 前需要先执行 `quantix risk sync industry --standard shenwan`",
        "启用 `industry-limit` 前同样需要先执行 `quantix risk sync industry --standard shenwan`",
        "如果本地 SQLite 行业引用表尚未同步完成，`industry-blocklist` 会 fail-closed 并拒绝买单",
        "如果本地 SQLite 行业引用表尚未同步完成，`industry-limit` 也会 fail-closed 并拒绝买单",
        "运行时行业解析顺序固定为：当前 SW 映射 -> 查询月份快照 -> 历史 SW 映射 -> 最新本地快照",
        "月度快照会在该月第一次成功命中生效标准时冻结",
        "`industry-blocklist` 继续使用精确字符串匹配",
        "`industry-blocklist` 只拦截新的买单，不影响卖出路径",
        "实盘导入当前只支持项目标准化 CSV/JSON",
        "failed rebuild 会保留上一次成功镜像状态",
        "`industry-limit` 会按目标行业的买后集中度执行真实拦截；`auto-reduce` 当前仅输出人工减仓建议，不会自动卖出",
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
        "live-ready hardening / reconciliation scaffolding",
        "不是真实 broker live execution",
        "同一个 mock-live 订单在 partial fill 场景下可能写出多笔 `TradeRecord`",
        "docs/standards/MOCK_USAGE_POLICY.md",
        "live 模式仍在开发中",
        "通用 `target_mode=live` 仍在开发中",
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
        "quantix strategy request execute --request-id",
        "quantix strategy request cancel --request-id",
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
        "strategy signal list` 支持按 `--strategy-instance`、`--strategy`、`--code`、`--approval-status`、`--signal-status` 过滤，并在过滤后应用 `--limit`",
        "strategy daemon run --once` 首次启动只 bootstrap 到最新 bar",
        "批准 signal 只会创建 `execution_request`，不会自动交易",
        "request execute` 会手动消费一个 `pending execution_request`",
    ] {
        assert!(
            contents.contains(expected),
            "expected README to contain {expected}"
        );
    }
}

#[test]
fn readme_documents_phase29c_execution_automation_boundary() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    for expected in [
        "Phase 29C: 执行自动化收口",
        "quantix execution config init",
        "quantix execution config show",
        "quantix execution daemon run",
        "quantix execution daemon run --once",
        "QUANTIX_EXECUTION_CONFIG_PATH",
        "~/.quantix/execution/config.json",
        "`execution_request` 当前新增 `in_progress`",
        "`execution daemon` 只消费 `pending execution_request`",
        "自动审批当前只支持 `manual|always`",
        "`strategy request execute` 与 `execution daemon` 复用同一条 request 消费路径",
        "当 payload 内存在结构化 `execution_diagnostics` 时，`strategy request show` 会新增 `Execution Diagnostics` section，展示 `code`、`summary`、`operator_action`、`hint_command` 等字段",
        "非 completion 类结构化诊断会在紧凑输出里追加 `diag=<code>`，例如 `bridge_qmt_mode_not_live`",
        "`request_completed_order_terminal` / `request_completed_order_non_terminal` 不会重复显示为 `diag=<code>`；非终态完成仍沿用 `semantics=request_completed_order_non_terminal`",
        "reconciliation 会收敛 delayed fill、partial fill 与 `unknown` 恢复语义",
        "`live` adapter 仍未实现",
        "当前真实提交只走受 `qmt.mode=live` 保护的 `qmt_live` 路径",
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
        "quantix risk rule set --type volatility-limit --value 4%",
        "quantix risk sync industry --standard shenwan",
        "quantix risk rule set --type industry-blocklist --value 银行,地产",
        "quantix risk import live-trades --account <ID> --input <FILE>",
        "quantix risk rebuild live-account --account <ID>",
        "quantix risk rule enable --type position-limit",
        "quantix risk rule disable --type daily-loss-limit",
        "quantix risk status",
        "quantix risk pnl",
        "quantix risk position",
        "quantix risk status --source paper|live_import [--account <ID>]",
        "quantix risk pnl --source paper|live_import [--account <ID>]",
        "quantix risk position --source paper|live_import [--account <ID>]",
        "quantix risk log [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]",
        "quantix risk lock release",
        "QUANTIX_RISK_PATH",
        "QUANTIX_UPSTREAM_MYSQL_URL",
        "industry_reference.db",
        "`risk status`、`risk pnl`、`risk position` 依赖已初始化的 paper-trade 账户",
        "`--source live_import` 要求显式指定 `--account`",
        "风控 CLI 当前接受 `position-limit`、`daily-loss-limit`、`volatility-limit`、`industry-limit`、`auto-reduce`、`industry-blocklist` 六类 rule type",
        "`volatility-limit` 仅接受百分比值，例如 `4%`",
        "`volatility-limit` 固定使用 `ATR(14) / latest_close * 100`",
        "`volatility-limit` 缺少或不足日线时会拒绝买单",
        "当前真正已交付的运行时增强规则是 `volatility-limit`、`industry-limit` 与 `industry-blocklist`",
        "`industry-limit` 会按目标行业的买后集中度执行真实拦截",
        "`auto-reduce` 当前已交付 recommendation-only workflow：触发时会在 `risk status` / `risk pnl` / `risk position` 输出人工减仓建议，但不会自动卖出",
        "`industry-blocklist` 现已成为受支持的风险规则",
        "`risk sync industry --standard shenwan` 会刷新 `industry_reference_current` 和 `industry_reference_history`",
        "`risk sync industry --standard shenwan` 目前只支持 `shenwan`",
        "Phase 27D v1 使用 `SW 一级行业` 作为运行时生效标准",
        "`security_class_2024` / CSRC 2024 仍保留在系统中作为并行分类标准，不是该 v1 规则的运行时生效标准",
        "运行时风险评估只读取本地 SQLite reference/snapshot 表",
        "MySQL 仅作为上游同步来源，不参与运行时查询",
        "启用 `industry-blocklist` 或 `industry-limit` 前，先运行 `quantix risk sync industry --standard shenwan`",
        "如果本地 SQLite 行业引用表为空或未同步完成，`industry-blocklist` 与 `industry-limit` 都会 fail-closed 并拒绝买单",
        "运行时解析顺序：1. 当前 SW 映射 2. 查询月份快照 3. 历史 SW 映射 4. 最新本地快照",
        "月度快照会在该月第一次成功命中 `SW 一级行业` 时冻结",
        "`industry-blocklist` 采用精确字符串匹配，不做模糊归一化",
        "`industry-blocklist` 不影响卖出路径",
        "行业白名单继续延后到后续 Phase；`auto-reduce` 当前仅交付人工减仓建议，不自动执行卖出",
        "实盘导入当前只支持项目标准化 CSV/JSON",
        "live_import 镜像账户不会回写 `paper_trade.json`",
        "failed rebuild 不会覆盖上一次成功镜像状态",
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
        "| `live` | 通用 live 语义未实现；如需真实提交请改走 `qmt_live` request + `execution bridge qmt-live` |",
        "| `paper` | 模拟盘模式（当前支持 `ma_cross` 单次执行） |",
        "| `mock_live` | mock-live 模式（支持非终态订单生命周期模拟） |",
        "quantix strategy run -n ma_cross --mode paper -c 000001",
        "quantix strategy run -n ma_cross --mode mock_live -c 000001",
        "quantix strategy signal approve --signal-id <ID> --target-mode qmt_live --target-account <ACCOUNT>",
        "quantix execution bridge qmt-live --request-id <ID> [--yes]",
        "QUANTIX_STRATEGY_RUNTIME_DB_PATH",
        "~/.quantix/strategy/runtime.db",
        "首次使用前请先执行 `quantix trade init`",
        "`mock_live` 可能返回 `accepted`、`partially_filled`、`unknown` 等非终态状态",
        "live-ready hardening / reconciliation scaffolding",
        "不是真实 broker live execution",
        "同一个 mock-live 订单在 partial fill 路径下可能生成多笔 `TradeRecord`",
        "docs/standards/MOCK_USAGE_POLICY.md",
        "通用 `target_mode=live` 仍未实现",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn mock_usage_policy_documents_current_mock_and_real_boundary() {
    let policy_path = repo_root()
        .join("docs")
        .join("standards")
        .join("MOCK_USAGE_POLICY.md");
    let contents = fs::read_to_string(policy_path).expect("expected MOCK_USAGE_POLICY.md to exist");

    for expected in [
        "quantix anomaly run --mock",
        "quantix strategy run --mode mock_live",
        "`mock_live` 是模拟执行路径，不是真实 broker live execution",
        "真实提交路径应以当前受保护的 `qmt_live` 语义为准",
        "禁止真实路径失败后静默回落到 MOCK",
        "不能直接写成“真实能力完成”",
    ] {
        assert!(
            contents.contains(expected),
            "expected MOCK_USAGE_POLICY.md to contain {expected}"
        );
    }
}

#[test]
fn function_map_and_migration_docs_include_guarded_qmt_live_in_current_capability_summaries() {
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md to exist");
    let migration = fs::read_to_string(
        repo_root()
            .join("docs")
            .join("MIGRATION_FROM_DAILY_STOCK_ANALYSIS.md"),
    )
    .expect("expected MIGRATION_FROM_DAILY_STOCK_ANALYSIS.md to exist");

    for expected in [
        "本文件是 Quantix-Rust 当前主线唯一的功能全景图与状态注册表。",
        "换言之，本文档同时承担“单一功能注册表”职责。",
        "新增功能或设计项先在本注册表增加或更新节点，并显式标明状态、证据和边界",
        "`paper` + `mock_live` + guarded `qmt_live` | 当前已实现的执行目标",
        "Bridge v1 已支持受能力门控的 `qmt_live` 真实提交通道",
    ] {
        assert!(
            function_tree.contains(expected),
            "expected FUNCTION_TREE.md to contain {expected}"
        );
    }

    assert!(
        migration.contains("✅ paper/mock_live + guarded qmt_live"),
        "expected migration summary to include guarded qmt_live"
    );
}

#[test]
fn current_state_summary_docs_lock_mock_and_qmt_live_boundary() {
    let readme =
        fs::read_to_string(repo_root().join("README.md")).expect("expected README.md to exist");
    let gap_analysis = fs::read_to_string(repo_root().join("docs").join("GAP_ANALYSIS.md"))
        .expect("expected GAP_ANALYSIS.md to exist");
    let qmt_guide = fs::read_to_string(repo_root().join("docs").join("QMT_LIVE_TRADING_SETUP.md"))
        .expect("expected QMT_LIVE_TRADING_SETUP.md to exist");

    for expected in [
        "已有受能力门控的 `qmt_live` 真实提交通道",
        "通用 `target_mode=live` 统一语义",
        "仍无法宣称“通用自动化实盘交易能力已完整交付”",
    ] {
        assert!(
            gap_analysis.contains(expected),
            "expected GAP_ANALYSIS.md to contain {expected}"
        );
    }

    for expected in [
        "功能状态、能力边界、已设计/待实现项都以单一注册表为准",
        "功能全景图与状态注册表：[FUNCTION_TREE.md](FUNCTION_TREE.md)",
        "不再维护独立规划文档；新增计划或设计项先登记到 `FUNCTION_TREE.md`",
    ] {
        assert!(
            readme.contains(expected),
            "expected README.md to contain {expected}"
        );
    }

    for unexpected in [
        "ROADMAP.md](ROADMAP.md)",
        "DEVELOPMENT_ROADMAP.md",
        "ROADMAP_REVIEW.md",
    ] {
        assert!(
            !readme.contains(unexpected),
            "expected README.md not to contain {unexpected}"
        );
    }

    assert!(
        !repo_root().join("ROADMAP.md").exists(),
        "expected ROADMAP.md to be removed so FUNCTION_TREE.md remains the sole feature registry"
    );

    for expected in [
        "受能力门控的 `qmt_live` 真实提交流程",
        "通用 `target_mode=live` 仍未实现，不应与 `qmt_live` 混写",
        "`mock_live` 是模拟执行路径，不属于本指南覆盖范围",
        "功能全景图与状态注册表",
    ] {
        assert!(
            qmt_guide.contains(expected),
            "expected QMT_LIVE_TRADING_SETUP.md to contain {expected}"
        );
    }

    assert!(
        !qmt_guide.contains("历史开发路线图"),
        "expected QMT_LIVE_TRADING_SETUP.md not to mention the deleted historical roadmap"
    );
    assert!(
        !qmt_guide.contains("../ROADMAP.md"),
        "expected QMT_LIVE_TRADING_SETUP.md not to link to ROADMAP.md"
    );
}

#[test]
fn architecture_docs_keep_current_qmt_live_conclusion_and_legacy_roadmap_docs_are_removed() {
    let architecture = fs::read_to_string(
        repo_root()
            .join("docs")
            .join("architecture")
            .join("WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md"),
    )
    .expect("expected WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md to exist");

    for expected in [
        "`QMT preview contract` 仍然保留，作为 `qmt-preview` 路径",
        "受能力门控的 `qmt_live` 真实提交通道已经补齐",
        "通用 `target_mode=live` 仍未实现",
    ] {
        assert!(
            architecture.contains(expected),
            "expected architecture doc to contain {expected}"
        );
    }

    assert!(
        !repo_root().join("ROADMAP.md").exists(),
        "expected ROADMAP.md to be removed"
    );
    assert!(
        !repo_root().join("docs").join("ROADMAP_REVIEW.md").exists(),
        "expected ROADMAP_REVIEW.md to be removed"
    );
    assert!(
        !repo_root()
            .join("docs")
            .join("DEVELOPMENT_ROADMAP.md")
            .exists(),
        "expected DEVELOPMENT_ROADMAP.md to be removed"
    );
}

#[test]
fn active_doc_entrypoints_do_not_reference_deleted_legacy_roadmap_docs() {
    let active_docs = [
        repo_root().join("CHANGELOG.md"),
        repo_root().join("README.md"),
        repo_root().join("FUNCTION_TREE.md"),
        repo_root().join("docs").join("QMT_LIVE_TRADING_SETUP.md"),
    ];

    for doc_path in active_docs {
        let content = fs::read_to_string(&doc_path)
            .unwrap_or_else(|_| panic!("expected {doc_path:?} to exist"));

        for unexpected in [
            "ROADMAP.md",
            "FUNCTION_MAP.md",
            "DEVELOPMENT_ROADMAP.md",
            "ROADMAP_REVIEW.md",
            "路线图文档",
            "路线图文件",
            concat!("两份活跃", "事实源"),
            concat!("两份活跃规划/", "能力边界入口"),
            concat!("当前活跃的规划与能力边界", "文档仅保留两份"),
        ] {
            assert!(
                !content.contains(unexpected),
                "expected {doc_path:?} not to contain {unexpected}"
            );
        }
    }
}

#[test]
fn main_workspace_docs_do_not_reintroduce_parallel_roadmap_or_function_map_language() {
    let root = repo_root();
    let mut docs = Vec::new();
    collect_main_workspace_doc_paths(&root, &mut docs);

    assert!(
        !docs.is_empty(),
        "expected repository docs to be discoverable"
    );

    for doc_path in docs {
        let content = fs::read_to_string(&doc_path)
            .unwrap_or_else(|_| panic!("expected {doc_path:?} to be readable as UTF-8"));

        for unexpected in [
            "ROADMAP.md",
            "DEVELOPMENT_ROADMAP.md",
            "ROADMAP_REVIEW.md",
            "FUNCTION_MAP.md",
            "FUNCTION_MAP",
            "Backlog",
            "backlog",
            "roadmap",
            "ROADMAP",
            "路线图",
        ] {
            assert!(
                !content.contains(unexpected),
                "expected {doc_path:?} not to reintroduce parallel feature-truth term {unexpected}"
            );
        }
    }
}

#[test]
fn deployment_docs_do_not_publish_placeholder_support_email() {
    for relative_path in [
        "docs/guides/PRODUCTION_DEPLOYMENT.md",
        "docs/operations/DEPLOYMENT.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        assert!(
            !content.contains("support@example.com"),
            "expected {relative_path} not to publish placeholder support email"
        );
    }
}

#[test]
fn active_setup_docs_do_not_publish_machine_specific_absolute_paths() {
    for relative_path in [
        "docs/QMT_LIVE_TRADING_SETUP.md",
        "docs/architecture/WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        assert!(
            !content.contains("/opt/claude/quantix-rust"),
            "expected {relative_path} not to publish a machine-specific repository path"
        );
    }
}

#[test]
fn zellij_install_aliases_do_not_assume_home_repo_name() {
    let script_path = "scripts/zellij/install.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert!(
        !script.contains("~/quantix-rust/scripts/zellij/start-session.sh"),
        "expected {script_path} aliases not to assume the repository lives at ~/quantix-rust"
    );
    assert!(
        script.contains("$PROJECT_ROOT/scripts/zellij/start-session.sh"),
        "expected {script_path} aliases to derive start-session.sh from PROJECT_ROOT"
    );
}

#[test]
fn production_compose_requires_explicit_image_name() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));

    assert!(
        compose.contains("image: ${IMAGE_NAME:?IMAGE_NAME required}:${VERSION:-latest}"),
        "expected {compose_path} to require IMAGE_NAME instead of publishing a default image"
    );
    assert!(
        !compose.contains("image: ghcr.io/chengjon/quantix-rust/quantix:${VERSION:-latest}")
            && !compose.contains("${IMAGE_NAME:-ghcr.io/chengjon/quantix-rust/quantix}"),
        "expected {compose_path} not to default production deployments to the maintainer image"
    );

    for relative_path in [
        "docs/guides/PRODUCTION_DEPLOYMENT.md",
        "docs/operations/DEPLOYMENT.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        assert!(
            content.contains("IMAGE_NAME=ghcr.io/your-org/quantix-rust/quantix"),
            "expected {relative_path} to document the required IMAGE_NAME setting"
        );
    }
}

#[test]
fn production_compose_quantix_service_has_single_labels_block() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));
    let quantix_service = compose
        .split("\n  postgres:\n")
        .next()
        .expect("expected quantix service to precede postgres service");

    let labels_blocks = quantix_service
        .lines()
        .filter(|line| *line == "    labels:")
        .count();

    assert_eq!(
        labels_blocks, 1,
        "expected quantix service in {compose_path} to have a single labels block"
    );
    assert!(
        quantix_service.contains("\"prometheus.io/scrape=true\"")
            && quantix_service.contains("\"traefik.enable=true\""),
        "expected the single quantix labels block in {compose_path} to retain prometheus and traefik labels"
    );
}

#[test]
fn production_compose_requires_explicit_public_hosts() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));

    for expected in [
        r#"Host(`${QUANTIX_PUBLIC_HOST:?QUANTIX_PUBLIC_HOST required}`)"#,
        r#"Host(`${GRAFANA_PUBLIC_HOST:?GRAFANA_PUBLIC_HOST required}`)"#,
        r#"Host(`${TRAEFIK_PUBLIC_HOST:?TRAEFIK_PUBLIC_HOST required}`)"#,
        "GF_SERVER_ROOT_URL=https://${GRAFANA_PUBLIC_HOST:?GRAFANA_PUBLIC_HOST required}",
    ] {
        assert!(
            compose.contains(expected),
            "expected {compose_path} to require explicit public host setting {expected}"
        );
    }

    for stale_host in [
        "quantix.example.com",
        "grafana.example.com",
        "traefik.example.com",
    ] {
        assert!(
            !compose.contains(stale_host),
            "expected {compose_path} not to publish placeholder host {stale_host}"
        );
    }

    for relative_path in [
        "docs/guides/PRODUCTION_DEPLOYMENT.md",
        "docs/operations/DEPLOYMENT.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        for expected in [
            "QUANTIX_PUBLIC_HOST=quantix.your-domain.com",
            "GRAFANA_PUBLIC_HOST=grafana.your-domain.com",
            "TRAEFIK_PUBLIC_HOST=traefik.your-domain.com",
        ] {
            assert!(
                content.contains(expected),
                "expected {relative_path} to document required public host setting {expected}"
            );
        }

        for stale_url in [
            "https://quantix.example.com",
            "https://grafana.example.com",
            "https://traefik.example.com",
        ] {
            assert!(
                !content.contains(stale_url),
                "expected {relative_path} not to publish placeholder URL {stale_url}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn deploy_script_derives_default_image_name_from_github_repository() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-script-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");
    let production_k8s_dir = temp_dir.join("k8s/overlays/production");
    fs::create_dir_all(&production_k8s_dir).expect("expected fake production k8s dir");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    let output = std::process::Command::new("bash")
        .arg(repo_root().join("scripts/deploy/deploy.sh"))
        .arg("--environment")
        .arg("production")
        .arg("--dry-run")
        .env("PATH", path)
        .env("GITHUB_REPOSITORY", "example-org/example-repo")
        .env("PRODUCTION_K8S_DIR", &production_k8s_dir)
        .env_remove("IMAGE_NAME")
        .output()
        .expect("expected deploy script dry-run to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&temp_dir);

    assert!(
        output.status.success(),
        "expected deploy script dry-run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("ghcr.io/example-org/example-repo/quantix:latest"),
        "expected deploy script to derive image name from GITHUB_REPOSITORY\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("ghcr.io/chengjon/quantix-rust/quantix"),
        "expected deploy script not to default to the maintainer image\nstdout:\n{stdout}"
    );
}

#[cfg(unix)]
#[test]
fn deploy_script_reuses_configured_endpoint_values() {
    use std::os::unix::fs::PermissionsExt;

    let script_path = "scripts/deploy/deploy.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    for expected in [
        r#"HEALTH_URL="${HEALTH_URL:-http://localhost:8080/health}""#,
        r#"DEPLOY_ACCESS_URL="${DEPLOY_ACCESS_URL:-}""#,
        r#"curl -f "$HEALTH_URL""#,
    ] {
        assert!(
            script.contains(expected),
            "expected {script_path} to contain endpoint configuration {expected}"
        );
    }
    assert!(
        !script.contains("curl -f http://localhost:8080/health"),
        "expected {script_path} not to hardcode the health check URL"
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-endpoint-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");
    let production_k8s_dir = temp_dir.join("k8s/overlays/production");
    fs::create_dir_all(&production_k8s_dir).expect("expected fake production k8s dir");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    let output = std::process::Command::new("bash")
        .arg(repo_root().join(script_path))
        .arg("--environment")
        .arg("production")
        .arg("--dry-run")
        .env("PATH", path)
        .env("GITHUB_REPOSITORY", "example-org/example-repo")
        .env("PRODUCTION_K8S_DIR", &production_k8s_dir)
        .env("DEPLOY_ACCESS_URL", "https://deploy.example.invalid")
        .env_remove("IMAGE_NAME")
        .output()
        .expect("expected deploy script dry-run to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&temp_dir);

    assert!(
        output.status.success(),
        "expected deploy script dry-run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("生产环境访问: https://deploy.example.invalid"),
        "expected deploy script to log configured production access URL\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("https://quantix.example.com"),
        "expected deploy script not to publish placeholder production access URL\nstdout:\n{stdout}"
    );
}

#[cfg(unix)]
#[test]
fn deploy_script_requires_explicit_deploy_path_configuration() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-path-config-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    for (environment, missing_variable) in [
        ("staging", "STAGING_COMPOSE_FILE"),
        ("production", "PRODUCTION_K8S_DIR"),
    ] {
        let output = std::process::Command::new("bash")
            .arg(repo_root().join("scripts/deploy/deploy.sh"))
            .arg("--environment")
            .arg(environment)
            .arg("--dry-run")
            .env("PATH", &path)
            .env("GITHUB_REPOSITORY", "example-org/example-repo")
            .env_remove("IMAGE_NAME")
            .env_remove("STAGING_COMPOSE_FILE")
            .env_remove("PRODUCTION_K8S_DIR")
            .output()
            .unwrap_or_else(|_| {
                panic!("expected deploy script dry-run for {environment} to execute")
            });
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "expected deploy script dry-run for {environment} to reject missing {missing_variable}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(missing_variable) || stderr.contains(missing_variable),
            "expected deploy script dry-run for {environment} to mention missing {missing_variable}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn deployment_docs_show_explicit_deploy_path_configuration() {
    let doc_path = "docs/operations/DEPLOYMENT.md";
    let contents = fs::read_to_string(repo_root().join(doc_path))
        .unwrap_or_else(|_| panic!("expected {doc_path} to be readable"));

    for line in contents
        .lines()
        .filter(|line| line.contains("deploy.sh --environment production"))
    {
        assert!(
            line.contains("PRODUCTION_K8S_DIR="),
            "expected production deploy script example in {doc_path} to set PRODUCTION_K8S_DIR: {line}"
        );
    }
}

#[test]
fn health_check_script_reuses_response_matcher() {
    let script_path = "scripts/health-check.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert_eq!(
        script.matches("is_healthy_response()").count(),
        1,
        "expected {script_path} to define exactly one health response matcher"
    );
    assert!(
        script.contains(r#"printf '%s\n' "$response" | is_healthy_response"#),
        "expected curl branch in {script_path} to use the shared response matcher"
    );
    assert!(
        script.contains(r#"wget -q --timeout="$TIMEOUT" -O - "$HEALTH_URL" | is_healthy_response"#),
        "expected wget branch in {script_path} to use the shared response matcher"
    );
    assert!(
        !script.contains(
            r#"echo "$response" | grep -q '"status":"ok"' || echo "$response" | grep -q '"healthy":true'"#
        ),
        "expected {script_path} not to duplicate the curl response matcher"
    );
}

#[test]
fn runtime_install_services_script_reuses_service_list() {
    let script_path = "scripts/runtime/install-services.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert!(
        script.contains("SERVICES=("),
        "expected {script_path} to define one shared service list"
    );
    assert!(
        script
            .matches(r#"for service in "${SERVICES[@]}"; do"#)
            .count()
            >= 2,
        "expected {script_path} to reuse the shared service list for enablement and display"
    );
    assert!(
        script.contains(r#"systemctl enable "$service" 2>/dev/null || true"#),
        "expected {script_path} to enable services from the shared list"
    );
    for service in [
        "quantix-data-collector.service",
        "quantix-strategy-runner.service",
        "quantix-task-scheduler.service",
    ] {
        assert_eq!(
            script.matches(service).count(),
            1,
            "expected {service} to appear only in the shared service list in {script_path}"
        );
    }
    assert!(
        script.contains("./scripts/runtime/services.sh"),
        "expected {script_path} to point operators at the runtime service manager"
    );
    assert!(
        !script.contains("./scripts/services.sh"),
        "expected {script_path} not to reference the old service manager path"
    );
}

#[test]
fn runtime_services_script_reuses_service_validation() {
    let script_path = "scripts/runtime/services.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert!(
        script.contains("require_valid_service()"),
        "expected {script_path} to define one shared service validation helper"
    );
    for action in [
        "start", "stop", "restart", "status", "logs", "enable", "disable",
    ] {
        let case_arm = format!("{action})");
        let arm_start = script
            .find(&case_arm)
            .unwrap_or_else(|| panic!("expected {script_path} to define {action} action"));
        let arm = &script[arm_start..];
        let arm_end = arm.find(";;").unwrap_or_else(|| {
            panic!("expected {script_path} {action} action to terminate with ;;")
        });
        assert!(
            arm[..arm_end].contains(r#"require_valid_service "$SERVICE""#),
            "expected {script_path} {action} action to use the shared service validator"
        );
    }
    assert_eq!(
        script.matches(r#"if [ -z "$SERVICE" ]"#).count(),
        0,
        "expected {script_path} not to duplicate missing service checks in each action"
    );
    assert_eq!(
        script
            .matches(r#"if [ -z "${SERVICES[$SERVICE]}" ]"#)
            .count(),
        0,
        "expected {script_path} not to duplicate unknown service checks in each action"
    );
    assert!(
        script.contains(r#"get_service_name "$1""#),
        "expected {script_path} service lookups to quote the requested service name"
    );
}

#[test]
fn importer_reuses_date_parser_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parse_date(value: &str, format: &str) -> Result<NaiveDate>"),
        "expected {source_path} to define one reusable importer date parser helper"
    );
    assert!(
        source.contains("let date = parse_date(&row.date, &self.config.date_format)?;"),
        "expected csv_row_to_kline in {source_path} to use the reusable date parser"
    );
    assert!(
        source.contains("let date = parse_date(&row.date, \"%Y-%m-%d\")?;"),
        "expected json_row_to_kline in {source_path} to use the reusable date parser"
    );

    let csv_row_start = source
        .find("fn csv_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define csv_row_to_kline"));
    let csv_row_body = &source[csv_row_start
        ..source[csv_row_start..]
            .find("fn json_row_to_kline")
            .map(|offset| csv_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !csv_row_body.contains("NaiveDate::parse_from_str"),
        "expected csv_row_to_kline in {source_path} not to inline date parsing"
    );

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !json_row_body.contains("NaiveDate::parse_from_str"),
        "expected json_row_to_kline in {source_path} not to inline date parsing"
    );
}

#[test]
fn importer_reuses_required_decimal_parser_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains(
            "fn parse_required_decimal(value: &str, field_name: &str) -> Result<Decimal>"
        ),
        "expected {source_path} to define one reusable required decimal parser helper"
    );

    for expected_call in [
        "parse_required_decimal(&row.open, \"open\")?",
        "parse_required_decimal(&row.high, \"high\")?",
        "parse_required_decimal(&row.low, \"low\")?",
        "parse_required_decimal(&row.close, \"close\")?",
        ".map(|s| parse_required_decimal(s, \"amount\"))",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let csv_row_start = source
        .find("fn csv_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define csv_row_to_kline"));
    let csv_row_body = &source[csv_row_start
        ..source[csv_row_start..]
            .find("fn json_row_to_kline")
            .map(|offset| csv_row_start + offset)
            .unwrap_or(source.len())];
    for inline_parse in [
        "Decimal::from_str(&row.open)",
        "Decimal::from_str(&row.high)",
        "Decimal::from_str(&row.low)",
        "Decimal::from_str(&row.close)",
        "Decimal::from_str(s).map_err",
    ] {
        assert!(
            !csv_row_body.contains(inline_parse),
            "expected csv_row_to_kline in {source_path} not to inline `{inline_parse}`"
        );
    }

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    for inline_parse in [
        "Decimal::from_str(&row.open)",
        "Decimal::from_str(&row.close)",
    ] {
        assert!(
            !json_row_body.contains(inline_parse),
            "expected json_row_to_kline in {source_path} not to inline `{inline_parse}`"
        );
    }
}

#[test]
fn importer_reuses_optional_decimal_default_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parse_optional_decimal_or_default(value: Option<&str>) -> Decimal"),
        "expected {source_path} to define one reusable optional decimal default helper"
    );
    for expected_call in [
        "high: parse_optional_decimal_or_default(row.high.as_deref()),",
        "low: parse_optional_decimal_or_default(row.low.as_deref()),",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let json_row_start = source
        .find("fn json_row_to_kline")
        .unwrap_or_else(|| panic!("expected {source_path} to define json_row_to_kline"));
    let json_row_body = &source[json_row_start
        ..source[json_row_start..]
            .find("fn parse_date")
            .map(|offset| json_row_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !json_row_body.contains(".map(|s| Decimal::from_str(s).unwrap_or_default())"),
        "expected json_row_to_kline in {source_path} not to inline optional decimal defaults"
    );
}

#[test]
fn importer_reuses_parquet_date_conversion_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn date_from_parquet_day(days: i32) -> Result<NaiveDate>"),
        "expected {source_path} to define one reusable Parquet Date32 conversion helper"
    );

    let import_parquet_start = source
        .find("fn import_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define import_parquet"));
    let import_parquet_end = [
        "fn csv_row_to_kline",
        "fn json_row_to_kline",
        "fn parse_date",
        "fn date_from_parquet_day",
        "fn parse_required_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[import_parquet_start..]
            .find(marker)
            .map(|offset| import_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let import_parquet_body = &source[import_parquet_start..import_parquet_end];

    assert!(
        import_parquet_body.contains("date_from_parquet_day(dates.value(i))?"),
        "expected import_parquet in {source_path} to call date_from_parquet_day"
    );
    assert!(
        !import_parquet_body.contains("NaiveDate::from_ymd_opt(1970, 1, 1)"),
        "expected import_parquet in {source_path} not to inline Parquet Date32 epoch conversion"
    );
    assert!(
        !import_parquet_body.contains("chrono::Duration::days("),
        "expected import_parquet in {source_path} not to inline Parquet Date32 day offsets"
    );
}

#[test]
fn importer_reuses_parquet_decimal_conversion_helper() {
    let source_path = "src/io/importer.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn decimal_from_f64_or_default(value: f64) -> Decimal"),
        "expected {source_path} to define one reusable Parquet f64 Decimal conversion helper"
    );

    let import_parquet_start = source
        .find("fn import_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define import_parquet"));
    let import_parquet_end = [
        "fn csv_row_to_kline",
        "fn json_row_to_kline",
        "fn parse_date",
        "fn decimal_from_f64_or_default",
        "fn parse_required_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[import_parquet_start..]
            .find(marker)
            .map(|offset| import_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let import_parquet_body = &source[import_parquet_start..import_parquet_end];

    for expected_call in [
        "decimal_from_f64_or_default(opens.value(i))",
        "decimal_from_f64_or_default(0.0)",
        "decimal_from_f64_or_default(closes.value(i))",
    ] {
        assert!(
            import_parquet_body.contains(expected_call),
            "expected import_parquet in {source_path} to call `{expected_call}`"
        );
    }

    assert!(
        !import_parquet_body.contains("Decimal::from_f64_retain("),
        "expected import_parquet in {source_path} not to inline Parquet f64 Decimal conversion"
    );
}

#[test]
fn exporter_reuses_decimal_formatter_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn format_decimal(value: Decimal, precision: usize) -> String"),
        "expected {source_path} to define one reusable CSV decimal formatter helper"
    );
    for expected_call in [
        "format_decimal(kline.open, decimal_precision)",
        "format_decimal(kline.high, decimal_precision)",
        "format_decimal(kline.low, decimal_precision)",
        "format_decimal(kline.close, decimal_precision)",
    ] {
        assert!(
            source.contains(expected_call),
            "expected {source_path} to contain `{expected_call}`"
        );
    }

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];
    assert!(
        !export_csv_body.contains("\"{:.prec$}\""),
        "expected export_csv in {source_path} not to inline Decimal precision formatting"
    );
}

#[test]
fn exporter_reuses_csv_kline_record_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains(
            "fn csv_kline_record(kline: &Kline, decimal_precision: usize) -> [String; 9]"
        ),
        "expected {source_path} to define one reusable CSV kline record helper"
    );
    assert!(
        source.contains("csv_kline_record(kline, self.config.decimal_precision)"),
        "expected export_csv in {source_path} to use the CSV kline record helper"
    );

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];

    for inline_detail in [
        "kline.date.to_string()",
        "format_decimal(kline.open",
        "format_decimal(kline.high",
        "format_decimal(kline.low",
        "format_decimal(kline.close",
        "format!(\"{:?}\", kline.adjust_type)",
    ] {
        assert!(
            !export_csv_body.contains(inline_detail),
            "expected export_csv in {source_path} not to inline CSV record field `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_csv_kline_header_constant() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    let header_start = source
        .find("const CSV_KLINE_HEADER: [&str; 9] = [")
        .unwrap_or_else(|| {
            panic!("expected {source_path} to define one reusable CSV kline header constant")
        });
    let header_body = &source[header_start
        ..source[header_start..]
            .find("];")
            .map(|offset| header_start + offset)
            .unwrap_or(source.len())];
    for expected_header in [
        "\"code\"",
        "\"date\"",
        "\"open\"",
        "\"high\"",
        "\"low\"",
        "\"close\"",
        "\"volume\"",
        "\"amount\"",
        "\"adjust_type\"",
    ] {
        assert!(
            header_body.contains(expected_header),
            "expected CSV_KLINE_HEADER in {source_path} to contain `{expected_header}`"
        );
    }

    assert!(
        source.contains("wtr.write_record(CSV_KLINE_HEADER)"),
        "expected export_csv in {source_path} to use CSV_KLINE_HEADER"
    );

    let export_csv_start = source
        .find("fn export_csv")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_csv"));
    let export_csv_body = &source[export_csv_start
        ..source[export_csv_start..]
            .find("fn export_json")
            .map(|offset| export_csv_start + offset)
            .unwrap_or(source.len())];

    for inline_header in [
        "\"code\"",
        "\"date\"",
        "\"open\"",
        "\"high\"",
        "\"low\"",
        "\"close\"",
        "\"volume\"",
        "\"amount\"",
        "\"adjust_type\"",
    ] {
        assert!(
            !export_csv_body.contains(inline_header),
            "expected export_csv in {source_path} not to inline CSV header field `{inline_header}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_decimal_conversion_helpers() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_helper in [
        "fn decimal_to_f64_or_zero(value: Decimal) -> f64",
        "fn optional_decimal_to_f64_or_zero(value: Option<Decimal>) -> f64",
    ] {
        assert!(
            source.contains(expected_helper),
            "expected {source_path} to define `{expected_helper}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = ["fn decimal_to_f64_or_zero", "fn format_decimal"]
        .iter()
        .filter_map(|marker| {
            source[export_parquet_start..]
                .find(marker)
                .map(|offset| export_parquet_start + offset)
        })
        .min()
        .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    for expected_call in [
        "decimal_to_f64_or_zero(k.open)",
        "decimal_to_f64_or_zero(k.high)",
        "decimal_to_f64_or_zero(k.low)",
        "decimal_to_f64_or_zero(k.close)",
        "optional_decimal_to_f64_or_zero(k.amount)",
    ] {
        assert!(
            export_parquet_body.contains(expected_call),
            "expected export_parquet in {source_path} to call `{expected_call}`"
        );
    }

    for inline_detail in [
        ".to_f64().unwrap_or(0.0)",
        ".map(|a| a.to_f64().unwrap_or(0.0)).unwrap_or(0.0)",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Decimal conversion `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_date_conversion_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn date_to_parquet_day(date: NaiveDate) -> i32"),
        "expected {source_path} to define one reusable Parquet Date32 conversion helper"
    );

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains(".map(|k| date_to_parquet_day(k.date))"),
        "expected export_parquet in {source_path} to map dates through date_to_parquet_day"
    );
    for inline_detail in [
        "NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()",
        ".signed_duration_since(",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet date conversion `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_schema_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    assert!(
        source.contains("fn parquet_kline_schema() -> arrow::datatypes::Schema"),
        "expected {source_path} to define one reusable Parquet kline schema helper"
    );

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains("let schema = parquet_kline_schema();"),
        "expected export_parquet in {source_path} to use parquet_kline_schema"
    );
    for inline_detail in ["Schema::new(vec![", "Field::new("] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet schema detail `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_record_batch_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_signature_part in [
        "fn parquet_kline_record_batch(",
        "schema: &arrow::datatypes::Schema",
        "klines: &[Kline]",
        ") -> Result<RecordBatch>",
    ] {
        assert!(
            source.contains(expected_signature_part),
            "expected {source_path} to define one reusable Parquet kline RecordBatch helper containing `{expected_signature_part}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn parquet_kline_record_batch",
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body.contains("let batch = parquet_kline_record_batch(&schema, klines)?;"),
        "expected export_parquet in {source_path} to use parquet_kline_record_batch"
    );
    for inline_detail in [
        "StringArray::from(",
        "Float64Array::from(",
        "PrimitiveArray::<",
        "RecordBatch::try_new(",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline RecordBatch detail `{inline_detail}`"
        );
    }
}

#[test]
fn exporter_reuses_parquet_writer_helper() {
    let source_path = "src/io/exporter.rs";
    let source = fs::read_to_string(repo_root().join(source_path))
        .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));

    for expected_signature_part in [
        "fn write_parquet_record_batch(",
        "path: &Path",
        "schema: arrow::datatypes::Schema",
        "batch: &RecordBatch",
        ") -> Result<()>",
    ] {
        assert!(
            source.contains(expected_signature_part),
            "expected {source_path} to define one reusable Parquet writer helper containing `{expected_signature_part}`"
        );
    }

    let export_parquet_start = source
        .find("fn export_parquet")
        .unwrap_or_else(|| panic!("expected {source_path} to define export_parquet"));
    let export_parquet_end = [
        "fn write_parquet_record_batch",
        "fn parquet_kline_record_batch",
        "fn parquet_kline_schema",
        "fn date_to_parquet_day",
        "fn decimal_to_f64_or_zero",
        "fn format_decimal",
    ]
    .iter()
    .filter_map(|marker| {
        source[export_parquet_start..]
            .find(marker)
            .map(|offset| export_parquet_start + offset)
    })
    .min()
    .unwrap_or(source.len());
    let export_parquet_body = &source[export_parquet_start..export_parquet_end];

    assert!(
        export_parquet_body
            .contains("write_parquet_record_batch(output_path.as_ref(), schema, &batch)"),
        "expected export_parquet in {source_path} to use write_parquet_record_batch"
    );
    for inline_detail in [
        "File::create(",
        "WriterProperties::builder()",
        "ArrowWriter::try_new(",
        "writer.write(",
        "writer.close()",
    ] {
        assert!(
            !export_parquet_body.contains(inline_detail),
            "expected export_parquet in {source_path} not to inline Parquet writer detail `{inline_detail}`"
        );
    }
}

#[test]
fn unit_tests_avoid_tcp_backed_default_source_unwraps() {
    for source_path in [
        "src/sources/quote_collector.rs",
        "src/sources/tdx.rs",
        "src/tasks/collect_scheduler.rs",
    ] {
        let source = fs::read_to_string(repo_root().join(source_path))
            .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));
        let tests_start = source
            .find("#[cfg(test)]")
            .unwrap_or_else(|| panic!("expected {source_path} to define test module"));
        let test_source = &source[tests_start..];
        for forbidden in [
            "TdxSource::with_default_config()",
            "QuoteCollector::with_default_config()",
            "with_default_config().unwrap()",
            "with_default_config().expect(",
        ] {
            assert!(
                !test_source.contains(forbidden),
                "expected tests in {source_path} not to call `{forbidden}`; use offline_tdx_source() for unit tests that do not need live TDX connectivity"
            );
        }
    }
}

#[test]
fn unit_tests_reuse_offline_tdx_source_helper() {
    let tdx_source_path = "src/sources/tdx.rs";
    let tdx_source = fs::read_to_string(repo_root().join(tdx_source_path))
        .unwrap_or_else(|_| panic!("expected {tdx_source_path} to be readable"));
    assert!(
        tdx_source.contains("pub(crate) fn offline_tdx_source() -> TdxSource"),
        "expected {tdx_source_path} to define one shared offline TDX source test helper"
    );
    let helper_start = tdx_source
        .find("pub(crate) fn offline_tdx_source() -> TdxSource")
        .unwrap_or_else(|| panic!("expected {tdx_source_path} to define offline_tdx_source"));
    let helper_body = &tdx_source[helper_start
        ..tdx_source[helper_start..]
            .find("\n}\n")
            .map(|offset| helper_start + offset)
            .unwrap_or(tdx_source.len())];
    assert!(
        !helper_body.contains("TdxSource::new("),
        "expected offline_tdx_source() in {tdx_source_path} not to call TdxSource::new(), which creates TCP handles"
    );

    for source_path in [
        "src/sources/quote_collector.rs",
        "src/sources/tdx.rs",
        "src/tasks/collect_scheduler.rs",
    ] {
        let source = fs::read_to_string(repo_root().join(source_path))
            .unwrap_or_else(|_| panic!("expected {source_path} to be readable"));
        let tests_start = source
            .find("#[cfg(test)]")
            .unwrap_or_else(|| panic!("expected {source_path} to define test module"));
        let test_source = &source[tests_start..];
        assert!(
            !test_source.contains("TdxSource::new(1, vec![], 7709, 10).unwrap()"),
            "expected tests in {source_path} to use offline_tdx_source() instead of hand-written offline TDX source construction"
        );
    }
}

#[test]
fn main_workspace_status_bearing_docs_defer_to_function_tree_registry() {
    let root = repo_root();
    let mut docs = Vec::new();
    collect_main_workspace_doc_paths(&root, &mut docs);

    let status_terms = [
        "已实现",
        "部分实现",
        "待实现",
        "未实现",
        "不可用",
        "当前可用",
        "当前不可用",
        "已设计",
        "非目标",
        "可用能力",
        "功能状态",
        "状态注册表",
    ];

    for doc_path in docs {
        let relative = doc_path
            .strip_prefix(&root)
            .unwrap_or(&doc_path)
            .to_string_lossy();
        if relative == "FUNCTION_TREE.md" || relative.starts_with("logs/") {
            continue;
        }

        let content = fs::read_to_string(&doc_path)
            .unwrap_or_else(|_| panic!("expected {doc_path:?} to be readable as UTF-8"));
        if !status_terms
            .iter()
            .any(|status_term| content.contains(status_term))
        {
            continue;
        }

        assert!(
            content.contains("FUNCTION_TREE.md"),
            "expected status-bearing doc {doc_path:?} to point readers to FUNCTION_TREE.md"
        );
        assert!(
            content.contains("状态注册表"),
            "expected status-bearing doc {doc_path:?} to defer to FUNCTION_TREE.md status registry"
        );
        assert!(
            content.contains("不作为功能状态注册表") || content.contains("不是功能状态注册表"),
            "expected status-bearing doc {doc_path:?} not to present itself as a feature status registry"
        );
    }
}

#[test]
fn status_bearing_docs_defer_to_function_tree_registry() {
    let docs = [
        repo_root().join("docs").join("GAP_ANALYSIS.md"),
        repo_root().join("docs").join("USER_MANUAL.md"),
        repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"),
        repo_root()
            .join("docs")
            .join("standards")
            .join("DEVELOPMENT_GUIDELINES.md"),
    ];

    for doc_path in docs {
        let content = fs::read_to_string(&doc_path)
            .unwrap_or_else(|_| panic!("expected {doc_path:?} to exist"));
        assert!(
            content.contains("FUNCTION_TREE.md"),
            "expected {doc_path:?} to point status readers to FUNCTION_TREE.md"
        );
        assert!(
            content.contains("状态注册表"),
            "expected {doc_path:?} to defer to the FUNCTION_TREE.md status registry"
        );
        assert!(
            content.contains("不作为功能状态注册表") || content.contains("不是功能状态注册表"),
            "expected {doc_path:?} not to present itself as a feature status registry"
        );
    }
}

#[test]
fn function_tree_status_registry_is_single_source_of_status() {
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md to exist");

    for expected in [
        "本文件是 Quantix-Rust 当前主线唯一的功能全景图与状态注册表。",
        "换言之，本文档同时承担“单一功能注册表”职责。",
        "新增功能或设计项先在本注册表增加或更新节点，并显式标明状态、证据和边界",
        "| 功能节点 | 状态 | 证据 | 边界 |",
        "只以“状态注册表”中的表格行作为状态真相",
        "后续树状代码块是证据摘录",
    ] {
        assert!(
            function_tree.contains(expected),
            "expected FUNCTION_TREE.md to contain {expected}"
        );
    }

    for unexpected in ["[未实现]", "[待实现]"] {
        assert!(
            !function_tree.contains(unexpected),
            "expected FUNCTION_TREE.md not to contain legacy status tag {unexpected}"
        );
    }

    let status_tags = ["[已实现]", "[部分实现]", "[已设计/待实现]", "[非目标]"];
    let mut in_code_fence = false;

    for (line_index, line) in function_tree.lines().enumerate() {
        if line.starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }

        if in_code_fence {
            for status_tag in status_tags {
                assert!(
                    !line.contains(status_tag),
                    "expected FUNCTION_TREE.md evidence code fence line {} not to contain status tag {status_tag}",
                    line_index + 1
                );
            }
        }
    }

    assert!(
        !in_code_fence,
        "expected FUNCTION_TREE.md code fences to be balanced"
    );

    let mut status_registry_rows = 0;
    let mut designed_registry_rows = 0;

    for (line_index, line) in function_tree.lines().enumerate() {
        if !(line.starts_with('|')
            && status_tags
                .iter()
                .any(|status_tag| line.contains(status_tag)))
        {
            continue;
        }

        let cells: Vec<_> = line
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect();
        assert!(
            cells.len() >= 4,
            "expected FUNCTION_TREE.md status registry line {} to have feature/status/evidence/boundary cells: {line}",
            line_index + 1
        );
        assert!(
            !cells[0].is_empty() && !cells[2].is_empty() && !cells[3].is_empty(),
            "expected FUNCTION_TREE.md status registry line {} to include non-empty feature/evidence/boundary cells: {line}",
            line_index + 1
        );
        assert!(
            status_tags.iter().any(|status_tag| cells[1] == *status_tag),
            "expected FUNCTION_TREE.md status registry line {} to keep status in the second cell: {line}",
            line_index + 1
        );

        status_registry_rows += 1;
        if cells[1] == "[已设计/待实现]" {
            designed_registry_rows += 1;
        }
    }

    assert!(
        status_registry_rows >= 90,
        "expected FUNCTION_TREE.md to keep status/evidence/boundary registry rows; found {status_registry_rows}"
    );
    assert!(
        designed_registry_rows >= 15,
        "expected FUNCTION_TREE.md to keep explicit designed/pending registry rows; found {designed_registry_rows}"
    );
}

#[test]
fn generated_cli_manual_keeps_qmt_and_mock_boundary() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "QMT 执行兼容入口",
        "quantix execution qmt preview",
        "quantix execution qmt live",
        "提交真实订单 (需要确认)",
        "不直接提交订单",
        "使用模拟数据（测试用）",
        "模拟数据股票数量（仅与 --mock 一起使用）",
        "运行模式: backtest | paper | mock_live | live（live 会被拒绝并提示改走 qmt_live request / execution bridge；不支持直接传 qmt_live）",
        "账户类型: paper | mock_live | qmt_live（兼容 live 别名）",
        "目标执行模式: paper | mock_live | qmt_live（live 将被拒绝并提示改走 qmt_live）",
        "目标模式过滤: paper | mock_live | qmt_live | live（legacy rejected mode）",
        "bridge qmt.mode=live",
        "`qmt.supports` 包含 `order_submit`",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to contain {expected}"
        );
    }
}

#[test]
fn generated_cli_manual_documents_market_sort_by_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "quantix market sector",
        "quantix market concept",
        "未知排序字段会在读取 ClickHouse 或输出板块表格前返回显式 <code>Unsupported</code>",
        "错误包含 <code>不支持的 sort_by</code>",
        "支持列表 <code>change, change_pct</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document market sort_by fail-closed boundary: {expected}"
        );
    }

    assert!(
        !cli_manual.contains("未知排序字段会静默回退"),
        "CLI_COMMAND_MANUAL.html still describes market sort_by as silently falling back"
    );
}

#[test]
fn generated_cli_manual_documents_data_export_parquet_as_wired() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual
            .contains("<code>csv</code> 和 <code>parquet</code> 分支都会调用导出器写出实际文件"),
        "expected CLI_COMMAND_MANUAL.html to document CSV/Parquet data export as wired"
    );
    assert!(
        cli_manual.contains("错误包含 <code>data export format 不支持</code>"),
        "expected CLI_COMMAND_MANUAL.html to document data export unknown format fail-closed boundary"
    );
    assert!(
        cli_manual.contains("未知 <code>--format</code> 应在读取数据前失败关闭且不输出导出提示"),
        "expected CLI_COMMAND_MANUAL.html to document data export validates format before data reads"
    );
    for stale in [
        "当前只有 <code>csv</code> 分支真正落盘",
        "Parquet 导出暂未实现",
        "把 Parquet 占位实现误判为已完成能力",
        "未知格式会继续读取 ClickHouse",
    ] {
        assert!(
            !cli_manual.contains(stale),
            "CLI_COMMAND_MANUAL.html still contains stale data export boundary: {stale}"
        );
    }
}

#[test]
fn generated_cli_manual_documents_fundamental_dividend_as_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains(
            "<code>capital-flow</code> 与 <code>dividend</code> 已有命令壳，但真实资金流向/分红数据源尚未接线，执行时会返回显式 <code>Unsupported</code>"
        ),
        "expected CLI_COMMAND_MANUAL.html to document fundamental dividend as fail-closed"
    );
    assert!(
        !cli_manual.contains("<code>dividend</code> 仍是占位输出"),
        "CLI_COMMAND_MANUAL.html still describes fundamental dividend as placeholder output"
    );
}

#[test]
fn generated_cli_manual_documents_fundamental_capital_flow_as_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("<code>capital-flow</code> 与 <code>dividend</code> 已有命令壳"),
        "expected CLI_COMMAND_MANUAL.html to document fundamental capital-flow as a command shell"
    );
    assert!(
        cli_manual.contains("quantix fundamental capital-flow [OPTIONS] --code &lt;CODE&gt;"),
        "expected CLI_COMMAND_MANUAL.html to include the fundamental capital-flow leaf command"
    );
    assert!(
        cli_manual.contains("资金流向数据源尚未接入，并返回显式 Unsupported"),
        "expected CLI_COMMAND_MANUAL.html to document fundamental capital-flow as fail-closed"
    );
    assert!(
        !cli_manual.contains("<code>quantix fundamental capital-flow</code> 目前还没有 CLI 入口"),
        "CLI_COMMAND_MANUAL.html still describes fundamental capital-flow as missing a CLI entry"
    );
}

#[test]
fn generated_cli_manual_documents_import_from_excel_as_parser_backed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("quantix import from-excel [OPTIONS] --file &lt;FILE&gt;"),
        "expected CLI_COMMAND_MANUAL.html to include the import from-excel leaf command"
    );
    assert!(
        cli_manual.contains(
            "<code>quantix import from-excel</code> 会读取首个或指定 worksheet 的 watchlist 代码/名称行，并输出解析结果"
        ),
        "expected CLI_COMMAND_MANUAL.html to document import from-excel as parser-backed"
    );
    assert!(
        !cli_manual.contains("真实 Excel parser 尚未接线"),
        "CLI_COMMAND_MANUAL.html still describes import from-excel as parser-unwired"
    );
    assert!(
        !cli_manual.contains("执行时会返回显式 <code>Unsupported</code>；如果需要实际处理 Excel"),
        "CLI_COMMAND_MANUAL.html still describes import from-excel as fail-closed"
    );
    assert!(
        !cli_manual.contains("<code>quantix import from-excel</code> 仍无 CLI 入口"),
        "CLI_COMMAND_MANUAL.html still describes import from-excel as missing a CLI entry"
    );
}

#[test]
fn generated_cli_manual_documents_ai_config_test_as_non_connectivity_check() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("输出“已配置（未发起真实连通性测试）”"),
        "expected CLI_COMMAND_MANUAL.html to document ai config --test as a non-connectivity check"
    );
    assert!(
        cli_manual.contains("运行时标题为“检查 LLM 配置状态”"),
        "expected CLI_COMMAND_MANUAL.html to document the ai config --test runtime status-check heading"
    );
    assert!(
        !cli_manual.contains("输出“可用”占位结果"),
        "CLI_COMMAND_MANUAL.html still describes ai config --test as printing a fake availability result"
    );
    assert!(
        !cli_manual.contains("测试 LLM 连通性"),
        "CLI_COMMAND_MANUAL.html still describes ai config --test as a real connectivity test"
    );
}

#[test]
fn generated_cli_manual_documents_ai_unwired_provider_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains(
            "ai analyze/decide/ask/market</code> 在未配置 DeepSeek/OpenAI/Ollama provider 时会返回显式 <code>Unsupported</code>"
        ),
        "expected CLI_COMMAND_MANUAL.html to document missing wired AI provider fail-closed behavior"
    );
    assert!(
        cli_manual.contains("AI provider 尚未配置"),
        "expected CLI_COMMAND_MANUAL.html to include the missing AI provider boundary error text"
    );
    assert!(
        cli_manual.contains("runtime 只会选择已接线的 DeepSeek、OpenAI 或 Ollama adapter"),
        "expected CLI_COMMAND_MANUAL.html to document wired AI runtime providers"
    );
    assert!(
        cli_manual.contains("只配置 Gemini/Anthropic 等未接线 provider，会失败关闭而不是静默回退"),
        "expected CLI_COMMAND_MANUAL.html to document unwired AI provider fail-closed behavior"
    );
    assert!(
        cli_manual.contains("<code>ai config</code> 仍用于查看配置状态"),
        "expected CLI_COMMAND_MANUAL.html to preserve ai config as the status-view entry"
    );
    assert!(
        !cli_manual.contains("未配置任何 LLM 提供商"),
        "CLI_COMMAND_MANUAL.html still documents the old successful missing-LLM prompt"
    );
}

#[test]
fn generated_cli_manual_documents_sentiment_provider_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains(
            "sentiment show/history/mentions</code> 当前返回显式 <code>Unsupported</code>"
        ),
        "expected CLI_COMMAND_MANUAL.html to document sentiment provider fail-closed behavior"
    );
    assert!(
        cli_manual.contains("sentiment provider 尚未接线"),
        "expected CLI_COMMAND_MANUAL.html to include the sentiment provider boundary error text"
    );
    assert!(
        !cli_manual.contains("输出模板验收"),
        "CLI_COMMAND_MANUAL.html still describes sentiment as placeholder output verification"
    );
    assert!(
        !cli_manual.contains("空 provider 列表创建"),
        "CLI_COMMAND_MANUAL.html still documents an empty-provider sentiment aggregator path"
    );
    assert!(
        !cli_manual.contains("默认聚合结果和空来源提示"),
        "CLI_COMMAND_MANUAL.html still describes sentiment show as a successful empty aggregate"
    );
}

#[test]
fn generated_cli_manual_documents_news_missing_provider_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("news search/code/trend</code> 当前返回显式 <code>Unsupported</code>"),
        "expected CLI_COMMAND_MANUAL.html to document news missing-provider fail-closed behavior"
    );
    assert!(
        cli_manual.contains("news provider 尚未配置"),
        "expected CLI_COMMAND_MANUAL.html to include the news provider boundary error text"
    );
    assert!(
        cli_manual.contains("<code>providers</code> 则用于查看三类搜索后端的环境准备情况"),
        "expected CLI_COMMAND_MANUAL.html to keep news providers as status-only"
    );
    assert!(
        !cli_manual.contains("未配置 provider 时会输出显式配置提示"),
        "CLI_COMMAND_MANUAL.html still describes missing news providers as a successful stdout prompt"
    );
    assert!(
        !cli_manual.contains("未配置时会提示需要配置搜索 provider"),
        "CLI_COMMAND_MANUAL.html still describes news search missing-provider behavior as prompt-only"
    );
    assert!(
        !cli_manual.contains("未配置 provider 时同样返回配置提示"),
        "CLI_COMMAND_MANUAL.html still describes news code missing-provider behavior as prompt-only"
    );
}

#[test]
fn generated_cli_manual_documents_notify_check_and_test_missing_config_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("notify channel 不支持"),
        "expected CLI_COMMAND_MANUAL.html to include the notify channel unsupported boundary"
    );
    assert!(
        cli_manual.contains("notify channel 尚未配置"),
        "expected CLI_COMMAND_MANUAL.html to include the notify channel missing-config boundary"
    );
    assert!(
        cli_manual.contains("配置不足时返回显式 <code>Unsupported</code>"),
        "expected CLI_COMMAND_MANUAL.html to document notify check as fail-closed"
    );
    assert!(
        cli_manual.contains("不会在 stdout 打印占位配置提示后成功退出"),
        "expected CLI_COMMAND_MANUAL.html to reject stdout-only missing-config prompts"
    );
    assert!(
        cli_manual.contains("<code>--channel all</code> 保留按 <code>NotificationConfig::from_env()</code> 聚合发送的行为"),
        "expected CLI_COMMAND_MANUAL.html to preserve notify test --channel all aggregate behavior"
    );
    assert!(
        cli_manual.contains("指定单一渠道时会先解析目标渠道并检查必需环境变量"),
        "expected CLI_COMMAND_MANUAL.html to document notify test single-channel config validation"
    );
    assert!(
        cli_manual.contains(
            "指定 <code>--channel</code> 时会在任何发送进度 stdout 之前验证渠道名和必需环境变量"
        ),
        "expected CLI_COMMAND_MANUAL.html to document notify send pre-output channel validation"
    );
    assert!(
        cli_manual.contains("不会在 stdout 打印测试成功占位内容后成功退出"),
        "expected CLI_COMMAND_MANUAL.html to reject notify test placeholder success output"
    );
    assert!(
        !cli_manual.contains("配置不足时直接打印所需变量名"),
        "CLI_COMMAND_MANUAL.html still describes notify check missing config as a successful stdout prompt"
    );
    assert!(
        !cli_manual.contains(
            "当前 <code>--channel</code> 不会像 <code>notify send</code> 那样真正缩窄发送渠道"
        ),
        "CLI_COMMAND_MANUAL.html still describes notify test channel as display-only"
    );
}

#[test]
fn generated_cli_manual_documents_import_from_image_vision_provider_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("按 <code>--model deepseek|openai</code> 选择对应 Vision provider"),
        "expected CLI_COMMAND_MANUAL.html to document import from-image model provider selection"
    );
    assert!(
        cli_manual.contains("Vision provider 尚未配置"),
        "expected CLI_COMMAND_MANUAL.html to include the missing Vision provider boundary"
    );
    assert!(
        cli_manual.contains("image format 不支持"),
        "expected CLI_COMMAND_MANUAL.html to include the unsupported image format boundary"
    );
    assert!(
        cli_manual.contains("Vision provider 不支持"),
        "expected CLI_COMMAND_MANUAL.html to include the unsupported Vision provider boundary"
    );
    assert!(
        cli_manual.contains("png, jpg, jpeg, gif, webp"),
        "expected CLI_COMMAND_MANUAL.html to document supported image formats"
    );
    assert!(
        cli_manual.contains("支持列表 <code>deepseek, openai</code>"),
        "expected CLI_COMMAND_MANUAL.html to document supported Vision providers"
    );
    assert!(
        cli_manual.contains("Vision provider 配置校验或请求前返回显式 <code>Unsupported</code>"),
        "expected CLI_COMMAND_MANUAL.html to document image format validation before Vision provider use"
    );
    assert!(
        cli_manual.contains("provider 配置校验或请求前返回显式 <code>Unsupported</code>"),
        "expected CLI_COMMAND_MANUAL.html to document unsupported Vision provider validation before provider use"
    );
    assert!(
        cli_manual.contains("<code>OPENAI_BASE_URL</code> 和 <code>OPENAI_VISION_MODEL</code>"),
        "expected CLI_COMMAND_MANUAL.html to document OpenAI Vision request configuration"
    );
    assert!(
        cli_manual.contains("不会在 stdout 打印图片识别占位错误后成功退出"),
        "expected CLI_COMMAND_MANUAL.html to reject stdout-only image recognition failures"
    );
    assert!(
        !cli_manual.contains("CLI 上的 <code>--model</code> 目前更接近参数占位"),
        "CLI_COMMAND_MANUAL.html still describes import from-image --model as a placeholder"
    );
    assert!(
        !cli_manual.contains("不支持的 Vision provider:"),
        "CLI_COMMAND_MANUAL.html still documents the old non-Unsupported Vision provider error text"
    );
}

#[test]
fn generated_cli_manual_documents_strategy_signal_list_filters_as_wired() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    assert!(
        cli_manual.contains("<code>--strategy-instance</code>、<code>--strategy</code>、<code>--code</code>、<code>--approval-status</code>、<code>--signal-status</code> 都会参与结果过滤"),
        "expected CLI_COMMAND_MANUAL.html to document strategy signal list filters as wired"
    );
    assert!(
        cli_manual.contains("<code>--limit</code> 会在过滤后限制输出条数"),
        "expected CLI_COMMAND_MANUAL.html to document strategy signal list limit as post-filter"
    );
    assert!(
        !cli_manual.contains("还没有传入 handler"),
        "CLI_COMMAND_MANUAL.html still says strategy signal list filters are parsed but not passed to the handler"
    );
}

#[test]
fn generated_cli_manual_documents_ai_runtime_context_warnings() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "运行时会明确打印“当前使用模拟价格/指标上下文”",
        "运行时会明确打印“当前使用模拟技术面分析上下文”",
        "运行时会明确打印“当前 AI 问答参数边界”",
        "运行时会明确打印“当前使用固定提示词”",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document AI runtime warning: {expected}"
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
        "quantix strategy request execute --request-id <ID>",
        "quantix strategy request cancel --request-id <ID> [--reason <TEXT>]",
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
        "request completed 但订单仍非终态时会额外输出 `semantics=request_completed_order_non_terminal`",
        "strategy request show` 会同时展示 `request_status`、`order_status`、`executed_at`、`failed_at`、`canceled_at` 等诊断字段",
        "mock_live request 即使返回 `accepted` 也会被标记为 `completed`",
        "不会自动交易，不会修改 paper 账户",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_phase29c_execution_automation_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### execution - 执行自动化",
        "quantix execution config init",
        "quantix execution config show",
        "quantix execution daemon run",
        "quantix execution daemon run --once",
        "QUANTIX_EXECUTION_CONFIG_PATH",
        "~/.quantix/execution/config.json",
        "`execution_request` 当前新增 `in_progress`",
        "`execution daemon` 只会 claim/consume `pending execution_request`",
        "`strategy request execute` 与 `execution daemon` 复用同一条 request 消费路径",
        "`manual|always` 是当前 auto-approval 的全部策略面",
        "`always` 下 `strategy daemon` 生成 signal 后会直接创建 `pending execution_request`",
        "`mock_live` request 即使返回 `accepted` 也会被标记为 `completed`",
        "request completed 但订单仍非终态时会额外输出 `semantics=request_completed_order_non_terminal`",
        "request 详情会展示 `request_status`、`order_status`、`executed_at`、`failed_at`、`canceled_at` 等诊断字段；若存在结构化 `execution_diagnostics`，还会单独展示 `Execution Diagnostics` section",
        "`quantix execution daemon run --once` 与 `strategy request list` 会在紧凑输出里带上 `executed_at`、`failed_at`、`canceled_at` 等诊断字段（若存在）",
        "非 completion 类结构化诊断会在紧凑输出里追加 `diag=<code>`，例如 `bridge_qmt_mode_not_live`",
        "`request_completed_order_terminal` / `request_completed_order_non_terminal` 不会重复显示为 `diag=<code>`；非终态完成仍沿用 `semantics=request_completed_order_non_terminal`",
        "reconciliation 会收敛 delayed fill、partial fill 与 `unknown` 恢复语义",
        "`live` adapter 仍未实现",
        "当前真实提交只走受 `qmt.mode=live` 保护的 `qmt_live` 路径",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}

#[test]
fn user_manual_documents_foundation_p0_task_boundary() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### task - 任务调度",
        "实验性 Foundation P0 任务入口",
        "`add` - 添加定时任务（当前未开放）",
        "`list` - 列出预置任务模板",
        "`start` - 以前台模式启动预置任务模板",
        "`task add` 当前只是保留入口",
        "Foundation P0 仅支持预置任务模板；请使用 `quantix task list` 查看可运行任务",
        "📋 Foundation P0 预置任务模板:",
        "💡 Foundation P0 只支持前台启动预置模板: `quantix task start`",
        "`--daemon` | 后台运行（Foundation P0 不支持）",
        "🛑 停止任务调度器...",
        "💡 提示: 在运行中的调度器按 Ctrl+C 停止",
        "📊 Foundation P0 任务状态:",
        "状态: 仅支持当前进程内调度器",
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
