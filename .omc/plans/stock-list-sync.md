# 股票列表同步功能方案

## 目标

实现 `quantix data sync-stock-list` 命令，从东方财富 API 获取 A 股完整股票列表，写入 ClickHouse `stock_info` 表。同时接入菜单的"数据同步"子菜单。

## 现有基础

| 组件 | 状态 | 说明 |
|------|------|------|
| `StockInfo` 模型 | ✅ 已有 | `data/models.rs` - code, name, market, list_date, delist_date |
| `stock_info` 表 DDL | ✅ 已有 | `db/clickhouse/schema.rs` - ReplacingMergeTree |
| EastMoney API | ⚠️ 半成品 | `sources/eastmoney.rs` - `get_stock_list()` 有URL但解析为空 |
| ClickHouse 读写 | ❌ 缺失 | 无 INSERT/SELECT 方法 |
| Fetcher trait | ❌ 缺失 | 无 `get_stock_list()` 批量方法 |
| CLI 命令 | ❌ 缺失 | 无 sync-stock-list 子命令 |
| 菜单接入 | ❌ 缺失 | 数据同步子菜单无股票列表项 |

## 实现方案（3个数据源，优先级从高到低）

### 数据源优先级

1. **东方财富 API**（首选，无需本地软件，纯HTTP）
2. **Bridge TDX**（备选，需 Bridge 服务运行）
3. **TDX 本地文件**（补充，读取 vipdoc 下的文件列表）

### 步骤

#### Step 1: 完善 EastMoney 股票列表获取
- 文件: `src/sources/eastmoney.rs`
- 实现 `parse_stock_list()` 解析 JSON 响应
- 字段映射: 东方财富响应 → `StockInfo` 结构

#### Step 2: 添加 ClickHouse stock_info 读写方法
- 文件: `src/db/clickhouse.rs` (或适当模块)
- `insert_stock_list(batch: &[StockInfo])` - 批量 INSERT
- `query_stock_list(market: Option<Market>)` - 查询已同步列表
- `count_stock_list()` - 统计数量

#### Step 3: Fetcher trait 扩展
- 文件: `src/data/fetcher.rs`
- 添加 `get_stock_list() -> Result<Vec<StockInfo>>`

#### Step 4: CLI 命令注册
- 文件: `src/cli/commands/data.rs`
- 添加 `SyncStockList` 子命令到 `DataCommands`
- 文件: `src/cli/handlers/data_handler.rs`
- 实现 `sync_stock_list()` handler: fetch → dedup → insert → report

#### Step 5: 菜单接入
- 文件: `src/cli/handlers/app_shell.rs`
- 在数据同步子菜单添加"同步股票列表"选项
- 执行前提示: "将从东方财富API获取A股列表，写入ClickHouse"
- 执行后显示: 同步数量、新增/更新数

## 预计改动文件

| 文件 | 改动类型 |
|------|----------|
| `src/sources/eastmoney.rs` | 完善 parse_stock_list |
| `src/db/clickhouse.rs` | 新增 stock_info 读写方法 |
| `src/data/fetcher.rs` | trait 新增 get_stock_list |
| `src/cli/commands/data.rs` | 新增子命令 |
| `src/cli/handlers/data_handler.rs` | 新增 handler |
| `src/cli/handlers/app_shell.rs` | 菜单接入 |

## 前置条件

- ClickHouse 运行中且 `stock_info` 表已创建
- 网络可访问东方财富 API（`push2.eastmoney.com`）
- `.env` 中 ClickHouse 连接配置正确
