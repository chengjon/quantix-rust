# tdx-api Bridge 集成 — 完成总结与下一步计划

> 日期：2026-06-06
> 状态：已按审核反馈修订

---

## 一、已完成工作

### P0: E2E 验证（基线打通）

- `TdxApiClient` REST 客户端；live 验证使用 `http://192.168.123.104:8089`，默认运行时通过 `TDX_API_URL` 配置（Docker 默认 `http://tdx-api:8080`）
- 修复 health/search/kline 三个 endpoint 的兼容问题
  - `health()` 绕过 Go envelope 包装（直接返回 `{status, time}`）
  - `search()` 对中文关键词做 URL 编码
  - Go 端 `null` → Rust 空 `Vec` 反序列化（`deserialize_null_default` helper）
- 8 个 API 全部验证通过

### P1a: 交易日历同步

- `quantix data tdx-api sync-calendar [--year N]` 从 tdx-api 按季度获取交易日
- 写入 `config/holidays.json`，供 `TradingCalendar` 消费
- `TradingCalendar::sync_trading_days(year, Vec<NaiveDate>)` 核心方法无 sources 依赖

### P1b: CollectScheduler 备选数据源

- `collect_once()` 中 tdx-api 作为行情 fallback
- 当主要数据源不可用时自动降级到 tdx-api

### P2: THS 前复权 K 线导入 ClickHouse

- `import-klines --code 600519 --type day` 单股导入
- `insert_kline_data_batch_with_source()` 支持 THS_QFQ/TDX source 参数
- `get_latest_kline_date()` 增量检查（查询 ClickHouse 最新日期）

### P2b: 批量增量导入

- `import-klines --all [--exchange sh]` 全 A 股批量导入
- 100ms 限流保护 tdx-api 服务
- 客户端增量过滤：获取全量历史后，只插入最新日期之后的数据
- `--force` 参数跳过增量检查，覆盖导入

### P2c: 异步任务管理

- `pull-kline` — 创建 K 线拉取异步任务
- `pull-trade` — 创建成交拉取异步任务
- `cancel-task` — 取消异步任务
- 通过 `task-info --id <ID>` 跟踪进度

### P3a: 逐笔成交导入 TDengine

- `import-ticks --code 600000 --date 20260601`
- TDengine tick_data STABLE 超级表，子表按股票代码自动创建
- ISO 时间戳解析（支持有/无毫秒两种格式）
- 批量插入（每 5000 条一批）

### P3b: Docker Compose + 定时脚本

- `docker-compose.yml` 新增 tdx-api 服务定义
- quantix 服务 `TDX_API_URL=http://tdx-api:8080` 环境变量
- `scripts/daily-update.sh` — sync-calendar + import-klines --all
- 建议定时任务：`0 18 * * 1-5 /opt/claude/quantix-rust/scripts/daily-update.sh --all`

### P3c: 文档更新

- README Phase 31 章节（18 个子命令 + 使用说明）
- FUNCTION_TREE: sources/、quantix data、db、CLI tree、依赖表、更新日志
- CHANGELOG 2026-06-06 条目（完整功能列表）

### 额外修复

- init_market_cli 测试：`rg` → `grep`（环境兼容性），3 个测试恢复通过
- CLAUDE.md 技术债条目：4 项标记为已解决（handlers 拆分、cli/mod.rs 重构、TODO 清零；`src/sources/tdx_api.rs` 生产路径 `unwrap` 已清理，仓库其他 `unwrap` 需以独立 debt gate 继续跟踪）
- CLI 解析测试 + clickhouse::Row derive 修复

---

## 二、代码量统计

| 文件 | 当前文件总行数 | 角色 |
|------|------:|------|
| `src/sources/tdx_api.rs` | 1309 | 核心桥接模块 |
| `src/cli/handlers/tdx_api_handler.rs` | 473 | 18 个子命令处理 |
| `src/cli/commands/data.rs` | 323 | 命令枚举定义（TdxApiCommands） |
| `src/db/clickhouse/kline.rs` | 282 | source 参数 + 增量查询 |
| `src/db/tdengine.rs` | 203 | tick 表 + 批量插入 + REST SQL |
| `src/core/trading_calendar.rs` | 616 | sync_trading_days() |
| `src/tasks/collect_scheduler.rs` | 369 | tdx_api_fallback 字段 |
| `docker-compose.yml` | 239 | tdx-api 服务定义 |
| `scripts/daily-update.sh` | 22 | 定时同步脚本 |

---

## 三、提交记录（21 个 commit，`6b4b285^..HEAD`）

```
8c34744 fix: replace rg with grep, update tech debt
2d0a478 docs: record rest review handoff backfill
978bc1d docs: add tdx-api REST source design review spec
d454685 docs: update README, FUNCTION_TREE, CHANGELOG
d0c0da9 docs: record graphiti backfill location convention
ade14e5 docs: record graphiti backfill for import-klines closure
67361f2 fix: repair tdx-api import ticks compile path
9cac8a7 test: add CLI parsing tests, fix Row derive
4fb91f0 fix: parse tick timestamps from ISO string
f97c002 feat: import-ticks → TDengine
e9c7ad5 feat: Docker Compose + daily update script
83f5326 fix: make import-klines --all truly incremental
9264b4d feat: pull-trade, pull-kline, cancel-task
fe290f8 feat: batch import-klines --all + incremental
2b0cb34 feat: import-klines single stock → ClickHouse
da256cf feat: tdx-api HTTP bridge integration (P0-P1)
cdf9161 feat: CollectScheduler fallback
39d04ad feat: sync-calendar
faac9a1 fix: P0 E2E verification fixes
e2bb9d2 docs: design review
6b4b285 docs: initial design
```

---

## 四、下一步计划

### 近期（可立即推进）

| 优先级 | 任务 | 说明 |
|--------|------|------|
| **P0** | E2E 测试 `import-klines --all` | 需要运行中的 ClickHouse + tdx-api；验证全量/增量导入正确性 |
| **P0** | E2E 测试 `import-ticks` | 需要运行中的 TDengine + tdx-api；验证逐笔数据完整导入 |
| **P1** | 分阶段清理 clippy warnings | `cargo clippy --lib -p quantix-cli --message-format short` 当前 exit 0；lib summary 为 110 warnings；已清理 `src/sources/tdx_api.rs`、`src/sources/eastmoney.rs`、`src/core/trading_calendar.rs` 低风险项和部分 CLI handler unused imports |
| **P1** | `println!` → `tracing` | `monitoring/`、`anomaly/` 库模块中的 println 替换为 tracing 宏 |

### 中期（功能增强）

| 任务 | 说明 |
|------|------|
| import-klines 进度条 | `--all` 模式下使用 `indicatif` 显示进度百分比和预估时间 |
| import-ticks 批量模式 | `--all` + 日期范围，自动遍历多股票多日 |
| K 线数据质量校验 | 导入后自动检查 OHLCV 一致性、缺失日期、异常价格 |
| 定时任务 systemd 集成 | `daily-update.sh` → systemd timer unit，替代 crontab |
| 定时监控告警 | import 失败时通过 `quantix notify send` 渠道告警 |

### 远期（架构方向）

| 任务 | 说明 |
|------|------|
| 数据源统一抽象 | `DataSource` trait 统一 tdx-api / eastmoney / bridge_tdx 接口 |
| 流式 K 线 WebSocket | tdx-api WebSocket 推送替代轮询，实时入库 |
| 因子计算管线 | 基于已导入的 THS 前复权数据跑 `factor compute` pipeline |
| 回测数据自动准备 | 策略回测前自动检查/补全所需 K 线数据 |

---

## 五、当前测试状态

| 测试套件 | 结果 |
|----------|------|
| `cargo test --lib --quiet` | 695 passed；exit 0 |
| `cargo test -p quantix-cli --quiet` | 1302 passed，6 ignored；exit 0 |
| `repo_hygiene_test main_workspace_status_bearing_docs_defer_to_function_tree_registry` | 1 passed；exit 0 |
| `cargo build --release --quiet` | tdx-api cleanup 前通过；cleanup 后重新验证未取得 exit code（MCP 120s 超时后 LTO 构建仍在运行，已停止后台进程）；不作为当前 cleanup 后通过证据；现有 `target/release/quantix` 为 71M |
| `cargo clippy --lib -p quantix-cli --message-format short` | exit 0；lib summary 110 warnings；`src/sources/tdx_api.rs`、`src/sources/eastmoney.rs` 无 warning |
| `cargo clippy --all-targets --all-features --message-format short` | exit 0；all-targets warning backlog 仍存在；`src/sources/tdx_api.rs` 无 warning |
