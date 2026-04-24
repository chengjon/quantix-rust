# Quantix-Rust 手工测试结果模板

用于记录本轮 CLI 手工测试结果。建议配合以下文档一起使用：

- `docs/MANUAL_TESTING_GUIDE.md`
- `docs/CLI_COMMAND_MANUAL.html`

---

## 基本信息

测试日期:

测试二进制: `./target/debug/quantix`

测试环境:

测试人:

备注:

---

## 一、总览检查

- `cargo build`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix strategy --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix data source --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix backtest --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix performance --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix execution qmt --help`:
  - 返回码:
  - 输出摘要:
  - 判定:

---

## 二、通过

- 命令:
  - 返回码:
  - 输出摘要:
  - 说明:

- 命令:
  - 返回码:
  - 输出摘要:
  - 说明:

- 命令:
  - 返回码:
  - 输出摘要:
  - 说明:

---

## 三、外部依赖失败

- 命令:
  - 依赖项:
  - 返回码:
  - 失败摘要:
  - 是否可复现:

- 命令:
  - 依赖项:
  - 返回码:
  - 失败摘要:
  - 是否可复现:

---

## 四、CLI 缺陷

- 命令:
  - 问题类型: 参数解析 / 输出异常 / 状态未落盘 / 兼容路径失效 / 其他
  - 返回码:
  - 现象:
  - 复现步骤:

- 命令:
  - 问题类型: 参数解析 / 输出异常 / 状态未落盘 / 兼容路径失效 / 其他
  - 返回码:
  - 现象:
  - 复现步骤:

---

## 五、重点结论

- `data source`:

- `backtest`:

- `performance`:

- `strategy`:

- `execution qmt`:

- `compatibility`:

---

## 六、重点命令逐项记录

### 1. data source

- `./target/debug/quantix data source list`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix data source add --type akshare`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix data source set-default --name akshare`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix data source test --name akshare`
  - 返回码:
  - 输出摘要:
  - 判定:

### 2. backtest

- `./target/debug/quantix backtest run --code 600519 --start 20240101 --end 20241231 --short 5 --long 20`
  - 报告 ID:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix backtest list`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix backtest report --id <REPORT_ID>`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix backtest compare --id <REPORT_ID_A> --id <REPORT_ID_B>`
  - 返回码:
  - 输出摘要:
  - 判定:

### 3. performance

- `./target/debug/quantix performance list`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix performance report --id <REPORT_ID>`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix performance compare --id <REPORT_ID_A> --id <REPORT_ID_B>`
  - 返回码:
  - 输出摘要:
  - 判定:

### 4. strategy

- `./target/debug/quantix strategy create --id ma-demo --name ma_cross --code 600519 --param fast=5 --param slow=20`
  - 实例 ID:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix strategy show --id ma-demo`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix strategy update --id ma-demo --param fast=8 --param slow=21 --enable`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix strategy delete --id ma-demo`
  - 返回码:
  - 输出摘要:
  - 判定:

### 5. execution qmt

- `./target/debug/quantix execution qmt status`
  - 返回码:

---

## 七、市场分析专项验收

建议优先执行：

- `source scripts/dev/market_cli_env.example.sh`
- `cp .env.market.local.example .env.market.local`
- `scripts/dev/init_market_cli_local_env.sh`
- `scripts/dev/run_market_cli_acceptance.sh`
- `scripts/dev/run_market_cli_formal_sequence.sh`
- `scripts/dev/generate_market_cli_acceptance_report.sh`
- `scripts/dev/check_market_cli_prereqs.sh`
- `./target/debug/quantix risk sync industry --standard shenwan`
- `./target/debug/quantix market foundation`
- `./target/debug/quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10`
- `scripts/dev/verify_market_cli_smoke.sh`

### 0. 环境前置检查

- `scripts/dev/run_market_cli_acceptance.sh`
  - 返回码:
  - 输出摘要:
  - precheck 结论:
  - smoke 结论:
  - 下一步动作:
  - 判定:

- `scripts/dev/generate_market_cli_acceptance_report.sh`
  - 返回码:
  - 生成报告路径:
  - 归档摘要:
  - 判定:

- `scripts/dev/run_market_cli_formal_sequence.sh`
  - 返回码:
  - sync industry exit:
  - market foundation exit:
  - market strength exit:
  - 日志路径:
  - 判定:

- `scripts/dev/check_market_cli_prereqs.sh`
  - 返回码:
  - 输出摘要:
  - ClickHouse 配置:
  - MySQL 同步环境:
  - 行业 SQLite 状态:
  - 补救动作:
  - 判定:

### 1. 行业同步前置

- `./target/debug/quantix risk sync industry --standard shenwan`
  - 返回码:
  - 输出摘要:
  - 判定:
  - 备注:

### 2. 全市场基础摘要

- `./target/debug/quantix market foundation`
  - 返回码:
  - 输出摘要:
  - A 股总数:
  - 行业覆盖摘要:
  - 判定:

### 3. 强弱板块分析

- `./target/debug/quantix market strength --date 2026-03-09 --strong-top 3 --weak-top 3 --stock-top 10`
  - 返回码:
  - 输出摘要:
  - 强势板块摘要:
  - 弱势板块摘要:
  - 强势板块个股 TopN:
  - 判定:

### 4. 专项脚本验收

- `scripts/dev/verify_market_cli_smoke.sh`
  - 返回码:
  - 输出摘要:
  - 本地检查结论:
  - 外部依赖检查结论:
  - 判定:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix execution qmt preview --request-id <REQUEST_ID>`
  - request ID:
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix execution qmt live --request-id <REQUEST_ID>`
  - request ID:
  - 是否真实下单:
  - 返回码:
  - 输出摘要:
  - 判定:

### 6. compatibility

- `./target/debug/quantix execution bridge qmt-preview --request-id <REQUEST_ID>`
  - request ID:
  - 返回码:
  - 输出摘要:
  - 与 `execution qmt preview` 是否一致:
  - 判定:

- `./target/debug/quantix analyze screener presets`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix analyze screener preset-list`
  - 返回码:
  - 输出摘要:
  - 判定:

- `./target/debug/quantix analyze backtest --id <REPORT_ID>`
  - 返回码:
  - 输出摘要:
  - 与 `backtest report` 是否一致:
  - 判定:

---

## 七、最终结论

是否可进入下一轮 CLI 优化:

建议优先修复项:

阻塞项:

附加说明:
