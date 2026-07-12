use super::*;

#[test]
fn production_compose_requires_explicit_image_name() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));

    assert!(
        compose.contains("image: ${IMAGE_NAME:?IMAGE_NAME required}:${VERSION:-latest}"),
        "expected {compose_path} to require IMAGE_NAME instead of publishing a default image"
    );
    assert!(
        !compose.contains("image: ghcr.io/chengjon/quantix-rust/quantix:${VERSION:-latest}")
            && !compose.contains("${IMAGE_NAME:-ghcr.io/chengjon/quantix-rust/quantix}"),
        "expected {compose_path} not to default production deployments to the maintainer image"
    );

    for relative_path in [
        "docs/guides/PRODUCTION_DEPLOYMENT.md",
        "docs/operations/DEPLOYMENT.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        assert!(
            content.contains("IMAGE_NAME=ghcr.io/your-org/quantix-rust/quantix"),
            "expected {relative_path} to document the required IMAGE_NAME setting"
        );
    }
}

#[test]
fn production_compose_quantix_service_has_single_labels_block() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));
    let quantix_service = compose
        .split("\n  postgres:\n")
        .next()
        .expect("expected quantix service to precede postgres service");

    let labels_blocks = quantix_service
        .lines()
        .filter(|line| *line == "    labels:")
        .count();

    assert_eq!(
        labels_blocks, 1,
        "expected quantix service in {compose_path} to have a single labels block"
    );
    assert!(
        quantix_service.contains("\"prometheus.io/scrape=true\"")
            && quantix_service.contains("\"traefik.enable=true\""),
        "expected the single quantix labels block in {compose_path} to retain prometheus and traefik labels"
    );
}

#[test]
fn production_compose_requires_explicit_public_hosts() {
    let compose_path = "docker-compose.prod.yml";
    let compose = fs::read_to_string(repo_root().join(compose_path))
        .unwrap_or_else(|_| panic!("expected {compose_path} to be readable"));

    for expected in [
        r#"Host(`${QUANTIX_PUBLIC_HOST:?QUANTIX_PUBLIC_HOST required}`)"#,
        r#"Host(`${GRAFANA_PUBLIC_HOST:?GRAFANA_PUBLIC_HOST required}`)"#,
        r#"Host(`${TRAEFIK_PUBLIC_HOST:?TRAEFIK_PUBLIC_HOST required}`)"#,
        "GF_SERVER_ROOT_URL=https://${GRAFANA_PUBLIC_HOST:?GRAFANA_PUBLIC_HOST required}",
    ] {
        assert!(
            compose.contains(expected),
            "expected {compose_path} to require explicit public host setting {expected}"
        );
    }

    for stale_host in [
        "quantix.example.com",
        "grafana.example.com",
        "traefik.example.com",
    ] {
        assert!(
            !compose.contains(stale_host),
            "expected {compose_path} not to publish placeholder host {stale_host}"
        );
    }

    for relative_path in [
        "docs/guides/PRODUCTION_DEPLOYMENT.md",
        "docs/operations/DEPLOYMENT.md",
    ] {
        let content = fs::read_to_string(repo_root().join(relative_path))
            .unwrap_or_else(|_| panic!("expected {relative_path} to be readable"));

        for expected in [
            "QUANTIX_PUBLIC_HOST=quantix.your-domain.com",
            "GRAFANA_PUBLIC_HOST=grafana.your-domain.com",
            "TRAEFIK_PUBLIC_HOST=traefik.your-domain.com",
        ] {
            assert!(
                content.contains(expected),
                "expected {relative_path} to document required public host setting {expected}"
            );
        }

        for stale_url in [
            "https://quantix.example.com",
            "https://grafana.example.com",
            "https://traefik.example.com",
        ] {
            assert!(
                !content.contains(stale_url),
                "expected {relative_path} not to publish placeholder URL {stale_url}"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn deploy_script_derives_default_image_name_from_github_repository() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-script-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");
    let production_k8s_dir = temp_dir.join("k8s/overlays/production");
    fs::create_dir_all(&production_k8s_dir).expect("expected fake production k8s dir");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    let output = std::process::Command::new("bash")
        .arg(repo_root().join("scripts/deploy/deploy.sh"))
        .arg("--environment")
        .arg("production")
        .arg("--dry-run")
        .env("PATH", path)
        .env("GITHUB_REPOSITORY", "example-org/example-repo")
        .env("PRODUCTION_K8S_DIR", &production_k8s_dir)
        .env_remove("IMAGE_NAME")
        .output()
        .expect("expected deploy script dry-run to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&temp_dir);

    assert!(
        output.status.success(),
        "expected deploy script dry-run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("ghcr.io/example-org/example-repo/quantix:latest"),
        "expected deploy script to derive image name from GITHUB_REPOSITORY\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("ghcr.io/chengjon/quantix-rust/quantix"),
        "expected deploy script not to default to the maintainer image\nstdout:\n{stdout}"
    );
}

#[cfg(unix)]
#[test]
fn deploy_script_reuses_configured_endpoint_values() {
    use std::os::unix::fs::PermissionsExt;

    let script_path = "scripts/deploy/deploy.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    for expected in [
        r#"HEALTH_URL="${HEALTH_URL:-http://localhost:8080/health}""#,
        r#"DEPLOY_ACCESS_URL="${DEPLOY_ACCESS_URL:-}""#,
        r#"curl -f "$HEALTH_URL""#,
    ] {
        assert!(
            script.contains(expected),
            "expected {script_path} to contain endpoint configuration {expected}"
        );
    }
    assert!(
        !script.contains("curl -f http://localhost:8080/health"),
        "expected {script_path} not to hardcode the health check URL"
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-endpoint-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");
    let production_k8s_dir = temp_dir.join("k8s/overlays/production");
    fs::create_dir_all(&production_k8s_dir).expect("expected fake production k8s dir");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    let output = std::process::Command::new("bash")
        .arg(repo_root().join(script_path))
        .arg("--environment")
        .arg("production")
        .arg("--dry-run")
        .env("PATH", path)
        .env("GITHUB_REPOSITORY", "example-org/example-repo")
        .env("PRODUCTION_K8S_DIR", &production_k8s_dir)
        .env("DEPLOY_ACCESS_URL", "https://deploy.example.invalid")
        .env_remove("IMAGE_NAME")
        .output()
        .expect("expected deploy script dry-run to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&temp_dir);

    assert!(
        output.status.success(),
        "expected deploy script dry-run to succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("生产环境访问: https://deploy.example.invalid"),
        "expected deploy script to log configured production access URL\nstdout:\n{stdout}"
    );
    assert!(
        !stdout.contains("https://quantix.example.com"),
        "expected deploy script not to publish placeholder production access URL\nstdout:\n{stdout}"
    );
}

#[cfg(unix)]
#[test]
fn deploy_script_requires_explicit_deploy_path_configuration() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = std::env::temp_dir().join(format!(
        "quantix-deploy-path-config-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("expected system time after unix epoch")
            .as_nanos()
    ));
    let fake_bin = temp_dir.join("bin");
    fs::create_dir_all(&fake_bin).expect("expected fake bin dir to be creatable");

    for executable in ["docker", "kubectl"] {
        let path = fake_bin.join(executable);
        fs::write(&path, "#!/bin/sh\nexit 0\n")
            .unwrap_or_else(|_| panic!("expected fake {executable} to be writable"));
        let mut permissions = fs::metadata(&path)
            .unwrap_or_else(|_| panic!("expected fake {executable} metadata"))
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions)
            .unwrap_or_else(|_| panic!("expected fake {executable} to be executable"));
    }

    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    for (environment, missing_variable) in [
        ("staging", "STAGING_COMPOSE_FILE"),
        ("production", "PRODUCTION_K8S_DIR"),
    ] {
        let output = std::process::Command::new("bash")
            .arg(repo_root().join("scripts/deploy/deploy.sh"))
            .arg("--environment")
            .arg(environment)
            .arg("--dry-run")
            .env("PATH", &path)
            .env("GITHUB_REPOSITORY", "example-org/example-repo")
            .env_remove("IMAGE_NAME")
            .env_remove("STAGING_COMPOSE_FILE")
            .env_remove("PRODUCTION_K8S_DIR")
            .output()
            .unwrap_or_else(|_| {
                panic!("expected deploy script dry-run for {environment} to execute")
            });
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "expected deploy script dry-run for {environment} to reject missing {missing_variable}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        assert!(
            stdout.contains(missing_variable) || stderr.contains(missing_variable),
            "expected deploy script dry-run for {environment} to mention missing {missing_variable}\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn deployment_docs_show_explicit_deploy_path_configuration() {
    let doc_path = "docs/operations/DEPLOYMENT.md";
    let contents = fs::read_to_string(repo_root().join(doc_path))
        .unwrap_or_else(|_| panic!("expected {doc_path} to be readable"));

    for line in contents
        .lines()
        .filter(|line| line.contains("deploy.sh --environment production"))
    {
        assert!(
            line.contains("PRODUCTION_K8S_DIR="),
            "expected production deploy script example in {doc_path} to set PRODUCTION_K8S_DIR: {line}"
        );
    }
}

#[test]
fn health_check_script_reuses_response_matcher() {
    let script_path = "scripts/health-check.sh";
    let script = fs::read_to_string(repo_root().join(script_path))
        .unwrap_or_else(|_| panic!("expected {script_path} to be readable"));

    assert_eq!(
        script.matches("is_healthy_response()").count(),
        1,
        "expected {script_path} to define exactly one health response matcher"
    );
    assert!(
        script.contains(r#"printf '%s\n' "$response" | is_healthy_response"#),
        "expected curl branch in {script_path} to use the shared response matcher"
    );
    assert!(
        script.contains(r#"wget -q --timeout="$TIMEOUT" -O - "$HEALTH_URL" | is_healthy_response"#),
        "expected wget branch in {script_path} to use the shared response matcher"
    );
    assert!(
        !script.contains(
            r#"echo "$response" | grep -q '"status":"ok"' || echo "$response" | grep -q '"healthy":true'"#
        ),
        "expected {script_path} not to duplicate the curl response matcher"
    );
}
