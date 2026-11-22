//! Pure bounds for one catalog fetch: object format, transfer budget and selected blob sizes.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "untrusted repository observations return typed errors instead of terminating the process"
)]

const SHA256_HEX_LENGTH: usize = 64;
const FIELD_SEPARATOR: char = ' ';
const LINE_SEPARATOR: char = '\n';
const BLOB_TYPE: &str = "blob";
const MANIFEST_FILE: &str = "manifest.ncl";
/// `ulimit -f` counts 512-byte blocks in a POSIX shell.
pub const FILE_SIZE_BLOCK_BYTES: u64 = 512;
/// The smallest transfer budget an operator may select.
pub const MINIMUM_TRANSFER_LIMIT_BYTES: u64 = 65_536;
/// The default budget: every pack content byte plus the manifest, and as much again for trees,
/// pack framing and the commit object.
pub const DEFAULT_TRANSFER_LIMIT_BYTES: u64 = super::super::model::MAXIMUM_PACK_BYTES
    .saturating_add(super::super::model::MAXIMUM_MANIFEST_BYTES)
    .saturating_mul(2);
/// Selected pack content, including the manifest, never exceeds this total.
pub const MAXIMUM_SELECTED_BYTES: u64 =
    super::super::model::MAXIMUM_PACK_BYTES.saturating_add(super::super::model::MAXIMUM_MANIFEST_BYTES);

/// The object format of the staging repository, chosen by the full commit name's length.
pub fn object_format(commit: &str) -> &'static str {
    if commit.len() == SHA256_HEX_LENGTH {
        "sha256"
    } else {
        "sha1"
    }
}

pub fn check_budget(limit_bytes: u64) -> Result<(), super::super::Error> {
    if limit_bytes < MINIMUM_TRANSFER_LIMIT_BYTES {
        return Err(super::super::Error::Invalid("catalog transfer limit"));
    }
    Ok(())
}

/// Whole 512-byte blocks for `ulimit -f`, never above the remaining budget.
pub fn file_size_blocks(remaining_bytes: u64) -> u64 {
    (remaining_bytes / FILE_SIZE_BLOCK_BYTES).max(1)
}

/// Validate `git cat-file --batch-check='%(objectname) %(objecttype) %(objectsize)'` output for
/// the selected entries, in order, before any blob content is read.
///
/// Every object must be a present blob within its file bound, and the selected total must stay
/// within the pack content bound. Returns each entry's exact size.
pub fn validate_object_sizes(
    output: &[u8],
    entries: &[super::source::TreeEntry],
) -> Result<Vec<u64>, super::super::Error> {
    let text = std::str::from_utf8(output).map_err(|_| super::super::Error::Invalid("catalog object sizes"))?;
    let lines = text.split(LINE_SEPARATOR).filter(|line| !line.is_empty()).collect::<Vec<_>>();
    if lines.len() != entries.len() {
        return Err(super::super::Error::Invalid("catalog object sizes"));
    }
    let mut sizes = Vec::with_capacity(entries.len());
    let mut total_bytes = 0u64;
    for (line, entry) in lines.iter().zip(entries) {
        let mut fields = line.split(FIELD_SEPARATOR);
        let (Some(object), Some(kind), Some(size), None) = (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(super::super::Error::Invalid("catalog object missing"));
        };
        let size_bytes = size.parse::<u64>().map_err(|_| super::super::Error::Invalid("catalog object sizes"))?;
        if object != entry.object || kind != BLOB_TYPE {
            return Err(super::super::Error::Invalid("catalog object kind"));
        }
        let file_limit_bytes = if entry.path == MANIFEST_FILE {
            super::super::model::MAXIMUM_MANIFEST_BYTES
        } else {
            super::super::model::MAXIMUM_PACK_BYTES
        };
        total_bytes = total_bytes.saturating_add(size_bytes);
        if size_bytes > file_limit_bytes || total_bytes > MAXIMUM_SELECTED_BYTES {
            return Err(super::super::Error::Invalid("catalog object size bound"));
        }
        sizes.push(size_bytes);
    }
    debug_assert_eq!(sizes.len(), entries.len());
    Ok(sizes)
}
