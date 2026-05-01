# Market Fundamentals JSON Template

这个目录提供 `quantix data import-fundamentals --input <json>` 所需的输入模板。

模板文件：

- `market_fundamentals.template.json`

## 谁提供这份 JSON

当前 `quantix` CLI 只负责导入，不负责自动抓取全市场基本面。

所以这份 JSON 应由上游数据链路提供，例如：

- 你现有的数据库导出脚本
- 第三方终端 / 数据商导出的中间文件
- Excel / CSV 清洗后的转换脚本
- 手工维护的临时快照

## 目标结构

顶层必须是 JSON 数组，每个元素对应一个 `MarketFundamentalSyncRecord`：

```json
{
  "code": "600519",
  "snapshot_date": "2026-03-14",
  "market_cap": 23000.5,
  "latest_report_profit": 862.1,
  "profit_source": "report",
  "pe_dynamic": 27.4
}
```

## 字段说明

| 字段 | 必填 | 类型 | 说明 |
| --- | --- | --- | --- |
| `code` | 是 | string | 6 位股票代码，建议保留前导零 |
| `snapshot_date` | 是 | string | 快照日期，格式必须是 `YYYY-MM-DD` |
| `market_cap` | 否 | number/null | 总市值，建议统一使用“亿元” |
| `latest_report_profit` | 否 | number/null | 上一会计周期净利润，建议统一使用“亿元” |
| `profit_source` | 是 | string | 利润来源标记，如 `report` / `manual` / `eastmoney` |
| `pe_dynamic` | 否 | number/null | 动态市盈率 |

## 强约束

- 顶层必须是 `[]`
- `snapshot_date` 只能用 `YYYY-MM-DD`
- 数值字段必须是 JSON number，不能写成字符串
- 可空字段请用 `null`
- `code` 必须是字符串，不能写成数字

## 常见字段映射

如果你的原始数据来自 CSV / Excel，建议按下面映射转换：

| 目标字段 | 常见上游列名 |
| --- | --- |
| `code` | `code` / `股票代码` / `证券代码` / `ts_code` |
| `snapshot_date` | `snapshot_date` / `快照日期` / `交易日期` / `date` |
| `market_cap` | `market_cap` / `总市值` / `市值(亿)` / `总市值(亿)` |
| `latest_report_profit` | `latest_report_profit` / `归母净利润` / `净利润` / `最新报告净利润(亿)` |
| `profit_source` | `profit_source` / `利润来源` / `profit_origin` |
| `pe_dynamic` | `pe_dynamic` / `动态市盈率` / `市盈率TTM` / `PE_TTM` |

## 单位要求

建议统一成下面口径：

- `market_cap`: 亿元
- `latest_report_profit`: 亿元
- `pe_dynamic`: 倍

如果上游不是这个单位，先换算再导入。否则排序可能对，但展示值会失真。

例子：

- 如果上游总市值是“元”，则先除以 `100000000`
- 如果上游净利润是“元”，则先除以 `100000000`
- 如果上游市值已经是“亿”，可以直接写入

## 推荐导入流程

1. 先用 scratch ClickHouse 演练，不碰正式库：

```bash
CLICKHOUSE_URL=http://192.168.123.104:8123 \
CLICKHOUSE_DB=quantix \
CLICKHOUSE_USER=default \
CLICKHOUSE_PASSWORD=c790414J \
MARKET_FUNDAMENTALS_REHEARSAL_INPUT=/abs/path/market_fundamentals.json \
scripts/dev/run_market_cli_import_fundamentals_rehearsal.sh
```

2. rehearsal 通过后，再导入正式库：

```bash
cargo run --bin quantix -- data import-fundamentals --input /abs/path/market_fundamentals.json
```

3. 导入完成后，验证市场强弱 CLI：

```bash
cargo run --bin quantix -- market strength --date 2026-03-14 --strong-top 3 --weak-top 3 --stock-top 10
cargo run --bin quantix -- market strength-stocks --date 2026-03-14 --strong-top 3 --metric profit --top 10
scripts/dev/run_market_cli_delivery_gate.sh
```

## 最小交付要求

如果你现在只想尽快打通正式链路，最低要求是：

- `snapshot_date` 覆盖目标交易日
- `code` 覆盖全市场，或者至少覆盖你要观察的强势板块成分股
- `market_cap` 和 `latest_report_profit` 尽量非空
- 全文件是合法 JSON

## CSV 表头建议

如果你要让外部同事提供 CSV，再转 JSON，建议要求 CSV 至少包含这 6 列：

```text
code,snapshot_date,market_cap,latest_report_profit,profit_source,pe_dynamic
```

这样转换成本最低，也最不容易丢字段。
