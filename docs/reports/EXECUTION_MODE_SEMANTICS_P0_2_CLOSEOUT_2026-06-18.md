# Execution Mode Semantics P0.2 Closeout

Date: 2026-06-18

Status: P0.2 low-intrusion semantics hardening closed

FUNCTION_TREE node: `Execution mode semantics hardening`

Governance nodes:

- `project-governance/P0.2`
- `project-governance/P0.2b`
- `project-governance/P0.2c`
- `project-governance/P0.2d`
- `project-governance/P0.2e`
- `project-governance/P0.2f`

## Closure Decision

The P0.2 low-intrusion execution-mode semantics hardening stage is closed.

This stage established and locked the current semantic baseline for three
execution channels:

| Channel | Current role | Fill source | Status source | Implementation state |
|---|---|---|---|---|
| `paper_immediate` | Existing local paper path | Quantix local ledger immediate fill | Local `TradeRecord` | Available through the existing paper adapter semantics |
| `paper_sim_lifecycle` | Future isolated simulator | Future local simulated matcher | Future local simulated order lifecycle | Not implemented |
| `qmt_live` | Guarded real-money path | miniQMT / broker / exchange path | miniQMT / broker query result | Available only through guarded qmt live / bridge paths |

The stage intentionally did not implement cross-cutting runtime redesigns.
Future changes to adapter capabilities, order state response shapes, broker
identity reconciliation, or simulated lifecycle storage must be separate
projects with fresh impact analysis and tailored tests.

## Completed Slices

| Slice | Main PR / commit | Result |
|---|---|---|
| P0.2a proposal and registry alignment | PR #232, `3658484` | Added the semantics proposal and FUNCTION_TREE baseline. |
| P0.2b paper immediate-fill behavior lock | PR #234, `eda2ef2` | Added `IMMEDIATE_FILL_ONLY`, tests, and hygiene guard to keep paper submit/query as immediate filled local ledger semantics. |
| P0.2c channel risk notices | PR #235, `2c1f4ea` | Added standard risk notice catalog and submit-time logging for paper/qmt live paths. |
| P0.2d storage namespace helper | PR #236, `90e6336` | Added stable storage namespace constants and helper for the three semantic channels. |
| P0.2e configured-mode namespace binding | PR #240, `4d82661` | Added read-only binding from configured execution mode values to semantic channels and namespaces. |

Related GitNexus metadata refresh PRs:

| PR / commit | Purpose |
|---|---|
| PR #233, `91e6b7c` | Metadata refresh after P0.2a. |
| PR #237, `e663e2e` | Metadata refresh after P0.2d. |
| PR #238, `0b1b025` | Metadata stabilization after P0.2d. |
| PR #239, `8845d8f` | Force-analyze metadata refresh after P0.2d. |

## Current Code-Level Baseline

`src/execution/mode_semantics.rs` now owns the low-intrusion semantic catalog:

- channel constants:
  - `paper_immediate`,
  - `paper_sim_lifecycle`,
  - `qmt_live`;
- risk notice strings for the three channels;
- stable path-segment-safe storage namespaces;
- `storage_namespace_for_channel`;
- `storage_binding_for_configured_execution_mode`;
- `log_execution_mode_risk_notice`.

`paper` maps to `paper_immediate` for configured-mode namespace binding.
`qmt_live` maps to `qmt_live`. `paper_sim_lifecycle` is cataloged for the
future isolated simulator but remains unavailable. `live`, `mock_live`, and
unknown values are intentionally not folded into this three-channel binding.

`runtime_switching_allowed` is represented as false metadata only. It does not
change runtime configuration loading or execution behavior.

## Explicit Non-Changes

P0.2 did not change:

- `ExecutionAdapter` trait contracts,
- `OrderStatus` enum variants or response shapes,
- `CliRuntime::load()` default paths,
- execution daemon request consumption behavior,
- qmt live gate / bridge runtime behavior,
- miniQMT version or field compatibility gates,
- task id to external order id reconciliation,
- paper account ledger formulas,
- paper simulated lifecycle storage,
- matcher logic,
- `.unwrap()` cleanup state.

The `.unwrap()` cleanup line remains closed. No new `.unwrap()` cleanup work is
authorized by this stage.

## Verification Evidence

Local verification run during the P0.2e closeout included:

- TDD red/green for `storage_binding_for_configured_execution_mode`,
- `cargo test --test execution_mode_semantics_test`,
- `cargo fmt --check`,
- `cargo clippy --all-targets --all-features -- -D warnings`,
- `cargo test --all-targets`,
- `git diff --check`,
- FUNCTION_TREE validation,
- FUNCTION_TREE status with active gates at 0,
- GitNexus impact and detect-changes with LOW or clean/no process impact.

Remote verification:

- PR #240 CI passed for Lint and Test.
- Master CI run `27711568718` passed Documentation, Lint, and Test.
- Build, Coverage, and Benchmark were skipped by workflow rules.

Post-merge local state:

- current master: `4d82661 feat: bind execution config modes to namespaces (#240)`,
- worktree clean,
- FUNCTION_TREE active gates: 0.

## Known Tooling Issue

GitNexus analysis and generated metadata showed small nondeterministic count
oscillation while refreshing AGENTS/CLAUDE metadata:

- after P0.2d: `16437` / `16438` symbol count alternation,
- after P0.2e: `16447` / `16448` symbol count alternation.

This is treated as GitNexus generated metadata/index determinism, not product
behavior. Future agents should not open repeated metadata-only PRs to chase the
oscillation unless the GitNexus determinism issue is first understood.

Graphiti memory writes were attempted for P0.2d and P0.2e closeout, but ingest
remained in `processing` with queue depth 0 and no error. Backfill is required
when Graphiti ingest is healthy.

Graphiti backfill required

## Remaining Architecture Work

The following work is deferred and requires separate authorization:

1. Execution capability abstraction.
   - Add an `ExecutionCapabilities` descriptor or equivalent.
   - Migrate upper layers away from hard-coded mode checks gradually.
   - Requires fresh GitNexus impact and focused tests.

2. Status source layering.
   - Extend order query/state responses with status source metadata.
   - Keep local immediate fill, local simulation, and broker state visibly
     distinct.
   - This may affect serialization, CLI display, and frontend consumers.

3. qmt live capability and identity hardening.
   - Add miniQMT / bridge version and schema compatibility gates.
   - Improve `task_id` to external order id reconciliation.
   - Split local validation, bridge failure, broker rejection, and unknown
     broker state errors.

4. `paper_sim_lifecycle` isolated simulator.
   - Use separate module, storage, tests, and matcher abstraction.
   - Do not extend `paper_immediate` in place.
   - Add explicit freeze/release and order expiry semantics only inside that
     isolated project.

## Next Recommended Step

Do not continue adding behavior under P0.2. The next executable work should be a
new, separately authorized project. The recommended first candidate is qmt live
capability and identity hardening, because it protects the real-money path and
can remain isolated from paper simulator work.
