---
description: "Initialize function-tree governance files for a repo"
---

Use the `myskills:function-tree` skill.

Read `.governance/README.md` if it exists. Interpret the user arguments as
`<program> --ref <ref> [--description <text>]`; default the program to
`project-governance` only when the user clearly wants the current repo default.

Run `init` with `--root "$(git rev-parse --show-toplevel)"`. If
`FUNCTION_TREE.md` already exists, confirm the helper will back it up before
refreshing.
