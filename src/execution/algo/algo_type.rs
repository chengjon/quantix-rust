//! `AlgoType` enum and its `Display` / `FromStr` impls, used by the CLI
//! parser and the algorithm dispatch in `cli::handlers::algo`.

/// 算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AlgoType {
    /// 时间加权平均价格
    TWAP,
    /// 成交量加权平均价格
    VWAP,
    /// 参与率 (Percentage of Volume)
    POV,
    /// 冰山订单
    Iceberg,
}

impl std::fmt::Display for AlgoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlgoType::TWAP => write!(f, "TWAP"),
            AlgoType::VWAP => write!(f, "VWAP"),
            AlgoType::POV => write!(f, "POV"),
            AlgoType::Iceberg => write!(f, "Iceberg"),
        }
    }
}

impl std::str::FromStr for AlgoType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TWAP" => Ok(AlgoType::TWAP),
            "VWAP" => Ok(AlgoType::VWAP),
            "POV" => Ok(AlgoType::POV),
            "ICEBERG" => Ok(AlgoType::Iceberg),
            _ => Err(format!("Unknown algorithm type: {}", s)),
        }
    }
}
