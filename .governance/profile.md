# Quantix Function-Tree Profile

This profile is project-local. It extends the public `function-tree` skill with
Quantix-specific gates, evidence expectations, and tool routing. Do not copy
these rules into the public skill.

## Scope

- Root project: `/opt/claude/quantix-rust`
- Function-tree program: `project-governance`
- Feature status source of truth: `FUNCTION_TREE.md`
- Governance state directory: `.governance/`
- Guard wrapper: `.governance/guards/ft-scope-check.sh`

## Project Rules

- Treat `FUNCTION_TREE.md` as the sole feature panorama and status registry.
- Do not create competing feature-status sources in issues, reports, or ad hoc
  notes.
- Keep project-specific governance here or in agent rules, not inside the
  public `function-tree` skill.
- Do not revert unrelated dirty worktree changes.
- Do not edit production code during audit-spec or governance-only work unless
  the user explicitly shifts the task to remediation.

## Required Context Before Authorization

Before authorizing a function-tree node that can touch source code, read or
query the project-local context relevant to the task:

- `FUNCTION_TREE.md`
- `AGENTS.md`
- `docs/agents/domain.md`
- `docs/guides/GRAPHITI_MCP_WORKFLOW.md` when memory, review, debug, handoff,
  terminology, or documentation decisions are involved
- `docs/standards/CODE_AUDIT_METHODOLOGY.md` and
  `docs/standards/MOCK_USAGE_POLICY.md` for audit or mock-related work

Use context-mode tools for broad scans, summaries, and large files. Prefer
`ctx_batch_execute`, `ctx_search`, `ctx_execute`, and `ctx_execute_file` over
raw shell output.

## GitNexus Gates

For source-code work, capture the applicable GitNexus gates in the node's
`commit_gate` or `closeout_gate` before implementation:

- Run GitNexus impact analysis before editing a function, class, method, or
  shared symbol.
- Warn before proceeding if impact analysis returns HIGH or CRITICAL risk.
- Use GitNexus query/context for unfamiliar code and process-flow discovery.
- Use GitNexus rename for renames; do not rename symbols with find-and-replace.
- Run GitNexus detect-changes before committing code changes.

These gates are not required for documentation-only, `.governance/`, or local
agent-config edits unless they also modify code symbols.

## Graphiti Gates

Use Graphiti as semantic memory, not as the source of truth for current code,
task status, merge state, ownership, or runtime behavior.

Before starting these workflows, attempt Graphiti reads:

- Design or terminology work: `quantix_rust_main`; also `quantix_rust_docs`
  when docs or naming norms matter.
- Review handling: `quantix_rust_review`; also `quantix_rust_main` when design
  intent matters.
- Debugging: `quantix_rust_debug`; also `quantix_rust_main` when prior design
  context may matter.
- Handoff or resume: `quantix_rust_handoff`.

After conclusions converge, write compact memories to the appropriate group:

- Design decisions: `quantix_rust_main`
- Review conclusions: `quantix_rust_review`
- Debug root cause and verification: `quantix_rust_debug`
- Handoff checkpoints: `quantix_rust_handoff`
- Documentation, terminology, and naming decisions: `quantix_rust_docs`

After every Graphiti write, verify ingest with `get_ingest_status` until the
episode is completed. If Graphiti is unavailable or ingest fails, leave a local
summary and mark `Graphiti backfill required`.

## Closure Gates

Once a node is in closure stage, prioritize gate closure over cleanup.

Closure stage means implementation is done and remaining work is validation,
runtime gates, acceptance checks, documentation alignment, or release-readiness
checks. During closure:

- Run task-relevant checks before cosmetic cleanup.
- Do not expand the change surface for incidental renames, wording polish, or
  opportunistic refactors.
- Any post-closure code change must be tied to a failing gate, blocked
  verification step, or acceptance gap.
- If gates are closed, treat remaining cleanup as follow-up work.

## Recommended Node Gates

For a source-code remediation node, include gates similar to:

- `gitnexus_impact` completed for every modified symbol.
- HIGH or CRITICAL impact, if any, was explicitly reported before proceeding.
- `cargo fmt --check` or scoped formatting gate completed.
- Task-relevant `cargo test`, `cargo clippy`, integration, or smoke gate
  completed or explicitly recorded as blocked with evidence.
- `gitnexus_detect_changes({scope: "all"})` or an equivalent scoped change
  check completed before commit.
- Graphiti memory written and ingest verified if the task produced a durable
  design, review, debug, docs, or handoff conclusion.

For governance-only nodes, include gates similar to:

- `function-tree validate` passes.
- `function-tree gate --verbose` shows the expected node state.
- `git diff --check -- FUNCTION_TREE.md .governance` has no output.
- Generated sections were refreshed with `function-tree doc`; durable local
  notes remain in the project-notes block.

## Hook Behavior

Project-local and Codex user-level hooks may run
`.governance/guards/ft-scope-check.sh` after edits. The guard:

- Allows `.governance/` files as governance state.
- Enforces `allowed_paths` for active source-edit authorization.
- Reports but does not block when there is no active source-edit authorization.

Because this repository may have unrelated dirty files, keep function-tree
authorizations narrow and avoid opening a source-edit gate for governance-only
changes.
