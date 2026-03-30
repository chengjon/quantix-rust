# A-Share Plugin Strategy Architecture Design

**Date:** 2026-03-28
**Status:** Draft for user review
**Depends On:** Current repository state on 2026-03-28

> This document captures the recommended architecture direction for evolving quantix-rust into an A-share quant platform with an independent data foundation, pluggable strategy insertion, shared backtest/simulation execution semantics, and durable horizontal comparison across strategy runs and strategy versions.

---

## Goal

Design the smallest durable architecture that lets quantix-rust support:

1. an **independent data foundation** that remains decoupled from strategy code
2. **plugin-like strategy insertion** with both built-in Rust implementations and external integrations
3. **two strategy shapes**:
   - complete end-to-end strategies
   - composable building blocks such as filters, signal generators, sizers, and policy combiners
4. one **shared decision contract** reused by:
   - historical backtest
   - forward-like simulation / mock-live / paper
   - future real execution modes
5. durable **result recording and horizontal comparison** at both:
   - individual run level
   - aggregated strategy-version / parameter-family / market-phase level

This design is intentionally architecture-first. It does not attempt to specify a full implementation plan, hyperparameter search platform, or distributed execution system.

## Current Repository Findings

The repository already contains strong lower-half seams that should be preserved:

- `src/analysis/backtest.rs`
  - provides an existing backtest engine
- `src/execution/kernel.rs`
  - already separates orchestration, risk evaluation, fill application, and execution flow
- `src/execution/adapter.rs`
  - already exposes a clean adapter boundary for execution backends
- `src/execution/runtime_store.rs`
  - already acts as a runtime audit/control-plane store
- `src/strategy/config.rs`
  - already models configured strategy instances with IDs and JSON parameters

The main architectural weakness is in the strategy layer:

- `src/strategy/trait_def.rs`
  - current `Strategy` trait is too narrow (`on_bar(&Kline) -> Signal`)
- `src/strategy/registry.rs`
  - strategy resolution is currently hardcoded and only partially plugin-like
- `src/strategy/runtime.rs`
  - runtime flow still contains strategy-specific logic rather than catalog-driven execution

The result is a split upper-half architecture:

- one contract for simple strategy callbacks
- another contract for configured evaluators
- partially strategy-specific runtime wiring

That split is the first thing that should be removed.

## Bottom-Up Scope

### User jobs

The minimum architecture must support these user jobs cleanly:

1. register or add a new strategy without rewriting core execution flow
2. define a complete strategy or compose a strategy from reusable parts
3. run the same strategy semantics in backtest and forward-like simulation
4. persist each run with enough metadata to reproduce and compare it later
5. compare outcomes by run, strategy version, parameter family, and market context

### Exact boundary

This design must define:

- the plugin boundary
- the strategy decision contract
- the relationship between backtest, simulation, paper, and future live execution
- the result-comparison boundary
- the A-share realism assumptions that make comparisons meaningful

This design must not require:

- direct plugin access to ClickHouse/SQLite/source adapters
- immediate distributed scheduling
- immediate real broker completion
- raw native dynamic library loading (`.so`, `.dll`) as the primary extension model

### Explicitly deferred

This design does not include:

- distributed workers or broker clusters
- hyperparameter search / grid search infrastructure
- full portfolio optimizer or factor research platform
- UI/dashboard product design
- concrete command syntax for every future workflow

## Approaches Considered

### Option A: Keep the current trait + hardcoded registry and add more strategy names

Pros:

- smallest short-term change
- easy to keep current code running

Cons:

- not truly plugin-like
- backtest and daemon paths will continue to drift
- difficult to support composable strategy parts
- comparison quality will degrade as contracts diverge

### Option B: Introduce a versioned in-process catalog with one shared decision contract

Pros:

- strong type safety
- aligns well with the current Rust architecture
- easy to verify parity between backtest and simulation
- creates a stable host contract before external integration

Cons:

- external strategy authors still need host cooperation unless an adapter layer is added

### Option C: Use a dual-layer model with a shared host contract, versioned in-process catalog, and external out-of-process/WASM plugin boundary

Pros:

- matches the requested “Rust core + external access” direction
- supports both high-performance native strategies and flexible external research workflows
- keeps the host authoritative over data, execution semantics, and result recording
- avoids unsafe native dynamic loading as the primary extension mechanism

Cons:

- larger architecture surface than Option B alone
- requires clear versioning and manifest rules from the start

## Recommendation

Choose **Option C**, but stage it in this order:

1. **first** unify the strategy contract and evaluation pipeline inside the Rust host
2. **then** add a versioned strategy catalog for built-in strategies and composable parts
3. **then** expose the same host contract through an external plugin protocol or WASM boundary

The important point is that “plugin support” should mean **stable host-owned contracts**, not “arbitrary code can execute anywhere.”

quantix-rust should remain the authority for:

- data access semantics
- market-rule semantics
- execution-mode semantics
- result persistence and comparability

## Architecture

### Layered model

The target architecture is a five-layer system:

1. **Data Foundation**
2. **Strategy Catalog**
3. **Decision Pipeline**
4. **Execution / Simulation Drivers**
5. **Experiment & Comparison Store**

The flow is:

`data foundation -> decision context -> strategy or composed pipeline -> normalized decision -> mode-specific execution/simulation -> metrics aggregation -> result persistence -> comparison view`

### Layer 1: Data Foundation

Purpose:

- remain independent from strategy/plugin code
- provide normalized bars, snapshots, features, and market-rule context
- preserve point-in-time correctness and data provenance

Rules:

- strategy code must not directly query ClickHouse, SQLite tables, or source adapters
- all strategy inputs flow through a host-owned data access contract
- data loaders may use the existing `StrategyBarLoader` / fallback patterns internally, but those are host seams, not plugin seams

### Layer 2: Strategy Catalog

Purpose:

- register available strategy capabilities in a versioned, inspectable way
- support both complete strategies and composable parts

Each catalog entry should carry metadata such as:

- `strategy_id`
- `strategy_version`
- `interface_version`
- `plugin_kind` (`builtin`, `external_process`, `wasm`)
- `strategy_shape` (`complete`, `filter`, `signal_generator`, `sizer`, `combiner`, `policy`)
- `param_schema`
- `supported_timeframes`
- `warmup_bars`
- `supports_multi_symbol`

Important rule:

- a “complete strategy” is modeled as a valid top-level catalog entry
- a “composed strategy” is modeled as a declared pipeline of component catalog entries

That lets the platform treat both forms uniformly.

### Layer 3: Decision Pipeline

Purpose:

- give backtest, simulation, paper, and future live the same strategy semantics
- normalize strategy outputs before execution logic sees them

Recommended contract shape:

1. host resolves a catalog entry or composition spec
2. host creates a strategy instance from a factory
3. strategy instance declares `warmup_bars()`
4. host feeds `DecisionContext`
5. strategy returns `StrategyDecision`

#### `DecisionContext`

The strategy input should evolve beyond raw `Kline` and include at least:

- current bar / event
- rolling window or lookback view
- symbol or universe identity
- current portfolio or position state
- available capital or exposure budget
- risk constraints in effect
- market-rule profile
- data snapshot identity / provenance
- optional derived features prepared by the host

#### `StrategyDecision`

The output should evolve beyond `Buy/Sell/Hold` and support:

- signal direction
- target position or weight
- ranked candidate list when applicable
- confidence / score
- explanatory metadata
- policy hints for downstream translation

The execution layer can still translate a simple strategy into `Buy/Sell/Hold`, but the architecture should not lock the system there.

### Layer 4: Execution and Simulation Drivers

Purpose:

- preserve one strategy contract while varying mode-specific semantics

Modes should differ only in:

- data advance model
- fill model
- latency assumptions
- adapter behavior
- persistence detail

Modes should not differ in:

- strategy input contract
- strategy output contract
- decision normalization rules

That means:

- **backtest** = historical replay + simulation fill logic
- **mock-live / paper** = forward-like evaluation + paper adapter / delayed or observed fill logic
- **future live** = same decision contract + real adapter

The existing execution seams already support this direction:

- `ExecutionAdapter`
- `ExecutionKernel`
- `RiskEvaluator`
- `FillDeltaApplier`

These should remain host-owned boundaries below the strategy decision layer.

### Layer 5: Experiment and Comparison Store

Purpose:

- keep research/result comparison separate from operational audit
- support both per-run and aggregated comparison views

Important rule:

- `runtime_store` remains the operational audit/control-plane store
- a separate experiment/result store becomes the research comparison store

This avoids forcing one storage model to serve two very different jobs.

## Plugin Model

### Built-in plugins

Built-in plugins are compile-time registered Rust implementations.

Use them for:

- core production strategies
- high-performance components
- host-trusted building blocks

Advantages:

- type safety
- good performance
- easier testability
- strong control over interface evolution

### External plugins

External plugins should be hosted through either:

- **out-of-process protocol**
- or **WASM sandbox**

They should not be based primarily on arbitrary native dynamic libraries.

Use them for:

- research-oriented strategies
- Python or other language interoperability
- looser experimentation workflows

Every external plugin should declare a manifest containing at least:

- plugin identity
- interface version
- strategy kind / supported shapes
- parameter schema
- capabilities
- required data shape
- supported timeframes

Host rule:

- the host validates the manifest and remains responsible for feeding normalized context and recording results

## Strategy Shapes

The system should support two first-class shapes.

### Shape A: Complete strategy

One entry produces a complete decision pipeline outcome.

Examples:

- MA crossover strategy
- momentum strategy
- mean reversion strategy

### Shape B: Composable building blocks

The architecture should support reusable roles such as:

- `Filter`
- `SignalGenerator`
- `Sizer`
- `Combiner`
- `Policy`

Recommended composition chain:

`Filter -> SignalGenerator -> Sizer -> Combiner/Policy`

This aligns well with the user goal of “搭积木”, and it also gives the host a better way to compare what part of a strategy family is actually driving outcomes.

## Result Model

The comparison model should support **both**:

1. individual run inspection
2. aggregated strategy-version and parameter-family comparison

### Required run identity fields

Every run artifact should record at least:

- `run_id`
- `strategy_instance_id`
- `strategy_id`
- `strategy_version`
- `interface_version`
- `plugin_kind`
- `composition_id` or pipeline spec hash when composed
- `param_hash`
- raw parameter payload
- `symbol` or `universe_id`
- `timeframe`
- `mode` (`backtest`, `simulation`, `paper`, `live`)
- `data_snapshot_id`
- `market_rule_profile`
- `started_at` / `finished_at`
- `seed` if stochastic behavior exists

### Required metric fields

At minimum the architecture should reserve space for:

- win rate
- total return
- annualized return when meaningful
- max drawdown
- trade count
- turnover
- average holding period
- fee assumption
- slippage assumption
- fill ratio where applicable
- live-vs-backtest drift metrics later

### Required comparison views

The system should support comparison at these levels:

- run vs run
- version vs version
- parameter family vs parameter family
- market phase bucket vs market phase bucket

## A-Share Market Realism Rules

For comparisons to remain meaningful, the architecture must make A-share assumptions explicit rather than scattering them across code paths.

The shared market-rule profile should eventually capture at least:

- T+1 constraints
- lot size rules
- suspension handling
- limit up / limit down handling
- corporate action / adjust-type assumptions
- bar timestamp convention

If these assumptions differ between runs, they must be recorded as part of run identity, otherwise horizontal comparison will be misleading.

## Boundary Rules

### Must do

- keep the data foundation independent from plugin internals
- keep one strategy decision contract across all evaluation modes
- separate operational runtime audit from research comparison storage
- keep market-rule assumptions explicit and recorded

### Must not do

- do not let strategies reach directly into storage engines or data vendors
- do not allow backtest and paper to evolve separate decision contracts
- do not treat a hardcoded registry as the final plugin architecture
- do not use unsafe native dynamic loading as the default extensibility story

## Risks and Mitigations

### Risk 1: Contract drift between backtest and daemon/paper

If backtest and forward-like modes use different strategy semantics, result comparison becomes untrustworthy.

Mitigation:

- one `DecisionContext`
- one `StrategyDecision`
- different drivers only below that line

### Risk 2: Fake plugin architecture

If every new strategy still requires host rewiring in multiple places, the system is only plugin-like in name.

Mitigation:

- versioned catalog
- factory-based resolution
- stable manifest / interface versioning

### Risk 3: Audit store and experiment store become tangled

Operational audit and research comparison have different lifecycles and query patterns.

Mitigation:

- keep `runtime_store` for control-plane and audit
- add a separate experiment/result boundary

### Risk 4: Data coupling breaks the independent data foundation

Direct strategy access to DB/source details will make later replacement or consistency work much harder.

Mitigation:

- host-owned normalized data contract
- no direct plugin dependency on source/storage internals

### Risk 5: A-share realism is inconsistent across modes

Ignoring or inconsistently applying T+1, suspensions, or limit rules will distort comparison outcomes.

Mitigation:

- make market rules explicit input/configuration
- persist them with each run artifact

## Recommended Evolution Order

The architecture should evolve in this sequence:

1. **Unify strategy contract**
   - remove the split between current callback and configured evaluator paths
2. **Add versioned strategy catalog**
   - built-in complete strategies and composable parts share the same metadata model
3. **Normalize decision pipeline**
   - shared host-owned `DecisionContext` and `StrategyDecision`
4. **Align verification lanes**
   - backtest, simulation, paper share strategy semantics
5. **Create experiment/result store**
   - durable per-run and aggregated comparison support
6. **Add external plugin protocol**
   - out-of-process or WASM, after host contracts are stable

This order is important. If external plugins arrive before the core host contract stabilizes, the platform will lock in a weak interface and multiply maintenance cost.

## Open Questions

These questions are narrowed enough that they can be resolved in the next design/plan pass rather than before accepting this document:

1. should first-class portfolio / multi-symbol strategies be included in the first host contract or deferred behind a single-symbol-first contract?
2. should the experiment/result store be SQLite-first, Postgres-first, or abstracted behind a repository boundary from day one?
3. should the first external strategy bridge target Python process plugins or WASM first?

## Recommendation Summary

quantix-rust should become a **host-owned strategy evaluation platform**, not just a collection of strategy implementations.

The architecture should preserve the current lower-half strengths and refactor the upper-half strategy layer around:

- one shared strategy decision contract
- one versioned strategy catalog
- one parity-preserving evaluation pipeline
- one explicit split between operational audit and research comparison
- one dual-layer plugin model with built-in Rust and external sandboxed/process-based integrations

That is the shape most likely to let the project support quick strategy insertion, reliable backtest/simulation parity, and meaningful horizontal evaluation over time.
