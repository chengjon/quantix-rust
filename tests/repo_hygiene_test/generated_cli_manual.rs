use super::*;

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
fn generated_cli_manual_documents_screener_sort_by_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "quantix analyze screener run",
        "未知排序字段会在读取 ClickHouse 日线数据或输出筛选表格前返回显式 <code>Unsupported</code>",
        "错误包含 <code>不支持的 sort_by</code>",
        "支持列表 <code>code, score</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document screener sort_by fail-closed boundary: {expected}"
        );
    }

    assert!(
        !cli_manual.contains("未知排序字段会静默回退"),
        "CLI_COMMAND_MANUAL.html still describes sort_by as silently falling back"
    );
}

#[test]
fn generated_cli_manual_documents_account_type_fail_closed() {
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for expected in [
        "quantix account register",
        "无效 <code>--account-type</code> 会在写入账户注册表前返回显式 <code>Unsupported</code>",
        "错误包含 <code>无效的账户类型</code>",
        "支持列表 <code>paper, mock_live, qmt_live</code>",
        "兼容 <code>live</code> 别名",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document account type fail-closed boundary: {expected}"
        );
    }

    assert!(
        !cli_manual.contains("无效账户类型会静默回退"),
        "CLI_COMMAND_MANUAL.html still describes account type as silently falling back"
    );
}

#[test]
fn docs_document_account_split_target_type_fail_closed() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let user_manual = fs::read_to_string(repo_root().join("docs").join("USER_MANUAL.md"))
        .expect("expected USER_MANUAL.md");
    let changelog =
        fs::read_to_string(repo_root().join("CHANGELOG.md")).expect("expected CHANGELOG.md");
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md");
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for (label, contents) in [
        ("README.md", readme.as_str()),
        ("docs/USER_MANUAL.md", user_manual.as_str()),
        ("CHANGELOG.md", changelog.as_str()),
        ("FUNCTION_TREE.md", function_tree.as_str()),
    ] {
        for expected in [
            "account split --target-type",
            "Unsupported",
            "无效的目标类型",
            "single, group",
        ] {
            assert!(
                contents.contains(expected),
                "expected {label} to document account split target-type fail-closed boundary: {expected}"
            );
        }
    }

    for expected in [
        "quantix account split",
        "<code>--target-type</code> 仅支持 <code>single</code> 或 <code>group</code>",
        "未知目标类型会在输出 <code>订单拆分预览</code> 前返回显式 <code>Unsupported</code>",
        "错误包含 <code>无效的目标类型</code>",
        "支持列表 <code>single, group</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document account split target-type fail-closed boundary: {expected}"
        );
    }

    assert!(
        !cli_manual.contains("未知目标类型会静默回退"),
        "CLI_COMMAND_MANUAL.html still describes target type as silently falling back"
    );
}

#[test]
fn generated_cli_manual_documents_algo_create_type_fail_closed_boundary() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let changelog =
        fs::read_to_string(repo_root().join("CHANGELOG.md")).expect("expected CHANGELOG.md");
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md");
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for (label, contents) in [
        ("README.md", readme.as_str()),
        ("CHANGELOG.md", changelog.as_str()),
        ("FUNCTION_TREE.md", function_tree.as_str()),
    ] {
        for expected in [
            "algo create --algo-type",
            "Unsupported",
            "不支持的算法类型",
            "twap, vwap",
        ] {
            assert!(
                contents.contains(expected),
                "expected {label} to document algo create algo-type fail-closed boundary: {expected}"
            );
        }
    }

    for expected in [
        "quantix algo create",
        "<code>--algo-type</code> 仅支持 <code>twap</code> 或 <code>vwap</code>",
        "未知算法类型会在初始化算法上下文前返回显式 <code>Unsupported</code>",
        "错误包含 <code>不支持的算法类型</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document algo create algo-type fail-closed boundary: {expected}"
        );
    }

    assert!(
        cli_manual.contains("quantix algo start --algo-id &lt;ALGO_ID&gt;"),
        "expected algo start docs to remain keyed by algo-id"
    );
    assert!(
        !cli_manual.contains("quantix algo start --algo-id &lt;ALGO_ID&gt; --algo-type"),
        "algo start docs should not imply --algo-type is accepted"
    );
}

#[test]
fn generated_cli_manual_documents_algo_plan_type_fail_closed_boundary() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let changelog =
        fs::read_to_string(repo_root().join("CHANGELOG.md")).expect("expected CHANGELOG.md");
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md");
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for (label, contents) in [
        ("README.md", readme.as_str()),
        ("CHANGELOG.md", changelog.as_str()),
        ("FUNCTION_TREE.md", function_tree.as_str()),
    ] {
        for expected in [
            "algo plan --algo-type",
            "Unsupported",
            "不支持的算法类型",
            "twap, vwap",
        ] {
            assert!(
                contents.contains(expected),
                "expected {label} to document algo plan algo-type fail-closed boundary: {expected}"
            );
        }
    }

    for expected in [
        "quantix algo plan",
        "<code>--algo-type</code> 仅支持 <code>twap</code> 或 <code>vwap</code>",
        "未知算法类型会在输出切片预览前返回显式 <code>Unsupported</code>",
        "错误包含 <code>不支持的算法类型</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document algo plan algo-type fail-closed boundary: {expected}"
        );
    }

    assert!(
        cli_manual.contains("quantix algo start --algo-id &lt;ALGO_ID&gt;"),
        "expected algo start docs to remain keyed by algo-id"
    );
    assert!(
        !cli_manual.contains("quantix algo start --algo-id &lt;ALGO_ID&gt; --algo-type"),
        "algo start docs should not imply --algo-type is accepted"
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
fn generated_cli_manual_documents_notify_send_level_fail_closed() {
    let readme = fs::read_to_string(repo_root().join("README.md")).expect("expected README.md");
    let changelog =
        fs::read_to_string(repo_root().join("CHANGELOG.md")).expect("expected CHANGELOG.md");
    let function_tree = fs::read_to_string(repo_root().join("FUNCTION_TREE.md"))
        .expect("expected FUNCTION_TREE.md");
    let cli_manual = fs::read_to_string(repo_root().join("docs").join("CLI_COMMAND_MANUAL.html"))
        .expect("expected CLI_COMMAND_MANUAL.html to exist");

    for (label, contents) in [
        ("README.md", readme.as_str()),
        ("CHANGELOG.md", changelog.as_str()),
        ("FUNCTION_TREE.md", function_tree.as_str()),
    ] {
        for expected in [
            "notify send --level",
            "Unsupported",
            "无效的通知级别",
            "info, warning, error, critical",
        ] {
            assert!(
                contents.contains(expected),
                "expected {label} to document notify send level fail-closed boundary: {expected}"
            );
        }
    }

    for expected in [
        "quantix notify send",
        "<code>--level</code> 仅支持 <code>info</code>, <code>warning</code>, <code>error</code>, <code>critical</code>",
        "未知级别会在任何发送进度 stdout 之前返回显式 <code>Unsupported</code>",
        "错误包含 <code>无效的通知级别</code>",
    ] {
        assert!(
            cli_manual.contains(expected),
            "expected CLI_COMMAND_MANUAL.html to document notify send level fail-closed boundary: {expected}"
        );
    }
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
