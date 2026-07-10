#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 板块类型：Sector 行业板块、Concept 概念板块。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardType {
    Sector,
    Concept,
}

/// 板块排名排序字段：当前仅 ChangePct 涨跌幅，保留枚举供后续扩展。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardSortBy {
    ChangePct,
}

/// 板块排名展示行：board_code 板块代码、board_name 名称、board_type 类型、rank 排名、change_pct 涨跌幅。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardRankRow {
    pub board_code: String,
    pub board_name: String,
    pub board_type: BoardType,
    pub rank: usize,
    pub change_pct: f64,
}

impl BoardRankRow {
    /// 构造 BoardRankRow：按参数填充 board_code/board_name/board_type/rank/change_pct。
    pub fn new(
        board_code: impl Into<String>,
        board_name: impl Into<String>,
        board_type: BoardType,
        rank: usize,
        change_pct: f64,
    ) -> Self {
        Self {
            board_code: board_code.into(),
            board_name: board_name.into(),
            board_type,
            rank,
            change_pct,
        }
    }
}

/// 北向资金快照：trade_date 日期、sh_amount 沪股通净流入、sz_amount 深股通净流入、total_amount 合计、balance 当日余额。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NorthFlowSnapshot {
    pub trade_date: NaiveDate,
    pub sh_amount: f64,
    pub sz_amount: f64,
    pub total_amount: f64,
    pub balance: f64,
}

impl NorthFlowSnapshot {
    /// 构造 NorthFlowSnapshot：按参数填充 trade_date/sh_amount/sz_amount/total_amount/balance。
    pub fn new(
        trade_date: NaiveDate,
        sh_amount: f64,
        sz_amount: f64,
        total_amount: f64,
        balance: f64,
    ) -> Self {
        Self {
            trade_date,
            sh_amount,
            sz_amount,
            total_amount,
            balance,
        }
    }
}

/// 市场情绪快照：trade_date 日期、up_count/down_count 涨跌家数、limit_up_count/limit_down_count 涨跌停数、seal_rate 封板率、break_rate 炸板率、consecutive_board_count 连板数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketSentimentSnapshot {
    pub trade_date: NaiveDate,
    pub up_count: usize,
    pub down_count: usize,
    pub limit_up_count: usize,
    pub limit_down_count: usize,
    pub seal_rate: f64,
    pub break_rate: f64,
    pub consecutive_board_count: usize,
}

impl MarketSentimentSnapshot {
    /// 构造 MarketSentimentSnapshot：按参数填充当日涨跌停/封板/炸板等市场情绪字段。
    pub fn new(
        trade_date: NaiveDate,
        up_count: usize,
        down_count: usize,
        limit_up_count: usize,
        limit_down_count: usize,
        seal_rate: f64,
        break_rate: f64,
        consecutive_board_count: usize,
    ) -> Self {
        Self {
            trade_date,
            up_count,
            down_count,
            limit_up_count,
            limit_down_count,
            seal_rate,
            break_rate,
            consecutive_board_count,
        }
    }
}

/// 龙头查询过滤器：Sector 按行业、Concept 按概念、All 全市场。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderFilter {
    Sector(String),
    Concept(String),
    All,
}

/// 龙头展示行：code、name、sector_name 可选所属行业、concept_name 可选所属概念、change_pct 涨跌幅。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderRow {
    pub code: String,
    pub name: String,
    pub sector_name: Option<String>,
    pub concept_name: Option<String>,
    pub change_pct: f64,
}

impl LeaderRow {
    /// 构造 LeaderRow：按参数填充 code/name/sector_name/concept_name/change_pct。
    pub fn new(
        code: impl Into<String>,
        name: impl Into<String>,
        sector_name: Option<String>,
        concept_name: Option<String>,
        change_pct: f64,
    ) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            sector_name,
            concept_name,
            change_pct,
        }
    }
}

/// 市场总览：top_sectors 行业榜、top_concepts 概念榜、north_flow 可选北向资金快照、sentiment 可选情绪快照。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketOverview {
    pub top_sectors: Vec<BoardRankRow>,
    pub top_concepts: Vec<BoardRankRow>,
    pub north_flow: Option<NorthFlowSnapshot>,
    pub sentiment: Option<MarketSentimentSnapshot>,
}
