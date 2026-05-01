# Market CLI Acceptance Report

生成时间: 2026-04-27 10:33:26 CST

## 日志来源

- acceptance orchestrator: /opt/claude/quantix-rust/logs/run_market_cli_acceptance_20260426_123458.log
- precheck: /opt/claude/quantix-rust/logs/check_market_cli_prereqs_20260427_094836.log
- smoke: /opt/claude/quantix-rust/logs/verify_market_cli_smoke_20260426_123459.log
- formal sequence: /opt/claude/quantix-rust/logs/market_cli_formal_sequence_20260427_101224.log

## 摘要

- precheck: PASS=7 WARN=0 FAIL=0
- smoke: PASS=6 WARN=3 FAIL=0
- formal:
  - sync industry exit=0 log=/opt/claude/quantix-rust/logs/market_cli_sync_industry_20260427_101224.log
    - summary: exit=0 completed; see log for refreshed industry reference details
  - market foundation exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_foundation_20260427_101224.log
    - summary: A股总数=4202 已匹配行业=4202 未匹配行业=0 行业数=31
    - total_stocks: 4202
    - classified_stocks: 4202
    - unclassified_stocks: 0
    - sector_count: 31
    - top_sector: 1 机械设备 417
  - market strength exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_20260427_101224.log
    - summary: 基础数据=A股=4203 行业覆盖=4203 未覆盖=0; 候选股数=40; 快照来源=tdx_fallback; TDX覆盖=4203/4430; 强势首行=1 BK0001 银行 2.35%; 弱势首行=1 BK0001 银行 2.35%; 总市值首行=N/A; 净利润首行=N/A
    - base: A股=4203 行业覆盖=4203 未覆盖=0
    - candidate_stock_count: 40
    - snapshot_source: tdx_fallback
    - tdx_coverage: 4203/4430
    - top_strong_sector: 1 BK0001 银行 2.35%
    - top_weak_sector: 1 BK0001 银行 2.35%
    - top_market_cap_stock: N/A
    - top_profit_stock: N/A
  - market strength-stocks exit=0 log=/opt/claude/quantix-rust/logs/market_cli_market_strength_stocks_20260427_101224.log
    - summary: 行业过滤=银行; 指标=上一会计周期净利润; 覆盖=10/10; 首行=1 银行 600036 招商银行 39.87 1501.80
    - sector_filter: 银行
    - metric: 上一会计周期净利润
    - coverage: 10/10
    - top_row: 1 银行 600036 招商银行 39.87 1501.80

## 当前判定

- 如果 precheck 或 smoke 的 FAIL 大于 0，应先修复 CLI/脚本问题后再继续。
- 如果只有 WARN，请优先查看 precheck 日志中的 `[REMEDIATION]` 段落。
- 当 warning 收敛到可接受范围后，再执行正式命令链路：
  - `quantix risk sync industry --standard shenwan`
  - `quantix market foundation`
  - `quantix market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10`
  - `quantix market strength-stocks --date 2026-03-14 --strong-top 3 --sector 银行 --metric profit --top 10`

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
