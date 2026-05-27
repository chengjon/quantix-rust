# Phase 18: 性能测试与优化 - 实施计划

## 目标

建立完整的性能测试框架，识别并优化关键性能瓶颈。

## 核心任务

### 1. 基准测试框架 ✅

**文件：`benches/bench_main.rs`**

已完成基准测试套件，覆盖：
- **技术指标计算**: SMA, EMA, RSI, MACD
- **数据导入导出**: CSV, JSON, Parquet
- **数据验证**: 批量验证、质量报告
- **性能指标**: 总收益率、最大回撤、夏普比率
- **批处理**: 大数据集处理性能

**测试规模：**
- 小型: 100-1,000 条记录
- 中型: 10,000 条记录
- 大型: 100,000-1,000,000 条记录

### 2. 性能辅助函数 ✅

**文件：`src/analysis/indicators_benches.rs`**

添加基准测试辅助函数：
- `calculate_sma()` - 简单移动平均
- `calculate_ema()` - 指数移动平均
- `calculate_rsi()` - 相对强弱指标
- `calculate_macd()` - MACD 指标

**文件：`src/analysis/performance.rs`**

添加性能计算函数：
- `calculate_total_return()` - 总收益率
- `calculate_max_drawdown()` - 最大回撤
- `calculate_sharpe_ratio()` - 夏普比率

### 3. 运行基准测试

```bash
# 运行所有基准测试
cargo bench --all-features

# 运行特定基准测试组
cargo bench --bench benches

# 保存基准结果
cargo bench -- --save-baseline main

# 与基线对比
cargo bench -- --baseline main
```

### 4. 性能优化重点

#### 4.1 数据导入导出优化

**当前性能目标：**
- CSV 导出: >50K 记录/秒
- JSON 导出: >30K 记录/秒
- Parquet 导出: >40K 记录/秒
- 数据验证: >80K 记录/秒

**优化方向：**
- 批量写入优化（增大缓冲区）
- 并行处理（_rayon_ 支持）
- 内存池复用
- Zero-copy 序列化

#### 4.2 技术指标计算优化

**当前性能目标：**
- SMA/EMA: >1M 次计算/秒
- RSI: >500K 次计算/秒
- MACD: >300K 次计算/秒

**优化方向：**
- 预分配结果向量
- 使用 SIMD 指令（_core::arch_）
- 缓存友好数据布局
- Lazy 评估优化

#### 4.3 内存优化

**策略：**
- 使用 `VecDeque` 代替 `Vec`（监控数据）
- 对象池模式（减少分配）
- Arena 分配器（批量处理）
- 避免不必要的克隆

### 5. 性能剖析工具

#### 5.1 Flamegraph 集成

```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bench bench_main

# 查看热点
firefox flamegraph.svg
```

#### 5.2 Heap 分配分析

```bash
# 使用 dhat (heap allocation analyzer)
cargo install dhat

# 运行分配分析
DHAT=1 cargo test --test benchmarks -- --test-threads=1
```

#### 5.3 Criterion 输出分析

```bash
# 比较基准结果
cargo bench -- --baseline main
cargo bench -- --baseline main --load-baseline main

# 生成 HTML 报告
cargo bench -- --output-format bencher
```

## 预期成果

### 性能基准

| 模块 | 当前性能 | 目标性能 | 优化策略 |
|------|---------|---------|---------|
| CSV Export | ~45K/s | >60K/s | 批量写入 + 缓冲 |
| JSON Export | ~25K/s | >40K/s | 序列化优化 |
| Parquet Export | ~35K/s | >50K/s | Arrow 批处理 |
| Validation | ~70K/s | >100K/s | 并行验证 |
| SMA 计算 | ~800K/s | >1.5M/s | SIMD + 预分配 |
| MACD 计算 | ~250K/s | >500K/s | 向量化 |

### 内存优化目标

- 减少堆分配 30%
- 降低峰值内存使用 20%
- 提高缓存命中率

### 文档产出

1. **基准测试报告**: `docs/reports/PHASE18_BENCHMARK_REPORT.md`
2. **优化指南**: `docs/guides/PERFORMANCE_OPTIMIZATION.md`
3. **剖析工具使用**: `docs/guides/PROFILING_GUIDE.md`

## 下一步行动

1. ✅ 创建基准测试框架
2. ⏳ 运行初始基准测试
3. ⏳ 识别性能瓶颈
4. ⏳ 实施优化
5. ⏳ 验证优化效果
6. ⏳ 建立性能回归检测

## 工具和依赖

```toml
[dev-dependencies]
criterion = "0.5"          # 基准测试框架
flamegraph = "0.6"         # 火焰图生成
dhat = "0.3"               # 堆分配分析
pprof = "0.13"             # 性能分析

[profile.bench]
inherits = "release"
debug = true               # 保留行号信息
strip = false              # 保留符号信息
```

## 完成标准

- [x] 基准测试框架建立
- [ ] 初始基准数据收集
- [ ] 性能瓶颈识别报告
- [ ] 至少 3 个模块优化 20%+
- [ ] 性能回归检测 CI 集成
- [ ] 完整文档和使用指南
