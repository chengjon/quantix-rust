# Quantix → miniQMT Controlled Evidence Alignment Status Update

Date: 2026-05-18

## Document Role

This document is a response for the miniQMT side.

It summarizes the Quantix work that is already completed and relevant to the miniQMT controlled-evidence integration path, including the follow-up receive/apply result for the real `quantix_regression` evidence file.

This document is **not** the feature status registry. The authoritative source of feature status, evidence, and boundary state remains `FUNCTION_TREE.md`.

## Boundary Agreement

The current boundary is:

- miniQMT owns dataset publication, promotion evidence validation, promotion preview / apply, and registry ownership.
- Quantix is the upstream consumer, regression producer, and evidence producer.
- Quantix only consumes published manifest / artifact inputs.
- Quantix does not own miniQMT registry apply.
- Quantix does not implement production ClickHouse writes in this slice.

## Completed Quantix Work Relevant to miniQMT

### 1. Manifest and dataset identity intake

Quantix already understands the miniQMT market dataset manifest contract and carries the identity fields needed for evidence generation.

Implemented in:

- `src/miniqmt_market.rs`
- `tests/miniqmt_market_manifest_test.rs`

What is now available:

- explicit `dataset_version` intake
- manifest validation for published datasets
- identity fields such as `lineage_id`, `payload_hash`, `maturity`, `quality_status`
- optional `rows_hash` support
- resolved artifact identity export for downstream evidence

### 2. Artifact resolution with fail-closed verification

Quantix now supports dry-run artifact resolution against the miniQMT manifest and can verify local artifact content hashes when requested.

Implemented in:

- `src/miniqmt_market.rs`
- `src/cli/handlers/import.rs`
- `tests/miniqmt_market_manifest_test.rs`
- `tests/miniqmt_market_import_handler_test.rs`

What is now available:

- dry-run artifact resolution from a manifest
- opt-in local path / `file://` artifact SHA-256 verification
- fail-closed mismatch handling
- artifact identity propagation into the resolved result

### 3. Raw Quantix regression report generation

Quantix can now emit a raw regression report JSON for a miniQMT market artifact.

Implemented in:

- `src/miniqmt_market.rs`
- `src/cli/handlers/import.rs`

What is now available:

- regression report JSON generation
- explicit `source_command`
- run metadata
- sample identity fields for the artifact and dataset
- `database_target` recorded in the report flow
- `writes_performed=false` for the current dry-run path
- controlled persistence policy guard for `dry-run-only`, `clickhouse-shadow:<table>`, and `clickhouse-production:<table>`

### 4. miniQMT-shaped `quantix_regression` evidence JSON

Quantix can now generate a controlled evidence JSON shaped for the miniQMT evidence flow.

Implemented in:

- `src/miniqmt_market.rs`
- `src/cli/handlers/import.rs`

What is now available:

- evidence JSON generation from the raw regression report
- `schema_version = "evidence.v1"`
- `environment.consumer_system = "quantix-rust"`
- `result_summary.evidence_type = "promotion_consumer_regression"`
- `result_summary.dataset_version`
- `result_summary.lineage_id`
- `result_summary.payload_hash`
- raw report reference and hash information
- `database_target` and `writes_performed` recorded explicitly
- one real evidence instance for `kline_daily_20260518_v1` has been generated and accepted by miniQMT's controlled-evidence flow

### 5. Controlled persistence policy guard

Quantix now has a local policy guard for database target semantics before any ClickHouse write path exists.

Implemented in:

- `src/miniqmt_market.rs`
- `tests/miniqmt_market_manifest_test.rs`

What is now available:

- `dry-run-only` passes only when `writes_performed=false`
- `clickhouse-shadow:<table>` requires a future explicit write path before it can produce a passed report
- `clickhouse-production:<table>` fails closed because production writes are not implemented in this slice
- unsupported database targets fail closed

### 6. CLI surface for the dry-run evidence path

Quantix has a CLI entry point for the miniQMT manifest workflow.

Implemented in:

- `src/cli/commands/info.rs`
- `src/cli/handlers/import.rs`
- `src/cli/handlers/mod.rs`
- `src/cli/tests/import.rs`

Current command shape:

```text
quantix import market-manifest \
  --manifest <manifest.json> \
  --dataset-version <dataset_version> \
  --artifact-type <artifact_type> \
  [--schema-version <schema_version>] \
  [--artifact-hash <artifact_hash>] \
  [--verify-artifact-file] \
  [--regression-report-output <report.json>] \
  [--evidence-output <evidence.evidence.json>] \
  [--consumer-build-commit <commit>] \
  [--database-target dry-run-only]
```

Current behavior:

- resolve the target artifact from the published manifest
- optionally verify the local artifact file hash
- write a raw regression report JSON when requested
- write a miniQMT-shaped evidence JSON when requested
- stay dry-run-only for the current slice and fail closed for unsupported persistence semantics

## Verification Completed

The following gates passed after the implementation:

- `cargo fmt --check`
- `cargo test --test miniqmt_market_manifest_test`
- `cargo test --test miniqmt_market_import_handler_test`
- `cargo test --test repo_hygiene_test`
- `cargo test --all-targets --quiet`
- `cargo clippy --all-targets --quiet`

## miniQMT Receive / Apply Result

Quantix generated real `quantix_regression` evidence for the miniQMT dataset below:

- `dataset_version`: `kline_daily_20260518_v1`
- `lineage_id`: `lin_kline_daily_20260518_v1`
- `payload_hash`: `268b62bb0fb0891833ef1998d4993d6531cc6a9d84aaecb911da0cd559d2357e`
- `artifact_hash`: `6166deee3de84798e11703b8b5616aa2dd772c9460225e81c86e66323f5a6706`
- `database_target`: `dry-run-only`
- `writes_performed`: `false`

Quantix-side outputs:

- raw report: `docs/reports/evidence/miniqmt/quantix_regression_kline_daily_20260518_v1.json`
- evidence JSON: `docs/reports/evidence/miniqmt/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json`
- receive result: `docs/reports/MINIQMT_QUANTIX_REGRESSION_EVIDENCE_RECEIVE_RESULT_2026-05-18.md`

miniQMT-side received outputs:

- evidence JSON: `DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json`
- receive result: `DOCS/xtdata-api/2026-05-18-quantix-regression-evidence-receive-result.md`
- post-Quantix remaining gates runbook: `DOCS/xtdata-api/2026-05-19-post-quantix-remaining-gates-runbook.md`

miniQMT-side result:

- local evidence validator: passed
- server plan-only preview: passed
- server apply: passed
- `quantix_regression` is recorded as passed promotion evidence
- `promotion-gaps` no longer reports `quantix_regression`

The receive-side evidence file may contain miniQMT-side metadata after apply, so it is no longer expected to remain byte-identical to the original Quantix output. The original Quantix evidence remains the producer-side artifact.

## What Is Still Not Done

These items are intentionally not completed in the current slice:

- real double-read / comparison summary against the next source of truth
- actual ClickHouse shadow-table import write path
- production ClickHouse writes
- miniQMT registry apply from Quantix
- miniQMT validator / preview / apply wrapper inside Quantix
- MyStocks validated forward `mystocks_dry_run` evidence completion, which is not owned by Quantix and has since been completed
- manual promote to `validated`, which is a miniQMT owner/operator gate and has since been completed
- manual promote to `authoritative-ready`, which is a miniQMT owner/operator gate and has since been completed
- authoritative approval / rollback readiness, which is a miniQMT owner/operator gate

The Quantix-owned `quantix_regression` promotion gap is closed for this dataset. miniQMT's post-Quantix runbook now records that MyStocks validated-forward `mystocks_dry_run`, manual promote to `validated`, and manual promote to `authoritative-ready` are also closed. The remaining `authoritative` approval / rollback readiness gaps should stay separate from the already-closed Quantix evidence slice.

## Recommended Next Phase

The next Quantix-owned development phase should focus on:

1. double-read / comparison checks
2. ClickHouse shadow import write path behind the existing controlled persistence policy
3. optional miniQMT evidence validation / preview wrapper for operator convenience, without moving registry ownership into Quantix
4. promotion gating only after miniQMT and the other upstream evidence producers complete their own gates

The next non-Quantix gates are:

1. miniQMT-owned authoritative approval.
2. miniQMT-owned rollback / fallback readiness before final `authoritative` status.

## Files To Review For the Current State

- `FUNCTION_TREE.md`
- `README.md`
- `src/miniqmt_market.rs`
- `src/cli/commands/info.rs`
- `src/cli/handlers/import.rs`
- `tests/miniqmt_market_manifest_test.rs`
- `tests/miniqmt_market_import_handler_test.rs`
- `docs/superpowers/specs/2026-05-18-miniqmt-controlled-evidence-alignment-spec.md`
- `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md`
- `docs/reports/MINIQMT_QUANTIX_REGRESSION_EVIDENCE_RECEIVE_RESULT_2026-05-18.md`
- miniQMT `DOCS/xtdata-api/2026-05-19-post-quantix-remaining-gates-runbook.md`
- miniQMT `DOCS/xtdata-api/2026-05-20-authoritative-ready-promote-result.md`

## Short Closing Statement

Quantix has completed the first dry-run controlled-evidence slice for the miniQMT integration path, generated a real `quantix_regression` evidence file for `kline_daily_20260518_v1`, and had that evidence accepted by miniQMT validator / preview / apply.

The current state is ready for the next phase: Quantix can proceed on double-read and controlled persistence, while miniQMT handles the remaining `authoritative` approval / rollback readiness gates.
