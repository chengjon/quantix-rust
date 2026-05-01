# Quantix-Rust 手工测试指引

## 2026-04-18 最新测试顺序

本节基于当前 CLI 实现补充，优先级高于本文后续较早期的命令清单。

### 测试目标

- 优先验证当前架构对齐主线新增的命令面是否可正常返回
- 优先覆盖纯本地、低外部依赖的命令
- 把需要外部环境的命令单独归类，避免误判为 CLI 实现故障

### 建议记录格式

每条命令至少记录以下 4 项：

- 命令
- 返回码
- 标准输出摘要
- 是否符合预期

建议额外记录：

- 是否依赖外部环境
- 是否产生文件或状态副作用

### 推荐执行顺序

#### 0. 构建与总览

```bash
cargo build
./target/debug/quantix --help
./target/debug/quantix strategy --help
./target/debug/quantix data source --help
./target/debug/quantix backtest --help
./target/debug/quantix performance --help
./target/debug/quantix execution qmt --help
```

判定重点：

- help 能正常展开
- 新命令组 `data source`、`backtest`、`performance`、`execution qmt` 可见
- `strategy` 下可见 `create/update/delete`

#### 1. data source 命令组

```bash
./target/debug/quantix data source list
./target/debug/quantix data source add --type akshare
./target/debug/quantix data source list
./target/debug/quantix data source set-default --name akshare
./target/debug/quantix data source test --name akshare
```

判定重点：

- `list` 能列出当前配置或默认配置
- `add` 能正常写入或更新配置
- `set-default` 后再次 `list` 能看到默认源变化
- `test` 若失败，应能明确区分为网络/API 不可用，而不是 CLI 参数错误

备注：

- 若本地无 AkShare 环境或网络受限，`test` 可记为“外部依赖失败”

#### 2. backtest 命令组

```bash
./target/debug/quantix backtest run --code 600519 --start 20240101 --end 20241231 --short 5 --long 20
./target/debug/quantix backtest list
./target/debug/quantix backtest report --id <REPORT_ID>
./target/debug/quantix backtest compare --id <REPORT_ID_A> --id <REPORT_ID_B>
```

判定重点：

- `run` 能生成报告 ID 或明确报出数据缺失原因
- `list` 能读取到新生成的报告
- `report` 能读取单份报告摘要
- `compare` 能输出多报告对比表

备注：

- 若 `run` 失败，先确认是否是数据源或历史数据不可用，再判断是否为 CLI 问题
- 如果当前环境只能生成一份报告，可先重复跑两次不同参数，再做 `compare`

#### 3. performance 命令组

```bash
./target/debug/quantix performance list
./target/debug/quantix performance report --id <REPORT_ID>
./target/debug/quantix performance compare --id <REPORT_ID_A> --id <REPORT_ID_B>
```

判定重点：

- `performance` 命令面与 `backtest` 的已保存报告能够联通
- `report` 与 `compare` 能正常读取而不是重复要求重新回测

#### 4. strategy 实例管理

```bash
./target/debug/quantix strategy list
./target/debug/quantix strategy create --id ma-demo --name ma_cross --code 600519 --param fast=5 --param slow=20
./target/debug/quantix strategy list
./target/debug/quantix strategy show --id ma-demo
./target/debug/quantix strategy update --id ma-demo --param fast=8 --param slow=21 --enable
./target/debug/quantix strategy show --id ma-demo
./target/debug/quantix strategy delete --id ma-demo
./target/debug/quantix strategy list
```

判定重点：

- `list` 现在应同时显示内置策略目录和已配置实例
- `create` 后 `show --id` 能看到实例详情
- `update` 后参数和启用状态变化可见
- `delete` 后实例从 `list` 中消失

补充建议：

- 再补一组 `--disabled` 创建用例
- 再补一组 `show --name ma_cross`，确认内置策略展示仍正常

#### 5. execution qmt 兼容入口

推荐先测新入口，再测旧入口兼容性。

```bash
./target/debug/quantix execution qmt status
./target/debug/quantix execution qmt preview --request-id <REQUEST_ID>
./target/debug/quantix execution qmt query --order-id <ORDER_ID>
./target/debug/quantix execution bridge status
./target/debug/quantix execution bridge qmt-preview --request-id <REQUEST_ID>
./target/debug/quantix execution bridge qmt-query --order-id <ORDER_ID>
```

判定重点：

- `execution qmt` 新入口可以正常解析并返回
- `execution bridge` 旧入口仍兼容
- 若桥接服务未启动，应返回明确的连接失败/外部依赖错误

高风险命令：

```bash
./target/debug/quantix execution qmt live --request-id <REQUEST_ID>
./target/debug/quantix execution bridge qmt-live --request-id <REQUEST_ID>
```

执行要求：

- 仅在确认是测试账户、且 bridge 已满足 `qmt.enabled=true`、`qmt.mode=live`、`qmt.supports` 包含 `order_submit` 时再测
- 默认应出现 `YES` 二次确认提示
- 不建议在未知券商环境直接执行

#### 6. 向后兼容路径

```bash
./target/debug/quantix analyze screener presets
./target/debug/quantix analyze screener preset-list
./target/debug/quantix analyze backtest --id <REPORT_ID>
```

判定重点：

- `presets` 新别名可用
- `preset-list` 旧路径未断
- `analyze backtest` 仍能委托到新的回测报告读取逻辑

### 建议的最终人工测试结论模板

```text
测试日期：
测试二进制：
测试范围：data source / backtest / performance / strategy instance / execution qmt

通过：
- 列出所有通过的命令

外部依赖失败：
- 列出因网络、桥接、上游 API 导致失败的命令

CLI 缺陷：
- 列出参数解析、输出异常、状态未落盘、兼容路径失效等问题
```

### 现阶段优先级结论

建议你实际手工执行时按以下优先级推进：

1. `data source`
2. `backtest`
3. `performance`
4. `strategy create/update/delete`
5. `execution qmt`

原因：

- 这 5 组正对应当前 CLI 对齐主线
- 它们最能暴露“入口是否统一、状态是否落盘、兼容层是否生效”
- 插件化、多 crate、Wasm 当前都不是手工测试优先级

### 可勾选执行清单

你可以直接复制这一段，边测边勾选：

```text
测试日期：2026-04-18
测试二进制：./target/debug/quantix

[ ] 0. 构建与总览
[ ] cargo build
[ ] ./target/debug/quantix --help
[ ] ./target/debug/quantix strategy --help
[ ] ./target/debug/quantix data source --help
[ ] ./target/debug/quantix backtest --help
[ ] ./target/debug/quantix performance --help
[ ] ./target/debug/quantix execution qmt --help

[ ] 1. data source
[ ] ./target/debug/quantix data source list
[ ] ./target/debug/quantix data source add --type akshare
[ ] ./target/debug/quantix data source list
[ ] ./target/debug/quantix data source set-default --name akshare
[ ] ./target/debug/quantix data source test --name akshare

[ ] 2. backtest
[ ] ./target/debug/quantix backtest run --code 600519 --start 20240101 --end 20241231 --short 5 --long 20
[ ] ./target/debug/quantix backtest list
[ ] ./target/debug/quantix backtest report --id <REPORT_ID>
[ ] ./target/debug/quantix backtest compare --id <REPORT_ID_A> --id <REPORT_ID_B>

[ ] 3. performance
[ ] ./target/debug/quantix performance list
[ ] ./target/debug/quantix performance report --id <REPORT_ID>
[ ] ./target/debug/quantix performance compare --id <REPORT_ID_A> --id <REPORT_ID_B>

[ ] 4. strategy 实例管理
[ ] ./target/debug/quantix strategy list
[ ] ./target/debug/quantix strategy create --id ma-demo --name ma_cross --code 600519 --param fast=5 --param slow=20
[ ] ./target/debug/quantix strategy list
[ ] ./target/debug/quantix strategy show --id ma-demo
[ ] ./target/debug/quantix strategy update --id ma-demo --param fast=8 --param slow=21 --enable
[ ] ./target/debug/quantix strategy show --id ma-demo
[ ] ./target/debug/quantix strategy delete --id ma-demo
[ ] ./target/debug/quantix strategy list
[ ] ./target/debug/quantix strategy create --id ma-demo-disabled --name ma_cross --code 600519 --param fast=5 --param slow=20 --disabled
[ ] ./target/debug/quantix strategy show --id ma-demo-disabled
[ ] ./target/debug/quantix strategy delete --id ma-demo-disabled
[ ] ./target/debug/quantix strategy show --name ma_cross

[ ] 5. execution qmt 兼容入口
[ ] ./target/debug/quantix execution qmt status
[ ] ./target/debug/quantix execution qmt preview --request-id <REQUEST_ID>
[ ] ./target/debug/quantix execution qmt query --order-id <ORDER_ID>
[ ] ./target/debug/quantix execution bridge status
[ ] ./target/debug/quantix execution bridge qmt-preview --request-id <REQUEST_ID>
[ ] ./target/debug/quantix execution bridge qmt-query --order-id <ORDER_ID>

[ ] 6. 高风险实盘确认路径
[ ] ./target/debug/quantix execution qmt live --request-id <REQUEST_ID>
[ ] ./target/debug/quantix execution bridge qmt-live --request-id <REQUEST_ID>

[ ] 7. 向后兼容路径
[ ] ./target/debug/quantix analyze screener presets
[ ] ./target/debug/quantix analyze screener preset-list
[ ] ./target/debug/quantix analyze backtest --id <REPORT_ID>

外部依赖失败：

CLI 缺陷：

备注：
```

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
quantix market foundation
quantix market overview
quantix market sector
quantix market concept
quantix market north
quantix market sentiment
quantix market leader
quantix market strength
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
| 5 | `quantix risk sync industry --standard shenwan` | 行业 SQLite 引用表生成成功 | MySQL |
| 6 | `quantix market foundation` | A 股总数与行业覆盖摘要可返回 | 网络 + 本地行业 SQLite |
| 7 | `quantix market overview` | 网络请求正常（东方财富公开API） | 网络 |
| 8 | `quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10` | 强弱板块与强势板块个股 TopN 返回 | 网络 + 本地行业 SQLite |
| 8.1 | `quantix market strength-stocks --date 2026-03-09 --strong-top 3 --sector 银行 --metric profit --top 10` | 单个强势行业内个股利润排行返回，行业过滤生效 | 网络 + 本地行业 SQLite |
| 9 | `quantix trade init --capital 1000000` | 模拟账户创建 | PostgreSQL |
| 10 | `quantix trade buy --code 600519 -n 100 --price 1800` | 交易执行 | PostgreSQL |
| 11 | `quantix trade position` | 持仓显示正确 | PostgreSQL |
| 12 | `quantix analyze indicators --code 600519 --indicators ma,rsi` | 指标计算 | 数据源 |
| 13 | `quantix stop set --code 600519 --profit 10 --stop-loss 5` | 止盈止损写入 | PostgreSQL |
| 14 | `quantix risk status` | 风控状态 | PostgreSQL |
| 15 | `quantix algo plan --code 600519 --side buy -n 1000 --algo-type twap --duration 30` | 算法预览（不执行） | 无 |
| 16 | `quantix anomaly run --top-n 5` | 异常检测运行 | 网络 |
| 17 | `quantix news search --query "茅台"` | 新闻搜索 | TAVILY_API_KEY |
| 18 | `quantix ai analyze --code 600519` | AI 分析 | DEEPSEEK_API_KEY |
| 19 | `quantix fundamental show --code 600519` | 基本面数据 | 网络 |
| 20 | `quantix notify test` | 通知渠道连通 | 对应通知渠道 API Key |
| 21 | `quantix import resolve "茅台"` | 名称解析 | PostgreSQL |

### 无外部依赖的纯本地命令

以下命令不需要数据库或网络，可以随时测试：

- `quantix algo plan` — 算法切片预览
- `quantix import from-text "茅台 平安银行"` — 文本解析（但 resolve 需要数据库）
- `quantix status` — 系统状态展示

### 需要网络但无需 API Key 的命令

这些使用东方财富公开接口：

- `quantix market foundation/overview/sector/concept/north/sentiment/leader/strength`
- `quantix anomaly run`

### 市场分析专项 smoke

若你要对这次市场分析交付做一轮独立验收，优先执行：

```bash
source scripts/dev/market_cli_env.example.sh
scripts/dev/init_market_cli_local_env.sh
scripts/dev/doctor_market_cli_env.sh
scripts/dev/run_market_cli_acceptance.sh
scripts/dev/run_market_cli_formal_sequence.sh
scripts/dev/generate_market_cli_acceptance_report.sh
scripts/dev/check_market_cli_prereqs.sh
scripts/dev/verify_market_cli_smoke.sh
```

判定原则：

- `check_market_cli_prereqs.sh` 用于提前暴露环境缺口，不直接判定业务实现是否正确
- `scripts/dev/market_cli_env.example.sh` 提供可直接修改的环境变量模板
- `.env.market.local.example` 提供本机未纳管的持久化模板；复制为 `.env.market.local` 后，这几条 market 脚本会自动加载
- `scripts/dev/init_market_cli_local_env.sh` 会在缺失时从 example 复制 `.env.market.local`，并阻止带着 `replace-me` 占位值继续执行
- `scripts/dev/doctor_market_cli_env.sh` 会显示 `.env`、`.env.market.local` 和运行时最终值之间的覆盖关系
- `scripts/dev/run_market_cli_acceptance.sh` 是单入口编排脚本，会串起 template 提示、precheck 和 smoke
- `scripts/dev/run_market_cli_acceptance.sh` / `scripts/dev/run_market_cli_formal_sequence.sh` 现在都会先调用 `init_market_cli_local_env.sh`
- `scripts/dev/run_market_cli_formal_sequence.sh` 会顺序执行正式 `risk sync / foundation / strength`，并把每一步的 exit code 和日志路径稳定记录下来
- `scripts/dev/generate_market_cli_acceptance_report.sh` 会从最近一次 acceptance / precheck / smoke 日志生成 Markdown 报告草稿
- 若脚本提示缺少 `industry_reference.db`、上游 MySQL 环境变量、或 ClickHouse 配置，应先补环境再跑正式命令
- ClickHouse 运行时变量使用 `CLICKHOUSE_URL` / `CLICKHOUSE_DB`，不是 `QUANTIX_CLICKHOUSE_*`
- 若出现 warning，优先按脚本的 `[REMEDIATION]` 段落执行补救动作，再重跑 precheck
- 本地 help / build 检查必须通过
- `risk sync industry`、`market foundation`、`market strength` 属于外部依赖检查
- 若失败信息明确指向 MySQL、网络、SQLite 行业引用表未同步、超时等外部条件，可记为 expected-warn
- 若参数解析、命令路径、输出结构本身异常，应记为 CLI 缺陷
