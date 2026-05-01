Quantix 的止盈止损模块（`src/stop/`）是一套基于规则的实时风控评估系统，为自选池中的股票提供固定价格止损/止盈、百分比阈值触发、以及跟踪止损三种模式。该模块以 **SQLite** 作为持久化后端，通过 `StopService` 业务层实现规则的 CRUD 与实时评估，并与 Monitor 自选池快照深度集成，在每次自选池刷新时自动执行止盈止损判定。

Sources: [mod.rs](src/stop/mod.rs#L1-L16), [models.rs](src/stop/models.rs#L1-L171)

## 模块架构总览

止盈止损模块遵循项目一贯的 **三层分离** 架构模式：**Models（数据模型）→ Service（业务逻辑）→ Storage（持久化）**。`StopService<RS>` 通过泛型参数接受任意实现了 `StopRuleStore` trait 的存储后端，使得业务逻辑与数据访问完全解耦，便于单元测试中使用内存 Fake Store 进行隔离验证。

```
┌──────────────────────────────────────────────────────────┐
│                     CLI / Monitor 层                      │
│  quantix stop set/update/list/status/history/remove      │
│  monitor watchlist → evaluate_stop_rules_for_snapshot    │
└──────────────┬──────────────────────────┬────────────────┘
               │                          │
               ▼                          ▼
┌──────────────────────────┐  ┌───────────────────────────────┐
│    StopService<RS>       │  │  MonitorQuoteRow / TradeStore  │
│  ┌────────────────────┐  │  │  (行情快照 + 持仓成本映射)      │
│  │ evaluate_rule()    │  │  └──────────┬────────────────────┘
│  │ evaluate_rules()   │  │             │
│  │ status_rows()      │◄─┼─────────────┘
│  │ set/update/remove  │  │
│  └────────┬───────────┘  │
└───────────┼──────────────┘
            │
            ▼
┌──────────────────────────┐
│   StopRuleStore trait    │
│  ┌────────────────────┐  │
│  │SqliteStopRuleStore │  │
│  │  stop_rules 表     │  │
│  │  stop_history 表   │  │
│  └────────────────────┘  │
└──────────────────────────┘
```

上述架构图展示了从 CLI/Monitor 调用入口到 SQLite 持久化层的完整数据流向。`StopService` 的核心职责是**规则的 CRUD 操作**与**实时评估逻辑**，评估过程需要外部注入行情数据（`MonitorQuoteRow`）和持仓成本映射（`avg_cost_by_code`），体现了**纯计算服务**的设计理念——服务本身不持有行情状态。

Sources: [service.rs](src/stop/service.rs#L1-L53), [storage.rs](src/stop/storage.rs#L1-L56)

## 核心数据模型

### StopRule — 规则实体

`StopRule` 是止盈止损模块的中心数据结构，每条规则以股票代码（`code`）作为唯一标识，存储于 SQLite 的 `stop_rules` 表中。一条规则可同时包含止损和止盈条件，通过不同的字段组合支持三种触发模式。

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | `String` | 股票代码，主键 |
| `stop_loss_price` | `Option<f64>` | 固定止损价格 |
| `take_profit_price` | `Option<f64>` | 固定止盈价格 |
| `stop_loss_pct` | `Option<f64>` | 止损百分比（相对于锚点价格） |
| `take_profit_pct` | `Option<f64>` | 止盈百分比（相对于锚点价格） |
| `trailing_pct` | `Option<f64>` | 跟踪止损回撤百分比 |
| `highest_price` | `Option<f64>` | 跟踪止损期间记录的最高价 |
| `reference_price` | `Option<f64>` | 百分比模式的参考锚点价格 |
| `last_triggered_at` | `Option<DateTime<Utc>>` | 最近一次触发时间 |
| `created_at` / `updated_at` | `DateTime<Utc>` | 创建/更新时间戳 |

Sources: [models.rs](src/stop/models.rs#L5-L18)

### 触发类型枚举

系统定义了三种互斥的触发类型 `StopTriggerKind`，对应不同的风控策略：

| 枚举值 | 含义 | 触发条件 |
|--------|------|----------|
| `Loss` | 固定止损 | 当前价 ≤ 止损价 或 止损百分比阈值 |
| `Profit` | 固定止盈 | 当前价 ≥ 止盈价 或 止盈百分比阈值 |
| `TrailingLoss` | 跟踪止损 | 当前价 ≤ 最高价 × (1 - trailing_pct/100) |

Sources: [models.rs](src/stop/models.rs#L20-L25)

### 评估状态枚举

`StopEvalState` 描述规则评估后的状态判定，用于 `stop status` 命令的状态展示：

| 枚举值 | 含义 |
|--------|------|
| `Armed` | 规则正常就绪，未触发 |
| `Triggered` | 已触发止损/止盈条件 |
| `AnchorMissing` | 百分比规则缺少锚点价格，无法计算阈值 |
| `QuoteMissing` | 无行情数据，无法评估 |

Sources: [models.rs](src/stop/models.rs#L149-L156)

### 锚点价格来源

百分比模式需要一个基准价格（锚点）来计算实际触发阈值。`StopAnchorSource` 枚举标识了锚点的来源：

| 枚举值 | 含义 | 优先级 |
|--------|------|--------|
| `PositionCost` | 模拟交易持仓的均价 | 高（优先使用） |
| `ReferencePrice` | 规则中手动设定的参考价 | 低（Fallback） |

Sources: [models.rs](src/stop/models.rs#L133-L147)

## 规则 CRUD 与输入验证

### 创建规则（set）

`StopService::set_rule()` 方法创建一条新的止盈止损规则。创建过程包含两步：**输入验证** 和 **持久化 + 历史记录**。输入验证由 `validate_stop_rule_inputs()` 函数执行，确保规则的语义正确性。

**验证规则矩阵：**

| 约束条件 | 错误消息 |
|----------|----------|
| 至少指定一个阈值条件 | "stop set 至少需要一个条件" |
| `--loss` 与 `--loss-pct` 互斥 | "不能同时指定 --loss 和 --loss-pct" |
| `--profit` 与 `--profit-pct` 互斥 | "不能同时指定 --profit 和 --profit-pct" |
| `--trailing` 与 `--loss`/`--loss-pct` 互斥 | "不能同时指定 --trailing 和 --loss/--loss-pct" |
| 所有价格/百分比必须为有限正数 | "{flag} 必须是有限正数" |
| `--trailing` 必须在 (0, 100) 开区间内 | "--trailing 必须在 0 到 100 之间" |

Sources: [service.rs](src/stop/service.rs#L52-L91), [service.rs](src/stop/service.rs#L392-L436)

### 更新规则（update）

`StopService::update_rule()` 使用 `StopRuleUpdate` 结构实现**部分更新（patch）**语义。`StopRuleUpdate` 的每个字段类型为 `Option<Option<f64>>`：外层 `None` 表示不修改该字段，`Some(None)` 表示清除该字段，`Some(Some(v))` 表示设置新值。更新完成后会对合并后的规则重新执行输入验证，确保修改后的规则仍然满足"至少一个条件"的约束。

Sources: [service.rs](src/stop/service.rs#L118-L139), [models.rs](src/stop/models.rs#L123-L131)

### 删除规则（remove）

删除操作会先查询现有规则，执行数据库删除，最后将删除事件追加到历史记录。如果规则不存在则返回 `false`，不产生历史事件。

Sources: [service.rs](src/stop/service.rs#L141-L156)

## 实时评估引擎

评估引擎是止盈止损模块的核心算法，由 `evaluate_rule_state()` 纯函数实现。该函数接收规则、当前价格、持仓成本和观测时间戳四个参数，返回完整的 `EvaluatedRuleState` 结构，包含更新后的规则、触发结果和状态信息。

### 评估流程

```mermaid
flowchart TD
    A[输入: StopRule + current_price + position_cost] --> B{解析锚点价格}
    B -->|position_cost 存在| C[锚点 = PositionCost]
    B -->|reference_price 存在| D[锚点 = ReferencePrice]
    B -->|两者均无| E[锚点 = None]
    
    C --> F{current_price 是否存在?}
    D --> F
    E --> F
    F -->|None| G[状态: QuoteMissing]
    F -->|Some| H{是否有跟踪止损?}
    
    H -->|trailing_pct 存在| I[更新 highest_price]
    I --> J[计算跟踪阈值 = highest × (1-pct/100)]
    J --> K{当前价 ≤ 跟踪阈值?}
    K -->|是| L[触发 TrailingLoss]
    K -->|否| M[未触发]
    
    H -->|无跟踪| N{有止损条件?}
    N -->|stop_loss_price| O[止损阈值 = 固定价格]
    N -->|stop_loss_pct + 锚点| P[止损阈值 = 锚点 × (1-pct/100)]
    
    O --> Q{当前价 ≤ 止损阈值?}
    P --> Q
    Q -->|是| R[触发 Loss]
    Q -->|否| S{有止盈条件?}
    
    S -->|take_profit_price| T[止盈阈值 = 固定价格]
    S -->|take_profit_pct + 锚点| U[止盈阈值 = 锚点 × (1+pct/100)]
    
    T --> V{当前价 ≥ 止盈阈值?}
    U --> V
    V -->|是| W[触发 Profit]
    V -->|否| X[状态: Armed]
```

### 优先级与触发逻辑

评估引擎按严格的优先级顺序检查触发条件：**跟踪止损 → 固定/百分比止损 → 固定/百分比止盈**。这意味着一条同时配置了跟踪止损和止盈的规则，在跟踪止损先满足时不会检查止盈条件。

**阈值计算规则：**

| 模式 | 止损阈值 | 止盈阈值 |
|------|----------|----------|
| 固定价格 | `stop_loss_price` | `take_profit_price` |
| 百分比 | `anchor × (1 - stop_loss_pct/100)` | `anchor × (1 + take_profit_pct/100)` |
| 跟踪止损 | `highest_price × (1 - trailing_pct/100)` | 不适用 |

Sources: [service.rs](src/stop/service.rs#L255-L377)

### 跟踪止损的 highest_price 更新机制

跟踪止损模式在每次评估时会自动更新 `highest_price` 字段。如果规则中已有历史最高价，则取 `max(existing_highest, current_price)` 作为新的最高价。这确保了跟踪止损阈值会随着股价上涨而**上移**，实现利润锁定效果。当 `highest_price` 发生变化时，`updated_at` 时间戳也会同步更新，以便持久化时识别规则已被修改。

Sources: [service.rs](src/stop/service.rs#L277-L284)

### 锚点解析策略

`resolve_anchor()` 函数实现了两级锚点解析策略：优先使用实时持仓成本（`position_cost`），其次使用规则中存储的参考价（`reference_price`）。在 CLI 层的 `resolve_stop_reference_price()` 函数中，参考价的获取也遵循类似的 Fallback 逻辑——先尝试从实时行情获取最新价，再从交易持仓的均价中获取。这确保了百分比规则在设定时能够自动获取合理的锚点价格。

Sources: [service.rs](src/stop/service.rs#L379-L390), [mod.rs](src/cli/handlers/mod.rs#L3219-L3246)

## 批量评估与 Monitor 集成

### evaluate_rules 批量评估

`StopService::evaluate_rules()` 方法将规则列表与行情数据进行批量匹配评估。它先将 `MonitorQuoteRow` 列表转换为以股票代码为键的 HashMap，然后对每条规则查找对应的最新价格进行独立评估。`evaluate_rules_with_anchor_map()` 变体额外接受持仓成本映射 `avg_cost_by_code`，用于百分比规则的锚点解析。

Sources: [service.rs](src/stop/service.rs#L171-L218)

### status_rows 状态面板

`status_rows()` 方法生成用于 `stop status` 命令的 `StopStatusRow` 列表。每个 `StopStatusRow` 包含了规则的完整评估快照：最新价格、锚点价格及来源、止损/止盈阈值、跟踪百分比、历史最高价、最近触发时间和评估状态。这使得用户无需手动计算即可一目了然地查看每条规则的实时状态。

Sources: [service.rs](src/stop/service.rs#L220-L252), [models.rs](src/stop/models.rs#L158-L170)

### Monitor Watchlist 集成

止盈止损模块与 Monitor 自选池系统通过 `evaluate_stop_rules_for_snapshot()` 函数深度集成。当用户执行 `monitor watchlist` 命令时，系统会在加载自选池快照后自动执行以下流程：

1. 从 `StopRuleStore` 加载所有止盈止损规则
2. 从交易存储构建持仓成本映射（`build_avg_cost_map_from_trade_store`）
3. 调用 `evaluate_rules_with_anchor_map()` 进行批量评估
4. 对于 `updated_rule` 与原始规则不同的条目，执行 `upsert_rule` 持久化更新（如跟踪止损的最高价变化）
5. 对于触发的规则，构建 `StopHistoryEvent::Trigger` 事件并追加到历史记录
6. 返回 `Vec<TriggeredStop>` 列表，在自选池输出中展示

Sources: [mod.rs](src/cli/handlers/mod.rs#L2693-L2749)

## 历史审计机制

每条规则的 CRUD 操作（Set、Update、Remove）和触发事件（Trigger）都会产生一条 `StopHistoryEvent` 记录，存储于 `stop_history` 表中。该机制提供了完整的规则变更审计追踪。

| 字段 | 说明 |
|------|------|
| `id` | UUID 唯一标识 |
| `code` | 股票代码 |
| `event_type` | 事件类型：Set / Update / Remove / Trigger |
| `trigger_kind` | 触发类型（仅 Trigger 事件）：Loss / Profit / Trailing |
| `trigger_price` | 触发时的实际价格 |
| `anchor_price` / `anchor_source` | 触发时的锚点价格及来源 |
| `snapshot_json` | 规则完整快照（JSON 序列化） |
| `created_at` | 事件时间戳 |

`stop_history` 表建立了 `(code, created_at)` 和 `(event_type, created_at)` 两个复合索引，支持按股票代码、日期和事件类型的高效查询。CLI 的 `stop history` 命令通过 `StopHistoryFilter` 结构支持多维度过滤和条数限制。

Sources: [models.rs](src/stop/models.rs#L46-L121), [storage.rs](src/stop/storage.rs#L29-L46)

## SQLite 持久化层

### 数据库路径

止盈止损规则与 Monitor 告警共享同一个 SQLite 数据库文件，路径通过 `CliRuntime::monitor_db_path` 解析。默认路径为 `~/.quantix/monitor/alerts.db`，可通过 `QUANTIX_MONITOR_DB_PATH` 环境变量覆盖。

Sources: [runtime.rs](src/core/runtime.rs#L144-L157), [mod.rs](src/cli/handlers/mod.rs#L3397-L3400)

### Schema 设计

**`stop_rules` 表**存储当前生效的规则，以 `code` 为主键，使用 `INSERT ... ON CONFLICT DO UPDATE` 实现 Upsert 语义。`stop_loss_pct`、`take_profit_pct` 和 `reference_price` 三个字段通过 `ensure_stop_rule_schema_extensions()` 方法实现了向后兼容的 Schema 迁移——当检测到旧版表缺少这些列时自动执行 `ALTER TABLE ADD COLUMN`。

**`stop_history` 表**追加写入规则变更和触发事件，支持通过 `QueryBuilder` 动态构建的 WHERE 条件进行灵活查询。

Sources: [storage.rs](src/stop/storage.rs#L13-L116), [storage.rs](src/stop/storage.rs#L296-L387)

### 连接池配置

`SqliteStopRuleStore` 使用 `max_connections(1)` 的连接池配置，这符合 SQLite 单写者模型的并发约束。数据库文件在首次访问时自动创建，父目录也会按需递归创建。

Sources: [storage.rs](src/stop/storage.rs#L58-L77)

## CLI 命令体系

止盈止损通过 `quantix stop` 子命令暴露所有操作。命令定义位于 `StopCommands` 枚举中，使用 Clap 的 `ArgGroup` 确保互斥参数的编译时验证。

| 子命令 | 功能 | 关键参数 |
|--------|------|----------|
| `stop set <code>` | 创建规则 | `--loss`, `--profit`, `--loss-pct`, `--profit-pct`, `--trailing` |
| `stop update <code>` | 更新规则 | 同上 + `--clear-loss`, `--clear-profit` 等清除标志 |
| `stop list` | 列出所有规则 | 无 |
| `stop status` | 查看实时评估状态 | `--code` 过滤 |
| `stop history` | 查看变更历史 | `--code`, `--date`, `--type`, `--limit` |
| `stop remove <code>` | 删除规则 | 无 |

所有 `set` 和 `update` 操作都会预先验证目标股票是否存在于自选池中（`ensure_watchlist_contains_code`），确保只为已关注的股票设定止盈止损规则。百分比规则在创建时会自动从实时行情或持仓均价中解析 `reference_price`，无需手动指定。

Sources: [monitor.rs](src/cli/commands/monitor.rs#L164-L295), [mod.rs](src/cli/handlers/mod.rs#L2454-L2887)

## 设计决策总结

| 设计点 | 决策 | 原因 |
|--------|------|------|
| `StopService<RS>` 泛型 | 便于测试注入 FakeStore | 业务逻辑与持久化解耦 |
| `Option<Option<f64>>` patch 语义 | 区分"不修改"和"清除" | 支持部分更新 |
| 跟踪止损优先级最高 | 先检查 trailing 再检查 fixed/percent | 避免跟踪止损与固定止损冲突 |
| 锚点使用 PositionCost 优先 | 实时持仓均价更准确 | 反映真实持仓成本 |
| SQLite 单连接池 | 符合 SQLite 并发模型 | 避免写锁冲突 |
| Schema 迁移用 PRAGMA 检测 | 向后兼容旧版数据库 | 无需破坏性迁移 |

---

**相关阅读：**
- 了解止盈止损如何与模拟交易持仓联动，参阅 [模拟交易、费用计算与交易报告](17-mo-ni-jiao-yi-fei-yong-ji-suan-yu-jiao-yi-bao-gao)
- 了解自选池行情数据如何驱动评估，参阅 [自选池管理：分组、标签与多源行情解析](21-zi-xuan-chi-guan-li-fen-zu-biao-qian-yu-duo-yuan-xing-qing-jie-xi)
- 了解 Monitor 守护进程如何自动触发止盈止损评估，参阅 [Monitor 服务：价格告警、事件存储与 systemd 集成](26-monitor-fu-wu-jie-ge-gao-jing-shi-jian-cun-chu-yu-systemd-ji-cheng)
- 了解全局风控框架的设计理念，参阅 [风控服务：规则引擎、行业集中度与波动率检查](16-feng-kong-fu-wu-gui-ze-yin-qing-xing-ye-ji-zhong-du-yu-bo-dong-lu-jian-cha)