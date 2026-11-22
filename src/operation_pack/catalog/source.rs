//! Pure validation of an operator-selected catalog repository source and its exact tree listing.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "untrusted repository listings return typed errors instead of terminating the process"
)]

pub const FETCH_RECEIPT_SCHEMA: &str = "lattice.operation-pack-catalog-fetch.v1";
const ALLOWED_SCHEMES: [&str; 3] = ["file://", "https://", "ssh://"];
const MAXIMUM_LOCATOR_BYTES: usize = 2_048;
const SHA1_HEX_LENGTH: usize = 40;
const SHA256_HEX_LENGTH: usize = 64;
const REGULAR_MODE: &str = "100644";
const EXECUTABLE_MODE: &str = "100755";
const SYMLINK_MODE: &str = "120000";
const SUBMODULE_MODE: &str = "160000";
const BLOB_TYPE: &str = "blob";
const RECORD_SEPARATOR: u8 = 0;
const PATH_SEPARATOR: char = '/';
/// The pack manifest plus every declared content item.
pub const MAXIMUM_TREE_FILES: usize = super::super::model::MAXIMUM_CONTENT_ITEMS.saturating_add(1);

/// One regular file of the selected pack tree at an exact commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub path: String,
    pub object: String,
    pub is_executable: bool,
}

/// Accept only explicit `file://`, `https://` and `ssh://` locators; never options or helpers.
pub fn validate_repository_locator(value: &str) -> Result<(), super::super::Error> {
    let has_allowed_scheme =
        ALLOWED_SCHEMES.iter().any(|scheme| value.strip_prefix(scheme).is_some_and(|rest| !rest.is_empty()));
    let has_safe_bytes = value.bytes().all(|byte| byte.is_ascii_graphic());
    if !has_allowed_scheme || !has_safe_bytes || value.len() > MAXIMUM_LOCATOR_BYTES {
        return Err(super::super::Error::Invalid("catalog repository locator"));
    }
    Ok(())
}

/// Accept only a full lowercase SHA-1 or SHA-256 object name, never a branch, tag or prefix.
pub fn validate_commit(value: &str) -> Result<(), super::super::Error> {
    let has_object_length = value.len() == SHA1_HEX_LENGTH || value.len() == SHA256_HEX_LENGTH;
    let has_lowercase_hex = value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !has_object_length || !has_lowercase_hex {
        return Err(super::super::Error::Invalid("catalog commit"));
    }
    Ok(())
}

pub fn validate_subdirectory(value: Option<&str>) -> Result<(), super::super::Error> {
    value.map_or(Ok(()), super::super::validation::validate_safe_relative_path)
}

/// Parse `git ls-tree -r -z --full-tree` output and keep only safe regular files.
///
/// Symlinks, submodules, unsafe paths, entries outside the selected subdirectory and oversized
/// listings refuse the whole fetch.
pub fn parse_tree(listing: &[u8], subdirectory: Option<&str>) -> Result<Vec<TreeEntry>, super::super::Error> {
    let mut entries = Vec::with_capacity(MAXIMUM_TREE_FILES);
    for record in listing.split(|byte| *byte == RECORD_SEPARATOR).filter(|record| !record.is_empty()) {
        if entries.len() == MAXIMUM_TREE_FILES {
            return Err(super::super::Error::Invalid("catalog tree bound"));
        }
        entries.push(parse_record(record, subdirectory)?);
    }
    if entries.is_empty() {
        return Err(super::super::Error::Invalid("catalog tree is empty"));
    }
    debug_assert!(entries.len() <= MAXIMUM_TREE_FILES);
    Ok(entries)
}

fn parse_record(record: &[u8], subdirectory: Option<&str>) -> Result<TreeEntry, super::super::Error> {
    let text = std::str::from_utf8(record).map_err(|_| super::super::Error::Invalid("catalog path encoding"))?;
    let (header, full_path) = text.split_once('\t').ok_or(super::super::Error::Invalid("catalog tree record"))?;
    let mut fields = header.split(' ');
    let (Some(mode), Some(kind), Some(object), None) = (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return Err(super::super::Error::Invalid("catalog tree record"));
    };
    let is_executable = match mode {
        REGULAR_MODE => false,
        EXECUTABLE_MODE => true,
        SYMLINK_MODE => return Err(super::super::Error::Invalid("catalog symlink member")),
        SUBMODULE_MODE => return Err(super::super::Error::Invalid("catalog submodule member")),
        _ => return Err(super::super::Error::Invalid("catalog member kind")),
    };
    if kind != BLOB_TYPE {
        return Err(super::super::Error::Invalid("catalog member kind"));
    }
    validate_commit(object).map_err(|_| super::super::Error::Invalid("catalog object name"))?;
    let path = match subdirectory {
        Some(prefix) => full_path
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_prefix(PATH_SEPARATOR))
            .ok_or(super::super::Error::Invalid("catalog member outside subdirectory"))?,
        None => full_path,
    };
    super::super::validation::validate_safe_relative_path(path)?;
    Ok(TreeEntry {
        path: path.to_string(),
        object: object.to_string(),
        is_executable,
    })
}
