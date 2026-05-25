# Quantix-Rust 手工测试指引

> 状态源说明：本文是手工测试操作指引，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../FUNCTION_TREE.md) 的状态注册表行为准。

## 前置准备

### 1. 编译项目

```bash
# Release 模式（性能好，编译慢）
cargo build --release

# Debug 模式（编译快，推荐测试用）
cargo build
```

> `cargo build` 只写入 `target/` 目录，不影响项目源码结构。`target/` 已在 `.gitignore` 中。

下文统一用 `quantix` 代指 `./target/debug/quantix`。

### 1.1 命令验证约定

- 先执行一次 `cargo build`，再用 `./target/debug/quantix` 做命令行为验证。
- 不要把 `timeout 5 cargo run -- ...` 作为命令逻辑验证依据。`cargo run` 可能把时间耗在 cargo 锁等待、依赖编译或 crate 编译上。
- `cargo` 适合验证“是否可编译”；`./target/debug/quantix` 适合验证“命令是否按预期执行并产生副作用”。
- 运行 `scripts/verify_features.sh` 时，脚本已经遵循这个约定：先构建二进制，再执行 smoke checks。

### 1.2 Smoke 检查分层

- 纯本地 smoke：只依赖已构建的 `quantix` 二进制和本地文件状态，例如 help、strategy list、execution config show、risk status。
- 外部依赖 smoke：依赖桥接服务、数据库、上游 API 或网络环境，例如 `execution bridge status`、`fundamental valuation`。
- 记录结果时应区分“本地 CLI 回归”和“外部依赖不可用”，避免把环境故障误判成命令实现缺陷。

### 2. 初始化

```bash
quantix init
quantix status
quantix status --health
```

### 3. 配置 API Key

在项目根目录创建 `.env` 文件（已被 `.gitignore` 排除）：

```bash
# === AI 模型 ===
DEEPSEEK_API_KEY=your_deepseek_key
# OPENAI_API_KEY=your_openai_key
# ANTHROPIC_API_KEY=your_anthropic_key
# GOOGLE_API_KEY=your_google_key

# === 新闻搜索 ===
TAVILY_API_KEY=your_tavily_key
# SERPAPI_API_KEY=your_serpapi_key
# BOCHA_API_KEY=your_bocha_key

# === 数据库 ===
POSTGRES_URL=postgres://quantix:quantix@localhost:5432/quantix

# === 通知渠道（按需启用） ===
# TELEGRAM_BOT_TOKEN=xxx
# TELEGRAM_CHAT_ID=xxx
# WECHAT_WORK_WEBHOOK_URL=https://qyapi.weixin.qq.com/...
# FEISHU_WEBHOOK_URL=https://open.feishu.cn/...
# SLACK_WEBHOOK_URL=https://hooks.slack.com/...
# DINGTALK_WEBHOOK_URL=https://oapi.dingtalk.com/...
# PUSHPLUS_TOKEN=xxx
```

---

## API Key 与配置文件对照表

| 配置文件 | 内容 |
|---------|------|
| `config/default.toml` | 数据库连接、通知渠道 webhook URL |
| `config/ai.toml` | AI 模型 provider、base_url、模型名 |
| `config/news.toml` | 新闻搜索 provider 配置 |
| `.env`（环境变量） | 所有 API Key |

**原则**：Key 全部通过环境变量传入，不硬编码在配置文件里。

| 类别 | 环境变量 | 对应命令 | 必需程度 |
|------|---------|---------|---------|
| AI | `DEEPSEEK_API_KEY` | `ai analyze/decide/ask` | 推荐 |
| AI | `OPENAI_API_KEY` | `ai --model openai` | 可选 |
| AI | `ANTHROPIC_API_KEY` | `ai --model anthropic` | 可选 |
| AI | `GOOGLE_API_KEY` | `ai --model gemini` | 可选 |
| 新闻 | `TAVILY_API_KEY` | `news search/code/trend` | 推荐 |
| 新闻 | `SERPAPI_API_KEY` | `news --provider serpapi` | 可选 |
| 新闻 | `BOCHA_API_KEY` | `news --provider bocha` | 可选 |
| 图片导入 | `DEEPSEEK_API_KEY` 或 `OPENAI_API_KEY` | `import from-image` | 可选 |
| 数据库 | `POSTGRES_URL` | `watchlist`、`risk` 等持久化功能 | 必须 |
| 通知 | `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` | `notify --channel telegram` | 可选 |
| 通知 | `WECHAT_WORK_WEBHOOK_URL` | `notify --channel wechat_work` | 可选 |
| 通知 | `FEISHU_WEBHOOK_URL` | `notify --channel feishu` | 可选 |
| 通知 | `SLACK_WEBHOOK_URL` | `notify --channel slack` | 可选 |
| 通知 | `DINGTALK_WEBHOOK_URL` | `notify --channel dingtalk` | 可选 |
| 通知 | `PUSHPLUS_TOKEN` | `notify --channel pushplus` | 可选 |
| 执行桥接 | `BRIDGE_BASE_URL` / `BRIDGE_API_KEY` | `execution bridge` | 可选 |

---

## 完整命令清单

### 1. 自选池 watchlist

```bash
quantix watchlist add --code 600519 --group 蓝筹
quantix watchlist add --code 000001
quantix watchlist list
quantix watchlist list --group 蓝筹
quantix watchlist move --code 600519 --group 核心
quantix watchlist tag add --code 600519 --tag 白酒
quantix watchlist tag remove --code 600519 --tag 白酒
quantix watchlist group create --name 科技
quantix watchlist group list
quantix watchlist history --code 600519
quantix watchlist remove --code 600519
```

### 2. 模拟交易 trade

```bash
quantix trade init --capital 1000000
quantix trade buy --code 600519 --quantity 100 --price 1800.00
quantix trade sell --code 600519 --quantity 100 --price 1850.00
quantix trade overview
quantix trade position
quantix trade cash
quantix trade fees --code 600519
quantix trade history
quantix trade reset --capital 500000
```

### 3. 市场 market

```bash
quantix market overview
quantix market sector
quantix market concept
quantix market north
quantix market sentiment
quantix market leader
```

### 4. 数据 data

```bash
quantix data query --code 600519 --start 2025-01-01 --end 2025-03-01
quantix data export --code 600519 --format csv --output data.csv
```

### 5. 分析 analyze

```bash
quantix analyze indicators --code 600519 --indicators ma,macd,rsi
quantix analyze candle-pattern --code 600519
quantix analyze backtest --id <backtest_id>

# 选股筛选
quantix analyze screener preset-list
quantix analyze screener run --codes 600519,000001 --preset volume_surge
quantix analyze screener run --watchlist --group 蓝筹
```

### 6. 策略 strategy

```bash
quantix strategy list
quantix strategy show --name <策略名>
quantix strategy run --name <策略名> --code 600519
quantix strategy signal list
quantix strategy signal list --code 600519
quantix strategy request list
quantix strategy config init
quantix strategy config show
quantix strategy daemon run --once
```

### 7. 监控 monitor

```bash
quantix monitor watchlist
quantix monitor alert add 600519 --above 1900
quantix monitor alert add 000001 --below 15.5
quantix monitor config show
quantix monitor config set --interval-seconds 60
quantix monitor daemon run
quantix monitor event list --limit 10
quantix monitor event list --code 600519
```

### 8. 止盈止损 stop

```bash
quantix stop set --code 600519 --profit 10 --stop-loss 5
quantix stop list
quantix stop status --code 600519
quantix stop update --code 600519 --profit 15
quantix stop history
quantix stop remove --code 600519
```

### 9. 风险管理 risk

```bash
quantix risk status
quantix risk pnl
quantix risk position
quantix risk rule list
quantix risk rule set --type max_position --value 0.3
quantix risk log
quantix risk lock release
quantix risk import live-trades --account <id> --input trades.csv
quantix risk rebuild live-account --account <id>
```

### 10. 算法交易 algo (TWAP/VWAP)

```bash
quantix algo plan --code 600519 --side buy --quantity 1000 --algo-type twap --duration 30
quantix algo create --code 600519 --side buy --quantity 1000 --algo-type twap --duration 30
quantix algo list
quantix algo status --id <task_id>
quantix algo start --id <task_id>
quantix algo pause --id <task_id>
quantix algo resume --id <task_id>
quantix algo cancel --id <task_id>
```

### 11. 异常检测 anomaly

```bash
quantix anomaly run
quantix anomaly run --top-n 10 --period 5 --min-volume 5000
quantix anomaly run --format json
```

### 12. 账户管理 account

```bash
quantix account register --id paper --name 模拟账户 --capital 1000000
quantix account list
quantix account show --id paper
quantix account update --id paper --name 主账户
quantix account default --id paper
quantix account remove --id paper
quantix account group create --id g1 --name 组A --strategy equal
quantix account group list
quantix account summary
quantix account split --id paper --amount 100000 --parts 3
```

### 13. 执行自动化 execution

```bash
quantix execution config init
quantix execution config show
quantix execution daemon run
quantix execution bridge --help
```

### 14. 通知 notify

```bash
quantix notify test
quantix notify test --channel dingtalk
quantix notify send --title "测试" --message "hello"
quantix notify list
quantix notify check --channel feishu
```

### 15. AI 决策 ai

```bash
quantix ai analyze --code 600519
quantix ai analyze --code 600519 --model deepseek --with-news
quantix ai decide --code 600519
quantix ai ask --question "茅台最近走势如何"
quantix ai market
quantix ai config show
```

### 16. 新闻 news

```bash
quantix news search --query "茅台"
quantix news search --query "新能源" --days 7 --max 10
quantix news code --code 600519
quantix news trend --query "AI"
quantix news providers
```

### 17. 基本面 fundamental

```bash
quantix fundamental show --code 600519
quantix fundamental valuation --code 600519
quantix fundamental earnings --code 600519
quantix fundamental institution --code 600519
quantix fundamental dragon-tiger --code 600519
quantix fundamental dividend --code 600519
```

### 18. 舆情 sentiment

```bash
quantix sentiment show --code 600519
quantix sentiment history --code 600519
quantix sentiment mentions --code 600519
```

### 19. 智能导入 import

```bash
quantix import resolve "贵州茅台"
quantix import from-text "茅台 平安银行 宁德时代"
quantix import from-csv --input stocks.csv
quantix import from-clipboard
quantix import from-image --image screenshot.png
```

### 20. 任务调度 task（实验性）

```bash
quantix task list
quantix task start
quantix task stop
quantix task status
```

### 21. 系统状态 status

```bash
quantix status
quantix status --health
```

---

## 推荐测试顺序

按依赖从少到多排列，逐步验证：

| 步骤 | 命令 | 验证点 | 依赖 |
|------|------|--------|------|
| 1 | `quantix init` | 数据库创建成功 | PostgreSQL |
| 2 | `quantix status --health` | DB 连接正常 | PostgreSQL |
| 3 | `quantix watchlist add --code 600519` | 写入成功 | PostgreSQL |
| 4 | `quantix watchlist list` | 能读回数据 | PostgreSQL |
| 5 | `quantix market overview` | 网络请求正常（东方财富公开API） | 网络 |
| 6 | `quantix trade init --capital 1000000` | 模拟账户创建 | PostgreSQL |
| 7 | `quantix trade buy --code 600519 -n 100 --price 1800` | 交易执行 | PostgreSQL |
| 8 | `quantix trade position` | 持仓显示正确 | PostgreSQL |
| 9 | `quantix analyze indicators --code 600519 --indicators ma,rsi` | 指标计算 | 数据源 |
| 10 | `quantix stop set --code 600519 --profit 10 --stop-loss 5` | 止盈止损写入 | PostgreSQL |
| 11 | `quantix risk status` | 风控状态 | PostgreSQL |
| 12 | `quantix algo plan --code 600519 --side buy -n 1000 --algo-type twap --duration 30` | 算法预览（不执行） | 无 |
| 13 | `quantix anomaly run --top-n 5` | 异常检测运行 | 网络 |
| 14 | `quantix news search --query "茅台"` | 新闻搜索 | TAVILY_API_KEY |
| 15 | `quantix ai analyze --code 600519` | AI 分析 | DEEPSEEK_API_KEY |
| 16 | `quantix fundamental show --code 600519` | 基本面数据 | 网络 |
| 17 | `quantix notify test` | 通知渠道连通 | 对应通知渠道 API Key |
| 18 | `quantix import resolve "茅台"` | 名称解析 | PostgreSQL |

### 无外部依赖的纯本地命令

以下命令不需要数据库或网络，可以随时测试：

- `quantix algo plan` — 算法切片预览
- `quantix import from-text "茅台 平安银行"` — 文本解析（但 resolve 需要数据库）
- `quantix status` — 系统状态展示

### 需要网络但无需 API Key 的命令

这些使用东方财富公开接口：

- `quantix market overview/sector/concept/north/sentiment/leader`
- `quantix anomaly run`
