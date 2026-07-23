#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-sync-rats-target nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript

use std::collections::BTreeSet;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SOURCE_ARGUMENT_INDEX: usize = 1;
const DESTINATION_ARGUMENT_INDEX: usize = 2;
const SELF_TEST_ARGUMENT_COUNT: usize = 2;
const SELF_TEST_ARGUMENT: &str = "--self-test";
const EXPECTED_REVISION: &str = "5b932c689431cedb05428efe727889455ebe0beb";
const GIT_PROGRAM: &str = "git";
const FAILURE_EXIT_CODE: u8 = 1;
const STAGING_PREFIX: &str = ".rats-sync-stage";
const WORKTREE_PREFIX: &str = "lattice-rats-sync-worktree";
const SOURCE_METADATA_FILE_NAME: &str = "SOURCE";
const SOURCE_REPOSITORY: &str = "https://github.com/brittonr/rats";
const SNAPSHOT_PATHS: &[&str] = &[
    "docs/generated/snapshots",
    "ratcore/Cargo.toml",
    "ratcore/src",
    "subwayrat/crates/rat-nodegraph/Cargo.toml",
    "subwayrat/crates/rat-nodegraph/src",
    "subwayrat/crates/rat-widgets/Cargo.toml",
    "subwayrat/crates/rat-widgets/src",
    "subwayrat/crates/rat-keymap/Cargo.toml",
    "subwayrat/crates/rat-keymap/src",
    "subwayrat/crates/rat-fuzzy/Cargo.toml",
    "subwayrat/crates/rat-fuzzy/src",
    "subwayrat/crates/rat-streaming/Cargo.toml",
    "subwayrat/crates/rat-streaming/src",
    "subwayrat/crates/rat-layers/Cargo.toml",
    "subwayrat/crates/rat-layers/src",
    "subwayrat/crates/rat-canvas/Cargo.toml",
    "subwayrat/crates/rat-canvas/src",
    "subwayrat/crates/rat-tree/Cargo.toml",
    "subwayrat/crates/rat-tree/src",
    "subwayrat/crates/rat-table/Cargo.toml",
    "subwayrat/crates/rat-table/src",
    "subwayrat/crates/rat-editor/Cargo.toml",
    "subwayrat/crates/rat-editor/src",
    "subwayrat/crates/rat-chrome/Cargo.toml",
    "subwayrat/crates/rat-chrome/src",
    "subwayrat/crates/rat-spinner/Cargo.toml",
    "subwayrat/crates/rat-spinner/src",
    "subwayrat/crates/rat-inline/Cargo.toml",
    "subwayrat/crates/rat-inline/src",
    "subwayrat/crates/rat-markdown/Cargo.toml",
    "subwayrat/crates/rat-markdown/src",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run() -> Result<(), String> {
    let arguments = std::env::args().collect::<Vec<_>>();
    if arguments.len() == SELF_TEST_ARGUMENT_COUNT && arguments[1] == SELF_TEST_ARGUMENT {
        self_test()?;
        println!("Rats sync self-test passed");
        return Ok(());
    }
    let (source_root, destination_root) = parse_arguments(&arguments)?;
    validate_snapshot_paths(SNAPSHOT_PATHS)?;
    verify_revision_exists(&source_root)?;
    verify_source_head(&source_root)?;
    let process_id = std::process::id();
    let worktree_root = std::env::temp_dir().join(format!("{WORKTREE_PREFIX}-{process_id}"));
    let destination_parent = destination_root
        .parent()
        .ok_or_else(|| "Rats destination must have a parent directory".to_string())?;
    let staging_root = destination_parent.join(format!("{STAGING_PREFIX}-{process_id}"));
    clean_path(&worktree_root)?;
    clean_path(&staging_root)?;
    add_worktree(&source_root, &worktree_root)?;
    let snapshot_result = stage_snapshot(&worktree_root, &staging_root)
        .and_then(|()| replace_destination(&staging_root, &destination_root));
    let remove_result = remove_worktree(&source_root, &worktree_root);
    snapshot_result?;
    remove_result?;
    println!("synced Rats revision {EXPECTED_REVISION} into {}", destination_root.display());
    Ok(())
}

fn parse_arguments(arguments: &[String]) -> Result<(PathBuf, PathBuf), String> {
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: scripts/sync_rats.rs SOURCE_REPOSITORY DESTINATION_DIRECTORY".to_string());
    }
    let source_root = PathBuf::from(&arguments[SOURCE_ARGUMENT_INDEX]);
    let destination_root = PathBuf::from(&arguments[DESTINATION_ARGUMENT_INDEX]);
    if source_root == destination_root {
        return Err("source and destination must differ".to_string());
    }
    Ok((source_root, destination_root))
}

fn validate_snapshot_paths(paths: &[&str]) -> Result<(), String> {
    let mut unique = BTreeSet::new();
    for path in paths {
        let candidate = Path::new(path);
        if candidate.as_os_str().is_empty()
            || candidate.is_absolute()
            || candidate.components().any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(format!("snapshot path is not safely relative: {path}"));
        }
        if !unique.insert(*path) {
            return Err(format!("duplicate snapshot path: {path}"));
        }
    }
    Ok(())
}

fn verify_revision_exists(source_root: &Path) -> Result<(), String> {
    run_git(source_root, &["cat-file", "-e", &format!("{EXPECTED_REVISION}^{{commit}}")])
}

fn verify_source_head(source_root: &Path) -> Result<(), String> {
    let actual_revision = run_git_stdout(source_root, &["rev-parse", "HEAD"])?;
    if actual_revision == EXPECTED_REVISION {
        Ok(())
    } else {
        Err(format!(
            "Rats source HEAD {actual_revision} differs from pinned revision {EXPECTED_REVISION}; review and update the pin"
        ))
    }
}

fn add_worktree(source_root: &Path, worktree_root: &Path) -> Result<(), String> {
    let worktree = worktree_root.to_str().ok_or_else(|| "Rats worktree path is not UTF-8".to_string())?;
    run_git(source_root, &["worktree", "add", "--detach", "--force", worktree, EXPECTED_REVISION])
}

fn remove_worktree(source_root: &Path, worktree_root: &Path) -> Result<(), String> {
    let worktree = worktree_root.to_str().ok_or_else(|| "Rats worktree path is not UTF-8".to_string())?;
    run_git(source_root, &["worktree", "remove", "--force", worktree])
}

fn run_git(source_root: &Path, arguments: &[&str]) -> Result<(), String> {
    run_git_output(source_root, arguments).map(|_| ())
}

fn run_git_stdout(source_root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = run_git_output(source_root, arguments)?;
    String::from_utf8(output.stdout)
        .map(|stdout| stdout.trim().to_string())
        .map_err(|_| "git stdout was not UTF-8".to_string())
}

fn run_git_output(source_root: &Path, arguments: &[&str]) -> Result<std::process::Output, String> {
    let output = Command::new(GIT_PROGRAM)
        .args(arguments)
        .current_dir(source_root)
        .output()
        .map_err(|error| format!("run git in {}: {error}", source_root.display()))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&output.stderr).trim()))
    }
}

fn stage_snapshot(worktree_root: &Path, staging_root: &Path) -> Result<(), String> {
    fs::create_dir_all(staging_root).map_err(|error| format!("create {}: {error}", staging_root.display()))?;
    for relative in SNAPSHOT_PATHS {
        let source = worktree_root.join(relative);
        let destination = staging_root.join(relative);
        if source.is_dir() {
            copy_directory(&source, &destination)?;
        } else if source.is_file() {
            copy_file(&source, &destination)?;
        } else {
            return Err(format!("required Rats snapshot path is missing: {}", source.display()));
        }
    }
    fs::write(staging_root.join(SOURCE_METADATA_FILE_NAME), source_metadata())
        .map_err(|error| format!("write Rats source metadata: {error}"))?;
    Ok(())
}

fn replace_destination(staging_root: &Path, destination_root: &Path) -> Result<(), String> {
    if destination_root.exists() {
        fs::remove_dir_all(destination_root)
            .map_err(|error| format!("remove {}: {error}", destination_root.display()))?;
    }
    fs::rename(staging_root, destination_root)
        .map_err(|error| format!("move staged Rats snapshot into {}: {error}", destination_root.display()))
}

fn clean_path(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| format!("remove {}: {error}", path.display()))?;
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("create {}: {error}", destination.display()))?;
    for entry in fs::read_dir(source).map_err(|error| format!("read {}: {error}", source.display()))? {
        let entry = entry.map_err(|error| format!("read directory entry: {error}"))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type().map_err(|error| format!("read file type: {error}"))?;
        if file_type.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            copy_file(&source_path, &destination_path)?;
        } else {
            return Err(format!("unsupported snapshot entry {}", source_path.display()));
        }
    }
    Ok(())
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::copy(source, destination)
        .map(|_bytes_copied| ())
        .map_err(|error| format!("copy {} to {}: {error}", source.display(), destination.display()))
}

fn source_metadata() -> String {
    let paths = SNAPSHOT_PATHS.iter().map(|path| format!("- {path}")).collect::<Vec<_>>().join("\n");
    format!("name: rats\nrepository: {SOURCE_REPOSITORY}\nrevision: {EXPECTED_REVISION}\npaths:\n{paths}\n")
}

fn self_test() -> Result<(), String> {
    validate_snapshot_paths(SNAPSHOT_PATHS)?;
    if !source_metadata().contains(&format!("revision: {EXPECTED_REVISION}"))
        || !source_metadata().contains("- ratcore/src")
        || !source_metadata().contains("- docs/generated/snapshots")
    {
        return Err("source metadata omitted required identity or paths".to_string());
    }
    if validate_snapshot_paths(&["ratcore/src", "ratcore/src"]).is_ok() {
        return Err("duplicate snapshot path was accepted".to_string());
    }
    if validate_snapshot_paths(&["../outside"]).is_ok() || validate_snapshot_paths(&["/absolute"]).is_ok() {
        return Err("unsafe snapshot path was accepted".to_string());
    }
    Ok(())
}
