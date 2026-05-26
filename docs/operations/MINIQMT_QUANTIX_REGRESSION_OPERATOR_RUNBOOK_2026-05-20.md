# miniQMT Quantix Regression Operator Runbook

Date: 2026-05-20

## Role

This runbook describes how to generate and hand off Quantix `quantix_regression` controlled evidence for a miniQMT market dataset.

This document is not the feature registry. Quantix feature status, evidence, and boundaries remain registered in `FUNCTION_TREE.md`.

## Ownership Boundary

Quantix owns:

- Reading only miniQMT published release manifest / artifact identity.
- Verifying explicit `dataset_version`, `lineage_id`, `payload_hash`, and artifact hash.
- Generating a dry-run Quantix regression report.
- Generating a miniQMT-shaped `quantix_regression` evidence JSON.
- Recording `database_target` and `writes_performed`.

miniQMT owns:

- Dataset publication.
- Promotion requirements / gaps.
- Evidence validator.
- Promotion evidence preview / apply.
- Registry writes.
- Manual maturity promotion.
- Final `authoritative` approval and rollback / fallback readiness.

Quantix must not:

- Read miniQMT raw / candidate / job intermediate files as a replacement for release artifacts.
- Use implicit `latest`.
- Write miniQMT registry state.
- Treat HTTP 200, job completion, export completion, or Quantix evidence generation as promotion readiness.
- Write production ClickHouse tables from this path.

## Current Proven Dataset

The following dataset identity has already completed the Quantix evidence path and was accepted by miniQMT:

| Field | Value |
|---|---|
| `dataset_version` | `kline_daily_20260518_v1` |
| `lineage_id` | `lin_kline_daily_20260518_v1` |
| `payload_hash` | `268b62bb0fb0891833ef1998d4993d6531cc6a9d84aaecb911da0cd559d2357e` |
| artifact hash | `6166deee3de84798e11703b8b5616aa2dd772c9460225e81c86e66323f5a6706` |
| evidence key | `quantix_regression` |
| database target | `dry-run-only` |
| writes performed | `false` |

As of 2026-05-20, miniQMT has also closed MyStocks validated-forward evidence and manually promoted this dataset to `authoritative-ready`. It is not final `authoritative`; that still requires miniQMT owner/operator approval and rollback / fallback readiness.

## Inputs Required From miniQMT

The operator must provide explicit values:

- Published release `manifest.json` path.
- Published artifact type, for example `parquet`.
- Expected `dataset_version`.
- Expected `lineage_id`.
- Expected `payload_hash`.
- Expected artifact hash.
- Quantix build commit.
- Output path for the raw Quantix report.
- Output path for the `quantix_regression` evidence JSON.

The manifest and artifact must come from miniQMT release / manifest / artifact outputs. Do not use raw, candidate, or job intermediate paths.

## Step 1: Generate Quantix Evidence

Run from the Quantix repository:

```bash
cargo run --manifest-path /opt/claude/quantix-rust/Cargo.toml --bin quantix -- \
  import market-manifest \
  --manifest /mnt/d/MyCode3/miniQMT/bridge/logs/uat_market_data_validation_forward_logfix_release/domain=kline_daily/dataset_version=kline_daily_20260518_v1/manifest.json \
  --dataset-version kline_daily_20260518_v1 \
  --artifact-type parquet \
  --schema-version v1 \
  --artifact-hash 6166deee3de84798e11703b8b5616aa2dd772c9460225e81c86e66323f5a6706 \
  --verify-artifact-file \
  --regression-report-output docs/reports/evidence/miniqmt/quantix_regression_kline_daily_20260518_v1.json \
  --evidence-output docs/reports/evidence/miniqmt/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json \
  --consumer-build-commit f242316473c31d1f726adb3efba1da927c639171 \
  --database-target dry-run-only
```

Expected behavior:

- The command prints the resolved artifact summary.
- The raw report file is created.
- The evidence JSON file is created.
- Artifact hash verification passes before evidence can be marked passed.
- Local Parquet artifacts populate `sample_symbols` and `sample_dates` when the payload contains recognizable symbol/date columns.
- `database_target` remains `dry-run-only`.
- `writes_performed` remains `false`.

Failure behavior:

- Dataset version mismatch fails.
- Artifact hash mismatch fails.
- Unsupported artifact URI for file verification fails.
- `clickhouse-shadow:<table>` without a real write path fails closed.
- `clickhouse-production:<table>` fails closed.

## Step 2: Copy Evidence To miniQMT Evidence Directory

Copy the evidence JSON into miniQMT's accepted evidence directory:

```bash
cp \
  /opt/claude/quantix-rust/docs/reports/evidence/miniqmt/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json \
  /mnt/d/MyCode3/miniQMT/DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json
```

The miniQMT validator expects evidence under `DOCS/**/evidence/*.evidence.json`.

## Step 3: Run miniQMT Local Validator

Run from the miniQMT repository:

```bash
python bridge/scripts/validate_market_data_promotion_evidence.py \
  --json \
  --emit-request \
  --evidence-key quantix_regression \
  --dataset-version kline_daily_20260518_v1 \
  --lineage-id lin_kline_daily_20260518_v1 \
  --payload-hash 268b62bb0fb0891833ef1998d4993d6531cc6a9d84aaecb911da0cd559d2357e \
  DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json
```

Pass criteria:

- `passed=true`
- `errors=[]`
- Evidence key is `quantix_regression`.
- Dataset identity matches the miniQMT release identity.
- `regression.passed=true`
- `regression.failed_checks=[]`

## Step 4: Run miniQMT Server Preview

Start the miniQMT API using the registry / release directories for the intended environment, then run plan-only apply:

```bash
python bridge/scripts/apply_market_data_promotion_evidence.py \
  --base-url http://127.0.0.1:18080 \
  --api-key change-me-in-production \
  --dataset-version kline_daily_20260518_v1 \
  --evidence-key quantix_regression \
  --evidence-path DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json \
  --recorded-by quantix-rust \
  --notes "Quantix controlled regression evidence preview" \
  --plan-only
```

Preview is mandatory before apply.

## Step 5: Apply miniQMT Promotion Evidence

Only after preview is accepted, run apply without `--plan-only`:

```bash
python bridge/scripts/apply_market_data_promotion_evidence.py \
  --base-url http://127.0.0.1:18080 \
  --api-key change-me-in-production \
  --dataset-version kline_daily_20260518_v1 \
  --evidence-key quantix_regression \
  --evidence-path DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json \
  --recorded-by quantix-rust \
  --notes "Quantix controlled regression evidence accepted by miniQMT operator"
```

miniQMT remains the registry owner. Quantix does not own this apply step.

## Step 6: Recheck Gaps

After apply, miniQMT should show:

```text
promotion_evidence_gaps does not include quantix_regression
```

For the current proven dataset, miniQMT also records:

```text
current_maturity=authoritative-ready
effective_maturity=authoritative-ready
evaluated_maturity=authoritative-ready
promotion_ready.authoritative=false
```

The remaining gates are final `authoritative` approval and rollback / fallback readiness.

## Failure Handling

When any gate fails:

1. Do not manually edit evidence fields to force validation.
2. Fix the upstream source of mismatch, then regenerate the Quantix evidence.
3. Keep `dataset_version`, `lineage_id`, `payload_hash`, and artifact hash explicit in every rerun.
4. Do not switch to production writes to bypass a dry-run limitation.
5. Record the failed command and result in the relevant evidence / report directory.

## Future Quantix Work

These are not implemented by this runbook:

- Real double-read comparison against a Quantix source of truth.
- ClickHouse shadow-table import.
- A Quantix wrapper that drives miniQMT validator / preview.
- Production ClickHouse writes.

Those items must remain reflected in `FUNCTION_TREE.md` until implemented and verified.
