#!/usr/bin/env -S nix --option builders '' --option secret-key-files '' shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#git nixpkgs#gnutar nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript
---cargo
[package]
name = "sync-vendored-sibling"
version = "0.1.0"
edition = "2024"
---

use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;

const FLAG_SOURCE: &str = "--source";
const FLAG_SOURCE_URL: &str = "--source-url";
const FLAG_REVISION: &str = "--revision";
const FLAG_DESTINATION: &str = "--destination";
const FLAG_NAME: &str = "--name";
const FLAG_PATH: &str = "--path";
const FLAG_STRIP_PREFIX: &str = "--strip-prefix";
const SELF_TEST_FLAG: &str = "--self-test";
const VENDOR_ROOT: &str = "vendor-cargo-deps";
const PROVENANCE_FILE_NAME: &str = "SOURCE";
const ARCHIVE_FILE_NAME: &str = "source.tar";
const STAGING_DIRECTORY_NAME: &str = "staging";
const TEMP_DIRECTORY_PREFIX: &str = "lattice-vendor-sync";
const GENERIC_FAILURE_EXIT_CODE: u8 = 1;
const MIN_DESTINATION_COMPONENT_COUNT: usize = 2;
const ARGUMENT_PAIR_WIDTH: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SyncRequest {
    source: PathBuf,
    source_url: String,
    revision: String,
    destination: PathBuf,
    name: String,
    paths: Vec<PathBuf>,
    strip_prefix: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = if args.as_slice() == [SELF_TEST_FLAG] {
        run_self_tests()
    } else {
        parse_request(&args).and_then(sync_snapshot)
    };
    match result {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("vendor sync failed: {error}");
            ExitCode::from(GENERIC_FAILURE_EXIT_CODE)
        }
    }
}

fn run_self_tests() -> Result<String, String> {
    let valid = argument_strings(&[
        FLAG_SOURCE,
        "/tmp/source",
        FLAG_SOURCE_URL,
        "https://example.invalid/source",
        FLAG_REVISION,
        "abc123",
        FLAG_DESTINATION,
        "vendor-cargo-deps/core",
        FLAG_NAME,
        "core",
        FLAG_PATH,
        "core",
    ]);
    let request = parse_request(&valid)?;
    if request.destination != PathBuf::from("vendor-cargo-deps/core") {
        return Err("positive self-test parsed the wrong destination".to_string());
    }

    let mut unsafe_destination = valid;
    let destination_index = unsafe_destination
        .iter()
        .position(|value| value == FLAG_DESTINATION)
        .and_then(|index| index.checked_add(1))
        .ok_or_else(|| "negative self-test could not find destination argument".to_string())?;
    unsafe_destination[destination_index] = "../outside".to_string();
    if parse_request(&unsafe_destination).is_ok() {
        return Err("negative self-test accepted an escaping destination".to_string());
    }

    Ok("vendor sync self-tests passed".to_string())
}

fn argument_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn parse_request(args: &[String]) -> Result<SyncRequest, String> {
    let mut source = None;
    let mut source_url = None;
    let mut revision = None;
    let mut destination = None;
    let mut name = None;
    let mut paths = Vec::new();
    let mut strip_prefix = None;
    let mut index = 0usize;

    while index < args.len() {
        let flag = &args[index];
        let value_index = index.saturating_add(1);
        let value = args.get(value_index).ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            FLAG_SOURCE => set_once(&mut source, PathBuf::from(value), FLAG_SOURCE)?,
            FLAG_SOURCE_URL => set_once(&mut source_url, value.clone(), FLAG_SOURCE_URL)?,
            FLAG_REVISION => set_once(&mut revision, value.clone(), FLAG_REVISION)?,
            FLAG_DESTINATION => {
                set_once(&mut destination, PathBuf::from(value), FLAG_DESTINATION)?;
            }
            FLAG_NAME => set_once(&mut name, value.clone(), FLAG_NAME)?,
            FLAG_PATH => paths.push(PathBuf::from(value)),
            FLAG_STRIP_PREFIX => {
                set_once(&mut strip_prefix, PathBuf::from(value), FLAG_STRIP_PREFIX)?;
            }
            _ => return Err(format!("unsupported argument: {flag}")),
        }
        index = index.saturating_add(ARGUMENT_PAIR_WIDTH);
    }

    let request = SyncRequest {
        source: required(source, FLAG_SOURCE)?,
        source_url: required(source_url, FLAG_SOURCE_URL)?,
        revision: required(revision, FLAG_REVISION)?,
        destination: required(destination, FLAG_DESTINATION)?,
        name: required(name, FLAG_NAME)?,
        paths,
        strip_prefix,
    };
    validate_request(&request)?;
    Ok(request)
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("duplicate argument: {flag}"));
    }
    *slot = Some(value);
    Ok(())
}

fn required<T>(value: Option<T>, flag: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required argument: {flag}"))
}

fn validate_request(request: &SyncRequest) -> Result<(), String> {
    if request.source_url.trim().is_empty() {
        return Err("source URL cannot be empty".to_string());
    }
    if request.revision.trim().is_empty() {
        return Err("revision cannot be empty".to_string());
    }
    if request.name.trim().is_empty() {
        return Err("source name cannot be empty".to_string());
    }
    if request.paths.is_empty() {
        return Err("at least one archive path is required".to_string());
    }
    validate_vendor_destination(&request.destination)?;
    for path in &request.paths {
        validate_relative_archive_path(path)?;
    }
    if let Some(prefix) = &request.strip_prefix {
        validate_relative_archive_path(prefix)?;
    }
    Ok(())
}

fn validate_vendor_destination(path: &Path) -> Result<(), String> {
    let components = normal_components(path)?;
    if components.len() < MIN_DESTINATION_COMPONENT_COUNT {
        return Err("destination must name a child below vendor-cargo-deps/".to_string());
    }
    if components.first().is_none_or(|component| component != VENDOR_ROOT) {
        return Err("destination must remain below vendor-cargo-deps/".to_string());
    }
    Ok(())
}

fn validate_relative_archive_path(path: &Path) -> Result<(), String> {
    let components = normal_components(path)?;
    if components.is_empty() {
        return Err("archive path cannot be empty".to_string());
    }
    Ok(())
}

fn normal_components(path: &Path) -> Result<Vec<String>, String> {
    if path.is_absolute() {
        return Err(format!("absolute path is not allowed: {}", path.display()));
    }
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(format!("unsafe path component: {}", path.display()));
            }
        }
    }
    Ok(components)
}

fn sync_snapshot(request: SyncRequest) -> Result<String, String> {
    let working_directory =
        std::env::current_dir().map_err(|error| format!("failed to inspect working directory: {error}"))?;
    let source = request
        .source
        .canonicalize()
        .map_err(|error| format!("failed to resolve source repository: {error}"))?;
    let revision = resolve_revision(&source, &request.revision)?;
    let destination = working_directory.join(&request.destination);
    let temp_directory = std::env::temp_dir().join(format!("{TEMP_DIRECTORY_PREFIX}-{}", std::process::id()));
    reset_path(&temp_directory)?;
    fs::create_dir_all(&temp_directory).map_err(|error| format!("failed to create temporary directory: {error}"))?;

    let result = sync_snapshot_in_temp(&request, &source, &revision, &destination, &temp_directory);
    let cleanup_result = reset_path(&temp_directory);
    match (result, cleanup_result) {
        (Ok(message), Ok(())) => Ok(message),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

fn sync_snapshot_in_temp(
    request: &SyncRequest,
    source: &Path,
    revision: &str,
    destination: &Path,
    temp_directory: &Path,
) -> Result<String, String> {
    let archive = temp_directory.join(ARCHIVE_FILE_NAME);
    let staging = temp_directory.join(STAGING_DIRECTORY_NAME);
    fs::create_dir_all(&staging).map_err(|error| format!("failed to create staging directory: {error}"))?;
    create_archive(source, revision, &request.paths, &archive)?;
    run_status(Command::new("tar").arg("-xf").arg(&archive).arg("-C").arg(&staging), "extract source archive")?;

    let extracted_root = match &request.strip_prefix {
        Some(prefix) => staging.join(prefix),
        None => staging,
    };
    if !extracted_root.is_dir() {
        return Err(format!("extracted source root is missing: {}", extracted_root.display()));
    }

    reset_path(destination)?;
    copy_tree(&extracted_root, destination)?;
    write_provenance(destination, request, revision)?;
    Ok(format!("synced {} at {} to {}", request.name, revision, request.destination.display()))
}

fn resolve_revision(source: &Path, revision: &str) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source)
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("{revision}^{{commit}}"))
        .output()
        .map_err(|error| format!("failed to start git revision lookup: {error}"))?;
    if !output.status.success() {
        return Err(format!("source revision is unavailable: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let resolved =
        String::from_utf8(output.stdout).map_err(|error| format!("git revision output is not UTF-8: {error}"))?;
    let resolved = resolved.trim().to_string();
    if resolved.is_empty() {
        return Err("git revision lookup returned an empty revision".to_string());
    }
    Ok(resolved)
}

fn create_archive(source: &Path, revision: &str, paths: &[PathBuf], archive: &Path) -> Result<(), String> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(source)
        .arg("archive")
        .arg("--format=tar")
        .arg("--output")
        .arg(archive)
        .arg(revision)
        .arg("--");
    for path in paths {
        command.arg(path);
    }
    run_status(&mut command, "create source archive")
}

fn run_status(command: &mut Command, label: &str) -> Result<(), String> {
    let output = command.output().map_err(|error| format!("failed to {label}: {error}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!("failed to {label}: {}", String::from_utf8_lossy(&output.stderr).trim()))
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("failed to create {}: {error}", destination.display()))?;
    let mut entries = fs::read_dir(source)
        .map_err(|error| format!("failed to read {}: {error}", source.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to enumerate {}: {error}", source.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type =
            entry.file_type().map_err(|error| format!("failed to inspect {}: {error}", source_path.display()))?;
        if file_type.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|error| {
                format!("failed to copy {} to {}: {error}", source_path.display(), destination_path.display())
            })?;
        } else {
            return Err(format!("unsupported archive entry type: {}", source_path.display()));
        }
    }
    Ok(())
}

fn reset_path(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("failed to inspect {}: {error}", path.display())),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).map_err(|error| format!("failed to remove {}: {error}", path.display()))
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).map_err(|error| format!("failed to remove {}: {error}", path.display()))
    } else {
        Err(format!("unsupported destination type: {}", path.display()))
    }
}

fn write_provenance(destination: &Path, request: &SyncRequest, revision: &str) -> Result<(), String> {
    let mut text =
        format!("name: {}\nrepository: {}\nrevision: {}\npaths:\n", request.name, request.source_url, revision);
    for path in &request.paths {
        text.push_str("- ");
        text.push_str(&path.display().to_string());
        text.push('\n');
    }
    if let Some(prefix) = &request.strip_prefix {
        text.push_str("strip-prefix: ");
        text.push_str(&prefix.display().to_string());
        text.push('\n');
    }
    fs::write(destination.join(PROVENANCE_FILE_NAME), text)
        .map_err(|error| format!("failed to write provenance: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn parses_valid_single_crate_snapshot() {
        let request = parse_request(&strings(&[
            FLAG_SOURCE,
            "/tmp/source",
            FLAG_SOURCE_URL,
            "https://example.invalid/source",
            FLAG_REVISION,
            "abc123",
            FLAG_DESTINATION,
            "vendor-cargo-deps/core",
            FLAG_NAME,
            "core",
            FLAG_PATH,
            "core",
            FLAG_STRIP_PREFIX,
            "core",
        ]))
        .expect("valid request should parse");

        assert_eq!(request.destination, PathBuf::from("vendor-cargo-deps/core"));
        assert_eq!(request.paths, vec![PathBuf::from("core")]);
        assert_eq!(request.strip_prefix, Some(PathBuf::from("core")));
    }

    #[test]
    fn rejects_destination_outside_vendor_root() {
        let error = parse_request(&strings(&[
            FLAG_SOURCE,
            "/tmp/source",
            FLAG_SOURCE_URL,
            "https://example.invalid/source",
            FLAG_REVISION,
            "abc123",
            FLAG_DESTINATION,
            "../outside",
            FLAG_NAME,
            "core",
            FLAG_PATH,
            "core",
        ]))
        .expect_err("escaping destination should be rejected");

        assert!(error.contains("unsafe path component"));
    }

    #[test]
    fn rejects_absolute_or_missing_archive_paths() {
        let absolute = validate_relative_archive_path(Path::new("/absolute"))
            .expect_err("absolute archive path should be rejected");
        let empty = validate_relative_archive_path(Path::new(".")).expect_err("empty archive path should be rejected");

        assert!(absolute.contains("absolute path"));
        assert!(empty.contains("cannot be empty"));
    }

    #[test]
    fn rejects_duplicate_single_value_argument() {
        let error = parse_request(&strings(&[
            FLAG_SOURCE,
            "/tmp/source",
            FLAG_SOURCE,
            "/tmp/other",
            FLAG_SOURCE_URL,
            "https://example.invalid/source",
            FLAG_REVISION,
            "abc123",
            FLAG_DESTINATION,
            "vendor-cargo-deps/core",
            FLAG_NAME,
            "core",
            FLAG_PATH,
            "core",
        ]))
        .expect_err("duplicate source should be rejected");

        assert!(error.contains("duplicate argument"));
    }

    #[test]
    fn provenance_omits_local_source_path() {
        let request = SyncRequest {
            source: PathBuf::from("/private/local/source"),
            source_url: "https://example.invalid/source".to_string(),
            revision: "abc123".to_string(),
            destination: PathBuf::from("vendor-cargo-deps/core"),
            name: "core".to_string(),
            paths: vec![PathBuf::from("core")],
            strip_prefix: None,
        };
        let rendered =
            format!("name: {}\nrepository: {}\nrevision: {}\n", request.name, request.source_url, request.revision);

        assert!(!rendered.contains("/private/local/source"));
        assert!(rendered.contains("https://example.invalid/source"));
    }
}
