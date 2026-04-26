//! Account Command Handlers
//!
//! 多账户管理 CLI 命令处理器

use super::super::{AccountCommands, AccountGroupCommands};
use crate::account::storage::{load_registry, save_registry};
use crate::account::{
    AccountConfig, AccountRegistry, AccountRouter, AccountType, AllocationStrategy,
    JsonAccountRegistryStore, OrderSplitRequest, SplitTarget,
};
use crate::core::{QuantixError, Result};
use rust_decimal::Decimal;

/// 运行账户命令
pub async fn run_account_command(cmd: AccountCommands) -> Result<()> {
    match cmd {
        AccountCommands::Register {
            id,
            account_type,
            capital,
            adapter,
        } => {
            run_account_register(id, account_type, capital, adapter).await?;
        }
        AccountCommands::List {
            account_type,
            enabled_only,
        } => {
            run_account_list(account_type, enabled_only).await?;
        }
        AccountCommands::Show { id } => {
            run_account_show(id).await?;
        }
        AccountCommands::Update {
            id,
            enable,
            disable,
            capital,
            adapter,
        } => {
            run_account_update(id, enable, disable, capital, adapter).await?;
        }
        AccountCommands::Remove { id } => {
            run_account_remove(id).await?;
        }
        AccountCommands::Default { id } => {
            run_account_set_default(id).await?;
        }
        AccountCommands::Group(group_cmd) => {
            run_account_group_command(group_cmd).await?;
        }
        AccountCommands::Summary => {
            run_account_summary().await?;
        }
        AccountCommands::Split {
            code,
            side,
            quantity,
            target_type,
            target_id,
            price,
        } => {
            run_account_split(code, side, quantity, target_type, target_id, price).await?;
        }
    }
    Ok(())
}

/// 注册新账户
async fn run_account_register(
    id: String,
    account_type: String,
    capital: f64,
    adapter: String,
) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let acc_type = parse_account_type_input(&account_type)?;

    let mut config = AccountConfig::new(id.clone(), acc_type, parse_positive_capital(capital)?);
    config.adapter_name = adapter;

    registry.register_account(config).await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户注册成功: {}", id);
    Ok(())
}

/// 列出所有账户
async fn run_account_list(account_type: Option<String>, enabled_only: bool) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let accounts = if enabled_only {
        registry.list_enabled_accounts().await
    } else {
        registry.list_accounts().await
    };

    // 按类型过滤
    let filtered: Vec<_> = if let Some(ref type_filter) = account_type {
        let filter_type = parse_account_type_input(type_filter)?;
        accounts
            .into_iter()
            .filter(|a| a.account_type == filter_type)
            .collect()
    } else {
        accounts
    };

    if filtered.is_empty() {
        println!("📋 没有找到账户");
        return Ok(());
    }

    println!("📋 账户列表 (共 {} 个)", filtered.len());
    println!("{}", "─".repeat(80));
    println!(
        "{:<15} {:<10} {:<12} {:<15} {:<8}",
        "账户ID", "类型", "初始资金", "适配器", "状态"
    );
    println!("{}", "─".repeat(80));

    for account in filtered {
        let status = if account.enabled { "启用" } else { "禁用" };
        println!(
            "{:<15} {:<10} {:<12.2} {:<15} {:<8}",
            account.account_id,
            format!("{}", account.account_type).to_lowercase(),
            account.initial_capital,
            account.adapter_name,
            status
        );
    }

    Ok(())
}

/// 查看账户详情
async fn run_account_show(id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let account = registry
        .get_account(&id)
        .await
        .ok_or_else(|| QuantixError::Other(format!("账户不存在: {}", id)))?;

    println!("📋 账户详情: {}", account.account_id);
    println!("{}", "─".repeat(40));
    println!("  类型: {}", account.account_type);
    println!("  初始资金: {:.2}", account.initial_capital);
    println!("  适配器: {}", account.adapter_name);
    println!("  状态: {}", if account.enabled { "启用" } else { "禁用" });
    println!(
        "  创建时间: {}",
        account.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  更新时间: {}",
        account.updated_at.format("%Y-%m-%d %H:%M:%S")
    );

    // 显示所属组
    let groups = registry.get_account_groups(&id).await;
    if !groups.is_empty() {
        println!("  所属组:");
        for group in groups {
            println!("    - {} ({})", group.group_name, group.group_id);
        }
    }

    Ok(())
}

/// 更新账户配置
async fn run_account_update(
    id: String,
    enable: bool,
    disable: bool,
    capital: Option<f64>,
    adapter: Option<String>,
) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let mut account = registry
        .get_account(&id)
        .await
        .ok_or_else(|| QuantixError::Other(format!("账户不存在: {}", id)))?;

    if enable && disable {
        return Err(QuantixError::Other("不能同时启用和禁用账户".to_string()));
    }

    if enable {
        account.enabled = true;
    }
    if disable {
        account.enabled = false;
    }
    if let Some(c) = capital {
        account.initial_capital = parse_positive_capital(c)?;
    }
    if let Some(a) = adapter {
        account.adapter_name = a;
    }

    account.touch();
    registry.update_account(account).await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户更新成功: {}", id);
    Ok(())
}

fn parse_positive_capital(capital: f64) -> Result<Decimal> {
    let decimal = Decimal::from_f64_retain(capital)
        .ok_or_else(|| QuantixError::Other(format!("账户初始资金无法解析: {capital}")))?;

    if decimal <= Decimal::ZERO {
        return Err(QuantixError::Other(format!(
            "账户初始资金必须大于 0: {capital}"
        )));
    }

    Ok(decimal)
}

fn parse_account_type_input(input: &str) -> Result<AccountType> {
    match input.to_lowercase().as_str() {
        "paper" => Ok(AccountType::Paper),
        "mock_live" => Ok(AccountType::MockLive),
        "qmt_live" | "live" => Ok(AccountType::Live),
        _ => Err(QuantixError::Other(format!(
            "无效的账户类型: {}，支持: paper, mock_live, qmt_live（兼容 live 别名）",
            input
        ))),
    }
}

/// 删除账户
async fn run_account_remove(id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry.unregister_account(&id).await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户已删除: {}", id);
    Ok(())
}

/// 设置默认账户
async fn run_account_set_default(id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry.set_default_account(&id).await?;
    save_registry(&store, &registry).await?;

    println!("✅ 默认账户已设置: {}", id);
    Ok(())
}

/// 运行账户组命令
async fn run_account_group_command(cmd: AccountGroupCommands) -> Result<()> {
    match cmd {
        AccountGroupCommands::Create { id, name, strategy } => {
            run_group_create(id, name, strategy).await?;
        }
        AccountGroupCommands::List => {
            run_group_list().await?;
        }
        AccountGroupCommands::Show { id } => {
            run_group_show(id).await?;
        }
        AccountGroupCommands::Remove { id } => {
            run_group_remove(id).await?;
        }
        AccountGroupCommands::AddAccount {
            group_id,
            account_id,
        } => {
            run_group_add_account(group_id, account_id).await?;
        }
        AccountGroupCommands::RemoveAccount {
            group_id,
            account_id,
        } => {
            run_group_remove_account(group_id, account_id).await?;
        }
        AccountGroupCommands::SetStrategy {
            group_id,
            strategy,
            primary_account,
        } => {
            run_group_set_strategy(group_id, strategy, primary_account).await?;
        }
    }
    Ok(())
}

/// 创建账户组
async fn run_group_create(id: String, name: String, strategy: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry.create_group(id.clone(), name.clone()).await?;

    // 设置分配策略
    let alloc_strategy = parse_allocation_strategy(&strategy, None)?;
    match &alloc_strategy {
        AllocationStrategy::Equal => {} // 默认，无需设置
        _ => {
            registry
                .set_group_allocation_strategy(&id, alloc_strategy)
                .await?;
        }
    }

    save_registry(&store, &registry).await?;

    println!("✅ 账户组创建成功: {} ({})", id, name);
    Ok(())
}

/// 列出账户组
async fn run_group_list() -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let groups = registry.list_groups().await;

    if groups.is_empty() {
        println!("📋 没有找到账户组");
        return Ok(());
    }

    println!("📋 账户组列表 (共 {} 个)", groups.len());
    println!("{}", "─".repeat(80));
    println!(
        "{:<15} {:<20} {:<8} {:<20}",
        "组ID", "组名称", "账户数", "分配策略"
    );
    println!("{}", "─".repeat(80));

    for group in groups {
        let strategy_name = match &group.allocation_strategy {
            AllocationStrategy::Equal => "equal",
            AllocationStrategy::Proportional => "proportional",
            AllocationStrategy::Weighted(_) => "weighted",
            AllocationStrategy::PrimaryFirst { .. } => "primary_first",
        };
        println!(
            "{:<15} {:<20} {:<8} {:<20}",
            group.group_id,
            group.group_name,
            group.account_ids.len(),
            strategy_name
        );
    }

    Ok(())
}

/// 查看账户组详情
async fn run_group_show(id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let group = registry
        .get_group(&id)
        .await
        .ok_or_else(|| QuantixError::Other(format!("账户组不存在: {}", id)))?;

    let strategy_name = match &group.allocation_strategy {
        AllocationStrategy::Equal => "equal",
        AllocationStrategy::Proportional => "proportional",
        AllocationStrategy::Weighted(_) => "weighted",
        AllocationStrategy::PrimaryFirst { .. } => "primary_first",
    };

    println!("📋 账户组详情: {}", group.group_id);
    println!("{}", "─".repeat(40));
    println!("  名称: {}", group.group_name);
    println!("  分配策略: {}", strategy_name);
    println!("  账户数: {}", group.account_ids.len());
    println!("  账户列表:");

    for account_id in &group.account_ids {
        if let Some(account) = registry.get_account(account_id).await {
            let status = if account.enabled { "✓" } else { "✗" };
            println!(
                "    {} {} ({}) - {:.2}",
                status, account.account_id, account.adapter_name, account.initial_capital
            );
        }
    }
    println!(
        "  创建时间: {}",
        group.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  更新时间: {}",
        group.updated_at.format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}

/// 删除账户组
async fn run_group_remove(id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry.delete_group(&id).await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户组已删除: {}", id);
    Ok(())
}

/// 向账户组添加账户
async fn run_group_add_account(group_id: String, account_id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry
        .add_account_to_group(&group_id, account_id.clone())
        .await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户 {} 已添加到组 {}", account_id, group_id);
    Ok(())
}

/// 从账户组移除账户
async fn run_group_remove_account(group_id: String, account_id: String) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    registry
        .remove_account_from_group(&group_id, &account_id)
        .await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户 {} 已从组 {} 移除", account_id, group_id);
    Ok(())
}

/// 设置分配策略
async fn run_group_set_strategy(
    group_id: String,
    strategy: String,
    primary_account: Option<String>,
) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let alloc_strategy = parse_allocation_strategy(&strategy, primary_account)?;
    registry
        .set_group_allocation_strategy(&group_id, alloc_strategy)
        .await?;
    save_registry(&store, &registry).await?;

    println!("✅ 账户组 {} 分配策略已更新", group_id);
    Ok(())
}

/// 资金聚合视图
async fn run_account_summary() -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;

    let accounts = registry.list_enabled_accounts().await;

    if accounts.is_empty() {
        println!("📋 没有启用的账户");
        return Ok(());
    }

    let total_capital: Decimal = accounts.iter().map(|a| a.initial_capital).sum();
    let paper_capital: Decimal = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::Paper)
        .map(|a| a.initial_capital)
        .sum();
    let live_capital: Decimal = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::Live)
        .map(|a| a.initial_capital)
        .sum();
    let mock_live_capital: Decimal = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::MockLive)
        .map(|a| a.initial_capital)
        .sum();

    println!("📊 资金聚合视图");
    println!("{}", "─".repeat(50));
    println!("  总账户数: {}", accounts.len());
    println!("  总资金: {:.2}", total_capital);
    println!("{}", "─".repeat(50));
    let paper_count = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::Paper)
        .count();
    let live_count = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::Live)
        .count();
    let mock_count = accounts
        .iter()
        .filter(|a| a.account_type == AccountType::MockLive)
        .count();
    println!("  模拟账户资金: {:.2} ({})", paper_capital, paper_count);
    println!("  qmt_live 账户资金: {:.2} ({})", live_capital, live_count);
    println!("  模拟实盘资金: {:.2} ({})", mock_live_capital, mock_count);

    // 显示账户组信息
    let groups = registry.list_groups().await;
    if !groups.is_empty() {
        println!("{}", "─".repeat(50));
        println!("  账户组数: {}", groups.len());
        for group in &groups {
            let mut group_capital = Decimal::ZERO;
            for acc_id in &group.account_ids {
                if let Some(acc) = registry.get_account(acc_id).await {
                    group_capital += acc.initial_capital;
                }
            }
            println!(
                "    {} ({}): {:.2}",
                group.group_name,
                group.account_ids.len(),
                group_capital
            );
        }
    }

    Ok(())
}

/// 订单拆分预览
async fn run_account_split(
    code: String,
    side: String,
    quantity: i64,
    target_type: String,
    target_id: String,
    price: Option<f64>,
) -> Result<()> {
    let store = JsonAccountRegistryStore::default_store();
    let registry = load_or_create_registry(&store).await?;
    let router = AccountRouter::new(registry);

    let target = match target_type.to_lowercase().as_str() {
        "single" => SplitTarget::Single(target_id),
        "group" => SplitTarget::Group(target_id),
        _ => {
            return Err(QuantixError::Other(format!(
                "无效的目标类型: {}，支持: single, group",
                target_type
            )));
        }
    };

    let request = OrderSplitRequest {
        symbol: code.clone(),
        side: side.clone(),
        total_quantity: quantity,
        price: price.map(|p| Decimal::from_f64_retain(p).unwrap_or_default()),
        target,
    };

    let result = router.split_order(request).await?;

    let strategy_name = match &result.strategy {
        AllocationStrategy::Equal => "equal",
        AllocationStrategy::Proportional => "proportional",
        AllocationStrategy::Weighted(_) => "weighted",
        AllocationStrategy::PrimaryFirst { .. } => "primary_first",
    };

    println!("📊 订单拆分预览");
    println!("{}", "─".repeat(60));
    println!("  股票: {}", code);
    println!("  方向: {}", side);
    println!("  总数量: {}", quantity);
    println!("  分配策略: {}", strategy_name);
    println!("{}", "─".repeat(60));
    println!("  拆分结果:");

    let mut total_split = 0i64;
    for split in &result.splits {
        let price_str = split
            .price
            .map(|p| format!("{:.2}", p))
            .unwrap_or_else(|| "市价".to_string());
        println!(
            "    账户 {:<15}: {} 股 @ {}",
            split.account_id, split.quantity, price_str
        );
        total_split += split.quantity;
    }

    println!("{}", "─".repeat(60));
    println!(
        "  拆分后总量: {} (差额: {})",
        total_split,
        quantity - total_split
    );

    Ok(())
}

// ============ Helper Functions ============

/// 加载或创建注册表
async fn load_or_create_registry(store: &JsonAccountRegistryStore) -> Result<AccountRegistry> {
    load_registry(store).await
}

#[cfg(test)]
mod tests {
    use super::parse_account_type_input;
    use crate::account::AccountType;

    #[test]
    fn parse_account_type_prefers_qmt_live_wording_but_keeps_live_alias() {
        assert_eq!(parse_account_type_input("paper").unwrap(), AccountType::Paper);
        assert_eq!(
            parse_account_type_input("mock_live").unwrap(),
            AccountType::MockLive
        );
        assert_eq!(
            parse_account_type_input("qmt_live").unwrap(),
            AccountType::Live
        );
        assert_eq!(parse_account_type_input("live").unwrap(), AccountType::Live);
    }

    #[test]
    fn parse_account_type_error_mentions_qmt_live_boundary() {
        let err = parse_account_type_input("invalid").unwrap_err().to_string();
        assert!(err.contains("paper, mock_live, qmt_live"));
        assert!(err.contains("live"));
    }
}

/// 解析分配策略
fn parse_allocation_strategy(
    strategy: &str,
    primary_account: Option<String>,
) -> Result<AllocationStrategy> {
    match strategy.to_lowercase().as_str() {
        "equal" => Ok(AllocationStrategy::Equal),
        "proportional" => Ok(AllocationStrategy::Proportional),
        "weighted" => {
            // 权重需要额外配置，这里返回默认的 Equal
            println!("⚠️  加权分配需要额外配置权重，暂时使用平均分配");
            Ok(AllocationStrategy::Equal)
        }
        "primary_first" => {
            let primary = primary_account.ok_or_else(|| {
                QuantixError::Other("primary_first 策略需要指定主账户".to_string())
            })?;
            Ok(AllocationStrategy::PrimaryFirst {
                primary_account_id: primary,
            })
        }
        _ => Err(QuantixError::Other(format!(
            "无效的分配策略: {}，支持: equal, proportional, weighted, primary_first",
            strategy
        ))),
    }
}
