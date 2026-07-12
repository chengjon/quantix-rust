//! Repository hygiene integration tests, split by concern.
//!
//! Historical note: this file was once 3380 lines and was split into
//! thematic submodules per the file-size limits in CLAUDE.md. Shared
//! helpers (`repo_root`, `is_repo_doc_path`, `collect_main_workspace_doc_paths`,
//! `collect_rust_paths`) live below and are re-exported to submodules
//! via `use super::*;`.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn is_repo_doc_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("md" | "html")
    )
}

fn collect_main_workspace_doc_paths(dir: &Path, docs: &mut Vec<PathBuf>) {
    let skip_dirs = [
        ".git",
        ".gitnexus",
        ".idea",
        ".vscode",
        ".worktrees",
        ".zread",
        "archive",
        "node_modules",
        "openspec",
        "plans",
        "reports",
        "superpowers",
        "target",
    ];

    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("expected {dir:?} to be readable")) {
        let entry = entry.unwrap_or_else(|_| panic!("expected readable entry under {dir:?}"));
        let path = entry.path();

        if path.is_dir() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !skip_dirs.contains(&name) {
                collect_main_workspace_doc_paths(&path, docs);
            }
            continue;
        }

        if is_repo_doc_path(&path) {
            docs.push(path);
        }
    }
}

fn collect_rust_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|_| panic!("expected {dir:?} to be readable")) {
        let entry = entry.unwrap_or_else(|_| panic!("expected readable entry under {dir:?}"));
        let path = entry.path();

        if path.is_dir() {
            collect_rust_paths(&path, paths);
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
}

#[path = "repo_hygiene_test/architecture.rs"]
mod architecture;
#[path = "repo_hygiene_test/code_hygiene.rs"]
mod code_hygiene;
#[path = "repo_hygiene_test/generated_cli_manual.rs"]
mod generated_cli_manual;
#[path = "repo_hygiene_test/importer_exporter.rs"]
mod importer_exporter;
#[path = "repo_hygiene_test/phase29bc.rs"]
mod phase29bc;
#[path = "repo_hygiene_test/phase_docs.rs"]
mod phase_docs;
#[path = "repo_hygiene_test/production_deploy.rs"]
mod production_deploy;
#[path = "repo_hygiene_test/runtime_scripts.rs"]
mod runtime_scripts;
#[path = "repo_hygiene_test/status_function.rs"]
mod status_function;
