# Changelog / Function Tree Sync Checkpoint

Date: 2026-05-01
Branch: `docs/changelog-function-tree-sync-20260501`

Historical note:
This checkpoint captures the 2026-05-01 transition state. Current active capability-boundary documentation now lives in root `FUNCTION_TREE.md`; mentions of the old function-list document below are preserved as historical context only.

## Summary

- `CHANGELOG.md` was updated to add missing 2026-04-30 and 2026-05-01 entries for:
  - structured execution diagnostics
  - miniQMT task-contract bridge/runtime alignment
  - `QmtTaskSubmitService`
  - `qmt_live` receipt/result semantics
- At that checkpoint, the old function-list document remained the canonical expanded function-tree document.
- root `FUNCTION_TREE.md` was added as the compatibility entrypoint expected by external coordination, with a concise execution/bridge capability tree and a pointer back to the old function-list document.
- The old function-list document was updated at that checkpoint to document:
  - `BridgeRuntimeSettings` contract-loading fields
  - `request_diagnostics`
  - `qmt_live_gate`
  - `qmt_task_submit_service`
  - `qmt_live_adapter` receipt/result behavior
  - `/api/v1/task/execute` and `/api/v1/task/result/{task_id}` endpoints

## Verification

Fresh verification completed with:

```bash
cargo test --test repo_hygiene_test -- --nocapture
```

Observed result:

- `repo_hygiene_test`: 25 passed, 0 failed

## Graphiti

Attempted docs memory write:

- group: `quantix_rust_docs`
- episode uuid: `9d16735e-badc-4f17-aa76-644be50666f5`

Repeated ingest checks remained stuck in `processing`.

Graphiti backfill required
