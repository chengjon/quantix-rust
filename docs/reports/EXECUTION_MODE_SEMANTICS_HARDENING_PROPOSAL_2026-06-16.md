# Execution Mode Semantics Hardening Proposal

Date: 2026-06-16

Status: proposal / governance baseline

FUNCTION_TREE node: `Execution mode semantics hardening`

Governance node: `project-governance/P0.2`

## Purpose

This document records the official design input for the next trading-execution
mainline stage. It does not implement runtime behavior. It defines the semantic
baseline that future execution work must follow before changing adapters,
storage, order status contracts, broker reconciliation, or simulated matching.

The core decision is to keep three execution channels separate:

| Channel | Fill source | Status source | miniQMT dependency | Current role |
|---|---|---|---|---|
| `paper_immediate` | Quantix local paper ledger immediate fill | Local `TradeRecord` | No | Existing fast regression and strategy execution closure |
| `paper_sim_lifecycle` | Quantix local simulated matcher | Local simulated order lifecycle | No | Future isolated simulation engine |
| `qmt_live` | miniQMT / broker / exchange path | miniQMT / broker query result | Yes | Real-money path guarded by bridge capability and live gates |

The existing `paper` adapter behavior maps to `paper_immediate`. It is a local
ledger simulator, not a broker simulator and not a market/exchange matching
engine.

## Official Input

The accepted architecture input is:

- Do not mix simulated and live trading semantics.
- Real trading must be bound to miniQMT / bridge capabilities and the broker
  status returned through that path.
- Local paper modes may share abstract interfaces, but they must not share the
  same fill-source or status-source assumptions.
- The system should be governed by five durable dimensions:
  - adapter capability,
  - status source,
  - fill source,
  - storage isolation,
  - structured error taxonomy.
- Low-intrusion semantic hardening should land before cross-cutting trait,
  storage, enum, status, broker, or matcher refactors.

This proposal accepts those constraints without changing production code.

## Current Baseline

### `paper_immediate`

The current paper path performs a local immediate fill:

1. `PaperExecutionAdapter::submit_order` converts the adapter request into a
   `TradeOrderRequest`.
2. `TradeService::buy` or `TradeService::sell` updates local paper cash,
   positions, and trade records.
3. The adapter returns `OrderStatus::Filled` and the local `TradeRecord.id` as
   `adapter_order_id`.
4. `query_order` reads the local filled `TradeRecord`.
5. `cancel_order` fail-closes for filled or missing local paper records.

This path is useful for strategy, execution, account, and regression closure.
It must not be documented as real broker execution, market liquidity validation,
or slippage validation.

### `qmt_live`

The qmt live path is broker-bound:

1. `QmtLiveExecutionAdapter::submit_order` runs the qmt live gate.
2. Submission goes through `QmtTaskSubmitService`.
3. The initial local response is `OrderStatus::PendingSubmit` with a task
   identity.
4. Subsequent query/cancel semantics depend on bridge and miniQMT capabilities,
   broker identity resolution, and broker-side state.

The source of truth for live order progress is miniQMT / broker state, not a
local paper ledger.

### `paper_sim_lifecycle`

This channel is not implemented. It is reserved for a future isolated simulated
order lifecycle with its own order store, matcher, freeze/release model, and
tests. It must not be implemented by extending `paper_immediate` in place.

## P0.2 Stage Scope

P0.2 is a semantic-hardening stage. It should be split into small slices.

### P0.2a: Proposal And Registry Alignment

This current slice is documentation and governance only:

- add this proposal,
- register the semantic boundary in `FUNCTION_TREE.md`,
- preserve active code behavior,
- avoid all production-code changes.

Explicit non-goals:

- no `ExecutionAdapter` trait changes,
- no `OrderStatus` or response shape changes,
- no qmt live implementation changes,
- no paper lifecycle implementation,
- no storage changes,
- no matcher changes,
- no `.unwrap()` cleanup.

### P0.2b: `paper_immediate` Behavior Lock

Future low-intrusion implementation slice:

- add an explicit `IMMEDIATE_FILL_ONLY` marker or equivalent code-level
  boundary,
- keep submit returning `Filled`,
- keep query limited to filled local `TradeRecord`,
- keep filled/missing cancel fail-closed,
- add hygiene checks preventing pending lifecycle logic from being added to the
  immediate-fill adapter.

This slice may touch `src/execution/paper.rs`, focused tests, and repo hygiene
tests only after fresh GitNexus impact analysis.

### P0.2c: Channel Risk Logging

Future low-intrusion implementation slice:

- standardize channel labels:
  - `[paper_immediate]`,
  - `[paper_sim_lifecycle]`,
  - `[qmt_live]`;
- print risk text at controlled execution boundaries;
- avoid changing execution decisions.

Standard risk texts:

- `paper_immediate`: local ledger only; no real matching; not valid for
  liquidity or slippage verification.
- `paper_sim_lifecycle`: local simulated matching; may diverge from broker
  behavior.
- `qmt_live`: real-money path; order status is governed by miniQMT / broker
  state.

### P0.2d: Execution Mode Storage Isolation Assessment

Future assessment slice:

- identify where execution mode is loaded,
- identify whether runtime mode switching is possible,
- identify current storage roots and account namespaces,
- recommend enforcement if any path can mix live and simulated state.

This should start as evidence collection before code changes.

### P0.2e: Paper Cancel Failure Taxonomy Seed

Future low-intrusion implementation slice:

- define internal paper cancel failure categories such as:
  - `OrderNotFound`,
  - `OrderAlreadyFilled`;
- keep external adapter compatibility until a later public error taxonomy
  project changes upstream contracts.

This slice must not force global `AdapterError` migration.

## Deferred Cross-Cutting Projects

The following are explicitly out of P0.2a and must be separate work.

### Execution Capabilities MVP

Introduce a single capability descriptor, for example:

```rust
fn capabilities(&self) -> ExecutionCapabilities;
```

Do not replace all mode checks in one PR. First let each adapter report static
capabilities, then migrate upper layers gradually.

### Status Source Layering

Do not let `OrderStatus` alone carry semantics. A future status response should
also expose whether status came from local immediate fill, local simulation, or
broker/miniQMT state.

This may require response shape changes and display/logging updates, so it must
be a separate project.

### qmt_live miniQMT Capability And Identity Hardening

This project must be bound to miniQMT and bridge reality:

- bridge / miniQMT version and schema gates,
- required field compatibility checks,
- `task_id <-> external_order_id` reconciliation,
- restart and disconnect recovery,
- broker rejection and unknown-state taxonomy.

It must not depend on local paper simulation semantics.

### `paper_sim_lifecycle`

This project must be isolated from `paper_immediate`:

- separate order store,
- separate lifecycle state,
- separate matcher abstraction,
- freeze/release cash and holdings,
- explicit `expire_orders(now)` instead of a hidden background daemon,
- characterization tests before sharing any ledger calculation utility.

## Risk Admission Rule

For future cross-cutting execution work:

- LOW GitNexus impact can enter design authorization.
- MEDIUM impact requires an explicit test plan and user confirmation.
- HIGH or CRITICAL impact must stop for split, redesign, or separate approval.

This is stricter than a blanket LOW/MEDIUM allow-list because trading behavior
touches real-money and simulated-account semantics.

## Documentation Wording Standard

Use these phrases consistently:

- `paper_immediate`: local immediate-fill ledger simulation.
- `paper_sim_lifecycle`: local simulated order lifecycle.
- `qmt_live`: miniQMT / broker-backed live execution.

Avoid these phrases unless the implementation really supports them:

- paper market matching,
- broker-like paper order book,
- live-compatible paper status,
- miniQMT-equivalent paper lifecycle,
- real fill for paper orders.

## Acceptance Boundary For This Proposal

P0.2a is complete when:

- this proposal exists,
- `FUNCTION_TREE.md` records the semantic hardening boundary,
- governance records the P0.2 scope,
- no production code changed,
- `function-tree validate` passes,
- `git diff --check` passes,
- GitNexus detect-changes reports documentation/governance-only scope,
- Graphiti memory is written and ingest is verified.

