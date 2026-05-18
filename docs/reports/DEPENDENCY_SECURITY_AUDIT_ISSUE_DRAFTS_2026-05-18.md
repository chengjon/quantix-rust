# Dependency Security Audit Issue Drafts - 2026-05-18

> Drafts only. These have not been published to GitHub issues. They are grouped by dependency owner so the security workstream can proceed without mixing into the closed quality-gate recovery.

## Parent

- Spec: `docs/superpowers/specs/2026-05-18-dependency-security-audit-execution-spec.md`
- Evidence: `docs/CODE_AUDIT_EVIDENCE/dependency-security/advisories-2026-05-18.json`
- CI run: https://github.com/chengjon/quantix-rust/actions/runs/26007862679

## Proposed Breakdown

### 1. Migrate or contain SQLx/MySQL security advisories

- Type: HITL
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2024-0363`, `RUSTSEC-2023-0071`, `RUSTSEC-2026-0097`

## What to build

Decide whether the MySQL feature remains part of the supported runtime surface, then either migrate SQLx to a fixed series or remove/gate vulnerable feature paths with explicit documentation.

## Acceptance criteria

- [ ] The SQLx advisory and rsa transitive path are either removed from cargo audit output or explicitly accepted with owner and revisit trigger.
- [ ] Any SQLx feature change is reflected in CLI/runtime documentation and tests that exercise database-backed paths.
- [ ] cargo audit, cargo check --all-targets, and relevant database tests are recorded after the change.

## Blocked by

- None - can start after this evidence commit.

### 2. Upgrade HTTP client stack for idna and rustls-pemfile advisories

- Type: AFK
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2024-0421`, `RUSTSEC-2025-0134`

## What to build

Evaluate a reqwest/cookie-store upgrade that removes the idna 0.3 and rustls-pemfile 1.0 paths without changing market/news provider behavior.

## Acceptance criteria

- [ ] idna 0.3.0 and rustls-pemfile 1.0.4 no longer appear in the resolved dependency graph or have documented acceptance.
- [ ] HTTP-dependent market/news smoke or unit tests still pass.
- [ ] cargo audit delta is recorded before and after the migration.

## Blocked by

- None - can start after this evidence commit.

### 3. Remove yanked vulnerable lz4_flex via ClickHouse and Parquet paths

- Type: AFK
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2026-0041`, `NO-ID:lz4_flex:yanked`

## What to build

Try the narrowest lockfile/dependency update that moves lz4_flex from 0.11.5 to a fixed non-yanked release while preserving ClickHouse and parquet IO behavior.

## Acceptance criteria

- [ ] cargo audit no longer reports lz4_flex RUSTSEC-2026-0041 or yanked status.
- [ ] ClickHouse and parquet-related tests still pass or any missing coverage is called out.
- [ ] The lockfile diff is limited to the dependency chain required for lz4_flex unless evidence justifies a wider update.

## Blocked by

- None - can start after this evidence commit.

### 4. Assess Polars upgrade path for fast-float advisories

- Type: HITL
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2025-0003`, `RUSTSEC-2024-0379`

## What to build

Evaluate whether a Polars upgrade or risk acceptance is the right response to fast-float advisories that currently have no fixed direct crate upgrade.

## Acceptance criteria

- [ ] A Polars migration candidate is tested against factor/data pipeline gates, or a risk acceptance record explains why migration is deferred.
- [ ] fast-float reachability through polars-arrow/polars-io is documented in the advisory register.
- [ ] No broad Polars version change is merged without factor score and parquet export regression evidence.

## Blocked by

- None - can start after this evidence commit.

### 5. Decide TAOS optional feature security posture

- Type: HITL
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2021-0041`, `RUSTSEC-2023-0065`

## What to build

Decide whether optional TAOS support remains enabled under all-features, then upgrade, gate, remove, or risk-accept the parse_duration and tungstenite paths.

## Acceptance criteria

- [ ] parse_duration and tungstenite 0.18 reachability is classified as all-features-only or removed.
- [ ] The TAOS feature boundary is documented if retained or changed.
- [ ] cargo audit behavior is documented for default and all-features builds.

## Blocked by

- None - can start after this evidence commit.

### 6. Upgrade or contain Prometheus protobuf advisory

- Type: AFK
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2024-0437`

## What to build

Evaluate a Prometheus dependency upgrade or metrics-path containment that removes protobuf 2.28.0 from the resolved graph.

## Acceptance criteria

- [ ] protobuf 2.28.0 no longer appears in cargo audit output or has documented acceptance.
- [ ] Monitoring/metrics tests or compile gates verify the metrics surface after migration.
- [ ] Any public metrics behavior change is documented.

## Blocked by

- None - can start after this evidence commit.

### 7. Clean direct and UI/config utility dependency warnings

- Type: AFK
- Blocked by: Evidence baseline only
- Advisory IDs: `RUSTSEC-2021-0141`, `RUSTSEC-2025-0119`, `RUSTSEC-2024-0320`, `RUSTSEC-2026-0002`, `RUSTSEC-2026-0097`

## What to build

Handle direct and utility warning paths such as dotenv, indicatif/number_prefix, config/yaml-rust, optional ratatui/lru, and direct rand usage without touching unrelated security migrations.

## Acceptance criteria

- [ ] Each warning is either removed from audit output or has an explicit documented acceptance row.
- [ ] Optional UI-only advisories are classified separately from default runtime advisories.
- [ ] Changes remain scoped to utility dependencies and their direct tests.

## Blocked by

- None - can start after this evidence commit.

## Publishing Notes

- Publish blocker/decision issues first (`sqlx`, `taos-ws`, `polars`) if the team wants human sign-off before dependency migration.
- AFK issues should stay narrow and must include before/after `cargo audit` evidence.
- Do not close the security audit workstream until every advisory is removed or explicitly accepted with owner and revisit condition.
