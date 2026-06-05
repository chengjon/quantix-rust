# Remote Live Branch Triage - 2026-06-05

## Scope

This report classifies the remaining live non-master remote branches after the
stale-pointer cleanup closed. It does not delete or archive any branch: every
branch below still has branch-only content and must be treated as product/design
backlog until a focused merge or supersession review is authorized.

## Executive Decision

- Stop cleanup deletion here: no remaining non-master remote branch is a
  zero-diff stale pointer.
- Treat the board as 6 work streams, not 23 isolated cleanup tasks.
- The next executable stage should make one product/design decision per work
  stream: merge candidate, superseded with archive tag, or intentionally
  preserved backlog.
- Do not delete guidance refresh branches only by name similarity. They still
  have branch-only commits and must be compared against the current docs and the
  latest implementation stream before any archive action.

## Board Summary

| Group | Branches | Strongest current candidate | Branch-only range | Diff-file range | Disposition |
| --- | ---: | --- | ---: | ---: | --- |
| fix | 1 | `fix/cli-fail-closed-next` | 1-1 | 2-2 | Single fix candidate; requires code impact and test gates before merge review |
| phase24b | 1 | `phase24b-guidance` | 1-1 | 3-3 | Docs/guidance candidate; compare against current docs before archive or merge |
| phase27d | 5 | `phase27d-trade-cli-wiring` | 1-5 | 4-9 | Implementation stack; latest trade CLI wiring appears to contain the stack |
| phase29a | 8 | `phase29a-paper-mode` | 1-5 | 1-15 | Paper execution stack plus duplicate guidance candidates; latest paper mode appears strongest |
| phase29b | 5 | `phase29b-signal-runtime-store` | 1-7 | 5-18 | Daemon/runtime stack plus duplicate guidance candidates; latest signal runtime store appears strongest |
| phase29c | 3 | `phase29c-lifecycle-review` | 1-3 | 1-1 | Design-review spec chain; lifecycle review appears to contain follow-up commits |

## Containment Evidence Within Groups

- `phase27d-resolver-base` is an ancestor of `phase27d-ruletype`,
  `phase27d-sqlite-resolver`, `phase27d-blocklist-enforcement-core`, and
  `phase27d-trade-cli-wiring`.
- `phase27d-ruletype` is an ancestor of `phase27d-sqlite-resolver`,
  `phase27d-blocklist-enforcement-core`, and `phase27d-trade-cli-wiring`.
- `phase27d-sqlite-resolver` is an ancestor of
  `phase27d-blocklist-enforcement-core` and `phase27d-trade-cli-wiring`.
- `phase27d-blocklist-enforcement-core` is an ancestor of
  `phase27d-trade-cli-wiring`.
- `phase29a-runtime-path` is an ancestor of `phase29a-runtime-store`,
  `phase29a-signal-translation`, `phase29a-paper-kernel`, and
  `phase29a-paper-mode`.
- `phase29a-runtime-store` is an ancestor of `phase29a-signal-translation`,
  `phase29a-paper-kernel`, and `phase29a-paper-mode`.
- `phase29a-signal-translation` is an ancestor of `phase29a-paper-kernel` and
  `phase29a-paper-mode`.
- `phase29a-paper-kernel` is an ancestor of `phase29a-paper-mode`.
- `phase29b-config-stores` is an ancestor of
  `phase29b-signal-runtime-store`.
- `phase29c-review-followups` is an ancestor of
  `phase29c-filldelta-review` and `phase29c-lifecycle-review`.
- `phase29c-filldelta-review` is an ancestor of
  `phase29c-lifecycle-review`.

## Work Stream Details

### fix

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `fix/cli-fail-closed-next` | 1 | 2 | `21a84e716d18` | Single fail-closed fix candidate |

Primary files:

- `src/cli/handlers/account.rs`
- `tests/account_cli_validation_test.rs`

Branch-only commits:

- `21a84e7 fix: fail closed account group strategy validation`

Decision note: this is no longer cleanup work. It is a small code candidate that
should be reviewed with GitNexus impact on the touched account handler symbol
and the relevant account CLI validation tests.

### phase24b

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `phase24b-guidance` | 1 | 3 | `a0f207719202` | Guidance/docs candidate |

Branch-only commits:

- `a0f2077 docs: add phase24b monitor automation usage`

Primary files:

- `README.md`
- `docs/USER_MANUAL.md`
- `tests/repo_hygiene_test.rs`

Decision note: preserve until the current monitor automation docs are compared
against this branch. If the guidance is superseded, archive with a tag before
deleting the remote branch.

### phase27d

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `phase27d-resolver-base` | 1 | 4 | `1c625dd8b6fd` | Earlier stack step |
| `phase27d-ruletype` | 2 | 8 | `bd8da4489038` | Earlier stack step |
| `phase27d-sqlite-resolver` | 3 | 8 | `09409fac46b5` | Earlier stack step |
| `phase27d-blocklist-enforcement-core` | 4 | 8 | `64f52b377cc7` | Earlier stack step |
| `phase27d-trade-cli-wiring` | 5 | 9 | `7ec86d4a5ac1` | Strongest current candidate |

Primary files:

- `src/cli/handlers.rs`
- `src/cli/tests/risk.rs`
- `src/risk/industry.rs`
- `src/risk/industry_store.rs`
- `src/risk/mod.rs`
- `src/risk/models.rs`
- `src/risk/service.rs`
- `tests/risk_industry_test.rs`
- `tests/risk_service_test.rs`

Branch-only commits:

- `1c625dd feat: add phase27d industry resolver`
- `bd8da44 feat: add industry-blocklist rule type`
- `09409fa feat: add sqlite shenwan industry resolver`
- `64f52b3 feat: enforce industry-blocklist in buy checks`
- `7ec86d4 feat: wire trade cli to runtime risk checks`

Decision note: this group is an implementation stack, not five independent
cleanup items. The review should start from `phase27d-trade-cli-wiring` and only
then decide whether earlier stack branches are superseded.

### phase29a

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `phase29a-guidance` | 1 | 4 | `daf5b83aeb86` | Guidance/docs candidate |
| `phase29a-guidance-refresh` | 1 | 4 | `82898fa8f750` | Guidance/docs candidate |
| `phase29a-guidance-refresh2` | 1 | 4 | `c9a9f5578bf8` | Guidance/docs candidate |
| `phase29a-runtime-path` | 1 | 1 | `b6268b333e77` | Earlier stack step |
| `phase29a-runtime-store` | 2 | 6 | `21ec10a905b7` | Earlier stack step |
| `phase29a-signal-translation` | 3 | 9 | `f45b3023b824` | Earlier stack step |
| `phase29a-paper-kernel` | 4 | 12 | `c262b67cf2c6` | Earlier stack step |
| `phase29a-paper-mode` | 5 | 15 | `e17258a234ae` | Strongest current candidate |

Primary files:

- `README.md`
- `docs/QUICKSTART.md`
- `docs/USER_MANUAL.md`
- `src/cli/handlers.rs`
- `src/cli/tests/mod.rs`
- `src/cli/tests/strategy.rs`
- `src/core/runtime.rs`
- `src/execution/adapter.rs`
- `src/execution/kernel.rs`
- `src/execution/mod.rs`
- `src/execution/models.rs`
- `src/execution/paper.rs`
- `src/execution/runtime_store.rs`
- `src/lib.rs`
- `src/strategy/mod.rs`
- `src/strategy/runtime.rs`
- `tests/execution_kernel_test.rs`
- `tests/execution_runtime_store_test.rs`
- `tests/repo_hygiene_test.rs`

Branch-only commits:

- `b6268b3 feat: add strategy runtime config paths`
- `21ec10a feat: add phase29a execution runtime store`
- `f45b302 feat: add phase29a strategy signal translation`
- `c262b67 feat: add phase29a paper execution kernel`
- `e17258a feat: wire phase29a strategy paper mode`
- `daf5b83 docs: add phase29a strategy paper guidance`
- `82898fa docs: add phase29a strategy paper guidance`
- `c9a9f55 docs: add phase29a strategy paper guidance`

Decision note: review `phase29a-paper-mode` as the implementation candidate and
compare the three guidance branches against the current docs. Do not spend more
cleanup time on individual earlier stack branches until this group-level
decision is made.

### phase29b

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `phase29b-guidance` | 1 | 5 | `60283d84c4d7` | Guidance/docs candidate |
| `phase29b-guidance-refresh` | 1 | 5 | `d9aff5ac1c24` | Guidance/docs candidate |
| `phase29b-guidance-refresh2` | 1 | 5 | `928ee0baced2` | Guidance/docs candidate |
| `phase29b-config-stores` | 6 | 18 | `ac5d31cc0bbf` | Earlier stack step |
| `phase29b-signal-runtime-store` | 7 | 18 | `e06a9fd2cfa2` | Strongest current candidate |

Primary files:

- `README.md`
- `docs/QUICKSTART.md`
- `docs/USER_MANUAL.md`
- `src/cli/handlers.rs`
- `src/cli/tests/mod.rs`
- `src/cli/tests/strategy.rs`
- `src/core/runtime.rs`
- `src/execution/adapter.rs`
- `src/execution/kernel.rs`
- `src/execution/mod.rs`
- `src/execution/models.rs`
- `src/execution/paper.rs`
- `src/execution/runtime_store.rs`
- `src/lib.rs`
- `src/strategy/config.rs`
- `src/strategy/mod.rs`
- `src/strategy/runtime.rs`
- `src/strategy/service_config.rs`
- `tests/monitor_systemd_test.rs`
- `tests/repo_hygiene_test.rs`
- `tests/strategy_daemon_test.rs`

Branch-only commits:

- `b6268b3 feat: add strategy runtime config paths`
- `21ec10a feat: add phase29a execution runtime store`
- `f45b302 feat: add phase29a strategy signal translation`
- `c262b67 feat: add phase29a paper execution kernel`
- `e17258a feat: wire phase29a strategy paper mode`
- `ac5d31c feat: add phase29b strategy daemon config stores`
- `e06a9fd feat: add phase29b signal runtime store`
- `60283d8 docs: add phase29b strategy daemon guidance`
- `d9aff5a docs: add phase29b strategy daemon guidance`
- `928ee0b docs: add phase29b strategy daemon guidance`

Decision note: review `phase29b-signal-runtime-store` as the implementation
candidate and compare the three guidance branches against the current docs.
The implementation branch inherits the `phase29a` paper execution stack, so the
review must account for both `phase29a` runtime behavior and `phase29b`
daemon/config behavior. Earlier runtime/config branches should not be deleted
until that review proves they are fully superseded.

### phase29c

| Branch | Branch-only commits | Diff files | Tip | Role |
| --- | ---: | ---: | --- | --- |
| `phase29c-review-followups` | 1 | 1 | `e76b4a9d3a5b` | Earlier design-review step |
| `phase29c-filldelta-review` | 2 | 1 | `6a15c21097ad` | Earlier design-review step |
| `phase29c-lifecycle-review` | 3 | 1 | `433367127ed5` | Most complete design-review chain |

Primary file:

- `docs/superpowers/specs/2026-03-22-phase29c-mock-live-execution-foundation-design.md`

Branch-only commits:

- `e76b4a9 docs: refine phase29c design review follow-ups`
- `6a15c21 docs: refine phase29c fill delta accounting design`
- `4333671 docs: refine phase29c mock live execution design`

Decision note: this is a design-document chain. Start from
`phase29c-lifecycle-review`, then determine whether the two earlier review
branches can be archived as superseded.

## Verification Evidence

- Current board after pruning: `origin/master` plus 23 non-master remote
  branches.
- Open PR list at triage start: empty.
- Branch classifications used `git rev-list --left-right --count
  master...origin/<branch>` and `git diff --name-only $(git merge-base master
  origin/<branch>) origin/<branch>`.
- Same-group containment used `git merge-base --is-ancestor`.

## Stop Rule

This line should stop expanding cleanup work after this report and PR. The
remaining work is not branch cleanup; it is roadmap triage. A future stage may
choose one group at a time for merge/supersession review, but only with fresh
GitNexus impact, tests, and archive tags before any live remote branch deletion.
