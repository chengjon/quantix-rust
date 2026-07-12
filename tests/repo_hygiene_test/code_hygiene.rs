use super::*;

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
fn paper_adapter_remains_immediate_fill_only() {
    let path = repo_root().join("src/execution/paper.rs");
    let contents = fs::read_to_string(&path).expect("expected paper execution adapter to exist");
    let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);

    assert!(
        production.contains("IMMEDIATE_FILL_ONLY"),
        "paper adapter must keep an explicit immediate-fill marker"
    );
    for forbidden_status in ["OrderStatus::Submitted", "OrderStatus::PendingCancel"] {
        assert!(
            !production.contains(forbidden_status),
            "paper adapter must not introduce pending lifecycle status {forbidden_status}"
        );
    }
}

#[test]
fn concrete_execution_adapters_emit_standard_risk_notices_on_submit() {
    for (relative_path, expected_channel) in [
        ("src/execution/paper.rs", "PAPER_IMMEDIATE_CHANNEL"),
        ("src/execution/qmt_live_adapter.rs", "QMT_LIVE_CHANNEL"),
    ] {
        let path = repo_root().join(relative_path);
        let contents = fs::read_to_string(&path).expect("expected execution adapter to exist");
        let production = contents.split("#[cfg(test)]").next().unwrap_or(&contents);

        assert!(
            production.contains(expected_channel),
            "{relative_path} should use the standard execution-mode channel marker"
        );
        assert!(
            production.contains("log_execution_mode_risk_notice("),
            "{relative_path} should emit the standard execution-mode risk notice on submit"
        );
    }
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
