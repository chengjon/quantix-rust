# tdx-api Follow-up Value Analysis

Date: 2026-06-06
Scope: post-commit follow-up analysis after `e8a5bb2 chore: close tdx-api bridge audit cleanup`

## Current Baseline

The tdx-api bridge is already absorbed into quantix-rust as a REST integration, not as vendored tdx-api service source.

Evidence:

- `src/sources/tdx_api.rs` defines the quantix-rust REST client.
- `src/cli/handlers/tdx_api_handler.rs` wires the `quantix data tdx-api` command surface.
- `docker-compose.yml` defines `TDX_API_URL=http://tdx-api:8080` for quantix and a `tdx-api` service built from `/opt/claude/tdx-api`.
- `FUNCTION_TREE.md` records `tdx-api` as implemented for quotes, K lines, minute data, search, trading calendar, tick data, async tasks, and import commands.
- Latest verification in this session:
  - `cargo test -p quantix-cli tdx_api --quiet`: exit 0, 15 passed.
  - `cargo test -p quantix-cli --test bridge_tdx_source_test --quiet`: exit 0, 2 passed.
  - `cargo test -p quantix-cli --quiet`: exit 0, 1302 passed, 6 ignored.

## Value Criteria

A follow-up item is valuable enough for this OpenSpec change only if it meets all of these criteria:

- It directly reduces release risk for the tdx-api import path.
- It requires or verifies real external dependencies, not just unit-level command parsing.
- It produces repeatable evidence that can be rerun before release.
- It does not mix unrelated repository-wide hygiene or speculative architecture work into the import validation slice.

## Follow-up Disposition

| Item | Value | Decision | Rationale |
|------|-------|----------|-----------|
| E2E `import-klines --all` with ClickHouse + tdx-api | High | Include now | Validates the most important tdx-api to ClickHouse K-line import path under real dependencies. Must exercise the `--all` path; an approved `--exchange` bound may be used to keep the gate rerunnable. |
| E2E `import-ticks` with TDengine + tdx-api | High | Include now | Validates the tdx-api to TDengine tick-data path under real dependencies. This is the second P0 import path called out by the closeout report. |
| K-line data quality checks after import | High | Include now | The most dangerous import failure is silent corrupt data. Evidence should check non-empty rows, OHLCV sanity, source tagging, date ordering, and incremental behavior. |
| Release build re-verification | Medium | Include as closure gate | The previous cleanup session did not capture a post-cleanup `cargo build --release --quiet` exit code. This is not a tdx-api feature, but it is a release-readiness gate for this slice. |
| Clippy warning reduction | Medium | Defer to separate tech-debt change | The project still has warning backlog, but reducing counts is not the same as validating tdx-api imports. Keep it out of this E2E hardening change. |
| `println!` to `tracing` in library modules | Medium | Defer to separate logging hygiene change | This is repository-wide governance, not specific to ClickHouse or TDengine import correctness. |
| `import-klines --all` progress bar | Low now | Defer | Useful operator UX, but lower priority than proving imported data is correct. |
| `import-ticks` batch mode | Medium future | Defer until single-code/date E2E passes | Batch traversal should build on a proven single import path and a known data-quality contract. |
| K-line import failure notification | Medium future | Defer until import gates are stable | Alerting should wrap stable gates. Adding notification before the gate is reliable increases noise. |
| `daily-update.sh` systemd timer | Low now | Defer | Deployment packaging is useful after E2E acceptance is stable. |
| Unified `DataSource` trait for tdx-api / eastmoney / bridge_tdx | Potentially high, high risk | Defer to architecture change | This changes abstraction ownership and should not be driven by the E2E import slice. |
| tdx-api WebSocket K-line streaming | Speculative | Defer | Real-time streaming needs a separate product requirement and runtime model. |
| Factor pipeline over imported THS data | High future | Defer until data quality gate exists | Factor correctness depends on trustworthy imported K-line data. |
| Backtest data auto-preparation | High future | Defer until import and quality gates exist | Backtest prep should consume validated import outputs, not define the validation baseline. |

## OpenSpec Recommendation

Create a focused OpenSpec change:

```text
openspec/changes/tdx-api-import-e2e-hardening/
```

Goal:

```text
Move tdx-api ClickHouse K-line import and TDengine tick import from "code integrated and tests passing" to "acceptable, releasable, and rerunnable under real dependency environments."
```

Included capabilities:

- Dependency preflight for tdx-api, ClickHouse, and TDengine.
- ClickHouse K-line E2E through `quantix data tdx-api import-klines --all`.
- TDengine tick E2E through `quantix data tdx-api import-ticks`.
- Data-quality evidence for imported K-line and tick records.
- Release gate evidence, including package tests and a captured release-build exit code.

Non-goals:

- Do not vendor tdx-api service source into quantix-rust.
- Do not reduce broad clippy warning counts in this change.
- Do not refactor the data-source abstraction.
- Do not add WebSocket streaming, factor computation, backtest auto-preparation, notification, progress UI, or systemd packaging in this change.

## Acceptance Summary

This follow-up should be considered complete only when:

- Missing external dependencies produce a clear BLOCKED result, not a false pass.
- ClickHouse E2E evidence proves non-empty K-line import, sane data, source tagging, and incremental rerun behavior.
- TDengine E2E evidence proves non-empty tick import and sane timestamp/price/volume fields.
- Evidence artifacts identify the tdx-api URL, database targets, command lines, selected market/date/code scope, row counts, and validation query results.
- `cargo test -p quantix-cli --quiet` and `cargo build --release --quiet` both produce captured exit 0 evidence after the E2E hardening work.
