# Algo Plan Algo-Type Fail-Closed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `quantix algo plan --algo-type <unknown>` fail closed with `QuantixError::Unsupported` before emitting any plan preview.

**Architecture:** Follow the existing `algo create --algo-type` hardening pattern. Add one CLI regression test, change only the unknown algo type branch in `run_algo_plan`, then update docs and repo hygiene guards so the boundary stays documented.

**Tech Stack:** Rust, Cargo integration tests, generated CLI HTML manual, repo hygiene tests, GitNexus impact/detect_changes.

---

### Task 1: Algo Plan Runtime Validation

**Files:**
- Modify: `tests/algo_cli_validation_test.rs`
- Modify: `src/cli/handlers/algo.rs`

- [ ] **Step 1: Write the failing test**

Add this test after `algo_plan_rejects_unknown_output_format_before_emitting_preview` in `tests/algo_cli_validation_test.rs`:

```rust
#[test]
fn algo_plan_rejects_unsupported_algo_type_before_emitting_preview() {
    let (stdout, stderr, success) = run_quantix(&[
        "algo",
        "plan",
        "--code",
        "600519.SH",
        "--side",
        "buy",
        "--quantity",
        "1000",
        "--algo-type",
        "iceberg",
        "--duration",
        "10",
        "--slices",
        "2",
        "--output",
        "json",
    ]);

    assert!(
        !success,
        "expected algo plan to fail for unsupported algo type, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no slice preview on unsupported algo type, stdout={stdout}"
    );
    assert!(
        stderr.contains("不支持的算法类型: iceberg"),
        "expected algo type validation guidance in stderr, stderr={stderr}"
    );
    assert!(
        stderr.contains("Unsupported"),
        "expected Unsupported error kind for unsupported algo type, stderr={stderr}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --test algo_cli_validation_test algo_plan_rejects_unsupported_algo_type_before_emitting_preview -- --exact
```

Expected: FAIL because stderr contains `Other("不支持的算法类型: iceberg")` instead of `Unsupported`.

- [ ] **Step 3: Run GitNexus impact before editing the function**

Run GitNexus impact on `run_algo_plan` upstream with `cwd` set to the worktree. Expected risk should be low enough to continue without extra user approval; if HIGH or CRITICAL, stop and report.

- [ ] **Step 4: Write minimal implementation**

In `src/cli/handlers/algo.rs`, change only the `_` branch in `run_algo_plan` algo type parsing from:

```rust
_ => return Err(QuantixError::Other(format!("不支持的算法类型: {}", algo_type))),
```

to:

```rust
_ => {
    return Err(QuantixError::Unsupported(format!(
        "不支持的算法类型: {}",
        algo_type
    )));
}
```

- [ ] **Step 5: Run test to verify it passes**

Run:

```bash
cargo test --test algo_cli_validation_test algo_plan_rejects_unsupported_algo_type_before_emitting_preview -- --exact
```

Expected: PASS, 1 passed.

### Task 2: Documentation And Hygiene Guard

**Files:**
- Modify: `README.md`
- Modify: `CHANGELOG.md`
- Modify: `FUNCTION_TREE.md`
- Modify: `docs/CLI_COMMAND_MANUAL.html`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Update docs**

Add `algo plan --algo-type` to the existing algo fail-closed statements:

```text
algo create/plan --algo-type
```

Document that only `twap, vwap` are supported and unknown algo types fail closed with explicit `Unsupported` before preview output.

- [ ] **Step 2: Add repo hygiene guard**

Add a focused test next to `generated_cli_manual_documents_algo_create_type_fail_closed_boundary` that asserts README, CHANGELOG, FUNCTION_TREE, and CLI manual document the `algo plan --algo-type` boundary and do not imply `algo start` accepts `--algo-type`.

- [ ] **Step 3: Run focused hygiene guard**

Run:

```bash
cargo test --test repo_hygiene_test generated_cli_manual_documents_algo_plan_type_fail_closed_boundary -- --exact
```

Expected: PASS, 1 passed.

### Task 3: Closure Gates

**Files:**
- All modified files above.

- [ ] **Step 1: Format**

Run:

```bash
cargo fmt --check
```

Expected: exit 0.

- [ ] **Step 2: Run relevant tests**

Run:

```bash
cargo test --test algo_cli_validation_test
cargo test --test repo_hygiene_test
```

Expected: all tests pass.

- [ ] **Step 3: Run GitNexus detect_changes**

Run GitNexus `detect_changes` with `scope: all` before commit and `scope: compare, base_ref: master` after commit. Expected risk should be low, with no unexpected execution flows.

- [ ] **Step 4: Commit and push**

Run:

```bash
git status --short --branch
git add README.md CHANGELOG.md FUNCTION_TREE.md docs/CLI_COMMAND_MANUAL.html src/cli/handlers/algo.rs tests/algo_cli_validation_test.rs tests/repo_hygiene_test.rs docs/superpowers/plans/2026-06-04-algo-plan-algo-type-fail-closed.md
git commit -m "fix: fail closed algo plan type validation"
git push -u origin fix/algo-plan-algo-type-validation
```

Expected: branch pushed with one commit.

### Self-Review

- Spec coverage: runtime behavior, docs, hygiene, formatting, tests, GitNexus gates, commit/push are covered.
- Placeholder scan: no TBD/TODO/fill-in placeholders.
- Type consistency: test/function/error names match existing Rust symbols and recent fail-closed naming.
