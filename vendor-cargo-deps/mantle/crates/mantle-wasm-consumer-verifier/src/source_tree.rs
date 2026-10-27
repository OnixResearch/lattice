//! Remeasure the producer's source-tree identity without copying the tree.
//! The record encoding and breadth-first sorted traversal match
//! crunch-wasm-component::files::copy_source_tree, including empty directories.
use std::borrow::ToOwned;
use std::collections::VecDeque;
use std::format;
use std::io::Read;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::fs::Dir;
use cap_std::fs::OpenOptions;
use cap_std::fs::OpenOptionsExt;

use super::ConsumerMemberKind;
use super::ConsumerSourceFile;
use super::Measurement;

const MAX_TREE_ENTRIES: usize = 10_000;
const MAX_TREE_DEPTH: u32 = 64;
const MAX_TREE_BYTES: u64 = 536_870_912;

pub(super) fn measure(root: Dir, expected_bytes: u64) -> Measurement {
    if expected_bytes > MAX_TREE_BYTES {
        return Measurement::LengthMismatch;
    }
    match measure_inner(root, expected_bytes) {
        Ok((digest, files)) => Measurement::Identity {
            digest,
            kind: ConsumerMemberKind::SourceTree,
            source_files: files,
        },
        Err(measurement) => measurement,
    }
}

fn record(hasher: &mut blake3::Hasher, tag: &[u8], relative: &str, size: u64) {
    hasher.update(tag);
    hasher.update(&(relative.len() as u64).to_le_bytes());
    hasher.update(relative.as_bytes());
    hasher.update(&size.to_le_bytes());
}

fn measure_inner(root: Dir, expected_bytes: u64) -> Result<(String, Vec<ConsumerSourceFile>), Measurement> {
    let mut queue = VecDeque::from([(root, String::new(), 0_u32)]);
    let mut entry_count = 0_usize;
    let mut byte_count = 0_u64;
    let mut hasher = blake3::Hasher::new();
    let mut files = Vec::new();
    while let Some((directory, relative, depth)) = queue.pop_front() {
        if depth > MAX_TREE_DEPTH {
            return Err(Measurement::LengthMismatch);
        }
        let mut names = Vec::new();
        for entry in directory.entries().map_err(|_| Measurement::Missing)? {
            let entry = entry.map_err(|_| Measurement::Missing)?;
            entry_count = entry_count.checked_add(1).ok_or(Measurement::LengthMismatch)?;
            if entry_count > MAX_TREE_ENTRIES {
                return Err(Measurement::LengthMismatch);
            }
            names.push(entry.file_name());
        }
        names.sort();
        for name in names {
            let name_text = name.to_str().ok_or(Measurement::Missing)?;
            if name_text.is_empty() || name_text == "." || name_text == ".." {
                return Err(Measurement::Missing);
            }
            let child = if relative.is_empty() {
                name_text.to_owned()
            } else {
                format!("{relative}/{name_text}")
            };
            let metadata = directory.symlink_metadata(&name).map_err(|_| Measurement::Missing)?;
            if metadata.is_dir() {
                let child_directory = directory.open_dir_nofollow(&name).map_err(|_| Measurement::Missing)?;
                record(&mut hasher, b"dir\0", &child, 0);
                queue.push_back((child_directory, child, depth + 1));
            } else if metadata.is_file() {
                let size = metadata.len();
                byte_count = byte_count.checked_add(size).ok_or(Measurement::LengthMismatch)?;
                if byte_count > expected_bytes {
                    return Err(Measurement::LengthMismatch);
                }
                let mut options = OpenOptions::new();
                options.read(true).follow(FollowSymlinks::No).custom_flags(0o4000);
                let file = directory.open_with(&name, &options).map_err(|_| Measurement::Missing)?;
                let opened = file.metadata().map_err(|_| Measurement::Missing)?;
                if !opened.is_file() || opened.len() != size {
                    return Err(Measurement::LengthMismatch);
                }
                record(&mut hasher, b"file\0", &child, size);
                let mut file_hasher = blake3::Hasher::new();
                let mut reader = file.take(size.saturating_add(1));
                let mut buffer = [0_u8; 65_536];
                let mut read = 0_u64;
                loop {
                    let count = reader.read(&mut buffer).map_err(|_| Measurement::Missing)?;
                    if count == 0 {
                        break;
                    }
                    read += count as u64;
                    if read > size {
                        return Err(Measurement::LengthMismatch);
                    }
                    hasher.update(&buffer[..count]);
                    file_hasher.update(&buffer[..count]);
                }
                if read != size {
                    return Err(Measurement::LengthMismatch);
                }
                files.push(ConsumerSourceFile {
                    relative_path: child,
                    digest_blake3: file_hasher.finalize().to_hex().to_string(),
                    size_bytes: size,
                });
            } else {
                return Err(Measurement::Missing);
            }
        }
    }
    if byte_count != expected_bytes {
        return Err(Measurement::LengthMismatch);
    }
    Ok((hasher.finalize().to_hex().to_string(), files))
}
