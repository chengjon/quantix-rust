# quantix-rust

## What This Is

`quantix-rust` 是面向 A 股量化交易场景的 Rust CLI 工具，和 Python `quantix` 共享数据源与数据库，重点承担高性能分析、策略执行与 operator 工作流。当前产品重心不是继续横向扩功能，而是把策略执行主线从 `paper` / `mock_live` 推进到语义清晰、边界明确、接近实盘约束的 live-ready 状态。

## Core Value

策略执行主线必须可靠、可解释、可验证，且不能让用户误以为未完成的 real-live broker 能力已经可安全使用。

## Requirements

### Validated

- ✓ 策略执行主线已闭环到 `paper` / `mock_live` / `execution_request` / `execution daemon`
- ✓ operator 工作流已覆盖 `watchlist`、`screener`、`market`、`monitor`、`stop`、`trade`、`risk`
- ✓ QMT preview-only bridge 与 execution bridge CLI 已接入当前执行链路

### Active

- [ ] 完成 Phase 29C live-ready execution hardening
- [ ] 收紧 execution mainline 语义、排障信息与文档边界
- [ ] 完成 real live / broker execution 的边界设计与实现收口
- [ ] 在不打断执行主线的前提下推进 risk、market、monitor 后续能力

### Out of Scope

- Frontend-first 或大规模 TUI 重构优先于 execution mainline 收口 — 不符合当前 P0 优先级
- 与执行主线无关的格式整理型提交 — 会稀释当前主线收敛
- 无明确用户需求牵引的通用基础设施扩展 — 先完成 execution / live-ready 再扩展

## Context

- 当前项目级 backlog 以根目录 `ROADMAP.md` 为准，2026-04-11 起同步迁移到 `.planning/` 作为正式 GSD 入口。
- 根目录 README 已明确：当前已交付到 Phase 29C，真实 `live` broker execution 仍未完成。
- Graphiti 已记录若干关键约束：`ROADMAP.md` 是 canonical roadmap source，且不允许 frontend-first 路线图挤占当前 P0 执行与风险优先级。
- 最近一次工作集中在 QMT live gate 跟进测试与文档澄清，说明 `qmt.mode=live` 的真实提交通道仍是当前 backlog 的一部分。

## Constraints

- **Priority**: 先完成 execution mainline 收口 — 当前 backlog 必须优先服务 Phase 29C / execution semantics / live boundary
- **Compatibility**: 不能误导用户把 preview-only / `mock_live` 当作 real live broker 路径 — README 和用户手册必须与实现保持一致
- **Safety**: 任何 real live / broker execution 推进都需要显式 gating 与可验证边界 — 避免 accidental live submission
- **Process**: 后续 phase 规划应走 GSD 流程 — 使用 `.planning/ROADMAP.md`、`STATE.md`、phase docs、UAT/verification

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 以 execution mainline 作为 v1.0 当前 active scope | 当前已知 backlog 中，这条线最连续且最接近用户可感知价值 | ✓ Good |
| 用根目录 `ROADMAP.md` 反向初始化 `.planning/` | 仓库此前没有正式 GSD 结构，但 backlog 已存在且优先级明确 | ✓ Good |
| 将 risk / market / monitor / infra 作为后续 phase，而非当前 phase 1 | 避免次级队列抢占 live-ready / broker 边界收口 | ✓ Good |

---
*Last updated: 2026-04-11 after migrating root backlog into formal GSD planning files*
