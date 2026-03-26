# Changelog

All notable changes to this project are documented here.

## 2026-03-26

### Added

- Added `docs/FUNCTION_MAP.md` to record the current completed functional design and system-level function tree.
- Added Windows Bridge v1 integration on the Rust side:
  - `src/bridge/*` HTTP client, models, and error layer
  - `src/sources/bridge_tdx.rs` for `TDX bridge source`
  - `src/execution/qmt_bridge.rs` for `QMT preview-only` request previewing
  - `quantix execution bridge status`
  - `quantix execution bridge qmt-preview --request-id <ID>`
- Added bridge-focused test coverage:
  - `tests/bridge_client_test.rs`
  - `tests/bridge_tdx_source_test.rs`
  - `tests/watchlist_bridge_lookup_test.rs`
  - `tests/qmt_bridge_preview_test.rs`

### Changed

- Merged the strategy/execution prerequisite branch chain required by the bridge work into local `master`.
- Updated `README.md` and `docs/USER_MANUAL.md` to reflect the current completed tasks, execution boundaries, and Windows Bridge v1 operator workflow.
- Updated architecture and implementation-plan docs to use the canonical Windows-side path:
  - `/mnt/d/mystocks/quantix/quantix_bridge`

### Completed Design State

- `quantix-rust` continues to own:
  - `execution_request`
  - frozen execution snapshots
  - `ExecutionKernel`
  - `runtime.db`
  - paper/mock-live execution state
- Windows Bridge v1 currently completes:
  - `TDX bridge source`
  - `QMT preview-only`
- Real live broker execution remains deferred.
