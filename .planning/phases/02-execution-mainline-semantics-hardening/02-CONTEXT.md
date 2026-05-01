# Phase 2 Context

## Phase

Phase 2: Execution mainline semantics hardening

## Objective

收紧 execution request 生命周期语义、排障信息与文档边界，避免 request 完成和订单终态混淆。

## Scope Inputs

- 根目录 `ROADMAP.md` 的 P0.2 条目
- `README.md`
- `docs/USER_MANUAL.md`
- daemon / operator / request lifecycle 相关代码

## Must Cover

- `request completed` 与订单终态的语义区分
- daemon/operator 的 request 诊断与可观测性
- `paper` / `mock_live` / `live` 文档边界收紧

## Non-Goals

- 在语义未收敛前直接推进 real live broker 实现
