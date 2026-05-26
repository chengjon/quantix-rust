# Function-Tree Governance

This directory is the local function-tree workspace for Quantix. It records
governance state for `FUNCTION_TREE.md` work and project-scoped authorization
before implementation.

## Quick Use

When a task mentions `FUNCTION_TREE.md`, `/ft`, function-tree, scope
authorization, active gates, or closeout:

1. Load the public skill `myskills:function-tree`.
2. Read this file and `.governance/profile.md`.
3. Prefer the project-local slash commands when available:

   - Claude Code: `.claude/commands/ft/*.md`
   - Codex: `.codex/commands/ft/*.md`

   These map to `/ft:status`, `/ft:gate`, `/ft:new-node`, `/ft:observe`,
   `/ft:authorize`, `/ft:transition`, `/ft:implement`, `/ft:closeout`,
   `/ft:doc`, `/ft:init`, `/ft:install-guard`, and `/ft:repair`.

4. Check current state:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs status --root /opt/claude/quantix-rust
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs gate --verbose --root /opt/claude/quantix-rust
   ```

5. If no suitable active node exists, create one:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs new-node project-governance QX.Y \
     --title "Short task title" \
     --ref project/root/task-ref \
     --root /opt/claude/quantix-rust
   ```

6. Record evidence before authorization:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs observe project-governance QX.Y \
     --evidence path/or/note \
     --kind baseline \
     --note "What was observed" \
     --root /opt/claude/quantix-rust
   ```

7. Authorize a narrow scope before source edits:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs authorize project-governance QX.Y \
     --allowed path/or/glob \
     --non-goal "What must stay out of scope" \
     --commit-gate "Verification required before commit" \
     --closeout-gate "Verification required before closing" \
     --root /opt/claude/quantix-rust
   ```

8. Move through legal implementation states:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs transition project-governance QX.Y --to approved-for-implementation --root /opt/claude/quantix-rust
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs transition project-governance QX.Y --to implementation-ready --root /opt/claude/quantix-rust
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs transition project-governance QX.Y --to implementation-landed --root /opt/claude/quantix-rust
   ```

9. Close the node after verification:

   ```bash
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs closeout project-governance QX.Y \
     --summary "What landed" \
     --gate "Verification evidence" \
     --root /opt/claude/quantix-rust

   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs transition project-governance QX.Y --to closed --root /opt/claude/quantix-rust
   node /root/.codex/skills/function-tree/scripts/ft-governance.cjs doc --root /opt/claude/quantix-rust
   ```

## Local Files

- `profile.md` - Quantix-specific gates and evidence rules.
- `active-gates.json` - machine-readable active gates.
- `active-gates.md` - generated active gate summary.
- `programs/project-governance/tree.md` - program overview.
- `programs/project-governance/nodes.json` - node state.
- `programs/project-governance/cards/*.yaml` - generated task cards.
- `guards/ft-scope-check.sh` - hook wrapper for scope checks.
- `backups/FUNCTION_TREE.*.md` - backups created before doc refreshes.

## Rules Of Thumb

- Keep `FUNCTION_TREE.md` as the only feature status registry.
- Keep Quantix-specific requirements in `.governance/profile.md`, not in the
  public function-tree skill.
- Use narrow `allowed_paths`; do not open broad source-edit authorization for
  governance-only work.
- For source-code changes, include the GitNexus and runtime gates from
  `.governance/profile.md`.
- For docs, naming, design, review, debug, or handoff conclusions, write the
  required Graphiti memory and verify ingest.
- Do not hand-edit generated active gate markdown. Change node state through
  the helper and let it sync generated files.
