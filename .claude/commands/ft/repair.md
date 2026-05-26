---
description: "Repair generated function-tree active gate files"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs repair --root "$(git rev-parse --show-toplevel)"
```

Use this only to rebuild active gates from node state and drop closed or
archived gates from active gate files.
