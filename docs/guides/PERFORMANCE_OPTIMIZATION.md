# 性能优化指南

## 概述

本指南提供 quantix-rust 项目的性能优化最佳实践和工具使用方法。

## 目录

1. [基准测试](#基准测试)
2. [性能剖析](#性能剖析)
3. [优化策略](#优化策略)
4. [持续监控](#持续监控)

---

## 基准测试

### 运行基准测试

```bash
# 运行所有基准测试
cargo bench --all-features

# 运行特定基准测试组
cargo bench --bench bench_main

# 保存基线
cargo bench -- --save-baseline main

# 与基线对比
cargo bench -- --baseline main
```

### 基准测试组织结构

```
benches/
└── bench_main.rs          # 主基准测试套件
    ├── bench_indicators   # 技术指标计算基准
    ├── bench_export       # 数据导出基准
    ├── bench_validation   # 数据验证基准
    ├── bench_performance  # 性能指标基准
    └── bench_batch        # 批处理基准
```

### 编写新基准测试

```rust
use criterion::{black_box, Bencher, BenchmarkId, Criterion};

fn bench_my_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_module");

    for size in [100, 1000, 10000].iter() {
        let data = generate_test_data(*size);

        group.bench_with_input(
            BenchmarkId::new("function_name", size),
            &data,
            |b, data| {
                b.iter(|| {
                    my_function(black_box(data))
                })
            }
        );
    }

    group.finish();
}
```

### 解释基准测试结果

```
indicators/sma_5          time:   [2.1456 µs 2.1532 µs 2.1619 µs]
                        change: [-2.345% -1.234% -0.123%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**指标说明：**
- `time`: 平均执行时间（置信区间）
- `change`: 与基线对比的性能变化
- `p`: 统计显著性 (p < 0.05 表示显著)

---

## 性能剖析

### Flamegraph（火焰图）

火焰图可视化 CPU 使用情况，帮助识别热点代码。

```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bench bench_main

# 查看火焰图
firefox flamegraph.svg
```

**解读火焰图：**
- 横轴宽度 = CPU 时间占比
- 纵向堆栈 = 调用栈深度
- 宽的平顶函数 = 优化目标

### DHat（堆分配分析）

分析堆内存分配，减少 GC 压力。

```bash
# 安装 dhat
cargo install dhat

# 运行分配分析
DHAT=1 cargo test --bench bench_main -- --test-threads=1

# 查看结果
dhat-ddhat target/dhat.heap
```

**优化建议：**
- 预分配容器容量
- 使用 `Vec::with_capacity()`
- 避免不必要的克隆
- 重用对象（对象池）

### Criterion 输出分析

```bash
# 生成详细报告
cargo bench -- --output-format pretty

# 生成 JSON（用于 CI/CD）
cargo bench -- --output-format json > results.json
```

---

## 优化策略

### 1. 批量操作优化

**问题：** 小批量操作开销大

**解决方案：**
```rust
// ❌ 小批量
for chunk in data.chunks(10) {
    process(chunk);
}

// ✅ 最优批量
for chunk in data.chunks(1000) {
    process(chunk);
}
```

**基准测试确定最优批次大小：**
```rust
for batch_size in [100, 500, 1000, 5000, 10000].iter() {
    group.bench_with_input(
        BenchmarkId::new("batch_size", batch_size),
        batch_size,
        |b, size| {
            b.iter(|| process_in_batches(data, *size))
        }
    );
}
```

### 2. 并行处理优化

**CPU 密集型任务：**
```rust
use rayon::prelude::*;

// ✅ 并行迭代器
data.par_iter()
    .map(|item| process_item(item))
    .collect()
```

**IO 密集型任务：**
```rust
use tokio::task::JoinSet;
use futures::stream::{self, StreamExt};

// ✅ 并发 IO
let mut tasks = JoinSet::new();
for item in items {
    tasks.spawn(async move {
        process_item_async(item).await
    });
}

while let Some(result) = tasks.next().await {
    handle_result(result);
}
```

### 3. 内存优化

#### 预分配
```rust
// ❌ 逐步增长
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}

// ✅ 预分配
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}
```

#### 避免克隆
```rust
// ❌ 克隆数据
for item in items.clone() {
    process(item);
}

// ✅ 使用引用
for item in &items {
    process(item);
}
```

#### 使用 Cow（Copy-on-Write）
```rust
use std::borrow::Cow;

fn process_data(data: Cow<[u8]>) {
    // 只在修改时才克隆
}

// 调用
process_data(Cow::Borrowed(&data));  // 无克隆
process_data(Cow::Owned(data.to_vec())); // 需要克隆
```

### 4. 算法优化

#### 选择合适的数据结构
```rust
use std::collections::{HashMap, HashSet};

// ✅ 快速查找
let set: HashSet<_> = items.iter().cloned().collect();
if set.contains(&target) { /* ... */ }

// ✅ 键值映射
let map: HashMap<_, _> = items.iter().map(|v| (v.id, v)).collect();
if let Some(value) = map.get(&id) { /* ... */ }
```

#### 使用缓存
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

type Cache = Arc<RwLock<HashMap<String, Decimal>>>;

async fn get_cached_value(
    cache: &Cache,
    key: &str,
    compute_fn: impl Fn() -> Decimal,
) -> Decimal {
    // 快速路径: 读取缓存
    {
        let reader = cache.read().await;
        if let Some(&value) = reader.get(key) {
            return value;
        }
    }

    // 慢速路径: 计算并缓存
    let value = compute_fn();
    let mut writer = cache.write().await;
    writer.insert(key.to_string(), value);
    value
}
```

### 5. Zero-Copy 优化

使用 Arrow 的列式格式进行零拷贝操作：

```rust
use arrow::array::{Float64Array, PrimitiveArray};
use arrow::datatypes::DataType;

// ❌ 拷贝数据
let closes: Vec<f64> = klines.iter().map(|k| k.close.to_f64().unwrap()).collect();

// ✅ 零拷贝
let closes = Float64Array::from(
    klines.iter()
        .map(|k| k.close.to_f64().unwrap())
        .collect::<Vec<f64>>()
);
```

---

## 持续监控

### CI/CD 集成

在 `.github/workflows/benchmark.yml` 中：

```yaml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        run: cargo bench -- --output-format bencher | tee benchmark.txt

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: benchmark.txt
```

### 性能回归检测

保存基线并在 CI 中对比：

```bash
# 初始化基线
cargo bench -- --save-baseline main

# CI 中对比
cargo bench -- --baseline main
```

如果性能下降 >5%，CI 失败。

### 定期性能审查

每周/每月运行完整性能测试：

```bash
# 完整性能测试套件
./scripts/dev/run_benchmarks.sh

# 生成报告
./scripts/dev/run_benchmarks.sh --html

# 生成火焰图
./scripts/dev/run_benchmarks.sh --flamegraph
```

---

## 常见性能问题

### 问题 1: CSV 导出慢

**症状：** 导出 10 万条记录需要 >5 秒

**诊断：**
```bash
cargo bench --bench bench_main export/csv
cargo flamegraph --bench bench_main
```

**优化：**
1. 增大批次大小（5000-10000）
2. 预分配字符串容量
3. 批量写入而非逐行

### 问题 2: 技术指标计算慢

**症状：** 计算 MACD（1000 条）>100ms

**诊断：**
```bash
cargo bench --bench bench_main indicators/macd
cargo flamegraph --bench bench_main
```

**优化：**
1. 预分配结果向量
2. 使用滑动窗口避免重新计算
3. SIMD 优化（整数运算）

### 问题 3: 内存占用高

**症状：** 导出 100 万条记录使用 >2GB 内存

**诊断：**
```bash
DHAT=1 cargo test --bench bench_main -- --test-threads=1
dhat-ddhat target/dhat.heap
```

**优化：**
1. 流式处理（不一次性加载所有数据）
2. 使用 VecDeque 限制历史数据大小
3. 及时释放临时变量

---

## 参考资源

- **Criterion Book**: https://bheisler.github.io/criterion.rs/book/
- **Flamegraph Guide**: https://nnethercote.github.io/perf-book/flamegraphs.html
- **Rust Performance**: https://nnethercote.github.io/perf-book/
- **Optimizing Rust**: https://gist.github.com/jFransham/369a86eff00e5f280ed25121454acec

---

**最后更新**: Phase 18
**维护者**: MyStocks Team
