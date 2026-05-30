# GitNexus MCP Workflow Follow-Ups

Status: reference notes
Date: 2026-05-31
Related guide: `docs/guides/GITNEXUS_MCP_DAILY_WORKFLOW_RECOMMENDATIONS.md`

This note is not a feature-status registry. Current feature status, evidence,
and boundaries remain governed by the root `FUNCTION_TREE.md` registry.

## Purpose

This note captures follow-up recommendations from the GitNexus MCP daily workflow guide review. The main guide has been merged. These items should be used as future reference when deciding what to observe, investigate, or promote into stronger project rules.

## Recommended Next Steps

### 1. Observe One Real Development Task

Do not keep expanding the merged guide without new evidence. Use it during the next real development task and observe the full loop:

```text
gitnexus_query/gitnexus_context -> gitnexus_impact -> edit/test -> gitnexus_detect_changes(scope: "staged") -> commit/PR
```

During that task, record whether:

- `cwd` / `worktree` examples are clear enough.
- `scope: "all"` versus `scope: "staged"` reduces false positives.
- `relationTypes + summaryOnly` gives usable signal on hub symbols.
- Non-Rust changes have enough alternative gate guidance.

The guide should be calibrated by real workflow friction, not by more theoretical edits.

### 2. Investigate `gitnexus analyze` Generated Doc Side Effects

Repeated runs of `gitnexus analyze` generated temporary modifications in:

```text
AGENTS.md
CLAUDE.md
.claude/skills/gitnexus/gitnexus-cli/SKILL.md
.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md
.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md
```

Those changes were restored because they were outside the documentation scope. This behavior should be investigated separately from the workflow guide.

Questions to answer:

- Why does `gitnexus analyze` rewrite agent-facing docs?
- Is this expected generation, migration behavior, or a bug?
- Can the behavior be disabled, isolated, or made explicit?
- Should generated updates require an explicit command instead of happening during analyze?

Suggested follow-up issue title:

```text
Investigate gitnexus analyze generated doc side effects
```

Suggested acceptance criteria:

- Running `gitnexus analyze` does not leave unrelated agent docs dirty by default, or the project documents why it does.
- If doc generation is intentional, the command or flag that performs it is explicit.
- A normal index refresh leaves `git status --short` clean when no source files changed.

### 3. Do Not Immediately Promote the Whole Guide to `AGENTS.md`

Keep `AGENTS.md` reserved for mandatory rules. Keep the guide as operational guidance until its recommendations have been tested in real tasks.

Good candidates for later promotion, after practical validation:

- Use `gitnexus_detect_changes({ scope: "staged", cwd: "<current checkout/worktree>" })` as the standard pre-commit scope gate.
- In linked worktrees, pass `cwd` or `worktree` explicitly.
- Treat `scope: "all"` as a local sanity check, not as final commit evidence.
- Pair non-Rust changes with non-GitNexus gates.

Avoid promoting untested habits into `AGENTS.md`; otherwise the mandatory rule set becomes noisy and harder to follow.

### 4. Add a Feedback Loop for the Guide

Use the guide as the first place to record GitNexus workflow friction:

```text
When a real GitNexus workflow exposes false positives, false negatives, tool side effects,
or ambiguous instructions, update the guide first. Promote only stable, repeated findings
to AGENTS.md.
```

This keeps `AGENTS.md` focused while still preserving operational learning.

### 5. Stop Expanding the Completed Documentation PR

The merged guide already covers the current review feedback. Further improvements should be scoped as separate tasks, preferably with a concrete trigger:

- A real workflow exposed ambiguity.
- A GitNexus tool behavior changed.
- The project decided to promote a recommendation into a mandatory rule.
- A follow-up issue was accepted for investigation.

## Current Recommendation

Treat the merged guide as good enough for first use. The highest-value next task is the generated-doc-side-effects investigation, followed by one real development task using the guide end to end.
