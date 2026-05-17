# Pattern Hotspots

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

The scan covered 411 files under `src`, `tests`, `benches`, `examples`, `scripts`, and `build.rs`.

## Highest Match Files

| Rank | File | Bucket | Pattern hits |
|---:|---|---|---:|
| 1 | `src/cli/handlers/fundamental.rs` | production | 166 |
| 2 | `tests/execution_runtime_store_test.rs` | test | 155 |
| 3 | `tests/execution_kernel_test.rs` | test | 154 |
| 4 | `src/cli/handlers/tests/strategy_execution.rs` | test | 121 |
| 5 | `src/cli/handlers/app_shell.rs` | production | 118 |
| 6 | `tests/factor_pipeline_test.rs` | test | 113 |
| 7 | `tests/risk_service_test.rs` | test | 95 |
| 8 | `tests/execution_daemon_test.rs` | test | 87 |
| 9 | `src/cli/tests/risk.rs` | test | 84 |
| 10 | `tests/strategy_daemon_test.rs` | test | 82 |
| 11 | `src/cli/handlers/ai.rs` | production | 78 |
| 12 | `src/core/runtime.rs` | production | 74 |
| 13 | `src/cli/handlers/market_output.rs` | production | 70 |

## Manual Classification Notes

- `unsafe {` initially appeared in production files because `src/sync/etl.rs` contains test code in a production path. A line-level review found 74 matches inside `#[cfg(test)]` modules and 54 matches in test files; no production runtime unsafe block was found.
- `TODO[^-]` includes `src/tui/app.rs:8`, supporting carried-forward `AUDIT-S3-009`.
- High-volume `println!` matches are mostly CLI output paths and were not converted directly into findings.
