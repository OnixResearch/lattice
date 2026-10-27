use std::path::Path;

pub(crate) const SOURCE_REVISION: &str = "3a2aa043068a82663cf72ec05ed5d56a7d65ecd5";
pub(crate) const SNAPSHOT_SCHEMA: &str = "basalt.vendored-ucan.snapshot.v1";
pub(crate) const DESTINATION_DIRECTORY: &str = "vendor/ucan";
pub(crate) const REVISION_FILE: &str = "vendor/ucan-revision.txt";
pub(crate) const MANIFEST_FILE: &str = "vendor/ucan.snapshot";
pub(crate) const STAGING_DIRECTORY: &str = "target/vendored-ucan-import";
const MODE_CHECK: &str = "--check";
const MODE_CHECK_SOURCE: &str = "--check-source";
const MODE_CHECK_SOURCE_REVISION: &str = "--check-source-revision";
const MODE_IMPORT: &str = "--import";
const MODE_IMPORT_REVISION: &str = "--import-revision";
const CHECK_ARGUMENT_COUNT: usize = 2;
const SOURCE_ARGUMENT_COUNT: usize = 3;
const SOURCE_REVISION_ARGUMENT_COUNT: usize = 4;
const SOURCE_PATH_ARGUMENT_INDEX: usize = 2;
const SOURCE_REVISION_ARGUMENT_INDEX: usize = 3;
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;
const GIT_SHA1_HEX_LENGTH: usize = 40;
const GIT_SHA256_HEX_LENGTH: usize = 64;
pub(crate) const MAXIMUM_SNAPSHOT_ENTRIES: usize = 2_048;
const BYTES_PER_MEBIBYTE: u64 = 1_048_576;
const MAXIMUM_SNAPSHOT_MEBIBYTES: u64 = 64;
pub(crate) const MAXIMUM_SNAPSHOT_BYTES: u64 = MAXIMUM_SNAPSHOT_MEBIBYTES * BYTES_PER_MEBIBYTE;
pub(crate) const EXECUTABLE_MODE_MASK: u32 = 0o111;
pub(crate) const REGULAR_FILE_MODE: &str = "100644";
pub(crate) const EXECUTABLE_FILE_MODE: &str = "100755";
pub(crate) const GIT_COMMIT_SUFFIX: &str = "^{commit}";

pub(crate) const SOURCE_SELECTION: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "README.md",
    "LICENSES",
    "license-policy.tsv",
    "docs/licensing.md",
    "docs/production-readiness.json",
    "docs/production-readiness.md",
    "docs/holder-authorization-recheck.md",
    "docs/verified-boundaries.md",
    "src",
    "crates/ucan-core",
    "crates/ucan-nostr",
    "vendor/verified-logic",
    "vendor/verified-logic-revision.txt",
    "tools/check-production-readiness",
    "tools/check-aeneas-time-bounds-proof",
    "proofs/aeneas-time-bounds",
];

pub(crate) const REQUIRED_SNAPSHOT_PATHS: &[&str] = &[
    "Cargo.toml",
    "src/lib.rs",
    "src/token.rs",
    "src/token/holder_invocation.rs",
    "src/token/holder_invocation/recheck.rs",
    "crates/ucan-core/src/holder_invocation/recheck.rs",
    "docs/holder-authorization-recheck.md",
    "crates/ucan-core/Cargo.toml",
    "crates/ucan-core/LICENSE",
    "crates/ucan-core/src/lib.rs",
    "crates/ucan-nostr/Cargo.toml",
    "crates/ucan-nostr/LICENSE",
    "crates/ucan-nostr/src/lib.rs",
    "vendor/verified-logic/Cargo.toml",
    "vendor/verified-logic/LICENSE",
    "vendor/verified-logic/src/lib.rs",
    "LICENSES/AGPL-3.0-or-later.txt",
    "LICENSES/MPL-2.0.txt",
    "docs/licensing.md",
    "docs/production-readiness.json",
    "docs/verified-boundaries.md",
    "license-policy.tsv",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct SnapshotEntry {
    pub(crate) mode: &'static str,
    pub(crate) digest: String,
    pub(crate) path: String,
    pub(crate) byte_len: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode<'a> {
    Check,
    CheckSource(&'a Path, &'a str),
    Import(&'a Path, &'a str),
}

// r[impl ucan_token_flow.vendored_source.exact_revision]
pub(crate) fn parse_mode<'a>(arguments: &'a [String]) -> Result<Mode<'a>, String> {
    if arguments.len() == CHECK_ARGUMENT_COUNT && arguments[1] == MODE_CHECK {
        return Ok(Mode::Check);
    }
    if arguments.len() == SOURCE_ARGUMENT_COUNT && arguments[1] == MODE_CHECK_SOURCE {
        return Ok(Mode::CheckSource(
            Path::new(&arguments[SOURCE_PATH_ARGUMENT_INDEX]),
            SOURCE_REVISION,
        ));
    }
    if arguments.len() == SOURCE_ARGUMENT_COUNT && arguments[1] == MODE_IMPORT {
        return Ok(Mode::Import(
            Path::new(&arguments[SOURCE_PATH_ARGUMENT_INDEX]),
            SOURCE_REVISION,
        ));
    }
    if arguments.len() == SOURCE_REVISION_ARGUMENT_COUNT
        && arguments[1] == MODE_CHECK_SOURCE_REVISION
    {
        validate_source_revision(&arguments[SOURCE_REVISION_ARGUMENT_INDEX])?;
        return Ok(Mode::CheckSource(
            Path::new(&arguments[SOURCE_PATH_ARGUMENT_INDEX]),
            &arguments[SOURCE_REVISION_ARGUMENT_INDEX],
        ));
    }
    if arguments.len() == SOURCE_REVISION_ARGUMENT_COUNT && arguments[1] == MODE_IMPORT_REVISION {
        validate_source_revision(&arguments[SOURCE_REVISION_ARGUMENT_INDEX])?;
        return Ok(Mode::Import(
            Path::new(&arguments[SOURCE_PATH_ARGUMENT_INDEX]),
            &arguments[SOURCE_REVISION_ARGUMENT_INDEX],
        ));
    }
    Err(format!(
        "usage: sync-vendored-ucan.rs {MODE_CHECK} | {MODE_CHECK_SOURCE} SOURCE_REPOSITORY | {MODE_CHECK_SOURCE_REVISION} SOURCE_REPOSITORY REVISION | {MODE_IMPORT} SOURCE_REPOSITORY | {MODE_IMPORT_REVISION} SOURCE_REPOSITORY REVISION"
    ))
}

// r[impl ucan_token_flow.vendored_source.exact_revision]
pub(crate) fn validate_source_revision(revision: &str) -> Result<(), String> {
    let length = revision.len();
    let valid_length = length == GIT_SHA1_HEX_LENGTH || length == GIT_SHA256_HEX_LENGTH;
    let canonical_hex = revision
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid_length || !canonical_hex {
        return Err(String::from(
            "source revision must be a full lowercase Git object ID",
        ));
    }
    Ok(())
}

pub(crate) fn validate_required_entries(entries: &[SnapshotEntry]) -> Result<(), String> {
    if entries.is_empty() {
        return Err(String::from("vendored UCAN snapshot is empty"));
    }
    for required in REQUIRED_SNAPSHOT_PATHS {
        if !entries.iter().any(|entry| entry.path == *required) {
            return Err(format!(
                "vendored UCAN snapshot is missing required path {required}"
            ));
        }
    }
    Ok(())
}

pub(crate) fn render_manifest_material(entries: &[SnapshotEntry]) -> String {
    let mut material = String::new();
    for entry in entries {
        material.push_str(entry.mode);
        material.push('\t');
        material.push_str(&entry.digest);
        material.push('\t');
        material.push_str(&entry.byte_len.to_string());
        material.push('\t');
        material.push_str(&entry.path);
        material.push('\n');
    }
    material
}
