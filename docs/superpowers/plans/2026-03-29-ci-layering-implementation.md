# CI Layering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split GitHub Actions CI so pull requests run only the fast quality gate, branch pushes run fuller validation, and `main` keeps the heaviest reporting jobs.

**Architecture:** Keep the existing single workflow file, but separate heavy concerns into explicit jobs with scoped `if:` conditions instead of embedding coverage and outdated checks inside broader jobs. Preserve current commands and deployment behavior where possible so the change is operationally safe and easy to review.

**Tech Stack:** GitHub Actions workflow YAML, Rust cargo commands, repo-local verification via YAML parsing and diff review

---

### Task 1: Restructure Workflow Job Boundaries

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Review the current workflow layout**

Run: `sed -n '1,360p' .github/workflows/ci.yml`
Expected: See `lint`, `test`, `security`, `build`, `bench`, and `docs` in one workflow, with coverage embedded in `test` and outdated embedded in `security`.

- [ ] **Step 2: Edit workflow structure**

Apply these changes in `.github/workflows/ci.yml`:

- keep `lint` unchanged and available to both `pull_request` and `push`
- keep `test` on both `pull_request` and `push`, but remove coverage generation from this job
- rename or split `security` so the audit portion remains on both `pull_request` and `push`
- add a dedicated `coverage` job gated to `push` on `main`
- add a dedicated `dependency_outdated` job gated to `push` on `main`
- gate `build` to `push` on `main` or `develop`
- keep `bench` on `push` to `main`
- keep `docs` generation on `push`, and keep Pages deploy only on `push` to `main`

- [ ] **Step 3: Review the edited workflow for trigger correctness**

Run: `sed -n '1,420p' .github/workflows/ci.yml`
Expected: The workflow shows explicit job-level gating that matches the spec.

### Task 2: Add Focused Verification for Job Gating

**Files:**
- Create: `tests/ci_workflow_structure_test.rs`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Write the failing test**

Create a Rust integration test that reads `.github/workflows/ci.yml` as text and asserts:

- a `coverage` job exists
- a `dependency_outdated` job exists
- `build` is gated to `push`
- `bench` remains gated to `push` on `main`
- the workflow still includes both `pull_request` and `push`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test ci_workflow_structure_test -- --nocapture`
Expected: FAIL because the current workflow does not yet contain the new split jobs and gating text.

- [ ] **Step 3: Write minimal implementation**

Finish the `.github/workflows/ci.yml` edits from Task 1 so the new structure satisfies the assertions without changing unrelated commands or cache setup.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test ci_workflow_structure_test -- --nocapture`
Expected: PASS

### Task 3: Verify YAML and Final Trigger Semantics

**Files:**
- Modify: `.github/workflows/ci.yml`
- Test: `tests/ci_workflow_structure_test.rs`

- [ ] **Step 1: Validate YAML syntax**

Run: `python - <<'PY'
import yaml, pathlib
path = pathlib.Path('.github/workflows/ci.yml')
yaml.safe_load(path.read_text())
print('yaml ok')
PY`
Expected: `yaml ok`

- [ ] **Step 2: Re-run the workflow structure test**

Run: `cargo test --test ci_workflow_structure_test -- --nocapture`
Expected: PASS

- [ ] **Step 3: Inspect the final diff**

Run: `git diff -- .github/workflows/ci.yml tests/ci_workflow_structure_test.rs`
Expected: Only the CI workflow split and its focused regression test are present.

- [ ] **Step 4: Run change-scope verification**

Run: `git status --short .github/workflows/ci.yml tests/ci_workflow_structure_test.rs`
Expected: Only the workflow file and the new workflow test appear for this task's changes.

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/ci.yml tests/ci_workflow_structure_test.rs
git commit -m "ci: layer pull request and main workflow jobs"
```
