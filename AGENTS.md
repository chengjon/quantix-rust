<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **quantix-rust** (16438 symbols, 31251 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> Index stale? Run `node .gitnexus/run.cjs analyze` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? `npx gitnexus analyze` (npm 11 crash → `npm i -g gitnexus`; #1939).

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows. For regression review, compare against the default branch: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})`.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/quantix-rust/context` | Codebase overview, check index freshness |
| `gitnexus://repo/quantix-rust/clusters` | All functional areas |
| `gitnexus://repo/quantix-rust/processes` | All execution flows |
| `gitnexus://repo/quantix-rust/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->

# Graphiti MCP — Semantic Memory

This project uses Graphiti MCP as a mandatory semantic-memory workflow layer for AI CLI development. It is not a task-status system, not a code truth source, and not a runtime dependency.

Full workflow guide:

- `docs/guides/GRAPHITI_MCP_WORKFLOW.md`

## Always Do

- **MUST attempt Graphiti reads before starting** these workflows:
  - design work: search `quantix_rust_main`, and `quantix_rust_docs` when norms or terminology may matter
  - review handling: search `quantix_rust_review`, and `quantix_rust_main` when design intent may matter
  - debugging: search `quantix_rust_debug`, and `quantix_rust_main` when the bug may relate to prior design decisions
  - handoff/resume: search `quantix_rust_handoff` first
- **MUST write a Graphiti memory after conclusions converge** for:
  - design decisions -> `quantix_rust_main`
  - review conclusions -> `quantix_rust_review`
  - debug root cause and verification -> `quantix_rust_debug`
  - pause/handoff checkpoints -> `quantix_rust_handoff`
  - documentation / terminology / naming decisions -> `quantix_rust_docs`
- **MUST verify ingest after every write**:
  1. `add_memory`
  2. capture `episode_uuid`
  3. `get_ingest_status` until `completed`
- **MUST use stable `group_id` values**:
  - `quantix_rust_main`
  - `quantix_rust_review`
  - `quantix_rust_debug`
  - `quantix_rust_handoff`
  - `quantix_rust_docs`

## Fallback Rule

- If Graphiti MCP is unavailable, unconfigured, or ingest fails, work may continue, but only if you leave an equivalent local summary and explicitly write:

```text
Graphiti backfill required
```

- After Graphiti becomes available again, that memory MUST be backfilled into the correct `group_id`.

## Never Do

- NEVER use Graphiti as the authority for current task status, approval, merge state, or ownership.
- NEVER use Graphiti instead of GitNexus for code structure, impact analysis, or call-chain understanding.
- NEVER add `graphiti-api` or `graphiti-mcp` as a runtime dependency of this repository's application code.
- NEVER treat `add_memory` success as proof that the memory is already searchable.
- NEVER dump raw long logs, full transcripts, or full diffs into Graphiti as-is; write compact conclusion-oriented summaries instead.

## Closure-Stage Gatekeeping

- **MUST switch to gate-closure priority once a task is in closure stage.** Closure stage means the main implementation is done and the remaining work is primarily validation, runtime gates, acceptance checks, docs alignment, or release-readiness checks.
- **MUST prioritize runtime gate closure over further cleanup.** Finish the task-relevant `cargo test`, `cargo clippy`, `cargo fmt --check`, integration checks, repo hygiene checks, manual verification, or equivalent gates before any additional cleanup work.
- **MUST NOT expand closure-stage work into cosmetic drift.** Do not continue with incidental renames, comment polish, output wording tweaks, mechanical warning cleanup, or opportunistic refactors before the gate loop is closed.
- **Any post-closure-stage code change must justify its gate relation.** The agent should be able to state which failing gate, blocked verification step, or acceptance gap the change is needed for.
- **If the gate loop is closed, stop expanding the change surface.** Any remaining cosmetic or opportunistic cleanup must be treated as a separate follow-up task, not part of the current task.
