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
  - [task - 任务调度](#task---任务调度)
  - [analyze - 分析工具](#analyze---分析工具)
  - [watchlist - 自选池](#watchlist---自选池)
  - [market - 市场分析](#market---市场分析)
  - [monitor - 实时监控](#monitor---实时监控)
  - [stop - 止盈止损](#stop---止盈止损)
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

# TDX 数据源
export TDX_HOST="192.168.1.100"
export TDX_PORT=7709

# 自选池 JSON 存储路径（可选）
export QUANTIX_WATCHLIST_PATH="$HOME/.quantix/watchlist/watchlist.json"

# 监控告警 / 止盈止损 SQLite 路径（可选）
export QUANTIX_MONITOR_DB_PATH="$HOME/.quantix/monitor/alerts.db"
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
| `paper` | 模拟盘模式 (开发中) |

##### 示例

```bash
# 运行均线交叉策略回测
quantix strategy run -n ma_cross

# 运行策略回测指定股票
quantix strategy run -n ma_cross -c 000001

# 使用实盘模式
quantix strategy run -n ma_cross --mode live
```

##### 输出

```
运行策略: ma_cross (backtest)
股票代码: 000001
```

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

提供 Phase 24A 的最小监控闭环：一次性自选池扫描加持久化价格告警管理。

#### 存储路径

- 默认路径：`~/.quantix/monitor/alerts.db`
- 可通过 `QUANTIX_MONITOR_DB_PATH` 覆盖
- 告警使用 SQLite 持久化，`watchlist --once` 命中时会在终端输出并更新最后触发时间

#### P0 范围

- 只支持 `watchlist --once`
- 只支持价格阈值告警的添加、列表、删除
- `--refresh`、`--repeat`、系统通知延后到后续 Phase

#### 命令摘要

```bash
quantix monitor watchlist --once
quantix monitor alert add <CODE> (--above <PRICE> | --below <PRICE>)
quantix monitor alert list
quantix monitor alert remove <ID>
```

#### 参数约束

- `watchlist` 当前必须显式带 `--once`
- `alert add` 必须且只能指定一个阈值：`--above` 或 `--below`
- 当前只复用现有自选池与 TDX 行情链路，不提供板块/概念监控

#### 常用示例

```bash
quantix monitor watchlist --once
quantix monitor alert add 000001 --above 16.0
quantix monitor alert add 000001 --below 15.0
quantix monitor alert list
quantix monitor alert remove 1
```

---

### stop - 止盈止损

提供 Phase 25A 的最小止盈止损闭环：为自选池代码维护单条规则，并在 `monitor watchlist --once` 里直接复用当前快照价格做评估。

#### 存储路径

- 默认路径：`~/.quantix/monitor/alerts.db`
- 复用 `QUANTIX_MONITOR_DB_PATH` 指向的 SQLite 路径
- 止盈止损规则与价格告警共用同一个 monitor DB

#### P0 范围

- 仅支持固定止损价、固定止盈价、跟踪止损百分比
- 仅允许为当前本地自选池中的代码设置规则
- 每个代码只保留一条有效规则，重复 `stop set` 会整条覆盖旧规则
- `quantix monitor watchlist --once` 会在输出监控快照后继续评估止盈止损规则
- `stop status`、`stop history`、`stop update`、`--loss-pct`、`--profit-pct` 延后到后续 Phase

#### 命令摘要

```bash
quantix stop set <CODE> [--loss <PRICE>] [--profit <PRICE>] [--trailing <PCT>]
quantix stop list
quantix stop remove <CODE>
```

#### 参数约束

- `stop set` 至少需要一个条件：`--loss`、`--profit`、`--trailing`
- `--loss` 与 `--trailing` 不能同时使用
- `--loss`、`--profit` 必须是有限正数
- `--trailing` 必须在 0 到 100 之间，且可以与 `--profit` 组合

#### 常用示例

```bash
quantix stop set 000001 --loss 14.5
quantix stop set 000001 --profit 18.0
quantix stop set 000001 --trailing 5 --profit 18.0
quantix stop list
quantix monitor watchlist --once
quantix stop remove 000001
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
