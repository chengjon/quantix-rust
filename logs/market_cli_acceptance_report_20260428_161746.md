# Market CLI Acceptance Report

生成时间: 2026-04-28 16:17:46 CST

## 日志来源

- acceptance orchestrator: /opt/claude/quantix-rust/logs/run_market_cli_acceptance_20260426_123458.log
- precheck: /opt/claude/quantix-rust/logs/check_market_cli_prereqs_20260428_144941.log
- smoke: /opt/claude/quantix-rust/logs/verify_market_cli_smoke_20260428_161515.log
- formal sequence: /opt/claude/quantix-rust/logs/market_cli_formal_sequence_20260428_144957.log

## 摘要

- precheck: PASS=7 WARN=0 FAIL=0
- smoke: PASS=6 WARN=3 FAIL=0
- formal:
  - sync industry exit=0 log=/opt/claude/quantix-rust/logs/market_cli_sync_industry_20260428_144957.log
    - summary: exit=0 completed; see log for refreshed industry reference details
  - market foundation exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_foundation_20260428_144957.log
    - summary: A股总数=4209 已匹配行业=4209 未匹配行业=0 行业数=31
    - total_stocks: 4209
    - classified_stocks: 4209
    - unclassified_stocks: 0
    - sector_count: 31
    - top_sector: 1 机械设备 418
  - market strength exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_20260428_144957.log
    - summary: 基础数据=A股=4209 行业覆盖=4209 未覆盖=0; 候选股数=194; 快照来源=tdx_configured; TDX覆盖=4209/4430; 强势首行=1 derived:煤炭 煤炭 2.42%; 弱势首行=31 derived:计算机 计算机 -2.25%; 总市值首行=N/A; 净利润首行=N/A
    - base: A股=4209 行业覆盖=4209 未覆盖=0
    - candidate_stock_count: 194
    - snapshot_source: tdx_configured
    - tdx_coverage: 4209/4430
    - top_strong_sector: 1 derived:煤炭 煤炭 2.42%
    - top_weak_sector: 31 derived:计算机 计算机 -2.25%
    - top_market_cap_stock: N/A
    - top_profit_stock: N/A
  - market strength-stocks exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_stocks_20260428_144957.log
    - summary: 行业过滤=煤炭; 指标=上一会计周期净利润; 覆盖=0/35; 首行=📭 没有可展示的个股数据
    - sector_filter: 煤炭
    - metric: 上一会计周期净利润
    - coverage: 0/35
    - top_row: 📭 没有可展示的个股数据

## 当前判定

- 如果 precheck 或 smoke 的 FAIL 大于 0，应先修复 CLI/脚本问题后再继续。
- 如果只有 WARN，请优先查看 precheck 日志中的 `[REMEDIATION]` 段落。
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
