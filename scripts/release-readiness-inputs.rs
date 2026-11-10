#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-release-inputs nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc nixpkgs#coreutils -c cargo -q -Zscript
---cargo
[package]
name = "lattice-release-readiness-inputs"
version = "0.1.0"
edition = "2024"

[dependencies]
blake3 = "=1.8.5"
serde_json = "=1.0.149"
---

#[path = "release_readiness/core.rs"]
mod core;

use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::process::Stdio;

use serde_json::Value;
use serde_json::json;

const MAX_STREAM_BYTES: u64 = 16 * 1024 * 1024;
const NATIVE_TIMEOUT: &str = "60s";

fn main() -> ExitCode {
    match run() {
        Ok(passed) => {
            if passed {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("release input observation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

// r[impl lattice.release_integration.inputs]
fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 1 || !Path::new(&args[0]).is_absolute() {
        return Err("expected one absolute native Cairn executable path; run from the Lattice root".into());
    }
    let root = std::env::current_dir()?.canonicalize()?;
    let output = root.join(core::OUTPUT_ROOT);
    fs::create_dir_all(&output)?;
    if !output.canonicalize()?.starts_with(&root) {
        return Err("release observation output escapes repository root".into());
    }
    // An interrupted or rejected new observation must not leave old evidence or
    // completion inputs eligible for a later assessment.
    for path in [
        root.join(core::CONTRACT_PATH),
        root.join(core::RESULTS_PATH),
        root.join(core::VALIDATION_PATH),
        root.join(core::STDERR_PATH),
        root.join(core::SOURCE_MANIFEST_PATH),
        root.join(core::OBSERVATION_PATH),
    ] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let native = fs::canonicalize(&args[0])?;
    let policy_path = root.join("cairn-policy/generated/cairn-policy.json");
    let policy_bytes = read_bounded(&policy_path)?;
    let policy: Value = serde_json::from_slice(&policy_bytes)?;
    let selected = &policy["release_readiness_policy"];
    if selected["mcp_completion_contract"] != core::CONTRACT_PATH
        || selected["mcp_completion_results"] != core::RESULTS_PATH
    {
        return Err(
            "selected release policy must declare the maintained target/release-readiness MCP observation outputs"
                .into(),
        );
    }
    let subject = source_manifest(&root)?;
    let subject_hash = core::json_hash(&subject);
    let native_binary_hash = file_hash(&native)?;
    let request = json!({"root": root, "policy": policy_path});
    let command = vec![
        native.to_str().ok_or("native path is not UTF-8")?.to_string(),
        "mcp".to_string(),
        "--call".to_string(),
        "cairn_validate".to_string(),
        request.to_string(),
    ];
    let mut child = Command::new("timeout")
        .arg(NATIVE_TIMEOUT)
        .args(&command)
        .current_dir(&root)
        .env_remove("CAIRN_GLOBAL_REGISTRY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("native stdout is unavailable")?;
    let stderr = child.stderr.take().ok_or("native stderr is unavailable")?;
    let stdout_reader = std::thread::spawn(move || read_stream(stdout, "native stdout"));
    let stderr_reader = std::thread::spawn(move || read_stream(stderr, "native stderr"));
    let status = child.wait()?;
    let stdout = stdout_reader.join().map_err(|_| "native stdout reader failed")??;
    let stderr = stderr_reader.join().map_err(|_| "native stderr reader failed")??;
    let exit_code = status.code();
    let observation = json!({
        "schema": "lattice.release-mcp-observation.v1", "command": command,
        "exit_code": exit_code,
        "native_binary_hash": native_binary_hash, "subject_hash": subject_hash,
        "policy_hash": core::json_hash(&policy),
        "stdout_hash": blake3::hash(&stdout).to_hex().to_string(),
        "stderr_hash": blake3::hash(&stderr).to_hex().to_string(),
        "boundary": "actual native MCP lifecycle validation only; heavyweight gates, source correctness and release authorization are not inferred",
    });
    // Retain real bytes before parsing them or deriving completion records, so
    // malformed observations remain inspectable.
    write_output(&root.join(core::VALIDATION_PATH), &stdout)?;
    write_output(&root.join(core::STDERR_PATH), &stderr)?;
    write_json(&root.join(core::SOURCE_MANIFEST_PATH), &subject)?;
    write_json(&root.join(core::OBSERVATION_PATH), &observation)?;
    exit_code.ok_or("native observation terminated without an exit code")?;
    let validation: Value = serde_json::from_slice(&stdout)?;
    let inputs = core::completion_inputs(&policy, &command, &validation, &observation, &subject_hash)?;
    if source_manifest(&root)? != subject
        || read_bounded(&policy_path)? != policy_bytes
        || file_hash(&native)? != native_binary_hash
    {
        return Err(
            "repository, policy, or native executable changed during MCP observation; no completion inputs written"
                .into(),
        );
    }
    write_json(&root.join(core::CONTRACT_PATH), &inputs.contract)?;
    write_json(&root.join(core::RESULTS_PATH), &inputs.results)?;
    println!(
        "{}",
        json!({"command": "lattice release MCP input observation", "passed": inputs.passed,
        "subject_hash": subject_hash, "policy_hash": core::json_hash(&policy),
        "observation_hash": core::json_hash(&observation), "contract": core::CONTRACT_PATH, "results": core::RESULTS_PATH})
    );
    Ok(inputs.passed)
}

fn read_stream(reader: impl Read, label: &str) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(MAX_STREAM_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_STREAM_BYTES {
        return Err(std::io::Error::other(format!("{label} exceeds the 16 MiB observation bound")));
    }
    Ok(bytes)
}

fn read_bounded(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = fs::File::open(path)?;
    read_stream(&mut file, "file")
        .map_err(|error| std::io::Error::new(error.kind(), format!("{}: {error}", path.display())))
}

fn file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    core::check_native_size(file.metadata()?.len())
        .map_err(|error| std::io::Error::other(format!("{}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn write_json(path: &Path, value: &Value) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_output(path, &bytes)
}

fn write_output(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::other(format!("observation output must not be a symlink: {}", path.display())));
    }
    fs::write(path, bytes)
}

fn source_manifest(root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut pending = [
        "src",
        "crates",
        "tests",
        "scripts",
        "examples",
        "docs",
        ".cairn",
        "cairn-policy",
        "builtins",
        "contracts",
        "evidence",
        "policy",
        "profiles",
        "verus",
        "vendor-cargo-deps",
        "fixtures/release-readiness",
        "Cargo.toml",
        "Cargo.lock",
        "flake.nix",
        "flake.lock",
        "LICENSE",
        "README.md",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "clippy.toml",
        "dylint.toml",
        "rustfmt.toml",
        "octet-baseline.json",
        ".pre-commit-config.yaml",
        ".config/nextest.toml",
        ".github/workflows/check.yml",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect::<Vec<_>>();
    let mut files = std::collections::BTreeMap::new();
    let mut total_bytes = 0_u64;
    while let Some(relative) = pending.pop() {
        let path = root.join(&relative);
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("release source manifest rejects symlink {}", relative.display()).into());
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&path)? {
                let name = entry?.file_name();
                let vendor_metadata = (name == ".cairn" || name == ".pi" || name == ".autoresearch")
                    && relative.starts_with("vendor-cargo-deps");
                if name != "target" && name != ".git" && name != ".direnv" && !vendor_metadata {
                    pending.push(relative.join(name));
                }
            }
        } else if metadata.is_file() {
            let bytes = read_bounded(&path)?;
            total_bytes =
                total_bytes.checked_add(bytes.len() as u64).ok_or("release source manifest byte count overflow")?;
            let digest = blake3::hash(&bytes).to_hex().to_string();
            files.insert(relative.to_string_lossy().into_owned(), digest);
            core::check_manifest_bounds(files.len(), total_bytes)?;
        } else {
            return Err(format!("release source manifest rejects special file {}", relative.display()).into());
        }
    }
    Ok(json!({"schema": "lattice.release-source-manifest.v1", "files": files}))
}
