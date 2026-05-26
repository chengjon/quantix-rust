---
description: "Transition a function-tree node through the legal state machine"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`. Interpret the user
arguments as `<program> <node-id> --to <status>`; default `program` to
`project-governance` when omitted and unambiguous.

Run `transition` with `--root "$(git rev-parse --show-toplevel)"`. If moving
to `approved-for-implementation`, first confirm evidence is current and
authorization has allowed paths, non-goals, commit gates, and closeout gates.
