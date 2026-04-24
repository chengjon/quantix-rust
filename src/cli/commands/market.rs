use clap::{ArgGroup, Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum MarketCommands {
    /// 获取全市场 A 股与行业分类基础数据摘要
    Foundation,

    /// 行业板块排名
    Sector {
        /// 限制返回条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 排序字段
        #[arg(long)]
        sort_by: Option<String>,
    },

    /// 概念板块排名
    Concept {
        /// 限制返回条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 排序字段
        #[arg(long)]
        sort_by: Option<String>,
    },

    /// 北向资金概览
    North {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 市场情绪概览
    Sentiment {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 龙头股识别
    #[command(group(
        ArgGroup::new("leader_filter")
            .args(["sector", "concept", "all"])
            .required(true)
            .multiple(false)
    ))]
    Leader {
        /// 行业名称
        #[arg(long)]
        sector: Option<String>,

        /// 概念名称
        #[arg(long)]
        concept: Option<String>,

        /// 全市场龙头
        #[arg(long)]
        all: bool,

        /// 限制返回条数
        #[arg(long)]
        limit: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 市场综合概览
    Overview {
        /// 板块列表条数
        #[arg(long)]
        top: Option<usize>,

        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,
    },

    /// 分析强势/弱势行业板块，并输出强势板块个股 Top10
    Strength {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 强势板块数量
        #[arg(long, default_value_t = 3)]
        strong_top: usize,

        /// 弱势板块数量
        #[arg(long, default_value_t = 3)]
        weak_top: usize,

        /// 强势板块内个股 TopN
        #[arg(long, default_value_t = 10)]
        stock_top: usize,
    },

    /// 仅输出强势板块个股排行
    StrengthStocks {
        /// 指定交易日期
        #[arg(long)]
        date: Option<String>,

        /// 强势板块数量
        #[arg(long, default_value_t = 3)]
        strong_top: usize,

        /// 排名字段
        #[arg(long, value_enum, default_value_t = StrengthStockMetric::MarketCap)]
        metric: StrengthStockMetric,

        /// 返回条数
        #[arg(long, default_value_t = 10)]
        top: usize,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum StrengthStockMetric {
    MarketCap,
    Profit,
}

#[derive(Subcommand, Debug)]
pub enum WatchlistCommands {
    /// 添加股票到自选池
    Add {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 分组名称
        #[arg(long)]
        group: Option<String>,
    },

    /// 从自选池移除股票
    Remove {
        /// 股票代码
        #[arg(long)]
        code: String,
    },

    /// 列出自选池
    List {
        /// 分组过滤
        #[arg(long)]
        group: Option<String>,

        /// 标签过滤
        #[arg(long)]
        tag: Option<String>,

        /// 展示最佳努力价格
        #[arg(long)]
        with_price: bool,
    },

    /// 移动股票到目标分组
    Move {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 目标分组
        #[arg(long)]
        group: String,
    },

    /// 分组管理
    #[command(subcommand)]
    Group(WatchlistGroupCommands),

    /// 标签管理
    #[command(subcommand)]
    Tag(WatchlistTagCommands),

    /// 查看历史
    History {
        /// 股票代码
        #[arg(long)]
        code: Option<String>,

        /// 返回条数
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum WatchlistGroupCommands {
    /// 创建分组
    Create {
        /// 分组名称
        #[arg(long)]
        name: String,
    },

    /// 列出分组
    List,
}

#[derive(Subcommand, Debug)]
pub enum WatchlistTagCommands {
    /// 添加标签
    Add {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 标签
        #[arg(long)]
        tag: String,
    },

    /// 删除标签
    Remove {
        /// 股票代码
        #[arg(long)]
        code: String,

        /// 标签
        #[arg(long)]
        tag: String,
    },

    /// 列出股票标签
    List {
        /// 股票代码
        #[arg(long)]
        code: String,
    },
}
