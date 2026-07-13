use super::*;

#[test]
fn test_execute_execution_bridge_qmt_live_source_keeps_confirmation_and_request_guidance() {
    let source = std::fs::read_to_string(
        repo_root()
            .join("src")
            .join("cli")
            .join("handlers")
            .join("execution_handler")
            .join("live.rs"),
    )
    .expect("expected src/cli/handlers/execution_handler/live.rs");

    assert!(
        source.contains("输入 'YES' 确认下单"),
        "expected manual qmt_live handler to keep explicit YES confirmation"
    );
    assert!(
        source.contains("查看 request 与后续收敛状态: quantix strategy request show"),
        "expected manual qmt_live handler to guide operators to request/reconciliation visibility"
    );
    assert!(
        source.contains("request 可能会显示为 completed"),
        "expected manual qmt_live handler to explain completed request visibility after submission"
    );
    assert!(
        source.contains("这只表示执行层已完成提交"),
        "expected manual qmt_live handler to explain completed request semantics"
    );
    assert!(
        source.contains("订单初始状态通常仍为 pending_submit"),
        "expected manual qmt_live handler to explain non-terminal initial order status"
    );
    assert!(
        !source.contains("查询订单状态: quantix execution bridge qmt-query --order-id"),
        "expected manual qmt_live handler to drop legacy qmt-query post-submit guidance"
    );
}
