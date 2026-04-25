# Market Strength Commands

本文档补充 `quantix market strength` 与 `quantix market strength-stocks` 的运行前置条件、输出语义和常用调用方式。

## 前置条件

- 先同步申万一级行业引用表：

```bash
quantix risk sync industry --standard shenwan
```

- `market foundation`、`market strength`、`market strength-stocks` 都依赖本地行业 SQLite 引用表。
- `market strength` 与 `market strength-stocks` 还依赖网络侧基础行情与基本面抓取能力。

## 命令说明

### 1. 全市场基础摘要

```bash
quantix market foundation
```

输出全市场 A 股数量、行业覆盖数、未覆盖数，以及行业覆盖 Top10。

### 2. 强弱板块分析

```bash
quantix market strength \
  --date 2026-03-09 \
  --strong-top 3 \
  --weak-top 3 \
  --stock-top 10
```

输出：

- 强势板块 TopN
- 弱势板块 TopN
- 强势板块内个股按总市值排序的 TopN
- 强势板块内个股按上一会计周期净利润排序的 TopN

### 3. 直接查看强势板块个股排行

```bash
quantix market strength-stocks \
  --date 2026-03-09 \
  --strong-top 3 \
  --metric market-cap \
  --top 10
```

参数说明：

- `--strong-top <N>`：先选取涨幅最强的前 N 个行业板块作为候选范围。
- `--metric <market-cap|profit>`：指定排序字段。
- `--top <N>`：输出前 N 条记录。

### 4. 过滤到单个强势行业

```bash
quantix market strength-stocks \
  --date 2026-03-09 \
  --strong-top 3 \
  --sector 银行 \
  --metric market-cap \
  --top 10
```

```bash
quantix market strength-stocks \
  --date 2026-03-09 \
  --strong-top 3 \
  --sector 银行 \
  --metric profit \
  --top 10
```

说明：

- `--sector` 只在强势板块候选范围内再做行业名精确过滤。
- 未指定 `--sector` 时，保持原始强势板块汇总统计口径。
- 指定 `--sector` 时，候选股数和覆盖数会按过滤后的结果重新计算。

## 推荐验收顺序

```bash
quantix market foundation
quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10
quantix market strength-stocks --date 2026-03-09 --strong-top 3 --metric market-cap --top 10
quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10
```

## 当前边界

- 只覆盖日度快照与只读查询。
- `--sector` 目前采用精确行业名匹配，不做模糊归一化。
- 如果某个行业不在本次强势板块 TopN 内，`strength-stocks --sector <NAME>` 会返回空结果。
