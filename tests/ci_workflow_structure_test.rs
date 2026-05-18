use std::fs;

#[test]
fn ci_workflow_is_split_by_pr_push_and_main_weight() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml")
        .expect("should read .github/workflows/ci.yml");

    assert!(
        workflow.contains("pull_request:"),
        "workflow should still support pull_request"
    );
    assert!(
        workflow.contains("push:"),
        "workflow should still support push"
    );
    assert!(
        workflow.contains("\n  coverage:\n"),
        "workflow should define a dedicated coverage job"
    );
    assert!(
        workflow.contains("\n  dependency_outdated:\n"),
        "workflow should define a dedicated dependency_outdated job"
    );
    assert!(
        !workflow.contains(
            "- name: Check documentation\n        if: github.event_name != 'pull_request'"
        ),
        "lint documentation check should remain part of the pull_request path"
    );
    assert!(
        workflow.contains("build:\n") && workflow.contains("github.event_name == 'push'"),
        "build job should be gated to push events"
    );
    assert!(
        workflow.contains("bench:\n") && workflow.contains("github.ref == 'refs/heads/main'"),
        "bench job should remain main-only"
    );
}
