# Scheduled Workflow Closure Report

Date: 2026-06-12

## Scope

This report closes the P0 scheduled workflow failure remediation line that followed the completed Clippy Cleanup project.

This line did not resume `.unwrap()` cleanup and did not change Rust production code. The work was limited to GitHub Actions scheduled workflow diagnosis, workflow configuration repair, and benchmark data branch preparation.

## Closed Items

### Security Audit Schedule Failure

Initial failure:

- Workflow: `Security Audit`
- Scheduled run: `27257486800`
- Head: `e5fae2fe3c7ac6722e6f8c29505ea728b0972d5f`
- Root cause: deprecated `actions/upload-artifact@v3` hard-failed during setup.

Remediation:

- PR: `#221`
- Merge commit: `74254ca575ff1a75a9b9b1fd60cd255f0e14a691`
- Main changes:
  - Upgraded `actions/upload-artifact@v3` to `actions/upload-artifact@v4`.
  - Hardened the outdated dependency audit path so runner timeout/report generation does not block the whole scheduled audit line.

Validation:

- Manual Security Audit dispatch on the PR branch passed.
- Scheduled Security Audit later passed on `74254ca`.
- Latest verified scheduled Security Audit on PR #222 head also passed:
  - Run: `27399328814`
  - Event: `schedule`
  - Head: `078c5902067469523f5cd1c605cee80a950982d7`
  - Conclusion: `success`

### CI Benchmark Schedule Failure

Initial failure:

- Workflow: `CI`
- Scheduled run: `27257612349`
- Head: `e5fae2fe3c7ac6722e6f8c29505ea728b0972d5f`
- Failed job: `Benchmark`
- Failed step: `Store benchmark result`
- Root cause: `benchmark-action/github-action-benchmark@v1` was configured with `tool: cargo` but read Criterion HTML output from `target/criterion/report/index.html`, which is not parseable cargo bench output.

Intermediate remediation:

- PR: `#221`
- Changed benchmark capture from Criterion HTML to `benchmark-output.txt`.
- This removed the HTML input mistake but did not fully close the failure because `tool: cargo` still could not parse Criterion output.

Final parser remediation:

- PR: `#222`
- Merge commit: `078c5902067469523f5cd1c605cee80a950982d7`
- Main changes:
  - Added `Convert Criterion results` step.
  - Converted `target/criterion/**/new/estimates.json` into `benchmark-output.json`.
  - Switched benchmark action from `tool: cargo` to `tool: customSmallerIsBetter`.
  - Kept the benchmark source as Criterion data while giving the action the custom JSON format it supports.

Validation:

- PR #222 checks passed for Lint/Test.
- Push CI on `master` passed:
  - Run: `27349062776`
  - Head: `078c5902067469523f5cd1c605cee80a950982d7`
  - Conclusion: `success`
- Real scheduled CI reached PR #222 head:
  - Run: `27399473557`
  - Event: `schedule`
  - Head: `078c5902067469523f5cd1c605cee80a950982d7`

The first attempt of run `27399473557` confirmed that PR #222 fixed the parser path:

- `Convert Criterion results` succeeded.
- The workflow converted 42 Criterion benchmark results.
- The remaining failure moved to `Store benchmark result` because the action tried to fetch `gh-pages`, but that branch did not exist.

## Benchmark Data Branch

The benchmark action expects a GitHub Pages branch when `auto-push: true` is used. The action documentation states that a GitHub Pages branch must be created before this mode is used.

Observed failure:

- Run: `27399473557`, attempt 1
- Failed step: `Benchmark / Store benchmark result`
- Error: `fatal: couldn't find remote ref gh-pages`

Remediation:

- Created remote branch: `gh-pages`
- Initial empty commit: `7cf843fa9cf688c0cfff2dc4dfe3b1fdecb067d4`
- No `master` code or documentation changes were made by the branch creation.

Validation:

- Reran failed jobs for run `27399473557`.
- Attempt 2 completed successfully:
  - Run: `27399473557`
  - Attempt: `2`
  - Event: `schedule`
  - Head: `078c5902067469523f5cd1c605cee80a950982d7`
  - Overall conclusion: `success`
  - `Benchmark`: `success`
  - `Test`: `success`
  - `Lint`: `success`
  - `Build`, `Coverage`, `Documentation`: skipped by workflow rules.
- The benchmark action updated `gh-pages` to:
  - `09313062c7cecac3ac7675754b8498aa99142b76`

## Current Repository State

As of the closure check:

- Branch: `master`
- Local worktree: clean
- Open PRs: `0`
- Latest scheduled CI on `master`: `27399473557`, success
- Latest scheduled Security Audit on `master`: `27399328814`, success
- Remote benchmark data branch: `gh-pages`

## Maintenance Notes

- Do not delete `gh-pages` unless benchmark history publishing is intentionally retired or the CI workflow is changed to stop using `auto-push: true`.
- If `gh-pages` is protected later, make sure the workflow token or configured credentials can still update benchmark data.
- If benchmark storage is moved away from GitHub Pages, update both:
  - `gh-pages-branch`
  - `benchmark-data-dir-path`
- Manual `workflow_dispatch` does not validate the Benchmark schedule path with the current workflow condition. The `Benchmark` job runs only on `schedule` or push to `refs/heads/main`.
- The repository's default branch is `master`, while some heavy-path workflow conditions still reference `refs/heads/main`. That behavior was not changed in this P0 line because it was outside the immediate scheduled failure root cause.

## Final Decision

The P0 scheduled workflow failure remediation line is closed.

No further action is required for PR #221 or PR #222. Any future CI behavior changes should be opened as a separate workflow governance task.
