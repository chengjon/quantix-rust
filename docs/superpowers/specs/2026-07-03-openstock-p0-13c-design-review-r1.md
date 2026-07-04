# R1 修订审核：P0.13c 多日范围查询

> 审核日期：2026-07-03
> 审核范围：commit `f9cad11`（`docs/superpowers/specs/2026-07-03-openstock-p0-13c-multi-day-range-design.md` v2）
> 审核基线：R1 审核 `docs/superpowers/specs/2026-07-03-openstock-p0-13c-design-review.md` 的三条问题

---

## 三条问题核查

| # | R1 审核问题 | R1 修订 | 状态 |
|---|----------|---------|:--:|
| 1 | `fetch_minute_share` 多日日期解析 | D6：client-side 逐日循环；INV-2C：日期从 `meta.trading_date` 取；`iter_dates_inclusive` helper | ✅ |
| 2 | `(None, None, None)` 与半开区间 | D5：三个 case 全部 Err；U3/U4/U7 覆盖；错误消息含参数名 | ✅ |
| 3 | CLI `date: String → Option<String>` | §3.3 明确标注 SUPERSET；`from_cli(None,None,None)` → Err | ✅ |

---

## R1 新增内容质量

| 新增项 | 评估 |
|--------|------|
| **D6**：client-side 循环 vs server-side range | ✅ 根因引用 `_eltdx_timeseries.py:181-208` 精确到行，证据充分 |
| **INV-2C**：`meta.trading_date` 解析 | ✅ 正确处理非交易日 server 返回相邻日 series 的情况 |
| **R6**：未来 server 支持时的切换路径 | ✅ 签名不变，仅改 impl——降低后续迁移成本 |
| **D5**：from_cli 拒绝三种模糊形态 | ✅ 错误消息三段式含参数名 + 用法提示，用户体验好 |
| **W5**：非交易日跳过 | ✅ mock 返回空 records，验证循环不 panic |
| **U3/U4/U7**：半开区间 + 全 None | ✅ from_cli 边界 7 测试全覆盖 |
| **§3.2**：两条路径的 server 支持对比表 | ✅ 一表胜千言——`fetch_minute_klines` server-side / `fetch_minute_share` client-side |

---

## 一致性检查

| 检查项 | 结论 |
|--------|------|
| D1-D6 决策链路完整 | ✅ D1(enum) → D2(handler校验) → D3(`--date`快捷) → D4(wiremock-field-klines) → D5(拒绝模糊) → D6(client循环) |
| INV 与实现对齐 | ✅ INV-1A(互斥) / INV-1B(端点) / INV-2A(兼容) / INV-2B(扁平) / INV-2C(trading_date) — 全有对应代码路径 |
| U1-U7 与 D5 对齐 | ✅ U3(start only Err) + U4(end only Err) + U7(all None Err) 全部对应 D5 决策 |
| W1-W5 覆盖两条路径 | ✅ W1-W2(minute_klines) + W3-W5(minute_share 循环) 无遗漏 |
| R1-R6 缓解措施 | ✅ R1(field名) / R2(OOM) / R3(测试破坏) / R4(边界) / R5(server限制) / R6(未来切换) — 各有一条缓解 |
| 文件清单与设计一致 | ✅ `models.rs` 固定为 `DateOrRange` 位置，行数上调到 ~540 |

---

## 结论

三条问题全部修正，新增 D5/D6/INV-2C/R6/W5/U3/U4/U7 逻辑自洽，无占位符，可进入实施计划。
