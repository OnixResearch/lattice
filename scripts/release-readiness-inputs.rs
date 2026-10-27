#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-release-inputs nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc -c cargo -q -Zscript
---cargo
[package]
name = "lattice-release-readiness-inputs"
version = "0.1.0"
edition = "2024"

[dependencies]
blake3 = "1.8.3"
serde_json = "1.0.149"
---

#[path = "release_readiness/core.rs"]
mod core;

use serde_json::{Value, json};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
const OUTPUT: &str = "target/release-readiness";
const CONTRACT: &str = "target/release-readiness/mcp-contract.json";
const RESULTS: &str = "target/release-readiness/mcp-results.json";

fn main() -> ExitCode {
    match run() {
        Ok(passed) => if passed { ExitCode::SUCCESS } else { ExitCode::FAILURE },
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
    let native = fs::canonicalize(&args[0])?;
    let root = std::env::current_dir()?.canonicalize()?;
    let policy_path = root.join("cairn-policy/generated/cairn-policy.json");
    let policy_bytes = read_bounded(&policy_path)?;
    let policy: Value = serde_json::from_slice(&policy_bytes)?;
    let selected = &policy["release_readiness_policy"];
    if selected["mcp_completion_contract"] != CONTRACT || selected["mcp_completion_results"] != RESULTS {
        return Err("selected release policy must declare the maintained target/release-readiness MCP observation outputs".into());
    }
    let output = root.join(OUTPUT);
    fs::create_dir_all(&output)?;
    if !output.canonicalize()?.starts_with(&root) {
        return Err("release observation output escapes repository root".into());
    }
    // An interrupted or rejected new observation must not leave old completion
    // inputs eligible for a later assessment.
    for path in [root.join(CONTRACT), root.join(RESULTS)] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let subject = source_manifest(&root)?;
    let subject_hash = core::json_hash(&subject);
    let mut native_hasher = blake3::Hasher::new();
    std::io::copy(&mut fs::File::open(&native)?, &mut native_hasher)?;
    let native_binary_hash = native_hasher.finalize().to_hex().to_string();
    let request = json!({"root": root, "policy": policy_path});
    let command = vec![native.to_str().ok_or("native path is not UTF-8")?.to_string(),
        "mcp".to_string(), "--call".to_string(), "cairn_validate".to_string(), request.to_string()];
    let mut child = Command::new("timeout").arg("60s").args(&command)
        .current_dir(&root).env_remove("CAIRN_GLOBAL_REGISTRY")
        .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdout = child.stdout.take().ok_or("native stdout is unavailable")?;
    let stderr = child.stderr.take().ok_or("native stderr is unavailable")?;
    let stdout_reader = std::thread::spawn(move || read_stream(stdout));
    let stderr_reader = std::thread::spawn(move || read_stream(stderr));
    let status = child.wait()?;
    let stdout = stdout_reader.join().map_err(|_| "native stdout reader failed")??;
    let stderr = stderr_reader.join().map_err(|_| "native stderr reader failed")??;
    let validation: Value = serde_json::from_slice(&stdout)?;
    let observation = json!({
        "schema": "lattice.release-mcp-observation.v1", "command": command,
        "exit_code": status.code().ok_or("native observation terminated without an exit code")?,
        "native_binary_hash": native_binary_hash, "subject_hash": subject_hash,
        "policy_hash": core::json_hash(&policy),
        "stdout_hash": blake3::hash(&stdout).to_hex().to_string(),
        "stderr_hash": blake3::hash(&stderr).to_hex().to_string(),
        "boundary": "actual native MCP lifecycle validation only; heavyweight gates, source correctness and release authorization are not inferred",
    });
    let inputs = core::completion_inputs(&policy, &command, &validation, &observation, &subject_hash)?;
    if source_manifest(&root)? != subject || read_bounded(&policy_path)? != policy_bytes {
        return Err("repository or policy changed during MCP observation; no completion inputs written".into());
    }
    // Retain real bytes before their referencing completion records.
    write_output(&output.join("validation.json"), &stdout)?;
    write_output(&output.join("validation.stderr"), &stderr)?;
    write_json(&output.join("source-manifest.json"), &subject)?;
    write_json(&output.join("observation.json"), &observation)?;
    write_json(&root.join(CONTRACT), &inputs.contract)?;
    write_json(&root.join(RESULTS), &inputs.results)?;
    println!("{}", json!({"command": "lattice release MCP input observation", "passed": inputs.passed,
        "subject_hash": subject_hash, "policy_hash": core::json_hash(&policy),
        "observation_hash": core::json_hash(&observation), "contract": CONTRACT, "results": RESULTS}));
    Ok(inputs.passed)
}

fn read_stream(reader: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(std::io::Error::other("native output exceeds the 16 MiB observation bound"));
    }
    Ok(bytes)
}

fn read_bounded(path: &Path) -> std::io::Result<Vec<u8>> {
    read_stream(fs::File::open(path)?)
}

fn write_json(path: &Path, value: &Value) -> std::io::Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_output(path, &bytes)
}

fn write_output(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(std::io::Error::other("observation outputs must not be symlinks"));
    }
    fs::write(path, bytes)
}

fn source_manifest(root: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let mut pending = ["src", "crates", "tests", "scripts", "examples", "docs", ".cairn", "cairn-policy",
        "fixtures/release-readiness", "Cargo.toml", "Cargo.lock", "flake.nix", "flake.lock", "README.md"]
        .into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let mut files = std::collections::BTreeMap::new();
    while let Some(relative) = pending.pop() {
        let path = root.join(&relative);
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(format!("release source manifest rejects symlink {}", relative.display()).into());
        }
        if metadata.is_dir() {
            for entry in fs::read_dir(&path)? {
                let name = entry?.file_name();
                if name != "target" && name != ".git" && name != ".direnv" {
                    pending.push(relative.join(name));
                }
            }
        } else if metadata.is_file() {
            let digest = blake3::hash(&read_bounded(&path)?).to_hex().to_string();
            files.insert(relative.to_string_lossy().into_owned(), digest);
        } else {
            return Err(format!("release source manifest rejects special file {}", relative.display()).into());
        }
    }
    Ok(json!({"schema": "lattice.release-source-manifest.v1", "files": files}))
}
