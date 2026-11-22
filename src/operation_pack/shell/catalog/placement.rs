//! Create-only, per-bundle atomic placement of admitted pack bundles into a local catalog.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "installation failures become per-bundle outcomes instead of terminating the process"
)]

use std::io::Write as _;

const STAGING_PREFIX: &str = ".lattice-catalog-";
const STAGING_SUFFIX: &str = ".staging";
/// Any entry at `<catalog>/<id>`, including a symlink or an unreadable one, blocks that bundle.
pub(super) fn existing_ids(
    catalog: &std::path::Path,
    manifest: &super::super::super::model::Manifest,
) -> std::collections::BTreeSet<String> {
    manifest
        .workflow_bundles
        .iter()
        .filter(|declaration| is_occupied(&catalog.join(&declaration.id)))
        .map(|declaration| declaration.id.clone())
        .collect()
}

pub(super) fn is_occupied(path: &std::path::Path) -> bool {
    !matches!(std::fs::symlink_metadata(path), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
}

/// Stage, re-verify and rename one admitted bundle; the staging directory is always removed.
pub(super) fn publish_bundle(
    request: &super::InstallRequest<'_>,
    parent: &std::path::Path,
    bundle_id: &str,
) -> Result<(), super::super::super::Error> {
    super::super::super::catalog::validate_bundle_id(bundle_id)?;
    let staging = parent.join(format!("{STAGING_PREFIX}{bundle_id}{STAGING_SUFFIX}"));
    std::fs::create_dir(&staging).map_err(|_| super::super::super::Error::Conflict("catalog staging exists"))?;
    let result = stage_and_publish(request, &staging, bundle_id);
    let cleanup = super::super::installation::remove_staging(&staging);
    result?;
    cleanup?;
    super::super::installation::sync_directory(parent)
}

fn stage_and_publish(
    request: &super::InstallRequest<'_>,
    staging: &std::path::Path,
    bundle_id: &str,
) -> Result<(), super::super::super::Error> {
    let root = cap_std::fs::Dir::open_ambient_dir(staging, cap_std::ambient_authority())
        .map_err(|_| super::super::super::Error::Io("catalog staging authority"))?;
    root.create_dir(bundle_id).map_err(|_| super::super::super::Error::Io("catalog staging create"))?;
    let bundle = root.open_dir(bundle_id).map_err(|_| super::super::super::Error::Io("catalog staging open"))?;
    write_bundle(request, &bundle, bundle_id)?;
    verify_staged(request, staging, bundle_id)?;
    let target = request.catalog.join(bundle_id);
    if is_occupied(&target) {
        return Err(super::super::super::Error::Conflict("catalog already holds this bundle id"));
    }
    std::fs::rename(staging.join(bundle_id), &target)
        .map_err(|_| super::super::super::Error::Io("catalog bundle publish"))?;
    super::super::installation::sync_directory(request.catalog)
}

/// Write each bundle file create-only beneath the staged bundle directory, then sync every
/// directory.
fn write_bundle(
    request: &super::InstallRequest<'_>,
    bundle: &cap_std::fs::Dir,
    bundle_id: &str,
) -> Result<(), super::super::super::Error> {
    let candidate = &request.admitted.candidate;
    let contents = super::super::super::catalog::bundle_contents(&candidate.manifest, bundle_id);
    let mut directories = std::collections::BTreeSet::new();
    directories.insert(std::path::PathBuf::from("."));
    for (relative, content) in contents {
        let bytes = super::super::read_content(request.pack_root, candidate, &content.id)?;
        let relative_path = std::path::Path::new(relative);
        if let Some(directory) = relative_path.parent().filter(|directory| !directory.as_os_str().is_empty()) {
            bundle
                .create_dir_all(directory)
                .map_err(|_| super::super::super::Error::Io("catalog bundle directory create"))?;
            directories.extend(
                directory
                    .ancestors()
                    .filter(|ancestor| !ancestor.as_os_str().is_empty())
                    .map(std::path::Path::to_path_buf),
            );
        }
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = bundle
            .open_with(relative_path, &options)
            .map_err(|_| super::super::super::Error::Conflict("catalog bundle file exists"))?;
        file.write_all(&bytes).map_err(|_| super::super::super::Error::Io("catalog bundle file write"))?;
        file.sync_all().map_err(|_| super::super::super::Error::Io("catalog bundle file sync"))?;
    }
    for directory in directories.iter().rev() {
        bundle
            .open(directory)
            .and_then(|opened| opened.sync_all())
            .map_err(|_| super::super::super::Error::Io("catalog bundle directory sync"))?;
    }
    Ok(())
}

/// Re-run the workflow-template loader over the staged bytes and re-apply the pack decision.
fn verify_staged(
    request: &super::InstallRequest<'_>,
    staging: &std::path::Path,
    bundle_id: &str,
) -> Result<(), super::super::super::Error> {
    let staged = crate::workflow_template::load_catalog(staging)
        .map_err(|_| super::super::super::Error::Io("catalog staging read"))?;
    let (observed, diagnostics) = super::super::super::catalog::decision::observe(&staged);
    let none = std::collections::BTreeSet::new();
    let outcomes = super::super::super::catalog::decision::decide(
        &request.admitted.candidate.manifest,
        &super::super::super::catalog::decision::Facts {
            observed: &observed,
            diagnostics: &diagnostics,
            existing: &none,
        },
    );
    let is_verified = outcomes.iter().any(|outcome| {
        outcome.id == bundle_id && outcome.status == super::super::super::catalog::decision::Status::Admitted
    });
    if !is_verified {
        return Err(super::super::super::Error::Invalid("staged bundle failed verification"));
    }
    Ok(())
}
