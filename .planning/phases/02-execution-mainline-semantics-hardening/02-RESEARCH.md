# Phase 2 Research

## Phase

Phase 2: Execution mainline semantics hardening

## Current State

Phase 1 closed the live-ready hardening foundation, but the execution mainline still carries a semantics gap between request lifecycle truth and order lifecycle truth.

The current code already exposes the main surfaces that Phase 2 must tighten:

- `src/cli/handlers/mod.rs` contains the operator-facing request and daemon formatters:
  - `format_strategy_request_detail`
  - `format_strategy_request_row`
  - `format_execution_daemon_summary`
  - `format_execution_request_result`
- `src/execution/daemon.rs` emits `ExecutionDaemonIterationSummary` and writes `execution_result` / `execution_error` payloads back into `execution_request.payload_json`.
- `src/execution/runtime_store.rs` already models request lifecycle states including `pending`, `in_progress`, `completed`, `failed`, and `canceled`.
- Existing tests already prove part of the semantics:
  - `src/cli/handlers/tests/mod.rs` now locks that `status: completed` and `order_status: accepted` can coexist in request detail output.
  - `tests/execution_daemon_test.rs` verifies that daemon consumption can end with `request_status = Completed` while `payload_json.execution_result.order_status` remains non-terminal, and that failure payloads persist `execution_error.message`.
  - `tests/execution_runtime_store_test.rs` verifies request status transitions, including `pending -> in_progress`.
  - `tests/repo_hygiene_test.rs` and docs now lock the `mock_live` / `qmt_live` / `live` boundary.

This means Phase 2 is not a greenfield implementation. It is a semantics and observability tightening pass over existing request, daemon, and operator surfaces.

## Gaps Against Phase 2 Scope

### 1. Request completion is still easy to misread as order completion

Today the request detail formatter prints both request status and `order_status`, but it still relies on the operator to infer their relationship. The one-line row and daemon summary outputs are even more compact, so an operator scanning CLI output can still collapse:

- `request_status = completed`
- `order_status = accepted | partially_filled | pending_cancel | unknown`

into a false mental model of "the order is done".

Phase 2 needs outputs that make this distinction explicit instead of merely implicit.

### 2. Diagnostics exist, but they are not yet consistently shaped for request triage

Current payloads already store useful fields such as:

- `execution_error.message`
- `executed_at`
- `failed_at`
- `client_order_id`
- adapter-specific fields for some paths

But the operator-facing summaries do not yet present a stable, compact diagnostic contract for:

- why a request failed
- whether a request is only accepted vs truly terminal
- whether a request is stuck `in_progress`
- which adapter or executor path wrote the current payload

Phase 2 should turn these from raw payload fragments into explicit CLI/operator semantics.

### 3. The low-risk entry points are helper formatters, not the full request-show handler

The safest implementation seam is the formatting layer:

- `format_strategy_request_detail`
- `format_strategy_request_row`
- `format_execution_daemon_summary`
- `format_execution_request_result`

Those helpers can be hardened with focused tests and have low blast radius.

By contrast, the broader `execute_strategy_request_show` path was previously identified as high-risk during earlier impact analysis. Phase 2 planning should therefore bias toward:

- tightening helper semantics first
- only expanding higher-risk handler behavior when tests prove the need

### 4. Request lifecycle and docs are closer now, but not fully unified

Phase 1 tightened README and USER_MANUAL language, but Phase 2 success criteria require the docs and CLI to converge on the same operator truth:

- request completion means execution handoff/processing completion
- order terminality is a separate dimension
- `paper`, `mock_live`, `qmt_live`, and generic `live` must remain clearly separated

That means the last docs pass in this phase should happen after the CLI semantics are finalized, not before.

## Code Surfaces Most Likely Needed

Primary implementation files:

- `src/cli/handlers/mod.rs`
- `src/execution/daemon.rs`
- `src/execution/runtime_store.rs`

Primary regression files:

- `src/cli/handlers/tests/mod.rs`
- `tests/execution_daemon_test.rs`
- `tests/execution_runtime_store_test.rs`
- `tests/repo_hygiene_test.rs`

Primary docs:

- `README.md`
- `docs/USER_MANUAL.md`

## Recommended Plan Shape

### Plan 01: Lock request-vs-order semantics in CLI output

Goal:
- make request detail, request row, and daemon summary outputs explicitly distinguish request lifecycle from order lifecycle

Why first:
- it addresses `SEM-01` directly
- it uses low-risk formatting seams
- it gives operators immediate clarity before broader diagnostic expansion

Likely work:
- extend `src/cli/handlers/tests/mod.rs` with explicit request-vs-order semantics regressions
- harden `format_strategy_request_detail`
- harden `format_strategy_request_row`
- harden `format_execution_daemon_summary`

### Plan 02: Tighten request diagnostics and stuck/failure observability

Goal:
- make failed and in-progress requests easier to diagnose from request payloads and daemon/operator output

Why second:
- depends on Plan 01 defining the output contract
- closes `SEM-02` without drifting into broker/live implementation

Likely work:
- tighten payload shaping in `src/execution/daemon.rs`
- add or normalize executor / adapter / timestamp / reason fields in `payload_json`
- extend `tests/execution_daemon_test.rs` and `tests/execution_runtime_store_test.rs`
- surface those diagnostics through `format_strategy_request_detail` and daemon summaries

### Plan 03: Re-sync docs and hygiene locks with the final semantics

Goal:
- update docs and repo-level wording to match the hardened CLI behavior from Plans 01 and 02

Why third:
- docs should lock the final operator truth, not an intermediate draft

Likely work:
- update `README.md`
- update `docs/USER_MANUAL.md`
- extend `tests/repo_hygiene_test.rs`

## Verification Strategy

Phase 2 should be validated at three layers:

### 1. Formatter-level unit coverage

Use `src/cli/handlers/tests/mod.rs` to lock:

- request detail output containing both request and order status
- daemon summary output for non-terminal, terminal, and failed requests
- request row output for completed, failed, canceled, and in-progress requests

### 2. Daemon/runtime integration coverage

Use:

- `tests/execution_daemon_test.rs`
- `tests/execution_runtime_store_test.rs`

to prove:

- request statuses transition correctly
- failure payloads preserve actionable reasons
- non-terminal order statuses are not rewritten into false terminal semantics

### 3. Repo-level wording coverage

Use:

- `tests/repo_hygiene_test.rs`

to prove docs and operator language do not drift back toward:

- "completed means filled"
- "`mock_live` equals real live"
- "generic `live` is already available"

## Validation Architecture

Phase 2 should be validated backward from the roadmap success criteria:

1. CLI/operator outputs explicitly distinguish request completion from order terminality.
2. daemon/operator outputs carry enough request diagnostics to localize failures and stuck states.
3. docs and CLI maintain the same `paper` / `mock_live` / `qmt_live` / `live` boundary.

Validation dimensions:

- Semantics correctness:
  - request and order states are rendered as separate truths
  - non-terminal order states remain visible after request completion
  - failed and canceled requests surface explicit reasons

- Observability correctness:
  - daemon summaries surface enough information to explain the latest request outcome
  - request detail output includes the fields an operator needs for triage
  - `in_progress` remains diagnosable rather than silent

- Boundary correctness:
  - CLI wording and docs do not imply real broker live execution where it does not exist
  - `qmt_live` remains the only explicitly guarded real-submit path

## Risks And Constraints

- Do not let Phase 2 mutate into Phase 3 real-broker work.
- Prefer helper-formatting seams before editing higher-risk command handlers.
- Keep diffs reviewable: semantic clarity and diagnostics first, docs sync second.
- Preserve the Phase 1 wording contract unless a stronger Phase 2 wording is intentionally locked in tests.

## Candidate Verification Commands

- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --lib 'cli::handlers::tests::test_format_strategy_request_detail_keeps_request_status_separate_from_order_status' -- --exact`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_daemon_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test execution_runtime_store_test`
- `env RUSTFLAGS=-Clink-arg=-fuse-ld=bfd cargo test --test repo_hygiene_test`

## Planning Recommendation

Proceed with a three-plan phase:

1. lock request-vs-order semantics in CLI output
2. harden request diagnostics and stuck/failure observability
3. re-sync docs and repo hygiene with the final semantics

This is the smallest plan shape that fully covers `SEM-01`, `SEM-02`, and `SEM-03` without prematurely entering the broker boundary work reserved for Phase 3.
