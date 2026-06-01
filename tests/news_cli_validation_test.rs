use std::process::Command;

const NEWS_PROVIDER_ENV_KEYS: &[&str] = &["TAVILY_API_KEY", "SERPAPI_API_KEY", "BOCHA_API_KEY"];

fn run_quantix_without_news_provider(args: &[&str]) -> (String, String, bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_quantix"));
    command.args(args);
    for key in NEWS_PROVIDER_ENV_KEYS {
        command.env_remove(key);
    }

    let output = command.output().expect("should run quantix binary");

    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

#[test]
fn news_search_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) =
        run_quantix_without_news_provider(&["news", "search", "--query", "半导体"]);

    assert!(
        !success,
        "expected news search to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder news search output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("news provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn news_code_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) =
        run_quantix_without_news_provider(&["news", "code", "--code", "600519.SH"]);

    assert!(
        !success,
        "expected news code to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder news code output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("news provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}

#[test]
fn news_trend_fails_closed_without_configured_provider() {
    let (stdout, stderr, success) = run_quantix_without_news_provider(&["news", "trend"]);

    assert!(
        !success,
        "expected news trend to fail without a configured provider, stdout={stdout}, stderr={stderr}"
    );
    assert!(
        stdout.is_empty(),
        "expected no placeholder news trend output before provider failure, stdout={stdout}"
    );
    assert!(
        stderr.contains("news provider 尚未配置"),
        "expected provider boundary in stderr, stderr={stderr}"
    );
}
