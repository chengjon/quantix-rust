# Quantix GSD Roadmap

## 🚧 v1.0 Execution Mainline Completion

把根目录 `ROADMAP.md` 中已经明确的 backlog 迁移到正式 GSD 结构。当前 milestone 聚焦 execution mainline 收口，先完成 live-ready hardening、语义加固与 real live / broker 边界，再推进 risk、market、monitor 与 infra backlog。

| Phase | Plans | Status | Completed |
|-------|-------|--------|-----------|
| 1 | 3/3 | Completed | 01, 02, 03 |
| 2 | 3/3 | Completed | 01, 02, 03 |
| 3 | 3/3 | Completed | 01, 02, 03 |
| 4 | 1/3 | In Progress | 01 |
| 5 | 0/0 | Pending | |
| 6 | 0/0 | Pending | |

- [x] **Phase 1: Phase 29C live-ready hardening**
- [x] **Phase 2: Execution mainline semantics hardening**
- [x] **Phase 3: Real live / broker execution closure**
- [ ] **Phase 4: Risk rule enhancement**
- [ ] **Phase 5: Market and notification expansion**
- [ ] **Phase 6: Infra and CLI backlog**

### Phase 1: Phase 29C live-ready hardening

**Goal:** 把现有 `paper` / `mock_live` 执行骨架推进到更接近真实 live 约束、但仍可控可验证的状态。
**Requirements**: EXE-01, EXE-02, EXE-03
**Depends on:** none
**Plans:** 3/3 plans executed
**Success Criteria**:
1. `mock_live` 能模拟 delayed fill、partial fill 和 `Unknown` 注入后的恢复路径。
2. open-order 与 account reconciliation 有最小脚手架与测试覆盖。
3. README / USER_MANUAL / handler 输出明确区分 `mock_live`、`qmt_live`、request `completed` 与订单终态。

### Phase 2: Execution mainline semantics hardening

**Goal:** 收紧 execution request 生命周期语义与 operator 排障信息，避免请求状态和订单终态混淆。
**Requirements**: SEM-01, SEM-02, SEM-03
**Depends on:** Phase 1
**Plans:** 3/3 plans executed
**Success Criteria**:
1. CLI/operator 明确区分 request completion 和订单终态。
2. daemon/operator 输出足够的 request 诊断与可观测信息。
3. README / USER_MANUAL / CLI 语义边界不再混淆 `paper`、`mock_live`、`live`。

### Phase 3: Real live / broker execution closure

**Goal:** 以显式 safety gating 为前提，完成 live adapter 与 QMT real execution 边界收口。
**Requirements**: LIV-01, LIV-02, LIV-03
**Depends on:** Phase 2
**Plans:** 0/3 plans executed
**Success Criteria**:
1. live adapter 契约、broker path 边界与 gating 机制明确并落地。
2. QMT 从 preview-only 到真实执行的路径有显式 live gate 和回归验证。
3. live broker 路径具备最小安全约束、验证流程与文档说明。

### Phase 4: Risk rule enhancement

**Goal:** 在 execution 主线稳定后，补齐最关键的实盘风险规则能力。
**Requirements**: RSK-01
**Depends on:** Phase 3
**Plans:** 0/0 plans executed
**Success Criteria**:
1. 风控规则覆盖实盘导入、波动率规则、行业规则与自动减仓。
2. 新规则不会破坏现有 operator / execution 主线。
3. 风控策略有文档与验证入口。

### Phase 5: Market and notification expansion

**Goal:** 补齐用户可见的市场分析扩展能力与系统通知能力。
**Requirements**: MKT-01, OPS-01
**Depends on:** Phase 4
**Plans:** 0/0 plans executed
**Success Criteria**:
1. 市场分析支持历史、详情、实时三个方向的后续能力。
2. 系统通知可覆盖关键运行或执行事件。
3. 用户手册与 README 更新到当前交付边界。

### Phase 6: Infra and CLI backlog

**Goal:** 在不打断主线的情况下处理工程占坑与基础设施补齐项。
**Requirements**: INF-01, INF-02
**Depends on:** Phase 5
**Plans:** 0/0 plans executed
**Success Criteria**:
1. TUI 菜单与 Parquet 导出按需推进，不改变主线优先级。
2. 节假日数据、batch 进度、health/metrics 导出形成独立可执行 backlog。
3. backlog 项不会与 execution mainline 语义边界相冲突。
