# Phase27D Branch Closure - 2026-06-05

## Decision

Close the live remote `phase27d-*` branch stack as covered by the current `master`
implementation, rather than merging the old branch tips.

This is a capability-coverage closure, not a claim that the branch commits were
merged byte-for-byte. The old stack still has patch-unique commits versus
`master`, but direct replay conflicts with the newer industry-risk architecture
already present on `master`.

## Branches Closed

| Remote branch | Tip | Tip subject |
| --- | --- | --- |
| `origin/phase27d-resolver-base` | `1c625dd8b6fd` | `feat: add phase27d industry resolver` |
| `origin/phase27d-ruletype` | `bd8da4489038` | `feat: add industry-blocklist rule type` |
| `origin/phase27d-sqlite-resolver` | `09409fac46b5` | `feat: add sqlite shenwan industry resolver` |
| `origin/phase27d-blocklist-enforcement-core` | `64f52b377cc7` | `feat: enforce industry-blocklist in buy checks` |
| `origin/phase27d-trade-cli-wiring` | `7ec86d4a5ac1` | `feat: wire trade cli to runtime risk checks` |

The stack was cumulative. `origin/phase27d-trade-cli-wiring` contained the full
five-commit sequence:

```text
1c625dd8b6fd feat: add phase27d industry resolver
bd8da4489038 feat: add industry-blocklist rule type
09409fac46b5 feat: add sqlite shenwan industry resolver
64f52b377cc7 feat: enforce industry-blocklist in buy checks
7ec86d4a5ac1 feat: wire trade cli to runtime risk checks
```

## Why The Old Tips Were Not Merged

An isolated replay attempt from current `origin/master` was made in a temporary
worktree. The first commit, `1c625dd8b6fd`, conflicted immediately:

```text
AA src/risk/industry.rs
AA src/risk/industry_store.rs
UU src/risk/mod.rs
AA tests/risk_industry_test.rs
```

The conflict was architectural, not a small textual drift. Current `master`
already contains the newer industry-risk split:

- `src/risk/industry.rs`
- `src/risk/industry_resolver.rs`
- `src/risk/industry_store.rs`
- `src/risk/industry_sync.rs`
- `src/risk/service/industry_checks.rs`
- `src/cli/handlers/trade_handler.rs`

The old branch used names such as `ClickHouseLatestIndustryReader`,
`SqliteIndustrySnapshotStore`, and `execute_trade_command_with_runtime_risk`.
Current `master` uses the newer `SqliteIndustryStore`, `IndustryResolver`,
`industry_sync`, and `execute_trade_command_with_risk` path. Keeping both would
duplicate the domain model and increase risk.

## Coverage Matrix

| Old commit | Old capability | Current `master` coverage |
| --- | --- | --- |
| `1c625dd8b6fd` | Industry domain model and resolver base | Covered by `src/risk/industry.rs`, `src/risk/industry_resolver.rs`, `src/risk/industry_store.rs`, and `tests/risk_industry_test.rs`. Some old symbols were intentionally replaced by newer names. |
| `bd8da4489038` | `industry-blocklist` risk rule type and CLI parsing | Covered by `RiskRuleType::IndustryBlocklist`, CLI parsing in `src/cli/tests/risk.rs`, and rule storage tests in `tests/risk_service_test.rs`. |
| `09409fac46b5` | SQLite Shenwan current/history resolver and refresh APIs | Covered by `SqliteIndustryStore`, `industry_sync`, `upsert_shenwan_current_rows`, `upsert_shenwan_history_rows`, current/history lookup tests, and sync tests. |
| `64f52b377cc7` | Buy-side industry blocklist enforcement | Covered by `evaluate_industry_blocklist`, `RiskService::with_industry_resolver`, and risk service blocklist tests. |
| `7ec86d4a5ac1` | Trade CLI runtime risk wiring | Covered by `execute_trade_command_with_risk` and CLI handler tests for industry-limit buy rejection and sell success. |

## Verification

The following target gates were run on current `master` before closure:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test risk_industry_test --test risk_industry_sync_test --test risk_service_test
```

Result:

```text
risk_industry_sync_test: 2 passed, 0 failed
risk_industry_test: 10 passed, 0 failed
risk_service_test: 31 passed, 0 failed
```

Additional focused CLI gates:

```text
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::handlers::tests::trade::test_execute_trade_buy_rejects_when_industry_limit_exceeds_threshold
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::handlers::tests::trade::test_execute_trade_sell_succeeds_and_returns_trade_summary
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --lib cli::tests::risk::run_risk_rule_set_industry_blocklist_dispatches_to_handler
```

Result:

```text
test_execute_trade_buy_rejects_when_industry_limit_exceeds_threshold: 1 passed, 0 failed
test_execute_trade_sell_succeeds_and_returns_trade_summary: 1 passed, 0 failed
run_risk_rule_set_industry_blocklist_dispatches_to_handler: 1 passed, 0 failed
```

GitNexus scope check before this report:

```text
detect_changes(scope=all): changed_files=0, affected_count=0, risk_level=none, stale=false
```

## Archive Tags

The old tips are preserved under archive tags:

| Archive tag | Tip |
| --- | --- |
| `archive/phase27d-resolver-base-20260605` | `1c625dd8b6fd` |
| `archive/phase27d-ruletype-20260605` | `bd8da4489038` |
| `archive/phase27d-sqlite-resolver-20260605` | `09409fac46b5` |
| `archive/phase27d-blocklist-enforcement-core-20260605` | `64f52b377cc7` |
| `archive/phase27d-trade-cli-wiring-20260605` | `7ec86d4a5ac1` |

## Closure Boundary

This closure removes stale remote branch-board noise for `phase27d`.

It does not reopen implementation work in phase29a or phase29b. The remaining
remote branch board after this closure should be implementation-oriented around
those phase29 stacks only.
