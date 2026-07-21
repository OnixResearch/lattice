#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-sync-animus-target nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc nixpkgs#clang nixpkgs#mold -c cargo -q -Zscript

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SOURCE_ARGUMENT_INDEX: usize = 1;
const DESTINATION_ARGUMENT_INDEX: usize = 2;
const EXPECTED_REVISION: &str = "37256d392d75fab519dcc18cff8b3abc2d162bd4";
const GIT_PROGRAM: &str = "git";
const FAILURE_EXIT_CODE: u8 = 1;
const README_FILE_NAME: &str = "README.md";
const LICENSE_DIRECTORY_NAME: &str = "LICENSES";
const CONFIG_DIRECTORY_NAME: &str = "config";
const POLICY_DIRECTORY_NAME: &str = "policy";
const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";
const SOURCE_METADATA_FILE_NAME: &str = "source.ncl";
const CRATES_DIRECTORY_NAME: &str = "crates";
const CRATE_EDITION: &str = "edition = \"2024\"";
const CRATE_LICENSE: &str = "license = \"MIT OR Apache-2.0\"";
const CRATE_README: &str = "readme = \"../../README.md\"";
const CRATE_RUST_VERSION: &str = "rust-version = \"1.85\"";
const CRATE_VERSION: &str = "version = \"0.1.0\"";
const WORKSPACE_LINT_BLOCK: &str = "[lints]\nworkspace = true\n";
const EXPLICIT_LINT_BLOCK: &str = r#"[lints.rust]
missing_docs = "deny"
unsafe_code = "forbid"
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(animus_octet)'] }

[lints.clippy]
all = { level = "deny", priority = -1 }
module_name_repetitions = { level = "allow", priority = 1 }
pedantic = { level = "deny", priority = -1 }
undocumented_unsafe_blocks = "deny"
"#;
const STAGING_PREFIX: &str = ".animus-sync-stage";
const WORKTREE_PREFIX: &str = "lattice-animus-sync-worktree";
const REQUIRED_CRATES: &[&str] = &[
    "animus-adapter-observability",
    "animus-adapter-safe",
    "animus-adapters",
    "animus-core",
    "animus-router",
    "animus-runtime",
    "animus-steel",
];

const NORMALIZED_WORKSPACE_MANIFEST: &str = r#"[workspace]
members = [
  "crates/animus-adapter-observability",
  "crates/animus-adapter-safe",
  "crates/animus-adapters",
  "crates/animus-core",
  "crates/animus-router",
  "crates/animus-runtime",
  "crates/animus-steel",
]
default-members = [
  "crates/animus-adapter-observability",
  "crates/animus-adapter-safe",
  "crates/animus-adapters",
  "crates/animus-core",
  "crates/animus-router",
  "crates/animus-runtime",
  "crates/animus-steel",
]
resolver = "2"

[workspace.package]
edition = "2024"
license = "MIT OR Apache-2.0"
readme = "README.md"
rust-version = "1.85"
version = "0.1.0"

[workspace.lints.rust]
missing_docs = "deny"
unsafe_code = "forbid"
unexpected_cfgs = { level = "warn", check-cfg = ['cfg(animus_octet)'] }

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
module_name_repetitions = { level = "allow", priority = 1 }
pedantic = { level = "deny", priority = -1 }
undocumented_unsafe_blocks = "deny"
"#;

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
    let arguments: Vec<String> = std::env::args().collect();
    let (source_root, destination_root) = parse_arguments(&arguments)?;
    verify_revision_exists(&source_root)?;
    let process_id = std::process::id();
    let worktree_root = std::env::temp_dir().join(format!("{WORKTREE_PREFIX}-{process_id}"));
    let destination_parent = destination_root
        .parent()
        .ok_or_else(|| "Animus destination must have a parent directory".to_string())?;
    let staging_root = destination_parent.join(format!("{STAGING_PREFIX}-{process_id}"));
    clean_path(&worktree_root)?;
    clean_path(&staging_root)?;
    add_worktree(&source_root, &worktree_root)?;
    let snapshot_result = stage_snapshot(&worktree_root, &staging_root)
        .and_then(|()| replace_destination(&staging_root, &destination_root));
    let remove_result = remove_worktree(&source_root, &worktree_root);
    snapshot_result?;
    remove_result?;
    println!("synced Animus revision {EXPECTED_REVISION} into {}", destination_root.display());
    Ok(())
}

fn parse_arguments(arguments: &[String]) -> Result<(PathBuf, PathBuf), String> {
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: scripts/sync_animus.rs SOURCE_REPOSITORY DESTINATION_DIRECTORY".to_string());
    }
    let source_root = PathBuf::from(&arguments[SOURCE_ARGUMENT_INDEX]);
    let destination_root = PathBuf::from(&arguments[DESTINATION_ARGUMENT_INDEX]);
    if source_root == destination_root {
        return Err("source and destination must differ".to_string());
    }
    Ok((source_root, destination_root))
}

fn verify_revision_exists(source_root: &Path) -> Result<(), String> {
    run_git(source_root, &["cat-file", "-e", &format!("{EXPECTED_REVISION}^{{commit}}")])
}

fn add_worktree(source_root: &Path, worktree_root: &Path) -> Result<(), String> {
    let worktree = worktree_root.to_str().ok_or_else(|| "Animus worktree path is not UTF-8".to_string())?;
    run_git(source_root, &["worktree", "add", "--detach", "--force", worktree, EXPECTED_REVISION])
}

fn remove_worktree(source_root: &Path, worktree_root: &Path) -> Result<(), String> {
    let worktree = worktree_root.to_str().ok_or_else(|| "Animus worktree path is not UTF-8".to_string())?;
    run_git(source_root, &["worktree", "remove", "--force", worktree])
}

fn run_git(source_root: &Path, arguments: &[&str]) -> Result<(), String> {
    let output = Command::new(GIT_PROGRAM)
        .args(arguments)
        .current_dir(source_root)
        .output()
        .map_err(|error| format!("run git in {}: {error}", source_root.display()))?;
    if !output.status.success() {
        return Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(())
}

fn stage_snapshot(worktree_root: &Path, staging_root: &Path) -> Result<(), String> {
    fs::create_dir_all(staging_root).map_err(|error| format!("create {}: {error}", staging_root.display()))?;
    copy_file(&worktree_root.join(README_FILE_NAME), &staging_root.join(README_FILE_NAME))?;
    for directory_name in [LICENSE_DIRECTORY_NAME, CONFIG_DIRECTORY_NAME, POLICY_DIRECTORY_NAME] {
        copy_directory(&worktree_root.join(directory_name), &staging_root.join(directory_name))?;
    }
    let staging_crates = staging_root.join(CRATES_DIRECTORY_NAME);
    fs::create_dir_all(&staging_crates).map_err(|error| format!("create {}: {error}", staging_crates.display()))?;
    for crate_name in REQUIRED_CRATES {
        let staging_crate = staging_crates.join(crate_name);
        copy_directory(&worktree_root.join(CRATES_DIRECTORY_NAME).join(crate_name), &staging_crate)?;
        normalize_copied_manifest(&staging_crate.join(CARGO_MANIFEST_FILE_NAME))?;
    }
    fs::write(staging_root.join(CARGO_MANIFEST_FILE_NAME), NORMALIZED_WORKSPACE_MANIFEST)
        .map_err(|error| format!("write normalized Animus workspace manifest: {error}"))?;
    fs::write(staging_root.join(SOURCE_METADATA_FILE_NAME), source_metadata())
        .map_err(|error| format!("write Animus source metadata: {error}"))?;
    Ok(())
}

fn normalize_copied_manifest(manifest_path: &Path) -> Result<(), String> {
    let source =
        fs::read_to_string(manifest_path).map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let normalized = normalize_manifest(&source)?;
    fs::write(manifest_path, normalized).map_err(|error| format!("write {}: {error}", manifest_path.display()))
}

fn normalize_manifest(source: &str) -> Result<String, String> {
    let replacements = [
        ("edition.workspace = true", CRATE_EDITION),
        ("license.workspace = true", CRATE_LICENSE),
        ("readme.workspace = true", CRATE_README),
        ("rust-version.workspace = true", CRATE_RUST_VERSION),
        ("version.workspace = true", CRATE_VERSION),
        (WORKSPACE_LINT_BLOCK, EXPLICIT_LINT_BLOCK),
    ];
    let mut normalized = source.to_string();
    for (workspace_value, explicit_value) in replacements {
        if normalized.matches(workspace_value).count() != 1 {
            return Err(format!("expected one manifest field {workspace_value:?}"));
        }
        normalized = normalized.replacen(workspace_value, explicit_value, 1);
    }
    Ok(normalized)
}

fn replace_destination(staging_root: &Path, destination_root: &Path) -> Result<(), String> {
    if destination_root.exists() {
        fs::remove_dir_all(destination_root)
            .map_err(|error| format!("remove {}: {error}", destination_root.display()))?;
    }
    fs::rename(staging_root, destination_root).map_err(|error| {
        format!(
            "move staged Animus snapshot from {} to {}: {error}",
            staging_root.display(),
            destination_root.display()
        )
    })
}

fn clean_path(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|error| format!("remove {}: {error}", path.display()))?;
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("create {}: {error}", destination.display()))?;
    let entries = fs::read_dir(source).map_err(|error| format!("read {}: {error}", source.display()))?;
    for entry in entries {
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
    fs::copy(source, destination)
        .map(|_bytes_copied| ())
        .map_err(|error| format!("copy {} to {}: {error}", source.display(), destination.display()))
}

fn source_metadata() -> String {
    let crate_rows = REQUIRED_CRATES
        .iter()
        .map(|crate_name| format!("    \"{crate_name}\","))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"let Source = {{
  repository | String,
  revision | String,
  crates | Array String,
  normalized_workspace_manifest | Bool,
  normalized_crate_manifests | Bool,
  includes_compile_time_assets | Bool,
}} in
{{
  repository = "ssh://git@github.com/OnixResearch/animus.git",
  revision = "{EXPECTED_REVISION}",
  crates = [
{crate_rows}
  ],
  normalized_workspace_manifest = true,
  normalized_crate_manifests = true,
  includes_compile_time_assets = true,
}} | Source
"#
    )
}
