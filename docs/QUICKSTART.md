# quantix-rust 快速开始指南

## Cursor + WSL 最小前置检查

在开始开发或验证前，先确认下面几项：

### 1. Rust 工具链

```bash
cargo --version
rustc --version
```

- 若以上命令不可用，当前环境只适合做代码阅读、文档审阅和结构一致性检查。
- 要做构建、测试、clippy 或 CI 相关验证，必须先安装 Rust toolchain。

### 2. 执行环境边界

- 默认以 `WSL/Linux` 作为命令执行环境。
- 默认以 Linux 路径作为文档和命令示例基线。
- `Cursor` 可以作为主编辑器，但不能作为唯一执行路径；关键流程应能在纯 CLI 下完成。

### 3. 可选服务和环境变量

- PostgreSQL / ClickHouse / Bridge 都是按需启用，不是所有改动都必须启动。
- 只在你要验证对应功能时再配置相关环境变量。
- 当前常见入口包括：

```bash
export POSTGRES_URL="postgresql://localhost:5432/quantix"
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DB="quantix"
export QUANTIX_BRIDGE_BASE_URL="http://127.0.0.1:8080"
export QUANTIX_BRIDGE_API_KEY="your-api-key"
```

### 4. WSL + Cursor 并存约束

- WSL 负责 `cargo`、脚本、测试、路径一致性。
- Cursor 负责跨文件编辑、审阅和重构。
- 不依赖 Windows 专有路径作为唯一数据路径。
- 文档中的“必须”项应可在 WSL 独立完成。

## 🚀 5分钟上手

### 1. 环境准备

```bash
# 安装 Rust (https://rustup.rs/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust
```

### 2. 配置数据库

```bash
# PostgreSQL (可选)
export POSTGRES_URL="postgresql://localhost:5432/quantix"

# ClickHouse (推荐)
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DB="quantix"

# TDX 数据源 (可选)
export TDX_HOST="192.168.1.100"
export TDX_PORT=7709
```

### 3. 构建和测试

```bash
# 开发运行
cargo run -- --help

# 运行测试
cargo test --all-features

# 构建发布版本
cargo build --release
```

## 📚 重要文档

- [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md) - **必读！**
- [用户手册](docs/USER_MANUAL.md)
- [代码规范总结](archive/reports/CODE_RULE_SUMMARY.md)

## 🎯 常用命令

### 代码质量

```bash
# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy -- -D warnings

# 安全审计
cargo install cargo-audit
cargo audit
```

### CLI 使用

```bash
# 初始化配置
cargo run -- init

# 交互式菜单
cargo run -- menu

# 查询数据
cargo run -- data query --code 000001 --period 1d

# 运行回测
cargo run -- strategy run -n ma_cross --code 000001

# 运行 paper 策略前先初始化 paper 账户
cargo run -- trade init --capital 1000000
cargo run -- strategy run -n ma_cross --mode paper --code 000001

# 初始化策略信号守护进程配置
cargo run -- strategy config init

# 当主读取器为空或失败时，允许 daemon 回退到本地 TDX day 文件
export QUANTIX_TDX_ROOT=/mnt/d/ProgramData/tdx_20251231
export QUANTIX_TDX_MARKET=sz

# 跑一轮 signal daemon（不会自动交易）
cargo run -- strategy daemon run --once

# 查看待审批 signal
cargo run -- strategy signal list --approval-status pending

# 启动任务调度器
cargo run -- task start

# 计算技术指标
cargo run -- analyze indicators --code 000001 --indicators ma5,ma20,rsi14

# 健康检查
cargo run -- status --health
```

## ⚠️ 开发规范

**重要**: 所有代码必须符合 [开发规范指南](docs/standards/DEVELOPMENT_GUIDELINES.md)

### 核心规则

1. **错误处理**: 禁止 `unwrap()/expect()`，使用 `Result<T, QuantixError>`
2. **类型安全**: 价格/数量使用 `Decimal`，禁止 `f64/f32`
3. **性能**: 大数据集使用 Polars，I/O 操作异步
4. **测试**: 核心逻辑必须有单元测试

### 提交前检查

```bash
# 1. 格式化代码
cargo fmt

# 2. 代码检查
cargo clippy -- -D warnings

# 3. 运行测试
cargo test --all-features

# 4. 安全审计
cargo audit
```

## 🔄 CI/CD

项目使用 GitHub Actions 自动检查：
- ✅ 代码格式（rustfmt）
- ✅ 代码质量（clippy）
- ✅ 单元测试和集成测试
- ✅ 安全审计
- ✅ 多平台构建

## 📖 更多资源

- [完整文档](docs/)
- [API 文档](https://docs.rs/quantix-cli/)
- [问题反馈](https://github.com/chengjon/quantix-rust/issues)

---

**祝你使用愉快！** 🎉
