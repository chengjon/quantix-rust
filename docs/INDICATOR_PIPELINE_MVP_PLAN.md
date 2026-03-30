# 指标工厂 + 缓存 + 配置驱动 MVP 落地清单（可执行版）

> 目标：在不破坏现有指标实现的前提下，新增“配置驱动 + 指标工厂 + 缓存”最小可行结构，并支持后续扩展到流式计算。

---

## 1. 总体设计（最小可行结构）

### 1.1 核心思路

- **保持现有批量指标函数不变**（如 `sma/ema/rsi/macd`）。
- 新增 **配置层** 和 **指标注册/工厂层**，实现“通过配置选择指标 + 参数”。
- 新增 **策略配置适配层**，让 `ConfiguredStrategyInstance` 也能映射到统一内部配置。
- 新增 **缓存层**，以 `dataset_fingerprint + instance_id + range` 作为 key，避免重复计算。
- 第一批真实消费者冻结为：`strategy/ma_cross`

### 1.2 MVP 结构图

```
analysis/
├── indicators.rs                # 现有批量指标实现（保持不动）
├── indicator_config.rs          # 新增：配置模型 + instance_id + strategy映射
├── indicator_registry.rs        # 新增：指标工厂/注册表
├── indicator_cache.rs           # 新增：缓存层
└── pipeline.rs                  # 新增：统一入口（MVP 必做）
```

### 1.3 最小接口模型

- `IndicatorSpec`
  - `name: String`
  - `params: HashMap<String, Value>`
  - `instance_id: IndicatorInstanceId`

- `IndicatorPipelineConfig`
  - `indicators: Vec<IndicatorSpec>`

- `IndicatorMeta`
  - `canonical_name: &'static str`
  - `lookback: usize`
  - `warmup_len: usize`

- `IndicatorSeries`
  - `ScalarSeries(Vec<Option<Decimal>>)`
  - `MacdSeries(Vec<Option<MacdPoint>>)`
  - `KdjSeries(Vec<Option<KdjPoint>>)`
  - `AtrSeries(Vec<Option<Decimal>>)`

- `IndicatorRegistry`
  - `register_builtin()`
  - `descriptor(name, params) -> Result<IndicatorDescriptor>`
  - `compute(instance, input) -> Result<IndicatorSeries>`

- `IndicatorCache`
  - `get_or_compute(key, || compute())`

---

## 2. 可执行落地清单（按文件/接口/估算工时）

> 估算以“单人、熟悉仓库”为基准；工作量给出 0.5 天/1 天等粗粒度。

### 2.1 `src/analysis/indicator_config.rs`

**接口设计**

- `IndicatorInstanceId(pub String)`
- `IndicatorSpec`
  - `name: String`
  - `params: HashMap<String, serde_json::Value>`
  - `instance_id: IndicatorInstanceId`
- `IndicatorPipelineConfig`
  - `indicators: Vec<IndicatorSpec>`
- `impl TryFrom<&ConfiguredStrategyInstance> for IndicatorPipelineConfig`

**估算工作量**：0.5 天

**验收**

- 能从 JSON/TOML 反序列化
- `IndicatorSpec` 可表达任意指标参数
- `ConfiguredStrategyInstance(name=ma_cross)` 能映射为两个 `sma` 实例（fast/slow）
- `instance_id` 生成稳定且可复现

---

### 2.2 `src/analysis/indicator_registry.rs`

**接口设计**

- `type IndicatorFn = fn(&IndicatorParams, &IndicatorInput) -> Result<IndicatorSeries>`
- `IndicatorDescriptor { meta, compute_fn }`
- `IndicatorRegistry::register_builtin()`
- `IndicatorRegistry::descriptor(name, params)`
- `IndicatorRegistry::compute(instance, input)`

**最小实现范围**

- 先注册 3 个指标：`sma` / `ema` / `rsi`

**估算工作量**：1 天

**验收**

- 通过实例配置完成指标计算
- 可返回 `lookback/warmup` 元数据
- 参数缺失/非法时返回明确错误

---

### 2.3 `src/analysis/indicator_cache.rs`

**接口设计**

- `IndicatorCacheKey { dataset_fingerprint, instance_id, range }`
- `IndicatorCache::get_or_compute(key, || compute())`

**估算工作量**：0.5 天

**验收**

- 同一 key 重复计算不重复执行
- cache miss 能正常回退计算
- `sma(5)` 与 `sma(20)` 不会相互污染

---

### 2.4 `src/analysis/pipeline.rs`

**接口设计**

- `run_pipeline(config, dataset) -> IndicatorOutputMap`
- 内部串联 registry + cache
- `ma_cross` 第一批通过该入口消费结果

**估算工作量**：0.5 天

**验收**

- 能对配置中的多个指标依次计算并输出
- 输出键按 `instance_id` 唯一稳定

---

### 2.5 单元测试（新增）

**文件建议**

- `src/analysis/indicator_registry.rs` 内部测试
- `src/analysis/indicator_cache.rs` 内部测试
- `src/analysis/indicator_config.rs` 内部测试（策略映射）

**估算工作量**：0.5 天

**验收**

- 注册表能正确路由
- 缓存命中/失效逻辑通过测试
- `ma_cross -> sma(fast/slow)` 映射通过测试

---

## 3. 最小可行结构（MVP 设计建议）

### 3.1 数据结构定义（示意）

```rust
pub struct IndicatorInstanceId(pub String);

pub struct IndicatorSpec {
    pub name: String,
    pub params: HashMap<String, serde_json::Value>,
    pub instance_id: IndicatorInstanceId,
}

pub struct IndicatorPipelineConfig {
    pub indicators: Vec<IndicatorSpec>,
}
```

### 3.2 注册表与工厂（示意）

```rust
pub enum IndicatorSeries {
    ScalarSeries(Vec<Option<Decimal>>),
    MacdSeries(Vec<Option<MacdPoint>>),
    KdjSeries(Vec<Option<KdjPoint>>),
    AtrSeries(Vec<Option<Decimal>>),
}

pub struct IndicatorMeta {
    pub canonical_name: &'static str,
    pub lookback: usize,
    pub warmup_len: usize,
}

pub struct IndicatorDescriptor {
    pub meta: IndicatorMeta,
    pub compute_fn: IndicatorFn,
}
```

### 3.3 缓存层（示意）

```rust
pub struct IndicatorCacheKey {
    pub dataset_fingerprint: String,
    pub instance_id: IndicatorInstanceId,
    pub range: (usize, usize),
}

pub struct IndicatorCache {
    store: HashMap<IndicatorCacheKey, IndicatorSeries>,
}

impl IndicatorCache {
    pub fn get_or_compute<F>(&mut self, key: IndicatorCacheKey, compute: F) -> Result<IndicatorSeries>
    where
        F: FnOnce() -> Result<IndicatorSeries>,
    {
        if let Some(cached) = self.store.get(&key) {
            return Ok(cached.clone());
        }
        let value = compute()?;
        self.store.insert(key, value.clone());
        Ok(value)
    }
}
```

### 3.4 配置驱动入口（示意）

```rust
pub fn run_pipeline(
    config: &IndicatorPipelineConfig,
    dataset: &IndicatorInput,
    registry: &IndicatorRegistry,
    cache: &mut IndicatorCache,
) -> Result<IndicatorOutputMap> {
    let mut outputs = HashMap::new();
    for spec in &config.indicators {
        let key = IndicatorCacheKey::from(spec, dataset);
        let output = cache.get_or_compute(key, || registry.compute(spec, dataset))?;
        outputs.insert(spec.instance_id.clone(), output);
    }
    Ok(outputs)
}
```

### 3.5 `ma_cross` 第一批接入（示意）

```rust
let config = IndicatorPipelineConfig::try_from(configured_strategy_instance)?;
let outputs = run_pipeline(&config, dataset, registry, cache)?;

let fast = outputs.get(&IndicatorInstanceId("sma:period=5".into()));
let slow = outputs.get(&IndicatorInstanceId("sma:period=20".into()));
```

---

## 4. 落地顺序建议（最小风险）

1. `indicator_config.rs`（配置模型 + instance_id + `ConfiguredStrategyInstance` 映射）
2. `indicator_registry.rs`（注册表/工厂 + `IndicatorMeta`）
3. `indicator_cache.rs`（缓存层）
4. `pipeline.rs`（串联入口）
5. `ma_cross` 单点接入
6. 单元测试

---

## 5. 验收清单（最小）

- [ ] 配置可解析（JSON/TOML 任一即可）
- [ ] `ConfiguredStrategyInstance(name=ma_cross)` 可映射为统一内部配置
- [ ] 通过配置计算 SMA/EMA/RSI
- [ ] `sma(5)` 与 `sma(20)` 不覆盖
- [ ] 缓存命中可减少重复计算
- [ ] 单测覆盖注册表 + 缓存 + 策略映射

---

## 6. 后续扩展（不在 MVP 内）

- 流式/增量指标状态（SMA/EMA rolling state）
- 指标版本化与文档系统
- 更复杂的组合指标 / 多指标依赖图
- `macd/kdj/atr` 正式接入统一 enum 输出
