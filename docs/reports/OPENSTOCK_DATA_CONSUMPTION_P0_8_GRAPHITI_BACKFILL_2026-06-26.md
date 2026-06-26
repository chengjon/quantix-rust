# OpenStock Data Consumption P0.8 Graphiti Backfill

Date: 2026-06-26

Status: local Graphiti backfill record required

Branch: `docs/openstock-p0-8-graphiti-backfill`

Base commit: `821e72302a3df0cfa5dd6e113d618b248a48777d`

Related PR: `#297`

Related master CI: `28210895113`

## Summary

Graphiti backfill required

After P0.8 was merged and master CI passed, the required Graphiti closeout memory was queued but could not be verified as completed.

Episode:

```text
fb126253-d46e-41eb-98fd-924083015af3
```

Group:

```text
quantix_rust_main
```

Observed ingest state after repeated polling:

```text
state=processing
queue_depth=0
attempt_count=1
processed_at=null
last_error=null
last_error_code=null
queued_at=2026-06-26T01:21:13.915867+00:00
started_at=2026-06-26T01:21:13.943395+00:00
```

Graphiti search could still find existing OpenStock/OpenSpec nodes, but `get_ingest_status` for this closeout episode did not reach `completed`. Because ingest completion could not be verified, this report records the equivalent durable memory locally for later Graphiti backfill.

## Equivalent Memory Summary

P0.8 OpenStock data consumption OpenSpec closed on 2026-06-26.

PR #297 merged to master as:

```text
821e72302a3df0cfa5dd6e113d618b248a48777d
```

P0.8 added:

- `openspec/changes/openstock-data-consumption-p0-8/proposal.md`;
- `openspec/changes/openstock-data-consumption-p0-8/design.md`;
- `openspec/changes/openstock-data-consumption-p0-8/tasks.md`;
- `openspec/changes/openstock-data-consumption-p0-8/specs/openstock-data-consumption/spec.md`;
- `docs/reports/OPENSTOCK_DATA_CONSUMPTION_P0_8_OPENSPEC_2026-06-26.md`;
- `.governance/programs/project-governance/cards/P0.8.yaml`;
- README, CHANGELOG, FUNCTION_TREE, and governance status updates.

The slice formalized OpenStock data consumption as a broker-independent data line after P0.6 qmt_live runtime readiness was archived as `blocked_by_environment` and P0.7 ExecutionCapabilities documentation sync was closed.

Planned P0.8 implementation order:

- P0.8a: inventory current data models, kline/quote abstractions, storage tables, CLI consumers, OpenStock artifact shapes, and schema-gated persistence candidates.
- P0.8b: define a provider contract and fixture-owned parser tests, with no live network calls in CI.
- P0.8c: add a read-only CLI/status surface for config and local fixture validation.
- P0.8d: connect parsed fixture or local artifact data into the analysis/backtest path.
- P0.8e: consider persistence or ClickHouse shadow validation only after schema and rollback gates.

## Preserved Boundaries

P0.8 did not change:

- production Rust code;
- live OpenStock network behavior;
- CI live-network policy;
- ClickHouse persistence;
- qmt_live runtime readiness;
- qmt_live canary, submit, query, or cancel behavior;
- bridge protocol;
- runtime storage;
- `ExecutionAdapter`;
- `OrderStatus`;
- miniQMT market-manifest behavior;
- tdx-api behavior;
- `.unwrap()` cleanup scope.

## Verification

P0.8 verification completed before and after merge:

- `openspec validate openstock-data-consumption-p0-8 --strict`;
- `openspec validate --all --strict`;
- `git diff --check`;
- FUNCTION_TREE `scope-check`, `gate`, and `validate`;
- GitNexus `detect_changes` compare against master: LOW risk, `0` affected processes, documentation/governance/config-only file classes;
- PR #297 Lint and Test passed;
- master CI run `28210895113` passed Lint, Test, and Documentation.

## Next Step

Start P0.8a as a separate code-aware inventory slice before any parser, provider, CLI, persistence, or data-source routing implementation.
