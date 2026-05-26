---
description: "Refresh the root FUNCTION_TREE.md document"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs doc --root "$(git rev-parse --show-toplevel)"
```

The helper backs up an existing `FUNCTION_TREE.md` before updating it. Do not
hand-edit the generated section.
