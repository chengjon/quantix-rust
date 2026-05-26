---
description: "Prepare and close out a function-tree node"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`. Interpret the user
arguments as `<program> <node-id> --summary <text-or-path> [--compatibility
<text>] [--gate <text>]...`; default `program` to `project-governance` when
omitted and unambiguous.

Run the requested closeout, verify required gates, then transition to `closed`
only when the closeout evidence is complete. Refresh `FUNCTION_TREE.md` with
`doc` after closing.
