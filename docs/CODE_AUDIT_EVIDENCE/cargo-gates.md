# Cargo Gate Evidence

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

Captured during the 2026-05-15 audit run, with a 2026-05-18 follow-up release-build gate update.

## Gate Summary

| Gate | Exit status | Result | Evidence summary |
|---|---:|---|---|
| `cargo fmt --check` | 1 | FAIL | `src/factor/scoring.rs:1` needs rustfmt wrapping for the long `polars::prelude` import. |
| `cargo clippy --all-targets --all-features` | 0 | PASS with warnings | JSON diagnostic pass reported 220 warning diagnostics; highest-volume lints include `dead_code`, `unused_variables`, `clone_on_copy`, and `await_holding_lock`. |
| `cargo test --all-targets` | 101 | FAIL | `factor_score_cli_writes_csv_output` failed at `tests/factor_pipeline_test.rs:454`; expected CSV row `2026-01-02,000002.SZ,1.0,2` was missing. |
| `cargo build --release` | 0 | PASS | Follow-up gate evidence in `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log` confirms `cargo build --manifest-path /opt/claude/quantix-rust/Cargo.toml --release --quiet` exited 0 with existing warnings only. |

## Details

### `cargo fmt --check`

- Duration: 1093 ms.
- Failure excerpt starts at `src/factor/scoring.rs:1`.
- Finding: `AUDIT-S2-010`.

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

- Duration: 9609 ms.
- Exit status: 101.
- Finding: `AUDIT-S2-011`.

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

The release-build gate is closed by follow-up evidence. Full runtime release confidence still depends on committing the local formatting and factor CSV output fixes for `AUDIT-S2-010` and `AUDIT-S2-011`.
