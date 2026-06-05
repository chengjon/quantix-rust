# Remote Branch Board Audit - 2026-06-05

## Scope

This report records the remote branch board after closing the fully-covered `master-wip-*` branches and the one fully-covered notify branch. It deliberately does not delete branches that still have branch-only commits.

## Closed In This Pass

| Branch | Evidence | Action |
| --- | --- | --- |
| `master-wip-20260324-graphiti-sync` | Branch tip was already an ancestor of `master`; `master...branch` count was `373 0`; diff files vs merge-base: 0 | Deleted remote branch |
| `master-wip-20260325-continue` | Branch tip was already an ancestor of `master`; `master...branch` count was `366 0`; diff files vs merge-base: 0 | Deleted remote branch |
| `fix/notify-send-channel-validation` | Branch tip was already an ancestor of `master`; post-audit classification: branch-only commits 0 and diff files 0 | Deleted remote branch |

## Remaining Remote Branches

After pruning, `origin/master` plus 23 non-master remote branches remain. All 23 non-master branches below have branch-only content, so they were preserved for a separate product/design review instead of being deleted as stale pointers.

| Branch | Tip | Branch-only commits | Master-only commits | Diff files | Tip date | Tip subject |
| --- | --- | ---: | ---: | ---: | --- | --- |
| `fix/cli-fail-closed-next` | `21a84e716d18` | 1 | 18 | 2 | 2026-06-04 11:58:20 | fix: fail closed account group strategy validation |
| `phase24b-guidance` | `a0f207719202` | 1 | 445 | 3 | 2026-03-24 17:28:53 | docs: add phase24b monitor automation usage |
| `phase27d-blocklist-enforcement-core` | `64f52b377cc7` | 4 | 395 | 8 | 2026-03-25 08:00:46 | feat: enforce industry-blocklist in buy checks |
| `phase27d-resolver-base` | `1c625dd8b6fd` | 1 | 395 | 4 | 2026-03-25 03:30:56 | feat: add phase27d industry resolver |
| `phase27d-ruletype` | `bd8da4489038` | 2 | 395 | 8 | 2026-03-25 03:49:40 | feat: add industry-blocklist rule type |
| `phase27d-sqlite-resolver` | `09409fac46b5` | 3 | 395 | 8 | 2026-03-25 07:24:19 | feat: add sqlite shenwan industry resolver |
| `phase27d-trade-cli-wiring` | `7ec86d4a5ac1` | 5 | 395 | 9 | 2026-03-25 08:22:07 | feat: wire trade cli to runtime risk checks |
| `phase29a-guidance` | `daf5b83aeb86` | 1 | 445 | 4 | 2026-03-24 17:32:03 | docs: add phase29a strategy paper guidance |
| `phase29a-guidance-refresh` | `82898fa8f750` | 1 | 409 | 4 | 2026-03-25 00:56:53 | docs: add phase29a strategy paper guidance |
| `phase29a-guidance-refresh2` | `c9a9f5578bf8` | 1 | 409 | 4 | 2026-03-25 01:05:01 | docs: add phase29a strategy paper guidance |
| `phase29a-paper-kernel` | `c262b67cf2c6` | 4 | 395 | 12 | 2026-03-25 12:48:05 | feat: add phase29a paper execution kernel |
| `phase29a-paper-mode` | `e17258a234ae` | 5 | 395 | 15 | 2026-03-25 13:04:51 | feat: wire phase29a strategy paper mode |
| `phase29a-runtime-path` | `b6268b333e77` | 1 | 395 | 1 | 2026-03-25 11:21:57 | feat: add strategy runtime config paths |
| `phase29a-runtime-store` | `21ec10a905b7` | 2 | 395 | 6 | 2026-03-25 12:00:28 | feat: add phase29a execution runtime store |
| `phase29a-signal-translation` | `f45b3023b824` | 3 | 395 | 9 | 2026-03-25 12:36:04 | feat: add phase29a strategy signal translation |
| `phase29b-config-stores` | `ac5d31cc0bbf` | 6 | 395 | 18 | 2026-03-25 13:25:24 | feat: add phase29b strategy daemon config stores |
| `phase29b-guidance` | `60283d84c4d7` | 1 | 445 | 5 | 2026-03-24 17:40:51 | docs: add phase29b strategy daemon guidance |
| `phase29b-guidance-refresh` | `d9aff5ac1c24` | 1 | 409 | 5 | 2026-03-25 01:00:29 | docs: add phase29b strategy daemon guidance |
| `phase29b-guidance-refresh2` | `928ee0baced2` | 1 | 407 | 5 | 2026-03-25 01:17:01 | docs: add phase29b strategy daemon guidance |
| `phase29b-signal-runtime-store` | `e06a9fd2cfa2` | 7 | 395 | 18 | 2026-03-25 13:35:00 | feat: add phase29b signal runtime store |
| `phase29c-filldelta-review` | `6a15c21097ad` | 2 | 444 | 1 | 2026-03-24 13:31:18 | docs: refine phase29c fill delta accounting design |
| `phase29c-lifecycle-review` | `433367127ed5` | 3 | 444 | 1 | 2026-03-24 14:20:58 | docs: refine phase29c mock live execution design |
| `phase29c-review-followups` | `e76b4a9d3a5b` | 1 | 444 | 1 | 2026-03-24 13:07:25 | docs: refine phase29c design review follow-ups |

## Verification

- `git push origin --delete master-wip-20260324-graphiti-sync master-wip-20260325-continue`: succeeded in the prior closure pass.
- `git push origin --delete fix/notify-send-channel-validation`: succeeded in this pass.
- `git fetch --prune origin`: succeeded after deletion.
- `git branch -r --list 'origin/master*'`: now returns only `origin/master`.
- Remaining non-master remote branches were classified with `git rev-list --left-right --count master...<branch>` and `git diff --name-only $(git merge-base master <branch>) <branch>`.

## Decision

The remote stale-pointer cleanup is closed. The remaining 23 non-master remote branches are not stale pointers; they carry branch-only content and should be handled as a separate grouped product/design triage, not deleted as cleanup.
