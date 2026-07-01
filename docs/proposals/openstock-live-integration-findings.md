# OpenStock 真实联调发现的问题

> 联调日期：2026-07-01
> 联调环境：`http://192.168.123.104:8040`，API Key 已启用
> 客户端：`quantix` HEAD `4557768`（OpenStockClient 含 retry/circuit-breaker/CliRuntime 集成）
> 数据来源：5 个 P0 category 真实拉取（`STOCK_CODES` / `TRADE_DATES` / `INDEX_KLINES` / `ALL_STOCKS` / `WORKDAYS`）
>
> **修订（v2，吸收审核意见 `openstock-live-integration-findings-review.md`）**：
> - Q2 `StockListRecord` 改用 `#[serde(rename_all = "camelCase")]`，`trade_status` 独立成字段（不 alias 到 `listing_date`）
> - Q1 推进顺序简化：baostock 已知用 `start_date`/`end_date`，只测一种变体即可
> - 新增「live test 加日期过滤变体」步骤（`OPENSTOCK_LIVE_START/END` 环境变量）

---

## 联调结果总览

| # | 命令 | 来源 | 记录数 | 状态 | 备注 |
|---|------|------|-------:|------|------|
| 1 | `data openstock fetch-codes` | eltdx | 5532 | ✅ 通 | runtime 只返 `{code, symbol, market}`，无 `name` 字段 |
| 2 | `data openstock fetch-calendar --year 2026` | baostock | 365 | ✅ 通 | `calendar_date` + `is_trading_day:"0"/"1"` 已被 alias + loose-bool 解析器正确处理 |
| 3 | `data openstock fetch-index --symbol sh000001 --start 2026-06-01 --end 2026-06-30` | baostock | 500 | ⚠️ **语义错误** | 返回 2015-01-05..2017-01-18，**start/end 参数被 runtime 忽略**；latency 4.1s；数值字段全是字符串 |
| 4 | `data openstock fetch-all-stocks` | baostock | 500 | ⚠️ **截断** | 默认只返 500 条 + `quality_flags: ['fallback_day:2021-05-14']`；客户端期望的 `market`/`listing_date` 实际不存在，runtime 字段是 `code`/`code_name`/`tradeStatus` |
| 5 | `data openstock fetch-workdays --action today_is_workday` | eltdx | 1 | ✅ 通 | `today_is_workday=false` 返回正常 |

---

## 一、quantix 可做的修改

### Q1（HIGH）— 调研并修正 `INDEX_KLINES` 参数名

**现象**：客户端发送 `{code:"sh000001", start:"2026-06-01", end:"2026-06-30"}`，runtime 返回 2015-01-05 起的 500 条，**完全无视日期范围**。

**位置**：`src/sources/openstock_client.rs:350-364`

```rust
pub async fn fetch_index_klines(
    &self, code: &str, start: Option<&str>, end: Option<&str>,
) -> Result<OpenStockResponse<IndexKlineRecord>> {
    let mut params = serde_json::json!({ "code": code });
    if let Some(start) = start { params["start"] = Value::String(start.to_string()); }
    if let Some(end)   = end   { params["end"]   = Value::String(end.to_string());   }
    self.fetch("INDEX_KLINES", params).await
}
```

**待办**：
1. 先做对照实验，分别用 `start`/`end`、`start_date`/`end_date`、不带日期 三种 payload 测一遍 runtime 真实行为（参考 `TRADE_DATES` 已知用 `start_date`/`end_date`）
2. 若 runtime 接受的是 `start_date`/`end_date` —— 改客户端参数名，加 alias 双向兼容（runtime 升级时不会破）
3. 若 runtime 是 bug（参数确实叫 `start`/`end` 但被忽略）—— 转 §二.O1 报给 openstock

### Q2（MEDIUM）— `ALL_STOCKS` 字段映射修正

**现象**：`StockListRecord` 期望 `{code, name, market, listing_date}`，runtime 实际返回 `{code:"sh.000001", code_name:"上证综合指数", tradeStatus:"1"}`。

**位置**：`src/sources/openstock_codes.rs:48-60`

**修改方案**（吸收审核意见 #1 + #2）：

```rust
#[serde(rename_all = "camelCase")]   // runtime 用 camelCase（tradeStatus）
pub struct StockListRecord {
    #[serde(default)] pub code: Option<String>,
    #[serde(default, alias = "code_name")] pub name: Option<String>,   // runtime 用 snake_case code_name，单独 alias
    #[serde(default)] pub market: Option<String>,                      // runtime 当前不返，保持 Option
    #[serde(default)] pub listing_date: Option<String>,                // runtime 当前不返；不加 alias，等 runtime 补齐
    #[serde(default)] pub trade_status: Option<String>,                // 独立字段；语义=交易状态，≠上市日期
    #[serde(flatten)] pub extra: HashMap<String, serde_json::Value>,
}
```

**审核意见避坑**：
- ❌ 不要写 `alias = "trade_status"`——runtime 实际是 `tradeStatus`（camelCase），serde alias 是精确匹配，匹配不到
- ❌ 不要把 `tradeStatus` alias 到 `listing_date`——语义不对，`tradeStatus:"1"` 表示交易状态（1=正常 / 0=停牌），`parse_listing_date("1")` 必然报错
- ✅ 用 `#[serde(rename_all = "camelCase")]` 让 `trade_status` 自动对应 `tradeStatus`；`code_name` 仍单独 alias（snake_case 与 camelCase 同形）

注意 `code` 当前为 `"sh.000001"` 含前缀，会触发 `require_code` 的 `all_ascii_digit` 校验失败——需要改用 `normalize_symbol` 剥前缀（参考 `openstock_index.rs:104` 已有做法）。

### Q3（LOW）— `STOCK_CODES` 增加 `symbol` 字段透传

**现象**：runtime 返回 `{code:"sh689009", symbol:"sh689009", market:"a_share"}`，`StockCodeRecord` 只解析 `code` + `name`，丢失 `symbol`（与 `code` 同值但格式不同）与 `market`。

**位置**：`src/sources/openstock_codes.rs:38-45`

**修改方案**：保持当前 `code` 抽取逻辑，把 `symbol` 和 `market` 加进 `extra`（已经是 catch-all HashMap）。展示层（`fetch_openstock_codes` handler）可以读 `extra["symbol"]` / `extra["market"]` 用于打印。

### Q4（HIGH）— `INDEX_KLINES` 客户端解析容忍字符串数值

**现象**：runtime 返回的 `open/high/low/close/amount` 全是字符串 `"3258.6270"`，而非 JSON number。`volume` 同样是字符串 `"53135239168"`。

**现状**：`IndexKlineRecord` 字段已经是 `Option<serde_json::Value>`，`parse_decimal` / `parse_volume` 也已支持 `Value::String` 分支（`openstock_index.rs:152-176`）。**这部分已经对了**，无需改动。

**验证**：手动跑 `data openstock validate-index` 用 fixture 已通过——这条不需要动。

### Q5（INFO）— `WORKDAYS` 返回 `today_is_workday=false` 需要排查

**现象**：今天 2026-07-01 是周三，`fetch-workdays --action today_is_workday` 返回 `today_is_workday: false`。

**两种可能**：
- runtime bug：eltdx adapter 的 `is_workday` 逻辑错了
- 业务正确：2026-07-01 确实是 A 股休市日（特殊纪念日？节假日？）

**待办**：不在 quantix 改，但要在 §二.O4 提醒 openstock 侧确认。

---

## 二、建议 openstock 配合的事项

### O1（HIGH）— `INDEX_KLINES` 不遵守 `start`/`end` 参数

**复现**：
```bash
curl -X POST http://192.168.123.104:8040/data/fetch \
  -H "X-API-Key: ..." -H "Content-Type: application/json" \
  -d '{"data_category":"INDEX_KLINES","params":{"code":"sh000001","start":"2026-06-01","end":"2026-06-30"}}'
```

**预期**：返回 2026-06 区间的 K 线。

**实际**：返回 500 条从 2015-01-05 起的 K 线，时间区间被完全忽略。

**建议**：
1. 确认 `baostock._fetch_index_klines` adapter 实际接受的参数名（疑似 `start_date`/`end_date`，参考 `TRADE_DATES` 一致）
2. 要么改 adapter 接受 `start`/`end` 别名，要么改 `DATA_CAPABILITY_SCOPE` / `CONNECTION_GUIDE` 明确标注参数名
3. 现状下任何按区间拉取指数 K 线的消费端都会拿到错误数据 —— 优先级最高

### O2（HIGH）— `ALL_STOCKS` 字段契约不一致

**当前 `DATA_CAPABILITY_SCOPE.md` / `CONNECTION_GUIDE.md` 声明**：`ALL_STOCKS` = "Full stock list"。

**实际返回字段**：`{code, code_name, tradeStatus}`（示例：`{"code":"sh.000001","tradeStatus":"1","code_name":"上证综合指数"}`）。

**与同源 `STOCK_BASIC`（baostock）字段集冲突**：消费端无法仅从字段名推断出 `market` / `listing_date`。

**建议**：
1. 在 `DATA_CAPABILITY_SCOPE.md` 的 `ALL_STOCKS` 行标注真实字段集，或修 adapter 补齐 `market` / `listing_date`
2. `tradeStatus` 字段语义（字符串 "1"/"0"？还是 int？是否可空？）需文档化

### O3（MEDIUM）— `ALL_STOCKS` 默认截断 500 条 + `fallback_day` 行为

**现象**：不带 `day` 参数请求 `ALL_STOCKS`，返回 500 条 + `quality_flags: ['fallback_day:2021-05-14']`。

**问题**：
- 默认截断 500 条未在 `CONNECTION_GUIDE.md` 文档化
- `fallback_day:2021-05-14` 这个日期很可疑（2021 年的 fallback 用到 2026 年？baostock 内部缓存？）
- 消费端拿到 500/5532 ≈ 9% 的全市场，容易误用

**建议**：
1. `CONNECTION_GUIDE.md` 明确 `ALL_STOCKS` 默认上限 500，消费端需带 `day` 或分页
2. 排查 `fallback_day:2021-05-14` 是否为 baostock adapter bug
3. 上限行为考虑改成「全量」或「显式提示需分页」

### O4（LOW）— `WORKDAYS action=today_is_workday` 返回 false（2026-07-01 周三）

**复现**：
```bash
curl -X POST http://192.168.123.104:8040/data/fetch \
  -H "X-API-Key: ..." -H "Content-Type: application/json" \
  -d '{"data_category":"WORKDAYS","params":{"action":"today_is_workday"}}'
# → {"data":[{"action":"today_is_workday","today_is_workday":false}],"source":"eltdx"}
```

**待确认**：2026-07-01（周三）是否为 A 股交易日？若不是（特殊纪念日休市），业务正确；若是，eltdx adapter 有 bug。

### O5（INFO）— `STOCK_CODES` 字段集与 runtime 不符

**当前 `DATA_CAPABILITY_SCOPE.md`**：`STOCK_CODES` = "Lightweight provider code lists"。

**实际返回字段**：`{code:"sh689009", symbol:"sh689009", market:"a_share"}`（eltdx provider）。

**建议**：在 `DATA_CAPABILITY_SCOPE.md` 或 `/sources.provider_capabilities[].fields_typed` 中明确 `STOCK_CODES` eltdx 行的真实字段，便于消费端类型生成。

---

## 三、推进顺序建议

1. **quantix 侧先做 §一.Q1 调研**（简化版，吸收审核 §94）：
   - `fetch_trade_dates` doc comment（`openstock_client.rs:329-333`）已明确 baostock 用 `start_date`/`end_date`
   - INDEX_KLINES 同是 baostock provider，**只测 `start_date`/`end_date` 一种变体**即可，省掉 `start`/`end` 与不带日期两轮
   - 单次 curl 30 秒内可定方向
2. 根据 Q1 结果决定 §一.Q1 修改（`openstock_client.rs:357-361` 改参数名）或转 §二.O1 报 bug
3. **修完 Q1 后给 live test 加日期过滤变体**（吸收审核 #3）：
   - 现状：`tests/openstock_live_index.rs` 调用 `fetch_index_klines(&symbol, None, None)`，不验证日期过滤
   - 改造：新增 `OPENSTOCK_LIVE_START` / `OPENSTOCK_LIVE_END` 环境变量驱动的变体，assert 返回 K 线日期落在请求区间内
   - 同步检查 `validate_openstock_index` handler 的 `_start`/`_end` 未使用问题（`openstock_handler.rs:158-190`）
4. **并行**做 §一.Q2 字段映射修正（独立工作）
5. 把 §二.O2-O5 整理成一份 openstock issue 提交
6. 完成后再考虑是否启用 `persist-live` 写 shadow 表的端到端联调

---

> 本文档为联调记录，不作为功能状态注册表；功能当前状态以 [`FUNCTION_TREE.md`](../../FUNCTION_TREE.md) 中的状态注册表为准。
