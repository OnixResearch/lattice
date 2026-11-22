//! Explicit, operator-driven retrieval of one pack tree at an exact catalog repository commit.
//!
//! This is the only catalog path that may contact a repository. It runs `git` as a bounded
//! process with a cleared environment and lazy fetching disabled. The commit is fetched without
//! blob content, only the selected subdirectory's blobs are fetched explicitly, every file `git`
//! writes is capped at the remaining transfer budget, and every blob's size is checked before its
//! content is read. The tree is written into a private pinned staging directory and published
//! through directory handles only after the strict local pack loader accepts it.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "repository and filesystem observations return typed errors instead of terminating the process"
)]

use std::io::Write as _;

mod repository;

const TREE_DIRECTORY: &str = "tree";
const FETCH_STAGING_SUFFIX: &str = ".fetch-staging";
const BATCH_CHECK_FORMAT: &str = "--batch-check=%(objectname) %(objecttype) %(objectsize)";
const SIZE_OUTPUT_BYTES: usize = 65_536;
const LINE_SEPARATOR: &str = "\n";
#[cfg(unix)]
const REGULAR_FILE_MODE: u32 = 0o644;
#[cfg(unix)]
const EXECUTABLE_FILE_MODE: u32 = 0o755;

/// One explicit fetch: an allowlisted repository locator, an exact commit, a transfer budget and
/// a new destination.
#[derive(Debug, Clone)]
pub struct Request {
    pub repository: String,
    pub commit: String,
    pub subdirectory: Option<String>,
    pub expected_pack_identity: Option<String>,
    pub destination: std::path::PathBuf,
    pub transfer_limit_bytes: u64,
    pub process: crate::process::core::RuntimeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Receipt {
    pub schema: String,
    pub commit: String,
    pub object_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdirectory: Option<String>,
    pub pack_id: String,
    pub pack_identity: String,
    pub manifest_identity: String,
    pub file_count: usize,
    pub workflow_bundle_count: usize,
    pub transferred_bytes: u64,
    pub transfer_limit_bytes: u64,
    pub installed: bool,
    pub grants_authority: bool,
}

/// Fetch the exact commit into a private staging repository, materialize the selected regular
/// files, verify them with the strict pack loader, and publish the pack tree create-only.
pub fn run(request: &Request) -> Result<Receipt, super::super::super::Error> {
    super::super::super::catalog::source::validate_repository_locator(&request.repository)?;
    super::super::super::catalog::source::validate_commit(&request.commit)?;
    super::super::super::catalog::source::validate_subdirectory(request.subdirectory.as_deref())?;
    super::super::super::catalog::transfer::check_budget(request.transfer_limit_bytes)?;
    if let Some(identity) = &request.expected_pack_identity {
        super::super::super::validation::validate_identity(identity)?;
    }
    let is_absent = matches!(
        std::fs::symlink_metadata(&request.destination),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound
    );
    if !request.destination.is_absolute() || !is_absent {
        return Err(super::super::super::Error::Conflict("fetch destination must be absolute and absent"));
    }
    let parent = request
        .destination
        .parent()
        .ok_or(super::super::super::Error::Invalid("fetch destination parent"))?;
    super::super::installation::validate_directory(parent)?;
    let name = super::super::installation::safe_file_name(&request.destination)?;
    let staging = super::pinned::Staging::create(parent, &format!(".{name}{FETCH_STAGING_SUFFIX}"))?;
    let result = retrieve_tree(request, &staging, name);
    let cleanup = staging.remove();
    let receipt = result?;
    cleanup?;
    Ok(receipt)
}

fn retrieve_tree(
    request: &Request,
    staging: &super::pinned::Staging,
    name: &str,
) -> Result<Receipt, super::super::super::Error> {
    let git = repository::Git {
        staging: staging.path(),
        process: &request.process,
        limit_bytes: request.transfer_limit_bytes,
    };
    git.prepare(request)?;
    git.fetch_commit(&request.commit)?;
    let listing = git.list(&request.commit, request.subdirectory.as_deref())?;
    let entries = super::super::super::catalog::source::parse_tree(&listing, request.subdirectory.as_deref())?;
    git.fetch_blobs(&entries)?;
    let transferred_bytes = git.used_bytes()?;
    let sizes = super::super::super::catalog::transfer::validate_object_sizes(
        &git.run_local(
            &[repository::GIT_DIRECTORY_ARGUMENT, "cat-file", BATCH_CHECK_FORMAT],
            &object_lines(&entries),
            SIZE_OUTPUT_BYTES,
        )?,
        &entries,
    )?;
    staging
        .dir()
        .create_dir(TREE_DIRECTORY)
        .map_err(|_| super::super::super::Error::Io("fetch tree create"))?;
    let tree_inode = staging.entry_inode(TREE_DIRECTORY)?;
    let tree = staging
        .dir()
        .open_dir(TREE_DIRECTORY)
        .map_err(|_| super::super::super::Error::Io("fetch tree authority"))?;
    materialize(&git, &tree, &entries, &sizes)?;
    staging.ensure_pinned()?;
    let candidate = super::super::load(&staging.path().join(TREE_DIRECTORY))?;
    staging.ensure_pinned()?;
    if request.expected_pack_identity.as_ref().is_some_and(|expected| expected != &candidate.pack_identity) {
        return Err(super::super::super::Error::Invalid("fetched pack identity differs"));
    }
    staging.publish(TREE_DIRECTORY, tree_inode, super::pinned::Target {
        dir: staging.parent(),
        name,
    })?;
    Ok(Receipt {
        schema: super::super::super::catalog::source::FETCH_RECEIPT_SCHEMA.to_string(),
        commit: request.commit.clone(),
        object_format: super::super::super::catalog::transfer::object_format(&request.commit).to_string(),
        subdirectory: request.subdirectory.clone(),
        pack_id: candidate.manifest.pack_id.clone(),
        pack_identity: candidate.pack_identity.clone(),
        manifest_identity: candidate.manifest_identity.clone(),
        file_count: entries.len(),
        workflow_bundle_count: candidate.manifest.workflow_bundles.len(),
        transferred_bytes,
        transfer_limit_bytes: request.transfer_limit_bytes,
        installed: false,
        grants_authority: false,
    })
}

fn object_lines(entries: &[super::super::super::catalog::source::TreeEntry]) -> Vec<u8> {
    let mut lines = entries.iter().map(|entry| entry.object.as_str()).collect::<Vec<_>>().join(LINE_SEPARATOR);
    lines.push_str(LINE_SEPARATOR);
    lines.into_bytes()
}

/// Read each size-checked blob and write it create-only through the pinned tree handle.
fn materialize(
    git: &repository::Git<'_>,
    tree: &cap_std::fs::Dir,
    entries: &[super::super::super::catalog::source::TreeEntry],
    sizes: &[u64],
) -> Result<(), super::super::super::Error> {
    for (entry, size_bytes) in entries.iter().zip(sizes) {
        let limit_bytes = usize::try_from(*size_bytes)
            .map_err(|_| super::super::super::Error::Invalid("catalog object size bound"))?;
        let bytes = git.run_local(
            &[repository::GIT_DIRECTORY_ARGUMENT, "cat-file", "blob", &entry.object],
            &[],
            limit_bytes.max(1),
        )?;
        if u64::try_from(bytes.len()).ok() != Some(*size_bytes) {
            return Err(super::super::super::Error::Invalid("catalog object size bound"));
        }
        let path = std::path::Path::new(&entry.path);
        if let Some(directory) = path.parent().filter(|directory| !directory.as_os_str().is_empty()) {
            tree.create_dir_all(directory)
                .map_err(|_| super::super::super::Error::Io("fetch directory create"))?;
        }
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = tree
            .open_with(path, &options)
            .map_err(|_| super::super::super::Error::Conflict("fetch file exists"))?;
        file.write_all(&bytes).map_err(|_| super::super::super::Error::Io("fetch file write"))?;
        #[cfg(unix)]
        {
            use cap_std::fs::PermissionsExt as _;
            let mode = if entry.is_executable {
                EXECUTABLE_FILE_MODE
            } else {
                REGULAR_FILE_MODE
            };
            file.set_permissions(cap_std::fs::Permissions::from_mode(mode))
                .map_err(|_| super::super::super::Error::Io("fetch file mode"))?;
        }
        file.sync_all().map_err(|_| super::super::super::Error::Io("fetch file sync"))?;
    }
    super::pinned::sync(tree)
}
