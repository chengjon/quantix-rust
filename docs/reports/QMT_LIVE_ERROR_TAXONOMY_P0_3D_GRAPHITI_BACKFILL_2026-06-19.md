# P0.3d qmt_live Error Taxonomy Graphiti Backfill

Date: 2026-06-19

Graphiti backfill required

## Summary

P0.3d `qmt_live` error taxonomy seed was completed and merged.

- PR: #245
- Merge commit: `f0c70b64b37b9ba82182d379baaffb4108c22a78`
- Master CI: `27799550127`, completed successfully
- FUNCTION_TREE: P0.3d closed, active gates none, validation passed

## Implemented Scope

- Added `QmtLiveErrorCategory` in `src/execution/qmt_task_submit_service.rs`.
- Seeded qmt_live-local categories for bridge failure, broker rejection, broker unknown state, manual intervention required, and reserved local validation / local risk gate rejection.
- Added contract coverage in `tests/qmt_task_contract_test.rs`.

## Preserved Boundaries

- No CLI handler changes.
- No `request_diagnostics` changes.
- No `ExecutionAdapter` trait changes.
- No `OrderStatus` changes.
- No bridge protocol, response shape, storage schema, or background daemon changes.
- No `.unwrap()` cleanup resumed.

## Verification

- PR CI passed for Lint and Test; skipped matrix jobs followed existing workflow configuration.
- Master CI passed for Documentation, Lint, and Test; skipped matrix jobs followed existing workflow configuration.
- GitNexus reported LOW risk / no affected processes for closeout governance changes before commit.

## Graphiti Failure Evidence

Two Graphiti memory writes were attempted for group `quantix_rust_main` and both failed during ingest with `Request timed out.`:

- `e1c00fcc-7b98-4243-9081-7b914d1aafc1`
- `c0bfb5bb-4a77-4416-87b8-262f222c356a`

`mcp__graphiti_memory.get_status` later reported the Graphiti MCP server and Neo4j connection as healthy. The remaining action is to backfill this summary into `quantix_rust_main` when Graphiti ingest is stable.
