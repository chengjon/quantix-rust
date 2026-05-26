---
description: "Check whether current edits fit active function-tree authorization"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs scope-check --root "$(git rev-parse --show-toplevel)"
```

If the check fails, stop and report the violating paths. If it passes, continue
only within the active node's allowed paths and gates.
