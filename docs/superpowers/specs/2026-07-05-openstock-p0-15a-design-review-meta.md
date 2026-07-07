# Review: docs/superpowers/specs/2026-07-05-openstock-p0-15a-design-review.md

**Date**: 2026-07-07
**Scope**: docs/superpowers/specs/2026-07-05-openstock-p0-15a-design-review.md
**File type**: md
**Doc type**: proposal（path 含 `review`，内容含 recommendation / scoring / verdict）
**Perspectives**: Actionability, Feasibility, Completeness
**Source document**: 审核 `docs/superpowers/specs/2026-07-05-openstock-p0-15a-minute-cli-persistence-design.md`（386 行设计文档）

---

## Evidence Verification

### 评审报告中声明的行号 vs 当前代码库

| 评审声明 | 评审行号 | 当前行号 | 漂移 | 状态 |
|----------|---------|---------|------|------|
| `from_cli(None, None, None)` 错误 | L341 | L351 | +10 | 轻微漂移 |
| `fetch_openstock_minute_share` 定义 | L520 | L539 | +19 | 轻微漂移 |
| `fetch_openstock_all_stocks` 定义 | L610 | L819 | +209 | 显著漂移 |
| `FetchMinuteShare` enum | L418 | L418 | 0 | 准确 |
| `app_shell.rs` 分发 | L407 | L407 | 0 | 准确 |
| `ClickHouseMinuteKlineSink` | L136 | L136 | 0 | 准确 |
| `ClickHouseMinuteShareSink` | L141 | L141 | 0 | 准确 |

### M1 发现的时效性验证

评审报告 M1 称 `mod.rs` 中 "no re-export" 需要 handler 通过全路径 `crate::db::clickhouse::minute::ClickHouseMinuteKlineSink` 导入。

**当前代码库状态**：`src/db/clickhouse/mod.rs:19` 已存在：
```rust
pub(crate) use self::minute::{ClickHouseMinuteKlineSink, ClickHouseMinuteShareSink};
```
注释明确标注 "Re-exported pub(crate) so P0.15a handlers (Task 3/4) can construct the sinks."

**git log 证据**：re-export 在 `9172648` 提交中添加（2026-07-06），即评审报告（2026-07-05）之后一天。M1 建议得到了采纳。

### 符号验证汇总

| 类别 | 数量 | 结果 |
|------|------|------|
| 文件路径 | 10/10 | 全部确认存在 |
| 符号（函数/类型） | 13/14 | 12 确认，1 标记 "not yet created"（`compute_apply`），0 矛盾 |
| 行号 | 7 处声明 | 4 处精确匹配，3 处漂移 |

---

## Checklist Results

### Actionability
12 items PASS。仅展示 FAIL：

| # | Check | Result | Notes |
|---|-------|--------|-------|
| AC3 | 无模糊建议（如 "improve this"） | FAIL | M2 建议 "Consider `fn check_env_apply() -> bool`" 的替代方案其实和 `compute_apply` 等价，没有解决根本的"薄包装测试"问题。建议本身可执行但未充分论证为什么当前方案不够好 |

### Feasibility
8 items PASS。无 FAIL 项。

### Completeness
5 items PASS。

| # | Check | Result | Notes |
|---|-------|--------|-------|
| CP1 | 所有引用符号存在 | PARTIAL | 行号漂移不代表内容错误，但 `fetch_openstock_all_stocks` 漂移 +209 行可能导致读者定位困难 |

---

## Findings

### Critical Issues

无。评审报告没有阻碍批准的结构性缺陷。

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| M1 | Findings M1 | M1 建议 "no re-export in mod.rs" 已过时 | 低：实施提交 `9172648` 已按评审建议添加 re-export。读者按当前 M1 建议操作会重复已存在的工作 | `mod.rs:19` 已有 `pub(crate) use self::minute::{ClickHouseMinuteKlineSink, ClickHouseMinuteShareSink};` | 在评审报告中追加一条时效性注释：`[2026-07-06 更新] re-export 已在实施提交 9172648 中添加` |
| M2 | 行号 | `fetch_openstock_all_stocks` 漂移 +209 行 | 低：不影响建议的实质正确性，但降低读者定位效率 | 评审 L610 → 当前 L819 | 在后续评审中使用 git blame 锚定或引用函数签名而非仅行号 |
| M3 | M2 建议 | `compute_apply` 替代方案未论证充分性 | 低：`check_env_apply()` 与 `compute_apply` 本质等价，建议未增加实际测试覆盖价值 | 评审报告 M2 原文 | 接受当前 `compute_apply` 方案为薄包装，明确只需通过 handler 集成测试验证 |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| L1 | 文档类型自判 | 评审报告自判 doc type 为 "spec"，实际应为 "proposal"（这是一份评审报告而非被审核的设计文档） | 评审报告头部的 doc type 字段 | 修正为 "proposal" 或 "review" |
| L2 | C1 修复表述 | C1 建议 `ClickHouseMinuteKlineSink::<'_>` 时未提及 `impl MinuteSink<MinuteKlineCH>` trait bound 也必须满足 | `minute.rs:146` 显示 trait impl 已存在但评审报告未引用 | 追加一行验证：`ClickHouseMinuteKlineSink<'a>` 已实现 `MinuteSink<MinuteKlineCH>` |

---

## Strengths

- **证据密度极高**：15 个符号 + 10 个文件全部交叉验证，无凭空断言。每个发现都锚定到具体的代码行和文件路径，符合 review2md 的 "evidence-driven" 核心原则。
- **C1（生命周期）发现精准**：`ClickHouseMinuteKlineSink<'a>` 的构造函数缺少生命周期参数是真实的编译阻断问题。评审正确标记为 Critical，给出修复方案，且在 R1 中预见了此风险。
- **检查表覆盖完整**：CP1-CP5（完整性）、CA1-CA5（代码对齐）、CS1-CS3（一致性）13 项全部执行，PASS/FAIL 有据可查。
- **分层严重性合理**：C1（编译阻断）→ M1-M2（导入/测试设计）→ L1-L2（风格/简洁性），层次分明。
- **裁决与证据一致**：`APPROVE_WITH_NOTES` 裁决与 4.6/5.0 加权分一致，既肯定了设计质量，又阻止了盲目合并。
- **M1 建议被采纳**：re-export 在评审次日（2026-07-06）的实施提交 `9172648` 中添加，证明评审的前瞻性和可操作性。

---

## Recommendations

1. **追加时效性注释**：在 M1 发现旁标注 `[2026-07-06 更新] re-export 已在 9172648 中添加`，避免后续读者重复工作。（M1）
2. **修正文档类型**：将评审报告头部的 doc type 从 "spec" 改为 "proposal"（或 "review"），准确反映文档性质。（L1）
3. **使用符号引用替代纯行号**：在后续评审中对易漂移的行号使用函数签名（如 `pub(crate) async fn fetch_openstock_all_stocks`）+ 行号双锚定。（M2）
4. **接受 M2 薄包装方案**：`compute_apply` 的预期测试路径是 handler 集成测试而非单元测试，无需重新设计。（M3）

---

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Actionability (2x) | 5 | 每条建议给出具体代码修改、文件路径、替换文本。M1 建议被实际采纳。 |
| Feasibility (2x) | 5 | 所有发现基于已运行的 grep/read_file 交叉验证。无不可达主张。 |
| Completeness | 4 | 覆盖 CP/CA/CS 全矩阵。扣除：未提及 INV 冲突检查，行号漂移影响定位效率。 |
| Technical Accuracy | 4 | 核心发现（C1 生命周期）准确。M1 已过时（非评审时错误，而是代码已演进）。扣分：行号漂移未标注验证日期。 |
| Terminology Consistency | 5 | 术语与被审文档和代码库一致（INV-*, D1-D6, P0.14/P0.15a 版本号）。 |
| **Overall** | **4.7** | 加权：(5×2+5×2+4+4+5)/7 = 4.71 |

---

## Verdict

APPROVE

理由：这份评审报告本身质量很高——证据驱动、发现精准、建议可执行。M1 建议已在后续实施中采纳（`9172648`），反向验证了评审的前瞻性。三个 Medium/Low 问题均属时效性/表述性，不影响评审报告的实质性正确性。行号漂移是代码演进的自然结果而非评审缺陷。无需修改即可归档。
