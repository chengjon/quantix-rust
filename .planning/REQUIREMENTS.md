# Requirements: quantix-rust

**Defined:** 2026-04-11
**Core Value:** 策略执行主线必须可靠、可解释、可验证，且不能让用户误以为未完成的 real-live broker 能力已经可安全使用。

## v1 Requirements

### Execution Hardening

- [ ] **EXE-01**: `mock_live` 执行路径支持 delayed fill、partial fill 与 `Unknown` 状态注入后的恢复
- [ ] **EXE-02**: 系统具备 open-order / account reconciliation 的基础脚手架，可用于 live-ready 场景验证
- [ ] **EXE-03**: 网络故障与执行恢复路径可被注入并通过测试验证

### Mainline Semantics

- [ ] **SEM-01**: operator 能明确区分 `request completed` 与订单终态，避免把请求完成误判为成交完成
- [ ] **SEM-02**: daemon / operator 侧提供足够的 request 排障与可观测信息，能定位失败或卡住原因
- [ ] **SEM-03**: 文档与 CLI 语义清楚区分 `paper`、`mock_live` 与 `live`，避免 capability drift

### Live Broker Boundary

- [ ] **LIV-01**: 系统定义 real live adapter 的运行契约、gating 条件与 broker path 边界
- [ ] **LIV-02**: QMT 从 preview-only 进入真实执行需要显式 live gate，并有回归测试覆盖
- [ ] **LIV-03**: live adapter / broker path 拥有最小可验证安全约束与验证流程

### Risk And Expansion

- [ ] **RSK-01**: 风控规则增强覆盖实盘导入、波动率规则、行业规则与自动减仓
- [ ] **MKT-01**: 市场分析补齐历史、详情、实时三类用户可见能力
- [ ] **OPS-01**: 系统通知能力可支持关键运行事件提示

### Infrastructure Backlog

- [ ] **INF-01**: CLI/UX backlog 可交付 ratatui TUI 菜单与 Parquet 导出，但不挤占 execution mainline
- [ ] **INF-02**: 数据与运行时基础能力补齐节假日数据、batch 流式进度与 monitoring health/metrics

## v2 Requirements

### Deferred

- **BRG-01**: Wind bridge 支持
- **BRG-02**: Choice bridge 支持

## Out of Scope

| Feature | Reason |
|---------|--------|
| Frontend-first roadmap | 与当前 execution P0 主线冲突 |
| 大规模 UI/TUI 重构 | 当前不是用户价值最高的缺口 |
| 与主线无关的通用基础设施扩展 | 会稀释 live-ready / broker 收口节奏 |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| EXE-01 | Phase 1 | Pending |
| EXE-02 | Phase 1 | Pending |
| EXE-03 | Phase 1 | Pending |
| SEM-01 | Phase 2 | Pending |
| SEM-02 | Phase 2 | Pending |
| SEM-03 | Phase 2 | Pending |
| LIV-01 | Phase 3 | Pending |
| LIV-02 | Phase 3 | Pending |
| LIV-03 | Phase 3 | Pending |
| RSK-01 | Phase 4 | Pending |
| MKT-01 | Phase 5 | Pending |
| OPS-01 | Phase 5 | Pending |
| INF-01 | Phase 6 | Pending |
| INF-02 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-11*
*Last updated: 2026-04-11 after initializing GSD planning from the root roadmap*
