Graphiti backfill required

Date: 2026-06-06
Repository: /opt/claude/quantix-rust
Branch: master
Commit: 67361f2891cb39457b539ecbad39e4b45fac0e94

Graphiti episodes queued but still processing:
- quantix_rust_debug: 2fe9dc73-fb44-44ab-a357-fe13668114eb
- quantix_rust_handoff: 3afbe93b-5a9c-48e3-8463-6484ed9aa6ad

Summary:
The independent import-klines --all/--exchange closure line is complete through commit 67361f2.
Previous commit 9cac8a7 added CLI parser tests for import-klines and fixed the ClickHouse row derive by adding clickhouse::Row where required.
During the interrupted tdx-api source test gate, cargo test --lib sources::tdx_api::tests initially failed to compile in src/cli/handlers/tdx_api_handler.rs ImportTicks.

Root cause:
ImportTicks accessed TradeResp.list and TradeItem fields from src/sources/tdx_api.rs while TradeResp/TradeItem were private, and the branch still used stale AppConfig::load() plus direct access to optional database.tdengine.

Fix:
Commit 67361f2 made TradeResp/TradeItem and required fields public to match the KlineResp source API style, changed ImportTicks to AppConfig::load("config"), and returns QuantixError::Config when TDengine config is missing.

Verification passed:
- cargo test --lib sources::tdx_api::tests: 7/7
- cargo test --lib cli::tests::data: 11/11
- cargo test --test bridge_tdx_source_test: 2/2
- cargo fmt --check
- git diff --check
- GitNexus impact before edits: run_tdx_api_command LOW; TradeResp LOW; TradeItem LOW
- GitNexus staged detect before commit: fresh_for_staged_diff=true, stale=false
- local gitnexus analyze after commit: success
- final gitnexus detect_changes(scope=all): changed_files=0, risk_level=none, stale=false

Worktree note:
Tracked files are clean. The only remaining worktree item is the untracked REST design-review document:
docs/superpowers/specs/2026-06-05-tdx-api-rest-source-design-review.md
Do not mix that separate REST review line into import-klines closure.
