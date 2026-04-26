#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLockStateSource {
    Open,
    DailyLossLocked,
    ManualReleaseActive,
}
