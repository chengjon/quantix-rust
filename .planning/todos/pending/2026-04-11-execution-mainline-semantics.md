title: Tighten execution mainline semantics
area: execution
created: 2026-04-11T00:05:00Z
files: README.md, docs/USER_MANUAL.md, src/cli, src/strategy

## Problem

当前 backlog 已明确指出 `request completed` 与订单终态之间仍可能被混淆，daemon/operator 侧的 request 排障与可观测信息也还不够收敛。

## Solution

先审视现有 request 生命周期、CLI/daemon 输出与文档文案，明确应补的状态语义、诊断信息和验证路径，再形成 Phase 2 的计划。

## Why Now

这项工作是从 live-ready hardening 走向 real live / broker execution 的语义前提，不能等到 broker 接入后再补。
