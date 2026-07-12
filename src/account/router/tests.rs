//! Account Router integration tests.

use super::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;

async fn create_test_registry() -> AccountRegistry {
    let registry = AccountRegistry::new();

    // 创建测试账户
    let account1 = AccountConfig::new(
        "acc-1".to_string(),
        super::super::models::AccountType::Paper,
        dec!(100000),
    );
    let account2 = AccountConfig::new(
        "acc-2".to_string(),
        super::super::models::AccountType::Paper,
        dec!(200000),
    );
    let account3 = AccountConfig::new(
        "acc-3".to_string(),
        super::super::models::AccountType::Paper,
        dec!(300000),
    );

    registry.register_account(account1).await.unwrap();
    registry.register_account(account2).await.unwrap();
    registry.register_account(account3).await.unwrap();

    // 创建账户组
    registry
        .create_group("group-1".to_string(), "Test Group".to_string())
        .await
        .unwrap();
    registry
        .add_account_to_group("group-1", "acc-1".to_string())
        .await
        .unwrap();
    registry
        .add_account_to_group("group-1", "acc-2".to_string())
        .await
        .unwrap();
    registry
        .add_account_to_group("group-1", "acc-3".to_string())
        .await
        .unwrap();

    registry
}

#[tokio::test]
async fn test_split_order_single_account() {
    let registry = create_test_registry().await;
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Single("acc-1".to_string()),
    };

    let result = router.split_order(request).await.unwrap();
    assert_eq!(result.splits.len(), 1);
    assert_eq!(result.splits[0].quantity, 1000);
    assert_eq!(result.splits[0].account_id, "acc-1");
}

#[tokio::test]
async fn test_split_order_group_equal() {
    let registry = create_test_registry().await;
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Group("group-1".to_string()),
    };

    let result = router.split_order(request).await.unwrap();
    assert_eq!(result.splits.len(), 3);

    let total: i64 = result.splits.iter().map(|s| s.quantity).sum();
    assert_eq!(total, 1000);
}

#[tokio::test]
async fn test_split_order_group_proportional_keeps_fractional_allocations() {
    let registry = create_test_registry().await;
    registry
        .set_group_allocation_strategy("group-1", AllocationStrategy::Proportional)
        .await
        .unwrap();
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Group("group-1".to_string()),
    };

    let result = router.split_order(request).await.unwrap();
    let total: i64 = result.splits.iter().map(|s| s.quantity).sum();

    assert_eq!(result.splits.len(), 3);
    assert_eq!(total, 1000);
    assert!(result.splits.iter().all(|s| s.quantity > 0));
}

#[tokio::test]
async fn test_split_order_group_weighted_keeps_fractional_allocations() {
    let registry = create_test_registry().await;
    registry
        .set_group_allocation_strategy(
            "group-1",
            AllocationStrategy::Weighted(HashMap::from([
                ("acc-1".to_string(), dec!(1)),
                ("acc-2".to_string(), dec!(2)),
                ("acc-3".to_string(), dec!(3)),
            ])),
        )
        .await
        .unwrap();
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Group("group-1".to_string()),
    };

    let result = router.split_order(request).await.unwrap();
    let total: i64 = result.splits.iter().map(|s| s.quantity).sum();

    assert_eq!(result.splits.len(), 3);
    assert_eq!(total, 1000);
    assert!(result.splits.iter().all(|s| s.quantity > 0));
}

#[tokio::test]
async fn test_split_order_rejects_non_positive_total_quantity() {
    let registry = create_test_registry().await;
    let router = AccountRouter::new(registry);

    for total_quantity in [0, -100] {
        let request = OrderSplitRequest {
            symbol: "600519.SH".to_string(),
            side: "buy".to_string(),
            total_quantity,
            price: Some(dec!(100)),
            target: SplitTarget::Single("acc-1".to_string()),
        };

        let result = router.split_order(request).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("订单数量必须大于 0")
        );
    }
}

#[tokio::test]
async fn test_split_order_invalid_account() {
    let registry = create_test_registry().await;
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Single("nonexistent".to_string()),
    };

    let result = router.split_order(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_split_order_rejects_single_account_with_non_positive_capital() {
    let registry = AccountRegistry::new();
    registry
        .register_account(AccountConfig::new(
            "acc-1".to_string(),
            super::super::models::AccountType::Paper,
            dec!(0),
        ))
        .await
        .unwrap_err();

    let registry = AccountRegistry::with_accounts(
        HashMap::from([(
            "acc-1".to_string(),
            AccountConfig::new(
                "acc-1".to_string(),
                super::super::models::AccountType::Paper,
                dec!(0),
            ),
        )]),
        "acc-1".to_string(),
    );
    let router = AccountRouter::new(registry);

    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Single("acc-1".to_string()),
    };

    let result = router.split_order(request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("必须大于 0"));
}

#[tokio::test]
async fn test_split_order_group_skips_non_positive_capital_accounts() {
    let registry = AccountRegistry::with_accounts(
        HashMap::from([
            (
                "acc-1".to_string(),
                AccountConfig::new(
                    "acc-1".to_string(),
                    super::super::models::AccountType::Paper,
                    dec!(0),
                ),
            ),
            (
                "acc-2".to_string(),
                AccountConfig::new(
                    "acc-2".to_string(),
                    super::super::models::AccountType::Paper,
                    dec!(200000),
                ),
            ),
        ]),
        "acc-2".to_string(),
    );

    registry
        .create_group("group-1".to_string(), "Test Group".to_string())
        .await
        .unwrap();
    registry
        .add_account_to_group("group-1", "acc-1".to_string())
        .await
        .unwrap();
    registry
        .add_account_to_group("group-1", "acc-2".to_string())
        .await
        .unwrap();

    let router = AccountRouter::new(registry);
    let request = OrderSplitRequest {
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 1000,
        price: Some(dec!(100)),
        target: SplitTarget::Group("group-1".to_string()),
    };

    let result = router.split_order(request).await.unwrap();
    assert_eq!(result.splits.len(), 1);
    assert_eq!(result.splits[0].account_id, "acc-2");
    assert_eq!(result.splits[0].quantity, 1000);
}
