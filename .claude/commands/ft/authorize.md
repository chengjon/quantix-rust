---
description: "Prepare function-tree authorization for a node"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`. Interpret the user
arguments as `<program> <node-id> --allowed ... --non-goal ... --commit-gate ...
--closeout-gate ...`; default `program` to `project-governance` when omitted
and unambiguous.

Before authorizing source-code work, ensure the node has evidence and include
the project-specific GitNexus, Graphiti, runtime, or closure gates required by
`.governance/profile.md`.
