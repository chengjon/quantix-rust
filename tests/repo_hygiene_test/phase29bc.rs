use super::*;

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
