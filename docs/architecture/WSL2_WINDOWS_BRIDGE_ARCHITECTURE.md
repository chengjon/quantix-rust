# Quantix 架构方案: WSL2 开发 + Windows Bridge

> 版本: 2.0
> 日期: 2026-03-26
> 状态: 评审后重写
> 适用范围: 对齐当前 `quantix-rust` 执行内核与数据源架构

---

## 1. 摘要

本文替换上一版“Windows 统一网关一次性承接交易、行情、WebSocket、gRPC”的宽范围方案。

重写后的结论是：

- `WSL2` 继续作为 `quantix-rust` 的主要开发与运行环境
- `Windows` 侧只承接当前确实需要的专有能力
- `v1` 范围收敛为两项：
  - `TDX bridge source`：提供真实可用的远端行情与 K 线读取
  - `QMT bridge contract`：定义并验证券商执行边界，但首期不真实发单
- `quantix-rust` 继续保留当前执行架构的所有权：
  - `execution_request`
  - frozen execution snapshot
  - `ExecutionKernel::execute_request(...)`
  - `runtime.db`
  - risk evaluation
  - order / order_event 持久化
- `Windows bridge` 不是执行生命周期的 source of truth，只是远端能力边界

这意味着首期目标不是“把 live trading 做完”，而是先把跨 `WSL2 <-> Windows` 的稳定边界、配置、安全、错误模型和数据契约做对。

---

## 2. 设计目标

### 2.1 目标

1. 保持 `quantix-rust` 在 `WSL2` 中开发、测试、回测、策略研发的主路径不变
2. 通过明确的 bridge 边界访问 Windows 独有能力，而不是把执行状态迁移到 Windows
3. 为 `TDX` 提供可实际接入的远端数据源能力
4. 为 `QMT` 提供和当前 `ExecutionAdapter` 契约对齐的预演型 broker contract
5. 避免在首期把“跨系统边界验证”和“真实券商副作用”绑定在一起
6. 为后续真正的 `live` adapter 留出清晰升级路径

### 2.2 非目标

本期明确不做：

- 真实 `QMT` 发单
- `target_mode=live` 落地
- `Wind` / `Choice` 接入
- `gRPC` 接口
- 面向浏览器客户端的开放 API
- 广播式 WebSocket 行情推送
- 把 `runtime.db` 或风险状态迁移到 Windows
- 在 Windows bridge 内部复制一套交易状态机

---

## 3. 与当前 `quantix-rust` 架构的对齐

当前仓库已经形成了明确执行边界，本文必须服从这些既有约束：

### 3.1 执行生命周期仍归 `quantix-rust`

当前项目中：

- `ExecutionKernel` 负责执行编排
- `execution_request` 表达“可执行交接态”
- frozen snapshot 避免 request 语义漂移
- `runtime.db` 保存 run、signal、order、order_event、request 结果
- adapter 返回的状态会被内核立即用于 order-event 写入和 fill-delta 处理

因此，Windows bridge 不得直接成为新的“交易主状态机”。

### 3.2 Bridge 只提供远端能力，不拥有核心状态

Bridge 可以负责：

- 远端数据读取
- 券商接口探测
- broker-facing payload 校验
- 归一化错误与状态编码

Bridge 不负责：

- risk rule 判定
- request lifecycle 管理
- 最终 order / order_event source of truth
- 本地持仓或运行时状态替代

### 3.3 `QMT` 首期是 contract-first，不是 live-first

当前 `paper` / `mock_live` 仍是唯一真正可执行的 target mode。

本期 `QMT` 的目标是：

- 先对齐 `ExecutionAdapter` 返回语义
- 先对齐 request snapshot 所需字段
- 先验证 Windows 侧 SDK 可调用、参数可映射、状态可归一化

本期不让 `QMT bridge` 进入真实发单路径。

### 3.4 `TDX` 是首期真正交付能力

相比 `QMT live`，`TDX` 远端读取：

- 风险低
- 价值明确
- 更容易做成可验收的 bridge slice

因此 `TDX bridge source` 是 `v1` 的主要可运行交付物。

---

## 4. 总体架构

### 4.1 总览

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Windows Host                                                        │
│                                                                      │
│  quantix-bridge (Python / FastAPI, HTTP only in v1)                 │
│                                                                      │
│  ┌─────────────────────────┐    ┌─────────────────────────────────┐ │
│  │ TDX Market Service      │    │ QMT Broker Preview Service      │ │
│  │ - quote                 │    │ - account status                │ │
│  │ - batch quotes          │    │ - order payload validation      │ │
│  │ - kline                 │    │ - normalized preview response   │ │
│  └─────────────┬───────────┘    └─────────────────┬───────────────┘ │
│                │                                  │                 │
│           TDX / 本地网络                     miniQMT / xtquant      │
└────────────────┼──────────────────────────────────┼─────────────────┘
                 │ HTTP + API Key                    │ HTTP + API Key
┌────────────────▼──────────────────────────────────▼─────────────────┐
│ WSL2                                                               │
│                                                                     │
│ quantix-rust                                                       │
│                                                                     │
│  ┌─────────────────────────┐    ┌────────────────────────────────┐ │
│  │ BridgeHttpClient        │    │ Existing execution subsystem   │ │
│  │ - auth                  │    │ - ExecutionKernel             │ │
│  │ - retries               │    │ - execution_request           │ │
│  │ - timeout               │    │ - runtime.db                  │ │
│  └─────────────┬───────────┘    │ - paper / mock_live adapters   │ │
│                │                └────────────────────────────────┘ │
│                │                                                   │
│   ┌────────────▼────────────┐    ┌───────────────────────────────┐ │
│   │ BridgeTdxSource         │    │ QmtBridgePreviewAdapter       │ │
│   │ - remote quote/kline    │    │ - preview only               │ │
│   │ - source fallback       │    │ - not a live target mode     │ │
│   └─────────────────────────┘    └───────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 核心原则

1. 运行时审计与执行状态留在 `WSL2`
2. Windows bridge 只暴露“最小必要能力”
3. `TDX` 先落地，`QMT` 先定义边界
4. 先做 HTTP，同步请求优先；实时推送后置
5. 配置以显式为主，自动发现只作为辅助脚本，不作为运行时默认假设

---

## 5. 组件设计

## 5.1 Windows: `quantix-bridge`

建议保留单独的 Windows Python 项目，但范围收窄。

当前约定的 Windows 侧主目录布局是：

```text
/mnt/d/mystocks/quantix/
└── quantix_bridge/
```

### 5.1.1 必需组件

- `app/main.py`
  - FastAPI 入口
  - 只暴露 HTTP API
- `app/security.py`
  - API Key 校验
  - host allowlist 校验
  - 请求日志脱敏
- `app/services/tdx_service.py`
  - 远端行情与 K 线读取
  - 统一符号格式
- `app/services/qmt_preview_service.py`
  - `xtquant` 可用性检查
  - 账户连接状态读取
  - 下单 payload 映射校验
  - 返回归一化 preview 结果
- `app/models/*.py`
  - 统一响应模型
- `app/config.py`
  - bridge 配置

### 5.1.2 不应放入 bridge 的能力

- 风控规则执行
- frozen request snapshot 构造
- 订单生命周期审计落库
- 策略执行编排
- 多模式 execution daemon

## 5.2 WSL2: `quantix-rust`

### 5.2.1 新增 bridge 客户端层

建议新增：

```text
src/bridge/
├── mod.rs
├── client.rs
├── models.rs
└── error.rs
```

职责：

- HTTP 请求封装
- API Key 注入
- 超时与错误归一化
- Windows bridge response 到 Rust 内部模型的转换

### 5.2.2 `TDX` 侧集成

建议新增：

```text
src/sources/bridge_tdx.rs
```

职责：

- 调用 Windows bridge 的 `TDX` quote / kline API
- 返回和现有 `sources` 子系统可兼容的数据结构
- 允许上层配置选择本地 `TDX` 直连或 bridge `TDX`

### 5.2.3 `QMT` 侧集成

建议新增：

```text
src/execution/qmt_bridge.rs
```

职责：

- 定义一个和当前 `ExecutionAdapter` 契约对齐的 `QmtBridgePreviewAdapter`
- 仅做 preview / validation，不做真实提交
- 为未来真实 `QmtLiveExecutionAdapter` 预留代码位置

---

## 6. 数据流

## 6.1 `TDX` 行情读取流程

```text
quantix-rust CLI / service
    -> BridgeTdxSource
    -> BridgeHttpClient
    -> Windows quantix-bridge /api/v1/data/tdx/...
    -> TDX backend
    -> normalized response
    -> quantix-rust source layer
```

特点：

- 属于读操作
- 不改变现有 execution 生命周期
- 可以独立验收

## 6.2 `QMT` preview 流程

`QMT` 首期不是“执行订单”，而是“验证桥接契约”。

推荐流程：

```text
execution_request / frozen snapshot
    -> preview command or adapter test harness
    -> QmtBridgePreviewAdapter
    -> BridgeHttpClient
    -> Windows quantix-bridge /api/v1/broker/qmt/orders/preview
    -> xtquant parameter validation / account status check
    -> normalized preview response
    -> quantix-rust side prints or records preview result
```

关键点：

- preview 输入必须来自 frozen snapshot 或等价结构
- 不重新推导数量、价格、side
- 不触发真实券商副作用
- 不把 preview 结果误写成 live settlement

## 6.3 未来 `QMT live` 流程

未来如果要落地真实 `live`，推荐路径是：

1. 保持 `execution_request -> execute_request(...)` 的既有边界
2. 用真实 `QmtLiveExecutionAdapter` 取代 preview adapter
3. adapter 返回值必须完整对齐当前 `OrderInitialResponse` / `OrderQueryResponse`
4. cancel / query / unknown / partial fill 必须先定义清楚再开放

本文件不把这一步纳入 `v1`。

---

## 7. Bridge API 设计

`v1` 只使用 HTTP。

### 7.1 认证

所有业务接口必须要求：

- `X-Quantix-Api-Key: <secret>`

`GET /health` 可只暴露基础进程健康，不暴露敏感信息。

### 7.2 `GET /health`

响应示例：

```json
{
  "status": "ok",
  "service": "quantix-bridge",
  "version": "2.0.0"
}
```

### 7.3 `GET /api/v1/capabilities`

返回 bridge 当前启用能力。

```json
{
  "tdx": {
    "enabled": true,
    "supports": ["quote", "batch_quotes", "kline"]
  },
  "qmt": {
    "enabled": true,
    "mode": "preview_only",
    "supports": ["account_status", "order_preview"]
  }
}
```

### 7.4 `POST /api/v1/data/tdx/quotes`

请求：

```json
{
  "symbols": ["000001.SZ", "600519.SH"]
}
```

响应：

```json
{
  "quotes": [
    {
      "symbol": "000001.SZ",
      "name": "平安银行",
      "last": 15.50,
      "bid": 15.49,
      "ask": 15.51,
      "open": 15.45,
      "high": 15.60,
      "low": 15.40,
      "pre_close": 15.30,
      "volume": 12345678,
      "turnover": 191234567.89,
      "timestamp": "2026-03-26T14:30:00Z",
      "source": "tdx_bridge"
    }
  ]
}
```

### 7.5 `GET /api/v1/data/tdx/kline/{symbol}`

查询参数：

- `period`
- `start`
- `end`

响应：

```json
{
  "symbol": "000001.SZ",
  "period": "1d",
  "bars": [
    {
      "datetime": "2026-03-26",
      "open": 14.80,
      "high": 15.10,
      "low": 14.75,
      "close": 15.00,
      "volume": 87654321,
      "turnover": 1312345678.00
    }
  ],
  "source": "tdx_bridge"
}
```

### 7.6 `GET /api/v1/broker/qmt/account/status`

响应示例：

```json
{
  "adapter": "qmt",
  "mode": "preview_only",
  "sdk_available": true,
  "connected": true,
  "account_masked": "****1234"
}
```

### 7.7 `POST /api/v1/broker/qmt/orders/preview`

该接口是首期 `QMT` 的核心契约。

请求应尽量贴近当前 `ExecutionAdapter` 需要的字段：

```json
{
  "request_id": "req_123",
  "client_order_id": "cli_456",
  "symbol": "000001.SZ",
  "side": "buy",
  "quantity": 100,
  "price": "15.50",
  "order_type": "limit",
  "snapshot_metadata": {
    "strategy_name": "ma_cross",
    "source": "execution_request"
  }
}
```

响应必须直接兼容当前 adapter 所需的归一化语义：

```json
{
  "adapter_order_id": "preview-cli_456",
  "latest_status": "accepted",
  "filled_quantity": 0,
  "avg_fill_price": null,
  "fill_details": null,
  "rejection_reason": null,
  "broker_payload": {
    "market": "SZ",
    "qmt_order_type": "limit"
  }
}
```

若校验失败：

```json
{
  "adapter_order_id": "preview-cli_456",
  "latest_status": "rejected",
  "filled_quantity": 0,
  "avg_fill_price": null,
  "fill_details": null,
  "rejection_reason": "invalid_symbol_for_qmt"
}
```

### 7.8 状态映射规则

Bridge 对外状态应与当前 `quantix-rust` `OrderStatus` 对齐：

| Bridge `latest_status` | `quantix-rust` `OrderStatus` |
|------------------------|------------------------------|
| `submitted`            | `Submitted`                  |
| `accepted`             | `Accepted`                   |
| `partially_filled`     | `PartiallyFilled`            |
| `filled`               | `Filled`                     |
| `canceled`             | `Canceled`                   |
| `rejected`             | `Rejected`                   |
| `unknown`              | `Unknown`                    |

`pending_submit` 仍然由 `ExecutionKernel` 在本地落库时生成，不由 bridge 返回。

---

## 8. 配置设计

## 8.1 `quantix-rust` 配置

建议扩展现有 [config/default.toml](../../config/default.toml)：

```toml
[bridge]
enabled = false
base_url = "http://127.0.0.1:17580"
timeout_ms = 3000
api_key_env = "QUANTIX_BRIDGE_API_KEY"
discovery_mode = "explicit" # explicit | helper_script

[bridge.tdx]
enabled = false
prefer_bridge = false

[bridge.qmt]
enabled = false
mode = "preview_only"
```

设计要求：

- 运行时必须优先使用显式 `base_url`
- 不依赖 `/etc/resolv.conf` 作为默认发现机制
- 自动发现只能是辅助脚本，不能是运行时唯一方案

## 8.2 Windows bridge 配置

建议使用独立 YAML：

```yaml
server:
  host: "127.0.0.1"
  port: 17580

security:
  api_key: "${BRIDGE_API_KEY}"
  allow_origins: []
  allowed_hosts:
    - "127.0.0.1"
    - "::1"

tdx:
  enabled: true
  hosts:
    - "119.147.212.81:7709"
    - "14.215.128.18:7709"
  timeout_secs: 5

qmt:
  enabled: true
  mode: "preview_only"
  account: "${QMT_ACCOUNT}"
```

如需从 `WSL2` 访问 Windows host IP，应显式调整 `allowed_hosts` 和防火墙规则，而不是默认开放 `0.0.0.0`。

---

## 9. 安全与网络约束

### 9.1 默认安全策略

默认策略应是：

- 不开放到局域网
- 不开启宽松 CORS
- 不允许匿名访问
- 不在日志中记录账号、密码、API Key、完整请求 payload

### 9.2 访问方式

优先级建议：

1. 显式配置 `QUANTIX_BRIDGE_BASE_URL`
2. 使用辅助脚本写入环境变量
3. 手工确认 Windows host 可达后再启用 bridge

不建议：

- 在库代码里默认解析 `/etc/resolv.conf`
- 把某个 `172.x.x.x` 地址硬编码为默认值
- 用“自动发现成功率高”代替稳定配置

### 9.3 防火墙

若必须开放到 `WSL2`，规则应限制到：

- 当前主机
- 必要端口
- 必要网段

不建议“一次性放开 17580-17582 所有入站”。

### 9.4 传输协议

`v1` 使用 HTTP 足够。

原因：

- `TDX` 读接口是请求/响应型
- `QMT` 首期仅 preview
- 先把 contract 和状态语义稳定下来，比先加 gRPC / WS 更重要

`WebSocket` 或 `gRPC` 若后续需要，应在 `v2+` 单独立项。

---

## 10. 项目结构建议

### 10.1 Windows: `quantix-bridge`

```text
quantix-bridge/
├── pyproject.toml
├── config/
│   └── config.yaml
├── app/
│   ├── main.py
│   ├── config.py
│   ├── security.py
│   ├── models/
│   │   ├── common.py
│   │   ├── tdx.py
│   │   └── qmt.py
│   ├── routes/
│   │   ├── health.py
│   │   ├── capabilities.py
│   │   ├── tdx.py
│   │   └── qmt.py
│   └── services/
│       ├── tdx_service.py
│       └── qmt_preview_service.py
└── tests/
    ├── test_health.py
    ├── test_tdx_routes.py
    └── test_qmt_preview.py
```

### 10.2 WSL2: `quantix-rust`

```text
quantix-rust/
├── src/
│   ├── bridge/
│   │   ├── mod.rs
│   │   ├── client.rs
│   │   ├── models.rs
│   │   └── error.rs
│   ├── execution/
│   │   ├── mod.rs
│   │   └── qmt_bridge.rs
│   └── sources/
│       ├── mod.rs
│       └── bridge_tdx.rs
├── config/
│   └── default.toml
└── tests/
    ├── bridge_tdx_source_test.rs
    └── qmt_bridge_preview_test.rs
```

---

## 11. 实施计划

## 11.1 Phase 1: Bridge 基础骨架

交付：

- Windows `quantix-bridge` 最小服务
- `GET /health`
- `GET /api/v1/capabilities`
- API Key 校验

完成标准：

- `WSL2` 可通过显式 `base_url` 访问 bridge
- 未带 API Key 请求被拒绝

## 11.2 Phase 2: `TDX bridge source`

交付：

- `POST /api/v1/data/tdx/quotes`
- `GET /api/v1/data/tdx/kline/{symbol}`
- Rust `BridgeTdxSource`

完成标准：

- `quantix-rust` 可通过 bridge 读取实时行情
- `quantix-rust` 可通过 bridge 读取日线 K 线
- bridge 不可用时，错误信息明确且不污染 execution 生命周期

## 11.3 Phase 3: `QMT preview contract`

交付：

- `GET /api/v1/broker/qmt/account/status`
- `POST /api/v1/broker/qmt/orders/preview`
- Rust `QmtBridgePreviewAdapter`

完成标准：

- frozen snapshot 可映射到 preview 请求
- preview 响应字段与 `OrderInitialResponse` 语义对齐
- 不触发真实券商副作用

## 11.4 Phase 4: 文档与运维

交付：

- bridge 启动文档
- 显式配置说明
- 故障排查说明

完成标准：

- 用户能明确配置 `base_url`、API Key、Windows 侧依赖
- 网络不可达、认证失败、TDX 不可用、QMT SDK 不可用时都有明确排错路径

## 11.5 未来 Phase 5: 真实 `QMT live`

这不是本期范围，但可以作为后续单独 phase。

准入条件：

- `query_order` / `cancel_order` 语义清晰
- partial fill / unknown / retry 语义清晰
- 当前 request lifecycle 与 live adapter 映射已评审通过
- 安全策略已从“本机开发”提升到“可控生产使用”

---

## 12. 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| Windows host 地址发现不稳定 | 高 | 高 | 显式 `base_url`；自动发现仅作脚本辅助 |
| TDX 远端数据格式与本地源差异 | 中 | 中 | 在 bridge 层做统一归一化；Rust 侧做契约测试 |
| QMT SDK 仅 Windows 可用导致 CI 难验证 | 高 | 中 | 将 QMT preview 抽象成 contract test；Windows 专项测试单独运行 |
| 把 preview 误当成 live 执行 | 中 | 高 | 文档、命名、配置全部明确 `preview_only` |
| bridge 过度扩张为“第二套后端” | 中 | 高 | 严格限制其职责，不迁移 request/risk/runtime state |
| 安全边界配置错误 | 中 | 高 | 强制 API Key；默认本机绑定；最小防火墙开放 |

---

## 13. 开放问题

1. `TDX` bridge 返回的 symbol 规范是否统一采用 `000001.SZ` / `600519.SH`
2. `QMT` preview 是否需要支持多账户选择
3. 未来真实 `QMT live` 中，`Submitted` 与 `Accepted` 的映射边界如何定义
4. 未来真实 `QMT live` 是否需要 bridge 侧保存短期订单查询缓存

---

## 14. 最终结论

推荐采用以下路线：

- `v1` 真正交付 `TDX bridge source`
- `v1` 同时定义并验证 `QMT preview contract`
- 不在 `v1` 启用真实 `live`
- 保持当前 `quantix-rust` execution architecture 不变

这个方案的关键不是“尽快让 Windows 能发单”，而是先确保：

- 边界清晰
- 状态所有权清晰
- 契约与当前内核兼容
- 安全默认值正确

在这些前提下，后续再把 `QMT preview` 升级成真实 `live adapter`，才不会和当前项目的执行内核发生结构性冲突。
