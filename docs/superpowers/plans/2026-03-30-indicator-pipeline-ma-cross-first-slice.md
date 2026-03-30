# Indicator Pipeline MA Cross First Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the first indicator-pipeline slice around `strategy/ma_cross`, with stable instance IDs, lookback/warmup metadata, cache-safe keys, and a unified output model.

**Architecture:** Keep existing batch indicator functions in `src/analysis/indicators.rs` unchanged and add a thin pipeline layer beside them. Use `ConfiguredStrategyInstance -> IndicatorPipelineConfig` as the first real adapter, route `ma_cross` through the new pipeline, and keep the first slice limited to `sma/ema/rsi` with `ma_cross` consuming only `sma`.

**Tech Stack:** Rust, `serde`/`serde_json`, `rust_decimal`, existing strategy registry/config modules, focused integration tests under `tests/`, GitNexus impact analysis, `cargo test`

---

### Task 1: Add Indicator Config, Instance IDs, and Strategy Mapping

**Files:**
- Create: `src/analysis/indicator_config.rs`
- Modify: `src/analysis/mod.rs`
- Test: `tests/indicator_pipeline_test.rs`

- [ ] **Step 1: Review existing strategy-config shape and impact existing analysis exports**

Run:
```bash
gitnexus_impact({repo: "quantix-rust", target: "ConfiguredStrategyInstance", direction: "upstream", includeTests: true, maxDepth: 3})
gitnexus_impact({repo: "quantix-rust", target: "mod", direction: "upstream", includeTests: true})
```
Expected: `ConfiguredStrategyInstance` is used by strategy config/registry and no HIGH/CRITICAL blast radius appears for the analysis module export change.

- [ ] **Step 2: Write the failing config/mapping tests**

Create `tests/indicator_pipeline_test.rs` with tests that assert:

```rust
#[test]
fn config_maps_ma_cross_to_two_sma_instances() {
    let cfg = ConfiguredStrategyInstance {
        id: "ma_fast_5_slow_20".into(),
        name: "ma_cross".into(),
        enabled: true,
        params: serde_json::json!({"fast": 5, "slow": 20}),
    };

    let pipeline = IndicatorPipelineConfig::try_from(&cfg).unwrap();

    assert_eq!(pipeline.indicators.len(), 2);
    assert_eq!(pipeline.indicators[0].instance_id.0, "sma:period=5");
    assert_eq!(pipeline.indicators[1].instance_id.0, "sma:period=20");
}

#[test]
fn config_rejects_non_ma_cross_first_slice() {
    let cfg = ConfiguredStrategyInstance {
        id: "unknown".into(),
        name: "momentum".into(),
        enabled: true,
        params: serde_json::json!({}),
    };

    assert!(IndicatorPipelineConfig::try_from(&cfg).is_err());
}
```

- [ ] **Step 3: Run test to verify it fails**

Run:
```bash
cargo test --test indicator_pipeline_test config_ -- --nocapture
```
Expected: FAIL because `indicator_config.rs` and its exported types do not exist yet.

- [ ] **Step 4: Write minimal implementation**

Create `src/analysis/indicator_config.rs` with the smallest useful surface:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorInstanceId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorSpec {
    pub name: String,
    pub params: std::collections::HashMap<String, serde_json::Value>,
    pub instance_id: IndicatorInstanceId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndicatorPipelineConfig {
    pub indicators: Vec<IndicatorSpec>,
}
```

Add:

- stable `instance_id` generation with sorted param keys
- `impl TryFrom<&ConfiguredStrategyInstance> for IndicatorPipelineConfig`
- first-slice support only for `ma_cross`

Update `src/analysis/mod.rs` to export the new config types.

- [ ] **Step 5: Run test to verify it passes**

Run:
```bash
cargo test --test indicator_pipeline_test config_ -- --nocapture
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/analysis/indicator_config.rs src/analysis/mod.rs tests/indicator_pipeline_test.rs
git commit -m "feat: add indicator pipeline config mapping"
```

### Task 2: Add Registry Metadata and Unified Output Types

**Files:**
- Create: `src/analysis/indicator_registry.rs`
- Modify: `src/analysis/mod.rs`
- Test: `tests/indicator_pipeline_test.rs`

- [ ] **Step 1: Write the failing registry/output tests**

Extend `tests/indicator_pipeline_test.rs` with:

```rust
#[test]
fn registry_reports_sma_metadata() {
    let registry = IndicatorRegistry::register_builtin();
    let spec = spec("sma", &[("period", 5)]);

    let descriptor = registry.descriptor(&spec).unwrap();
    assert_eq!(descriptor.meta.lookback, 5);
    assert_eq!(descriptor.meta.warmup_len, 4);
}

#[test]
fn registry_computes_scalar_series_for_sma() {
    let registry = IndicatorRegistry::register_builtin();
    let input = close_input("demo", &[1, 2, 3, 4, 5]);
    let spec = spec("sma", &[("period", 3)]);

    let output = registry.compute(&spec, &input).unwrap();
    assert!(matches!(output, IndicatorSeries::ScalarSeries(_)));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test indicator_pipeline_test registry_ -- --nocapture
```
Expected: FAIL because registry types and enum outputs do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Create `src/analysis/indicator_registry.rs` with:

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
}
```

Add `IndicatorRegistry::register_builtin()`, `descriptor(&IndicatorSpec)`, and `compute(&IndicatorSpec, &IndicatorInput)` for only:

- `sma`
- `ema`
- `rsi`

Use the existing functions in `src/analysis/indicators.rs`; do not rewrite indicator math.

Update `src/analysis/mod.rs` exports.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test indicator_pipeline_test registry_ -- --nocapture
```
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/analysis/indicator_registry.rs src/analysis/mod.rs tests/indicator_pipeline_test.rs
git commit -m "feat: add indicator registry metadata and outputs"
```

### Task 3: Add Cache and Pipeline Runner

**Files:**
- Create: `src/analysis/indicator_cache.rs`
- Create: `src/analysis/pipeline.rs`
- Modify: `src/analysis/mod.rs`
- Test: `tests/indicator_pipeline_test.rs`

- [ ] **Step 1: Write the failing cache/pipeline tests**

Extend `tests/indicator_pipeline_test.rs` with:

```rust
#[test]
fn cache_key_keeps_sma_instances_separate() {
    let k1 = IndicatorCacheKey::new("demo", "sma:period=5", (0, 5));
    let k2 = IndicatorCacheKey::new("demo", "sma:period=20", (0, 5));
    assert_ne!(k1, k2);
}

#[test]
fn pipeline_returns_both_sma_instances_without_overwrite() {
    let mut pipeline = IndicatorPipeline::with_builtin_registry();
    let input = close_input("000001:1d", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
                                           11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
    let config = pipeline_config(&[("sma", 5), ("sma", 20)]);

    let output = pipeline.run(&config, &input).unwrap();
    assert!(output.contains_key(&IndicatorInstanceId("sma:period=5".into())));
    assert!(output.contains_key(&IndicatorInstanceId("sma:period=20".into())));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:
```bash
cargo test --test indicator_pipeline_test cache_ pipeline_ -- --nocapture
```
Expected: FAIL because cache and pipeline modules do not exist yet.

- [ ] **Step 3: Write minimal implementation**

Create `src/analysis/indicator_cache.rs` with:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IndicatorCacheKey {
    pub dataset_fingerprint: String,
    pub instance_id: IndicatorInstanceId,
    pub range: (usize, usize),
}
```

Create `src/analysis/pipeline.rs` with a small runner:

```rust
pub struct IndicatorPipeline {
    registry: IndicatorRegistry,
    cache: IndicatorCache,
}
```

Add:

- `IndicatorInput` carrying the close series plus cache-fingerprint fields
- `IndicatorPipeline::with_builtin_registry()`
- `IndicatorPipeline::run(&IndicatorPipelineConfig, &IndicatorInput) -> Result<IndicatorOutputMap>`

The runner must:

- fetch descriptor metadata first
- reject inputs shorter than `lookback`
- preserve warmup `None` values from the underlying series
- use `dataset_fingerprint + instance_id + range` cache keys

Update `src/analysis/mod.rs` exports.

- [ ] **Step 4: Run test to verify it passes**

Run:
```bash
cargo test --test indicator_pipeline_test cache_ pipeline_ -- --nocapture
```
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/analysis/indicator_cache.rs src/analysis/pipeline.rs src/analysis/mod.rs tests/indicator_pipeline_test.rs
git commit -m "feat: add indicator pipeline runtime"
```

### Task 4: Route `ma_cross` Through the Pipeline

**Files:**
- Modify: `src/strategy/registry.rs`
- Test: `tests/strategy_daemon_test.rs`
- Test: `tests/indicator_pipeline_test.rs`

- [ ] **Step 1: Review blast radius for existing strategy symbols**

Run:
```bash
gitnexus_impact({repo: "quantix-rust", target: "StrategyRegistry", direction: "upstream", includeTests: true, maxDepth: 3})
gitnexus_impact({repo: "quantix-rust", target: "ConfiguredStrategyEvaluator", direction: "upstream", includeTests: true, maxDepth: 3})
```
Expected: No HIGH/CRITICAL risk. If HIGH/CRITICAL appears, stop and review before editing.

- [ ] **Step 2: Write the failing strategy-integration tests**

Extend `tests/strategy_daemon_test.rs` with:

```rust
#[test]
fn strategy_registry_ma_cross_uses_pipeline_lookback() {
    let registry = StrategyRegistry::new();
    let evaluator = registry.build(&configured_ma_cross(5, 20)).unwrap();
    assert_eq!(evaluator.lookback_required(), 20);
}

#[test]
fn strategy_registry_ma_cross_pipeline_still_emits_buy_signal() {
    let registry = StrategyRegistry::new();
    let evaluator = registry.build(&configured_ma_cross(2, 3)).unwrap();
    let bars = vec![kline(1, 10), kline(2, 10), kline(3, 10), kline(4, 9), kline(5, 9), kline(6, 20)];
    let envelope = evaluator.evaluate(&bars).unwrap();
    assert_eq!(envelope.signal, Signal::Buy);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run:
```bash
cargo test --test strategy_daemon_test strategy_registry_ -- --nocapture
```
Expected: FAIL after test expectations are updated to require the pipeline-backed behavior.

- [ ] **Step 4: Write minimal implementation**

Modify `src/strategy/registry.rs` so `MaCrossEvaluator` no longer computes `ma()` directly.

Implement this shape:

```rust
struct MaCrossEvaluator {
    fast_id: IndicatorInstanceId,
    slow_id: IndicatorInstanceId,
    lookback: usize,
    pipeline_config: IndicatorPipelineConfig,
    pipeline: std::sync::Mutex<IndicatorPipeline>,
}
```

`from_config()` should:

- convert `ConfiguredStrategyInstance` via `IndicatorPipelineConfig::try_from`
- extract the two `sma` instance IDs
- set `lookback = slow`

`evaluate()` should:

- build `IndicatorInput` from `klines`
- call the pipeline
- read the two scalar series by `fast_id` and `slow_id`
- reproduce the existing cross-over signal semantics without changing public behavior

- [ ] **Step 5: Run test to verify it passes**

Run:
```bash
cargo test --test strategy_daemon_test strategy_registry_ -- --nocapture
cargo test --test indicator_pipeline_test -- --nocapture
```
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/strategy/registry.rs tests/strategy_daemon_test.rs tests/indicator_pipeline_test.rs
git commit -m "feat: route ma_cross through indicator pipeline"
```

### Task 5: Final Verification

**Files:**
- Modify: `src/analysis/indicator_config.rs`
- Modify: `src/analysis/indicator_registry.rs`
- Modify: `src/analysis/indicator_cache.rs`
- Modify: `src/analysis/pipeline.rs`
- Modify: `src/analysis/mod.rs`
- Modify: `src/strategy/registry.rs`
- Test: `tests/indicator_pipeline_test.rs`
- Test: `tests/strategy_daemon_test.rs`

- [ ] **Step 1: Run focused slice verification**

Run:
```bash
cargo test --test indicator_pipeline_test -- --nocapture
cargo test --test strategy_daemon_test strategy_registry_ -- --nocapture
```
Expected: PASS

- [ ] **Step 2: Run a broader strategy regression check**

Run:
```bash
cargo test --lib strategy::registry -- --nocapture
```
Expected: PASS

- [ ] **Step 3: Inspect the final diff**

Run:
```bash
git diff -- src/analysis/indicator_config.rs src/analysis/indicator_registry.rs src/analysis/indicator_cache.rs src/analysis/pipeline.rs src/analysis/mod.rs src/strategy/registry.rs tests/indicator_pipeline_test.rs tests/strategy_daemon_test.rs
```
Expected: Only the first-slice indicator pipeline and `ma_cross` integration changes appear.

- [ ] **Step 4: Run change-scope verification**

Run:
```bash
git status --short src/analysis src/strategy/registry.rs tests/indicator_pipeline_test.rs tests/strategy_daemon_test.rs
```
Expected: Only the planned files appear.

- [ ] **Step 5: Commit**

```bash
git add src/analysis src/strategy/registry.rs tests/indicator_pipeline_test.rs tests/strategy_daemon_test.rs
git commit -m "test: verify first indicator pipeline slice"
```
