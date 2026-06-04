# Local Branch Closure Audit - 2026-06-05

## Scope

This audit follows the P0/P1/P2 preserved-worktree closure reports and covers
remaining non-`master` local branches after the worktree board was cleared.

This audit does not touch root worktree dirt. The following files are treated as
external-line work and remain out of scope:

- `src/core/config.rs`
- `src/sources/mod.rs`
- `src/sources/tdx_api.rs`

## Dispositions

| Branch | Disposition | Evidence |
|---|---|---|
| `backup/master-pre-sync-20260501-3a8902f` | Archive, then remove local branch. | Local-only backup branch, 4 commits ahead of `master`, 209 commits behind. Head preserved as `archive/local-backup-master-pre-sync-20260501-20260605`. |
| `chore/mock-policy-qmt-gate` | Archive, then remove local branch. | Local-only older qmt-gate line. It was superseded by the later qmt-gate full/local-merge line and current `master` behavior, but its exact head is preserved as `archive/local-mock-policy-qmt-gate-20260605`. |
| `chore/mock-policy-qmt-gate-local-merge` | Remove local branch. | Points to `78ffda1284cadcc19710193fd283c0f4e23aabfd`, already preserved by `archive/p2-mock-policy-qmt-gate-full-20260605`. |
| `feat/local-fundamentals-import-cli` | Remove local branch. | Head is already contained in `master`; `master..branch` has 0 commits and no diff. |
| `rescue/dirty-master-2026-05-26` | Archive, then remove local branch. | Local-only rescue branch, 1 commit ahead of `master`, 172 commits behind. Head preserved as `archive/local-rescue-dirty-master-20260526-20260605`. |
| `wip/local-target-size-guard-20260502` | Remove local branch and stale remote branch. | Head is already contained in `master`; `master..branch` has 0 commits and no diff. A remote head still exists and can be pruned after this audit lands. |

## Archive Tags

| Tag | Preserved Head |
|---|---|
| `archive/local-backup-master-pre-sync-20260501-20260605` | `2bbae0391c9ca4a9e28cca534edeb7978616076a` |
| `archive/local-mock-policy-qmt-gate-20260605` | `b1a0b34bcccd45c8bc94ac4f6390a3f9ada62493` |
| `archive/local-rescue-dirty-master-20260526-20260605` | `14ab859af2ac1312aec67d8aa78dfca2e5a83f4b` |
| `archive/p2-mock-policy-qmt-gate-full-20260605` | `78ffda1284cadcc19710193fd283c0f4e23aabfd` |

Recovery command shape:

```bash
git fetch origin --tags
git switch -c <new-branch-name> <archive-tag-name>
```

## Closure Boundary

After this audit lands and the listed branch cleanup is applied:

- only `master` should remain as a local branch;
- only the root `master` worktree should remain;
- `wip/local-target-size-guard-20260502` should no longer exist as a remote
  branch;
- archived branch heads remain recoverable from tags.

This closes the local branch board without merging broad stale work into
`master`.
