# qmt_live Operational Safety P0.5 Design

## Context

Current qmt_live hardening has already established:

- static execution capability declarations;
- qmt_live-local bridge capability snapshot and compatibility descriptor;
- structured qmt_live gate diagnostics;
- qmt_live error taxonomy seed;
- qmt_live identity metadata recovery;
- reconciliation query refinement using complete local identity when available;
- a repository-level safety kill switch that blocks `mock_live` and `qmt_live` submissions while leaving `paper` available.

P0.5 should turn those primitives into a real-money operator workflow. The design principle is: qmt_live may be used only after an explicit preflight check, operator canary evidence, active kill-switch acceptance, auditable qmt_live action records, and manual-intervention visibility.

## Command Surface Strategy

The implementation SHOULD reuse the existing execution and safety command families before adding new namespaces:

- `quantix execution qmt status --checklist`
- `quantix execution qmt preview --request-id <ID>`
- `quantix execution qmt live --request-id <ID> [--yes]`
- `quantix execution qmt query <ID>`
- `quantix execution qmt cancel <ID>`
- `quantix safety kill-switch enable --reason <TEXT>`
- `quantix safety kill-switch disable`
- `quantix safety kill-switch status`

P0.5a MAY add a dedicated doctor subcommand only if GitNexus impact shows the existing `execution qmt status --checklist` path is a worse owner. If a doctor command is added, the preferred shape is:

```bash
quantix execution qmt doctor
```

The doctor MUST be read-only and MUST NOT submit, cancel, query broker orders by mutable endpoint, or mutate runtime store state.

## Slice Order

### P0.5a: Preflight Doctor

P0.5a owns the operator-facing readiness verdict. It should produce a deterministic summary from existing bridge capability and execution capability data.

Readiness checks:

- Windows Bridge base URL is configured.
- Windows Bridge capabilities endpoint is reachable.
- qmt capability section is present.
- `qmt.enabled == true`.
- `qmt.mode == "live"`.
- `qmt.supports` contains `order_submit`.
- qmt_live adapter capabilities identify broker status/fill/cancel sources.
- kill switch state is visible in the output.

Failure categories:

- `bridge_unreachable`
- `qmt_capability_missing`
- `qmt_disabled`
- `qmt_mode_not_live`
- `qmt_order_submit_missing`
- `qmt_live_capability_mismatch`
- `kill_switch_enabled`

### P0.5b: Canary Runbook And Evidence

P0.5b owns the human procedure for the first real-money path. It should add an operations runbook and an evidence artifact format.

Evidence fields:

- git commit hash;
- command lines;
- bridge base URL host label without secrets;
- qmt readiness summary;
- kill switch status before canary;
- request ID;
- preview payload hash;
- operator confirmation timestamp;
- live submission result summary;
- query result summary;
- reconciliation result summary;
- manual-intervention status;
- redacted notes.

### P0.5c: Kill Switch Acceptance

P0.5c owns proof that the existing kill switch is sufficient for qmt_live operations.

Required behavior:

- enabled kill switch blocks `qmt_live` approval and submission;
- enabled kill switch blocks `mock_live`;
- enabled kill switch does not block `paper`;
- enabled kill switch does not block `execution qmt status --checklist`, doctor/preflight, preview, or read-only query;
- block payload includes reason, enabled timestamp, blocked timestamp, and target mode.

If any of these behaviors already exists, the slice should preserve it and add only missing acceptance coverage or documentation.

### P0.5d: Audit Evidence

P0.5d owns enough qmt_live action evidence for post-incident review without committing raw broker secrets.

Audit evidence should be append-only or reproducible from existing runtime records where possible. The implementation should prefer existing runtime store payloads and reports over a new storage schema.

Minimum fields:

- request ID;
- target account alias or redacted account label;
- symbol;
- side;
- quantity;
- order type or price intent;
- local submission ID;
- client order ID;
- task ID;
- external order ID when present;
- bridge contract version;
- qmt_live error category when present;
- reconciliation decision;
- manual-intervention requirement when present.

### P0.5e: Manual-Intervention Report

P0.5e owns operator discovery of unresolved qmt_live ambiguity.

The first implementation SHOULD be read-only. It should list and show cases where qmt_live state indicates:

- identity mismatch;
- broker unknown state;
- missing external order ID after bridge task completion;
- reconciliation preserved local state because broker identity was ambiguous;
- bridge failure requiring operator review.

The report should include suggested human actions, but it MUST NOT mark cases resolved until a separately approved state-write workflow exists.

## Dependency And Risk Decisions

- qmt_live remains miniQMT/broker-state governed.
- Local Quantix state is evidence and workflow state, not broker truth.
- Existing safety kill switch is a system-level safety primitive; P0.5 should verify it instead of replacing it.
- Canary evidence is an operations artifact, not Graphiti memory.
- Manual-intervention reporting should start read-only to avoid accidental state rewrites.

## Non-Goals

- No bridge protocol changes.
- No response shape changes.
- No storage schema migrations in P0.5a or P0.5b.
- No global `ExecutionAdapter` trait migration.
- No `OrderStatus` changes.
- No simulated matching engine.
- No automatic background reconciliation daemon.
- No generic broker abstraction.
