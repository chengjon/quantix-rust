# Quantix-Rust Function Tree

> Updated: 2026-03-26
> Scope: completed functional design and current operator-facing capability tree
> Source of truth: current merged `master` worktree, README, USER_MANUAL, and implemented modules under `src/`

---

## 1. Current Design Summary

Quantix-Rust currently has a completed functional design around five stable centers:

1. Data acquisition and storage
2. Strategy generation and execution orchestration
3. Trade, risk, monitor, and stop operator workflows
4. Market / screener / analysis decision support
5. Windows bridge v1 integration for cross-platform access

The key architectural boundary is:

- `quantix-rust` in WSL2 owns runtime state, execution requests, execution kernel orchestration, paper/mock-live execution, and local audit storage
- Windows-side bridge work is an external capability boundary, not a second runtime state machine

Current canonical Windows-side bridge path:

```text
/mnt/d/mystocks/quantix/quantix_bridge
```

---

## 2. Top-Level Function Tree

```text
Quantix-Rust
├── Data Plane
│   ├── Historical query and export
│   ├── Real-time quote collection
│   ├── TDX direct / file parsing / fallback loading
│   ├── AkShare and EastMoney adapters
│   ├── ClickHouse / PostgreSQL / TDengine integration
│   └── ETL and batch import/export
│
├── Strategy Plane
│   ├── Strategy definitions
│   │   ├── ma_cross
│   │   ├── mean_reversion
│   │   ├── momentum
│   │   ├── breakout
│   │   └── grid
│   ├── Single-run execution
│   │   ├── backtest
│   │   ├── paper
│   │   └── mock_live
│   ├── Signal daemon
│   ├── Signal approval / rejection
│   ├── Execution request creation
│   └── Execution daemon consumption
│
├── Execution Plane
│   ├── ExecutionKernel
│   ├── runtime.db audit model
│   ├── paper adapter
│   ├── mock_live adapter
│   ├── request lifecycle closure
│   ├── recovery / pending-order handling
│   └── execution bridge CLI
│
├── Operator Plane
│   ├── watchlist
│   ├── monitor
│   ├── stop
│   ├── trade
│   ├── risk
│   ├── market
│   └── screener
│
├── Analysis Plane
│   ├── auction analysis
│   ├── indicators
│   ├── portfolio / performance
│   ├── candle patterns
│   └── Polars-based batch analysis
│
└── Windows Bridge v1
    ├── TDX bridge source
    ├── bridge HTTP client
    ├── watchlist bridge quote lookup
    ├── QMT preview-only adapter
    └── execution bridge CLI preview/status
```

---

## 3. Functional Tree By User Job

## 3.1 Data and Research

```text
Data and Research
├── Query K-line data
├── Export data files
├── Batch quote collection
├── TDX file parsing and复权
├── AkShare / EastMoney reads
├── Multi-period K-line access
├── Market sector / concept / northbound views
├── Screener preset execution
└── Candle pattern recognition
```

Primary modules:

- `src/data/*`
- `src/sources/*`
- `src/db/*`
- `src/market/*`
- `src/screener/*`
- `src/analysis/*`

## 3.2 Strategy and Execution

```text
Strategy and Execution
├── Run strategy in backtest mode
├── Run strategy in paper mode
├── Run strategy in mock_live mode
├── Persist run / signal / order / event audit rows
├── Generate signals through daemon
├── Approve or reject signals
├── Create frozen execution requests
├── Execute pending requests manually
├── Consume pending requests through execution daemon
└── Recover non-final mock_live orders
```

Primary modules:

- `src/strategy/*`
- `src/execution/*`
- `src/cli/handlers.rs`

## 3.3 Trade, Risk, and Monitoring

```text
Trade, Risk, and Monitoring
├── Local paper account init/reset
├── Local buy/sell and trade history
├── Fee and position reporting
├── Risk rule set / enable / disable
├── Live-trade import mirror and rebuild
├── Volatility and daily-loss risk enforcement
├── Watchlist monitoring and alerts
├── Stop rule set / update / history
└── systemd/user-service wrappers for monitor / strategy
```

Primary modules:

- `src/trade/*`
- `src/risk/*`
- `src/monitor/*`
- `src/monitoring/*`
- `src/stop/*`
- `src/watchlist/*`

---

## 4. Completed Functional Design Areas

## 4.1 Stable, User-Visible Features

These are implemented and documented as active capabilities:

- Data query / export
- Watchlist CRUD and price lookup
- Screener presets and watchlist-based screening
- Market sector / concept / overview reads
- Monitor alerting and event history
- Stop rule lifecycle and trigger audit
- Paper trade account workflows
- Risk rules, locks, import mirror, and rebuild
- Strategy paper execution
- Strategy mock_live execution
- Strategy signal daemon and request workflows
- Execution daemon and request consumption
- Windows bridge v1 status / preview integration

## 4.2 Completed Cross-Cutting Design Boundaries

The following design decisions are already embodied in code and docs:

- `runtime.db` is the execution audit store
- `execution_request` is the durable handoff object between approval and execution
- frozen snapshots prevent request intent drift
- `paper` and `mock_live` are the implemented execution targets
- `live` remains intentionally incomplete
- bridge work does not own execution state
- `QMT` is preview-only in bridge v1
- `TDX bridge source` is the first real bridge-delivered capability

---

## 5. Windows Bridge v1 Function Tree

```text
Windows Bridge v1
├── Rust Side
│   ├── src/bridge/client.rs
│   ├── src/bridge/models.rs
│   ├── src/sources/bridge_tdx.rs
│   ├── src/watchlist/resolver.rs (BridgeTdxWatchlistQuoteLookup)
│   ├── src/execution/qmt_bridge.rs
│   └── quantix execution bridge ...
│
└── Windows Side
    └── /mnt/d/mystocks/quantix/quantix_bridge
        ├── /health
        ├── /api/v1/capabilities
        ├── /api/v1/data/tdx/quotes
        ├── /api/v1/data/tdx/kline/{symbol}
        ├── /api/v1/broker/qmt/account/status
        └── /api/v1/broker/qmt/orders/preview
```

Current bridge boundary:

- `TDX bridge source` is functional
- `QMT preview-only` is functional
- `QMT live execution` is not part of the completed design

---

## 6. Current Non-Goals

These are explicitly outside the completed functional design:

- real live broker execution
- Windows-side ownership of runtime state
- Wind / Choice bridge integration
- bridge-side WebSocket / gRPC stack
- distributed workers or multi-process execution daemon coordination

---

## 7. Operator View

If an operator asks “what is already designed and usable?”, the short answer is:

```text
Usable today
├── local data / research / screening
├── local paper trading
├── local risk / monitor / stop workflows
├── strategy paper + mock_live + request lifecycle
├── execution daemon consumption
└── windows bridge v1
    ├── TDX bridge source
    └── QMT preview-only
```

If an operator asks “what remains intentionally deferred?”, the answer is:

```text
Deferred
├── real live adapter
├── real QMT order submission
├── Wind / Choice bridge support
├── bridge-owned order lifecycle
└── broader distributed runtime concerns
```
