# Phase 27C Risk Volatility Rule Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an ATR-based `volatility-limit` risk rule that blocks new buys when a symbol's recent daily volatility exceeds a configured percentage threshold, while leaving sell paths and account-lock semantics unchanged.

**Architecture:** Keep `RiskService::check_buy()` as the single orchestration entry point, but add a focused volatility evaluation helper and a thin risk-side daily-bar loader boundary. Reuse the existing ATR indicator and daily-bar loading capability, fail closed when bars are missing, and keep the rule as transient buy rejection rather than durable lock state.

**Tech Stack:** Rust, async_trait, chrono, rust_decimal, existing ATR indicator in `src/analysis/indicators.rs`, existing daily-bar loaders, GitNexus impact analysis, Graphiti MCP workflow, cargo test, repo hygiene tests.

---

## Preflight

- Read the approved spec in [2026-03-24-phase27c-risk-volatility-rule-design.md](/opt/claude/quantix-rust/docs/superpowers/specs/2026-03-24-phase27c-risk-volatility-rule-design.md).
- Use `@superpowers/test-driven-development` for every behavior change. No production edits before a failing test.
- Use `@superpowers/verification-before-completion` before claiming the phase is done.
- Graphiti is mandatory for design/review/debug/handoff memory. If implementation-time ingest fails or retries, leave an equivalent local note and write `Graphiti backfill required`.
- The repository already contains unrelated dirty files. Stage only files in this task and never revert unrelated user changes.
- GitNexus blast-radius notes already observed during planning:
  - `check_buy` is `CRITICAL`, but the d=1 callers are bounded to `src/cli/handlers.rs`, `src/execution/daemon.rs`, and tests.
  - `run_risk_command` is `CRITICAL`, but the d=1 caller is only `src/cli/mod.rs::run`.
  - `RiskRuleType` is `LOW`.

## File Map

- `src/risk/models.rs`
  - Add `RiskRuleType::VolatilityLimit` and percentage-only parsing support.
- `src/risk/mod.rs`
  - Re-export the new volatility helper surface if a new module is added.
- `src/risk/volatility.rs`
  - New focused helper for ATR-based volatility evaluation and risk-side daily-bar loading.
- `src/risk/service.rs`
  - Integrate `volatility-limit` into `check_buy()` without changing buy-lock semantics.
- `tests/risk_service_test.rs`
  - Add rule parsing and state-preservation tests.
- `tests/risk_volatility_test.rs`
  - New focused service tests for below-threshold, above-threshold, and insufficient-bars cases.
- `src/cli/tests/risk.rs`
  - Add parser / handler coverage for `volatility-limit`.
- `src/cli/handlers.rs`
  - Add focused tests for direct `trade buy` and strategy risk-bridge rejection reasons.
- `README.md`
  - Document `volatility-limit` as a supported rule.
- `docs/USER_MANUAL.md`
  - Document command examples, semantics, and fail-closed behavior.
- `tests/repo_hygiene_test.rs`
  - Lock updated docs wording and examples.

## Implementation Assumptions To Preserve

The following design constraints must remain true:

1. `volatility-limit` is a stock-level pre-buy gate, not an account lock
2. the formula is fixed to `ATR(14) / latest_close * 100`
3. the rule accepts percentage values only
4. missing or insufficient bars reject the buy instead of bypassing the rule
5. sell orders remain unaffected
6. no new `risk log` event type is added for volatility-triggered rejections

## Chunk 1: Rule Type And CLI Surface

### Task 1: Add `volatility-limit` to the risk rule model and CLI-facing rule parsing

**Files:**
- Modify: `src/risk/models.rs`
- Modify: `src/risk/mod.rs`
- Modify: `tests/risk_service_test.rs`
- Modify: `src/cli/tests/risk.rs`

- [ ] **Step 1: Run GitNexus impact analysis for rule-model symbols**

Run:
```text
gitnexus_impact({repo: "quantix-rust", target: "RiskRuleType", direction: "upstream", includeTests: true})
gitnexus_impact({repo: "quantix-rust", target: "RuleValue", direction: "upstream", includeTests: true})
```

Expected:
- `RiskRuleType` stays low risk; `RuleValue` may fan into existing risk tests and status rendering.

- [ ] **Step 2: Write failing parsing and persistence tests**

Add test coverage for:
- `set_rule("volatility-limit", "4%")` succeeds and stores `RuleValue::Percentage(dec!(4))`
- `set_rule("volatility-limit", "50000")` fails
- `Cli::try_parse_from(["quantix", "risk", "rule", "set", "--type", "volatility-limit", "--value", "4%"])` parses successfully
- `run risk rule set --type volatility-limit --value 4%` persists the rule into the JSON risk store

Suggested assertions:

```rust
assert_eq!(rule.rule_type, RiskRuleType::VolatilityLimit);
assert_eq!(rule.value, RuleValue::Percentage(dec!(4)));
assert!(err.to_string().contains("volatility-limit"));
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_service_test --lib cli::tests::risk:: -- --nocapture
```

Expected:
- FAIL because `volatility-limit` is not yet recognized.

- [ ] **Step 4: Implement the new rule type and value parsing**

Implement:
- `RiskRuleType::VolatilityLimit`
- `"volatility-limit"` support in `parse()` and `as_cli_str()`
- percentage-only parsing path in `RuleValue::parse()`
- any re-export updates required by `src/risk/mod.rs`

Keep behavior aligned with the spec:
- `4%` is valid
- amount syntax is invalid
- no new command family is introduced

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_service_test --lib cli::tests::risk:: -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run change detection and commit**

Run:
```text
gitnexus_detect_changes({repo: "quantix-rust", scope: "all"})
```

Expected:
- diff is focused to risk models, exports, and rule-surface tests; unrelated dirty files may still appear globally, so stage only the files above.

Commit:
```bash
git add src/risk/models.rs src/risk/mod.rs tests/risk_service_test.rs src/cli/tests/risk.rs
git commit -m "feat: add volatility-limit rule type"
```

## Chunk 2: ATR-Based Volatility Evaluation In RiskService

### Task 2: Add the risk-side bar-loading boundary and ATR-based volatility enforcement

**Files:**
- Create: `src/risk/volatility.rs`
- Modify: `src/risk/mod.rs`
- Modify: `src/risk/service.rs`
- Create: `tests/risk_volatility_test.rs`

- [ ] **Step 1: Run GitNexus impact analysis for buy-check orchestration**

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

- [ ] **Step 2: Write failing volatility-behavior tests**

Create focused tests for:
- rule configured, volatility below threshold -> buy allowed
- rule configured, volatility above threshold -> buy rejected with `volatility-limit` in the message
- rule configured, fewer than 15 bars available -> buy rejected with “检查失败” semantics
- rule configured, volatility rejection does not create a buy lock or append a new event

Suggested setup:
- fake `RiskStore`
- fake daily-bar loader returning deterministic `Vec<Kline>`
- snapshots using existing `RiskAccountSnapshot::new(...)`

Suggested assertions:

```rust
assert!(service.check_buy(...).await.is_ok());
assert!(err.to_string().contains("volatility-limit"));
assert!(err.to_string().contains("检查失败"));
assert!(!saved_state.buy_lock.locked);
assert!(saved_state.events.is_empty());
```

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_volatility_test -- --nocapture
```

Expected:
- FAIL because no volatility helper or ATR enforcement exists yet.

- [ ] **Step 4: Implement the helper and integrate it into `check_buy()`**

Implement in `src/risk/volatility.rs`:
- a thin risk-side daily-bar loading boundary
- ATR-based ratio computation using `src/analysis/indicators.rs::atr`
- minimum-bar validation for `period + 1`
- formatted error construction for:
  - above-threshold rejection
  - missing / insufficient bars

Integrate in `src/risk/service.rs`:
- keep current order:
  1. refresh state
  2. enforce buy lock
  3. enforce `position-limit`
  4. enforce `volatility-limit`
- keep sell behavior out of scope
- keep `risk_state` schema unchanged
- do not add volatility-triggered event writes

Prefer an API that still allows tests to inject a fake loader without forcing CLI or strategy code to compute ATR externally.

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_volatility_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run regression tests for the existing risk baseline**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_service_test -- --nocapture
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
git add src/risk/volatility.rs src/risk/mod.rs src/risk/service.rs tests/risk_volatility_test.rs tests/risk_service_test.rs
git commit -m "feat: enforce volatility-limit in buy checks"
```

## Chunk 3: Caller-Facing Regression Coverage

### Task 3: Prove direct trade buys and strategy risk bridges surface the same rejection reason

**Files:**
- Modify: `src/cli/handlers.rs`
- Modify: `src/cli/tests/risk.rs` (only if extra command-surface assertions are needed)
- Modify: `tests/execution_daemon_test.rs` (optional only if compile or behavior changes require explicit daemon regression coverage)

- [ ] **Step 1: Review caller blast radius from `check_buy` before editing tests**

Re-check or reuse the earlier impact result for `check_buy`.

Expected d=1 caller set:
- `src/cli/handlers.rs:evaluate`
- `src/cli/handlers.rs:execute_trade_command_with_risk`
- `src/execution/daemon.rs:evaluate`

- [ ] **Step 2: Write failing caller-surface tests**

Add focused tests in `src/cli/handlers.rs` for:
- direct `trade buy` rejection when `volatility-limit` is enabled and fake bars are too volatile
- strategy paper risk-bridge rejection carries the same `volatility-limit` reason
- strategy mock-live risk-bridge rejection carries the same `volatility-limit` reason
- direct sell path remains allowed even when the rule is enabled

Suggested assertions:

```rust
assert!(err.to_string().contains("volatility-limit"));
assert!(matches!(decision, RiskDecision::Reject { .. }));
assert_eq!(decision, RiskDecision::Allow); // for sell
```

If `tests/execution_daemon_test.rs` needs an explicit regression, add:
- pending request remains unexecuted or is rejected with the same reason when the symbol breaches `volatility-limit`

- [ ] **Step 3: Run focused tests to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --lib test_execute_trade_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --lib test_strategy_ -- --nocapture
```

Expected:
- FAIL because caller-facing volatility behavior is not yet covered or not yet wired for injection.

- [ ] **Step 4: Make the minimal caller-surface changes required**

Only change production code if tests prove it is necessary.

Allowed changes:
- expose or use a constructor/helper that lets tests and internal bridges inject a fake risk-side bar loader
- keep production call sites on the same `RiskService` orchestration surface
- add only the minimal glue needed for deterministic tests

Do not:
- move ATR computation into CLI handlers
- create a second rule-evaluation path for strategy code

- [ ] **Step 5: Re-run focused tests to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --lib test_execute_trade_ -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --lib test_strategy_ -- --nocapture
```

Expected:
- PASS

- [ ] **Step 6: Run broader regressions that cover runtime execution**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test strategy_paper_run_test --test strategy_mock_live_run_test --test execution_daemon_test -- --nocapture
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
git add src/cli/handlers.rs src/cli/tests/risk.rs tests/execution_daemon_test.rs
git commit -m "test: cover volatility-limit caller paths"
```

## Chunk 4: Docs And Hygiene

### Task 4: Document `volatility-limit` and update repo hygiene locks

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Modify: `tests/repo_hygiene_test.rs`

- [ ] **Step 1: Write the failing hygiene expectations**

Add assertions that docs mention:
- `quantix risk rule set --type volatility-limit --value 4%`
- `volatility-limit` is now supported
- the rule is ATR-based and fail-closed on missing bars
- sells remain unaffected

- [ ] **Step 2: Run hygiene test to verify RED**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- FAIL because docs do not yet mention the new rule.

- [ ] **Step 3: Update README and USER_MANUAL**

Document:
- `volatility-limit` as a supported risk rule
- percentage-only syntax
- ATR(14) / latest_close semantics
- fail-closed behavior when bars are unavailable
- that the rule applies only to new buys and does not affect sells

- [ ] **Step 4: Re-run hygiene test to verify GREEN**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test repo_hygiene_test -- --nocapture
```

Expected:
- PASS

- [ ] **Step 5: Run final focused verification for the whole slice**

Run:
```bash
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --test risk_service_test --test risk_volatility_test --test strategy_paper_run_test --test strategy_mock_live_run_test --test execution_daemon_test --test repo_hygiene_test -- --nocapture
CARGO_TARGET_DIR=/tmp/quantix-target-phase27c cargo test --lib cli::tests::risk:: -- --nocapture
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
git add README.md docs/USER_MANUAL.md tests/repo_hygiene_test.rs docs/superpowers/specs/2026-03-24-phase27c-risk-volatility-rule-design.md docs/superpowers/plans/2026-03-24-phase27c-risk-volatility-rule-implementation.md
git commit -m "docs: document volatility-limit risk rule"
```

## Verification Notes

- Use `CARGO_TARGET_DIR=/tmp/quantix-target-phase27c` consistently to avoid interference from existing local target-directory state.
- Because the repository is already dirty, always inspect staged files before each commit:

```bash
git diff --staged --stat
git diff --staged
```

- If Graphiti write-back during implementation still hits rate limits, append a local summary to this plan file or the implementation checkpoint note and include:

```text
Graphiti backfill required
```
