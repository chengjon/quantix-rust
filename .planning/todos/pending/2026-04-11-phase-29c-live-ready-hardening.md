title: Phase 29C live-ready hardening
area: execution
created: 2026-04-11T00:00:00Z
files: ROADMAP.md, docs/superpowers/specs/2026-03-17-phase29a-strategy-paper-execution-kernel-design.md, src/execution

## Problem

根路线图已经把 Phase 29C 定位为当前最优先 backlog，但正式 GSD 结构里还没有针对 delayed/partial fills、`Unknown` 恢复、network fault injection、reconciliation scaffolding 的分计划与验证入口。

## Solution

围绕 `mock_live` 与 execution kernel 补一组可执行计划，优先锁定行为测试，再补 adapter / reconciliation / failure injection 相关实现与文档。

## Why Now

如果不先收口 live-ready hardening，后续 real live / broker execution 会建立在语义和恢复能力不足的基础上。
