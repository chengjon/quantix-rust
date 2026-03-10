use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn ignore_file_covers_repo_local_noise() {
    let ignore_path = repo_root().join(".ignore");
    let contents = fs::read_to_string(ignore_path).expect("expected .ignore to exist");

    for entry in [".worktrees/", ".gitnexus/", "target/"] {
        assert!(
            contents.lines().any(|line| line.trim() == entry),
            "expected .ignore to contain {entry}"
        );
    }
}

#[test]
fn readme_documents_foundation_p0_workspace_constraints() {
    let readme_path = repo_root().join("README.md");
    let contents = fs::read_to_string(readme_path).expect("expected README.md to exist");

    assert!(
        contents.contains(".worktrees/"),
        "expected README to mention repo-local worktrees"
    );
    assert!(
        contents.contains("Foundation P0"),
        "expected README to mention Foundation P0"
    );
    assert!(
        contents.contains("daemon"),
        "expected README to describe daemon support boundary"
    );
    assert!(
        contents.contains("quantix market sector"),
        "expected README to advertise market sector command"
    );
    assert!(
        contents.contains("quantix market overview"),
        "expected README to advertise market overview command"
    );
    assert!(
        contents.contains("历史/详情/实时功能延后"),
        "expected README to describe deferred market features"
    );
}

#[test]
fn user_manual_documents_phase23_market_commands() {
    let manual_path = repo_root().join("docs").join("USER_MANUAL.md");
    let contents = fs::read_to_string(manual_path).expect("expected USER_MANUAL.md to exist");

    for expected in [
        "### market - 市场分析",
        "quantix market sector [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market concept [--top <N>] [--date <YYYY-MM-DD>] [--sort-by <FIELD>]",
        "quantix market north [--date <YYYY-MM-DD>]",
        "quantix market sentiment [--date <YYYY-MM-DD>]",
        "quantix market leader (--sector <NAME> | --concept <NAME> | --all) [--limit <N>] [--date <YYYY-MM-DD>]",
        "quantix market overview [--top <N>] [--date <YYYY-MM-DD>]",
        "历史/详情/实时能力延后到后续 Phase",
    ] {
        assert!(
            contents.contains(expected),
            "expected USER_MANUAL to contain {expected}"
        );
    }
}
