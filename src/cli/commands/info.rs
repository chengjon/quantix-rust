use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NotifyCommands {
    /// 发送测试通知
    Test {
        /// 指定渠道 (all | telegram | wechat_work | feishu | discord | slack | dingtalk | pushplus)
        #[arg(long, default_value = "all")]
        channel: String,

        /// 自定义测试消息
        #[arg(short, long)]
        message: Option<String>,
    },

    /// 发送自定义通知
    Send {
        /// 通知标题
        #[arg(short, long)]
        title: String,

        /// 通知内容
        #[arg(short = 'm', long)]
        message: String,

        /// 通知级别 (info | warning | error | critical)
        #[arg(long, default_value = "info")]
        level: String,

        /// 指定渠道 (可选，不指定则使用配置的默认渠道)
        #[arg(long)]
        channel: Option<String>,
    },

    /// 列出可用渠道
    List,

    /// 测试渠道连通性
    Check {
        /// 渠道名称
        #[arg(long)]
        channel: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AiCommands {
    /// AI 分析股票
    Analyze {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 使用的模型 (deepseek | openai | ollama)
        #[arg(short, long, default_value = "deepseek")]
        model: String,

        /// 是否包含新闻分析
        #[arg(long)]
        with_news: bool,
    },

    /// AI 交易决策
    Decide {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 当前持仓数量
        #[arg(long)]
        position: Option<i64>,

        /// 风险等级 (low | medium | high)
        #[arg(long, default_value = "medium")]
        risk: String,
    },

    /// 交互式问答
    Ask {
        /// 问题内容
        #[arg(short, long)]
        question: String,

        /// 相关股票代码 (可选)
        #[arg(short, long)]
        code: Option<String>,

        /// 使用的模型
        #[arg(short, long, default_value = "deepseek")]
        model: String,
    },

    /// 市场整体分析
    Market {
        /// 分析日期 (YYYYMMDD，默认今天)
        #[arg(short, long)]
        date: Option<String>,
    },

    /// AI 配置管理
    Config {
        /// 显示当前配置
        #[arg(long)]
        show: bool,

        /// 测试 API 连接
        #[arg(long)]
        test: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum NewsCommands {
    /// 搜索新闻
    Search {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,

        /// 相关股票代码
        #[arg(short, long)]
        code: Option<String>,

        /// 时间范围（天数）
        #[arg(short, long, default_value = "3")]
        days: u32,

        /// 最大结果数
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,

        /// 指定提供商 (tavily | serpapi | bocha)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// 按股票代码搜索新闻
    Code {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 时间范围（天数）
        #[arg(short, long, default_value = "3")]
        days: u32,

        /// 最大结果数
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },

    /// 新闻趋势分析
    Trend {
        /// 日期 (YYYYMMDD，默认今天)
        #[arg(short, long)]
        date: Option<String>,

        /// 股票代码 (可选)
        #[arg(short, long)]
        code: Option<String>,
    },

    /// 列出可用的新闻提供商
    Providers,
}

#[derive(Subcommand, Debug)]
pub enum FundamentalCommands {
    /// 显示基本面数据
    Show {
        /// 股票代码
        #[arg(short, long)]
        code: String,
    },

    /// 查看估值指标
    Valuation {
        /// 股票代码
        #[arg(short, long)]
        code: String,
    },

    /// 查看财报数据
    Earnings {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 年数
        #[arg(short, long, default_value = "3")]
        years: u32,
    },

    /// 查看机构持仓
    Institution {
        /// 股票代码
        #[arg(short, long)]
        code: String,
    },

    /// 查看龙虎榜
    DragonTiger {
        /// 股票代码 (可选，不指定则显示今日龙虎榜)
        #[arg(short, long)]
        code: Option<String>,

        /// 天数
        #[arg(short, long, default_value = "5")]
        days: u32,
    },

    /// 查看分红信息
    Dividend {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 年数
        #[arg(short, long, default_value = "5")]
        years: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum SentimentCommands {
    /// 显示舆情数据
    Show {
        /// 股票代码
        #[arg(short, long)]
        code: String,
    },

    /// 查看历史趋势
    History {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 天数
        #[arg(short, long, default_value = "7")]
        days: u32,
    },

    /// 查看社交媒体提及
    Mentions {
        /// 股票代码
        #[arg(short, long)]
        code: String,

        /// 最大数量
        #[arg(short = 'n', long, default_value = "20")]
        max: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum ImportCommands {
    /// 从图片识别股票代码 (需要 LLM Vision API)
    FromImage {
        /// 图片文件路径
        #[arg(short, long)]
        file: String,

        /// 指定 Vision 模型 (deepseek | openai)
        #[arg(short, long, default_value = "deepseek")]
        model: String,
    },

    /// 从 CSV 文件导入股票列表
    FromCsv {
        /// CSV 文件路径
        #[arg(short, long)]
        file: String,
    },

    /// 从剪贴板文本导入
    FromClipboard,

    /// 从文本解析股票代码/名称
    FromText {
        /// 文本内容 (股票代码或名称，逗号/空格/换行分隔)
        #[arg(short, long)]
        text: String,
    },

    /// 解析股票名称/代码
    Resolve {
        /// 输入文本 (代码或名称)
        #[arg(short, long)]
        input: String,
    },
}
