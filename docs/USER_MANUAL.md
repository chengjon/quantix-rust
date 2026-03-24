# Quantix-Rust 用户手册

## 目录

- [项目简介](#项目简介)
- [安装与配置](#安装与配置)
- [快速开始](#快速开始)
- [命令参考](#命令参考)
  - [init - 初始化](#init---初始化)
  - [menu - 交互式菜单](#menu---交互式菜单)
  - [data - 数据管理](#data---数据管理)
  - [strategy - 策略管理](#strategy---策略管理)
  - [execution - 执行自动化](#execution---执行自动化)
  - [task - 任务调度](#task---任务调度)
  - [analyze - 分析工具](#analyze---分析工具)
  - [watchlist - 自选池](#watchlist---自选池)
  - [market - 市场分析](#market---市场分析)
  - [monitor - 实时监控](#monitor---实时监控)
  - [stop - 止盈止损](#stop---止盈止损)
  - [trade - 模拟交易](#trade---模拟交易)
  - [risk - 风险管理](#risk---风险管理)
  - [status - 系统状态](#status---系统状态)
- [数据源](#数据源)
- [API 参考](#api-参考)
- [常见问题](#常见问题)

---

## 项目简介

**Quantix-Rust** 是一个 A股量化交易 CLI 工具，使用 Rust 实现高性能的数据采集、回测分析和任务调度功能。

### 核心特性

- **高性能数据采集** - 支持 TDX、AkShare、EastMoney 多数据源
- **实时行情处理** - 竞价数据采集、K线聚合
- **回测引擎** - 事件驱动回测框架，完整性能指标计算
- **任务调度** - 基于 Cron 的定时任务调度器
- **复权处理** - 通达信 day 文件解析与复权因子计算

### 技术架构

```
┌─────────────────────────────────────────────────────────┐
│                    Python quantix                      │
│           (数据采集、存储、Web API)                      │
└─────────────────────────────────────────────────────────┘
                          ↕ 共享数据库
┌─────────────────────────────────────────────────────────┐
│                      quantix-rust                       │
│           (高性能回测、实时分析、任务调度)                 │
└─────────────────────────────────────────────────────────┘
```

---

## 安装与配置

### 环境要求

- Rust 1.70+
- PostgreSQL 17+ (可选)
- TDengine 3.3+ (可选)
- ClickHouse (推荐用于 OLAP 分析)

### 安装步骤

```bash
# 克隆项目
git clone https://github.com/chengjon/quantix-rust.git
cd quantix-rust

# 构建发布版本
cargo build --release

# 或直接安装到系统
cargo install --path .
```

### 配置环境变量

```bash
# PostgreSQL
export POSTGRES_URL="postgresql://localhost:5432/quantix"

# ClickHouse
export CLICKHOUSE_URL="http://localhost:8123"
export CLICKHOUSE_DB="quantix"

# 策略 signal daemon 的本地 TDX fallback（可选）
export QUANTIX_TDX_ROOT="$HOME/.local/share/tdx"
export QUANTIX_TDX_MARKET="sz"

# TDX 数据源
export TDX_HOST="192.168.1.100"
export TDX_PORT=7709

# 自选池 JSON 存储路径（可选）
export QUANTIX_WATCHLIST_PATH="$HOME/.quantix/watchlist/watchlist.json"

# 监控告警 / 止盈止损 SQLite 路径（可选）
export QUANTIX_MONITOR_DB_PATH="$HOME/.quantix/monitor/alerts.db"

# 模拟交易 JSON 路径（可选）
export QUANTIX_TRADE_PATH="$HOME/.quantix/trade/paper_trade.json"

# 风控 JSON 路径（可选）
export QUANTIX_RISK_PATH="$HOME/.quantix/risk/risk_state.json"

# 策略运行时审计 SQLite 路径（可选）
export QUANTIX_STRATEGY_RUNTIME_DB_PATH="$HOME/.quantix/strategy/runtime.db"

# execution daemon JSON 配置路径（可选）
export QUANTIX_EXECUTION_CONFIG_PATH="$HOME/.quantix/execution/config.json"
```

### 运行测试

```bash
# 所有测试
cargo test

# 单个模块测试
cargo test --package quantix-cli --lib analysis::backtest::tests
```

---

## 快速开始

### 查看帮助信息

```bash
quantix --help
```

### 初始化配置

```bash
quantix init -c ../config
```

### 查看系统状态

```bash
quantix status
```

### 检查数据库连接

```bash
quantix status --health
```

### 自选池快速开始

```bash
# 创建分组并添加股票
quantix watchlist group create --name core
quantix watchlist add --code 000001 --group core

# 添加标签并查看列表
quantix watchlist tag add --code 000001 --tag bank
quantix watchlist list --group core --with-price

# 查看本地历史
quantix watchlist history --code 000001 --limit 20
```

### 市场分析快速开始

```bash
# 查看行业板块和概念板块
quantix market sector --top 10
quantix market concept --date 2026-03-09

# 查看北向资金和市场情绪
quantix market north --date 2026-03-09
quantix market sentiment

# 查看龙头股和概览
quantix market leader --sector 银行 --limit 5
quantix market overview --top 5
```

### 实时监控快速开始

```bash
# 一次性扫描自选池并显示触发告警
quantix monitor watchlist --once

# 添加、查看和删除价格告警
quantix monitor alert add 000001 --above 16.0
quantix monitor alert add 000001 --below 15.0
quantix monitor alert list
quantix monitor alert remove 1
```

### 止盈止损快速开始

```bash
# 为自选池代码设置规则
quantix stop set 000001 --loss 14.5
quantix stop set 000001 --trailing 5 --profit 18.0

# 通过 monitor 扫描同时评估价格告警和止盈止损
quantix monitor watchlist --once

# 查看和删除规则
quantix stop list
quantix stop remove 000001
```

### 模拟交易快速开始

```bash
# 初始化默认模拟账户
quantix trade init --capital 1000000

# 立即成交的限价买卖
quantix trade buy 000001 --price 15.0 --volume 1000
quantix trade sell 000001 --price 16.0 --volume 500

# 查看持仓与现金快照
quantix trade position
quantix trade cash
```

### 风控快速开始

```bash
# 基于纸面账户设置风控规则
quantix risk rule set --type position-limit --value 20%
quantix risk rule set --type daily-loss-limit --value 50000
quantix risk rule set --type industry-blocklist --value 银行,地产

# 查看规则和当前状态
quantix risk rule list
quantix risk status
quantix risk log

# 触发当日买入锁后可手动释放
quantix risk lock release
```

---

## 命令参考

### init - 初始化

初始化配置和数据库连接。

#### 用法

```bash
quantix init [-c|--config <PATH>]
```

#### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--config` | `-c` | 配置文件路径 | `../config` |

#### 示例

```bash
# 使用默认配置路径
quantix init

# 指定配置路径
quantix init -c /etc/quantix/config.toml
```

#### 输出

```
初始化 Quantix CLI...
配置路径: ../config
```

---

### menu - 交互式菜单

启动交互式菜单界面。

#### 用法

```bash
quantix menu [--tui]
```

#### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--tui` | 启用 TUI 界面 (ratatui) | false |

#### 示例

```bash
# 简单文本菜单
quantix menu

# TUI 界面菜单
quantix menu --tui
```

#### 输出

```
=== Quantix CLI 交互菜单 ===
1. 数据同步
2. 策略运行
3. 回测分析
4. 任务管理
0. 退出
```

---

### data - 数据管理

管理历史数据的查询和导出。

#### 子命令

- `query` - 查询历史数据
- `export` - 导出数据到文件

#### data query - 查询历史数据

##### 用法

```bash
quantix data query -c <CODE> [-s|--start <DATE>] [-e|--end <DATE>] [--type <TYPE>] [-l|--limit <N>]
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--code` | `-c` | 股票代码 | 必填 |
| `--start` | `-s` | 开始日期 (YYYYMMDD) | 无 |
| `--end` | `-e` | 结束日期 (YYYYMMDD) | 无 |
| `--type` | - | 数据类型 | `daily` |
| `--limit` | `-l` | 限制返回条数 | `100` |

##### 数据类型

| 类型 | 说明 |
|------|------|
| `daily` | 日线数据 |
| `1m` | 1分钟线 |
| `5m` | 5分钟线 |
| `15m` | 15分钟线 |
| `30m` | 30分钟线 |
| `60m` | 60分钟线 |

##### 示例

```bash
# 查询平安银行最近100条日线数据
quantix data query -c 000001

# 查询指定日期范围的数据
quantix data query -c 000001 -s 20240101 -e 20241231

# 查询1分钟线数据
quantix data query -c 000001 --type 1m -l 500

# 查询5分钟线数据
quantix data query -c 000001 --type 5m
```

##### 输出

```
查询数据: 000001 (daily)
日期范围: Some("20240101") - Some("20241231")
限制: 100
```

---

#### data export - 导出数据

##### 用法

```bash
quantix data export -c <CODE> [--format <FORMAT>] [-o|--output <DIR>]
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--code` | `-c` | 股票代码 | 必填 |
| `--format` | - | 输出格式 | `parquet` |
| `--output` | `-o` | 输出目录 | `./data` |

##### 支持格式

| 格式 | 说明 |
|------|------|
| `parquet` | Parquet 列式存储 |
| `csv` | CSV 文本格式 |
| `json` | JSON 格式 |

##### 示例

```bash
# 导出为 Parquet 格式
quantix data export -c 000001

# 导出为 CSV 格式
quantix data export -c 000001 --format csv

# 指定输出目录
quantix data export -c 000001 -o /data/exports
```

##### 输出

```
导出数据: 000001 -> ./data (parquet)
```

---

### strategy - 策略管理

管理量化交易策略的运行和查看。

#### 子命令

- `run` - 运行策略
- `list` - 列出所有策略
- `show` - 显示策略详情

#### strategy run - 运行策略

##### 用法

```bash
quantix strategy run -n <NAME> [--mode <MODE>] [-c|--code <CODE>]
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--name` | `-n` | 策略名称 | 必填 |
| `--mode` | - | 运行模式 | `backtest` |
| `--code` | `-c` | 股票代码 | 全部 |

##### 运行模式

| 模式 | 说明 |
|------|------|
| `backtest` | 回测模式 |
| `live` | 实盘模式 (开发中) |
| `paper` | 模拟盘模式（当前支持 `ma_cross` 单次执行） |
| `mock_live` | mock-live 模式（支持非终态订单生命周期模拟） |

##### 示例

```bash
# 运行均线交叉策略回测
quantix strategy run -n ma_cross

# 运行策略回测指定股票
quantix strategy run -n ma_cross -c 000001

# 使用 paper 模式单次执行
quantix trade init --capital 1000000
quantix strategy run -n ma_cross --mode paper -c 000001

# 使用 mock_live 模式单次执行
quantix strategy run -n ma_cross --mode mock_live -c 000001

# 使用实盘模式
quantix strategy run -n ma_cross --mode live
```

##### 输出

```
运行策略: ma_cross (backtest)
股票代码: 000001
```

##### 当前 Phase 29A 边界

- `paper` 模式当前只支持 `ma_cross`
- `paper` 模式当前只支持单代码、单次执行
- `mock_live` 模式当前支持非终态订单生命周期模拟
- 首次使用前请先执行 `quantix trade init`
- 运行审计默认写入 `~/.quantix/strategy/runtime.db`
- 可通过 `QUANTIX_STRATEGY_RUNTIME_DB_PATH` 覆盖该路径
- `mock_live` 可能返回 `accepted`、`partially_filled`、`unknown` 等非终态状态
- 同一个 mock-live 订单在 partial fill 路径下可能生成多笔 `TradeRecord`
- 这些增量成交会直接出现在 `trade history`、`trade fees`、`trade overview` 的本地视图中
- `live` 模式仍在开发中

##### Phase 29B: 策略信号守护进程

```bash
quantix strategy config init
quantix strategy config show

quantix strategy daemon run --once
quantix strategy daemon run

quantix strategy signal list --approval-status pending
quantix strategy signal approve --signal-id <ID> --target-mode paper --target-account default
quantix strategy signal reject --signal-id <ID> --reason "manual reject"
quantix strategy request list --status pending
quantix strategy request execute --request-id <ID>
quantix strategy request cancel --request-id <ID> [--reason <TEXT>]

quantix strategy service-config show
quantix strategy service-config set --quantix-bin /abs/path/to/quantix --env-file /abs/path/to/service.env
quantix strategy service install
quantix strategy service start
quantix strategy service status
```

默认路径：

- `~/.quantix/strategy/config.json`
- `~/.quantix/strategy/runtime.db`
- `~/.quantix/strategy/service.json`
- `~/.quantix/strategy/service.env`
- `~/.local/bin/quantix-strategy-run`

当前 Phase 29B 边界：

- `strategy daemon` 当前只支持单代码
- 同一代码下可配置多个策略实例
- 首次启动只 bootstrap 到最新 bar，不回补历史 signal
- `strategy daemon run --once` 首次启动可能只输出 `strategy daemon 未生成新信号`
- daemon 优先读取已落库日线；主读取器返回空或失败时，可回退到本地 TDX `day` 文件
- `QUANTIX_TDX_ROOT` 用于指定本地 TDX 根目录
- `QUANTIX_TDX_MARKET` 用于在 `sh/sz/bj/ds` 之间消解同代码歧义
- signal 批准后只会写入 `execution_request`
- `request execute` 会手动消费一个 `pending execution_request`
- 不会自动交易，不会修改 paper 账户
- `strategy run --mode paper` 仍保留为直接执行路径
- `execution daemon`、自动审批、live adapter 延后到后续 Phase

当前输出语义：

- `strategy signal list` 会输出 `source=<SOURCE> fallback=<BOOL>`
- `strategy signal approve` 会输出 `request_id signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS>`
- `strategy signal reject` 会输出 `signal_id signal_status=<STATUS> approval_status=<STATUS> reason=<TEXT>`
- `strategy request list` 会输出 `request_id signal=<ID> target=<MODE>/<ACCOUNT> status=<STATUS>`
- mock_live request 即使返回 `accepted` 也会被标记为 `completed`
- `strategy service install/start/stop/enable/disable` 成功时会输出明确消息

---

### execution - 执行自动化

执行 `pending execution_request` 的前台守护进程与配置命令。

#### 子命令

- `config` - 初始化或查看 execution daemon 配置
- `daemon` - 前台运行 execution daemon

#### 用法

```bash
quantix execution config init
quantix execution config show
quantix execution daemon run
quantix execution daemon run --once
```

#### 默认路径

- `~/.quantix/execution/config.json`
- `QUANTIX_EXECUTION_CONFIG_PATH`

#### 当前 Phase 29C 边界

- `execution_request` 当前新增 `in_progress`
- `execution daemon` 只会 claim/consume `pending execution_request`
- `strategy request execute` 与 `execution daemon` 复用同一条 request 消费路径
- `manual|always` 是当前 auto-approval 的全部策略面
- `manual` 下仍需显式 `strategy signal approve`
- `always` 下 `strategy daemon` 生成 signal 后会直接创建 `pending execution_request`
- `execution daemon` 不负责 signal 审批
- `execution daemon` 当前是单 worker、串行消费
- request 进入 `completed` 只表示成功进入执行层，不代表订单已终态
- `mock_live` request 即使返回 `accepted` 也会被标记为 `completed`
- `live` adapter 仍未实现

#### 当前输出语义

- `quantix execution config init/show` 会输出完整 JSON 配置
- `quantix execution daemon run --once` 在没有待消费 request 时会输出 `execution daemon 未找到 pending request`
- request 消费成功时会输出 `execution daemon consumed request status=completed`
- request 消费失败时会输出 `execution daemon consumed request status=failed`

---

#### strategy list - 列出策略

##### 用法

```bash
quantix strategy list
```

##### 示例

```bash
quantix strategy list
```

##### 输出

```
可用策略:
  - ma_cross: 均线交叉策略
```

---

#### strategy show - 显示策略详情

##### 用法

```bash
quantix strategy show -n <NAME>
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--name` | `-n` | 策略名称 | 必填 |

##### 示例

```bash
quantix strategy show -n ma_cross
```

##### 输出

```
策略详情: ma_cross
```

---

### task - 任务调度

管理定时任务的调度和执行。

#### 子命令

- `add` - 添加定时任务
- `list` - 列出所有任务
- `start` - 启动任务调度器
- `stop` - 停止任务调度器
- `status` - 查看任务状态

#### task add - 添加任务

##### 用法

```bash
quantix task add -n <NAME> [--cron <CRON>] -c <COMMAND>
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--name` | `-n` | 任务名称 | 必填 |
| `--cron` | - | Cron 表达式 | 必填 |
| `--command` | `-c` | 执行命令 | 必填 |

##### Cron 表达式格式

支持标准 Cron 表达式和扩展语法：

| 语法 | 说明 | 示例 |
|------|------|------|
| `*` | 任意值 | `* * * * *` |
| `,` | 列表 | `1,2,3 * * * *` |
| `-` | 范围 | `1-5 * * * *` |
| `*/N` | 步长 | `*/5 * * * *` |

格式: `分 时 日 月 周`

##### 示例

```bash
# 每天 9:30 执行数据同步
quantix task add -n morning_sync --cron "30 9 * * 1-5" -c "data sync"

# 每5分钟执行一次
quantix task add -n monitor --cron "*/5 * * * *" -c "status check"

# 盘前任务 (每个交易日 8:30)
quantix task add -n pre_market --cron "30 8 * * 1-5" -c "strategy run -n pre_market"
```

##### 输出

```
添加任务: morning_sync
Cron: 30 9 * * 1-5
命令: data sync
```

---

#### task list - 列出任务

##### 用法

```bash
quantix task list
```

##### 示例

```bash
quantix task list
```

##### 输出

```
定时任务列表:
```

---

#### task start - 启动调度器

##### 用法

```bash
quantix task start [--daemon]
```

##### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--daemon` | 后台运行 | false |

##### 示例

```bash
# 前台运行
quantix task start

# 后台运行
quantix task start --daemon
```

##### 输出

```
启动任务调度器...
```

---

#### task stop - 停止调度器

##### 用法

```bash
quantix task stop
```

##### 示例

```bash
quantix task stop
```

##### 输出

```
停止任务调度器...
```

---

#### task status - 查看状态

##### 用法

```bash
quantix task status
```

##### 示例

```bash
quantix task status
```

##### 输出

```
任务调度器状态:
```

---

### analyze - 分析工具

计算技术指标、查看回测报告，以及对小范围股票池执行日线选股筛选。

#### 子命令

- `indicators` - 计算技术指标
- `backtest` - 查看回测报告
- `screener` - 运行日线选股筛选

#### analyze indicators - 计算技术指标

##### 用法

```bash
quantix analyze indicators -c <CODE> -i <INDICATORS>
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--code` | `-c` | 股票代码 | 必填 |
| `--indicators` | `-i` | 指标列表 (逗号分隔) | 必填 |

##### 支持的指标

| 指标 | 说明 |
|------|------|
| `ma` | 移动平均线 |
| `ema` | 指数移动平均 |
| `rsi` | 相对强弱指标 |
| `macd` | 平滑异同移动平均线 |
| `kdj` | 随机指标 |
| `boll` | 布林带 |
| `atr` | 平均真实波幅 |
| `volume_ma` | 成交量均线 |

##### 示例

```bash
# 计算移动平均线
quantix analyze indicators -c 000001 -i ma5,ma10,ma20

# 计算多个指标
quantix analyze indicators -c 000001 -i ma5,ma10,rsi6,rsi12,macd

# 计算布林带
quantix analyze indicators -c 000001 -i boll
```

##### 输出

```
计算技术指标: 000001
指标: ma5,ma10,ma20
```

---

#### analyze backtest - 回测报告

##### 用法

```bash
quantix analyze backtest -i <ID>
```

##### 参数

| 参数 | 简写 | 说明 | 默认值 |
|------|------|------|--------|
| `--id` | `-i` | 回测 ID | 必填 |

##### 示例

```bash
quantix analyze backtest -i bt_20240101_000001
```

##### 输出

```
回测报告: bt_20240101_000001
```

---

#### analyze screener - 日线选股筛选

按你提供的小范围股票池运行参数化 preset。P0 只支持单指标 preset，多个 `--preset` 之间使用 `AND` 组合。

##### 用法

```bash
quantix analyze screener preset-list
quantix analyze screener run (--codes <CSV> | --watchlist [--group <NAME>]) --preset <SPEC> [--preset <SPEC> ...] [--sort-by <FIELD>] [--limit <N>]
```

##### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--codes` | 显式股票代码列表，逗号分隔 | 无 |
| `--watchlist` | 使用本地自选池作为股票池 | false |
| `--group` | 自选池分组，仅在 `--watchlist` 下生效 | 无 |
| `--preset` | 单个筛选条件，可重复传入 | 必填 |
| `--sort-by` | 排序字段：`code` 或 `score` | `code` |
| `--limit` | 限制返回条数 | 无 |

##### P0 约束

- 仅支持日线数据
- 必须二选一指定 `--codes` 或 `--watchlist`
- 不支持全市场扫描
- 不支持 DSL、自定义表达式、OR 逻辑
- `preset` 必须写成 `name:key=value,key=value`

##### 支持的 preset

| Preset | 参数 | 说明 |
|--------|------|------|
| `close_above_ma` | `period=<n>` | 收盘价高于均线 |
| `close_below_ma` | `period=<n>` | 收盘价低于均线 |
| `rsi_gte` | `period=<n>,value=<x>` | RSI 大于等于阈值 |
| `rsi_lte` | `period=<n>,value=<x>` | RSI 小于等于阈值 |
| `volume_ratio_gte` | `window=<n>,value=<x>` | 量比大于等于阈值 |

##### 示例

```bash
# 查看 preset 列表
quantix analyze screener preset-list

# 对显式代码列表筛选
quantix analyze screener run \
  --codes 000001,600519 \
  --preset close_above_ma:period=20

# 对自选池分组做 AND 组合筛选
quantix analyze screener run \
  --watchlist \
  --group core \
  --preset close_above_ma:period=20 \
  --preset volume_ratio_gte:window=5,value=1.5 \
  --sort-by score \
  --limit 20
```

##### 输出说明

- `命中=yes/no` 表示该股票是否同时满足全部 preset
- `评分` 仅用于当前 P0 排序，代表规则相对阈值的简单偏离度
- 数据不足时不会中断整次筛选，该规则会显示为未命中并附带原因

---

### watchlist - 自选池

管理本地 JSON 自选池，支持分组、标签、历史和最佳努力价格展示。

#### 存储路径

- 默认路径：`~/.quantix/watchlist/watchlist.json`
- 可通过 `QUANTIX_WATCHLIST_PATH` 覆盖
- `list --with-price` 会尝试补充名称和实时价格；外部行情不可用时会降级为空值，不影响命令返回

#### 子命令

- `add` - 添加股票
- `remove` - 删除股票
- `list` - 列出自选池
- `move` - 移动股票到目标分组
- `group create/list` - 分组管理
- `tag add/remove/list` - 标签管理
- `history` - 查看本地操作历史

#### 常用示例

```bash
quantix watchlist add --code 000001
quantix watchlist add --code 600519 --group core
quantix watchlist tag add --code 000001 --tag bank
quantix watchlist list --tag bank
quantix watchlist list --with-price
quantix watchlist history --code 000001 --limit 20
```

#### 命令摘要

```bash
quantix watchlist add --code 000001 [--group core]
quantix watchlist remove --code 000001
quantix watchlist list [--group core] [--tag bank] [--with-price]
quantix watchlist move --code 000001 --group core
quantix watchlist group create --name core
quantix watchlist group list
quantix watchlist tag add --code 000001 --tag bank
quantix watchlist tag remove --code 000001 --tag bank
quantix watchlist tag list --code 000001
quantix watchlist history [--code 000001] [--limit 20]
```

---

### market - 市场分析

查看 Phase 23 P0 的市场日度快照，包括板块排名、北向资金、市场情绪、龙头股和综合概览。

#### P0 范围

- 仅支持日度快照
- 仅支持只读查询
- 历史/详情/实时能力延后到后续 Phase

#### 命令摘要

```bash
quantix market sector [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]
quantix market concept [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]
quantix market north [--date <YYYY-MM-DD>]
quantix market sentiment [--date <YYYY-MM-DD>]
quantix market leader (--sector <NAME> | --concept <NAME> | --all) [--limit <N>] [--date <YYYY-MM-DD>]
quantix market overview [--top <N>] [--date <YYYY-MM-DD>]
```

#### 参数约束

- `--date` 格式必须是 `YYYY-MM-DD`
- `sector` / `concept` 的 `--sort-by` 当前只支持 `change` 或 `change_pct`
- `leader` 必须且只能指定 `--sector`、`--concept`、`--all` 之一
- `overview` 会组合行业、概念、北向资金和市场情绪四部分数据

#### 常用示例

```bash
quantix market sector --top 10
quantix market concept --date 2026-03-09
quantix market north
quantix market sentiment --date 2026-03-09
quantix market leader --concept 人工智能 --limit 10
quantix market overview --top 5
```

#### 延后能力

- 历史/详情/实时能力延后到后续 Phase
- 当前不支持 `sector show` / `concept show`
- 当前不支持 `north --history` / `north --stocks`
- 当前不支持 `sentiment --detail`

---

### monitor - 实时监控

提供 Phase 24B 的最小监控自动化闭环：一次性/重复自选池扫描、持久化价格告警、守护进程入口、`systemd --user` 服务管理，以及业务事件历史。

#### 存储路径

- 默认路径：`~/.quantix/monitor/alerts.db`
- 可通过 `QUANTIX_MONITOR_DB_PATH` 覆盖
- 告警使用 SQLite 持久化，`watchlist --once` 命中时会在终端输出并更新最后触发时间

#### 配置路径

- 默认路径：`~/.quantix/monitor/config.json`
- 可通过 `QUANTIX_MONITOR_CONFIG_PATH` 覆盖
- `watchlist --repeat`、`daemon run`、`service` 命令共享同一份 monitor 配置

#### Service 配置路径

- 默认路径：`~/.quantix/monitor/service.json`
- service wrapper 路径：`~/.local/bin/quantix-monitor-run`
- `service install` 会从 `service.json` 读取稳定的 `quantix` 二进制绝对路径

#### P0 范围

- 支持 `watchlist --once`、`watchlist --repeat`、`daemon run`
- 支持 `systemd --user` 用户服务的安装、启停、状态查看、自启开关
- 支持 `service-config show` / `service-config set --quantix-bin`
- 支持价格阈值告警的添加、列表、删除，以及业务事件历史查看
- 业务事件历史只记录价格告警命中和 stop 触发，不记录服务生命周期日志
- 当前后台服务能力面向 WSL2/Linux 的 `systemd --user`
- `service install` 要求 `service.json` 中的 `quantix` 路径存在且可执行
- `service uninstall` 会要求先执行 `service stop`
- `--refresh`、系统通知延后到后续 Phase

#### 命令摘要

```bash
quantix monitor watchlist --once
quantix monitor watchlist --repeat
quantix monitor alert add <CODE> (--above <PRICE> | --below <PRICE>)
quantix monitor alert list
quantix monitor alert remove <ID>
quantix monitor config show
quantix monitor config set --interval-seconds <N>
quantix monitor config set --group <GROUP>
quantix monitor config clear-group
quantix monitor config set --persist-events <true|false>
quantix monitor daemon run
quantix monitor service install
quantix monitor service uninstall
quantix monitor service start
quantix monitor service stop
quantix monitor service status
quantix monitor service enable
quantix monitor service disable
quantix monitor service-config show
quantix monitor service-config set --quantix-bin /absolute/path/to/quantix
quantix monitor event list [--limit <N>] [--code <CODE>] [--type <TYPE>]
```

#### 参数约束

- `watchlist` 当前必须且只能显式带 `--once` 或 `--repeat`
- `alert add` 必须且只能指定一个阈值：`--above` 或 `--below`
- `config set` 每次只允许修改一个字段
- `event list` 默认返回最近 20 条业务事件
- `service` 命令调用 `systemctl --user`
- `service-config set --quantix-bin` 必须传绝对路径
- 当前只复用现有自选池与 TDX 行情链路，不提供板块/概念监控

#### 常用示例

```bash
quantix monitor watchlist --once
quantix monitor watchlist --repeat
quantix monitor alert add 000001 --above 16.0
quantix monitor alert add 000001 --below 15.0
quantix monitor alert list
quantix monitor alert remove 1
quantix monitor config show
quantix monitor service install
quantix monitor service-config set --quantix-bin /usr/local/bin/quantix
quantix monitor event list --limit 10
```

---

### stop - 止盈止损

提供 Phase 25B 的 stop 闭环：为自选池代码维护固定价、百分比和 trailing 规则，支持局部更新、状态查看、历史审计，并在 `monitor watchlist --once` 中继续复用当前快照价格做评估。

#### 存储路径

- 默认路径：`~/.quantix/monitor/alerts.db`
- 复用 `QUANTIX_MONITOR_DB_PATH` 指向的 SQLite 路径
- 止盈止损规则与价格告警共用同一个 monitor DB

#### P0 范围

- 支持固定止损价、固定止盈价、百分比止损、百分比止盈、跟踪止损百分比
- 仅允许为当前本地自选池中的代码设置规则
- 每个代码只保留一条有效规则，重复 `stop set` 会整条覆盖旧规则
- `stop update` 采用 patch 语义，只修改显式传入的字段
- `quantix monitor watchlist --once` 会在输出监控快照后继续评估止盈止损规则
- 百分比阈值优先使用当前本地 `paper` 持仓 `avg_cost` 作为锚点
- 没有持仓时退回到规则持久化的 `reference_price`
- `stop status` 会显示 `anchor_source`、有效阈值和 `eval_state`
- `stop history` 会记录 `set`、`update`、`remove` 和 `trigger` 事件
- 当前不自动卖出，不直接触发执行请求

#### 命令摘要

```bash
quantix stop set <CODE> [--loss <PRICE>] [--profit <PRICE>] [--loss-pct <PCT>] [--profit-pct <PCT>] [--trailing <PCT>]
quantix stop update <CODE> [--loss <PRICE>] [--profit <PRICE>] [--loss-pct <PCT>] [--profit-pct <PCT>] [--trailing <PCT>] [--clear-loss] [--clear-profit] [--clear-loss-pct] [--clear-profit-pct] [--clear-trailing]
quantix stop list
quantix stop status [--code <CODE>]
quantix stop history [--code <CODE>] [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]
quantix stop remove <CODE>
```

#### 参数约束

- `stop set` / `stop update` 至少需要一个有效阈值或清理动作
- `--loss` 与 `--loss-pct` 互斥，`--profit` 与 `--profit-pct` 互斥
- `--trailing` 与 `--loss` / `--loss-pct` 互斥
- `--loss`、`--profit` 必须是有限正数
- `--loss-pct`、`--profit-pct` 必须是有限正数
- `--trailing` 必须在 0 到 100 之间，且可以与 `--profit` 组合
- `stop history --type` 当前支持：`set`、`update`、`remove`、`trigger`

#### 常用示例

```bash
quantix stop set 000001 --loss 14.5
quantix stop set 000001 --profit 18.0
quantix stop set 000001 --loss-pct 5
quantix stop update 000001 --profit-pct 12 --clear-profit
quantix stop set 000001 --trailing 5 --profit 18.0
quantix stop status --code 000001
quantix stop history --code 000001 --limit 10
quantix stop list
quantix monitor watchlist --once
quantix stop remove 000001
```

---

### trade - 模拟交易

提供 Phase 26A/28A 的最小模拟交易闭环：初始化/重置单账户、本地 JSON 持久化、按输入价格立即成交的限价买卖，以及查看成交历史、费用明细、账户概览和当前持仓视图。

#### 存储路径

- 默认路径：`~/.quantix/trade/paper_trade.json`
- 可通过 `QUANTIX_TRADE_PATH` 覆盖

#### P0 范围

- 仅支持单账户、本地纸上交易
- 买卖单按输入价格立即成交，不包含挂单、撤单、部分成交
- 手续费参数只允许通过 `trade init` / `trade reset` 设置
- `trade history`、`trade fees`、`trade overview` 是本地只读视图
- `trade overview --current`、`trade position --current` 使用 best-effort 实时行情
- 拿不到价格时降级为空，不让命令整体失败

#### 命令摘要

```bash
quantix trade init [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]
quantix trade reset [--capital <AMOUNT>] [--commission-rate <RATE>] [--commission-min <AMOUNT>] [--stamp-duty-rate <RATE>] [--transfer-fee-rate <RATE>]
quantix trade buy <CODE> --price <PRICE> --volume <N>
quantix trade sell <CODE> --price <PRICE> --volume <N>
quantix trade history [--code <CODE>] [--limit <N>]
quantix trade fees [--code <CODE>] [--limit <N>]
quantix trade overview [--current]
quantix trade position [--current]
quantix trade cash
```

#### 参数约束

- `trade init` / `trade reset` 只管理默认账户和费率配置
- `trade buy` / `trade sell` 的 `--price` 必须是有限正数，`--volume` 必须是正整数
- `trade sell` 仅允许卖出当前已持有且数量足够的代码
- `trade history` / `trade fees` 默认返回最近 20 条，支持 `--code` 和 `--limit`
- `trade overview --current` / `trade position --current` 依赖 best-effort 实时行情
- best-effort 实时行情拿不到价格时降级为空，不阻塞命令返回

#### 常用示例

```bash
quantix trade init --capital 1000000 --commission-rate 0.00025
quantix trade reset --capital 500000
quantix trade buy 000001 --price 15.0 --volume 1000
quantix trade sell 000001 --price 16.0 --volume 500
quantix trade history --code 000001 --limit 10
quantix trade fees --limit 10
quantix trade overview --current
quantix trade position --current
quantix trade cash
```

---

### risk - 风险管理

提供 Phase 27D 的风控闭环：在保留本地 paper-trade 风控的同时，支持导入标准化实盘流水、重建只读镜像账户，并通过 `--source paper|live_import` 显式切换风险视图。`industry-blocklist` 现已成为受支持的风险规则。

#### 存储路径

- 默认路径：`~/.quantix/risk/risk_state.json`
- 可通过 `QUANTIX_RISK_PATH` 覆盖

#### P0 范围

- 默认数据源仍是本地 paper-trade 账户
- 支持导入标准化 `CSV/JSON` 实盘流水并重建只读镜像账户
- 仅支持 `position-limit`、`daily-loss-limit`、`volatility-limit`、`industry-blocklist` 四类规则
- `trade buy` 会执行风控预检查，`trade sell` 仍然允许成交
- `trade init` / `trade reset` 会清除当日买入锁并保留已配置规则
- 日亏损只基于本地 paper-trade 账户资产快照，不做实时行情盯市
- `--source live_import` 只读，不会回写 `paper_trade.json`
- `--source live_import` 要求显式指定 `--account`
- `volatility-limit` 固定使用 `ATR(14) / latest_close * 100`
- `volatility-limit` 缺少或不足日线时会拒绝买单
- Phase 27D v1 使用 `SW 一级行业` 作为运行时生效标准
- `security_class_2024` / CSRC 2024 仍保留在系统中作为并行分类标准，不是该 v1 规则的运行时生效标准
- 运行时风险评估只读取本地 SQLite reference/snapshot 表
- MySQL 仅作为上游同步来源，不参与运行时查询
- 最终运行时边界保持为 ClickHouse + SQLite；MySQL 仅负责上游同步
- 运行时解析顺序：1. 当前 SW 映射 2. 查询月份快照 3. 历史 SW 映射 4. 最新本地快照
- 月度快照会在该月第一次成功命中 `SW 一级行业` 时冻结
- `industry-blocklist` 采用精确字符串匹配，不做模糊归一化
- `industry-blocklist` 不影响卖出路径
- 实盘导入当前只支持项目标准化 CSV/JSON
- failed rebuild 不会覆盖上一次成功镜像状态
- `risk status` 会显示锁状态来源、作用交易日、触发原因、触发时间
- `risk log` 仅记录规则变更、日亏损锁触发、手动释放、以及 rollover/reset 清锁事件
- `risk lock release` 仅对当前交易日生效，当日内不再自动重新锁定；次日或 `trade init/reset` 会自动清除该手动释放标记
- `risk log` 当前支持按事件写入日 `--date` 和事件类型 `--type` 过滤
- 行业白名单、自动减仓 继续延后到后续 Phase

#### 命令摘要

```bash
quantix risk import live-trades --account <ID> --input <FILE>
quantix risk rebuild live-account --account <ID>
quantix risk rule set --type position-limit --value 20%
quantix risk rule set --type daily-loss-limit --value 50000
quantix risk rule set --type daily-loss-limit --value 5%
quantix risk rule set --type volatility-limit --value 4%
quantix risk rule set --type industry-blocklist --value 银行,地产
quantix risk rule list
quantix risk rule enable --type position-limit
quantix risk rule disable --type daily-loss-limit
quantix risk status --source paper|live_import [--account <ID>]
quantix risk pnl --source paper|live_import [--account <ID>]
quantix risk position --source paper|live_import [--account <ID>]
quantix risk log [--limit <N>] [--date <YYYY-MM-DD>] [--type <EVENT>]
quantix risk lock release
```

#### 参数约束

- `position-limit` 仅接受百分比值，例如 `20%`
- `daily-loss-limit` 同时支持金额值和百分比值，例如 `50000` 或 `5%`
- `volatility-limit` 仅接受百分比值，例如 `4%`
- `volatility-limit` 固定使用 `ATR(14) / latest_close * 100`
- `industry-blocklist` 现已成为受支持的风险规则
- `industry-blocklist` 的值按逗号分隔行业名称，例如 `银行,地产`
- `risk status`、`risk pnl`、`risk position` 依赖已初始化的 paper-trade 账户；首次使用前请先执行 `quantix trade init`
- `risk import live-trades` 当前只接受项目标准化 `CSV/JSON`
- `risk rebuild live-account` 始终做全量 replay，不做增量重建
- `--source live_import` 要求显式指定 `--account`
- live_import 镜像账户不会回写 `paper_trade.json`
- failed rebuild 不会覆盖上一次成功镜像状态
- `volatility-limit` 缺少或不足日线时会拒绝买单
- 当日买入锁触发后，新的 `trade buy` 会被拒绝，但 `trade sell` 仍允许执行
- `risk status` 的 `状态来源` 当前只区分 `open`、`daily_loss_locked`、`manual_release_active`
- `risk log` 不依赖已初始化的 paper-trade 账户；没有事件时返回空列表视图
- `risk log` 默认返回最近 20 条事件，可用 `--limit` 调整，并支持 `--date`、`--type` 单独或组合过滤
- `risk log --date` 匹配事件写入日，也就是 `event.ts` 所在日期
- `risk lock release` 仅在存在活动买入锁时可用；若同日已手动释放，再次执行会返回成功且保持幂等
- `risk lock release` 在当日内抑制基于日亏损规则的自动重新加锁；次日或 `trade init/reset` 会清除该手动释放标记

#### 常用示例

```bash
quantix trade init --capital 1000000
quantix risk import live-trades --account live-001 --input /tmp/live.csv
quantix risk rebuild live-account --account live-001
quantix risk rule set --type position-limit --value 20%
quantix risk rule set --type daily-loss-limit --value 5%
quantix risk rule set --type volatility-limit --value 4%
quantix risk rule set --type industry-blocklist --value 银行,地产
quantix risk status
quantix risk status --source live_import --account live-001
quantix risk pnl
quantix risk pnl --source live_import --account live-001
quantix risk position
quantix risk position --source live_import --account live-001
quantix risk log --limit 10
quantix risk log --date 2026-03-12 --type buy-lock-released
quantix trade buy 000001 --price 15.0 --volume 1000
quantix risk lock release
```

---

### status - 系统状态

查看系统状态和健康检查。

#### 用法

```bash
quantix status [--health]
```

#### 参数

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--health` | 检查数据库连接 | false |

#### 示例

```bash
# 查看系统状态
quantix status

# 检查数据库连接
quantix status --health
```

#### 输出

```
Quantix CLI 状态:
  版本: 0.1.0
  模式: 共享数据库模式
```

---

## 数据源

Quantix-Rust 支持多种数据源，可根据需要切换使用。

### TDX (通达信)

实时行情数据采集。

```rust
use quantix_cli::sources::TdxSource;

let source = TdxSource::new("192.168.1.100", 7709);
let quotes = source.get_quotes(&["000001", "000002"]).await?;
```

### TDX 文件解析

通达信 day/gbbq 文件解析和复权计算。

```rust
use quantix_cli::sources::{TdxDayFile, FuquanCalculator};

let day_file = TdxDayFile::new("/path/to/day")?;
let records = day_file.read_all()?;

let factors = FuquanCalculator::calculate(&records, None)?;
```

### AkShare

财务数据和历史数据采集。

```rust
use quantix_cli::sources::AkShareSource;

let source = AkShareSource::new();
let data = source.get_stock_info("000001").await?;
```

### EastMoney

实时行情、资金流向、财务数据。

```rust
use quantix_cli::sources::{EastMoneySource, Board};

let source = EastMoneySource::new();
let stocks = source.get_stock_list(Board::HS300.as_str()).await?;
let quotes = source.get_realtime_quotes(&["000001".to_string()]).await?;
```

### 竞价数据采集

集合竞价数据采集和分析。

```rust
use quantix_cli::sources::AuctionCollector;

let collector = AuctionCollector::new();
let quotes = collector.collect_auction_quotes().await?;
```

### K线聚合器

实时聚合 Tick 数据为 K线。

```rust
use quantix_cli::sources::{KlineAggregator, KlinePeriod};

let aggregator = KlineAggregator::new();
// 自动聚合为 1m/5m/15m/30m/60m/1d K线
```

---

## API 参考

### 核心模块

#### 数据模型 (`src/data/models.rs`)

```rust
// K线数据
pub struct Kline {
    pub code: String,
    pub date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub amount: Option<Decimal>,
    pub adjust_type: AdjustType,
}

// 复权类型
pub enum AdjustType {
    None = 0,
    QFQ = 1,  // 前复权
    HFQ = 2,  // 后复权
}

// GBBQ 除权除息事件
pub struct GbbqEvent {
    pub code: String,
    pub event_date: NaiveDate,
    pub category: u8,
    pub dividend: f32,
    pub bonus_price: f32,
    pub bonus_share: f32,
    pub rights_share: f32,
    pub ex_price: Option<f64>,
    pub record_date: Option<NaiveDate>,
}
```

#### ClickHouse 客户端 (`src/db/clickhouse.rs`)

```rust
use quantix_cli::db::ClickHouseClient;

// 创建客户端
let client = ClickHouseClient::new("http://localhost:8123", "quantix").await?;

// 初始化数据库
client.init_database().await?;

// 查询 K线数据
let klines = client.get_kline_data("000001", "1d", None, None, Some(100)).await?;

// 批量插入 (优化版本)
client.insert_kline_data_batch(&klines, "1d").await?;

// 设置批次大小
let client = client.with_batch_size(500);
```

#### 回测引擎 (`src/analysis/backtest.rs`)

```rust
use quantix_cli::analysis::backtest::{BacktestEngine, BacktestConfig};

let config = BacktestConfig {
    initial_capital: dec!(1000000),
    commission_rate: dec!(0.0003),
    slippage_rate: dec!(0.001),
    ..Default::default()
};

let mut engine = BacktestEngine::new(config);
let result = engine.run(&mut strategy, &data).await?;
```

#### 技术指标 (`src/analysis/indicators.rs`)

```rust
use quantix_cli::analysis::indicators;

// 移动平均线
let ma5 = indicators::sma(&data, 5);
let ema20 = indicators::ema(&data, 20);

// RSI
let rsi = indicators::rsi(&data, 14);

// MACD
let macd = indicators::macd(&data, 12, 26, 9);

// 布林带
let boll = indicators::bollinger_bands(&data, 20, 2.0);
```

#### 任务调度器 (`src/tasks/scheduler.rs`)

```rust
use quantix_cli::tasks::{TaskScheduler, TaskTemplates};

let scheduler = TaskScheduler::new().await?;

// 添加预设任务
scheduler.add_task(TaskTemplates::market_open()).await?;
scheduler.add_task(TaskTemplates::market_close()).await?;

// 启动调度器
scheduler.start().await?;
```

---

## 常见问题

### Q1: 如何连接到已有的 Python quantix 数据库？

确保环境变量正确配置，然后使用 `init` 命令：

```bash
export POSTGRES_URL="postgresql://localhost:5432/quantix"
quantix init
```

### Q2: ClickHouse 批量插入性能如何调整？

使用 `with_batch_size()` 方法调整批次大小：

```rust
let client = ClickHouseClient::new(...).await?
    .with_batch_size(5000); // 增大批次
```

### Q3: 如何添加自定义策略？

1. 在 `src/strategy/` 目录下创建策略文件
2. 实现 `Strategy` trait
3. 注册到策略列表

```rust
use quantix_cli::strategy::Strategy;

#[async_trait]
impl Strategy for MyStrategy {
    async fn on_bar(&mut self, bar: &Kline) -> Result<Signal> {
        // 策略逻辑
    }
}
```

### Q4: 支持哪些技术指标？

完整列表请参考 `src/analysis/indicators.rs`，包括：
- MA/EMA - 移动平均线
- RSI - 相对强弱指标
- MACD - 平滑异同移动平均线
- KDJ - 随机指标
- BOLL - 布林带
- ATR - 平均真实波幅

### Q5: 如何运行单元测试？

```bash
# 所有测试
cargo test

# 特定模块
cargo test sources::tdx_file

# 带输出
cargo test -- --nocapture
```

---

## 版本历史

- **v0.1.0** (2024-01) - 初始版本
  - Phase 1-5: 基础数据采集、竞价分析、K线管理、回测引擎、任务调度
  - Phase 6: TDX 文件解析与复权
  - Phase 7: GBBQ 数据存储
  - Phase 8: 多周期 K线查询
  - Phase 9: EastMoney 数据采集
  - Phase 10: ClickHouse 批量导入优化

---

## 许可证

MIT License

## 链接

- [GitHub 仓库](https://github.com/chengjon/quantix-rust)
- [Python quantix](https://github.com/chengjon/mystocks)
- [问题反馈](https://github.com/chengjon/quantix-rust/issues)
