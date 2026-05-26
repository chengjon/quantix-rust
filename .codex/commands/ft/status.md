---
description: "Show local function-tree governance status"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs status --root "$(git rev-parse --show-toplevel)"
```

Summarize the program list and active gate count. Treat any extra user text as
context for the status summary.
