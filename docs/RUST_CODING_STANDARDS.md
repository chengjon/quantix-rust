# Rust 项目编码规则（通用版）

> 版本: 1.0 | 适用: Rust 2021/2024 Edition | 可直接迁移到其他 Rust 项目

---

## 一、文件行数限制（强制阈值）

| 文件类型 | 考虑拆分 | 强制拆分 | 备注 |
|----------|---------|---------|------|
| `.rs` 模块文件 | > 500 行 | > 800 行 | 含 `mod.rs` |
| `.rs` 单一 struct/enum 实现 | > 300 行 | > 500 行 | 应拆分 trait impl 到独立文件 |
| `handlers.rs` (CLI/路由) | > 800 行 | > 1200 行 | 按功能拆为 `handlers/xxx.rs` |
| `lib.rs` / `main.rs` | > 100 行 | > 150 行 | 仅保留 mod 声明和 re-export |
| 测试文件 `*_test.rs` | > 500 行 | > 1000 行 | 按测试模块拆分 |
| `Cargo.toml` | - | - | 依赖超过 40 个须审查必要性 |

### 拆分策略

```
# 拆分前
src/market/
├── mod.rs          (800+ 行，混合所有逻辑)

# 拆分后
src/market/
├── mod.rs          (仅 mod 声明 + pub use)
├── models.rs       (数据结构定义)
├── service.rs      (业务逻辑)
├── sentiment/
│   ├── mod.rs
│   ├── types.rs
│   ├── provider.rs  (trait 定义)
│   └── aggregator.rs
```

### 拆分原则

1. **按职责拆**: types / traits / impl / service 各归其位
2. **mod.rs 纯净**: `mod.rs` 仅允许 `pub mod` 声明和 `pub use` 重导出，禁止业务逻辑
3. **单文件单一关注点**: 一个文件只做一个事情（定义类型 OR 实现 trait OR 业务逻辑）
4. **handlers/ 目录模式**: 当 handler 超过阈值，从 `handlers.rs` 拆为 `handlers/mod.rs` + `handlers/xxx.rs`

---

## 二、模块组织规范

### 2.1 模块声明顺序（mod.rs 内）

```rust
// 1. 子模块声明（按字母序）
pub mod aggregator;
pub mod cache;
pub mod provider;
pub mod types;

// 2. 类型重导出（按使用频率排序）
pub use types::{NewsArticle, NewsSearchRequest};
pub use provider::NewsProvider;
pub use aggregator::NewsAggregator;
```

### 2.2 lib.rs / main.rs 规范

- **禁止**在 `lib.rs` 中写业务逻辑
- **禁止**超过 150 行
- 仅允许：`pub mod` 声明、`pub use` 重导出、模块级文档注释

```rust
// ✅ 正确的 lib.rs
pub mod ai;
pub mod cli;
pub mod core;
pub mod market;

pub use cli::Cli;
pub use core::{QuantixError, Result};
```

### 2.3 禁止循环依赖

```
允许: core <- market <- cli
禁止: market <-> cli (双向)
禁止: A -> B -> C -> A (环路)
```

单向依赖层级:

```
CLI 层 (最上层，处理用户交互)
  ↓
Service 层 (业务逻辑编排)
  ↓
Provider/Adapter 层 (外部 API / 数据源适配)
  ↓
Domain 层 (纯数据结构 + trait 定义)
  ↓
Core 层 (错误类型、配置、工具) — 无外部依赖
```

---

## 三、类型与 Trait 规范

### 3.1 类型定义

```rust
// ✅ 公共类型必须 #[derive] 必要 trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    pub code: Option<String>,
    pub name: Option<String>,
    pub confidence: f64,
}

// ✅ 枚举使用 serde rename_all
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchMethod {
    ExactCode,
    ExactName,
    Pinyin,
    Fuzzy,
}
```

### 3.2 Trait 定义

```rust
// ✅ Trait 放独立文件 (provider.rs / adapter.rs)
// ✅ 使用 async_trait 标注异步方法
#[async_trait]
pub trait NewsProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn search(&self, query: &str) -> Result<Vec<NewsArticle>>;
    fn is_available(&self) -> bool { true }  // 提供默认实现
}
```

### 3.3 类型命名规范

| 项 | 规范 | 示例 |
|----|------|------|
| Struct | UpperCamelCase | `MarketService`, `ImportItem` |
| Enum | UpperCamelCase | `MatchMethod`, `ImportSource` |
| Enum variant | UpperCamelCase | `ExactCode`, `FromImage` |
| Trait | UpperCamelCase + 名词/能力 | `NewsProvider`, `LlmAdapter` |
| 函数/方法 | snake_case | `get_sentiment`, `parse_file` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_RETRIES`, `DEFAULT_TIMEOUT` |
| 模块 | snake_case | `market`, `fundamental` |
| 文件名 | snake_case | `code_resolver.rs` |

### 3.4 显式类型标注

```rust
// ✅ 公共 API 必须显式标注返回类型
pub fn resolve(&self, input: &str) -> Option<CodeResolveResult> { ... }

// ✅ 公共 struct 字段必须显式类型
pub struct LlmConfig {
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub max_tokens: u32,
}

// ⚠️ 内部实现可省略，但建议标注
let items: Vec<ImportItem> = Vec::new();  // 推荐
let items = Vec::new();                    // 可接受（类型可推导时）
```

---

## 四、错误处理规范

### 4.1 统一错误类型

```rust
// core/error.rs
pub type Result<T> = std::result::Result<T, QuantixError>;

#[derive(Debug, thiserror::Error)]
pub enum QuantixError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),

    #[error("{0}")]
    Other(String),
}
```

### 4.2 错误处理红线

| 规则 | 要求 |
|------|------|
| **禁止 unwrap()** | 生产代码中禁止 `.unwrap()`，使用 `?` 或 `map_err` |
| **禁止 panic!()** | 业务逻辑禁止 panic，使用 `Result` 传播 |
| **错误上下文** | `map_err` 必须附加描述性信息 |
| **禁止吞错误** | 禁止 `let _ = may_fail()`，必须至少 log |
| **CLI 层兜底** | CLI handler 最外层打印用户友好信息 |

```rust
// ❌ 禁止
let config = std::fs::read_to_string(path).unwrap();
let _ = sender.send(msg);

// ✅ 正确
let config = std::fs::read_to_string(path)
    .map_err(|e| QuantixError::Other(format!("读取配置失败 {}: {}", path, e)))?;
if let Err(e) = sender.send(msg) {
    tracing::warn!("发送消息失败: {}", e);
}
```

---

## 五、日志与输出规范

### 5.1 日志 vs println

| 场景 | 使用 |
|------|------|
| CLI 面向用户的输出 | `println!` / `eprintln!` |
| 库/服务内部日志 | `tracing::info!` / `warn!` / `error!` |
| 调试信息 | `tracing::debug!` |
| **禁止** | 库模块中使用 `println!` |

```rust
// ✅ CLI handler
println!("✅ 解析完成: {} 只股票", items.len());

// ✅ 库/服务代码
tracing::info!(code = %code, "开始解析股票数据");
tracing::warn!(error = %e, "API 请求失败，使用缓存");
```

### 5.2 错误信息语言

- 面向用户的消息：中文
- 日志/调试信息：中文或英文均可
- commit message：英文

---

## 六、异步编程规范

### 6.1 async 函数

```rust
// ✅ async trait 使用 async_trait
#[async_trait]
pub trait DataProvider: Send + Sync {
    async fn fetch(&self, code: &str) -> Result<Data>;
}

// ✅ 普通 async 函数直接写
pub async fn get_sentiment(&self, code: &str) -> Result<SentimentData> { ... }
```

### 6.2 并发规范

| 规则 | 要求 |
|------|------|
| 独立请求 | 使用 `tokio::join!` 或 `futures::join!` 并发 |
| 共享状态 | 使用 `Arc<RwLock<T>>` 或 `Arc<Mutex<T>>` |
| 禁止阻塞 | async 函数内禁止阻塞操作（文件 IO 用 `tokio::fs`） |
| 超时保护 | 外部 API 调用必须设置超时 |

```rust
// ✅ 并发请求
let (news, fundamental) = tokio::join!(
    news_provider.search(code),
    fundamental_provider.get_data(code),
);
```

---

## 七、CLI 命令规范 (clap)

### 7.1 命令定义模式

```rust
#[derive(Subcommand, Debug)]
pub enum MarketCommands {
    /// 行业板块排名 (中文帮助文本)
    Sector {
        #[arg(long)]
        top: Option<usize>,

        #[arg(long)]
        date: Option<String>,
    },
}
```

### 7.2 Handler 模式

```rust
// 主分发函数
pub async fn run_market_command(cmd: MarketCommands) -> Result<()> {
    match cmd {
        MarketCommands::Sector { top, date } => run_market_sector(top, date).await,
        MarketCommands::North { date } => run_market_north(date).await,
    }
}

// 子函数命名: run_{模块}_{子命令}
async fn run_market_sector(top: Option<usize>, date: Option<String>) -> Result<()> {
    // 实现...
    Ok(())
}
```

### 7.3 CLI 输出格式

```rust
// ✅ 标准表格输出
println!("✅ 解析到 {} 只股票:", items.len());
println!();
println!("{:<10} {:<12} {}", "代码", "名称", "置信度");
println!("{}", "-".repeat(35));
for item in &items {
    println!("{:<10} {:<12} {:.0}%", item.code, item.name, item.confidence * 100.0);
}
```

---

## 八、测试规范

### 8.1 测试组织

```rust
// 单元测试: 放同一文件底部
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_code() {
        let resolver = CodeResolver::new();
        let result = resolver.resolve("000001").unwrap();
        assert_eq!(result.code, "000001");
    }
}

// 集成测试: 放 tests/ 目录
// tests/market_test.rs
```

### 8.2 测试要求

| 规则 | 要求 |
|------|------|
| 公共函数覆盖 | 每个 pub 函数至少 1 个正向测试 |
| 错误路径 | 关键错误分支必须有测试 |
| 命名 | `test_{功能}_{场景}` |
| 禁止外部依赖 | 单元测试不依赖网络/数据库 |

---

## 九、配置管理规范

### 9.1 配置来源优先级

```
环境变量 > TOML 配置文件 > 代码内默认值
```

### 9.2 配置红线

| 规则 | 要求 |
|------|------|
| **禁止硬编码** | URL / Token / 密钥 不许写在代码中 |
| **环境变量** | 敏感配置必须通过环境变量注入 |
| **.env.example** | 项目根目录必须维护 `.env.example` |
| **TOML 分模块** | `config/ai.toml`, `config/news.toml` 按功能拆分 |

---

## 十、依赖管理规范

### 10.1 依赖审查

| 规则 | 要求 |
|------|------|
| 新增依赖 | 必须说明理由（为什么不用已有依赖） |
| feature 精简 | 只启用需要的 feature |
| 版本锁定 | 使用 `cargo lock`，CI 中禁止自动升级 |
| 禁止重复 | 同类功能只用一个 crate（HTTP 只用 reqwest 等） |

### 10.2 常用依赖选型

| 功能 | 推荐 crate |
|------|-----------|
| HTTP 客户端 | `reqwest` |
| 序列化 | `serde` + `serde_json` |
| CLI | `clap` (derive 模式) |
| 异步运行时 | `tokio` |
| 数据库 | `sqlx` |
| 错误处理 | `thiserror` |
| 日志 | `tracing` + `tracing-subscriber` |
| 时间 | `chrono` |
| 数值 | `rust_decimal` (金融场景) |
| 测试 | 内置 `#[test]` + `tokio::test` |

---

## 十一、Commit 规范

### 11.1 微提交原则

| 规则 | 要求 |
|------|------|
| 单一意图 | 1 个 commit = 1 个明确语义目标 |
| 禁止混入 | 不要把"顺手改"混进主任务 |
| 分类提交 | feat / fix / refactor / test / docs / chore 分开 |
| 删除判定 | 必须"代码引用 + 功能树"双判定，未引用 ≠ 可删除 |

### 11.2 Commit Message 格式

```
<type>(<scope>): <subject>

<body>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

type: `feat` | `fix` | `refactor` | `test` | `docs` | `chore` | `perf`

---

## 十二、质量门禁

```bash
# 必须全部通过才能合并
cargo fmt --check          # 格式化检查
cargo clippy -- -D warnings # Lint 检查（warning 视为错误）
cargo test                  # 全部测试通过
cargo build --release       # Release 构建无错误
```

### Cargo.toml 配置建议

```toml
[lints.clippy]
# 强制检查项
unwrap_used = "warn"
expect_used = "warn"
print_stdout = "warn"        # 库代码禁止 println
print_stderr = "warn"
todo = "warn"
```

---

## 十三、方案先行准则

### 适用场景

- 涉及模块架构、目录结构的变更
- 新增外部依赖
- 修改公共 Trait / Error 类型
- 跨 3 个以上文件的变更

### 流程

1. 先描述方案（文字 + 文件清单）
2. 等待明确批准
3. 批准后方可执行
4. 未经审批的变更视为无效交付

---

## 十四、Rust 特有红线

| 规则 | 要求 |
|------|------|
| **禁止 `clone()` 滥用** | 优先使用引用 `&T`，仅在必要时 clone |
| **生命周期标注** | 公共 API 尽量避免复杂生命周期，用 `String` 替代 `&str` |
| **Unsafe** | 禁止使用 `unsafe`，除非有安全评审 |
| **trait object** | 优先静态分发（泛型），动态分发（`dyn`）仅在需要异构集合时 |
| **feature flag** | 可选功能必须用 feature gate 保护 |
| **文档注释** | 公共 mod / struct / trait / pub fn 必须有 `///` 文档注释 |
| **禁止 TODO 遗留** | `TODO` 必须带 issue 编号或人名 + 日期 |

---

## 附录: 规则速查卡

```
文件超 800 行？ → 拆分
lib.rs 超 150 行？ → 精简
用 unwrap()? → 改用 ? 或 map_err
库代码 println? → 改用 tracing
硬编码配置? → 提取到 .env / .toml
新增外部 API? → 先定义 Trait, 再实现
handler 超 1200 行? → 拆为 handlers/ 目录
commit 混了多个意图? → 拆分提交
循环依赖? → 重构模块边界
```
