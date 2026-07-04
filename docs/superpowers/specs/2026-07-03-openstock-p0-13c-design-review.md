# 审核意见：P0.13c 多日范围查询设计文档

> 审核日期：2026-07-03
> 审核范围：`docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md` 全文
> 审核基线：HEAD（P0.13b-1/2 已合并于 `f859dea`），对照 `src/sources/openstock_client.rs:692-822`（当前 `fetch_minute_klines` + `fetch_minute_share` 实现）

---

## 总体评价

设计整体简洁——`DateOrRange` enum 单参数表达互斥（D1）、`from_cli` handler 层校验（D2）、保留 `--date` 快捷路径（D3）、wiremock-first + live-verify（D4）。非目标列表清晰，风险 R1-R5 有缓解措施。但有一条**阻断级**问题——`fetch_minute_share` 的多日范围与 `parse_minute_share` 的单 `date` 参数不兼容。

---

## 阻断级问题

### 1. `fetch_minute_share` 的范围查询无法为每条记录确定日期（**CRITICAL**）

**问题**：当前 `parse_minute_share`（P0.13b-2 设计）的签名是：

```rust
fn parse_minute_share(code: &str, raw: &RawMinuteRecord, date: NaiveDate) -> Option<MinuteShare>
```

`date` 用于将 `raw.time_minutes`（如 `"0930"`）与日期组合成 `NaiveDateTime`：

```rust
let timestamp = date.and_hms_opt(hh, mm, 0)?;
```

但 `RawMinuteRecord` **不含日期字段**——`time_minutes` 只有时分（`"0930"`），日期来自查询参数。

对于单日查询（当前 P0.13b-2 行为），`date` 是已知的 `NaiveDate`——工作正常。

对于多日范围查询（P0.13c），OpenStock 返回的 response 可能包含**跨越多日的分时点**。此时调用方传入单个 `date: NaiveDate` 无法为每条记录确定正确的日期——`parse_minute_share` 会把所有记录的 `"0930"` 都拼到同一天。

**对比**：`fetch_minute_klines`**没有这个问题**——`/data/bars` 的 `time` 字段是完整 ISO 时间戳（`"2026-07-02T09:31:00+08:00"`），日期已在其中。

**影响**：`fetch_minute_share` 的范围查询返回的 `MinuteShare` 中，所有 `timestamp` 的日期都是错误的（全为同一个日期）。

**解决方案**（二选一）：

| 方案 | 做法 | 优劣 |
|------|------|------|
| **A** | `RawMinuteRecord` 增加 `date` 字段（如 OpenStock 在每条 record 中返回） | 需要确认 OpenStock MINUTE_DATA 的范围响应是否包含日期字段 |
| **B** | `fetch_minute_share` 在 `Range` 模式下**逐日循环**发起单日请求，合并结果 | `DateOrRange::Range` 退化为 N 次单日 fetch，失去 "一次请求" 的语义，但 parser 无需改动 |

**推荐先确认方案 A**：检查 OpenStock `/data/fetch MINUTE_DATA` 在范围查询时的响应 shape——每条 record 是否带有 `date` 或 `time` 字段。如果有，则 `RawMinuteRecord` 加字段 + `parse_minute_share` 从 record 中取日期。如果没有，方案 B（逐日循环）是唯一安全路径。

**同时**：设计 §5 INV-2B 写「Vec 扁平，按 timestamp 升序」——对于 `fetch_minute_share`，如果日期解析问题不解决，这条不变量无法验证。

---

## 需要修正

### 2. `Range { start: None, end: None }` 的行为未充分定义

**位置**：设计 L50

| 调用形态 | 行为 |
|---------|------|
| `date: None, start: None, end: None` | OpenStock 默认范围（与 P0.13b-1/2 当前行为一致） |

**问题**：当前 P0.13b-1/2 的 `--date` 是**必需参数**（`String`，非 `Option`），不存在「不传 date」的当前行为。这个 case 是**全新的**，不是 "一致"。

而且 `DateOrRange::Range { start: None, end: None }` 经过 `populate_body` 后不做任何操作（不设置任何 date/start_date/end_date 字段），wire body 将缺少日期约束。OpenStock 对此的默认行为未知——可能返回全历史、可能返回当天、可能报错。

**建议**：
- 要么把这个 case 也定义为错误（`from_cli` 在 `(None, None, None)` 时返回 `Err`，要求用户至少提供一个约束）
- 要么在 R2 风险基础上追加 R6：`start=None, end=None` 的 OpenStock 默认行为未验证

---

### 3. `FetchMinuteKlines` CLI 的 `date` 从 `String` 改为 `Option<String>` 的兼容性

**问题**：当前 `FetchMinuteKlines`（`data.rs:386`）的 `date` 是 `String`（必需）。设计 L73 改为 `date: Option<String>`（可选）。

这是**向后兼容的 SUPERSET**——旧 CLI `--date 2026-07-02` 仍然工作（clap 会把 `--date` 的值填入 `Some("2026-07-02")`）。但不传 `--date` 也不传 `--start`/`--end` 时，`from_cli(None, None, None)` 的行为需要明确定义（见上条）。

**建议**：在设计 §3.3 加一句 "`date` 改为 `Option<String>`，不传时 clap 默认 `None`；若也不传 `--start`/`--end`，`from_cli` 返回 `Err`（建议）"。

---

## 次要意见

| 位置 | 意见 |
|------|------|
| L50 `date=None, start=None, end=None` | 见上文——行为未定义且不是 "当前行为一致" |
| L51 `date + start + end` 冲突 | `from_cli` 错误消息建议包含参数名，如 `"--date cannot be combined with --start/--end; use either --date for single day or --start/--end for range"` |
| L298 `src/data/models.rs 或 src/sources/openstock_client.rs` | 建议固定到 `models.rs`——`MinutePeriod` 已在 `models.rs`，`DateOrRange` 是同级通用工具类型 |
| L64 R1 `/data/bars` 字段名假设 `start_date`/`end_date` | `fetch_index_klines`（envelope 路径，已知）用 `start_date`/`end_date`。`fetch_minute_klines`（直 reqwest）应同样用 `start_date`/`end_date`——与 `/data/fetch` 类别的 P0.10 实践一致。但 `/data/bars` 的实际契约仍需 live-verify |

---

## 已验证正确的部分

| 设计声明 | 结论 | 证据 |
|----------|------|------|
| D1 `DateOrRange` enum 单参数互斥 | ✅ | 编译时强制，优于三 `Option` 参数 |
| D2 handler 层校验（不用 clap group） | ✅ | 错误消息可定制，与现有 handler 风格一致 |
| D3 保留 `--date` 快捷路径 | ✅ | `DateOrRange::Date` 向后兼容 |
| D4 wiremock-first + live-verify | ✅ | 字段名漂移只需改 `populate_body`，不破坏签名 |
| INV-1A date 与 start/end 互斥 | ✅ | `from_cli` 集中校验 |
| INV-1B Range 端点含边界 + start>end 报错 | ✅ | U6 测试覆盖 |
| INV-2A 向后兼容 wiremock 测试 | ✅ | `Date(d)` 的 wire body 与当前完全一致 |
| INV-2B Vec 扁平（`fetch_minute_klines` 侧） | ✅ | 分钟蜡烛的 `time` 字段含完整日期，无需额外分组 |
| INV-3 不修改 P0.13b-1/2 类型 | ✅ | `MinuteBar`/`MinuteShare`/`MinutePeriod` struct 不变 |
| R3 现有测试破坏 | ✅ | `Date(d)` 调用形态等价，wire body 不变 |
| R4 from_cli 边界 | ✅ | U1-U6 覆盖 6 种边界 |

---

## 总结

| 维度 | 评价 |
|------|------|
| API 设计 | ✅ `DateOrRange` enum 优雅——编译时互斥，易扩展 |
| P0.13b-1 兼容性 | ✅ `fetch_minute_klines` 的 ISO timestamp 天然支持多日范围 |
| P0.13b-2 兼容性 | ❌ `fetch_minute_share` 的单 `date` parser 无法处理多日范围的 `time_minutes` |
| 边界行为定义 | ⚠️ `(None, None, None)` 行为未定义，依赖未验证的 OpenStock 默认值 |
| 决策完整性 | ✅ D1-D4 每条有 rejected alternatives |
| 测试覆盖 | ✅ 6 unit + 4 wiremock + 3 live |

**结论**：解决 `fetch_minute_share` 的多日日期解析问题（确认 OpenStock MINUTE_DATA 范围响应 shape 或退化为逐日循环）、明确 `(None, None, None)` 的 from_cli 行为后，即可编写实施计划。
