# Dependency Security Audit Execution Spec - 2026-05-18

## 1. Purpose

This spec starts a separate dependency security audit workstream for the RustSec
advisories currently blocking the repository security gate.

It is intentionally separate from the clean-checkout gate recovery that ended at:

- Commit: `3676cb4 ci: restore clean checkout gates`
- CI run: `https://github.com/chengjon/quantix-rust/actions/runs/26007862679`
- Closed quality gates in that run: `Lint`, `Test`, `Documentation`
- Remaining blocking gate in that run: `Security Audit`

The security audit must not be mixed into the prior fmt/test/clippy/docs recovery
patch. Dependency upgrades, lockfile churn, `cargo audit` policy changes, or
advisory ignores require their own evidence and review.

## 2. Scope

In scope:

- Rust dependency advisories reported by `cargo audit` against the current
  `Cargo.lock`.
- Direct dependency migrations needed to remove vulnerable transitive packages.
- Optional-feature containment where an advisory is only reachable under
  `--all-features`.
- Explicit risk-acceptance records only when no fixed upgrade exists or the
  remediation requires a larger product decision.
- CI/security-workflow policy changes only after the advisory has a documented
  decision.

Out of scope:

- Reopening the already closed quality gate work.
- Cosmetic refactors or warning cleanup unrelated to a RustSec advisory.
- Blanket `cargo audit --ignore` changes without per-advisory justification.
- Treating Security Audit as proof that fmt/test/clippy/docs are not closed.

## 3. Current Baseline

| Item | Current state |
| --- | --- |
| Branch baseline | `origin/master` at `3676cb4` |
| Dedicated worktree | `.worktrees/security-dependency-audit-20260518` |
| CI run evidence | `26007862679` |
| `Lint` | `success` |
| `Test` | `success` |
| `Documentation` | `success` |
| `Security Audit` | `failure` |
| CI security command | `.github/workflows/ci.yml` runs `cargo audit` |
| Scheduled security command | `.github/workflows/audit.yml` runs `cargo audit` and `cargo audit --json` |

## 4. Advisory Register

Initial evidence comes from the failed `Security Audit` job in run `26007862679`
and local `cargo metadata --locked --all-features` dependency-path inspection.
Every row starts as `OPEN` until a remediation, defer, or risk-acceptance record
is committed.

| ID | Crate | Version | Class | Initial dependency path | First decision needed | Status |
| --- | --- | --- | --- | --- | --- | --- |
| `RUSTSEC-2025-0003` | `fast-float` | `0.2.0` | vulnerability | `polars` / `polars-ops` -> `polars-arrow` / `polars-io` | Evaluate Polars upgrade path or documented risk; advisory reports no fixed direct upgrade for the crate. | `OPEN` |
| `RUSTSEC-2024-0379` | `fast-float` | `0.2.0` | unsound warning | Same as above | Same as above; do not duplicate remediation. | `OPEN` |
| `RUSTSEC-2024-0421` | `idna` | `0.3.0` | vulnerability | `reqwest 0.11.27` -> `cookie_store 0.20.0` | Evaluate `reqwest` / cookie stack upgrade. | `OPEN` |
| `RUSTSEC-2026-0041` | `lz4_flex` | `0.11.5` | vulnerability | `clickhouse 0.12.2`; `parquet 53.4.1` | Try narrow lockfile update to a fixed `lz4_flex` before broader dependency upgrades. | `OPEN` |
| none in audit output | `lz4_flex` | `0.11.5` | yanked warning | Same as above | Same remediation path as `RUSTSEC-2026-0041`. | `OPEN` |
| `RUSTSEC-2021-0041` | `parse_duration` | `2.1.1` | vulnerability | `taos-ws 0.5.12` under `--all-features` | Decide whether TAOS support is required, gated, upgraded, or risk-accepted; advisory reports no fixed upgrade. | `OPEN` |
| `RUSTSEC-2024-0437` | `protobuf` | `2.28.0` | vulnerability | `prometheus 0.13.4` | Evaluate Prometheus upgrade or replacement path. | `OPEN` |
| `RUSTSEC-2023-0071` | `rsa` | `0.9.10` | vulnerability | `sqlx 0.7.4` with `mysql` feature | Decide whether MySQL feature is required; evaluate SQLx upgrade or feature removal; advisory reports no fixed direct upgrade for `rsa`. | `OPEN` |
| `RUSTSEC-2024-0363` | `sqlx` | `0.7.4` | vulnerability | direct dependency | Evaluate upgrade to `sqlx >=0.8.1` and required code/test changes. | `OPEN` |
| `RUSTSEC-2023-0065` | `tungstenite` | `0.18.0` | vulnerability | `taos-ws 0.5.12` under `--all-features` | Decide TAOS feature remediation; direct `tokio-tungstenite 0.21.0` resolves to `tungstenite 0.21.0`. | `OPEN` |
| `RUSTSEC-2021-0141` | `dotenv` | `0.15.0` | unmaintained warning | direct dependency | Replace with `dotenvy` or remove dependency if env loading is not required. | `OPEN` |
| `RUSTSEC-2025-0119` | `number_prefix` | `0.4.0` | unmaintained warning | `indicatif 0.17.11` | Evaluate `indicatif` upgrade or risk acceptance. | `OPEN` |
| `RUSTSEC-2024-0436` | `paste` | `1.0.15` | unmaintained warning | `parquet`, `sqlx-core`, `statrs` | Track transitive owners; likely resolved only through broader dependency upgrades or accepted as warning. | `OPEN` |
| `RUSTSEC-2025-0134` | `rustls-pemfile` | `1.0.4` | unmaintained warning | `reqwest 0.11.27` | Evaluate `reqwest` upgrade. | `OPEN` |
| `RUSTSEC-2024-0320` | `yaml-rust` | `0.4.5` | unmaintained warning | `config 0.13.4` | Evaluate `config` upgrade or replacement. | `OPEN` |
| `RUSTSEC-2026-0002` | `lru` | `0.12.5` | unsound warning | `ratatui 0.25.0` under `--all-features` | Decide whether optional TUI feature should upgrade, be gated, or be accepted. | `OPEN` |
| `RUSTSEC-2026-0097` | `rand` | `0.8.5` | unsound warning | direct dependency plus `polars-ops`, `rust_decimal`, `statrs` | Evaluate direct `rand` migration and transitive blockers separately. | `OPEN` |
| `RUSTSEC-2026-0097` | `rand` | `0.9.2` | unsound warning | `rust_decimal` / PostgreSQL protocol stack | Evaluate `rust_decimal` / SQLx-related upgrade path. | `OPEN` |

## 5. Work Plan

### Phase 0: Preserve the Current Gate Boundary

- Record `3676cb4` and run `26007862679` as the boundary between quality-gate
  recovery and dependency-security work.
- Do not modify the previous clean-checkout recovery commit.
- Do not reuse the temporary cargo-update experiment as an implementation patch.

Acceptance:

- This spec exists in a branch or worktree that starts from `origin/master` at
  `3676cb4`.
- `git status --short` for the security worktree shows only intentional
  security-audit artifacts before commit.

### Phase 1: Reproduce and Normalize Audit Evidence

- Run or collect `cargo audit --json` evidence for the exact baseline.
- If local `cargo audit` is unavailable, use the GitHub Actions log as the
  initial source and install/run `cargo-audit` only in the isolated security
  worktree.
- Export a machine-readable advisory summary before changing dependencies.

Acceptance:

- The advisory count and IDs are reproducible from a named command or CI run.
- Each advisory has a direct, transitive, or optional-feature path.

### Phase 2: Low-Risk Remediation Candidates

Try narrow, separately reviewable changes first:

- `lz4_flex`: attempt a precise fixed-version lockfile update.
- `dotenv`: replace direct dependency with a maintained alternative or remove it.
- `idna` / `rustls-pemfile`: evaluate `reqwest` upgrade separately.
- `yaml-rust`: evaluate `config` upgrade separately.

Acceptance:

- Each candidate is isolated in a small commit or documented as not viable.
- `cargo audit` delta is recorded after each candidate.

### Phase 3: Larger Migration Decisions

Handle advisories that require product or feature decisions:

- `sqlx` / `rsa`: decide whether the `mysql` feature is required before a broad
  SQLx migration.
- `taos-ws` / optional all-features advisories: decide whether TAOS support is
  retained, upgraded, feature-gated, or explicitly deferred.
- `polars` / `fast-float`: evaluate whether a Polars upgrade is feasible within
  this repository's factor/data pipeline tests.
- `rand`: split direct usage from transitive dependency blockers.

Acceptance:

- Any retained vulnerability has a documented owner, reason, and expiration or
  revisit trigger.
- No advisory is hidden by CI configuration without a matching row in the
  advisory register.

### Phase 4: Verification and CI Closure

Required gates before claiming closure:

```bash
cargo audit
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-targets
cargo test --doc --all-features
```

If the final state intentionally retains advisories:

- `cargo audit` policy must name exact advisory IDs.
- Every ignored or accepted advisory must have a register row with rationale,
  owner, and revisit condition.
- CI must continue to fail on new, unregistered advisories.

## 6. Non-Negotiable Rules

- Do not combine dependency security remediation with unrelated cleanup.
- Do not make the prior clean-checkout gate recovery look unstable because
  Security Audit remains open.
- Do not use blanket `cargo audit --ignore` or workflow `continue-on-error`.
- Do not accept yanked packages without a written exception.
- Do not claim the security audit is closed until `cargo audit` evidence proves
  it, or until every remaining advisory has an explicit, reviewed acceptance.

## 7. First Implementation Slice

The first implementation slice should be evidence-only:

1. Commit this spec.
2. Produce an advisory evidence artifact from `cargo audit --json` or the CI log.
3. Open remediation issues or issue drafts grouped by dependency owner:
   `sqlx`, `reqwest`, `polars`, `taos-ws`, `clickhouse/parquet`,
   `prometheus`, direct utility dependencies.
4. Only then begin lockfile or dependency changes.
