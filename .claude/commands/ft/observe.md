---
description: "Record evidence for a function-tree node"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`. Interpret the user
arguments as `<program> <node-id> --evidence <path-or-note> [--kind <kind>]
[--note <text>]`; default `program` to `project-governance` when omitted and
unambiguous.

Run `observe` with `--root "$(git rev-parse --show-toplevel)"`. Do not
authorize or edit source code as part of observation unless the user explicitly
asks for the next gate.
