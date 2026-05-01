use super::state_helpers::pct_change;
use super::*;

pub async fn evaluate_industry_blocklist(
    rule: &RiskRule,
    projected_buy: &ProjectedBuyImpact,
    resolver: Option<&dyn RiskIndustryResolver>,
    now: DateTime<Utc>,
) -> Result<()> {
    let RuleValue::TextList(blocked_industries) = &rule.value else {
        return Err(QuantixError::Other(
            "risk rule industry-blocklist 配置无效".to_string(),
        ));
    };

    let resolver = resolver.ok_or_else(|| {
        QuantixError::Config(format!(
            "risk rule industry-blocklist 检查失败: code={} 原因=未配置行业解析器",
            projected_buy.code
        ))
    })?;

    let resolved = resolver
        .resolve(&projected_buy.code, now.date_naive(), now)
        .await
        .map_err(|err| {
            QuantixError::DataSource(format!(
                "risk rule industry-blocklist 检查失败: code={} 原因={}",
                projected_buy.code, err
            ))
        })?;

    if blocked_industries
        .iter()
        .any(|industry_name| industry_name == &resolved.industry_name)
    {
        return Err(QuantixError::Other(format!(
            "risk rule industry-blocklist 已拒绝: code={} industry={} blocked={}",
            resolved.code,
            resolved.industry_name,
            blocked_industries.join(",")
        )));
    }

    Ok(())
}

/// 行业集中度检查
///
/// 检查买入后目标行业的集中度是否超过限制。
/// 该检查依赖运行时行业解析；缺失解析器或解析失败时按 fail-closed 拒绝买单。
pub async fn evaluate_industry_limit(
    rule: &RiskRule,
    snapshot: &RiskAccountSnapshot,
    projected_buy: &ProjectedBuyImpact,
    resolver: Option<&dyn RiskIndustryResolver>,
    now: DateTime<Utc>,
) -> Result<()> {
    let RuleValue::Percentage(limit_pct) = rule.value.clone() else {
        return Err(QuantixError::Other(
            "risk rule industry-limit 配置无效，仅支持百分比".to_string(),
        ));
    };

    if projected_buy.projected_total_assets <= Decimal::ZERO {
        return Err(QuantixError::Other(
            "risk check projected_total_assets 必须大于 0".to_string(),
        ));
    }

    let resolver = resolver.ok_or_else(|| {
        QuantixError::Config(format!(
            "risk rule industry-limit 检查失败: code={} 原因=未配置行业解析器",
            projected_buy.code
        ))
    })?;

    let target_industry = resolver
        .resolve(&projected_buy.code, now.date_naive(), now)
        .await
        .map_err(|err| {
            QuantixError::DataSource(format!(
                "risk rule industry-limit 检查失败: code={} 原因={}",
                projected_buy.code, err
            ))
        })?;

    let mut current_industry_value = Decimal::ZERO;
    let mut current_target_position_value = Decimal::ZERO;
    for position in &snapshot.positions {
        let resolved_position = resolver
            .resolve(&position.code, now.date_naive(), now)
            .await
            .map_err(|err| {
                QuantixError::DataSource(format!(
                    "risk rule industry-limit 检查失败: code={} 原因={}",
                    position.code, err
                ))
            })?;

        if resolved_position.industry_name == target_industry.industry_name {
            current_industry_value += position.market_value;
        }

        if resolved_position.code == target_industry.code {
            current_target_position_value = position.market_value;
        }
    }

    let projected_increment =
        if projected_buy.projected_position_value > current_target_position_value {
            projected_buy.projected_position_value - current_target_position_value
        } else {
            Decimal::ZERO
        };
    let projected_industry_value = current_industry_value + projected_increment;
    let projected_ratio_pct =
        projected_industry_value / projected_buy.projected_total_assets * dec!(100);

    if projected_ratio_pct > limit_pct {
        return Err(QuantixError::Other(format!(
            "risk rule industry-limit 已超限: code={} industry={} limit={} projected_industry={} projected_ratio={}%",
            target_industry.code,
            target_industry.industry_name,
            limit_pct,
            projected_industry_value,
            projected_ratio_pct
        )));
    }

    Ok(())
}

/// 自动减仓触发检查
///
/// 检查是否需要触发自动减仓规则
/// 返回需要减仓的股票列表和减仓比例
pub fn check_auto_reduce_trigger(
    state: &RiskState,
    snapshot: &RiskAccountSnapshot,
    now: DateTime<Utc>,
) -> Option<AutoReduceDecision> {
    let rule = find_enabled_rule(state, RiskRuleType::AutoReduce)?;

    let baseline = state.daily_baseline.as_ref()?;
    let daily_pnl = snapshot.total_assets - baseline.starting_total_assets;
    let daily_pnl_pct = pct_change(daily_pnl, baseline.starting_total_assets);

    let triggered = match rule.value.clone() {
        RuleValue::Percentage(limit_pct) => daily_pnl_pct <= -limit_pct,
        RuleValue::Amount(limit) => daily_pnl <= -limit,
        RuleValue::TextList(_) => false,
    };

    if triggered {
        let reduce_ratio = dec!(50);
        Some(AutoReduceDecision {
            trigger_rule: rule.clone(),
            current_loss_pct: daily_pnl_pct,
            reduce_ratio,
            positions_to_reduce: snapshot.positions.clone(),
            triggered_at: now,
        })
    } else {
        None
    }
}

/// 自动减仓决策
#[derive(Debug, Clone)]
pub struct AutoReduceDecision {
    pub trigger_rule: RiskRule,
    pub current_loss_pct: Decimal,
    pub reduce_ratio: Decimal,
    pub positions_to_reduce: Vec<crate::risk::models::RiskPositionSnapshot>,
    pub triggered_at: DateTime<Utc>,
}
