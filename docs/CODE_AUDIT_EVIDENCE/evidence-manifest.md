# Evidence Manifest

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

## Scope

This manifest records the 2026-05-17 post-review supplement and the 2026-05-18 release-build gate closure update to the 2026-05-15 code audit evidence package. It preserves artifact actions and checksums without changing source-code findings.

The self-checksum of this manifest is intentionally not embedded because adding it would change the file content.

## Artifact Manifest

| Artifact | Action | Old SHA-256 | New SHA-256 | Command/source | Reason | Archive path |
|---|---|---:|---:|---|---|---|
| `docs/CODE_AUDIT_EVIDENCE/baseline.md` | unchanged | n/a | `4f175f9f09db0c567c6e231eea7e06014dfb9a078bf407190f7a666945f21522` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/cargo-gates.md` | refreshed | `5de6ee45def911c4d32b9563c827c3f2f808a1b33fb04aaa58939bb23d26c6d9` | `b2b164057fa631b20e019a8b47affc3b30c7a3b13af6aa537c5d663cb940efe2` | release-build gate follow-up | Records `AUDIT-S3-010` release-build pass evidence. | not archived; prior checksum recorded here |
| `docs/CODE_AUDIT_EVIDENCE/gitnexus-queries.md` | unchanged | n/a | `707009315dabc51631056057eba59740ea88b44b789409263b870c4f154bfee3` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/pattern-scan-summary.csv` | unchanged | n/a | `22f18059cb54144e5dec47fb4e5eb590907af647310c7ebfb10065c694c145e5` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/pattern-hotspots.md` | unchanged | n/a | `a56d7e962d5835875af83301ceebfca0beb9bd5cb911b42dce7fa28fa0130ff0` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/manual-review-log.md` | unchanged | n/a | `dac7fb900c6440708a2b10f2179a903bd448071253d6e4d764634977ae96ddcb` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/sampled-files.md` | unchanged | n/a | `dd9cceec32cdfa428be31bcd10026fd57ef08ee4b18fd3db54d4567f616451b5` | existing audit evidence | Recorded for manifest completeness. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/findings.csv` | refreshed | `6d3ee379fa9137a723d03dd3fcc9560bc86ae995b711bdbe98533dac8c801758` | `c4800aa076c9b93fc1f3915c1cccaea62e852e609479aac811cbba62cc07601f` | release-build gate follow-up | Changes `AUDIT-S3-010` from `needs-repro` to fixed after reproducible release-build pass. | not archived; prior checksum recorded here |
| `docs/CODE_AUDIT_EVIDENCE/graphiti-memory.md` | created | n/a | `e2060ca9a17bb3a233f12c45ca92be7c9d6a34b0c47d16bf40da18c57eb3ac45` | Graphiti read/write evidence from audit and review handling | Implements the review request for auditable read-side memory evidence. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/logs/README.md` | created | n/a | `0d63dbf04a73fa11a177e930b7935ebd13d982f4d3421838f99809c223eb4ea8` | post-review supplement | Documents the long-running gate log location required by the hardened spec. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/logs/cargo-build-release-20260517T174008Z.log` | created | n/a | `3aaaeddf97160668f9b6b8580b2ff0f7dc72786a5a89576a858da4b7c0879d7c` | release-build gate follow-up | Captures release-build process handling and exit status 0 evidence for `AUDIT-S3-010`. | n/a |
| `docs/CODE_AUDIT_EVIDENCE/archive/README.md` | created | n/a | `c83eefaa790612b1590365d9f3c16706e1873626202d21bc9b8f1f9f87222f2e` | post-review supplement | Documents the historical evidence archive location required by the hardened spec. | n/a |
| `docs/reports/CODE_AUDIT_FINAL_2026-05-15.md` | refreshed | `e55de0d86d64c95a2cc858c488d8f06c109b1dc9ce6a542f65196304606b0b50` | `dc07cc3aaa8405e5424e384f94c70e7de575195015e760ae1dd12c8392253455` | release-build gate follow-up | Updates `AUDIT-S3-010` from release-build unverified to fixed while keeping release confidence open until `AUDIT-S2-010` and `AUDIT-S2-011` are committed. | not archived; prior checksum recorded here |
| `docs/reports/CODE_AUDIT_MATT_SKILLS_ISSUE_DRAFTS_2026-05-15.md` | created | n/a | `fcf520fd2db88dc54af4715b8d3c4c1e06e7ebdfcc80cf7e271a56c8ac2e813b` | issue-tracking follow-up | Records GitHub issue state for #1/#2/#3 and release-build closure notes. | n/a |
| `docs/superpowers/specs/2026-05-15-code-audit-execution-spec.md` | refreshed | `8f00f09d3b2c5494a2b0173177c157d6524d5afec73ff315598128e0c8a9ee01` | `1c9dc2f5e18a707a4ed741be7469887a43c0fe640dc5c99574db2f56af548487` | Impeccable review handling | Hardens scan scope, long-running gates, Graphiti evidence, documentation hygiene, Phase 4 path coverage, clippy warning handling, and evidence refresh mechanics. | not archived; prior checksum recorded here |
| `docs/CODE_AUDIT_EVIDENCE/evidence-manifest.md` | created | n/a | self-referential | post-review supplement | Provides this manifest. | n/a |

## Boundary

The manifest is evidence metadata only. It does not define feature availability, designed/pending status, or implementation boundaries. Those remain in `FUNCTION_TREE.md`.
