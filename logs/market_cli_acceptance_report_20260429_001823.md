# Market CLI Acceptance Report

生成时间: 2026-04-29 00:18:23 CST

## 日志来源

- acceptance orchestrator: /opt/claude/quantix-rust/logs/run_market_cli_acceptance_20260426_123458.log
- precheck: /opt/claude/quantix-rust/logs/check_market_cli_prereqs_20260429_001808.log
- smoke: /opt/claude/quantix-rust/logs/verify_market_cli_smoke_20260428_161515.log
- formal sequence: /opt/claude/quantix-rust/logs/market_cli_formal_sequence_20260429_001445.log

## 摘要

- precheck: PASS=6 WARN=2 FAIL=0
  - fundamentals_state: empty
  - fundamentals_rows: 0
  - fundamentals_latest_snapshot: N/A
- smoke: PASS=6 WARN=3 FAIL=0
- formal:
  - sync industry exit=0 log=/opt/claude/quantix-rust/logs/market_cli_sync_industry_20260429_001445.log
    - summary: exit=0 completed; see log for refreshed industry reference details
  - market foundation exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_foundation_20260429_001445.log
    - summary: A股总数=4226 已匹配行业=4226 未匹配行业=0 行业数=31
    - total_stocks: 4226
    - classified_stocks: 4226
    - unclassified_stocks: 0
    - sector_count: 31
    - top_sector: 1 机械设备 421
  - market strength exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_20260429_001445.log
    - summary: 基础数据=A股=4226 行业覆盖=4226 未覆盖=0; 候选股数=195; 快照来源=tdx_fallback; TDX覆盖=4226/4430; 强势首行=1 derived:煤炭 煤炭 2.41%; 弱势首行=31 derived:计算机 计算机 -2.22%; 总市值首行=N/A; 净利润首行=N/A
    - base: A股=4226 行业覆盖=4226 未覆盖=0
    - candidate_stock_count: 195
    - snapshot_source: tdx_fallback
    - tdx_coverage: 4226/4430
    - top_strong_sector: 1 derived:煤炭 煤炭 2.41%
    - top_weak_sector: 31 derived:计算机 计算机 -2.22%
    - top_market_cap_stock: N/A
    - top_profit_stock: N/A
  - market strength-stocks exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_stocks_20260429_001445.log
    - summary: 行业过滤=煤炭; 指标=上一会计周期净利润; 覆盖=0/35; 首行=📭 没有可展示的个股数据
    - sector_filter: 煤炭
    - metric: 上一会计周期净利润
    - coverage: 0/35
    - top_row: 📭 没有可展示的个股数据

## 当前判定

- 如果 precheck 或 smoke 的 FAIL 大于 0，应先修复 CLI/脚本问题后再继续。
- 如果只有 WARN，请优先查看 precheck 日志中的 `[REMEDIATION]` 段落。
- 如果 `fundamentals_state` 为 `missing` 或 `empty`，即使 TDX 快照链路可用，`market strength` / `market strength-stocks` 的总市值、净利润 TopN 仍可能输出为空。
- 当 warning 收敛到可接受范围后，再执行正式命令链路：
  - `quantix risk sync industry --standard shenwan`
  - `quantix market foundation`
  - `quantix market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10`
  - `quantix market strength-stocks --date 2026-03-14 --strong-top 3 --sector 煤炭 --metric profit --top 10`

## 建议补充记录

- 环境模板是否已加载：`source scripts/dev/market_cli_env.example.sh`
- precheck 主要 warning:
- smoke 主要 warning:
- 正式命令执行结果:
  - sync industry:
  - market foundation:
  - market strength:
  - market strength-stocks:
- 结论:
