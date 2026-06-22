# AuctionCollector Offline Test Seam Graphiti Backfill

Graphiti backfill required.

## Local Summary

Master CI run `27923085656` failed after docs-only PR #263.

Failure:

- Job: Test
- Failed test: `sources::auction_collector::tests::test_auction_collector_creation`
- Local reproduction: `cargo test --lib sources::auction_collector::tests::test_auction_collector_creation -- --test-threads=1`
- Root cause: the unit test called `AuctionCollector::new()`, which calls `rustdx_complete::tcp::Tcp::new()` and attempts live external TDX TCP connectivity to a fixed stock IP.

Hotfix:

- PR #264 changed only `src/sources/auction_collector.rs`.
- The test now verifies the deterministic default watchlist instead of constructing a live `AuctionCollector`.
- Production `AuctionCollector::new()` and `with_watchlist()` behavior is unchanged.
- Function Tree node `auction-collector-offline-test-seam` is closed.

Verification:

- `cargo test --lib sources::auction_collector::tests::test_auction_collector_creation -- --test-threads=1`
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --lib --all-features -- --test-threads=1`
- `git diff --check`
- Function Tree `gate --verbose`
- Function Tree `validate`
- GitNexus detect_changes/compare: LOW, 0 affected processes
- PR #264 CI passed
- master CI run `27924014928` passed

Graphiti ingest status:

- `add_memory` queued episode `fc2be8c2-68dc-47e9-8b7d-6435a2cd4b4e` in `quantix_rust_debug`.
- `get_ingest_status` returned `failed`.
- Error: `Request timed out.`
- Error code: `apitimeouterror`.
- This file is the required local backfill record until Graphiti can be retried.
