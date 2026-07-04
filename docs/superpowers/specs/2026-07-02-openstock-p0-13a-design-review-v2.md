# 审核意见（第二版）：P0.13a Multi-period K-line Fetch 设计文档

> 审核日期：2026-07-02
> 审核范围：`docs/superpowers/specs/2026-07-02-openstock-p0-13a-multi-period-klines-design.md` v2（修订版）
> 审核基线：上一轮审核 `docs/superpowers/specs/2026-07-02-openstock-p0-13a-design-review.md`，对比 v2 变更

---

## 变更确认

上一轮审核提出的三条问题全部修了：

| # | 上一轮问题 | v2 状态 |
|---|----------|---------|
| 1 | `KlinePeriod` 名称与 `kline_aggregator.rs` 碰撞 | ✅ 新增 D8，全类型改名为 `BarPeriod` |
| 2 | `as_openstock_param()` None 返回 `""` | ✅ 改为返回 `Option<&str>`，`None` 时不发 `adjust` 字段 |
| 3 | `FromStr` 大小写未声明 | ✅ D6 加 "case-insensitive on input" |
| 4 | L134 `"hff"` 笔误 | ✅ 已修正为 `"hfq"` |
| 5 | D1 描述缺 `Day` | ✅ 改为 "day/week/month + qfq/hfq" |
| 6 | D5 测试计数模糊 | ✅ 改为 "5 unit/wiremock + 3 live = 8 tests" |

---

## 残留问题：五处 `KlinePeriod` → `BarPeriod` 重命名不彻底

D8 和 §Components 1 已将类型改名为 `BarPeriod`，但以下五处仍残留旧名称 `KlinePeriod`：

| # | 行号 | 位置 | 当前文本 | 应为 |
|---|:---:|------|---------|------|
| 1 | L195 | `FetchKlines` CLI 结构体 | `// parsed to KlinePeriod in handler` | `// parsed to BarPeriod in handler` |
| 2 | L224 | Error Handling 表 | `Handler parses with KlinePeriod::from_str` | `Handler parses with BarPeriod::from_str` |
| 3 | L244 | T1 测试描述 | `` `KlinePeriod` round-trip + strict rejection `` | `` `BarPeriod` round-trip + strict rejection `` |
| 4 | L272 | Phase 1 实施清单 | `` `KlinePeriod` enum + `FromStr` + tests (T1) `` | `` `BarPeriod` enum + `FromStr` + tests (T1) `` |
| 5 | L343 | Risk 表第三行 | `` `KlinePeriod::from_str` rejecting aliases `` | `` `BarPeriod::from_str` rejecting aliases `` |

**影响**：执行者看到 Phase 1 清单写 "`KlinePeriod` enum"，会疑惑到底是用新类型还是旧类型。前四处是纯文本修正，第五处（风险表）在评审时会引起混淆。

---

## 次要问题

### 1. Decision 计数过期（L36-37）

```markdown
Renumbered D1-D6 (D = design decision). The brainstorming Q1-Q5 map to
D1-D5; D6 covers the OpenSpec/governance approach.
```

v2 新增了 D7（OpenSpec approach）和 D8（Type naming），现在是 D1-D8。这段描述应更新为：

```markdown
Renumbered D1-D8 (D = design decision). The brainstorming Q1-Q5 map to
D1-D5; D6-D8 cover strictness, OpenSpec approach, and type naming.
```

### 2. OpenSpec layout 写 "D1-D5"（L296）

```markdown
├── design.md         # D1-D5 decisions (see "Decisions" table above)
```

应更新为 `# D1-D8 decisions`。

---

## 总结

| 维度 | 评价 |
|------|------|
| 上一轮修复质量 | ✅ 三条阻断/重要问题全部正确修复 |
| 重命名彻底性 | ⚠️ 五处残留 `KlinePeriod` 需全局替换 |
| 文档自洽性 | ⚠️ 两处 Decision 数量描述过期（D1-D6 → D1-D8） |

**结论**：修复五处 `KlinePeriod` 残留和两处 Decision 计数后，文档即可进入实施。
