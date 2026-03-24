# Graphiti Upstream Rate Limit Recommendations

## Purpose

本文件不是给 `quantix-rust` runtime 用的，而是给 `graphiti-mcp` 上游维护者的修改建议。

目标：

- 降低 `add_memory -> get_ingest_status` 在当前 NAS Graphiti 部署中的 `429` 失败率
- 提高 `quantix_rust_handoff` / `quantix_rust_docs` 这类工作流记忆的落图成功率
- 让 Graphiti 在 MCP 侧对上游限流更稳，而不是只靠用户多次重试

当前现象：

- MCP 服务健康检查正常
- `graphiti-memory` 工具可用
- episode 能进入 `queued` / `processing`
- 但多次在 ingest 阶段失败，`last_error` 为上游 `429`

这说明当前问题主要不在 Neo4j，也不在 MCP transport，而在：

- provider 速率限制
- episode 处理并发过高
- LLM 与 embedder 共用上游配额
- queue 重试只会“再撞一次”，不会主动整体降载

## Current Evidence

### 1. Current NAS config is aggressive for the observed upstream

File:

- `/opt/claude/graphiti/mcp_server/.env.nas`

Observed:

```env
SEMAPHORE_LIMIT=10
LLM_PROVIDER=anthropic
LLM_MODEL=glm-5
ANTHROPIC_API_URL=https://open.bigmodel.cn/api/anthropic

OPENAI_API_URL=https://open.bigmodel.cn/api/paas/v4
EMBEDDER_PROVIDER=openai
EMBEDDER_OPENAI_API_URL=https://open.bigmodel.cn/api/paas/v4
EMBEDDER_MODEL=text_embedding
```

Risk:

- `SEMAPHORE_LIMIT=10` may be too high for the current GLM-compatible quota
- LLM and embedder both hit the same provider family and likely the same account budget
- one episode causes multiple upstream requests, so effective request pressure is higher than the raw queue concurrency suggests

### 2. Queue retries exist but do not reduce system-wide pressure

File:

- `/opt/claude/graphiti/mcp_server/src/services/queue_service.py`

Observed defaults:

```python
max_retries = 2
retry_base_delay_seconds = 5.0
retry_max_delay_seconds = 60.0
retry_jitter_seconds = 1.0
```

Current behavior:

- rate-limit failures are treated as retriable
- the same task sleeps, then retries
- but there is no global cooldown after repeated `429`
- there is no provider-aware automatic throttling

### 3. Graphiti server itself already documents that high concurrency causes 429

File:

- `/opt/claude/graphiti/mcp_server/src/graphiti_mcp_server.py`

Observed guidance:

- too high `SEMAPHORE_LIMIT` leads to `429` and increased costs
- default `10` is only appropriate for stronger quotas

This matches the observed failure mode in practice.

## Recommended Changes

## Priority 1: Configuration-only changes

These should be attempted before changing Python code.

### Change 1: Lower `SEMAPHORE_LIMIT`

File:

- `/opt/claude/graphiti/mcp_server/.env.nas`

Recommendation:

```env
SEMAPHORE_LIMIT=2
```

If `429` still appears:

```env
SEMAPHORE_LIMIT=1
```

Reason:

- current workload is interactive AI memory ingestion, not bulk offline indexing
- one episode may trigger multiple internal LLM calls
- reducing concurrency is the fastest and lowest-risk way to stop `429`

### Change 2: Split embedder away from the GLM upstream

Files:

- `/opt/claude/graphiti/mcp_server/.env.nas`
- optional reference: `/opt/claude/graphiti/docs/graphiti-mcp-guide.md`

Recommendation:

Keep LLM on GLM, but move embeddings to a separate endpoint, ideally local Ollama or any independent embedding service.

Recommended shape:

```env
LLM_PROVIDER=anthropic
LLM_MODEL=glm-5
ANTHROPIC_API_URL=https://open.bigmodel.cn/api/anthropic

EMBEDDER_PROVIDER=openai
EMBEDDER_OPENAI_API_KEY=ollama
EMBEDDER_OPENAI_API_URL=http://192.168.123.74:11434/v1
EMBEDDER_MODEL=qwen3-embedding:0.6b
EMBEDDER_DIMENSIONS=1024
```

Reason:

- currently both semantic extraction and embedding appear to draw on the same provider budget
- separating them usually reduces `429` frequency significantly

### Change 3: Rebuild and validate with a single-episode probe

After `.env.nas` changes:

```bash
cd /volume5/docker5/graphiti/mcp_server
docker compose --env-file .env.nas -f docker/docker-compose-neo4j-external.yml up -d --build
```

Then test with one light write instead of many backfills.

Success criteria:

- `get_ingest_status` reaches `completed`
- no `429` in logs during the probe

## Priority 2: Code changes in `graphiti-mcp`

These are worth doing if config-only changes are not enough.

### Change 4: Make queue retry tuning configurable via env vars

Files:

- `/opt/claude/graphiti/mcp_server/src/services/queue_service.py`
- `/opt/claude/graphiti/mcp_server/src/graphiti_mcp_server.py`

Problem:

- retry behavior exists, but is hardcoded
- operators cannot quickly adjust retry strategy for different providers

Recommendation:

Expose the following env vars and pass them into `QueueService(...)`:

```env
GRAPHITI_MAX_RETRIES=4
GRAPHITI_RETRY_BASE_DELAY_SECONDS=15
GRAPHITI_RETRY_MAX_DELAY_SECONDS=180
GRAPHITI_RETRY_JITTER_SECONDS=3
```

Suggested implementation shape:

```python
queue_service = QueueService(
    max_retries=int(os.getenv("GRAPHITI_MAX_RETRIES", 2)),
    retry_base_delay_seconds=float(os.getenv("GRAPHITI_RETRY_BASE_DELAY_SECONDS", 5)),
    retry_max_delay_seconds=float(os.getenv("GRAPHITI_RETRY_MAX_DELAY_SECONDS", 60)),
    retry_jitter_seconds=float(os.getenv("GRAPHITI_RETRY_JITTER_SECONDS", 1)),
)
```

Reason:

- current 5s base delay is often too short for provider-side rate-limit recovery windows
- making it configurable avoids code edits for every deployment experiment

### Change 5: Add a global cooldown after repeated `429`

File:

- `/opt/claude/graphiti/mcp_server/src/services/queue_service.py`

Problem:

- current retry only delays the failed episode
- other queued episodes may continue to hit the provider immediately afterward
- this creates a thundering-herd pattern under sustained rate limiting

Recommendation:

Introduce a simple global or per-provider cooldown window:

- when a rate-limit error occurs
- record `next_global_retry_at`
- before processing new episodes, wait until that timestamp passes

Minimal behavior is enough:

- first `429`: sleep 15-30s globally
- repeated `429`: exponential increase up to a cap

Reason:

- this reduces repeated collision with provider limits
- it is more effective than only retrying individual tasks

### Change 6: Improve ingest status observability

Files:

- `/opt/claude/graphiti/mcp_server/src/services/queue_service.py`
- `/opt/claude/graphiti/mcp_server/src/models/response_types.py`

Recommendation:

Expose more detail when a task is in retry flow:

- `attempt_count`
- `last_error_code`
- `next_retry_at`

If these already exist internally, ensure they are consistently surfaced through `get_ingest_status`.

Reason:

- current users can see `failed`, but not always whether the system is still cooling down or retrying
- better visibility reduces blind repeated writes from MCP clients

## Priority 3: Operational guidance

### Change 7: Document provider-specific safe defaults

Files:

- `/opt/claude/graphiti/docs/graphiti-mcp-guide.md`
- `/opt/claude/graphiti/docs/GRAPHITI_MCP_WORKFLOW.md`

Recommendation:

Add a concrete note for GLM/OpenAI-compatible paths:

- if using GLM-compatible upstreams with unknown or lower RPM ceilings
- start with `SEMAPHORE_LIMIT=1-2`
- only scale up after observing stable ingest completion

Reason:

- current docs mention concurrency tuning in general
- but they do not give a concrete “safe starting point” for this deployment pattern

### Change 8: Add a rate-limit playbook

Files:

- `/opt/claude/graphiti/docs/GRAPHITI_NAS_RUNBOOK.md`

Recommendation:

Add a short runbook section:

1. detect repeated `429`
2. lower `SEMAPHORE_LIMIT`
3. split embedder away from LLM provider
4. rebuild service
5. verify with a single probe episode

Reason:

- this failure mode is now known and reproducible
- it should be an explicit operational recipe, not tribal knowledge

## Suggested Implementation Order

1. Lower `SEMAPHORE_LIMIT` to `2`
2. Move embedder to an independent endpoint
3. Rebuild and test one probe write
4. If `429` remains, expose retry tuning via env vars
5. Add global cooldown on repeated `429`
6. Improve `get_ingest_status` observability
7. Update docs/runbook

## Minimal First Patch

If the maintainer only wants the smallest safe first patch, I recommend:

1. Edit `.env.nas`
   - `SEMAPHORE_LIMIT=2`
   - move `EMBEDDER_*` to Ollama or another separate embedding provider
2. Restart `graphiti-mcp`
3. Verify with one MCP write

This is the highest-leverage change with the lowest implementation risk.
