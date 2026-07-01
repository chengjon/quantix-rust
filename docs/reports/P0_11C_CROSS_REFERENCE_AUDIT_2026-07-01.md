# 三文档交叉审核：P0.11c 编排一致性

> 审核日期：2026-07-01
> 审核范围：
> - `docs/reports/P0_11C_PREFLIGHT_AUDIT_2026-07-01.md` — 权威编排（五阶段）
> - `openspec/changes/openstock-data-consumption-p0-11/tasks.md` §3c — 33 步执行清单
> - `openspec/changes/openstock-data-consumption-p0-11/design.md` — 设计意图层（D1-D7 + Risks）
> 审核方法：逐步骤编号对照 + design.md stale reference 扫描 + 行数/文件清单核实

---

## 总体评价

三份文档的**逻辑编排一致**——五个 Phase 的顺序（迁出→schema→scheduler→删除→文档）、硬约束（Phase 1 先于 Phase 4）、四个 Decision 的默认推荐都对齐。tasks.md 在上一轮审核后已纳入了 `config.rs` 清理（3c.25）和 R10 风险。但从**执行者视角**看，三份文档的步骤编号体系完全不同，按编号交叉引用会直接出错。

---

## 必须修正

### 1. 步骤编号完全不对齐——三份文档用三套编号（**关键**）

三份文档对同一操作使用了不同的 `3c.N` 编号，执行者无法用编号跨文档定位：

| 操作 | audit §四 编号 | tasks.md 编号 | design.md 引用 |
|------|:---:|:---:|:---:|
| grep 审计全清 | `3c.18` | `3c.27` | `3c.12` (R3) |
| 删除 `config.rs::TdxApiConfig` | `3c.16` | `3c.25` | （无引用） |
| `app_shell.rs` dispatcher 重路由 | `3c.3` | `3c.9` | `3c.9` (D5，碰巧对齐) |
| scheduler rewire | `3c.8` | `3c.17` | `3c.5` (R4) |
| `docker-compose.yml` 注释 | `3c.22` | `3c.30` | `3c.15` (R6) |
| `FUNCTION_TREE.md` 更新 | `3c.20` | `3c.31` | `3c.16` (D7) |
| CHANGELOG/docs 更新 | `3c.23` | `3c.33` | `3c.17` (D2) |

**根因**：audit 从自己的 Phase 1 起编为 3c.1（无 Phase 0），tasks.md 从 Phase 0 起编为 3c.1（共 33 步），design.md 用的是 P0.11a 时期的旧编号。

**影响**：如果执行者按 design.md R3 的「在 3c.12 做 grep 审计」去 tasks.md 找 3c.12，找到的是「修改 import_ticks 写入 direction 列」——完全错误。

**建议**：三选一作为权威编号源（推荐 tasks.md），audit 和 design.md 的编号全部对齐到它。或者 audit 不标编号、只标 phase，让编号只存在于 tasks.md。当前"三套编号共存"状态不可交付。

---

### 2. audit 缺少 tasks.md 的 Phase 0（前置准备 3c.1-3c.6）

tasks.md §3c Phase 0 包含六个前置步骤：

| tasks.md 编号 | 内容 |
|:---:|---|
| `3c.1` | P0.11a + P0.11b 确认已合并 |
| `3c.2` | **2b.10 live smoke 通过**（阻塞项） |
| `3c.3` | Decision 1 确认 |
| `3c.4` | Decision 2 确认 |
| `3c.5` | Decision 3 确认 |
| `3c.6` | Decision 4 确认 |

audit **完全略过了这个 Phase**，直接从自己的 Phase 1（迁出）开始编排。虽然 audit §三 3.1/3.2 以文字形式说了同样的前置条件，但缺少步骤级 checklist 意味着：

- 执行者从 audit 进入实操时，没有一个可勾选的「确认四个 Decision」checklist
- 2b.10 live smoke 作为 P0.11c 准入条件的显式步骤只存在于 tasks.md，audit 没有同等强调

**建议**：audit §四 增加 Phase 0（或等效前置 checklist），引用 tasks.md 3c.1-3c.6 作为权威步骤，不重复编号。

---

### 3. design.md 七处任务编号已全部过期

design.md 作为设计意图文档，引用具体任务编号本就不该是它的职责——但既然引了，当前编号全部指向 P0.11a 时期的旧版 tasks.md：

| design.md 行号 | 当前位置的编号 | 应指向 tasks.md | 偏差 |
|------|:---:|:---:|---|
| L27 | `3c.17` | `3c.33` | 差 16 步 |
| L54 | `3c.5` | `3c.11`（或整个 Phase 2） | 差 6 步 |
| L111 | `3c.9` | `3c.9`（碰巧对齐） | 唯一对齐 |
| L119 | `3c.16` | `3c.31` | 差 15 步 |
| L135 (R3) | `3c.12` / `3c.13` | `3c.27` / `3c.28` | 差 15 步 |
| L136 (R4) | `3c.5` | `3c.17` | 差 12 步 |
| L138 (R6) | `3c.15` | `3c.30` | 差 15 步 |

**建议**：design.md 的 Mitigation 列改为引用 phase 名称而非具体编号（例如 "Phase 4 grep audit + build clean" 替代 "3c.12 grep audit + 3c.13 build clean"）。设计文档不应该硬编码会漂移的执行编号。tasks.md 已经承担了精确步骤的职责，design.md 退回设计意图层即可。

---

### 4. design.md D2 Consumer Map：`tdx_api_handler.rs` 行数过期

design.md L21 写「476 lines」，实际当前文件 726 行（P0.11a/b 期间膨胀了 250 行）。audit §1.1 已正确标注 726 行，tasks.md 3c.20 也写「726 行」。

**建议**：design.md D2 更新行数为 726，或改为「entire file (~730 lines, grew during P0.11a/b)」。

---

## 次要不一致

| 位置 | 差异 | 影响 |
|------|------|------|
| Phase 5 步骤顺序 | audit: FUNCTION_TREE → CLI_COMMAND → docker-compose → CHANGELOG。tasks.md: docker-compose → FUNCTION_TREE → CLI_COMMAND → CHANGELOG | 低——Phase 5 内步骤无依赖，顺序任意 |
| audit §一「11 个文件」 | 全域检索实为 15 个（上一轮审核已指出），但 audit 的 Phase 4 步骤已包含漏掉的 3 个文件 | audit 标题数未同步更新，但步骤已覆盖 |

---

## 已验证一致的部分

| 检查项 | 结论 |
|--------|------|
| 五个 Phase 逻辑顺序（迁出→schema→scheduler→删除→文档） | ✅ audit + tasks.md 一致 |
| Phase 1 必须先于 Phase 4 的硬约束 | ✅ 两份文档都明确标注 |
| `config.rs` 清理已纳入 | ✅ tasks.md 3c.25 含 ⚠️ CRITICAL 标记，audit 3c.16 也已加入 |
| 四个 Decision 的默认推荐值 | ✅ 三份文档一致（D1=A, D2=A, D3=B, D4=A） |
| R10 风险（编译断裂） | ✅ tasks.md Risks 已新增，audit §五 未单列但 Phase 4 步骤已包含缓解 |
| Decision 3=B 的 Phase 2 展开 | ✅ audit + tasks.md 都是加 `direction` 列 + 改写入逻辑 |
| Decision 1=A 的工作量定性 | ✅ audit 标注「需 P0.11d 规模切片」，tasks.md Phase 3 标注「实为 P0.11d 规模独立切片」 |
| P0.11c 准入条件 | ✅ audit §三 + tasks.md Phase 0 都要求 2b.10 + 四决策确认 |

---

## 总结

| 维度 | 评价 |
|------|------|
| 逻辑编排一致性 | ✅ 五阶段顺序、硬约束、决策默认值三方对齐 |
| 步骤编号对齐 | ❌ 三套编号共存，跨文档引用必然出错 |
| design.md 时效性 | ⚠️ 七处任务编号 + 一处行数过期；退回 phase 名称引用即可修复 |
| audit 覆盖完整性 | ⚠️ 缺少 Phase 0 checklist（内容在 §三文字中存在，但没有可执行步骤） |
| tasks.md 权威性 | ✅ 当前最完整的执行清单，建议作为唯一编号源 |

**结论**：三份文档的核心编排逻辑一致，但从执行者视角有严重可用性问题——同一操作在三份文档中是三个不同的编号。必须选 tasks.md 为权威编号源、audit 和 design.md 退回到 phase 级引用（不标具体编号），才能安全交付。
