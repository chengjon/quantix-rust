# Phase 1 Context

## Phase

Phase 1: Phase 29C live-ready hardening

## Objective

把现有 `paper` / `mock_live` 执行骨架推进到接近真实 live 约束、但仍可控可验证的状态。

## Scope Inputs

- 根目录 `ROADMAP.md` 的 P0.1 条目
- `docs/superpowers/specs/2026-03-17-phase29a-strategy-paper-execution-kernel-design.md`
- 当前 execution kernel、`mock_live`、request lifecycle 相关代码与测试

## Must Cover

- delayed / partial fills
- `Unknown` 注入与恢复
- open-order 扫描与 reconciliation
- 网络故障模拟
- account / order reconciliation scaffolding

## Non-Goals

- 真正的 broker `live` 下单
- 与 execution mainline 无关的 UX / infra 扩展
