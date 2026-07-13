//! `data openstock` 子命令枚举。
//!
//! 从原生 `commands/data.rs` 拆出，承载 [`OpenStockCommands`]。
//! 所有操作均为 dry-run 或只读；真正写库需同时传对应 `--apply` 与
//! 环境变量（参见各变体文档）。

use clap::Subcommand;

use crate::cli::command_types::data::OutputFormat;

/// data openstock 子命令枚举：ValidateFixture 本地 fixture 校验、ValidateLive 线上响应校验（dry-run，不联网不写库）。
#[derive(Subcommand, Debug)]
pub enum OpenStockCommands {
    /// 校验本地 OpenStock K 线 fixture
    ValidateFixture {
        /// 本地 fixture JSON 文件路径
        #[arg(long)]
        file: String,
    },

    /// 校验通过外部捕获的 OpenStock /data/bars 线上响应（dry-run，不联网，不写库）
    ValidateLive {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的代码（symbol/code）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的周期（必须为 daily）
        #[arg(long, default_value = "daily")]
        r#period: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// 请求时声明的 limit（如填写将与返回条数对比，检测服务端是否裁剪）
        #[arg(long)]
        limit: Option<u32>,
    },

    /// 将已校验的 live shadow 载荷写入 `quantix_shadow.openstock_daily_kline_shadow`。
    /// 默认为 dry-run；如需真正写入，必须同时传 `--apply` 与
    /// 环境变量 `QUANTIX_SHADOW_PERSIST_CONFIRM=yes`。
    PersistLive {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的代码（symbol/code）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的周期（必须为 daily）
        #[arg(long, default_value = "daily")]
        r#period: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: String,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: String,

        /// 请求时声明的 limit
        #[arg(long)]
        limit: Option<u32>,

        /// 真正执行写入；不传则只跑 dry-run gate
        #[arg(long)]
        apply: bool,
    },

    /// 按 `batch_id` 回滚（删除）shadow 行，幂等。
    ShadowRollback {
        /// `persist-live --apply` 返回的 batch_id
        #[arg(long)]
        batch_id: String,
    },

    /// 校验某 `batch_id` 在 shadow 表中的当前行数。
    ShadowVerify {
        /// `persist-live --apply` 返回的 batch_id
        #[arg(long)]
        batch_id: String,
    },

    /// 校验 STOCK_CODES / ALL_STOCKS 类目捕获载荷（dry-run，不联网，不写库）
    ValidateCodes {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 类目：`codes` 或 `all_stocks`，缺省按 `codes` 处理
        #[arg(long)]
        kind: Option<String>,
    },

    /// 校验 TRADE_DATES / WORKDAYS 类目捕获载荷（dry-run，不联网，不写库）
    ValidateCalendar {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 类目：`trade_dates` 或 `workdays`
        #[arg(long)]
        kind: String,
    },

    /// 校验 INDEX_KLINES 类目捕获载荷（dry-run，不联网，不写库）
    ValidateIndex {
        /// 已捕获的响应 JSON 文件路径；使用 `-` 从 stdin 读取
        #[arg(long)]
        payload: String,

        /// 请求时使用的指数代码（如 sh000001）
        #[arg(long)]
        symbol: String,

        /// 请求时使用的开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 请求时使用的结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取 STOCK_CODES 类目（联网，只读，不写库）
    FetchCodes,

    /// 实时拉取 TRADE_DATES 类目（联网，只读，不写库）
    ///
    /// runtime 实际接受 `start_date`/`end_date`（YYYY-MM-DD），忽略
    /// `year`。为便于使用，`--year` 仍保留并自动展开为当年 1-1..12-31
    /// 范围；与 `--start/--end` 二选一。
    FetchCalendar {
        /// 年份（如 2026），展开为 `<year>-01-01`..`<year>-12-31`
        #[arg(long, group = "range")]
        year: Option<u32>,

        /// 起始日期 (YYYY-MM-DD)，与 --end 同时使用
        #[arg(long, group = "range")]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)，与 --start 同时使用
        #[arg(long, group = "range")]
        end: Option<String>,
    },

    /// 实时拉取 INDEX_KLINES 类目（联网，只读，不写库）
    FetchIndex {
        /// 指数代码（如 sh000001）
        #[arg(long)]
        symbol: String,

        /// 开始日期 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取 ALL_STOCKS 类目（baostock 全市场快照，联网，只读，不写库）
    FetchAllStocks {
        /// 可选日期 (YYYY-MM-DD)；缺省时服务端回退到最近交易日
        #[arg(long)]
        day: Option<String>,
    },

    /// 实时拉取 WORKDAYS 类目（eltdx action 驱动，联网，只读，不写库）
    FetchWorkdays {
        /// 操作类型：today / today_is_workday / is_workday / range / next_workday / previous_workday
        #[arg(long, default_value = "today")]
        action: String,

        /// YYYY-MM-DD，用于 is_workday / next_workday / previous_workday
        #[arg(long)]
        date: Option<String>,

        /// YYYY-MM-DD，range 起始（含），与 --end 同时使用
        #[arg(long)]
        start: Option<String>,

        /// YYYY-MM-DD，range 结束（含），与 --start 同时使用
        #[arg(long)]
        end: Option<String>,
    },

    /// 实时拉取多周期 K 线（/data/bars，支持 day/week/month + none/qfq/hfq，联网，只读，不写库）
    FetchKlines {
        /// 标的代码（例如 600000、sh600000）
        #[arg(long)]
        symbol: String,

        /// K 线周期：day | week | month（大小写不敏感，拒绝 daily/weekly/monthly 别名）
        #[arg(long, default_value = "day")]
        period: String,

        /// 复权类型：none | qfq | hfq（大小写不敏感）
        #[arg(long, default_value = "none")]
        adjust: String,

        /// YYYY-MM-DD，起始日期（含，可选）
        #[arg(long)]
        start: Option<String>,

        /// YYYY-MM-DD，结束日期（含，可选）
        #[arg(long)]
        end: Option<String>,
    },

    /// 拉取分钟级 K 线蜡烛 (P0.13b-1, OpenStock /data/bars period=1m|5m|15m|30m|60m)
    FetchMinuteKlines {
        #[arg(long)]
        symbol: String,

        #[arg(long, default_value = "1m")]
        period: String,

        /// Single-day query (mutex with --start/--end)
        #[arg(long)]
        date: Option<String>,

        /// Range start (inclusive). Must pair with --end.
        #[arg(long)]
        start: Option<String>,

        /// Range end (inclusive). Must pair with --start.
        #[arg(long)]
        end: Option<String>,

        #[arg(long, default_value = "none")]
        adjust: String,

        /// Stream batches (P0.13d). Emits per-batch progress to stderr.
        /// Default false; when absent, batch API behavior is unchanged.
        #[arg(long, default_value_t = false)]
        stream: bool,
    },

    /// Fetch OpenStock MINUTE_DATA category (intraday time-share ticks).
    ///
    /// Returns per-minute price + avg_price for the given code/date.
    /// Distinct from fetch-minute-klines: no OHLC, no period/adjust.
    FetchMinuteShare {
        #[arg(long)]
        symbol: String,

        /// Single-day query (mutex with --start/--end)
        #[arg(long)]
        date: Option<String>,

        /// Range start (inclusive). Must pair with --end.
        #[arg(long)]
        start: Option<String>,

        /// Range end (inclusive). Must pair with --start.
        #[arg(long)]
        end: Option<String>,

        /// Stream batches (P0.13d). Emits per-batch progress to stderr.
        /// Default false; when absent, batch API behavior is unchanged.
        #[arg(long, default_value_t = false)]
        stream: bool,
    },

    /// Import minute klines into ClickHouse `minute_klines` (OpenStock).
    /// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
    ImportMinuteKlines {
        /// 股票代码 (e.g. sh600000)
        #[arg(short, long)]
        code: String,

        /// 周期: 1m / 5m / 15m / 30m / 60m
        #[arg(long, default_value = "1m")]
        period: String,

        /// 复权: none / qfq / hfq
        #[arg(long, default_value = "none")]
        adjust: String,

        /// 起始日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        end: Option<String>,

        /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
        #[arg(long, default_value_t = false)]
        apply: bool,
    },

    /// Import minute shares into ClickHouse `minute_shares` (OpenStock).
    /// Default is dry-run; pass --apply AND set QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to write.
    ImportMinuteShare {
        /// 股票代码 (e.g. sh600000)
        #[arg(short, long)]
        code: String,

        /// 起始日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        start: Option<String>,

        /// 结束日期 (YYYY-MM-DD, inclusive)
        #[arg(long)]
        end: Option<String>,

        /// 实际写入 ClickHouse (默认 dry-run; 需配合 QUANTIX_OPENSTOCK_MINUTE_APPLY=yes)
        #[arg(long, default_value_t = false)]
        apply: bool,
    },

    /// Import minute klines + shares for ALL active codes (P0.15b scheduler).
    /// Iterates `quantix.stock_info`, runs P0.15a import per code, tracks
    /// outcome in `quantix.import_state`. Default is dry-run; set
    /// QUANTIX_OPENSTOCK_MINUTE_APPLY=yes to actually write.
    ImportMinuteAll {
        /// Trade date to import. Defaults to today (Asia/Shanghai).
        /// Format: YYYY-MM-DD, or the literal "today".
        #[arg(long)]
        date: Option<String>,

        /// Output format for BatchSummary. Default: text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Dry-run: print plan (codes + batch_id) without writing.
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Query `quantix.import_state` for a date and print the latest batch
    /// summary (success/fail counts, failure detail) in text or json.
    ImportStatus {
        /// Trade date to query. Format: YYYY-MM-DD.
        #[arg(long)]
        date: String,

        /// Output format. Default: text.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },
}
