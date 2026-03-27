# QMT Live Trading 集成指南

> 本文档记录 Quantix Rust 与 QMT 实盘交易的完整集成方案。
> 适用于 AI 助手或开发者复现实盘交易功能。

---

## 一、系统架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Trading Flow                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐     HTTP/REST     ┌──────────────┐                    │
│  │              │ ────────────────> │              │                    │
│  │  WSL2/Linux  │    127.0.0.1      │   Windows    │ ┌────────────────┐ │
│  │  Rust App    │     :17580        │   Bridge     │ │  QMT (交易)    │ │
│  │              │ <──────────────── │              │ │  xtquant       │ │
│  └──────────────┘     JSON API      └──────────────┘ └────────────────┘ │
│                                         │                  │              │
│                                         │  ┌───────────────┘              │
│                                         │  │                              │
│                                         ▼  ▼                              │
│                                    ┌────────────────┐                   │
│                                    │  PyTDX (行情)  │                   │
│                                    │  独立行情服务  │                   │
│                                    │  无需QMT       │                   │
│                                    └────────────────┘                   │
│                                           │                             │
│                                           ▼                             │
│                                    ┌────────────────┐                   │
│                                    │  通达信服务器  │                   │
│                                    │  公共行情源    │                   │
│                                    └────────────────┘                   │
└─────────────────────────────────────────────────────────────────────────┘

组件说明：
- Rust App (WSL2): 量化策略引擎，运行在 Linux 环境
- Windows Bridge: FastAPI 服务，桥接 Rust 与 QMT/PyTDX
- QMT Client: 迅投 QMT 交易终端，需登录券商账户（仅交易时需要）
- xtquant/xttrader: QMT 官方 Python SDK（仅交易时需要）
- PyTDX: 独立行情服务，直接连接通达信服务器获取行情数据（无需 QMT）
```

**注意**: 行情数据现在可以通过 PyTDX 独立获取，无需启动 QMT 客户端。只有执行交易时才需要 QMT。

---

## 二、环境要求

### 2.1 Windows 端

| 组件 | 版本要求 | 说明 |
|------|----------|------|
| Python | 3.10+ | 推荐 3.12 |
| QMT 客户端 | 最新版 | 需支持 miniQMT 功能 |
| uv | 最新版 | Python 包管理器 |

### 2.2 WSL2 端

| 组件 | 版本要求 | 说明 |
|------|----------|------|
| Rust | 1.75+ | cargo 构建工具 |
| 模块依赖 | 见 Cargo.toml | reqwest, tokio, serde 等 |

### 2.3 网络要求

- WSL2 可访问 Windows `127.0.0.1`（默认已配置）
- Windows 防火墙允许 17580 端口

---

## 三、目录结构

```
D:\mystocks\
├── quantix-rust\                 # WSL2: /mnt/d/mystocks/quantix-rust
│   ├── src/
│   │   ├── bridge/
│   │   │   ├── client.rs         # Bridge HTTP 客户端
│   │   │   ├── models.rs         # API 数据模型
│   │   │   └── error.rs          # 错误处理
│   │   ├── execution/
│   │   │   ├── qmt_live_adapter.rs  # QMT 实盘适配器 ⭐
│   │   │   ├── daemon.rs         # 执行守护进程
│   │   │   └── adapter.rs        # ExecutionAdapter trait
│   │   └── core/
│   │       └── runtime.rs        # Bridge 配置加载
│   ├── tests/
│   │   └── bridge_integration_test.rs  # 集成测试
│   └── Cargo.toml
│
└── quantix\                      # Windows 项目目录
    └── quantix_bridge\           # Bridge 服务 ⭐
        ├── app/
        │   ├── main.py           # FastAPI 入口
        │   ├── config.py         # 配置管理
        │   ├── routes/
        │   │   ├── qmt.py        # QMT 交易接口
        │   │   ├── tdx.py        # TDX 行情接口
        │   │   └── capabilities.py
        │   ├── services/
        │   │   ├── qmt_service.py   # QMT 服务核心 ⭐
        │   │   └── tdx_service.py
        │   └── models/
        │       └── qmt.py        # QMT 数据模型
        ├── pyproject.toml
        ├── .env                  # 环境配置 ⭐
        └── test_bridge.py        # 测试脚本
```

---

## 四、配置步骤

### 4.1 QMT 客户端配置

1. **安装 QMT 客户端**
   - 路径示例：`D:\国金QMT交易端模拟\`
   - 主程序：`bin.x64\XtItClient.exe`

2. **启用 miniQMT 功能**
   - 联系券商开通 miniQMT 权限
   - 确保 `userdata_mini` 目录存在

3. **登录 QMT**
   - 启动客户端并登录
   - 保持登录状态（Bridge 需要连接）

### 4.2 Bridge 服务配置

1. **创建 `.env` 文件**

```bash
# D:\mystocks\quantix\quantix_bridge\.env

# Server settings
BRIDGE_SERVER_HOST=127.0.0.1
BRIDGE_SERVER_PORT=17580

# API Key (optional)
BRIDGE_API_KEY=

# TDX settings
BRIDGE_TDX_ENABLED=true

# QMT settings
BRIDGE_QMT_ENABLED=true
BRIDGE_QMT_MODE=live

# QMT userdata_mini path (IMPORTANT!)
BRIDGE_QMT_USERDATA_PATH=D:\\国金QMT交易端模拟\\userdata_mini

# QMT session ID
BRIDGE_QMT_SESSION_ID=123456

# QMT Account ID (您的资金账号)
BRIDGE_QMT_ACCOUNT_ID=40330341

# Logging
BRIDGE_LOG_LEVEL=INFO
```

2. **关键配置说明**

| 配置项 | 说明 | 示例 |
|--------|------|------|
| `BRIDGE_QMT_USERDATA_PATH` | QMT 的 userdata_mini 目录 | `D:\\国金QMT交易端模拟\\userdata_mini` |
| `BRIDGE_QMT_ACCOUNT_ID` | 资金账号 | `40330341` |
| `BRIDGE_QMT_MODE` | 运行模式 | `live` (实盘) 或 `preview_only` (预览) |

### 4.3 Rust 端配置

Rust 通过环境变量配置 Bridge 连接：

```bash
# 默认值（通常无需修改）
export QUANTIX_BRIDGE_BASE_URL=http://127.0.0.1:17580
export QUANTIX_BRIDGE_API_KEY=  # 可选
```

---

## 五、启动服务

### 5.1 启动 QMT 客户端

```powershell
# Windows: 启动 QMT 并登录
D:\国金QMT交易端模拟\bin.x64\XtItClient.exe
```

### 5.2 启动 Bridge 服务

```powershell
# Windows PowerShell
cd D:\mystocks\quantix\quantix_bridge

# 安装依赖（首次）
uv sync

# 启动服务
uv run uvicorn app.main:app --host 127.0.0.1 --port 17580
```

预期输出：
```
INFO:     Started server process [xxxx]
INFO:     Waiting for application startup.
INFO:     QMT connected: account=****0341
INFO:     QMT connected successfully
INFO:     Uvicorn running on http://127.0.0.1:17580
```

### 5.3 验证连接

```powershell
# 运行测试脚本
uv run python test_bridge.py
```

---

## 六、API 端点

### 6.1 健康检查

```bash
GET /health
GET /api/v1/capabilities
```

### 6.2 账户查询

```bash
# 账户状态
GET /api/v1/broker/qmt/account/status

# 资产信息
GET /api/v1/broker/qmt/account/asset

# 持仓查询
GET /api/v1/broker/qmt/positions
```

### 6.3 订单操作

```bash
# 订单预览（不实际下单）
POST /api/v1/broker/qmt/orders/preview
{
  "request_id": "uuid",
  "client_order_id": "client-001",
  "symbol": "600519.SH",
  "side": "buy",
  "quantity": 100,
  "price": "1500.00",
  "order_type": "limit"
}

# 提交订单
POST /api/v1/broker/qmt/orders
{
  "request_id": "uuid",
  "client_order_id": "live-001",
  "symbol": "600519.SH",
  "side": "buy",
  "quantity": 100,
  "price": "1500.00",
  "order_type": "limit",
  "strategy_name": "my_strategy",
  "order_remark": "test order"
}

# 查询订单
GET /api/v1/broker/qmt/orders/{order_id}

# 撤销订单
DELETE /api/v1/broker/qmt/orders/{order_id}
```

---

## 七、Rust 集成

### 7.1 QmtLiveExecutionAdapter

位置：`src/execution/qmt_live_adapter.rs`

```rust
use crate::bridge::client::BridgeHttpClient;
use crate::execution::qmt_live_adapter::QmtLiveExecutionAdapter;

// 创建 adapter
let client = BridgeHttpClient::new(
    "http://127.0.0.1:17580".to_string(),
    None
)?;
let adapter = QmtLiveExecutionAdapter::new(client);

// 提交订单
let response = adapter.submit_order(AdapterOrderRequest {
    client_order_id: "order-001".to_string(),
    symbol: "600519.SH".to_string(),
    side: OrderSide::Buy,
    quantity: 100,
    price: Decimal::new(1500, 0),
}).await?;
```

### 7.2 Daemon 集成

在 `src/execution/daemon.rs` 中使用 `qmt_live` 模式：

```rust
match request.target_mode.as_str() {
    "paper" => { /* ... */ },
    "mock_live" => { /* ... */ },
    "qmt_live" => {
        let bridge_client = create_bridge_client()?;
        let adapter = QmtLiveExecutionAdapter::new(bridge_client);
        let kernel = ExecutionKernel::new(store.clone(), adapter, risk);
        kernel.execute_request(prepared).await
    },
    // ...
}
```

### 7.3 运行集成测试

```bash
# WSL2
cd /opt/claude/quantix-rust
export QUANTIX_BRIDGE_BASE_URL=http://127.0.0.1:17580
cargo test --test bridge_integration_test -- --nocapture
```

---

## 八、测试验证

### 8.1 从 WSL2 测试（curl）

```bash
# 1. 健康检查
curl -s http://127.0.0.1:17580/health | jq .

# 2. 账户状态
curl -s http://127.0.0.1:17580/api/v1/broker/qmt/account/status | jq .

# 3. 资产查询
curl -s http://127.0.0.1:17580/api/v1/broker/qmt/account/asset | jq .

# 4. 持仓查询
curl -s http://127.0.0.1:17580/api/v1/broker/qmt/positions | jq .

# 5. 订单预览
curl -s -X POST http://127.0.0.1:17580/api/v1/broker/qmt/orders/preview \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "test-001",
    "client_order_id": "preview-001",
    "symbol": "600519.SH",
    "side": "buy",
    "quantity": 100,
    "price": "1500.00",
    "order_type": "limit"
  }' | jq .

# 6. 提交真实订单（谨慎！）
curl -s -X POST http://127.0.0.1:17580/api/v1/broker/qmt/orders \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "live-001",
    "client_order_id": "live-001",
    "symbol": "600519.SH",
    "side": "buy",
    "quantity": 100,
    "price": "1400.00",
    "order_type": "limit"
  }' | jq .

# 7. 查询订单
curl -s http://127.0.0.1:17580/api/v1/broker/qmt/orders/{order_id} | jq .

# 8. 撤销订单
curl -s -X DELETE http://127.0.0.1:17580/api/v1/broker/qmt/orders/{order_id} | jq .
```

### 8.2 预期测试结果

| 测试项 | 预期结果 |
|--------|----------|
| Health | `{"status": "ok"}` |
| Account Status | `connected: true` |
| Asset | `total_asset`, `cash` 有值 |
| Order Preview | `latest_status: "accepted"` |
| Order Submit | `adapter_order_id` 有值，`latest_status: "submitted"` |
| Order Cancel | `success: true` |

---

## 九、故障排除

### 9.1 QMT 连接失败

**症状**：`connected: false`

**检查**：
1. QMT 客户端是否已启动并登录？
2. `BRIDGE_QMT_USERDATA_PATH` 路径是否正确？
3. `BRIDGE_QMT_ACCOUNT_ID` 是否填写正确？
4. miniQMT 功能是否已开通？

**解决**：
```powershell
# 检查 userdata_mini 目录是否存在
dir D:\国金QMT交易端模拟\userdata_mini

# 重启 Bridge 服务
taskkill /f /im python.exe
uv run uvicorn app.main:app --host 127.0.0.1 --port 17580
```

### 9.2 WSL2 无法连接 Bridge

**症状**：Connection refused

**检查**：
1. Bridge 服务是否在运行？
2. 防火墙是否阻止 17580 端口？

**解决**：
```powershell
# 检查端口
netstat -ano | findstr 17580

# 允许防火墙（管理员权限）
netsh advfirewall firewall add rule name="Quantix Bridge" dir=in action=allow protocol=tcp localport=17580
```

### 9.3 订单被拒绝

**症状**：`rejection_reason` 有值

**可能原因**：
1. 资金不足
2. 股票代码错误
3. 价格超出涨跌停限制
4. 非交易时间

### 9.4 撤单失败

**症状**：`success: false`

**可能原因**：
1. 订单已成交
2. 订单已撤销
3. 订单不存在

---

## 十、安全注意事项

1. **API Key**：生产环境建议启用 API Key 认证
2. **网络隔离**：Bridge 仅监听 127.0.0.1，不暴露外网
3. **日志审计**：所有交易操作都有日志记录
4. **风控集成**：建议与 `src/risk/` 模块配合使用
5. **测试模式**：首次测试建议使用 `preview_only` 模式

---

## 十一、版本信息

| 组件 | 版本 | 日期 |
|------|------|------|
| quantix-rust | 0.1.0 | 2026-03-27 |
| quantix-bridge | 1.0.0 | 2026-03-27 |
| xtquant | 250516.1.1 | - |
| QMT | 国金模拟版 | - |

---

## 十二、参考链接

- [迅投 QMT 官方文档](https://dict.thinktrader.net/nativeApi/start_now.html)
- [xtquant PyPI](https://pypi.org/project/xtquant/)
- [项目 README](../../README.md)
- [开发路线图](./DEVELOPMENT_ROADMAP.md)

---

*文档更新：2026-03-27*
*测试验证：✅ 全部通过*
