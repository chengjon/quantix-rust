use quantix_cli::execution::mode_semantics::{
    PAPER_IMMEDIATE_CHANNEL, PAPER_IMMEDIATE_RISK_NOTICE, PAPER_SIM_LIFECYCLE_CHANNEL,
    PAPER_SIM_LIFECYCLE_RISK_NOTICE, QMT_LIVE_CHANNEL, QMT_LIVE_RISK_NOTICE,
    storage_namespace_for_channel,
};

#[test]
fn execution_mode_risk_notices_are_standardized() {
    for notice in [
        PAPER_IMMEDIATE_RISK_NOTICE,
        PAPER_SIM_LIFECYCLE_RISK_NOTICE,
        QMT_LIVE_RISK_NOTICE,
    ] {
        assert!(
            notice.starts_with('['),
            "execution mode risk notice must start with a channel tag"
        );
    }

    assert!(PAPER_IMMEDIATE_RISK_NOTICE.contains("[paper_immediate]"));
    assert!(PAPER_IMMEDIATE_RISK_NOTICE.contains("local ledger"));
    assert!(PAPER_IMMEDIATE_RISK_NOTICE.contains("no broker submission"));
    assert!(PAPER_IMMEDIATE_RISK_NOTICE.contains("no market matching"));

    assert!(PAPER_SIM_LIFECYCLE_RISK_NOTICE.contains("[paper_sim_lifecycle]"));
    assert!(PAPER_SIM_LIFECYCLE_RISK_NOTICE.contains("local simulated"));
    assert!(PAPER_SIM_LIFECYCLE_RISK_NOTICE.contains("broker behavior may differ"));

    assert!(QMT_LIVE_RISK_NOTICE.contains("[qmt_live]"));
    assert!(QMT_LIVE_RISK_NOTICE.contains("real-money"));
    assert!(QMT_LIVE_RISK_NOTICE.contains("miniQMT"));
    assert!(QMT_LIVE_RISK_NOTICE.contains("broker state"));
}

#[test]
fn execution_mode_storage_namespaces_are_stable_and_disjoint() {
    let namespaces = [
        storage_namespace_for_channel(PAPER_IMMEDIATE_CHANNEL).unwrap(),
        storage_namespace_for_channel(PAPER_SIM_LIFECYCLE_CHANNEL).unwrap(),
        storage_namespace_for_channel(QMT_LIVE_CHANNEL).unwrap(),
    ];

    assert_eq!(
        namespaces,
        ["paper-immediate", "paper-sim-lifecycle", "qmt-live"]
    );

    for namespace in namespaces {
        assert!(
            namespace
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-'),
            "storage namespace must be path-segment safe: {namespace}"
        );
    }

    assert!(storage_namespace_for_channel("unknown").is_none());
}
