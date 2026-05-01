use super::*;

#[tokio::test]
async fn test_vwap_initialize() {
    let mut executor = VwapExecutor::new();
    let params = AlgoParams::vwap("600519.SH".to_string(), "buy".to_string(), 1000, 30);

    let result = executor.initialize(params).await;
    assert!(result.is_ok());

    let algo_id = result.unwrap();
    assert!(algo_id.starts_with("VWAP-"));
}

#[test]
fn test_vwap_slice_plan() {
    let executor = VwapExecutor::new();
    let start = Utc::now();
    let end = start + Duration::hours(2);

    let params = AlgoParams {
        algo_type: AlgoType::VWAP,
        symbol: "600519.SH".to_string(),
        side: "buy".to_string(),
        total_quantity: 10000,
        start_time: start,
        end_time: end,
        interval_seconds: Some(300), // 5分钟
        randomize_timing: false,
        randomize_quantity: false,
        ..Default::default()
    };

    let plan = executor.get_slice_plan(&params).unwrap();

    // 验证总数量匹配
    let total: i64 = plan.slices.iter().map(|s| s.quantity).sum();
    assert_eq!(total, 10000);

    // 验证有权重
    for slice in &plan.slices {
        assert!(slice.volume_weight.is_some());
    }
}

#[test]
fn test_volume_weight() {
    let executor = VwapExecutor::new();

    // 开盘时间 (9:35)
    let time1: DateTime<Utc> = "2026-03-27T01:35:00Z".parse().unwrap(); // UTC 09:35 Beijing
    let weight1 = executor.get_volume_weight(time1);

    // 中午时间 (10:30 Beijing = 02:30 UTC)
    let time2: DateTime<Utc> = "2026-03-27T02:30:00Z".parse().unwrap();
    let weight2 = executor.get_volume_weight(time2);

    // 验证权重为正
    assert!(weight1 > Decimal::ZERO);
    assert!(weight2 > Decimal::ZERO);
}
