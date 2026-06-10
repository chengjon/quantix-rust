# Review: TDX_API_BRIDGE_SUMMARY_2026-06-06.md

**Type**: `.md` / proposal | **Perspective**: completeness + consistency + feasibility | **Date**: 2026-06-06 | **Reviewer**: Codex

## Summary

这份总结的大部分实现面引用能和当前仓库对上：引用的 10 个文件均存在，核心符号能检索到，`TdxApiCommands` 当前确实有 18 个子命令，`cargo test --lib` 也能复现 695 passed。

但它还不适合作为最终 closeout/source-of-truth：当前 `cargo test -p quantix-cli` 结论与 live gate 相反，提交数量与实际提交范围不一致，技术债和 clippy 数量的口径不清。建议修订后再归档。

## Verified

- L1 file references: `config/holidays.json`, `docker-compose.yml`, `scripts/daily-update.sh`, `src/cli/commands/data.rs`, `src/cli/handlers/tdx_api_handler.rs`, `src/core/trading_calendar.rs`, `src/db/clickhouse/kline.rs`, `src/db/tdengine.rs`, `src/sources/tdx_api.rs`, `src/tasks/collect_scheduler.rs` all exist.
- Named symbols: `TdxApiClient`, `TradingCalendar`, `collect_once`, `insert_kline_data_batch_with_source`, `get_latest_kline_date`, `DataSource`, `deserialize_null_default`, `health`, and `init_market_cli` were found in the repo.
- CLI command count: `src/cli/commands/data.rs` defines 18 `TdxApiCommands` variants.
- `cargo test --lib --quiet`: live run returned exit status 0 with `695 passed`.
- `src` TODO count: live `rg 'TODO|todo!' src` returned 0.
- clippy command status: `cargo clippy --all-targets --all-features --message-format short` returned exit status 0, but its warning scope is broader than the report states.

## Issues

- [ ] **[HIGH]** `cargo test -p quantix-cli` is reported as passing, but the current live gate fails — source lines 150-155.

  Evidence: running `cargo test -p quantix-cli --quiet` on 2026-06-06 returned `EXIT_STATUS=101`. The failing test was `main_workspace_status_bearing_docs_defer_to_function_tree_registry` in `repo_hygiene_test`; output reported `87 passed; 1 failed` and required `docs/guides/TDX_API_BRIDGE_GUIDE.md` to point readers to `FUNCTION_TREE.md`. Internal validation: the report's "当前测试状态" table gives an unconditional `121 passed` and does not include a timestamp, commit, command output, or known-failure note elsewhere.

  Recommendation: either fix the hygiene failure and rerun the suite, or change the table to the actual current status with command, commit, timestamp, and failing test name.

- [ ] **[MED]** The commit count/list is incomplete for the apparent implementation range — source lines 94-113.

  Evidence: all listed hashes exist, but `git rev-list --count 6b4b285^..HEAD` returns 21 commits, while the report says 19 and lists 16 hashes plus `+ 3 earlier docs/fix commits`. In that range, the report omits commits including `da256cf`, `67361f2`, `ade14e5`, `d0c0da9`, and `2d0a478`. Internal validation: the section does not state whether some commits are intentionally excluded by scope, such as docs-only backfills or intermediate fixes.

  Recommendation: define the exact range and inclusion rule, then either list all 21 commits or rename the section to "主要提交记录" and remove the exact count.

- [ ] **[MED]** The "unwrap 清零" debt wording is ambiguous and conflicts with current source-level evidence if read literally — source line 73.

  Evidence: `rg '\bunwrap\s*\(' src` finds 1118 matches. A heuristic pass excluding `tests.rs`, `tests/`, and `#[cfg(test)] mod tests` still found 84 source matches, including `src/db/tdengine.rs:119`, `src/db/clickhouse/kline.rs:129`, and `src/tasks/scheduler.rs:100`. Internal validation: line 73 says "CLAUDE.md 技术债：4 项标记为已解决" but the parenthetical says "unwrap 清零" without scope. The `TODO` part is supported for `src` because `rg 'TODO|todo!' src` returned 0.

  Recommendation: rewrite this as "CLAUDE.md 中对应技术债条目已标记 resolved" or add a scoped metric such as "特定历史清单清零"; do not imply repo-wide or production-source `unwrap` count is zero unless that gate is actually true.

- [ ] **[LOW]** The clippy warning count is technically true only for one summary line, not for all-targets output — source lines 126 and 158.

  Evidence: live `cargo clippy --all-targets --all-features --message-format short` returned exit status 0 and included `quantix-cli (lib) generated 146 warnings`, but the same output also included other target summaries, including `quantix-cli (lib test) generated 215 warnings (142 duplicates)` and 241 lines containing `warning:`. Internal validation: the report says simply `clippy | 146 warnings` and the next plan says "清理 146 个 clippy warnings" without naming the exact clippy command or scope.

  Recommendation: state the scope explicitly, for example "`cargo clippy --lib` generated 146 warnings" or give a separate all-targets warning summary.

- [ ] **[LOW]** The code-size table uses stale or unclear line-count semantics — source lines 78-90.

  Evidence: current file line counts differ materially from several entries: `src/sources/tdx_api.rs` is 1355 lines, not `~1100`; `src/cli/commands/data.rs` is 323 lines, not `~240`; `src/core/trading_calendar.rs` is 616 total lines while the table says `+30`. Internal validation: the table heading is "行数", but rows mix total approximate line count (`~470`, `22`) and delta notation (`+40`, `+70`, `+30`) without saying which metric is used.

  Recommendation: split the table into "current file LOC" and "estimated delta", or change the column name to "规模/变更量" and keep all rows in the same unit.

- [ ] **[LOW]** The live URL wording can be misread as the runtime default — source lines 12 and 60.

  Evidence: current code uses `DEFAULT_BASE_URL = "http://tdx-api:8080"` in `src/sources/tdx_api.rs` and `TDX_API_URL` in `src/core/config.rs`; `docker-compose.yml` sets `TDX_API_URL=http://tdx-api:8080`. The live smoke-test URL `http://192.168.123.104:8089` appears in design/test context, not as the runtime default. Internal validation: line 12 says the REST client connects to `http://192.168.123.104:8089`, while line 60 separately says Docker uses `http://tdx-api:8080`.

  Recommendation: change line 12 to "live 验证使用 `http://192.168.123.104:8089`；默认运行时通过 `TDX_API_URL` 配置，Docker 默认 `http://tdx-api:8080`."

## Checklist Results

| # | Check | Result | Notes |
|---|-------|--------|-------|
| C1 | Required sections | PASS | Has completed work, code stats, commit log, next steps, and test status. |
| C2 | Edge cases | PARTIAL | Pending DB-backed E2E tests are listed, but current CLI hygiene failure is absent. |
| C3 | Implicit assumptions | FAIL | Test, clippy, commit-count, URL, and debt metrics lack explicit scope. |
| C4 | Acceptance criteria | PARTIAL | Some commands are objectively verifiable; the current test table is contradicted by live evidence. |
| C5 | Missing roles/stakeholders | N/A | This is a technical closeout summary, not a stakeholder plan. |
| N1 | Terminology | PASS | Uses tdx-api/bridge/TDX terms consistently enough for this report. |
| N2 | Naming conventions | PASS | Referenced file and symbol names match repo conventions. |
| N3 | Formatting | PASS | Heading hierarchy and tables are readable. |
| N4 | Cross-references | PARTIAL | File references resolve; numeric/count references need scope fixes. |
| N5 | Style consistency | PASS | Chinese technical summary style is consistent. |
| F1 | Technical risk | PARTIAL | Major remaining E2E risks are listed, but the failing CLI hygiene gate is missing. |
| F2 | Dependency availability | PASS | Referenced local modules and files exist. |
| F3 | Timeline realism | N/A | No estimates are provided. |
| F4 | Resource constraints | N/A | No staffing/resource claims are provided. |
| F5 | Rollback plan | N/A | Not expected for a completion summary. |

## Suggestions

- Add a small "Verification Evidence" table with command, commit hash, date/time, exit status, and log path for each gate.
- Make all numeric claims scoped: commit range, clippy command, warning scope, code LOC source, and `unwrap`/`TODO` search scope.
- Re-run `cargo test -p quantix-cli --quiet` after fixing `docs/guides/TDX_API_BRIDGE_GUIDE.md` hygiene expectations, then update the status table.
- Consider moving unresolved operational requirements, such as DB-backed E2E tests and clippy cleanup, into a follow-up issue list rather than leaving them only in this report.

## Verdict

NEEDS_REVISION — implementation references mostly align with the repo, but the current test-status claim is false and several exact numeric/status claims need scope corrections before this can be treated as an accurate closeout document.
