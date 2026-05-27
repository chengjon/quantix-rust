# miniQMT Controlled Evidence Alignment Implementation Plan

## Document Role

本文保存 2026-05-18 阅读 miniQMT 上游 controlled evidence 对接指南后的 Quantix 侧发现、边界判断和实施切片，用于下一步实现。

本文不是功能状态注册表，不替代 `FUNCTION_TREE.md`。功能是否已实现、部分实现或仅设计，必须回到 `FUNCTION_TREE.md` 的状态注册表确认。

## Implementation Snapshot

This section is a handoff snapshot, not a feature registry. The authoritative feature status remains `FUNCTION_TREE.md`.

Current local implementation has advanced the first dry-run evidence slice:

- `FUNCTION_TREE.md` registers miniQMT market dataset consumer / controlled evidence nodes with explicit status, evidence, and boundary.
- `MarketDatasetManifest`, `MarketDatasetArtifact`, and `ResolvedMarketArtifact` carry the identity fields needed by Quantix evidence, including optional `rows_hash`.
- Local artifact SHA-256 verification exists for path / `file://` artifact URIs and fails closed on mismatch.
- `quantix import market-manifest` can emit a raw Quantix regression report JSON and a miniQMT-shaped `quantix_regression` evidence JSON.
- Local Parquet artifact payload sampling can populate sample symbols / dates in the raw report and evidence when `--verify-artifact-file` resolves a readable local artifact.
- Local Parquet metadata row-count verification records `computed_row_count` and fails closed when it differs from the manifest artifact `row_count`.
- Opt-in local reference artifact comparison can populate the regression comparison block when `--comparison-reference-artifact` points at a readable local artifact.
- The emitted evidence remains dry-run-only: `database_target` is explicit, `writes_performed=false`, and a controlled persistence policy guard fails closed for shadow targets without a write path or any production target.
- The operator runbook exists at `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md` and keeps miniQMT as validator / preview / apply / registry owner.

Remaining implementation work:

- Add real double-read / comparison checks against the intended Quantix source of truth.
- Add the actual ClickHouse shadow import write path after the controlled persistence policy guard.
- Add an optional Quantix convenience wrapper for miniQMT validator / preview after the manual runbook is proven useful.

## Source Material

- miniQMT 指南：`/mnt/d/MyCode3/miniQMT/DOCS/xtdata-api/2026-05-18-upstream-controlled-evidence-integration-guide.md`
- Quantix manifest 合同模块：`src/miniqmt_market.rs`
- Quantix CLI dry-run 入口：`src/cli/commands/info.rs`
- Quantix import handler：`src/cli/handlers/import.rs`
- Quantix 测试证据：`tests/miniqmt_market_manifest_test.rs`, `tests/miniqmt_market_import_handler_test.rs`
- Quantix 唯一功能状态注册表：`FUNCTION_TREE.md`

## Boundary Alignment

### miniQMT Owns

miniQMT 是 market dataset 发布和 promotion gate 的权威系统，负责：

- 发布不可变 dataset version。
- 生成并维护 manifest、artifact、lineage 和 payload identity。
- 定义 `dataset_version`, `lineage_id`, `payload_hash`, `rows_hash`, `artifact_hash`, `schema_version`, `quality_status`, `maturity`。
- 提供 evidence slot、template scaffold、local validator、promotion preview、promotion evidence apply。
- 维护 miniQMT registry 和 maturity promote gate。

### Quantix Owns

Quantix 是上游 consumer / regression / evidence producer，负责：

- 只读取明确 `dataset_version` 绑定的 release manifest / artifact。
- 校验 manifest 身份字段和 quality gate。
- 重算 artifact 文件 hash，必要时校验 `rows_hash`。
- 运行 Quantix 侧 regression / double-read / import-control。
- 记录 Quantix commit、运行命令、环境、ClickHouse target、是否实际写入。
- 生成 `quantix_regression` controlled evidence JSON。

### Explicit Non-Goals

Quantix 当前不应承担以下职责：

- 不读取 miniQMT raw / candidate / job intermediate 文件。
- 不使用隐式 `latest` dataset。
- 不把 HTTP 200、job completed、export completed 视为可切主源。
- 不拥有 miniQMT registry 写入和 maturity promote 决策。
- 不要求 miniQMT 写 Quantix / MyStocks 数据库。
- 不在第一阶段直接写 Quantix 生产 ClickHouse 表。

## Current Quantix Capability

### Implemented Or Partially Implemented

`src/miniqmt_market.rs` 已具备 consumer-side manifest 合同能力：

- `MarketDatasetManifest`
- `ManifestSource`
- `MarketDatasetArtifact`
- `ManifestQuality`
- `ManifestValidator`
- `ManifestIntake`
- `MarketArtifactSelector`
- `ResolvedMarketArtifact`
- `MarketArtifactRequest`
- `load_manifest_from_slice`
- `load_manifest_from_path`
- `resolve_artifact_from_slice`
- `resolve_artifact_from_path`

当前已解析的 manifest 字段：

- `dataset_version`
- `schema_version`
- `contract_profile`
- `domain`
- `maturity`
- `quality_status`
- `published`
- `lineage_id`
- `row_count`
- `payload_hash`
- `sources`
- `artifacts`
- `quality`

当前 artifact 字段：

- `type` / `artifact_type`
- `uri`
- `schema_version`
- `row_count`
- `hash`

当前 validator / selector 已覆盖：

- dataset version match。
- published dataset。
- non-empty `lineage_id`。
- non-empty `payload_hash`。
- blocking quality issue reject。
- published artifacts presence。
- expected artifact hash match。
- artifact type / schema version selection。
- ambiguous artifact reject。

CLI 当前入口：

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

当前命令定位是 dry-run artifact resolution / dry-run evidence generation，不执行导入。

## Remaining Gaps

### Identity Gaps

- Path / `file://` artifact SHA-256 verification 已有；HTTP 或对象存储 URI 的内容重算仍未实现。
- `rows_hash` 目前是 manifest / artifact 身份字段透传，尚未做 row-level recomputation。
- Evidence 仍依赖显式 `dataset_version`，不得引入隐式 `latest`。

### Regression Gaps

- 已有 opt-in local reference artifact comparison summary；它能比较本地 reference artifact 的 hash、row-count 和 sample symbols/dates，但这不等同于 ClickHouse / Quantix source-of-truth double-read。
- sample symbols / dates 提取已支持本地 Parquet。
- `failed_checks` / `warnings` 已能由本地 payload row-count 和 local reference comparison 填充；ClickHouse/source-of-truth comparison checks 仍未实现。

### Persistence Gaps

- CLI 已强制记录 `database_target` 与 `writes_performed=false`；ClickHouse shadow-table 写入策略仍未实现。
- 尚无 dataset identity 绑定到 Quantix 导入记录的实现。
- 生产 ClickHouse 写入仍是非目标，必须等 shadow import 和 miniQMT evidence loop 证明后另行批准。

### Operator Gaps

- 尚无 miniQMT validator / preview / apply 的 Quantix operator wrapper。
- 尚无本仓库内 runbook 明确如何把 Quantix evidence 同步回 miniQMT evidence directory。

## Evidence Contract Target

Quantix 生成的 evidence 应满足 miniQMT 指南里的 `quantix_regression` 最小合同：

```json
{
  "schema_version": "evidence.v1",
  "source_command": "quantix ...",
  "run_at": "2026-05-18T00:00:00Z",
  "environment": {
    "consumer_system": "quantix-rust",
    "consumer_build": "<commit-or-build-id>"
  },
  "result_summary": {
    "evidence_type": "promotion_consumer_regression",
    "consumer_system": "quantix-rust",
    "dataset_version": "<dataset_version>",
    "lineage_id": "<lineage_id>",
    "payload_hash": "<payload_hash>",
    "artifact": {
      "type": "<artifact_type>",
      "uri": "<artifact_uri>",
      "hash": "<artifact_hash>"
    },
    "regression": {
      "passed": true,
      "failed_checks": []
    }
  }
}
```

Quantix 应额外记录：

- raw report path。
- raw report hash / size。
- `rows_hash`，如果 manifest 或 artifact 暴露该字段。
- row count。
- sample symbols / dates。
- comparison summary。
- field mapping version。
- `database_target`。
- `writes_performed`。
- warnings。
- redaction notes。
- operator。
- generated_at。

## Implementation Slices

Local handoff note: Task 1 through Task 7 plus local Parquet payload sampling, local Parquet metadata row-count verification, and opt-in local reference artifact comparison have a dry-run implementation or operator-runbook artifact in the current worktree. Confirm authoritative feature status in `FUNCTION_TREE.md`; remaining implementation targets are ClickHouse/source-of-truth double-read checks and ClickHouse shadow import.

### Task 1: Register Boundaries In FUNCTION_TREE

- Modify: `FUNCTION_TREE.md`
- Goal: 把 miniQMT market dataset / controlled evidence 登记为功能节点。
- Required status split:
  - miniQMT manifest validation / dry-run artifact resolution: implemented or partially implemented, depending on final wording.
  - artifact file hash verification: designed / pending until code exists.
  - `quantix_regression` evidence generation: designed / pending until code exists.
  - ClickHouse shadow import from miniQMT artifact: designed / pending until gated persistence exists.
  - miniQMT registry preview / apply: external boundary, miniQMT-owned.
- Boundary note: `FUNCTION_TREE.md` 是状态真相；本文只作为实施规格。

Verification:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test
```

Expected: repo hygiene tests pass, especially the single status registry tests.

### Task 2: Add Manifest Identity Fields

- Modify: `src/miniqmt_market.rs`
- Test: `tests/miniqmt_market_manifest_test.rs`
- Goal: 支持 optional `rows_hash`，并让 resolved artifact 输出完整 evidence identity。

Expected additions:

- `MarketDatasetManifest.rows_hash: Option<String>` if miniQMT emits dataset-level rows hash.
- `MarketDatasetArtifact.rows_hash: Option<String>` if miniQMT emits artifact-level rows hash.
- `ResolvedMarketArtifact.lineage_id`
- `ResolvedMarketArtifact.payload_hash`
- `ResolvedMarketArtifact.maturity`
- `ResolvedMarketArtifact.quality_status`
- `ResolvedMarketArtifact.rows_hash`

TDD steps:

1. Add tests that parse manifest JSON with `rows_hash`.
2. Add tests that resolved artifact includes `lineage_id`, `payload_hash`, `maturity`, `quality_status`, and optional `rows_hash`.
3. Run the targeted test and confirm failure.
4. Implement the minimal model and mapping changes.
5. Run targeted tests again.

Commands:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test
```

### Task 3: Add Artifact Content Hash Verification

- Modify: `src/miniqmt_market.rs`
- Modify: `src/cli/commands/info.rs`
- Modify: `src/cli/handlers/import.rs`
- Test: `tests/miniqmt_market_manifest_test.rs`
- Test: `tests/miniqmt_market_import_handler_test.rs`
- Goal: 让 Quantix 能读取本地 artifact URI/path 并重算 SHA-256。

Behavior:

- Add an explicit CLI flag, for example `--verify-artifact-file`.
- Without the flag, existing dry-run behavior remains unchanged.
- With the flag, Quantix resolves the artifact path and computes content hash.
- Hash mismatch fails closed.
- Unsupported URI schemes fail with an actionable error.
- Candidate/raw/job paths are not accepted as a bypass around release manifest identity.

Commands:

```bash
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test
```

### Task 4: Generate Raw Quantix Regression Report

- Create or modify: implementation path to be chosen after checking CLI command structure.
- Test: new focused integration test.
- Goal: 生成 Quantix 自己的 raw regression report JSON。

Minimum report fields:

- `dataset_version`
- `lineage_id`
- `payload_hash`
- `rows_hash`
- `artifact.type`
- `artifact.uri`
- `artifact.hash`
- `artifact.computed_hash`
- `row_count`
- sample symbols / dates when parseable.
- `regression.passed`
- `regression.failed_checks`
- `warnings`
- `consumer_build.repo`
- `consumer_build.commit`
- `database_target`
- `writes_performed`

Initial persistence mode:

```json
{
  "database_target": "dry-run-only",
  "writes_performed": false
}
```

### Task 5: Generate miniQMT Controlled Evidence JSON

- Create or modify: implementation path to be chosen after checking CLI command structure.
- Test: evidence JSON shape test.
- Goal: wrap the raw regression report into miniQMT-compatible `quantix_regression` evidence.

Rules:

- Evidence must bind explicit `dataset_version`.
- `regression.passed=false` fails closed.
- Non-empty `failed_checks` fails closed.
- Artifact computed hash must match manifest hash before evidence can be marked passed.
- Evidence must include raw report reference and raw report hash / size.
- Evidence output must be easy to sync into miniQMT `DOCS/**/evidence/*.evidence.json`.

### Task 6: Add Controlled Persistence Policy

- Create or modify: implementation path to be chosen after checking existing ClickHouse import APIs.
- Test: persistence policy unit tests before any real ClickHouse integration.
- Goal: prevent accidental production writes.

Allowed phases:

| Phase | `database_target` | `writes_performed` | Boundary |
| --- | --- | --- | --- |
| 1 | `dry-run-only` | `false` | Default. No database writes. |
| 2 | `clickhouse-shadow:<table>` | `true` | Explicit shadow/staging write only. |
| 3 | `clickhouse-production:<table>` | `true` | Not implemented until separate approval and registry evidence loop are proven. |

Production writes are out of scope for the first implementation.

### Task 7: Operator Runbook / Wrapper

- Create or modify: `docs/operations/MINIQMT_QUANTIX_REGRESSION_OPERATOR_RUNBOOK_2026-05-20.md`.
- Goal: document how Quantix evidence is validated and applied by miniQMT tooling.

Operator flow:

1. Query miniQMT promotion requirements / gaps for explicit `dataset_version`.
2. Run Quantix regression / evidence command.
3. Validate evidence with miniQMT local validator.
4. Run miniQMT promotion apply in `--plan-only`.
5. Apply evidence to miniQMT registry only after preview is accepted.

Boundary:

- Quantix may provide wrapper convenience later.
- miniQMT remains registry owner.
- Default wrapper mode must be preview / plan-only.
- Current implementation is a manual runbook only; no Quantix wrapper command has been added.

## Shortest Safe Development Path

Recommended first pull request / commit slice:

1. Update `FUNCTION_TREE.md` with explicit miniQMT controlled evidence status nodes.
2. Add optional `rows_hash` and richer resolved artifact identity.
3. Add artifact content hash verification for local artifact paths.
4. Add tests for rows hash and hash mismatch.
5. Keep ClickHouse writes and miniQMT registry apply out of the first slice.

Recommended second slice:

1. Add raw Quantix regression report JSON.
2. Add miniQMT evidence JSON generator.
3. Keep `database_target="dry-run-only"` and `writes_performed=false`.

Recommended third slice:

1. Add ClickHouse shadow import policy.
2. Add miniQMT validator / preview runbook or wrapper.
3. Do not implement production writes.

## Verification Gates

Minimum gates for the first implementation slice:

```bash
cargo fmt --check
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_manifest_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test miniqmt_market_import_handler_test
cargo test --manifest-path /opt/claude/quantix-rust/Cargo.toml --test repo_hygiene_test
```

Before commit:

- Run GitNexus impact before editing Rust symbols.
- Run GitNexus detect changes on staged scope.
- Stage only files owned by this workstream.
- Do not mix dependency security audit remediation into this workstream.

## Open Decisions

1. Evidence output location inside Quantix should be a generated operator output path, not a status source.
2. Final evidence intended for miniQMT promotion should be copied or generated into miniQMT-compatible `DOCS/**/evidence/*.evidence.json`.
3. ClickHouse shadow table naming should be decided when implementing persistence; production table writes remain out of scope.
