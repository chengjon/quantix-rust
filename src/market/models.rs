#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardType {
    Sector,
    Concept,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardSortBy {
    ChangePct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoardRankRow {
    pub board_code: String,
    pub board_name: String,
    pub board_type: BoardType,
    pub rank: usize,
    pub change_pct: f64,
}

impl BoardRankRow {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NorthFlowSnapshot {
    pub trade_date: NaiveDate,
    pub sh_amount: f64,
    pub sz_amount: f64,
    pub total_amount: f64,
    pub balance: f64,
}

impl NorthFlowSnapshot {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeaderFilter {
    Sector(String),
    Concept(String),
    All,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LeaderRow {
    pub code: String,
    pub name: String,
    pub sector_name: Option<String>,
    pub concept_name: Option<String>,
    pub change_pct: f64,
}

impl LeaderRow {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarketOverview {
    pub top_sectors: Vec<BoardRankRow>,
    pub top_concepts: Vec<BoardRankRow>,
    pub north_flow: Option<NorthFlowSnapshot>,
    pub sentiment: Option<MarketSentimentSnapshot>,
}
