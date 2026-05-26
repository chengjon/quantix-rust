---
description: "Install the local function-tree scope guard wrapper"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`, then run:

```bash
node /root/.codex/skills/function-tree/scripts/ft-governance.cjs install-guard --root "$(git rev-parse --show-toplevel)"
```

Report the installed path and any hook snippet. Do not edit global agent config
unless the user explicitly requests it.
