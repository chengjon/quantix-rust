# OpenStock 数据消费 P0.8f — live shadow validation

日期：2026-06-28
分支：`feat/openstock-p0-8f-live-shadow-validation`
FUNCTION_TREE 节点：`P0.8f: OpenStock live shadow validation`（status: approved-for-implementation）

## 1. 目标

将 P0.8e shadow validation 设计门禁推进为第一条可执行 shadow validation。仅消费「通过外部捕获的 OpenStock `/data/bars` 真实响应」，产出 dry-run report，不写 ClickHouse、不替换生产数据源路由、不触碰 qmt_live/miniQMT。

## 2. 边界

授权路径（见 `.governance/programs/project-governance/cards/P0.8f.yaml`）：

- `src/sources/openstock.rs`
- `src/cli/commands/data.rs`
- `src/cli/handlers/openstock_handler.rs` / `app_shell.rs` / `mod.rs`
- `tests/openstock_live_shadow_validation_test.rs`
- `tests/openstock_live_validation_cli_test.rs`
- `openspec/changes/openstock-data-consumption-p0-8/tasks.md`
- `README.md`、`CHANGELOG.md`、`FUNCTION_TREE.md`
- 本报告文件

明令禁止（节选自 `non_goals`）：

- 不写 ClickHouse；不替换生产数据源路由
- 不触碰 qmt_live / miniQMT / ExecutionAdapter / OrderStatus
- 默认 CI 测试不发起 live OpenStock 网络请求
- 不把 API Key 入仓
- 不恢复 `.unwrap()` 清理

## 3. 实测线上证据（首轮与第二轮安全探测）

来源：P0.8f 节点 evidence baseline（`nodes.json` P0.8f 条目）。

- 服务地址：`http://192.168.123.109:8000`，`/health/live` 与 `/health/ready` 返回 200
- 业务接口（`/sources`、`/data/bars`、`/metrics` 等）一律要求 `X-API-Key`；缺失或非法时统一返回 401
- `/data/bars` POST，请求体含 `symbol + period + start + end` 或 `code + period + start_date + end_date`，正常返回 100 条 `KLINES` 记录
- 返回字段：`symbol/time/open/high/low/close/volume/amount/period`（与 P0.8b fixture 的 `code/date/...` 不同）
- 关键缺陷：服务端**不**按请求 `start/end/limit` 裁剪响应；返回近 100 条日线，存在请求日期与响应日期范围偏差
- 结论：客户端必须做请求/响应日期与条数漂移检测，并做本地 fail-closed 校验

第二轮探测采用 stdin 静默读取 `X-API-Key` 注入环境变量，密钥不落地文件、不写入命令行历史。

## 4. 设计

### 4.1 数据契约

`/data/bars` 响应 envelope（live）：

```json
{
  "provider": "openstock",
  "period": "daily",
  "adjust_type": "none",
  "records": [
    {
      "symbol": "600000",
      "time": "2026-06-22",
      "open": "9.80", "high": "10.15", "low": "9.70", "close": "10.05",
      "volume": 1234567, "amount": "12345678.90",
      "period": "daily"
    }
  ]
}
```

- `symbol/time/period` 为字符串
- `volume` 为整数；其余数值字段接受字符串或 JSON 数值
- `adjust_type` 缺省视为 `"none"`

### 4.2 核心 API（`src/sources/openstock.rs`）

```rust
pub struct LiveShadowRequest {
    pub symbol: String,
    pub period: String,
    pub start_date: String,      // YYYY-MM-DD
    pub end_date: String,        // YYYY-MM-DD
    pub limit: Option<u32>,
}

pub enum LiveShadowStatus { Ok, Drift, FailClosed }

pub struct LiveShadowReport {
    pub dry_run: bool,                                // 恒为 true
    pub source: &'static str,                          // "openstock_live_shadow"
    pub status: LiveShadowStatus,
    pub record_count: usize,
    pub mapped_count: usize,
    pub symbol: Option<String>,
    pub period: Option<String>,
    pub received_date_range: Option<(NaiveDate, NaiveDate)>,
    pub drifts: Vec<LiveShadowDrift>,
    pub fail_closed_errors: Vec<OpenStockKlineParseError>,
}

pub fn validate_live_shadow_payload(
    raw: &str,
    request: &LiveShadowRequest,
) -> Result<LiveShadowReport, OpenStockKlineParseError>;
```

### 4.3 fail-closed 与 drift 语义

- **fail-closed**：任何记录解析失败（缺字段、日期格式非法、价格字段非法、`high < low`、记录 `period` 非 daily、`symbol` 与请求不符）被收集进 `fail_closed_errors`，不中止整批；只要存在 fail-closed 错误，`status = FailClosed`。
- **drift**（仅在零 fail-closed 时计入）：
  - `received_count_exceeds_limit`：返回条数 > 请求 `limit`
  - `out_of_requested_window`：返回的 `[min_date, max_date]` 超出请求 `[start, end]`
- `Ok`：零 fail-closed、零 drift

### 4.4 CLI 接线

```
quantix data openstock validate-live \
  --payload <file|-> \      # `-` 从 stdin 读
  --symbol 600000 \
  --period daily \
  --start 2026-06-22 \
  --end 2026-06-23 \
  [--limit N]
```

- 不发起网络请求；`--payload` 期望操作员通过外部捕获得到的响应文件
- 失败模式：`fail_closed` 状态本身不退出码 ≠ 0（报告仍可打印）；只有 envelope 非法 / 文件读失败才报错退出

## 5. 测试

| 测试文件 | 用例数 | 说明 |
|---|---|---|
| `tests/openstock_live_shadow_validation_test.rs` | 10 | 单元层：valid 映射、limit drift、out-of-window drift、缺 `symbol`、坏 `time`、非 daily `period`、symbol 不匹配、envelope 非法、空 envelope、`Display` 实现 |
| `tests/openstock_live_validation_cli_test.rs` | 4 | CLI 层：valid 报告、limit drift 报告、fail-closed 报告、文件读失败退出 |

TDD 流程：先写 RED，验证 `validate_live_shadow_payload` 等符号未解析后再实现 GREEN。

## 6. 质量门禁

- `cargo fmt --check`：通过
- `cargo clippy --all-targets -- -D warnings`：通过
- `cargo test`：1358 passed, 6 ignored
- `git diff --check`：通过
- GitNexus `impact(Kline, upstream)`：CRITICAL，但本切片**只读消费** `Kline`，未修改 hub 符号本身
- GitNexus `detect_changes`：`risk: high`，但触发点全部为既有 `run_data_command` 的 cross_community 路由流，与既有 `ValidateFixture` 分支等价，不触达 ClickHouse / qmt_live；新增 `LiveShadow*` 符号尚未入索引（陈旧索引），符合预期

## 7. 风险与边界

- 本切片仅 dry-run 解析与报告；未触及任何持久化路径
- 服务端不裁剪 `start/end/limit` 的缺陷已被 drift 检测编码化，等价结论已在 evidence baseline 记录
- API Key 始终由操作员通过环境变量提供，本切片代码不含任何密钥读取或网络调用
- 陈旧 GitNexus 索引：`bfc68c8` → 当前 `042359d`，新增符号未入索引，按 P0.8e 既定方案后续重建

## 8. 下一步

- P0.8g：opt-in shadow persistence 设计（先做 design gate，再考虑实现）
- 启动条件：本切片合并后、dry-run 报告能稳定描述将写入的数据与 dedup key、GitNexus impact 确认 LOW/MEDIUM 且边界清晰

## 9. Graphiti ingest

- 本报告作为本地 backfill 凭证；Graphiti 可用时尝试 episode 写入，ingest 未达 `completed` 前不宣称 Graphiti 闭合
