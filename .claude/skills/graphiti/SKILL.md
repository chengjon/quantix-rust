---
name: graphiti
description: "Use when you need project memory in quantix-rust: prior design rationale, review conclusions, debug history, handoff context, or terminology decisions."
---

# Graphiti Memory

Use Graphiti semantic memory for project history and conclusion-oriented retrieval in `quantix-rust`.

## MCP Server

- Server name: `graphiti-memory`
- Endpoint: `http://192.168.123.104:8011/mcp`

## Primary Tools

- `get_status`: verify the Graphiti MCP server is healthy
- `search_nodes`: search project entities, norms, terminology, and summaries
- `search_memory_facts`: search relationships, rationale, review conclusions, and debug findings
- `get_episodes`: list recent memory entries for a group
- `add_memory`: write a compact conclusion-oriented memory
- `get_ingest_status`: confirm a write finished processing

## Group IDs

- `quantix_rust_main`: architecture decisions and implementation conclusions
- `quantix_rust_review`: review findings, acceptance or rejection rationale, residual risks
- `quantix_rust_debug`: symptom, root cause, fix path, verification result
- `quantix_rust_handoff`: pause points and handoff context
- `quantix_rust_docs`: naming, terminology, and documentation rules

## Retrieval Patterns

Use these combinations by default:

- Design intent: `search_memory_facts` on `quantix_rust_main`
- Project norms or terminology: `search_nodes` on `quantix_rust_docs`, then `search_memory_facts` if needed
- Review history: `search_memory_facts` on `quantix_rust_review`
- Debug history: `search_memory_facts` on `quantix_rust_debug`
- Handoff or resume: `get_episodes` and `search_memory_facts` on `quantix_rust_handoff`

## Recommended Flow

1. `get_status`
2. `search_nodes` or `search_memory_facts` with the right `group_ids`
3. If writing a memory: `add_memory`
4. Capture `episode_uuid`
5. Poll `get_ingest_status` until status is `completed`

## Example Queries

- "Why was this boundary chosen?" -> `search_memory_facts` in `quantix_rust_main`
- "What did the last review conclude?" -> `search_memory_facts` in `quantix_rust_review`
- "How was this bug diagnosed before?" -> `search_memory_facts` in `quantix_rust_debug`
- "What naming rule do we follow here?" -> `search_nodes` in `quantix_rust_docs`

## Notes

- Use Graphiti for historical rationale and condensed conclusions, not for current code truth.
- Do not use legacy names such as `search_memory` or `search_facts`; the current tool is `search_memory_facts`.
- After writes, do not assume the memory is searchable until `get_ingest_status` reports `completed`.
