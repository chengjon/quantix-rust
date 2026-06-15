<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **quantix-rust** (16377 symbols, 31865 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

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

# Project Coding Standards

> Full reference: [`docs/RUST_CODING_STANDARDS.md`](docs/RUST_CODING_STANDARDS.md)

## Mandatory Rules

### File Size Limits

| File Type | Warn | Force Split |
|-----------|------|-------------|
| `.rs` module file | > 500 lines | > 800 lines |
| `lib.rs` / `main.rs` | > 100 lines | > 150 lines |
| `handlers.rs` (CLI/routing) | > 800 lines | > 1200 lines |

`mod.rs` must only contain `pub mod` declarations and `pub use` re-exports. No business logic.

### Error Handling

- **FORBIDDEN**: `.unwrap()`, `.expect()`, `panic!()` in production code
- Use `?` operator or `.map_err(|e| QuantixError::Other(format!("context: {}", e)))`
- Never swallow errors: `let _ = may_fail()` is forbidden; use `if let Err(e) = ... { tracing::warn!(...) }`

### Logging

- CLI output: `println!` / `eprintln!`
- Library/internal: `tracing::info!` / `warn!` / `error!`
- **FORBIDDEN**: `println!` in library modules (use tracing)

### Types & Traits

- Public structs/enums: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Public API: explicit return type annotations required
- Traits in dedicated files (`provider.rs`, `adapter.rs`)
- Async traits: use `#[async_trait]`

### Module Dependencies

Direction: `cli -> service -> provider/adapter -> domain -> core`. No cycles. No bidirectional deps.

### Commit Format

```
<type>(<scope>): <subject>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

Types: `feat` | `fix` | `refactor` | `test` | `docs` | `chore` | `perf`

One commit = one semantic intent. No mixed changes.

## Quality Gates

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

All must pass before merge.

## Known Tech Debt

> Last updated 2026-06-08.

| Priority | Issue | Location | Status |
|----------|-------|----------|--------|
| ~~CRITICAL~~ | `handlers.rs` at 11K+ lines | `src/cli/handlers.rs` | ✅ Split into `src/cli/handlers/*.rs` module directory |
| ~~CRITICAL~~ | `cli/mod.rs` at 2K+ lines | `src/cli/mod.rs` | ✅ Extracted to `src/cli/commands/*.rs` (now 27 lines) |
| ~~HIGH~~ | 715 `.unwrap()` calls | Multiple files | ✅ Replaced with `?` / `map_err` (now 0) |
| ~~WARN~~ | 12+ TODO comments | Multiple files | ✅ All resolved (0 remaining) |
| ~~HIGH~~ | `println!` in library modules | `monitoring/`, `anomaly/` | ✅ Already 0 in library code |
| ~~HIGH~~ | 595 clippy warnings (workspace) | Multiple files | ✅ All resolved (0 warnings) |

## Agent skills

### Issue tracker

GitHub Issues on `chengjon/quantix-rust`. See `docs/agents/issue-tracker.md`.

### Triage labels

Uses `type:*` (bug/enhancement) and `status:*` (needs-triage/needs-info/ready-for-agent/ready-for-human/wontfix) prefixed labels. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout using existing governance docs (FUNCTION_TREE.md, audit reports) as domain context. See `docs/agents/domain.md`.
