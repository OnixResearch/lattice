//! Create-only, per-bundle atomic placement of admitted pack bundles into a local catalog.
//!
//! Every declared bundle's bytes are read and verified against the pack declaration into memory
//! before anything is staged, so any drift refuses the whole pack. The captured bytes are written
//! into one private, pinned staging directory, verified there by the workflow-template loader,
//! and each admitted bundle is renamed into the catalog through directory handles only while its
//! id is absent and the catalog stays within the loader's bundle limit.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "installation failures become per-bundle outcomes instead of terminating the process"
)]

use std::io::Write as _;

const STAGING_PREFIX: &str = ".lattice-catalog-";
const STAGING_SUFFIX: &str = ".staging";
const STAGING_IDENTITY_CHARACTERS: usize = 16;
const PLACEMENT_CODE: &str = "installation";
const CURRENT_DIRECTORY: &str = ".";

/// Verified bytes of every declared bundle: bundle id to `(bundle-relative path, bytes)`.
pub(super) type Captured = std::collections::BTreeMap<String, Vec<(String, Vec<u8>)>>;

/// Read every declared bundle file and verify it against the pack declaration.
///
/// Returns the captured bytes and the ids of every bundle whose bytes could not be read or no
/// longer match; a non-empty drift list must refuse the whole pack.
pub(super) fn capture(request: &super::InstallRequest<'_>) -> (Captured, Vec<String>) {
    let candidate = &request.admitted.candidate;
    let read = candidate
        .manifest
        .workflow_bundles
        .iter()
        .map(|declaration| {
            let contents = super::super::super::catalog::bundle_contents(&candidate.manifest, &declaration.id);
            let files = contents
                .iter()
                .map(|(relative, content)| {
                    super::super::read_content(request.pack_root, candidate, &content.id)
                        .map(|bytes| ((*relative).to_string(), bytes))
                })
                .collect::<Result<Vec<_>, _>>();
            (declaration.id.clone(), files)
        })
        .collect::<Vec<_>>();
    let drifted = read.iter().filter(|(_, files)| files.is_err()).map(|(id, _)| id.clone()).collect();
    let captured = read.into_iter().filter_map(|(id, files)| files.ok().map(|files| (id, files))).collect();
    (captured, drifted)
}

/// Decide every bundle against one loader result and the catalog's current entries.
pub(super) fn decide_against(
    manifest: &super::super::super::model::Manifest,
    loaded: &crate::workflow_template::Catalog,
    catalog: &cap_std::fs::Dir,
) -> Result<Vec<super::super::super::catalog::decision::BundleOutcome>, super::super::super::Error> {
    let (observed, diagnostics) = super::super::super::catalog::decision::observe(loaded);
    let existing = manifest
        .workflow_bundles
        .iter()
        .filter(|declaration| is_occupied(catalog, &declaration.id))
        .map(|declaration| declaration.id.clone())
        .collect();
    let facts = super::super::super::catalog::decision::Facts {
        observed: &observed,
        diagnostics: &diagnostics,
        existing: &existing,
        free_slots: super::super::super::catalog::decision::free_slots(directory_count(catalog)?),
    };
    Ok(super::super::super::catalog::decision::decide(manifest, &facts))
}

/// Stage the captured bundles privately, verify them, and publish each admitted one.
pub(super) fn publish(
    request: &super::InstallRequest<'_>,
    parent: &std::path::Path,
    catalog: &cap_std::fs::Dir,
    captured: &Captured,
) -> Result<Vec<super::super::super::catalog::decision::BundleOutcome>, super::super::super::Error> {
    let identity = request.admitted.candidate.pack_identity.trim_start_matches("b3:");
    let short_identity = identity.get(..STAGING_IDENTITY_CHARACTERS).unwrap_or(identity);
    let staging = super::pinned::Staging::create(parent, &format!("{STAGING_PREFIX}{short_identity}{STAGING_SUFFIX}"))?;
    let result = stage_and_publish(request, &staging, catalog, captured);
    let cleanup = staging.remove();
    let outcomes = result?;
    cleanup?;
    Ok(outcomes)
}

fn stage_and_publish(
    request: &super::InstallRequest<'_>,
    staging: &super::pinned::Staging,
    catalog: &cap_std::fs::Dir,
    captured: &Captured,
) -> Result<Vec<super::super::super::catalog::decision::BundleOutcome>, super::super::super::Error> {
    for (bundle_id, files) in captured {
        super::super::super::catalog::validate_bundle_id(bundle_id)?;
        write_bundle(staging.dir(), bundle_id, files)?;
    }
    let inodes = captured
        .keys()
        .map(|bundle_id| Ok((bundle_id.as_str(), staging.entry_inode(bundle_id)?)))
        .collect::<Result<std::collections::BTreeMap<_, _>, super::super::super::Error>>()?;
    staging.ensure_pinned()?;
    let loaded = crate::workflow_template::load_catalog(staging.path())
        .map_err(|_| super::super::super::Error::Io("catalog staging read"))?;
    staging.ensure_pinned()?;
    let mut outcomes = decide_against(&request.admitted.candidate.manifest, &loaded, catalog)?;
    for outcome in &mut outcomes {
        if outcome.status != super::super::super::catalog::decision::Status::Admitted {
            continue;
        }
        if super::super::super::catalog::decision::free_slots(directory_count(catalog)?) == 0 {
            super::super::super::catalog::decision::reject_for_capacity(outcome);
            continue;
        }
        let inode = *inodes.get(outcome.id.as_str()).ok_or(super::super::super::Error::Invalid("staged bundle"))?;
        let target = super::pinned::Target {
            dir: catalog,
            name: &outcome.id,
        };
        match staging.publish(&outcome.id, inode, target) {
            Ok(()) => outcome.status = super::super::super::catalog::decision::Status::Installed,
            Err(error) => {
                outcome.status = super::super::super::catalog::decision::Status::Rejected;
                outcome.code = Some(PLACEMENT_CODE.to_string());
                outcome.reason = Some(error.to_string());
            }
        }
    }
    Ok(outcomes)
}

/// Write one captured bundle create-only beneath the staging handle, then sync every directory.
fn write_bundle(
    staging: &cap_std::fs::Dir,
    bundle_id: &str,
    files: &[(String, Vec<u8>)],
) -> Result<(), super::super::super::Error> {
    staging
        .create_dir(bundle_id)
        .map_err(|_| super::super::super::Error::Io("catalog staging create"))?;
    let bundle = staging.open_dir(bundle_id).map_err(|_| super::super::super::Error::Io("catalog staging open"))?;
    let mut directories = std::collections::BTreeSet::new();
    directories.insert(std::path::PathBuf::from(CURRENT_DIRECTORY));
    for (relative, bytes) in files {
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
        file.write_all(bytes).map_err(|_| super::super::super::Error::Io("catalog bundle file write"))?;
        file.sync_all().map_err(|_| super::super::super::Error::Io("catalog bundle file sync"))?;
    }
    for directory in directories.iter().rev() {
        bundle
            .open(directory)
            .and_then(|opened| opened.sync_all())
            .map_err(|_| super::super::super::Error::Io("catalog bundle directory sync"))?;
    }
    super::pinned::sync(staging)
}

/// Any entry at `<catalog>/<id>`, including a symlink or an unreadable one, blocks that bundle.
fn is_occupied(catalog: &cap_std::fs::Dir, bundle_id: &str) -> bool {
    !matches!(catalog.symlink_metadata(bundle_id), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
}

/// Directories the workflow-template loader would count, stopping just past its limit.
fn directory_count(catalog: &cap_std::fs::Dir) -> Result<usize, super::super::super::Error> {
    let mut count = 0usize;
    for entry in catalog.entries().map_err(|_| super::super::super::Error::Io("catalog read"))? {
        let entry = entry.map_err(|_| super::super::super::Error::Io("catalog entry"))?;
        let file_type = entry.file_type().map_err(|_| super::super::super::Error::Io("catalog entry type"))?;
        if file_type.is_dir() && !file_type.is_symlink() {
            count = count.saturating_add(1);
        }
        if count > crate::workflow_template::MAX_CATALOG_BUNDLES {
            break;
        }
    }
    Ok(count)
}
