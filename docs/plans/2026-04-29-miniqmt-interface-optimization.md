# miniQMT 接口与相关代码优化方案

## 1. 文档目标

本文档用于把 `/mnt/d/MyCode3/miniQMT/DOCS/项目说明/miniQMT-Windows-qmt-agent-live-contract-开发文档.md` 中定义的 Windows qmt agent live contract，与 `quantix-rust` 当前已经落地的 `QMT preview + guarded qmt_live` 能力做一次对齐，给出本仓库侧可执行的接口与代码优化方案。

本文档关注的是 `quantix-rust` 这一侧应该如何收敛接口、模型、错误语义、执行生命周期与测试，而不是重写 miniQMT 项目本身。

## 2. 输入事实与当前基线

### 2.1 外部 miniQMT 文档要求

外部文档已经把目标 contract 明确为一条 `contract-first` 的 live 通道，核心约束如下：

- Windows 侧提供固定 `POST /api/v1/task/execute`
- Windows 侧提供固定 `GET /api/v1/task/result/{task_id}`
- 固定 `provider=qmt`、`method=submit_order`
- 认证头要求 `Authorization: Bearer <token>`
- 协议头要求 `X-Bridge-Contract-Version`
- `execute` 只表示 transport receipt，不等于 broker acknowledgement
- `result` 必须显式区分 bridge failure 和 broker-facing result
- 结果必须优先依赖 `client_order_id` / `local_submission_id` 做 identity echo
- 必须有稳定 failure code，例如 `live_bridge_timeout`、`live_bridge_auth_failed`

### 2.2 quantix-rust 当前状态

结合仓库现有文档、代码与测试，当前基线是：

- 推荐业务入口是 `quantix execution qmt`
- 兼容入口是 `quantix execution bridge`
- `qmt-preview` 仍然保留为 preview 路径
- `qmt-live` 已在能力门控下支持真实提交
- 真实提交前会校验 `qmt.enabled=true`、`qmt.mode=live`、`qmt.supports` 包含 `order_submit`

当前主要实现落点：

- `src/core/runtime.rs`
  - 仅支持 `QUANTIX_BRIDGE_BASE_URL` 与 `QUANTIX_BRIDGE_API_KEY`
- `src/bridge/client.rs`
  - 直接请求 `/api/v1/broker/qmt/orders*`、`/api/v1/capabilities`
  - 仍使用 `X-Quantix-Api-Key`
- `src/bridge/models.rs`
  - 只建模了 preview、live order、query、cancel、account 等 broker 风格接口
- `src/execution/qmt_bridge.rs`
  - `QmtBridgePreviewAdapter` 从 frozen request 生成 preview 请求
- `src/execution/qmt_live_adapter.rs`
  - `QmtLiveExecutionAdapter` 直接对 `/api/v1/broker/qmt/orders` 发起真实下单
- `src/cli/handlers/execution_handler.rs`
  - 手工 `qmt-live` 提交流程会在收到提交响应后直接把 `execution_request` 写成 `completed` 或 `failed`

### 2.3 结论

当前仓库已经具备可工作的 QMT live 提交骨架，但它的接口形态仍然偏“直接 broker API 调用”，与 miniQMT 文档要求的“task receipt/result contract”存在明显差距。

这意味着后续如果不先收敛 contract，`quantix-rust` 会在以下方面持续承受语义漂移：

- 提交成功到底表示“agent 已接单”还是“broker 已受理”
- CLI 与 runtime store 到底保存 `order_id` 还是 `task_id`
- bridge 错误与 broker reject 是否能稳定区分
- `query/cancel` 是跟踪 broker order，还是跟踪 Windows task

## 3. 当前差距矩阵

| 维度 | 当前实现 | miniQMT 目标 | 主要缺口 |
|------|----------|---------------|----------|
| 认证 | `X-Quantix-Api-Key` | `Authorization: Bearer` | 头部不兼容 |
| 协议协商 | 无 contract version | `X-Bridge-Contract-Version` | 无版本门控 |
| 提交入口 | `/api/v1/broker/qmt/orders` | `/api/v1/task/execute` | 接口风格不同 |
| 查询入口 | `/api/v1/broker/qmt/orders/{id}` | `/api/v1/task/result/{task_id}` | 查询对象不同 |
| 提交语义 | 返回即当作订单提交结果 | `execute` 只代表 receipt | receipt/result 未分层 |
| 身份绑定 | `client_order_id`、`adapter_order_id` 为主 | `client_order_id` / `local_submission_id` 必须稳定回显 | `local_submission_id` 缺失 |
| 失败语义 | 文本错误为主 | 稳定 `reason_code` / failure code | 错误不可枚举 |
| 生命周期 | 手工 live 提交后立即写回 request 终态 | 先接单，再轮询 broker-facing result | 当前写回过早 |
| 审计证据 | payload_json + 日志 | task/evidence 可追踪 | 缺 task/evidence 锚点 |
| 配置 | `BASE_URL`、`API_KEY` | token + contract version + timeout/poll | 配置项不足 |

## 4. 方案比较

### 方案 A：继续维持 broker 风格接口，只补头部和错误码

做法：

- 保留 `/api/v1/broker/qmt/orders*`
- 仅在 `BridgeHttpClient` 增加 Bearer 与 version header
- 在 Rust 侧把当前下单响应继续当作最终提交结果

优点：

- 改动小
- 兼容当前 `QmtLiveExecutionAdapter`

缺点：

- 无法对齐外部 miniQMT 文档的 receipt/result 语义
- 仍然把 transport acceptance 与 broker acknowledgement 混在一起
- 后续 identity echo 和 failure semantics 只能继续打补丁

### 方案 B：新增 task contract 作为规范层，保留 broker 风格接口作兼容内核

做法：

- 在 Rust 侧新增 `task/execute`、`task/result` 模型与客户端方法
- 保留现有 `/api/v1/broker/qmt/orders*` 作为过渡期兼容或 agent 内部实现
- `quantix-rust` 对外语义统一改为 task receipt/result
- 仅在拿到 broker-facing result 时才映射到订单终态

优点：

- 与外部 miniQMT 文档直接对齐
- 迁移风险明显小于整仓替换
- 可以渐进保留现有 CLI、live gate、query/cancel 能力

缺点：

- 需要补模型、错误枚举、持久化字段与测试
- `execution_request` 生命周期会有一次语义重整

### 方案 C：直接全面切换到 task contract，移除现有 broker 风格提交路径

做法：

- 删除或废弃 `/api/v1/broker/qmt/orders`
- `qmt_live` 全面依赖 task contract
- CLI、adapter、query/cancel 全量重写

优点：

- 目标状态最干净

缺点：

- 当前 blast radius 过大
- 对已有 `qmt_live`、手工 CLI、测试和文档破坏性太强

### 推荐结论

推荐 **方案 B**。

原因很直接：

- 它与 miniQMT 的目标 contract 一致
- 它不要求立即推翻 `quantix-rust` 当前的 guarded `qmt_live` 能力
- 它允许先把“提交语义正确化”，再逐步收敛 query/cancel/audit

## 5. 推荐目标形态

### 5.1 目标原则

- `contract first`：先收敛 HTTP contract 和状态语义，再谈实现细节
- `identity first`：任何 broker-facing 结果必须优先依赖 `client_order_id` / `local_submission_id`
- `no silent fallback`：miniQMT 失败不得静默切到其他通道
- `compatibility preserved`：保留 `execution bridge` 兼容入口，但业务主入口仍是 `execution qmt`

### 5.2 推荐 ID 语义

建议在 `quantix-rust` 中明确三层 ID：

- `client_order_id`
  - 继续使用业务侧可追踪 ID
  - 默认可沿用 `execution_request.request_id`
- `local_submission_id`
  - 每次实际向 Windows qmt agent 提交时生成一个新的 UUID
  - 用于区分同一 request 的多次尝试
- `task_id`
  - Windows qmt agent 返回的 receipt ID
  - 仅用于轮询 `task/result`

可选补充：

- `external_order_id`
  - 仅在 broker-facing result 已经可证实时出现

### 5.3 推荐提交语义

推荐把 `QMT live` 提交流程拆成两段：

1. `task/execute`
   - 返回 receipt
   - 最多只能证明“Windows agent 已接单并分配 task_id”
2. `task/result`
   - 返回 pending、bridge failure、broker-facing result 三类之一

这意味着 `quantix-rust` 不应再把“收到 submit 响应”直接写成 `execution_request.completed`。

### 5.4 推荐结果语义

Rust 侧建议统一成以下三类结果：

- `TaskPending`
  - receipt 已有，但 broker-facing 证据尚未完成
- `BridgeFailure`
  - timeout、auth、unsupported_version、unsupported_method、invalid_result、unavailable
- `BrokerEvent`
  - acknowledgement、reject、cancel、execution

其中：

- `BridgeFailure` 不能映射成 `OrderStatus::Rejected`
- `BrokerEvent::acknowledgement` 才可以映射到 `Submitted` / `Accepted`
- `BrokerEvent::execution` 才允许带成交信息

## 6. quantix-rust 代码优化落点

### 6.1 `src/core/runtime.rs`

需要新增 bridge 运行时配置：

- `QUANTIX_BRIDGE_BEARER_TOKEN`
- `QUANTIX_BRIDGE_CONTRACT_VERSION`
- `QUANTIX_BRIDGE_TIMEOUT_MS`
- `QUANTIX_BRIDGE_POLL_INTERVAL_MS`

兼容策略：

- `QUANTIX_BRIDGE_API_KEY` 暂时保留为 fallback
- 但新文档、测试与主路径应优先使用 Bearer token

建议把 `BridgeRuntimeSettings` 扩成：

- `base_url`
- `bearer_token: Option<String>`
- `api_key_fallback: Option<String>`
- `contract_version: String`
- `timeout_ms: u64`
- `poll_interval_ms: u64`

### 6.2 `src/bridge/error.rs`

当前只有：

- `Config`
- `Http`

这不足以支撑 task contract。

建议扩成可枚举错误：

- `Config`
- `Timeout`
- `Unavailable`
- `Unauthorized`
- `UnsupportedContractVersion`
- `UnsupportedMethod`
- `InvalidResult`
- `Protocol`
- `Http`

这样 CLI、adapter 和 execution daemon 才能稳定输出 `reason_code`。

### 6.3 `src/bridge/models.rs`

建议保留现有 preview/live broker models，同时新增 task contract models：

- `BridgeTaskExecuteRequest`
- `BridgeTaskExecuteReceipt`
- `BridgeTaskResultResponse`
- `BridgeTaskStatus`
- `BridgeFailureCode`
- `BridgeBrokerEventType`
- `BridgeIdentityEcho`
- `BridgeResultEvidenceRef`

其中 `BridgeTaskExecuteRequest` 应固定：

- `provider = "qmt"`
- `method = "submit_order"`

`params` 中建议包含：

- `request_id`
- `client_order_id`
- `local_submission_id`
- `symbol`
- `side`
- `quantity`
- `price`
- `order_type`
- `strategy_name`
- `order_remark`
- `snapshot_metadata`

### 6.4 `src/bridge/client.rs`

当前 `BridgeHttpClient` 偏“endpoint 拼装器”，后续应提升为“contract-aware client”。

建议新增：

- 自动注入 `Authorization: Bearer`
- 自动注入 `X-Bridge-Contract-Version`
- `task_execute_qmt_submit(...)`
- `task_result(...)`
- 统一 HTTP error -> `BridgeError` 映射
- 响应体结构校验

兼容要求：

- `capabilities()` 继续保留，用于 live preflight gate
- `qmt_preview_order()` 继续保留
- `qmt_query_order()` / `qmt_cancel_order()` / `qmt_account_status()` 可暂时保留为 broker 辅助接口

### 6.5 `src/execution/qmt_bridge.rs`

`QmtBridgePreviewAdapter` 不需要废弃，但建议把“task contract 提交服务”与 preview adapter 分开：

- 保留 `QmtBridgePreviewAdapter`
- 新增 `QmtTaskSubmitService` 或等价组件

职责分离：

- preview adapter 只做 frozen snapshot -> preview payload
- submit service 只负责 execute/result contract

### 6.6 `src/execution/qmt_live_adapter.rs`

这是本次语义改造的关键点。

当前实现的问题是：

- `submit_order()` 直接 POST `/api/v1/broker/qmt/orders`
- 收到响应后就把它解释为订单提交结果

推荐改造方向：

1. 真实提交前继续执行 `ensure_bridge_qmt_live_mode(...)`
2. 调用 `task_execute_qmt_submit(...)`
3. 得到 receipt 后先返回或记录：
   - `local_submission_id`
   - `task_id`
4. 在 `task/result` 已返回 broker-facing result 前，不把本次请求写成最终 broker 终态

与现有 `ExecutionAdapter` 的兼容建议：

- 如果适配器接口必须返回 `OrderInitialResponse`，则在 receipt 阶段统一返回：
  - `latest_status = PendingSubmit`
  - `adapter_order_id = local_submission_id`
- 真正的 `Submitted/Accepted/Rejected/Filled` 通过后续 result/query 流程写回

这比“把 receipt 当作 submitted”更安全，也更符合 miniQMT 文档。

### 6.7 `src/cli/handlers/execution_handler.rs`

当前手工 `execute_execution_bridge_qmt_live(...)` 需要调整：

- 现在的实现会在提交成功后直接 `try_complete_execution_request(...)`
- 新实现应改成：
  - `try_start_execution_request(...)`
  - 执行 `task/execute`
  - 把 `task_id`、`local_submission_id`、receipt 时间写入 request payload
  - 进入轮询窗口
  - 在窗口内拿到 broker-facing result 时再 `complete` 或 `fail`
  - 超时则写成“bridge pending / timeout evidence”，而不是误记为 broker reject

同时建议补充 CLI 输出：

- receipt 阶段打印 `task_id`
- result 阶段打印 `broker_event_type` 或 `bridge_failure_code`
- 查询提示优先指向 task 结果查询，而不是立即只给 `order_id`

### 6.8 `execution_request` / runtime store

当前 `execution_request.payload_json` 已经在保存：

- `execution_error`
- `execution_result`

建议先在 payload 中新增：

- `bridge_task`
  - `task_id`
  - `local_submission_id`
  - `submitted_at`
  - `receipt_status`
- `bridge_result`
  - `bridge_failure_code`
  - `broker_event_type`
  - `external_order_id`
  - `evidence_ref`

这样第一阶段无需立刻改数据库 schema，就能先完成 contract 对齐。

后续如果 live workflow 继续扩大，再考虑把这些字段结构化进 runtime store。

### 6.9 文档与帮助输出

后续代码落地后，需要同步更新：

- `README.md`
- `docs/USER_MANUAL.md`
- `docs/QMT_LIVE_TRADING_SETUP.md`
- `docs/architecture/WSL2_WINDOWS_BRIDGE_ARCHITECTURE.md`

重点是把以下语义写清楚：

- `qmt-preview` 仍是 preview
- `qmt-live` 现在先收到 receipt，再等待 result
- `execution qmt` 是推荐入口
- `execution bridge` 只是兼容入口

## 7. 推荐实施顺序

### Phase 1：配置与模型先对齐

目标：

- 补 Bearer token、contract version、poll config
- 在 `bridge/models.rs` 和 `bridge/error.rs` 建好 task contract 结构

完成标准：

- Rust 侧可以对 task contract 发出合法请求
- 不影响现有 preview path 和 capability gate

### Phase 2：引入 task execute/result 主链路

目标：

- `BridgeHttpClient` 支持 execute/result
- 手工 `qmt-live` 路径先切换到 task receipt/result

完成标准：

- CLI 能打印 `task_id`
- 超时、鉴权、unsupported version 能稳定区分

### Phase 3：改造 `QmtLiveExecutionAdapter` 与 request 生命周期

目标：

- `QmtLiveExecutionAdapter` 不再直接把 receipt 视为最终结果
- execution daemon / runtime store 保存 `local_submission_id` 与 `task_id`

完成标准：

- `execution_request` 在 receipt 阶段不被过早记为 completed
- broker-facing result 与 bridge failure 可以稳定拆分

### Phase 4：审计与兼容收口

目标：

- 增加 evidence 引用字段
- 收敛 query/cancel 与 order_id 展示逻辑
- 更新 README / USER_MANUAL / setup guide

完成标准：

- 手工操作、daemon 执行、文档描述三者一致

## 8. 测试矩阵建议

### 8.1 Rust 侧新增或改造测试

- `tests/qmt_task_contract_test.rs`
  - execute receipt 返回 `task_id`
  - result 返回 pending
  - result 返回 `live_bridge_timeout`
  - result 返回 broker acknowledgement
- `tests/qmt_live_adapter_test.rs`
  - receipt 阶段返回 `PendingSubmit`
  - preview_only gate 仍然拒绝
  - 缺少 `order_submit` 仍然拒绝
- `tests/qmt_live_gate_test.rs`
  - 保留现有能力门控测试
- `tests/qmt_bridge_preview_test.rs`
  - 保持 preview 路径不受 task contract 改造影响

### 8.2 最小契约验收

至少覆盖：

1. Bearer token 正确时 execute 成功，错误时返回 `auth_failed`
2. contract version 不匹配时返回 `unsupported_contract_version`
3. execute 成功只返回 receipt，不假装 broker success
4. result 可以区分 pending / bridge failure / broker event
5. result 必须稳定回显 `client_order_id` 或 `local_submission_id`
6. preview path 在 `preview_only` 下继续可用

## 9. 风险与待确认项

### 9.1 主要风险

- `ExecutionAdapter` 当前接口是“同步拿初始订单结果”思维，task receipt/result 会天然拉开时序
- Windows miniQMT 实际可返回的 identity 字段可能不够完整
- 现有 `qmt_query_order` / `qmt_cancel_order` 与新 `task/result` 的职责边界需要明确定义

### 9.2 待确认项

- `local_submission_id` 最终由 Rust 生成还是允许 Windows 侧回填覆盖
- `task/result` 超时后，Rust 是保留 processing 还是立即写 fail
- broker-facing result 中 `acknowledgement` 应映射到 `Submitted` 还是 `Accepted`
- Windows 侧是否会同时保留 `/api/v1/broker/qmt/orders*` 作为兼容接口

## 10. 最终建议

本仓库下一步不应该继续在现有 `/api/v1/broker/qmt/orders*` 上零散补丁，而应以 **“保留现有能力门控和 preview 路径，新增 task contract 规范层”** 作为 miniQMT 对接的主方向。

具体来说：

- 对外语义改成 `task/execute + task/result`
- 对内短期保留当前 broker 风格接口和能力门控
- 先改“提交语义”，再改“订单跟踪语义”
- 先把 `task_id`、`local_submission_id`、failure code 收敛，再推进更完整的 live workflow

这是当前 `quantix-rust` 与 miniQMT 文档之间，风险最低且长期可维护的对齐路径。
