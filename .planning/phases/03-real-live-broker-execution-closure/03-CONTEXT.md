# Phase 3 Context

## Phase

Phase 3: Real live / broker execution closure

## Objective

在显式 safety gating 前提下，收口 real live adapter、QMT real execution 与 broker path 边界。

## Scope Inputs

- 根目录 `ROADMAP.md` 的 P0.3 条目
- QMT preview-only / live gate 相关 CLI、bridge、handler、文档与测试

## Must Cover

- live adapter contract
- QMT 从 preview-only 到 real execution 的 gating
- broker/live path 的运行边界、验证流程与回归测试

## Non-Goals

- 在没有明确 gate 和验证之前开放默认 live submit
