# Cargo Gate Evidence

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

Captured during the 2026-05-15 audit run, with 2026-05-18 follow-up runtime gate updates.

## Gate Summary

| Gate | Exit status | Result | Evidence summary |
|---|---:|---|---|
| `cargo fmt --check` | 0 | PASS | Follow-up gate run passes after formatting the local factor scoring work. |
| `cargo clippy --all-targets --all-features` | 0 | PASS with warnings | JSON diagnostic pass reported 220 warning diagnostics; highest-volume lints include `dead_code`, `unused_variables`, `clone_on_copy`, and `await_holding_lock`. |
| `cargo test --all-targets` | 0 | PASS | Follow-up all-target test run passes after preserving plain factor score symbol strings before CSV output. |
| `cargo build --release` | 0 | PASS | Follow-up gate evidence in `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log` confirms `cargo build --manifest-path /opt/claude/quantix-rust/Cargo.toml --release --quiet` exited 0 with existing warnings only. |

## Details

### `cargo fmt --check`

- Initial audit run: exit status 1; failure excerpt started at `src/factor/scoring.rs:1`.
- Follow-up gate run: `cargo fmt --check` exits 0.
- Finding: `AUDIT-S2-010`, closed by follow-up formatting gate evidence.

### `cargo clippy --all-targets --all-features`

- Exit status: 0.
- JSON diagnostic pass reported 220 warnings.

| Lint | Count | Representative location |
|---|---:|---|
| `dead_code` | 28 | `src/cli/handlers/risk.rs:471` |
| `unused_variables` | 24 | `src/sources/eastmoney.rs:39` |
| `clippy::clone_on_copy` | 20 | `src/cli/handlers/market_output.rs:303` |
| `clippy::await_holding_lock` | 20 | `tests/watchlist_handler_test.rs:38` |
| `unused_imports` | 17 | `src/db/tdengine.rs:1` |
| `clippy::collapsible_if` | 16 | `src/cli/handlers/factor.rs:8` |

### `cargo test --all-targets`

- Initial audit run: exit status 101.
- Root cause: factor score CSV output used display-form string extraction from Polars `AnyValue`, so symbols were written with embedded display quotes and then escaped by CSV output.
- Local fix: factor score extraction now prefers `AnyValue::get_str()` for plain string values and only falls back to `to_string()` for non-string values.
- Follow-up targeted test: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test factor_pipeline_test factor_score_cli_writes_csv_output` exits 0.
- Follow-up factor pipeline test: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test factor_pipeline_test` exits 0.
- Follow-up all-target test: `cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --all-targets` exits 0.
- Finding: `AUDIT-S2-011`, closed by follow-up factor CSV output fix and gate evidence.

```text
test factor_score_cli_writes_csv_output ... FAILED
thread 'factor_score_cli_writes_csv_output' panicked at tests/factor_pipeline_test.rs:454:5:
assertion failed: csv.contains("2026-01-02,000002.SZ,1.0,2\n")
error: test failed, to rerun pass `--test factor_pipeline_test`
```

### `cargo build --release`

- Initial audit run: started as part of the gate baseline, exceeded the MCP command window while still active, and was recorded as `AUDIT-S3-010` / `NEEDS-REPRO`.
- Follow-up gate run: `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log`.
- Confirmation command: `cargo build --manifest-path /opt/claude/quantix-rust/Cargo.toml --release --quiet`.
- Confirmation result: exit status 0 with existing warnings only.
- Process cleanup: no unmanaged cargo/rustc release-build process remained after the follow-up run.
- Finding: `AUDIT-S3-010`, closed by reproducible release-build pass evidence.

## Gate Conclusion

The runtime gate loop is closed locally: formatting, all-target tests, release build, and repository documentation hygiene pass. GitHub issue closure still depends on committing and synchronizing the local `AUDIT-S2-010` and `AUDIT-S2-011` fix evidence.
