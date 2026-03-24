# Phase 27D Risk Industry Blocklist Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an `industry-blocklist` risk rule that rejects new buys when the target symbol belongs to a blocked industry, using latest `sector_daily` industry data first and monthly static code-to-industry snapshots as fallback.

**Architecture:** Keep `RiskService::check_buy()` as the single buy-check orchestration entry point, and add a focused risk-side industry resolver plus a SQLite-backed monthly snapshot store. Resolve industry membership in a fixed order: latest primary source, then query-month snapshot, then nearest historical snapshot, and fail closed only when all three tiers miss.

**Tech Stack:** Rust, async_trait, chrono, serde, sqlx/sqlite, existing ClickHouse client, existing risk service patterns, existing market `sector_daily` data path, GitNexus impact analysis, Graphiti MCP workflow, cargo test, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-24-phase27d-risk-industry-blocklist-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-24-phase27d-risk-industry-blocklist-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Graphiti is mandatory for design/review/debug/handoff memory. If implementation-time ingest retries or fails, leave an equivalent local note and write `Graphiti backfill required`.
- The repository may contain unrelated local state files. Stage only files from this task and never revert unrelated user changes.
- The current repository already has a risk-side volatility module. Reuse that style for dependency injection and helper boundaries instead of embedding I/O inside `RiskService`.
- Before implementing the snapshot store, verify the parent directory derived from `risk_state.json` exists or can be created, and is writable for a sibling DB such as `industry_snapshots.db`.

## File Map

- `src/risk/models.rs`
  - Add `RiskRuleType::IndustryBlocklist` and a structured `RuleValue::TextList(Vec<String>)`.
- `src/risk/mod.rs`
  - Re-export new industry resolver and snapshot-store surfaces.
- `src/risk/industry.rs`
  - New industry resolution helper, `IndustrySourceTier`, primary-source lookup boundary, and blocklist evaluation logic.
- `src/risk/industry_store.rs`
  - New SQLite store for month snapshots keyed by `(snapshot_month, code)`.
- `src/risk/service.rs`
  - Integrate `industry-blocklist` into `check_buy()` after `volatility-limit`.
- `src/cli/tests/risk.rs`
  - Add parser/dispatch coverage for `industry-blocklist`.
- `src/cli/handlers.rs`
  - Add focused trade/strategy caller tests for industry-blocklist rejection reasons.
- `tests/risk_service_test.rs`
  - Extend rule parsing tests for `TextList`.
- `tests/risk_industry_test.rs`
  - New focused resolver/store tests for latest/monthly/fallback precedence and month freeze semantics.
- `README.md`
  - Document `industry-blocklist` and month-snapshot fallback semantics.
- `docs/USER_MANUAL.md`
  - Document CLI examples, exact-match semantics, and three-tier resolution order.
- `tests/repo_hygiene_test.rs`
  - Lock the new docs wording and examples.

## Implementation Assumptions To Preserve

The following design constraints must remain true:

1. `industry-blocklist` is a stock-level pre-buy gate, not an account-level sector limit
2. resolution order is fixed: latest primary source -> query-month snapshot -> latest available snapshot
3. month snapshots freeze on the first successful latest-source resolution in that month
4. existing month snapshots are not overwritten in v1
5. industry-name matching is exact string matching in v1
6. if all three tiers miss, the rule remains fail-closed and rejects the buy

## Chunk 1: Rule Type, Structured Value, And CLI Surface

### Task 1: Add `industry-blocklist` to the risk rule model and parser

**Files:**
- Modify: `src/risk/models.rs`
- Modify: `tests/risk_service_test.rs`
- Modify: `src/cli/tests/risk.rs`

- [ ] **Step 1: Run GitNexus impact analysis for rule-model symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "RiskRuleType", direction: "upstream", includeTests: true})
gitnexus_impact({repo: "quantix-rust", target: "RuleValue", direction: "upstream", includeTests: true})
```

Expected:
- low risk focused on risk parsing, display, and tests.

- [ ] **Step 2: Write failing parsing and persistence tests**

Add tests for:
- `set_rule("industry-blocklist", "银行,地产")` succeeds
- value parses into a structured list preserving order
- whitespace-only segments are ignored, for example `银行, ,地产`
- trailing comma is ignored, for example `银行,`
- leading comma is ignored, for example `,地产`
- repeated commas collapse empty segments, for example `银行,,地产`
- CLI parser accepts `quantix risk rule set --type industry-blocklist --value 银行,地产`
- `run risk rule set --type industry-blocklist --value 银行,地产` persists the structured rule to the JSON risk store

Suggested assertions:

```rust
assert_eq!(rule.rule_type, RiskRuleType::IndustryBlocklist);
assert_eq!(rule.value, RuleValue::TextList(vec!["银行".to_string(), "地产".to_string()]));
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_service_test --lib cli::tests::risk:: -- --nocapture
```

Expected:
- FAIL because `industry-blocklist` and `TextList` do not exist yet.

- [ ] **Step 4: Implement the new rule type and structured value parser**

Implement:
- `RiskRuleType::IndustryBlocklist`
- CLI string `industry-blocklist`
- `RuleValue::TextList(Vec<String>)`
- parsing branch that:
  - splits on commas
  - trims entries
  - drops empties
  - errors if no usable industry names remain
- `display()` support that joins values with commas for CLI output

Keep behavior aligned with the spec:
- no fuzzy matching
- no alias normalization
- no new command surface

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_service_test --lib cli::tests::risk:: -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- staged scope remains on the rule model and CLI-facing rule tests; the repo may still show unrelated dirty files outside the staged set.

Commit:
```bash
git add src/risk/models.rs tests/risk_service_test.rs src/cli/tests/risk.rs
git commit -m "feat: add industry-blocklist rule type"
```

## Chunk 2: Industry Snapshot Store And Three-Tier Resolver

### Task 2: Add the SQLite month-snapshot store and risk-side industry resolver

**Files:**
- Create: `src/risk/industry_store.rs`
- Create: `src/risk/industry.rs`
- Modify: `src/risk/mod.rs`
- Create: `tests/risk_industry_test.rs`

- [ ] **Step 1: Write failing resolver and snapshot tests**

Add focused coverage for:
- latest-source lookup returns an industry and freezes the query-month snapshot
- a second latest-source success in the same month does not overwrite the existing snapshot row
- primary lookup failure falls back to the query-month snapshot
- query-month miss falls back to the most recent available snapshot
- all three tiers missing return a hard resolution error

Use temporary SQLite storage for the snapshot layer and fake latest-source readers so tests do not depend on live ClickHouse.

Suggested assertions:

```rust
assert_eq!(resolved.industry_name, "银行");
assert_eq!(resolved.source_tier, IndustrySourceTier::SnapshotMonth);
assert!(err.to_string().contains("latest/monthly/fallback"));
```

- [ ] **Step 2: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_industry_test -- --nocapture
```

Expected:
- FAIL because the resolver and snapshot store do not exist yet.

- [ ] **Step 3: Implement the snapshot store**

Implement in `src/risk/industry_store.rs`:
- SQLite schema bootstrap for `risk_industry_snapshots`
- row shape with:
  - `snapshot_month`
  - `code`
  - `industry_name`
  - `source`
  - `captured_at`
- unique key on `(snapshot_month, code)`
- lookup helpers for:
  - query-month row
  - latest available row
- insert-if-missing helper for month freeze

Derive the default DB path from the existing risk path sibling directory, for example by using `risk_state.json`’s directory plus a dedicated `industry_snapshots.db`, rather than expanding `CliRuntime` again in this slice.
Verify in implementation that missing parent directories are created before opening the SQLite store.

- [ ] **Step 4: Implement the resolver and precedence logic**

Implement in `src/risk/industry.rs`:
- a latest-source lookup boundary (for example `LatestIndustryReader` or `IndustryResolver` internals)
- a thin risk-side adapter over the existing ClickHouse client for `code -> latest industry` lookup using `sector_daily` with `sector_type = industry`
- three-tier resolution:
  1. latest source
  2. query-month snapshot
  3. latest available snapshot
- month-freeze behavior on the first successful latest-source resolution of that month
- structured output that can tell the caller which tier was used

Avoid touching `src/market/service.rs` unless absolutely necessary. There is no current direct `code -> industry` query path to wrap at the service layer, so prefer adding a thin risk-side adapter over widening the market service trait for this slice.

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_industry_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/risk/industry_store.rs src/risk/industry.rs src/risk/mod.rs tests/risk_industry_test.rs
git commit -m "feat: add industry snapshot resolver"
```

## Chunk 3: `check_buy()` Integration And Caller Regression Coverage

### Task 3: Enforce `industry-blocklist` in risk buy checks and prove caller paths surface the same reason

**Files:**
- Modify: `src/risk/service.rs`
- Modify: `tests/risk_service_test.rs`
- Modify: `src/cli/handlers.rs`

- [ ] **Step 1: Run GitNexus impact analysis for the buy-check path**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "check_buy", direction: "upstream", includeTests: true, maxDepth: 3})
```

Expected:
- `CRITICAL`

Review before editing:
- d=1 callers are bounded to:
  - `src/cli/handlers.rs:evaluate`
  - `src/cli/handlers.rs:execute_trade_command_with_risk`
  - `src/execution/daemon.rs:evaluate`
  - direct risk-service tests

- [ ] **Step 2: Write failing buy-check and caller tests**

Add tests for:
- blocked industry rejects a direct `trade buy`
- unblocked industry allows a direct `trade buy`
- strategy paper path surfaces `industry-blocklist` rejection reason
- strategy mock_live path surfaces `industry-blocklist` rejection reason
- sell path remains unaffected even if the code’s industry is blocked

For `RiskService`-level tests, also assert:
- a full three-tier miss returns the expected `检查失败`
- rejection does not create a buy lock
- rejection does not append a new `risk log` event

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_service_test --test risk_industry_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_execute_trade_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_strategy_ -- --nocapture
```

Expected:
- FAIL because `check_buy()` does not yet enforce the industry rule.

- [ ] **Step 4: Integrate the rule into `RiskService::check_buy()`**

Implement:
- default `RiskService::new(...)` path wires both the existing volatility loader and the new default industry resolver
- add or extend injection-friendly constructors so tests can provide a fake bar loader and a fake industry resolver together
- `check_buy()` checks `industry-blocklist` after `volatility-limit`
- `industry-blocklist` uses exact string matching against the resolved industry
- all-tier miss returns a hard `检查失败`
- no durable `risk log` event is written for per-buy industry rejections

Preserve existing injection-friendly constructors or extend them minimally so tests can provide fake bar loaders and fake industry resolvers together without duplicating business logic in CLI code.

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_service_test --test risk_industry_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_execute_trade_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_strategy_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run runtime-path regressions**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test strategy_paper_run_test --test strategy_mock_live_run_test --test execution_daemon_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 7: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add src/risk/service.rs tests/risk_service_test.rs src/cli/handlers.rs
git commit -m "feat: enforce industry-blocklist in buy checks"
```

## Chunk 4: Docs And Hygiene

### Task 4: Document `industry-blocklist` and the month-snapshot fallback chain

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene expectations**

Add expectations that docs mention:
- `quantix risk rule set --type industry-blocklist --value 银行,地产`
- exact-match blocklist semantics
- the three-tier resolution order
- monthly snapshot freeze behavior
- that sell paths remain unaffected
- that industry whitelist and auto-deleverage remain deferred

- [ ] **Step 2: Run hygiene test to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- FAIL because docs do not yet mention the industry blocklist rule.

- [ ] **Step 3: Update README and USER_MANUAL**

Document:
- `industry-blocklist` as a supported risk rule
- exact string matching semantics
- latest -> query-month snapshot -> latest snapshot precedence
- month snapshot freezes on the first successful latest-source resolution of that month
- fail-closed behavior only after all three tiers miss
- sell paths remain unaffected
- industry whitelist / auto-deleverage still deferred

- [ ] **Step 4: Re-run hygiene test to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Run final focused verification for the whole slice**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --test risk_service_test --test risk_industry_test --test strategy_paper_run_test --test strategy_mock_live_run_test --test execution_daemon_test --test repo_hygiene_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib cli::tests::risk:: -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_execute_trade_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27d cargo test --lib test_strategy_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Commit:
```bash
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs docs/superpowers/specs/2026-03-24-phase27d-risk-industry-blocklist-design.md docs/superpowers/plans/2026-03-24-phase27d-risk-industry-blocklist-implementation.md
git commit -m "docs: document industry-blocklist risk rule"
```

## Verification Notes

- Use `CARGO_TARGET_DIR=/tmp/quantix-target-phase27d` consistently to avoid interference from existing local target-directory state.
- Because the repository may contain unrelated local-state files, always inspect the staged set before each commit:

```bash
git diff --staged --stat
git diff --staged
```

- If Graphiti write-back during implementation retries or fails, append a local summary to this plan file or the current implementation checkpoint and include:

```text
Graphiti backfill required
```
