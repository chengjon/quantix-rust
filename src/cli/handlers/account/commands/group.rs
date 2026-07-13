//! 账户组命令处理函数。

use crate::account::storage::save_registry;
use crate::account::{AllocationStrategy, JsonAccountRegistryStore};
use crate::cli::commands::AccountGroupCommands;
use crate::core::{QuantixError, Result};

use super::load_or_create_registry;

/// 运行账户组命令
pub(super) async fn run_account_group_command(cmd: AccountGroupCommands) -> Result<()> {
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
        _ => Err(QuantixError::Unsupported(format!(
            "无效的分配策略: {}，支持: equal, proportional, weighted, primary_first",
            strategy
        ))),
    }
}
