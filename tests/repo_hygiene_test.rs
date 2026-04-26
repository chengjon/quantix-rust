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
        "reconciliation 会收敛 delayed fill、partial fill 与 `unknown` 恢复语义",
        "`live` adapter 仍未实现",
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
    let function_map = fs::read_to_string(repo_root().join("docs").join("FUNCTION_MAP.md"))
        .expect("expected FUNCTION_MAP.md to exist");
    let migration = fs::read_to_string(
        repo_root()
            .join("docs")
            .join("MIGRATION_FROM_DAILY_STOCK_ANALYSIS.md"),
    )
    .expect("expected MIGRATION_FROM_DAILY_STOCK_ANALYSIS.md to exist");

    for expected in [
        "`paper` + `mock_live` + guarded `qmt_live` | 当前已实现的执行目标",
        "Bridge v1 已支持受能力门控的 `qmt_live` 真实提交通道",
    ] {
        assert!(
            function_map.contains(expected),
            "expected FUNCTION_MAP.md to contain {expected}"
        );
    }

    assert!(
        migration.contains("✅ paper/mock_live + guarded qmt_live"),
        "expected migration summary to include guarded qmt_live"
    );
}

#[test]
fn current_state_summary_docs_lock_mock_and_qmt_live_boundary() {
    let gap_analysis = fs::read_to_string(repo_root().join("docs").join("GAP_ANALYSIS.md"))
        .expect("expected GAP_ANALYSIS.md to exist");
    let roadmap = fs::read_to_string(repo_root().join("docs").join("DEVELOPMENT_ROADMAP.md"))
        .expect("expected DEVELOPMENT_ROADMAP.md to exist");
    let qmt_guide =
        fs::read_to_string(repo_root().join("docs").join("QMT_LIVE_TRADING_SETUP.md"))
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
        "QMT Bridge v1 (`qmt-preview` + guarded `qmt_live`)",
        "guarded `qmt_live` 已落地",
    ] {
        assert!(
            roadmap.contains(expected),
            "expected DEVELOPMENT_ROADMAP.md to contain {expected}"
        );
    }

    for expected in [
        "受能力门控的 `qmt_live` 真实提交流程",
        "通用 `target_mode=live` 仍未实现，不应与 `qmt_live` 混写",
        "`mock_live` 是模拟执行路径，不属于本指南覆盖范围",
        "docs/standards/MOCK_USAGE_POLICY.md",
    ] {
        assert!(
            qmt_guide.contains(expected),
            "expected QMT_LIVE_TRADING_SETUP.md to contain {expected}"
        );
    }
}

#[test]
fn architecture_and_roadmap_review_docs_keep_current_qmt_live_conclusion() {
    let architecture = fs::read_to_string(
        repo_root()
            .join("docs")
            .join("architecture")
            .join("WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md"),
    )
    .expect("expected WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md to exist");
    let roadmap_review =
        fs::read_to_string(repo_root().join("docs").join("ROADMAP_REVIEW.md"))
            .expect("expected ROADMAP_REVIEW.md to exist");

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
        roadmap_review.contains(
            "guarded `qmt_live` 已落地，剩余主要是 callback/query/cancel 与实盘工作流 hardening"
        ),
        "expected roadmap review to keep the current qmt_live hardening conclusion"
    );
}

#[test]
fn generated_cli_manual_keeps_qmt_and_mock_boundary() {
    let cli_manual =
        fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
            .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "QMT 执行兼容入口",
        "quantix execution qmt preview",
        "quantix execution qmt live",
        "提交真实订单 (需要确认)",
        "不直接提交订单",
        "使用模拟数据（测试用）",
        "模拟数据股票数量（仅与 --mock 一起使用）",
        "账户类型: paper | mock_live | qmt_live（兼容 live 别名）",
        "目标执行模式: paper | mock_live | qmt_live（live 将被拒绝并提示改走 qmt_live）",
        "目标模式过滤: paper | mock_live | qmt_live | live（legacy rejected mode）",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to contain {expected}"
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
        "request 详情会展示 `request_status`、`order_status`、`executed_at`、`failed_at`、`canceled_at` 等诊断字段",
        "`quantix execution daemon run --once` 会在紧凑输出里带上 `executed_at`、`failed_at`、`canceled_at` 等诊断字段（若存在）",
        "reconciliation 会收敛 delayed fill、partial fill 与 `unknown` 恢复语义",
        "`live` adapter 仍未实现",
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
