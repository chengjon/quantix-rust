use super::*;

#[test]
fn runtime_install_services_script_reuses_service_list() {
    let script_path = "scripts/runtime/install-services.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert!(
        script.contains("SERVICES=("),
        "expected {script_path} to define one shared service list"
    );
    assert!(
        script
            .matches(r#"for service in "${SERVICES[@]}"; do"#)
            .count()
            >= 2,
        "expected {script_path} to reuse the shared service list for enablement and display"
    );
    assert!(
        script.contains(r#"systemctl enable "$service" 2>/dev/null || true"#),
        "expected {script_path} to enable services from the shared list"
    );
    for service in [
        "quantix-data-collector.service",
        "quantix-strategy-runner.service",
        "quantix-task-scheduler.service",
    ] {
        assert_eq!(
            script.matches(service).count(),
            1,
            "expected {service} to appear only in the shared service list in {script_path}"
        );
    }
    assert!(
        script.contains("./scripts/runtime/services.sh"),
        "expected {script_path} to point operators at the runtime service manager"
    );
    assert!(
        !script.contains("./scripts/services.sh"),
        "expected {script_path} not to reference the old service manager path"
    );
}

#[test]
fn runtime_services_script_reuses_service_validation() {
    let script_path = "scripts/runtime/services.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert!(
        script.contains("require_valid_service()"),
        "expected {script_path} to define one shared service validation helper"
    );
    for action in [
        "start", "stop", "restart", "status", "logs", "enable", "disable",
    ] {
        let case_arm = format!("{action})");
        let arm_start = script
            .find(&case_arm)
            .unwrap_or_else(|| panic!("expected {script_path} to define {action} action"));
        let arm = &script[arm_start..];
        let arm_end = arm.find(";;").unwrap_or_else(|| {
            panic!("expected {script_path} {action} action to terminate with ;;")
        });
        assert!(
            arm[..arm_end].contains(r#"require_valid_service "$SERVICE""#),
            "expected {script_path} {action} action to use the shared service validator"
        );
    }
    assert_eq!(
        script.matches(r#"if [ -z "$SERVICE" ]"#).count(),
        0,
        "expected {script_path} not to duplicate missing service checks in each action"
    );
    assert_eq!(
        script
            .matches(r#"if [ -z "${SERVICES[$SERVICE]}" ]"#)
            .count(),
        0,
        "expected {script_path} not to duplicate unknown service checks in each action"
    );
    assert!(
        script.contains(r#"get_service_name "$1""#),
        "expected {script_path} service lookups to quote the requested service name"
    );
}
