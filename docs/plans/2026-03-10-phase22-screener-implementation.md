# Phase 22 Screener Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the Phase 22 P0 daily screener so users can run parameterized single-indicator presets against explicit code lists or Phase 21 watchlist universes from the existing CLI.

**Architecture:** Add a small `screener` domain that parses preset invocations, resolves a small universe, fetches per-code daily candles from ClickHouse, evaluates AND-combined preset rules in Rust, and prints CLI results through the existing `analyze` command tree. Reuse Phase 21 watchlist storage/service and existing indicator functions; do not add a DSL, realtime source, or full-market scan.

**Tech Stack:** Rust, clap, existing Quantix CLI modules, ClickHouse client, existing indicator functions, Phase 21 watchlist JSON storage.

---

### Task 1: Add CLI surface for `analyze screener`

**Files:**
- Modify: `src/cli/mod.rs`
- Test: `src/cli/mod.rs`

**Step 1: Write the failing test**

Add parser tests for:

- `quantix analyze screener preset-list`
- `quantix analyze screener run --codes 000001,600519 --preset close_above_ma:period=20`
- `quantix analyze screener run --watchlist --group core --preset rsi_gte:period=14,value=55`

**Step 2: Run test to verify it fails**

Run: `cargo test cli::tests::parses_screener -- --nocapture`

Expected: parser failures because `AnalyzeCommands::Screener` does not exist yet.

**Step 3: Write minimal implementation**

- Add `AnalyzeCommands::Screener(ScreenerCommands)`
- Add:
  - `ScreenerCommands::PresetList`
  - `ScreenerCommands::Run { codes, watchlist, group, preset, limit, sort_by }`
- Keep the CLI shape exactly aligned with the approved design.

**Step 4: Run test to verify it passes**

Run: `cargo test cli::tests::parses_screener -- --nocapture`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add phase22 screener cli surface"
```

### Task 2: Add screener models and preset parser

**Files:**
- Create: `src/screener/mod.rs`
- Create: `src/screener/models.rs`
- Create: `src/screener/parser.rs`
- Modify: `src/lib.rs`
- Test: `tests/screener_parser_test.rs`

**Step 1: Write the failing test**

Cover:

- parsing `close_above_ma:period=20`
- parsing `rsi_gte:period=14,value=55`
- parsing repeated params into a stable map
- invalid preset name
- invalid param key
- invalid numeric param

**Step 2: Run test to verify it fails**

Run: `cargo test --test screener_parser_test -v`

Expected: FAIL because screener parser/module does not exist yet.

**Step 3: Write minimal implementation**

Create:

- `PresetKind`
- `PresetInvocation`
- `ScreenUniverse`
- parser function for `name:key=value,...`

Validation rules:

- only approved preset names exist
- only approved params per preset are accepted
- values must parse into expected numeric types

**Step 4: Run test to verify it passes**

Run: `cargo test --test screener_parser_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/screener/mod.rs src/screener/models.rs src/screener/parser.rs src/lib.rs tests/screener_parser_test.rs
git commit -m "feat: add phase22 screener preset parser"
```

### Task 3: Add preset evaluator logic

**Files:**
- Modify: `src/screener/models.rs`
- Create: `src/screener/evaluator.rs`
- Test: `tests/screener_evaluator_test.rs`

**Step 1: Write the failing test**

Add focused evaluator tests for:

- `close_above_ma`
- `close_below_ma`
- `rsi_gte`
- `rsi_lte`
- `volume_ratio_gte`
- insufficient lookback returns a non-match with reason

**Step 2: Run test to verify it fails**

Run: `cargo test --test screener_evaluator_test -v`

Expected: FAIL because evaluator does not exist yet.

**Step 3: Write minimal implementation**

- Compute only the last available rule value per stock
- Reuse existing indicator functions in `src/analysis/indicators.rs`
- Add small helper functions to determine lookback requirements per preset
- Return structured `RuleMatchDetail`

**Step 4: Run test to verify it passes**

Run: `cargo test --test screener_evaluator_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/screener/models.rs src/screener/evaluator.rs tests/screener_evaluator_test.rs
git commit -m "feat: add phase22 screener evaluator"
```

### Task 4: Add screener service for universe resolution and AND aggregation

**Files:**
- Create: `src/screener/service.rs`
- Modify: `src/screener/mod.rs`
- Test: `tests/screener_service_test.rs`

**Step 1: Write the failing test**

Cover:

- explicit `--codes` universe
- watchlist universe
- watchlist group filtering
- multi-preset AND behavior
- missing kline data does not crash whole run
- `limit` and sort application

Use fake data loader and fake name resolver traits in tests instead of hitting real ClickHouse.

**Step 2: Run test to verify it fails**

Run: `cargo test --test screener_service_test -v`

Expected: FAIL because service does not exist yet.

**Step 3: Write minimal implementation**

- Resolve codes from:
  - explicit CSV input
  - watchlist storage + watchlist service
- Calculate max lookback from all preset invocations
- Fetch daily candles per code through a thin trait-backed loader
- Evaluate all presets
- Apply AND aggregation
- Produce `ScreenRow`

**Step 4: Run test to verify it passes**

Run: `cargo test --test screener_service_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/screener/mod.rs src/screener/service.rs tests/screener_service_test.rs
git commit -m "feat: add phase22 screener service"
```

### Task 5: Wire screener into CLI handlers

**Files:**
- Modify: `src/cli/handlers.rs`
- Test: `tests/screener_handler_test.rs`

**Step 1: Write the failing test**

Cover:

- `preset-list` prints available presets
- `run --codes ... --preset ...` succeeds
- `run --watchlist --group ... --preset ...` resolves watchlist source
- invalid preset returns user-facing error

Use temporary watchlist storage and fakes for screener data loading where possible.

**Step 2: Run test to verify it fails**

Run: `cargo test --test screener_handler_test -v`

Expected: FAIL because handler dispatch is missing.

**Step 3: Write minimal implementation**

- Add `run_screener_command`
- Add thin CLI print helpers
- Keep handler logic thin; delegate parsing/evaluation work to the new `screener` module

**Step 4: Run test to verify it passes**

Run: `cargo test --test screener_handler_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add src/cli/handlers.rs tests/screener_handler_test.rs
git commit -m "feat: wire phase22 screener into cli"
```

### Task 6: Add end-to-end regression coverage and docs

**Files:**
- Modify: `README.md`
- Modify: `docs/USER_MANUAL.md`
- Test: `tests/screener_handler_test.rs`

**Step 1: Write the failing test**

If needed, extend handler/integration coverage to confirm:

- multiple presets combine with AND
- watchlist group universe works
- missing data path is readable and non-crashing

**Step 2: Run test to verify it fails**

Run: `cargo test --test screener_handler_test -v`

Expected: FAIL before the final coverage/doc adjustments.

**Step 3: Write minimal implementation**

- Add concise user-facing docs for:
  - preset list
  - repeated `--preset`
  - `--codes` vs `--watchlist`
- Keep docs limited to actual P0 support

**Step 4: Run test to verify it passes**

Run: `cargo test --test screener_handler_test -v`

Expected: PASS

**Step 5: Commit**

```bash
git add README.md docs/USER_MANUAL.md tests/screener_handler_test.rs
git commit -m "docs: document phase22 screener p0"
```

### Task 7: Full verification

**Files:**
- No intended code changes

**Step 1: Run targeted screener tests**

Run:

```bash
cargo test --test screener_parser_test -v
cargo test --test screener_evaluator_test -v
cargo test --test screener_service_test -v
cargo test --test screener_handler_test -v
```

Expected: all PASS

**Step 2: Run full repository verification**

Run:

```bash
cargo test --all-targets
```

Expected: PASS

**Step 3: Final commit if needed**

If verification required non-behavioral fixes:

```bash
git add <changed-files>
git commit -m "test: stabilize phase22 screener verification"
```
