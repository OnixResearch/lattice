//! Explicit, operator-driven retrieval of one pack tree at an exact catalog repository commit.
//!
//! This is the only catalog path that may contact a repository. It runs `git` as a bounded
//! process with a cleared environment, writes regular files only, and publishes the tree only
//! after the strict local pack loader accepts it.
#![allow(
    unknown_lints,
    assertion_density,
    reason = "repository and filesystem observations return typed errors instead of terminating the process"
)]
#![allow(
    ambient_env,
    reason = "the fetch shell forwards only an allowlisted transport environment to the git process"
)]

use std::io::Write as _;

const GIT_PROGRAM: &str = "git";
const GIT_DIRECTORY: &str = "repository.git";
const GIT_DIRECTORY_ARGUMENT: &str = "--git-dir=repository.git";
const TREE_DIRECTORY: &str = "tree";
const FETCH_STAGING_SUFFIX: &str = ".fetch-staging";
const FETCH_TIMEOUT_SECONDS: u64 = 600;
const LOCAL_TIMEOUT_SECONDS: u64 = 60;
const LISTING_OUTPUT_BYTES: usize = 1_048_576;
const REVISION_OUTPUT_BYTES: usize = 256;
const FORWARDED_ENVIRONMENT: [&str; 4] = ["PATH", "SSL_CERT_FILE", "SSL_CERT_DIR", "SSH_AUTH_SOCK"];
#[cfg(unix)]
const REGULAR_FILE_MODE: u32 = 0o644;
#[cfg(unix)]
const EXECUTABLE_FILE_MODE: u32 = 0o755;

/// One explicit fetch: an allowlisted repository locator, an exact commit and a new destination.
#[derive(Debug, Clone)]
pub struct Request {
    pub repository: String,
    pub commit: String,
    pub subdirectory: Option<String>,
    pub expected_pack_identity: Option<String>,
    pub destination: std::path::PathBuf,
    pub process: crate::process::core::RuntimeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Receipt {
    pub schema: String,
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdirectory: Option<String>,
    pub pack_id: String,
    pub pack_identity: String,
    pub manifest_identity: String,
    pub file_count: usize,
    pub workflow_bundle_count: usize,
    pub installed: bool,
    pub grants_authority: bool,
}

/// Fetch the exact commit into a private staging repository, materialize the selected regular
/// files, verify them with the strict pack loader, and publish the pack tree create-only.
pub fn run(request: &Request) -> Result<Receipt, super::super::super::Error> {
    super::super::super::catalog::source::validate_repository_locator(&request.repository)?;
    super::super::super::catalog::source::validate_commit(&request.commit)?;
    super::super::super::catalog::source::validate_subdirectory(request.subdirectory.as_deref())?;
    if let Some(identity) = &request.expected_pack_identity {
        super::super::super::validation::validate_identity(identity)?;
    }
    if !request.destination.is_absolute() || super::placement::is_occupied(&request.destination) {
        return Err(super::super::super::Error::Conflict("fetch destination must be absolute and absent"));
    }
    let parent = request
        .destination
        .parent()
        .ok_or(super::super::super::Error::Invalid("fetch destination parent"))?;
    super::super::installation::validate_directory(parent)?;
    let name = super::super::installation::safe_file_name(&request.destination)?;
    let staging = parent.join(format!(".{name}{FETCH_STAGING_SUFFIX}"));
    std::fs::create_dir(&staging).map_err(|_| super::super::super::Error::Conflict("fetch staging exists"))?;
    let result = retrieve_tree(request, &staging);
    let cleanup = super::super::installation::remove_staging(&staging);
    let receipt = result?;
    cleanup?;
    super::super::installation::sync_directory(parent)?;
    Ok(receipt)
}

fn retrieve_tree(request: &Request, staging: &std::path::Path) -> Result<Receipt, super::super::super::Error> {
    let git = Git {
        staging,
        process: &request.process,
    };
    retrieve_commit(&git, request)?;
    let mut listing_arguments = vec![
        GIT_DIRECTORY_ARGUMENT,
        "ls-tree",
        "-r",
        "-z",
        "--full-tree",
        &request.commit,
    ];
    if let Some(subdirectory) = &request.subdirectory {
        listing_arguments.extend(["--", subdirectory.as_str()]);
    }
    let listing = git.run(&listing_arguments, LISTING_OUTPUT_BYTES, LOCAL_TIMEOUT_SECONDS)?;
    let entries = super::super::super::catalog::source::parse_tree(&listing, request.subdirectory.as_deref())?;
    let tree = staging.join(TREE_DIRECTORY);
    std::fs::create_dir(&tree).map_err(|_| super::super::super::Error::Io("fetch tree create"))?;
    materialize(&git, &tree, &entries)?;
    let candidate = super::super::load(&tree)?;
    if request.expected_pack_identity.as_ref().is_some_and(|expected| expected != &candidate.pack_identity) {
        return Err(super::super::super::Error::Invalid("fetched pack identity differs"));
    }
    std::fs::rename(&tree, &request.destination).map_err(|_| super::super::super::Error::Io("fetch publish"))?;
    Ok(Receipt {
        schema: super::super::super::catalog::source::FETCH_RECEIPT_SCHEMA.to_string(),
        commit: request.commit.clone(),
        subdirectory: request.subdirectory.clone(),
        pack_id: candidate.manifest.pack_id.clone(),
        pack_identity: candidate.pack_identity.clone(),
        manifest_identity: candidate.manifest_identity.clone(),
        file_count: entries.len(),
        workflow_bundle_count: candidate.manifest.workflow_bundles.len(),
        installed: false,
        grants_authority: false,
    })
}

/// Fetch exactly one commit into a new bare repository and confirm the fetched object.
fn retrieve_commit(git: &Git<'_>, request: &Request) -> Result<(), super::super::super::Error> {
    git.run(&["init", "--bare", "--quiet", GIT_DIRECTORY], REVISION_OUTPUT_BYTES, LOCAL_TIMEOUT_SECONDS)?;
    git.run(
        &[
            "-c",
            "transfer.fsckObjects=true",
            GIT_DIRECTORY_ARGUMENT,
            "fetch",
            "--quiet",
            "--no-tags",
            "--depth=1",
            "--",
            &request.repository,
            &request.commit,
        ],
        REVISION_OUTPUT_BYTES,
        FETCH_TIMEOUT_SECONDS,
    )?;
    let peeled = format!("{}^{{commit}}", request.commit);
    let revision = git.run(
        &[
            GIT_DIRECTORY_ARGUMENT,
            "rev-parse",
            "--verify",
            "--end-of-options",
            &peeled,
        ],
        REVISION_OUTPUT_BYTES,
        LOCAL_TIMEOUT_SECONDS,
    )?;
    if revision.trim_ascii() != request.commit.as_bytes() {
        return Err(super::super::super::Error::Invalid("fetched commit differs"));
    }
    Ok(())
}

/// Write each listed blob create-only through a capability directory; never a link or device.
fn materialize(
    git: &Git<'_>,
    tree: &std::path::Path,
    entries: &[super::super::super::catalog::source::TreeEntry],
) -> Result<(), super::super::super::Error> {
    let root = cap_std::fs::Dir::open_ambient_dir(tree, cap_std::ambient_authority())
        .map_err(|_| super::super::super::Error::Io("fetch tree authority"))?;
    let content_limit_bytes = usize::try_from(super::super::super::model::MAXIMUM_PACK_BYTES)
        .map_err(|_| super::super::super::Error::Invalid("fetch content bound"))?;
    for entry in entries {
        let bytes = git.run(
            &[GIT_DIRECTORY_ARGUMENT, "cat-file", "blob", &entry.object],
            content_limit_bytes,
            LOCAL_TIMEOUT_SECONDS,
        )?;
        let path = std::path::Path::new(&entry.path);
        if let Some(directory) = path.parent().filter(|directory| !directory.as_os_str().is_empty()) {
            root.create_dir_all(directory)
                .map_err(|_| super::super::super::Error::Io("fetch directory create"))?;
        }
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = root
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
    Ok(())
}

/// A bounded `git` invocation inside the private staging directory with an isolated config.
struct Git<'git> {
    staging: &'git std::path::Path,
    process: &'git crate::process::core::RuntimeConfig,
}

impl Git<'_> {
    fn run(
        &self,
        arguments: &[&str],
        output_limit_bytes: usize,
        timeout_secs: u64,
    ) -> Result<Vec<u8>, super::super::super::Error> {
        let staging =
            self.staging.to_str().ok_or(super::super::super::Error::Invalid("fetch staging path encoding"))?;
        let mut environment = FORWARDED_ENVIRONMENT
            .iter()
            .filter_map(|key| {
                let value = std::env::var(key).ok().filter(|value| !value.is_empty())?;
                Some(((*key).to_string(), value))
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        environment.insert("HOME".to_string(), staging.to_string());
        environment.insert("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string());
        environment.insert("GIT_CONFIG_GLOBAL".to_string(), "/dev/null".to_string());
        environment.insert("GIT_TERMINAL_PROMPT".to_string(), "0".to_string());
        environment.insert("GIT_ALLOW_PROTOCOL".to_string(), "file:https:ssh".to_string());
        let request = crate::agent_node::ProcessRequest {
            program: GIT_PROGRAM.to_string(),
            args: arguments.iter().map(|argument| (*argument).to_string()).collect(),
            cwd: Some(staging.to_string()),
            env: environment,
            stdin: Vec::new(),
            timeout_secs,
            output_limit_bytes,
        };
        let output = crate::process::shell::run_bounded_clear_environment(&request, self.process)?;
        Ok(output.stdout)
    }
}
