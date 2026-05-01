# FUNCTION_TREE

本文件是当前功能树的兼容入口，便于与外部项目文档命名对齐。

- 完整展开版功能树：`docs/FUNCTION_MAP.md`
- 当前最新主线基线：`origin/master@51f4834`

## 2026-05-01 当前执行/桥接能力树

```text
quantix-rust
├── core/runtime
│   ├── CliRuntime
│   └── BridgeRuntimeSettings
│       ├── base_url / api_key
│       ├── bearer_token / contract_version
│       └── poll_interval_ms / poll_timeout_ms
│
├── bridge
│   ├── BridgeHttpClient
│   │   ├── capabilities
│   │   ├── task_execute_qmt_submit
│   │   ├── task_result
│   │   ├── qmt_preview_order
│   │   └── qmt_cancel_order
│   └── models
│       ├── BridgeTaskExecuteRequest / Receipt
│       ├── BridgeTaskResultResponse
│       │   └── pending 允许 result = null
│       ├── BridgeBrokerEventType
│       └── BridgeFailureCode
│
├── execution
│   ├── request_diagnostics
│   │   ├── execution_diagnostics 结构化负载
│   │   └── qmt_live gate 分类
│   ├── qmt_live_gate
│   │   └── ensure_bridge_qmt_live_mode
│   ├── qmt_task_submit_service
│   │   ├── submit_order -> task receipt
│   │   ├── query_task_result_once / by_task_id
│   │   └── poll_task_result_until_terminal
│   ├── qmt_live_adapter
│   │   ├── submit_order -> PendingSubmit
│   │   ├── query_order -> pending / accepted / rejected / filled
│   │   └── cancel_order -> legacy compatibility endpoint
│   └── qmt_bridge
│       └── preview-only path
│
└── focused verification
    ├── tests/bridge_client_test.rs
    ├── tests/qmt_task_contract_test.rs
    ├── tests/qmt_live_adapter_test.rs
    ├── tests/monitor_systemd_test.rs
    └── tests/strategy_systemd_test.rs
```

## 当前边界

- `paper` + `mock_live` + guarded `qmt_live` 是当前已实现执行目标
- 通用 `live` 语义仍故意保持不完整，不等于 `qmt_live`
- Windows Bridge v1 当前支持：
  - `TDX` bridge 数据读取
  - `QMT` preview
  - `QMT` guarded `qmt_live` task receipt/result 提交流程
