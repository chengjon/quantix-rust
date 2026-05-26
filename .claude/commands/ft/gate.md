---
description: "Show active function-tree gates and next allowed action"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs gate --verbose --root "$(git rev-parse --show-toplevel)"
```

Report every active gate, its status, allowed paths, blockers, and next action.
