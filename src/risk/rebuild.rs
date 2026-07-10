use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::core::{QuantixError, Result};
use crate::risk::{
    LiveImportCashBusinessType, LiveImportMirrorAccount, LiveImportMirrorPosition,
    LiveImportRecord, LiveImportRecordType, LiveImportTradeSide, SqliteLiveImportStore,
};

/// 实盘镜像重建引擎：基于 SqliteLiveImportStore 中的 trade/cash_dividend/cash_action 记录，按逻辑时间排序后回放出账户持仓与现金快照。
#[derive(Debug, Clone)]
pub struct SqliteLiveMirrorRebuildEngine {
    store: SqliteLiveImportStore,
}

impl SqliteLiveMirrorRebuildEngine {
    /// 创建引擎：注入已初始化的 SqliteLiveImportStore，后续 rebuild_account 从中拉记录。
    pub fn new(store: SqliteLiveImportStore) -> Self {
        Self { store }
    }

    pub async fn rebuild_account(
        &self,
        account_id: &str,
        rebuilt_at: DateTime<Utc>,
    ) -> Result<LiveImportMirrorAccount> {
        let mut records = self.store.list_records(account_id).await?;
        records.sort_by(|left, right| {
            logical_time(left)
                .cmp(&logical_time(right))
                .then_with(|| left.external_id.cmp(&right.external_id))
        });

        let rebuilt = rebuild_from_records(account_id, &records, rebuilt_at);
        match rebuilt {
            Ok(mirror) => {
                self.store.replace_mirror_account(&mirror).await?;
                self.store
                    .append_rebuild_audit(account_id, "success", None, rebuilt_at)
                    .await?;
                Ok(mirror)
            }
            Err(err) => {
                self.store
                    .append_rebuild_audit(account_id, "failed", Some(&err.to_string()), rebuilt_at)
                    .await?;
                Err(err)
            }
        }
    }
}

fn rebuild_from_records(
    account_id: &str,
    records: &[LiveImportRecord],
    rebuilt_at: DateTime<Utc>,
) -> Result<LiveImportMirrorAccount> {
    let latest_date = records
        .last()
        .map(|record| logical_time(record).date_naive())
        .unwrap_or_else(|| rebuilt_at.date_naive());
    let has_prior_day = records
        .iter()
        .any(|record| logical_time(record).date_naive() < latest_date);

    let mut cash_balance = Decimal::ZERO;
    let mut realized_pnl = Decimal::ZERO;
    let mut total_fees = Decimal::ZERO;
    let mut positions = BTreeMap::<String, LiveImportMirrorPosition>::new();
    let mut as_of = rebuilt_at;
    let mut current_date = rebuilt_at.date_naive();
    let mut current_day_started = false;
    let mut starting_total_assets = Decimal::ZERO;

    for record in records {
        as_of = logical_time(record);
        if as_of.date_naive() != current_date {
            current_date = as_of.date_naive();
            current_day_started = false;
        }
        if !current_day_started {
            starting_total_assets = if current_date == latest_date && !has_prior_day {
                Decimal::ZERO
            } else {
                current_total_assets(cash_balance, &positions)
            };
            current_day_started = true;
        }
        match record.record_type {
            LiveImportRecordType::Trade => {
                let code = record.code.clone().ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 code".to_string())
                })?;
                let side = record.side.ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 side".to_string())
                })?;
                let price = record.price.ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 price".to_string())
                })?;
                let volume = record.volume.ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 volume".to_string())
                })?;
                let fee_total = record.fee_total.ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 fee_total".to_string())
                })?;
                let executed_at = record.executed_at.ok_or_else(|| {
                    QuantixError::Other("risk rebuild trade 缺少 executed_at".to_string())
                })?;

                let amount = price * Decimal::from(volume);
                total_fees += fee_total;

                match side {
                    LiveImportTradeSide::Buy => {
                        cash_balance -= amount + fee_total;
                        match positions.get_mut(&code) {
                            Some(position) => {
                                let existing_cost =
                                    position.avg_cost * Decimal::from(position.volume);
                                let new_volume = position.volume + volume;
                                let new_cost = existing_cost + amount + fee_total;
                                position.volume = new_volume;
                                position.avg_cost = new_cost / Decimal::from(new_volume);
                                position.last_trade_at = executed_at;
                            }
                            None => {
                                positions.insert(
                                    code.clone(),
                                    LiveImportMirrorPosition {
                                        code,
                                        volume,
                                        avg_cost: (amount + fee_total) / Decimal::from(volume),
                                        last_trade_at: executed_at,
                                    },
                                );
                            }
                        }
                    }
                    LiveImportTradeSide::Sell => {
                        let position = positions.get_mut(&code).ok_or_else(|| {
                            QuantixError::Other(format!(
                                "risk rebuild 卖出数量超过当前持仓: {} 无持仓",
                                code
                            ))
                        })?;
                        if volume > position.volume {
                            return Err(QuantixError::Other(format!(
                                "risk rebuild 卖出数量超过当前持仓: {} 当前 {} 卖出 {}",
                                code, position.volume, volume
                            )));
                        }

                        let cost_basis = position.avg_cost * Decimal::from(volume);
                        let proceeds_minus_fee = amount - fee_total;
                        cash_balance += proceeds_minus_fee;
                        realized_pnl += proceeds_minus_fee - cost_basis;

                        if volume == position.volume {
                            positions.remove(&code);
                        } else {
                            position.volume -= volume;
                            position.last_trade_at = executed_at;
                        }
                    }
                }
            }
            LiveImportRecordType::Cash => {
                let business_type = record.business_type.ok_or_else(|| {
                    QuantixError::Other("risk rebuild cash 缺少 business_type".to_string())
                })?;
                let amount = record.amount.ok_or_else(|| {
                    QuantixError::Other("risk rebuild cash 缺少 amount".to_string())
                })?;
                let _occurred_at = record.occurred_at.ok_or_else(|| {
                    QuantixError::Other("risk rebuild cash 缺少 occurred_at".to_string())
                })?;

                match business_type {
                    LiveImportCashBusinessType::Deposit => {
                        if amount <= Decimal::ZERO {
                            return Err(QuantixError::Other(
                                "risk rebuild deposit amount 必须大于 0".to_string(),
                            ));
                        }
                        cash_balance += amount;
                    }
                    LiveImportCashBusinessType::Withdraw => {
                        if amount >= Decimal::ZERO {
                            return Err(QuantixError::Other(
                                "risk rebuild withdraw amount 必须小于 0".to_string(),
                            ));
                        }
                        cash_balance += amount;
                    }
                }
            }
        }
    }

    let current_total_assets = current_total_assets(cash_balance, &positions);
    if !has_prior_day {
        starting_total_assets = current_total_assets;
    }

    Ok(LiveImportMirrorAccount {
        account_id: account_id.to_string(),
        trading_date: current_date,
        as_of,
        starting_total_assets,
        current_total_assets,
        cash_balance,
        realized_pnl,
        total_fees,
        last_rebuild_at: rebuilt_at,
        positions: positions.into_values().collect(),
    })
}

fn logical_time(record: &LiveImportRecord) -> DateTime<Utc> {
    // 上游已通过 validate_* 保证 executed_at / occurred_at 至少一个存在；
    // 若数据畸形导致两者都为 None，回退到 UTC epoch 并记录 warn。
    record.executed_at.or(record.occurred_at).unwrap_or_else(|| {
        tracing::warn!(
            "live import record 缺少 executed_at / occurred_at，logical_time 回退 epoch"
        );
        DateTime::<Utc>::UNIX_EPOCH
    })
}

fn current_total_assets(
    cash_balance: Decimal,
    positions: &BTreeMap<String, LiveImportMirrorPosition>,
) -> Decimal {
    cash_balance
        + positions.values().fold(Decimal::ZERO, |acc, position| {
            acc + Decimal::from(position.volume) * position.avg_cost
        })
}
