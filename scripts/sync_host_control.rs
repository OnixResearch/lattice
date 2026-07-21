#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-sync-host-control-target nix shell "github:nix-community/fenix?rev=092bd452904e749efa39907aa4a20a42678ac31e#minimal.toolchain" nixpkgs#gcc nixpkgs#clang nixpkgs#mold -c cargo -q -Zscript

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SOURCE_ARGUMENT_INDEX: usize = 1;
const DESTINATION_ARGUMENT_INDEX: usize = 2;
const EXPECTED_REVISION: &str = "e359ccb80a52efb860307278510fa5297ac970ed";
const SOURCE_CRATE_PATH: &str = "crates/host-control-core";
const SOURCE_DIRECTORY_NAME: &str = "src";
const CARGO_MANIFEST_FILE_NAME: &str = "Cargo.toml";
const SOURCE_METADATA_FILE_NAME: &str = "source.ncl";
const GIT_PROGRAM: &str = "git";
const FAILURE_EXIT_CODE: u8 = 1;

const NORMALIZED_MANIFEST: &str = r#"[package]
name = "host-control-core"
description = "Transport-neutral no-std host topology, observation, layout, event, wait, and envelope contracts."
edition = "2021"
license = "MIT OR Apache-2.0"
rust-version = "1.85"
version = "0.1.0"

[features]
default = []
serde = ["dep:serde"]
preserves = ["serde", "dep:preserves"]

[dependencies]
blake3 = { version = "1.8.2", default-features = false }
preserves = { version = "4.996.0", optional = true }
serde = { version = "1.0.228", default-features = false, features = ["alloc", "derive"], optional = true }

[lints.rust]
unsafe_code = "forbid"
missing_docs = "deny"

[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
module_name_repetitions = "allow"
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
    verify_source_checkout(&source_root)?;
    replace_snapshot(&source_root, &destination_root)?;
    println!("synced host-control-core revision {EXPECTED_REVISION} into {}", destination_root.display());
    Ok(())
}

fn parse_arguments(arguments: &[String]) -> Result<(PathBuf, PathBuf), String> {
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: scripts/sync_host_control.rs SOURCE_REPOSITORY DESTINATION_DIRECTORY".to_string());
    }
    let source_root = PathBuf::from(&arguments[SOURCE_ARGUMENT_INDEX]);
    let destination_root = PathBuf::from(&arguments[DESTINATION_ARGUMENT_INDEX]);
    if source_root == destination_root {
        return Err("source and destination must differ".to_string());
    }
    Ok((source_root, destination_root))
}

fn verify_source_checkout(source_root: &Path) -> Result<(), String> {
    let revision = git_output(source_root, &["rev-parse", "HEAD"])?;
    if revision != EXPECTED_REVISION {
        return Err(format!("host-control revision mismatch: expected {EXPECTED_REVISION}, observed {revision}"));
    }
    let status = git_output(source_root, &["status", "--porcelain"])?;
    if !status.is_empty() {
        return Err("host-control source checkout has uncommitted changes".to_string());
    }
    Ok(())
}

fn git_output(source_root: &Path, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(GIT_PROGRAM)
        .args(arguments)
        .current_dir(source_root)
        .output()
        .map_err(|error| format!("run git in {}: {error}", source_root.display()))?;
    if !output.status.success() {
        return Err(format!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&output.stderr).trim()));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn replace_snapshot(source_root: &Path, destination_root: &Path) -> Result<(), String> {
    let source_crate = source_root.join(SOURCE_CRATE_PATH);
    let source_directory = source_crate.join(SOURCE_DIRECTORY_NAME);
    if !source_directory.is_dir() {
        return Err(format!("missing source directory {}", source_directory.display()));
    }
    if destination_root.exists() {
        fs::remove_dir_all(destination_root)
            .map_err(|error| format!("remove {}: {error}", destination_root.display()))?;
    }
    fs::create_dir_all(destination_root).map_err(|error| format!("create {}: {error}", destination_root.display()))?;
    copy_directory(&source_directory, &destination_root.join(SOURCE_DIRECTORY_NAME))?;
    fs::write(destination_root.join(CARGO_MANIFEST_FILE_NAME), NORMALIZED_MANIFEST)
        .map_err(|error| format!("write normalized Cargo manifest: {error}"))?;
    fs::write(destination_root.join(SOURCE_METADATA_FILE_NAME), source_metadata())
        .map_err(|error| format!("write source metadata: {error}"))?;
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
            fs::copy(&source_path, &destination_path)
                .map_err(|error| format!("copy {}: {error}", source_path.display()))?;
        } else {
            return Err(format!("unsupported snapshot entry {}", source_path.display()));
        }
    }
    Ok(())
}

fn source_metadata() -> String {
    format!(
        r#"let Source = {{
  repository | String,
  revision | String,
  crate_path | String,
  normalized_manifest | Bool,
}} in
{{
  repository = "ssh://git@github.com/OnixResearch/host-control.git",
  revision = "{EXPECTED_REVISION}",
  crate_path = "{SOURCE_CRATE_PATH}",
  normalized_manifest = true,
}} | Source
"#
    )
}
