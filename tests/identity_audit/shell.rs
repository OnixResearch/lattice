use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use super::core::BinarySnapshot;
use super::core::TextSnapshot;

const PUBLIC_ROOTS: &[&str] = &[
    "Cargo.toml",
    "README.md",
    "flake.nix",
    "src",
    "tests",
    "examples",
    "scripts",
];
const TEXT_EXTENSIONS: &[&str] = &["md", "ncl", "nix", "rs", "toml"];
const ACCEPTED_SPECS_ROOT: &str = ".cairn/specs";
const RENAME_ARCHIVE_ROOT: &str = concat!(".cairn/archive/2026-07-20-rename-n8n", "clone-to-lattice");

pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn read_public_text(root: &Path) -> io::Result<Vec<TextSnapshot>> {
    let mut snapshots = BTreeMap::new();
    for relative in PUBLIC_ROOTS {
        collect_text(root, &root.join(relative), &mut snapshots)?;
    }
    Ok(snapshots.into_values().collect())
}

pub fn read_accepted_specs(root: &Path) -> io::Result<Vec<TextSnapshot>> {
    let mut snapshots = BTreeMap::new();
    collect_text(root, &root.join(ACCEPTED_SPECS_ROOT), &mut snapshots)?;
    Ok(snapshots.into_values().collect())
}

pub fn read_rename_archive(root: &Path) -> io::Result<Vec<BinarySnapshot>> {
    let mut snapshots = BTreeMap::new();
    collect_binary(root, &root.join(RENAME_ARCHIVE_ROOT), &mut snapshots)?;
    Ok(snapshots.into_values().collect())
}

fn collect_text(root: &Path, path: &Path, snapshots: &mut BTreeMap<String, TextSnapshot>) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in sorted_entries(path)? {
            collect_text(root, &entry, snapshots)?;
        }
        return Ok(());
    }
    if !is_text_path(path) {
        return Ok(());
    }

    let relative = relative_path(root, path)?;
    let text = fs::read_to_string(path)?;
    let previous = snapshots.insert(relative.clone(), TextSnapshot::new(relative, text));
    assert!(previous.is_none(), "duplicate public snapshot path");
    Ok(())
}

fn collect_binary(root: &Path, path: &Path, snapshots: &mut BTreeMap<String, BinarySnapshot>) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_dir() {
        for entry in sorted_entries(path)? {
            collect_binary(root, &entry, snapshots)?;
        }
        return Ok(());
    }

    let relative = relative_path(root, path)?;
    let bytes = fs::read(path)?;
    let previous = snapshots.insert(relative.clone(), BinarySnapshot::new(relative, bytes));
    assert!(previous.is_none(), "duplicate archive snapshot path");
    Ok(())
}

fn sorted_entries(path: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries =
        fs::read_dir(path)?.map(|entry| entry.map(|value| value.path())).collect::<io::Result<Vec<_>>>()?;
    entries.sort();
    Ok(entries)
}

fn is_text_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| TEXT_EXTENSIONS.contains(&extension))
}

fn relative_path(root: &Path, path: &Path) -> io::Result<String> {
    let relative = path.strip_prefix(root).map_err(io::Error::other)?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}
