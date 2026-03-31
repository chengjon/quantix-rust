# 指标体系优化方案（含缺口与可实施接口）

> 本文档聚焦“先定契约，再做工厂/缓存/配置化”，避免新抽象与现有调用方脱节。  
> `docs/INDICATOR_PIPELINE_MVP_PLAN.md` 作为子文档，承载分任务实施清单。

---

## 1. 当前缺口（与规范对齐）

| 规范项 | 当前状态 | 关键风险 |
|---|---|---|
| 回测/实盘双模式 | ⚠️ 部分缺口 | 缺少统一 warmup/lookback 语义，批量与流式行为可能不一致 |
| 指标工厂注册 | ❌ 缺口 | 容易引入“仅新接口可用”，和现有调用方割裂 |
| 缓存机制 | ❌ 缺口 | key 设计不统一会造成脏缓存或低命中 |
| 参数配置化 | ❌ 缺口 | 配置层若不绑定现有输入输出模型，落地成本高 |
| 多实例输出模型 | ❌ 缺口 | 同名指标不同参数实例（如 `ma(5)`/`ma(20)`）容易覆盖冲突 |

---

## 2. 必须先冻结的关键契约（先决条件）

## 2.1 Lookback / Warmup 契约

### 目标

统一回答三个问题：
1. 指标最少需要多少输入（`lookback`）？
2. 在 warmup 区间如何表达“不可用值”？
3. 批量与流式在同样输入下，输出是否一致？

### 统一定义（MVP）

- `lookback`: 计算某指标“第一个有效值”所需最小样本数（如 SMA(20) = 20）
- `warmup_len = lookback - 1`
- warmup 区间输出统一为 `null`（Rust 侧 `Option::None`）
- 批量模式与流式模式必须满足：
  - 对同一输入序列、同一参数，末端有效值一致（误差容忍仅用于浮点）

### 接口建议

```rust
pub struct IndicatorMeta {
    pub canonical_name: &'static str,
    pub lookback: usize,
    pub warmup_len: usize,
}
```

---

## 2.2 多实例输出模型契约（防覆盖）

### 问题

策略通常同时使用多个同类指标实例（如 `sma(5)` + `sma(20)`）。若输出仅按 `name` 存储，会发生覆盖。

### 统一模型（MVP）

引入 `IndicatorInstanceId` 作为主键，而不是只用指标名。

- `IndicatorInstanceId` 由 `name + canonical_params` 组成
- 输出结构按实例维度存储

### 接口建议

```rust
pub struct IndicatorInstanceId(pub String); // 例如 "sma:period=5"

pub struct IndicatorOutputSeries {
    pub instance_id: IndicatorInstanceId,
    pub values: IndicatorSeries,
    pub meta: IndicatorMeta,
}

pub type IndicatorOutputMap = std::collections::HashMap<IndicatorInstanceId, IndicatorOutputSeries>;
```

### 约束

- `instance_id` 生成规则必须稳定（参数按 key 排序后序列化）
- `display_name` 可读，`instance_id` 可计算且稳定

---

## 2.3 第一批接入点契约（避免大爆炸改造）

### 原则

只改“最短闭环”，先打通端到端，避免一次性重构全策略层。

### MVP 第一批接入点（冻结）

1. **离线/回测路径**：`ma_cross` 相关指标调用点（SMA 优先）
2. **统一入口模块**：新增 `analysis/pipeline.rs`，先作为“新能力门面”，不改旧函数签名
3. **配置入口**：内部统一成 `IndicatorPipelineConfig`，外部先支持：
   - 最小 JSON/TOML 配置（仅 `sma/ema/rsi`）
   - `ConfiguredStrategyInstance` 映射到同一内部结构

### 明确不在第一批

- 不改所有策略
- 不一次性覆盖 MACD/KDJ/ATR 全家桶
- 不先上复杂 DAG 依赖调度

---

## 2.4 策略配置映射契约（第一批必须冻结）

### 背景

既然第一批真实消费者已经冻结为 `strategy/ma_cross`，文档就必须明确：

- 现有 `ConfiguredStrategyInstance` 如何映射到统一内部配置
- 策略层如何消费 pipeline 输出
- 这条路径不能要求调用方先改成全新的配置格式

### MVP 规则

- 现有策略配置输入保持不变，源模型是 `ConfiguredStrategyInstance`
- 新增一个适配层，将 `ConfiguredStrategyInstance` 转为 `IndicatorPipelineConfig`
- 对 `ma_cross(fast, slow)`：
  - 生成两个指标实例：`sma(period=fast)` 与 `sma(period=slow)`
  - `IndicatorInstanceId` 分别形如：
    - `sma:period=5`
    - `sma:period=20`
- `ma_cross` 策略逻辑不再直接调用 `ma()`，而是消费 pipeline 输出结果

### 接口建议

```rust
impl TryFrom<&ConfiguredStrategyInstance> for IndicatorPipelineConfig {
    type Error = QuantixError;
}
```

### 约束

- 第一批只要求支持 `ma_cross`
- `ConfiguredStrategyInstance.name != "ma_cross"` 时允许直接返回“暂不支持”
- 旧指标函数签名不变，适配层只负责“转配置 + 调 pipeline”

---

## 2.5 统一输出模型契约（MVP）

### 背景

第一批接入 `ma_cross` 只需要标量序列，但第二阶段已明确会扩展到 `MACD/KDJ/ATR`。若现在仍将主契约定为 `Vec<Option<Decimal>>`，后续扩展会再次破坏接口。

### 推荐模型

```rust
pub enum IndicatorSeries {
    ScalarSeries(Vec<Option<rust_decimal::Decimal>>),
    MacdSeries(Vec<Option<MacdPoint>>),
    KdjSeries(Vec<Option<KdjPoint>>),
    AtrSeries(Vec<Option<rust_decimal::Decimal>>),
}

pub struct IndicatorOutputSeries {
    pub instance_id: IndicatorInstanceId,
    pub values: IndicatorSeries,
    pub meta: IndicatorMeta,
}
```

### 说明

- 第一批注册指标 `sma/ema/rsi` 统一走 `ScalarSeries`
- 第二阶段扩展 `macd/kdj/atr` 时不需要重写工厂/缓存/pipeline 的外层接口
- `IndicatorOutputMap` 仍按 `IndicatorInstanceId` 聚合

---

## 3. 最小可行架构（在关键契约之上）

```
analysis/
├── indicators.rs                # 现有批量计算函数（保留）
├── indicator_config.rs          # 新增：配置模型（含实例ID与策略映射）
├── indicator_registry.rs        # 新增：工厂/注册表（含lookback元数据）
├── indicator_cache.rs           # 新增：缓存（key=dataset+instance_id+window）
└── pipeline.rs                  # 新增：统一执行入口
```

### 数据流

1. 读取 `IndicatorPipelineConfig`
2. 为每个 `IndicatorSpec` 生成稳定 `instance_id`
3. 从 registry 获取 `meta/lookback` 并校验输入长度
4. 按实例 key 查询缓存，miss 时计算
5. 产出 `IndicatorOutputMap`
6. `ma_cross` 通过 `instance_id` 读取快慢均线序列并生成信号

---

## 4. 缓存 key 契约（MVP）

```rust
pub struct IndicatorCacheKey {
    pub dataset_fingerprint: String,
    pub instance_id: IndicatorInstanceId,
    pub range: (usize, usize), // 预留：窗口化场景
}
```

### 说明

- `instance_id` 保证“同名不同参”不冲突
- `dataset_fingerprint` 至少包含：
  - 数据源标识（如策略/CLI/回测上下文）
  - 标的代码 / 周期
  - 时间范围或窗口边界
  - 复权/数据版本语义
  - 输入列集合（如 `close` 或 `high+low+close`）
  - 排序语义（默认按时间升序）
- 后续流式可扩展为“append-only 增量更新”

---

## 5. 实施顺序（按安全性）

1. 冻结基础契约（本文件 2.1 / 2.2 / 2.3）
2. 冻结策略配置映射与统一输出模型（本文件 2.4 / 2.5）
3. `indicator_config.rs`（实例 ID 生成、参数规范化、`ConfiguredStrategyInstance` 映射）
4. `indicator_registry.rs`（注册 `sma/ema/rsi` + `IndicatorMeta`）
5. `indicator_cache.rs`（`get_or_compute`）
6. `pipeline.rs`（连通 config + registry + cache）
7. `ma_cross` 单点接入（第一批调用方）

---

## 6. 验收标准（必须满足）

- [ ] `lookback/warmup` 行为可测试且统一（批量输出 warmup 为 `None`）
- [ ] 同时配置 `sma(5)` 与 `sma(20)` 不覆盖，输出键唯一稳定
- [ ] `ConfiguredStrategyInstance(name=ma_cross)` 可映射到统一内部配置
- [ ] 输出主契约为统一 enum，而不是仅 `Vec<Option<Decimal>>`
- [ ] 第一批接入点（`ma_cross`）可在不破坏旧接口的前提下运行
- [ ] 缓存命中后不重复计算，且不会跨参数污染

---

## 7. 与子文档的关系

- 本文档：定义“接口契约与边界”
- 子文档 `docs/INDICATOR_PIPELINE_MVP_PLAN.md`：定义“按文件/工时/任务拆分”的落地执行计划
