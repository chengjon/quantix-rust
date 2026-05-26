---
description: "Create a new function-tree governance node"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` and `.governance/profile.md`. Interpret the user
arguments as `<program> <node-id> --title <text> --ref <ref>`; default
`program` to `project-governance` when omitted and the intent is clearly for
this repo.

Run the helper with the user's arguments and `--root "$(git rev-parse
--show-toplevel)"`. If the title or ref is missing, ask for the missing value
instead of guessing.
