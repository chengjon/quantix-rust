use std::fs;

#[test]
fn ci_workflow_is_split_by_pr_push_and_main_weight() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml")
        .expect("should read .github/workflows/ci.yml");
    let audit_workflow = fs::read_to_string(".github/workflows/audit.yml")
        .expect("should read .github/workflows/audit.yml");

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
        !workflow.contains("\n  security:\n"),
        "ci workflow should not keep a duplicate security audit job"
    );
    assert!(
        !workflow.contains("\n  dependency_outdated:\n"),
        "ci workflow should not keep a duplicate dependency_outdated job"
    );
    assert!(
        audit_workflow.contains("\n  audit:\n"),
        "audit workflow should own the dedicated audit job"
    );
    assert!(
        audit_workflow.contains("\n  outdated:\n"),
        "audit workflow should own the dedicated outdated dependency job"
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

#[test]
fn cleanup_workflow_uses_repository_context() {
    let workflow = fs::read_to_string(".github/workflows/cleanup.yml")
        .expect("should read .github/workflows/cleanup.yml");

    assert!(
        !workflow.contains("chengjon/quantix-rust") && !workflow.contains("/repos/chengjon/"),
        "cleanup workflow should not hard-code the source repository"
    );
    assert!(
        workflow.contains("GH_REPO: ${{ github.repository }}"),
        "cleanup workflow should derive the GitHub repository from workflow context"
    );
    assert!(
        workflow.contains("DOCKER_REPO: ghcr.io/${{ github.repository }}"),
        "cleanup workflow should derive the GHCR repository from workflow context"
    );
    assert!(
        workflow.contains("/repos/${GH_REPO}/packages")
            && workflow.contains("/repos/${GH_REPO}/packages/$pkg_id/versions"),
        "cleanup workflow package API calls should use the derived repository"
    );
}

#[test]
fn docker_workflow_uses_repository_context() {
    let workflow = fs::read_to_string(".github/workflows/docker.yml")
        .expect("should read .github/workflows/docker.yml");

    assert!(
        !workflow.contains("chengjon/quantix-rust") && !workflow.contains("ghcr.io/chengjon"),
        "docker workflow should not hard-code the source or image repository"
    );
    assert!(
        workflow.contains("GH_REPO: ${{ github.repository }}"),
        "docker workflow should derive the GitHub repository from workflow context"
    );
    assert!(
        workflow.contains("REPO_URL: https://github.com/${{ github.repository }}"),
        "docker workflow should derive GitHub links from workflow context"
    );
    assert!(
        workflow.contains("DOCKER_REPO: ghcr.io/${{ github.repository }}"),
        "docker workflow should derive the GHCR repository from workflow context"
    );
    assert!(
        workflow.contains("${{ env.DOCKER_REPO }}/${{ env.DOCKER_IMAGE }}:latest")
            && workflow.contains("${{ env.REPO_URL }}/blob/main/CHANGELOG.md"),
        "docker workflow release notes should use derived image and repository URLs"
    );
}
