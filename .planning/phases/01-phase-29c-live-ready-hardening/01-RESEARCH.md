# Phase 1 Research

## Phase

Phase 1: Phase 29C live-ready hardening

## Current State

Phase 29C is not starting from zero. The codebase already contains the core mock-live foundation that the earlier design docs described:

- `src/execution/mock_live.rs` already defines `MockLiveClock` and `MockLiveExecutionAdapter`, and the symbol outline shows submit/query/cancel support plus private-state handling.
- `src/execution/runtime_store.rs` already contains schema constants and store helpers for `mock_live_orders`, optimistic versioned order updates, shared order lifecycle columns such as `remaining_quantity`, `last_transition_at`, and `version`.
- `src/execution/reconciliation.rs` already contains `OpenOrderScanner`, `ReconciliationService`, `ReconciliationSummary`, `OrderReconciliationResult`, and reconciliation action types.
- The test suite already exercises substantial lifecycle behavior:
  - `tests/mock_live_adapter_test.rs` covers accepted-by-default submit, partial/final fill progression, cancel resolution, one-shot unknown recovery, and duplicate fill-id handling.
  - `tests/execution_kernel_test.rs` covers non-final submit persistence, partial-fill accounting via fill-delta appliers, fill-apply failure behavior, risk rejection, duplicate client-order idempotency, recovery advancement, pending-cancel resolution, and unknown retry exhaustion.
  - `tests/strategy_mock_live_run_test.rs` covers end-to-end mock-live runs, run dedupe, recovery-driven order advancement, and applying only new fill deltas to account state.
- Design docs already lock key invariants:
  - Phase 29A keeps `paper_trade.json` / `risk_state.json` authoritative and uses `runtime.db` for durable execution audit.
  - Phase 29C introduces non-final states, `Unknown`, private mock-live state, query/recovery semantics, and reconciliation scaffolding.
  - Real broker `live` execution remains out of scope.

Net: the repository already appears to have the Phase 29C foundation slice implemented. Phase 1 planning should therefore focus on hardening gaps and closing the delta between current implementation and the repo-level backlog wording, not on inventing a fresh subsystem.

## Gaps Against Phase 1 Scope

The backlog phrasing in `.planning/ROADMAP.md` is broader than the currently verified foundation. The likely missing or weakly-closed areas are:

### 1. Delayed / partial fills are foundation-level, not full hardening-level

Current tests prove partial-fill progression exists, but they do not yet prove:

- broader fill-plan variants beyond the current happy-path sequences
- richer timing edge cases
- behavior under repeated recovery / polling loops across longer scenarios
- handler- or operator-level surfaces that make these states observable outside kernel tests

### 2. `Unknown` handling is present but narrow

Current tests cover one-shot unknown recovery and retry exhaustion, but likely gaps remain around:

- repeated `Unknown -> known -> Unknown` transitions
- user-facing visibility of exhaustion / recovery status
- open-order scanning behavior for long-lived unknowns
- interaction between unknown state and reconciliation summaries

### 3. Open-order scanning and query reconciliation need a system-level closure

`src/execution/reconciliation.rs` exists, but the roadmap language suggests the repo still needs stronger proof that:

- scanner thresholds and unknown timeouts are correct for operator workflows
- reconciliation actions map cleanly to real recovery outcomes
- reconciliation is visible through docs/CLI/operator entrypoints
- summary/reporting is sufficient for debugging stale and unknown orders

### 4. Network fault simulation is probably under-scoped in explicit tests and docs

The Phase 29C spec expects simulated network fault handling. Current symbol and test inventory suggests unknown/fault behavior exists in adapter-private state, but there is no direct evidence yet of a dedicated fault-matrix or operator-facing regression coverage for:

- transient query failures
- submit-time uncertainty
- repeated fault/retry chains
- reconciliation behavior after simulated transport faults

### 5. Account / order reconciliation scaffolding is present, but acceptance is probably incomplete

There is already a reconciliation module, but the backlog still treats account/order reconciliation scaffolding as unfinished. The likely gap is not “missing code” but “missing closure”:

- insufficient end-to-end tests tying runtime order state to paper account state
- missing docs/README explanations of how reconciliation is expected to be used
- possible missing command or handler surfaces for surfacing reconciliation summaries

## Recommended Plan Shape

The phase should be decomposed as hardening and closure work, not as greenfield implementation.

### Plan 01: Harden mock-live lifecycle matrix and fault injection

Goal:
- close adapter/kernel edge cases around delayed fills, repeated partial fills, `Unknown`, and network-fault-style transitions

Why first:
- this locks behavior before touching operator/reconciliation surfaces

Likely work:
- expand lifecycle/fault tests in `tests/mock_live_adapter_test.rs` and `tests/execution_kernel_test.rs`
- add missing adapter-private state transitions in `src/execution/mock_live.rs`
- verify runtime-store helpers support the needed private-state updates in `src/execution/runtime_store.rs`

### Plan 02: Close recovery, open-order scanning, and reconciliation behavior

Goal:
- make recovery and reconciliation behavior robust enough that stale/unknown orders can be inspected and advanced safely

Why second:
- depends on Plan 01 having a locked lifecycle/fault model

Likely work:
- extend `src/execution/reconciliation.rs`
- add recovery/scanner edge-case tests in `tests/execution_kernel_test.rs`
- add or tighten end-to-end reconciliation coverage in `tests/strategy_mock_live_run_test.rs`

### Plan 03: Surface live-ready semantics in docs and operator-facing outputs

Goal:
- make the current boundary explicit so users can understand mock-live behavior without assuming real live broker support

Why third:
- docs and operator output should reflect the hardened behavior, not an earlier draft

Likely work:
- tighten `README.md` and `docs/USER_MANUAL.md`
- review CLI handler tests, especially around execution status summaries and live-gate wording
- ensure repo hygiene / documentation tests capture the intended boundary

This three-plan shape matches the current codebase maturity: foundation code exists, but the “hardening” backlog still needs behavior closure, operator visibility, and explicit acceptance coverage.

## Verification Strategy

The phase should be verified at three levels:

### Unit / focused integration

- `tests/mock_live_adapter_test.rs`
  - extend fill-plan matrix
  - extend `Unknown` and simulated fault chains
  - prove cancel/recovery interactions remain idempotent

- `tests/execution_kernel_test.rs`
  - prove account state only mutates on newly observed fill deltas
  - prove unknown exhaustion does not corrupt public order truth
  - prove recovery/query loops handle stale, partial, cancel, and fault transitions

### End-to-end flow

- `tests/strategy_mock_live_run_test.rs`
  - prove mock-live run summaries and runtime rows remain coherent
  - prove reconciliation/recovery scenarios mutate only expected account deltas
  - prove dedupe/idempotency still holds after hardening changes

### Documentation / boundary

- `README.md`
- `docs/USER_MANUAL.md`
- any repo-hygiene or handler-level tests that encode live-vs-mock_live wording

The success bar for Phase 1 should not just be “tests pass”; it should be:

- delayed/partial/unknown/fault scenarios are explicitly covered
- reconciliation behavior has a documented and testable story
- user-facing summaries no longer overstate capability

## Validation Architecture

Phase 1 should be validated backward from the roadmap success criteria:

1. `mock_live` can simulate delayed fill, partial fill, and `Unknown` recovery.
2. open-order and account/order reconciliation have minimal but explicit scaffolding.
3. network fault injection and recovery behavior are reproducible and observable.

Validation dimensions:

- Lifecycle correctness:
  - statuses follow allowed transitions
  - `Unknown` stays non-terminal
  - repeated recovery does not double-apply fills

- Accounting correctness:
  - paper account changes only on successful new fill deltas
  - partial fills create only the expected incremental accounting mutations

- Recovery/reconciliation correctness:
  - stale/unknown orders are surfaced by scanners
  - reconciliation summaries and actions reflect real runtime state

- Boundary correctness:
  - docs and outputs say mock-live/live-ready, not real live broker support

## Risks And Constraints

- Do not turn this phase into real broker or QMT live execution work. That belongs to Phase 3.
- Do not rewrite the execution architecture. The kernel/adapter/runtime-store structure already exists and should be hardened in place.
- Preserve existing `paper` semantics and current request lifecycle behavior.
- Keep `paper_trade.json` and `risk_state.json` authoritative; `runtime.db` remains the audit and lifecycle store.
- Prefer test-first changes because many likely gaps are around edge-case behavior and user-visible semantics, not missing scaffolding.

## Candidate Files

Primary code:

- `src/execution/mock_live.rs`
- `src/execution/runtime_store.rs`
- `src/execution/reconciliation.rs`
- `src/execution/kernel.rs`

Primary tests:

- `tests/mock_live_adapter_test.rs`
- `tests/execution_kernel_test.rs`
- `tests/strategy_mock_live_run_test.rs`
- `src/cli/handlers/tests/mod.rs`
- `tests/repo_hygiene_test.rs`

Docs:

- `README.md`
- `docs/USER_MANUAL.md`
- `docs/superpowers/specs/2026-03-22-phase29c-mock-live-execution-foundation-design.md`

## Planning Recommendation

Proceed with a three-plan phase:

1. lifecycle/fault hardening
2. recovery/reconciliation closure
3. docs and operator-surface truth sync

That is the smallest decomposition that matches the current implementation reality and closes the roadmap gap without redoing Phase 29C from scratch.
