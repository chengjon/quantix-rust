# miniQMT Quantix Regression Evidence Receive Result

## Document Role

This document records the concrete receive result for the Quantix `quantix_regression` controlled evidence flow.
It is a handoff record, not the feature registry. The authoritative capability/status source remains `FUNCTION_TREE.md`.

## Inputs

- miniQMT handoff runbook: `DOCS/xtdata-api/2026-05-18-quantix-regression-evidence-handoff-runbook.md`
- Quantix evidence output:
  - raw report: `docs/reports/evidence/miniqmt/quantix_regression_kline_daily_20260518_v1.json`
  - evidence JSON: `docs/reports/evidence/miniqmt/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json`
- miniQMT receive path:
  - `DOCS/xtdata-api/evidence/2026-05-18-kline_daily_20260518_v1-quantix-regression.evidence.json`

## Quantix Output

Quantix generated a real `quantix_regression` evidence file from the validated forward market-data manifest/artifact.

Key fields in the generated evidence:

- `schema_version = evidence.v1`
- `source_command` captures the exact `quantix import market-manifest` invocation
- `environment.consumer_system = quantix-rust`
- `result_summary.evidence_type = promotion_consumer_regression`
- `result_summary.dataset_version = kline_daily_20260518_v1`
- `result_summary.lineage_id = lin_kline_daily_20260518_v1`
- `result_summary.payload_hash = 268b62bb0fb0891833ef1998d4993d6531cc6a9d84aaecb911da0cd559d2357e`
- `result_summary.rows_hash = d4fccc8a83d6144b1d8da1e818726db339c25026f5933cb056eef3cb6badda4f`
- `result_summary.row_count = 4`
- `result_summary.artifact.hash = 6166deee3de84798e11703b8b5616aa2dd772c9460225e81c86e66323f5a6706`
- `result_summary.artifact.computed_hash = 6166deee3de84798e11703b8b5616aa2dd772c9460225e81c86e66323f5a6706`
- `result_summary.consumer_build.commit = f242316473c31d1f726adb3efba1da927c639171`
- `result_summary.consumer_build.database_target = dry-run-only`
- `result_summary.consumer_build.writes_performed = false`

## miniQMT Receive Result

The evidence was copied into the miniQMT receive directory and the initial copied file was verified byte-for-byte before receive/apply processing.
After miniQMT apply, the receive-side evidence file carries miniQMT-side metadata such as `related_function_tree_node`; the registry-accepted copy is therefore not expected to remain byte-identical to the original Quantix output.

Verification results:

- local validator: passed
- plan-only preview: passed
- apply: passed
- promotion evidence record: `quantix_regression` recorded as passed

miniQMT-side observed result after apply:

- `quantix_regression` is backed by controlled evidence and recorded
- the miniQMT receive-side evidence file includes `related_function_tree_node = FUNCTION_TREE.md#8.1 P0: Market Data M1 authoritative-ready gap / Quantix Rust manifest-client evidence`
- `promotion-gaps` no longer reports `quantix_regression`
- current maturity for the dataset remains `candidate`
- this does not yet make the dataset authoritative

## Remaining Gaps

The current `promotion-gaps` result still shows:

- `mystocks_dry_run` missing or not passed
- authoritative approval / rollback readiness missing

So the Quantix evidence path is now accepted, but the promotion flow is not fully authoritative yet.

## Commands Used

Quantix evidence generation:

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

miniQMT validator:

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

miniQMT preview/apply:

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

## Short Closing Statement

Quantix has produced a real `quantix_regression` evidence file, miniQMT accepted it through validator / preview / apply, and the `quantix_regression` gap is no longer present.
The next independent work item is the remaining `mystocks_dry_run` evidence and authoritative rollback/approval readiness.
