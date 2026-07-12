use super::*;

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
        designed_registry_rows >= 14,
        "expected FUNCTION_TREE.md to keep explicit designed/pending registry rows; found {designed_registry_rows}"
    );
}
