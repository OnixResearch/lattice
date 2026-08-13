use crate::filesystem::{collect_snapshot_entries, compare_text_file, remove_directory_if_present};
use crate::model::{
    DESTINATION_DIRECTORY, MANIFEST_FILE, Mode, REVISION_FILE, SNAPSHOT_SCHEMA, STAGING_DIRECTORY,
    render_manifest_material, validate_required_entries, validate_source_revision,
};
use crate::process::{blake3_bytes, extract_source_revision};
use std::fs;
use std::path::Path;

pub(crate) fn run(root: &Path, mode: Mode<'_>) -> Result<(), String> {
    match mode {
        Mode::Check => check_snapshot(root),
        Mode::CheckSource(source, revision) => check_source(root, source, revision),
        Mode::Import(source, revision) => import_snapshot(root, source, revision),
    }
}

fn check_snapshot(root: &Path) -> Result<(), String> {
    let revision = read_recorded_revision(root)?;
    let destination = root.join(DESTINATION_DIRECTORY);
    let entries = collect_snapshot_entries(&destination)?;
    validate_required_entries(&entries)?;
    let manifest = render_manifest(&entries, &revision)?;
    compare_text_file(&root.join(MANIFEST_FILE), &manifest)?;
    println!(
        "vendored UCAN snapshot passed: revision {revision}, {} files",
        entries.len()
    );
    Ok(())
}

fn check_source(root: &Path, source: &Path, revision: &str) -> Result<(), String> {
    let staging = root.join(STAGING_DIRECTORY);
    extract_source_revision(source, &staging, revision)?;
    let result = (|| {
        let entries = collect_snapshot_entries(&staging)?;
        validate_required_entries(&entries)?;
        let manifest = render_manifest(&entries, revision)?;
        compare_text_file(&root.join(MANIFEST_FILE), &manifest)
    })();
    remove_directory_if_present(&staging)?;
    result?;
    println!(
        "vendored UCAN source parity passed: {} at {revision}",
        source.display()
    );
    Ok(())
}

fn import_snapshot(root: &Path, source: &Path, revision: &str) -> Result<(), String> {
    let staging = root.join(STAGING_DIRECTORY);
    extract_source_revision(source, &staging, revision)?;
    let entries = collect_snapshot_entries(&staging)?;
    validate_required_entries(&entries)?;
    let manifest = render_manifest(&entries, revision)?;
    let destination = root.join(DESTINATION_DIRECTORY);
    remove_directory_if_present(&destination)?;
    let destination_parent = destination
        .parent()
        .ok_or_else(|| format!("destination has no parent: {}", destination.display()))?;
    fs::create_dir_all(destination_parent)
        .map_err(|error| format!("cannot create {}: {error}", destination_parent.display()))?;
    fs::rename(&staging, &destination).map_err(|error| {
        format!(
            "cannot move {} to {}: {error}",
            staging.display(),
            destination.display()
        )
    })?;
    fs::write(root.join(REVISION_FILE), format!("{revision}\n"))
        .map_err(|error| format!("cannot write {REVISION_FILE}: {error}"))?;
    fs::write(root.join(MANIFEST_FILE), manifest)
        .map_err(|error| format!("cannot write {MANIFEST_FILE}: {error}"))?;
    check_snapshot(root)
}

fn render_manifest(
    entries: &[crate::model::SnapshotEntry],
    revision: &str,
) -> Result<String, String> {
    let material = render_manifest_material(entries);
    let root_digest = blake3_bytes(material.as_bytes())?;
    Ok(format!(
        "schema\t{SNAPSHOT_SCHEMA}\nsource-revision\t{revision}\nentry-count\t{}\ncontent-blake3\t{root_digest}\n{material}",
        entries.len()
    ))
}

fn read_recorded_revision(root: &Path) -> Result<String, String> {
    let path = root.join(REVISION_FILE);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let revision = text.trim();
    validate_source_revision(revision)?;
    Ok(revision.to_owned())
}
