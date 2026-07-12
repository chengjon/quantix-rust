use super::*;

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
