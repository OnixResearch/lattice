#!/usr/bin/env -S CARGO_TARGET_DIR=/tmp/lattice-sync-toon-format-target nix shell "github:nix-community/fenix?rev=7b33c6466f781cd699fe250c5b69dc4193da67a7#minimal.toolchain" nixpkgs#gcc nixpkgs#clang -c cargo -q -Zscript
---cargo
[package]
name = "sync-toon-format"
version = "0.1.0"
edition = "2024"

[dependencies]
blake3 = "1.8.3"
---

use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

const EXPECTED_ARGUMENT_COUNT: usize = 3;
const SOURCE_ARGUMENT_INDEX: usize = 1;
const DESTINATION_ARGUMENT_INDEX: usize = 2;
const FAILURE_EXIT_CODE: u8 = 1;
const EXPECTED_REVISION: &str = "2136cb1a35f3ed63be733bb74d36b89d3b4592dd";
const EXPECTED_VERSION: &str = "0.5.0";
const EXPECTED_UPSTREAM_MANIFEST_BLAKE3: &str = "94e46fabf052aeaba2be0d42d6eeb45ccb63ada21d13c38e9c25064dd81f19b5";
const EXPECTED_UPSTREAM_FILES_MANIFEST_BLAKE3: &str =
    "7aef44e975bdab7a0a705bf824f5ffa0e292bd2d1cb122c6f1585364ad33dc01";
const STAGING_NAME: &str = ".toon-format-sync-stage";
const BACKUP_NAME: &str = ".toon-format-sync-backup";
const COMPILE_STAGE_NAME: &str = ".toon-format-compile-stage";
const COMPILE_DIRECTORY_SUFFIX: &str = "-compile";
const INLINE_TEST_MARKER: &str = "#[cfg(test)]\nmod tests";
const DOC_EXAMPLE_FENCE: &str = "```";
const DOC_EXAMPLE_IGNORED_FENCE: &str = "```ignore";
const EXPECTED_COMPILE_MANIFEST_BLAKE3: &str = "2e255cece2de6905eb001898f6303699dad50422155480ad39d0f9a487a5356d";
const UPSTREAM_MANIFEST_NAME: &str = "Cargo.toml";
const UPSTREAM_ORIGINAL_MANIFEST_NAME: &str = "Cargo.toml.orig";
const VCS_INFO_NAME: &str = ".cargo_vcs_info.json";
const LICENSE_NAME: &str = "LICENSE";
const README_NAME: &str = "README.md";
const CHANGELOG_NAME: &str = "CHANGELOG.md";
const SOURCE_DIRECTORY_NAME: &str = "src";
const FILE_MANIFEST_NAME: &str = "UPSTREAM-FILES.b3";
const SOURCE_METADATA_NAME: &str = "SOURCE.ncl";
const PATCH_NOTES_NAME: &str = "PATCH.md";

const PATCHED_MANIFEST: &str = r#"# Lattice keeps the reviewed toon-format 0.5.0 library source unchanged.
# This manifest removes unused CLI, TUI, examples, tests, and dev dependencies.
# It also avoids serde_json/preserve_order feature unification in the product graph.

[package]
name = "toon-format"
version = "0.5.0"
edition = "2021"
authors = [
  "Johann Schopplich",
  "Shreyas K S",
]
license = "MIT"
description = "Token-Oriented Object Notation (TOON) - a token-efficient JSON alternative for LLM prompts"
repository = "https://github.com/toon-format/toon-rust"
homepage = "https://toonformat.dev"
documentation = "https://docs.rs/toon-format"
readme = "README.md"

[lib]
name = "toon_format"
path = "src/lib.rs"

[features]
default = []
cli = []
json_stream = []
layout = []

[dependencies]
serde = { version = "1.0.228", features = ["derive"] }
indexmap = "2.0"
serde_json = "1.0.145"
thiserror = "2.0.17"
"#;

const SOURCE_METADATA: &str = include_str!("../src/vendor-cargo-deps/toon-format/SOURCE.ncl");
const PATCH_NOTES: &str = include_str!("../src/vendor-cargo-deps/toon-format/PATCH.md");

fn main() -> ExitCode {
    match run(std::env::args().collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("toon-format sync failed: {error}");
            ExitCode::from(FAILURE_EXIT_CODE)
        }
    }
}

fn run(arguments: Vec<String>) -> Result<(), String> {
    if arguments.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: scripts/sync_toon_format.rs SOURCE DESTINATION".to_string());
    }
    let source = Path::new(&arguments[SOURCE_ARGUMENT_INDEX]);
    let destination = Path::new(&arguments[DESTINATION_ARGUMENT_INDEX]);
    validate_source(source)?;
    let parent = destination.parent().ok_or_else(|| "destination has no parent".to_string())?;
    let staging = parent.join(STAGING_NAME);
    let backup = parent.join(BACKUP_NAME);
    let compile_directory = parent.join(format!(
        "{}{COMPILE_DIRECTORY_SUFFIX}",
        destination.file_name().ok_or_else(|| "destination has no name".to_string())?.to_string_lossy()
    ));
    let compile_staging = parent.join(COMPILE_STAGE_NAME);
    remove_if_present(&staging)?;
    remove_if_present(&backup)?;
    remove_if_present(&compile_staging)?;
    fs::create_dir_all(&staging).map_err(|error| format!("create staging directory: {error}"))?;

    let mut source_files = Vec::new();
    collect_files(source, source, &mut source_files)?;
    source_files.sort();
    let mut manifest = String::new();
    for relative in source_files {
        if !should_copy(&relative) {
            continue;
        }
        let input = source.join(&relative);
        let output = staging.join(&relative);
        if let Some(output_parent) = output.parent() {
            fs::create_dir_all(output_parent).map_err(|error| format!("create output directory: {error}"))?;
        }
        let bytes = fs::read(&input).map_err(|error| format!("read {}: {error}", input.display()))?;
        fs::write(&output, &bytes).map_err(|error| format!("write {}: {error}", output.display()))?;
        let path = relative.to_string_lossy().replace('\\', "/");
        let digest = blake3::hash(&bytes).to_hex();
        writeln!(manifest, "{digest}  ./{path}").map_err(|error| format!("write manifest entry: {error}"))?;
    }
    fs::write(staging.join(FILE_MANIFEST_NAME), &manifest).map_err(|error| format!("write file manifest: {error}"))?;
    let manifest_identity = blake3::hash(manifest.as_bytes()).to_hex().to_string();
    if manifest_identity != EXPECTED_UPSTREAM_FILES_MANIFEST_BLAKE3 {
        return Err(format!("upstream file manifest changed: {manifest_identity}"));
    }
    fs::write(staging.join(UPSTREAM_MANIFEST_NAME), PATCHED_MANIFEST)
        .map_err(|error| format!("write patched manifest: {error}"))?;
    fs::write(staging.join(SOURCE_METADATA_NAME), SOURCE_METADATA)
        .map_err(|error| format!("write source metadata: {error}"))?;
    fs::write(staging.join(PATCH_NOTES_NAME), PATCH_NOTES).map_err(|error| format!("write patch notes: {error}"))?;

    if destination.exists() {
        fs::rename(destination, &backup).map_err(|error| format!("move current destination: {error}"))?;
    }
    if let Err(error) = fs::rename(&staging, destination) {
        if backup.exists() {
            let _ = fs::rename(&backup, destination);
        }
        return Err(format!("publish staged snapshot: {error}"));
    }
    remove_if_present(&backup)?;

    let compile_manifest = write_compile_tree(destination, &compile_staging)?;
    let compile_identity = blake3::hash(compile_manifest.as_bytes()).to_hex().to_string();
    if compile_identity != EXPECTED_COMPILE_MANIFEST_BLAKE3 {
        return Err(format!("derived compile tree changed: {compile_identity}"));
    }
    if compile_directory.exists() {
        let retired = parent.join(format!(
            "{}{COMPILE_DIRECTORY_SUFFIX}.retired",
            destination.file_name().ok_or_else(|| "destination has no name".to_string())?.to_string_lossy()
        ));
        remove_if_present(&retired)?;
        fs::rename(&compile_directory, &retired).map_err(|error| format!("retire compile tree: {error}"))?;
        remove_if_present(&retired)?;
    }
    fs::rename(&compile_staging, &compile_directory).map_err(|error| format!("publish compile tree: {error}"))?;
    Ok(())
}

fn write_compile_tree(pristine: &Path, staging: &Path) -> Result<String, String> {
    let compiled_roots: [&str; 5] = ["constants.rs", "decode", "encode", "types", "utils"];
    let source_root = pristine.join(SOURCE_DIRECTORY_NAME);
    let mut derived = Vec::new();
    for root in compiled_roots {
        let root_path = source_root.join(root);
        if root_path.is_file() {
            derived.push(std::path::PathBuf::from(root));
            continue;
        }
        let mut nested = Vec::new();
        collect_files(&root_path, &root_path, &mut nested)?;
        for relative in nested {
            derived.push(std::path::Path::new(root).join(relative));
        }
    }
    derived.sort();
    let mut manifest = String::new();
    for relative in derived {
        let input = source_root.join(&relative);
        let bytes = fs::read(&input).map_err(|error| format!("read {}: {error}", input.display()))?;
        let stripped = strip_inline_test_module(&bytes)?;
        let derived = ignore_upstream_doc_examples(&stripped)?;
        let output = staging.join(SOURCE_DIRECTORY_NAME).join(&relative);
        if let Some(output_parent) = output.parent() {
            fs::create_dir_all(output_parent).map_err(|error| format!("create compile directory: {error}"))?;
        }
        fs::write(&output, &derived).map_err(|error| format!("write {}: {error}", output.display()))?;
        let path = relative.to_string_lossy().replace('\\', "/");
        let digest = blake3::hash(&derived).to_hex();
        writeln!(manifest, "{digest}  ./{path}").map_err(|error| format!("write compile manifest entry: {error}"))?;
    }
    Ok(manifest)
}

fn strip_inline_test_module(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| format!("decode utf-8: {error}"))?;
    let occurrences = text.matches(INLINE_TEST_MARKER).count();
    if occurrences == 0 {
        return Ok(bytes.to_vec());
    }
    if occurrences > 1 {
        return Err("multiple inline test modules in one file".to_string());
    }
    let marker_index = text.find(INLINE_TEST_MARKER).ok_or_else(|| "marker vanished".to_string())?;
    let mut stripped = text[..marker_index].to_string();
    while stripped.ends_with(char::is_whitespace) {
        stripped.pop();
    }
    if !stripped.ends_with('}') {
        return Err("stripped file does not end at a balanced item boundary".to_string());
    }
    stripped.push('\n');
    Ok(stripped.into_bytes())
}

fn ignore_upstream_doc_examples(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| format!("decode utf-8: {error}"))?;
    let mut inside_example = false;
    let derived = text
        .lines()
        .map(|line| -> String {
            let fence_body = line.trim().trim_start_matches(|c| c == '/' || c == '!').trim();
            if fence_body == DOC_EXAMPLE_FENCE {
                inside_example = !inside_example;
                if inside_example {
                    return line.replacen(DOC_EXAMPLE_FENCE, DOC_EXAMPLE_IGNORED_FENCE, 1).to_string();
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut derived = derived;
    if text.ends_with('\n') {
        derived.push('\n');
    }
    Ok(derived.into_bytes())
}

fn validate_source(source: &Path) -> Result<(), String> {
    let cargo = fs::read_to_string(source.join(UPSTREAM_MANIFEST_NAME))
        .map_err(|error| format!("read upstream Cargo.toml: {error}"))?;
    let revision = fs::read_to_string(source.join(".cargo_vcs_info.json"))
        .map_err(|error| format!("read upstream revision: {error}"))?;
    let license =
        fs::read_to_string(source.join("LICENSE")).map_err(|error| format!("read upstream license: {error}"))?;
    validate_source_text(&cargo, &revision, &license)?;
    let observed = blake3::hash(cargo.as_bytes()).to_hex().to_string();
    if observed != EXPECTED_UPSTREAM_MANIFEST_BLAKE3 {
        return Err(format!("upstream generated manifest changed: {observed}"));
    }
    Ok(())
}

fn validate_source_text(cargo: &str, revision: &str, license: &str) -> Result<(), String> {
    if !cargo.contains(&format!("version = \"{EXPECTED_VERSION}\"")) {
        return Err("upstream version is not 0.5.0".to_string());
    }
    if !cargo.contains("features = [\"preserve_order\"]") {
        return Err("upstream serde_json feature shape changed".to_string());
    }
    if !revision.contains(EXPECTED_REVISION) {
        return Err("upstream revision changed".to_string());
    }
    if !license.contains("MIT License") {
        return Err("upstream license changed".to_string());
    }
    Ok(())
}

fn collect_files(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| format!("read {}: {error}", directory.display()))? {
        let entry = entry.map_err(|error| format!("read directory entry: {error}"))?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(|error| format!("read {} metadata: {error}", path.display()))?;
        if metadata.is_dir() {
            collect_files(root, &path, files)?;
        } else if metadata.is_file() {
            files.push(path.strip_prefix(root).map_err(|error| format!("strip source prefix: {error}"))?.to_path_buf());
        } else {
            return Err(format!("unsupported source member: {}", path.display()));
        }
    }
    Ok(())
}

fn should_copy(relative: &Path) -> bool {
    relative == Path::new(UPSTREAM_ORIGINAL_MANIFEST_NAME)
        || relative == Path::new(VCS_INFO_NAME)
        || relative == Path::new(LICENSE_NAME)
        || relative == Path::new(README_NAME)
        || relative == Path::new(CHANGELOG_NAME)
        || relative.starts_with(SOURCE_DIRECTORY_NAME)
}

fn remove_if_present(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| format!("remove {}: {error}", path.display()))?;
    } else if path.exists() {
        fs::remove_file(path).map_err(|error| format!("remove {}: {error}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ignore_upstream_doc_examples;
    use super::strip_inline_test_module;
    use super::validate_source_text;

    #[test]
    fn source_text_accepts_the_reviewed_release() {
        let cargo = "version = \"0.5.0\"\nfeatures = [\"preserve_order\"]\n";
        let revision = "2136cb1a35f3ed63be733bb74d36b89d3b4592dd";
        let license = "MIT License";
        assert_eq!(validate_source_text(cargo, revision, license), Ok(()));
    }

    #[test]
    fn inline_test_module_stripping_keeps_code_and_fails_off_boundaries() {
        let code = b"pub fn f() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {\n        assert_eq!(f(), 1);\n    }\n}\n";
        let stripped = strip_inline_test_module(code).expect("tail test module strips");
        assert_eq!(String::from_utf8(stripped.to_vec()).unwrap(), "pub fn f() -> u32 {\n    1\n}\n");

        let untouched = b"pub fn g() -> u32 {\n    2\n}\n";
        assert_eq!(strip_inline_test_module(untouched).unwrap(), untouched.to_vec());

        let double_marker = b"#[cfg(test)]\nmod tests {\n}\n#[cfg(test)]\nmod tests {\n}\n";
        assert!(strip_inline_test_module(double_marker).is_err());

        let unbalanced = b"pub fn h() {\n#[cfg(test)]\nmod tests {\n}";
        assert!(strip_inline_test_module(unbalanced).is_err());
    }

    #[test]
    fn upstream_doc_examples_become_ignore_fences() {
        let documented =
            b"/// Example:\n/// ```\n/// use toon_format::encode_default;\n/// ```\npub fn documented() {}\n";
        let derived = ignore_upstream_doc_examples(documented).expect("doc fences become ignore");
        assert_eq!(
            String::from_utf8(derived.to_vec()).unwrap(),
            "/// Example:\n/// ```ignore\n/// use toon_format::encode_default;\n/// ```\npub fn documented() {}\n"
        );

        let inner_documented = b"//! ```\n//! let value = 1;\n//! ```\n";
        let derived_inner = ignore_upstream_doc_examples(inner_documented).expect("inner doc fences become ignore");
        assert_eq!(String::from_utf8(derived_inner.to_vec()).unwrap(), "//! ```ignore\n//! let value = 1;\n//! ```\n");

        let untouched = b"pub fn plain() {}\n";
        assert_eq!(ignore_upstream_doc_examples(untouched).unwrap(), untouched.to_vec());
    }

    #[test]
    fn source_text_rejects_revision_and_feature_drift() {
        let cargo = "version = \"0.5.0\"\nserde_json = \"1\"\n";
        assert!(validate_source_text(cargo, "different", "MIT License").is_err());
    }
}
