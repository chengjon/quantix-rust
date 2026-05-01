title: Define QMT live execution boundary
area: bridge
created: 2026-04-11T00:10:00Z
files: README.md, docs/USER_MANUAL.md, src/bridge, src/cli/handlers

## Problem

仓库已经有 QMT preview-only 路径和 live gate 相关回归测试，但 real execution 的契约、显式 gating 条件和文档边界还没有在正式规划中单独固定下来。

## Solution

把 QMT 从 preview-only 到真实执行的差异、运行边界、安全前置条件和测试要求固化成 Phase 3 计划输入，避免后续实现漂移。

## Why Now

这是 README 和用户手册里仍明确未完成的缺口，也是当前最容易被误读为“已经支持 live”的区域。
