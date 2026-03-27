# Graphiti Memory Skill

Interact with Graphiti semantic memory layer for design decisions, code reviews, debugging, handoffs, and documentation.

## Usage

```
/graphiti <action> [options]
```

## Actions

| Action | Description | Example |
|--------|-------------|---------|
| `write` / `add` / `save` | Write a memory entry | `/graphiti write 设计决策：采用事件驱动架构` |
| `read` / `search` / `find` | Search memories | `/graphiti search 多账户管理` |
| `list` | List recent memories | `/graphiti list --group main` |
| `groups` | Show available groups | `/graphiti groups` |
| `health` | Check service status | `/graphiti health` |

## Group IDs

| Group ID | Alias | Purpose |
|----------|-------|---------|
| `quantix_rust_main` | `main` | 主设计决策和架构记录 |
| `quantix_rust_main_review` | `review` | 代码审查记录 |
| `quantix_rust_main_debug` | `debug` | 调试会话记录 |
| `quantix_rust_main_handoff` | `handoff` | 交接文档 |
| `quantix_rust_main_docs` | `docs` | 技术文档 |

## Options

- `--group <name>` / `-g <name>`: Specify group (default: main)
- `--date <YYYY-MM-DD>`: Filter by date
- `--limit <n>`: Limit results (default: 10)

## Examples

```bash
# 写入设计决策
/graphiti write 采用 AccountRouter 实现多账户智能路由，支持 Equal/Proportional/Weighted/PrimaryFirst 四种分配策略

# 写入代码审查
/graphiti write --group review 审查 notification.rs:623 修复了 borrow after move 错误

# 搜索记忆
/graphiti search 多账户

# 查看调试记录
/graphiti list --group debug

# 检查服务状态
/graphiti health
```

## Implementation

When invoked, use the Graphiti MCP via SSE connection:

**MCP Server**: `graphiti-memory`
**SSE URL**: `http://192.168.123.104:8011/mcp`

**Note**: Graphiti uses MCP protocol over SSE, not REST API. To use directly:

1. Ensure Graphiti MCP is connected (check with `/mcp` command)
2. Use MCP tools: `mcp__graphiti__add_memory`, `mcp__graphiti__search_memory`, etc.

**Health Check**:
```bash
curl http://192.168.123.104:8011/health
# Returns: {"status":"healthy","service":"graphiti-mcp"}
```

## Notes

- The Graphiti MCP server runs on NAS (192.168.123.104:8011)
- Memories are persisted in the Graphiti knowledge graph
- Use semantic search to find related memories
- Group IDs organize memories by purpose
