# Phase 3 Research

## Phase

Phase 3: Real live / broker execution closure

## Current State

Phase 2 closed the request-semantics and operator-diagnostics gap, so Phase 3 can now focus on the real-live broker boundary without re-opening the earlier meaning drift around `completed`, `order_status`, or `qmt_live`.

This phase is not greenfield. The repository already contains a meaningful QMT real-submit skeleton:

- `src/execution/qmt_live_adapter.rs` defines `QmtLiveExecutionAdapter` and already submits to the bridge `/api/v1/broker/qmt/orders` endpoint.
- `src/execution/qmt_live_gate.rs` defines `ensure_bridge_qmt_live_mode`, which currently rejects live submission unless bridge capabilities report `qmt.enabled=true` and `qmt.mode=live`.
- `src/execution/daemon.rs` already dispatches `target_mode=qmt_live` through the live adapter, while generic `target_mode=live` remains explicitly unsupported.
- `src/cli/handlers/mod.rs` already exposes `execution bridge qmt-live`, manual confirmation flow, request claiming, failure persistence, and post-submit query guidance.
- Existing tests already cover the first safety shell:
  - `tests/qmt_live_gate_test.rs` rejects `preview_only` and allows `live`
  - `tests/qmt_live_adapter_test.rs` rejects preview-only bridge mode and allows submit in live mode
  - `src/cli/handlers/tests/mod.rs` covers manual `qmt_live` bridge behavior and generic `live` boundary messaging
- Existing docs already state that:
  - `qmt_live` is the only guarded real-submit path
  - generic `target_mode=live` is still not implemented
  - QMT preview-only does not place real orders

That means Phase 3 is a closure and hardening phase: formalize the real-live contract, tighten the explicit gate, and lock the minimal operator verification flow so the implementation cannot silently drift.

## Gaps Against Phase 3 Scope

### 1. The real-live contract exists in code, but not yet as a phase-level planning artifact

The repo already has a live adapter, a gate helper, daemon dispatch, and a manual bridge path. What is still missing is a formal phase artifact that states:

- what `qmt_live` guarantees
- what generic `live` still does **not** guarantee
- which files own the broker boundary
- which tests prove the contract

Without that artifact, the code can continue to grow while the intended boundary remains tribal knowledge.

### 2. The QMT live gate is explicit, but still minimal

Today the gate checks:

- `qmt.enabled`
- `qmt.mode == live`

That is a real improvement over preview-only, but Phase 3 should explicitly decide whether the minimum live-submit contract also requires bridge capability support such as `order_submit`, plus consistent failure semantics across:

- `QmtLiveExecutionAdapter`
- `execution daemon`
- `execution bridge qmt-live`

The code already funnels through the same gate conceptually, but Phase 3 should lock that as a requirement-backed contract rather than a best-effort convention.

### 3. The operator verification path exists, but is not yet encoded as the minimum safety workflow

The manual handler already:

- prints a real-order confirmation screen
- requires `YES` unless skipped
- persists executor start / fail / success data
- prints a follow-up `qmt-query` command

However, the repository still lacks a formal statement of the minimum safe verification loop for real submit:

- preconditions before submission
- exact guarded path (`qmt_live`, not generic `live`)
- how to confirm bridge mode
- how to verify the broker-side order after submission
- which docs and tests lock those expectations

Phase 3 should formalize that workflow instead of leaving it implicit in handler code.

## Code Surfaces Most Likely Needed

Primary implementation files:

- `src/execution/qmt_live_gate.rs`
- `src/execution/qmt_live_adapter.rs`
- `src/execution/daemon.rs`
- `src/cli/handlers/mod.rs`

Primary regression files:

- `tests/qmt_live_gate_test.rs`
- `tests/qmt_live_adapter_test.rs`
- `tests/execution_daemon_test.rs`
- `src/cli/handlers/tests/mod.rs`
- `tests/repo_hygiene_test.rs`

Primary docs:

- `README.md`
- `docs/USER_MANUAL.md`

## Recommended Plan Shape

### Plan 01: Live adapter contract and broker boundary lock

Goal:

- define and lock the contract for `qmt_live` versus generic `live`

Why first:

- it closes `LIV-01`
- it turns the current code skeleton into an explicit boundary
- it reduces the risk of accidental capability drift before any deeper hardening

Likely work:

- extend gate / adapter / daemon / handler regressions around `qmt_live` versus `live`
- normalize the error and boundary messages that point operators toward the guarded path
- lock which request paths are allowed to submit real orders

### Plan 02: Explicit QMT live gate hardening

Goal:

- tighten the real-submit gate from "bridge says live" into a minimum real-submit contract with regression coverage

Why second:

- it closes `LIV-02`
- it builds on the contract defined in Plan 01
- it keeps the hardening focused on the existing QMT path instead of opening generic broker live support

Likely work:

- extend `ensure_bridge_qmt_live_mode`
- add failure-path regressions for missing capability support or misconfigured bridge state
- ensure both daemon-driven and manual-bridge submission paths persist the same failure semantics

### Plan 03: Minimal safety constraints and verification flow

Goal:

- encode the operator safety checklist, verification flow, and docs/test locks for real submit

Why third:

- it closes `LIV-03`
- docs and repo-hygiene locks should reflect the final contract from Plans 01 and 02
- it keeps Phase 3 focused on safe closure, not on broad broker expansion

Likely work:

- update README / USER_MANUAL real-submit wording
- lock repo-level wording around preconditions and post-submit verification
- ensure manual bridge output and docs teach the same minimal verification loop

## Verification Strategy

Phase 3 should validate at three layers:

### 1. Gate and adapter coverage

Use:

- `tests/qmt_live_gate_test.rs`
- `tests/qmt_live_adapter_test.rs`

to prove:

- preview-only and disabled configurations cannot submit
- live mode passes the gate
- the adapter only talks to the submit endpoint after the gate is satisfied

### 2. Daemon and CLI path coverage

Use:

- `tests/execution_daemon_test.rs`
- `src/cli/handlers/tests/mod.rs`

to prove:

- daemon `qmt_live` dispatch uses the guarded path
- generic `live` remains blocked with explicit guidance
- manual bridge execution persists the expected success/failure payloads

### 3. Documentation and hygiene coverage

Use:

- `tests/repo_hygiene_test.rs`

to prove:

- docs still teach `qmt_live` as the only guarded real-submit path
- docs do not claim that generic `target_mode=live` is ready
- the safety and verification workflow does not drift from the actual CLI behavior

## Validation Architecture

Phase 3 should be validated backward from the roadmap success criteria:

1. the live adapter contract and broker boundary are explicit and consistent
2. the QMT real-submit path is guarded by an explicit live gate with regressions
3. the minimum safety constraints and operator verification flow are documented and test-locked

Validation dimensions:

- Boundary correctness:
  - `qmt_live` is explicit
  - generic `live` remains unsupported
  - daemon / CLI / docs agree on that split

- Gate correctness:
  - preview-only or disabled bridge modes fail before real submission
  - live mode is necessary for real submit
  - failure payloads remain diagnosable

- Safety correctness:
  - real-submit flow surfaces meaningful confirmation and post-submit verification guidance
  - docs teach the same safety preconditions as the code
  - no wording implies that preview-only or generic live are safe substitutes for `qmt_live`

## Risks And Constraints

- Do not broaden Phase 3 into generic multi-broker live support.
- Keep generic `target_mode=live` explicitly unsupported until the real-live contract is stronger than the current QMT-only path.
- Prefer tightening the current gate and verification flow before adding any new broker features.
- Preserve the Phase 2 semantics contract: request completion still does not imply order terminality even on the `qmt_live` path.

## Candidate Verification Commands

- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test qmt_live_gate_test --test qmt_live_adapter_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib qmt_live`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test`

## Planning Recommendation

Proceed with a three-plan phase:

1. lock the `qmt_live` contract and generic `live` boundary
2. harden the explicit QMT live gate and its failure semantics
3. lock the minimal safety and verification workflow in docs and tests

This is the smallest phase shape that closes `LIV-01`, `LIV-02`, and `LIV-03` without pretending the repo already supports a broader real-live broker model.
