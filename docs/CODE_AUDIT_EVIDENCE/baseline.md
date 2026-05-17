# Code Audit Baseline

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Capture

| Field | Value |
|---|---|
| Captured at | 2026-05-15 11:34:26 Asia/Shanghai |
| Audit spec | `docs/superpowers/specs/2026-05-15-code-audit-execution-spec.md` |
| Methodology | `docs/standards/CODE_AUDIT_METHODOLOGY.md` |
| Branch | `master` |
| HEAD | `b30de3123cbba3ffd8a040a0caf60258140f643f` |
| Worktree status | Dirty: 164 status entries |
| Modified/deleted entries | 142 |
| Untracked entries | 22 |

The audit was executed against the current dirty worktree. Findings and gate outcomes describe the local workspace, not a clean release candidate.

## Toolchain

| Tool | Version |
|---|---|
| `cargo` | `cargo 1.90.0 (840b83a10 2025-07-30)` |
| `rustc` | `rustc 1.90.0 (1159e78c4 2025-09-14)` |
| `rustfmt` | `rustfmt 1.8.0-stable (1159e78c47 2025-09-14)` |

## GitNexus

| Check | Result |
|---|---|
| Repository | `quantix-rust` |
| Index state | `ready` |
| Indexed at | `2026-05-12T10:03:56.224Z` |
| Indexed commit | `b30de31` |
| Indexed files | 562 |
| Indexed symbols | 6441 |
| Indexed processes | 300 |
| Refresh command | `gitnexus analyze` |
| Refresh result | exit 0, `Already up to date` |

The GitNexus graph matches committed HEAD but does not index untracked files or uncommitted edits as graph symbols.

## Repository Counts

| Count | Value |
|---|---:|
| `src/` top-level modules | 28 |
| `src/cli/commands/*.rs` | 13 |
| `src/cli/handlers/*.rs` | 46 |
| `tests/*.rs` | 74 |
| `config/` files | 15 |

Top-level `src/` modules measured: `account`, `ai`, `analysis`, `anomaly`, `bridge`, `cli`, `core`, `data`, `db`, `execution`, `factor`, `fundamental`, `import`, `io`, `market`, `monitor`, `monitoring`, `news`, `risk`, `screener`, `sources`, `stop`, `strategy`, `sync`, `tasks`, `trade`, `tui`, `watchlist`.

## Existing Audit Evidence

The previous evidence package existed before this run and was treated as historical input. The prior `findings.csv` had 19 rows and one known open finding, `AUDIT-S3-009`, which was re-evaluated and carried forward.

## Baseline Constraints

- The worktree contains broad pre-existing edits and untracked files, so gate failures may be caused by in-progress local work.
- The audit does not remediate code; it records evidence and finding status.
- Feature availability claims remain outside this evidence package and must be read from `FUNCTION_TREE.md`.
