#![allow(
    unknown_lints,
    ambiguous_params,
    ambient_clock,
    assertion_density,
    clone_then_borrow,
    excessive_file_length,
    fallible_int_fallback,
    non_trait_imports,
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    sentinel_fallback,
    unbounded_collection_growth,
    reason = "the publication shell keeps the C8 commit order and its durable file effects explicit"
)]

//! Checkpoint root I/O and the node-boundary hook. Pure decisions live in `core`.

use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use lattice_workflow_contract::Identity;

use super::CheckpointRuntime;
use super::CommitMarker;
use super::DrainAcknowledgement;
use super::LocalRuntime;
use super::Refusal;
use super::WorkspaceObject;

mod hook;
pub use hook::HookSeed;
pub use hook::NodeBoundaryHook;

const OBJECTS: &str = "objects";
const WORKSPACE: &str = "workspace";
const REQUESTS: &str = "requests";
const MANIFESTS: &str = "manifests";
const COMMITS: &str = "commits";
const DRAIN_REQUEST: &str = "drain-request";
const DRAINED: &str = "drained.json";
const JSON_SUFFIX: &str = ".json";
const PUBLICATION_DIGITS: usize = 20;

/// One checkpoint root: content-addressed objects, manifests and commit markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointRoot {
    path: PathBuf,
}

impl CheckpointRoot {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn put_node_object(&self, identity: &Identity, bytes: &[u8]) -> Result<(), Refusal> {
        self.write_verified(OBJECTS, super::core::identity_hex(identity), bytes)
    }

    pub fn put_workspace_object(&self, digest: &str, bytes: &[u8]) -> Result<(), Refusal> {
        self.write_verified(WORKSPACE, digest, bytes)
    }

    pub fn put_request(&self, identity: &Identity, bytes: &[u8]) -> Result<(), Refusal> {
        self.write_verified(REQUESTS, &json_name(identity), bytes)
    }

    pub fn put_manifest(&self, identity: &Identity, bytes: &[u8]) -> Result<(), Refusal> {
        self.write_verified(MANIFESTS, &json_name(identity), bytes)
    }

    // r[impl lattice.application_checkpoint.commit]
    /// Write the commit marker last. Its file name orders publications in this root.
    pub fn commit(&self, manifest: &Identity) -> Result<u64, Refusal> {
        let publication = self.committed()?.last().map_or(0, |(publication, _)| publication.saturating_add(1));
        let marker = CommitMarker {
            manifest: manifest.clone(),
        };
        let bytes = serde_json::to_vec(&marker).map_err(|_| Refusal::Encoding)?;
        let name = format!(
            "{publication:0width$}-{}{JSON_SUFFIX}",
            super::core::identity_hex(manifest),
            width = PUBLICATION_DIGITS
        );
        self.write_verified(COMMITS, &name, &bytes)?;
        Ok(publication)
    }

    /// Committed manifests in publication order. Malformed marker names are ignored.
    pub fn committed(&self) -> Result<Vec<(u64, Identity)>, Refusal> {
        let directory = self.path.join(COMMITS);
        let entries = match std::fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(_) => return Err(Refusal::Io),
        };
        let mut committed = Vec::new();
        for entry in entries {
            let name = entry.map_err(|_| Refusal::Io)?.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some((publication, rest)) = name.split_once('-') else {
                continue;
            };
            let Some(digest) = rest.strip_suffix(JSON_SUFFIX) else {
                continue;
            };
            let (Ok(publication), Ok(identity)) = (
                publication.parse::<u64>(),
                Identity::new(format!("{}{digest}", lattice_workflow_contract::BLAKE3_PREFIX)),
            ) else {
                continue;
            };
            committed.push((publication, identity));
        }
        committed.sort();
        Ok(committed)
    }

    /// The commit marker for `manifest`, if one was written and names it.
    pub fn marker(&self, manifest: &Identity) -> Result<Option<CommitMarker>, Refusal> {
        let Some((publication, _)) = self.committed()?.into_iter().find(|(_, identity)| identity == manifest) else {
            return Ok(None);
        };
        let name = format!(
            "{publication:0width$}-{}{JSON_SUFFIX}",
            super::core::identity_hex(manifest),
            width = PUBLICATION_DIGITS
        );
        let Some(bytes) = self.read_bounded(COMMITS, &name)? else {
            return Ok(None);
        };
        let marker: CommitMarker = serde_json::from_slice(&bytes).map_err(|_| Refusal::Schema)?;
        Ok((marker.manifest == *manifest).then_some(marker))
    }

    pub fn manifest_bytes(&self, manifest: &Identity) -> Result<Option<Vec<u8>>, Refusal> {
        self.read_bounded(MANIFESTS, &json_name(manifest))
    }

    pub fn node_object(&self, identity: &Identity) -> Result<Option<Vec<u8>>, Refusal> {
        self.read_bounded(OBJECTS, super::core::identity_hex(identity))
    }

    pub fn workspace_object(&self, digest: &str) -> Result<Option<Vec<u8>>, Refusal> {
        if digest.len() != blake3::OUT_LEN * 2 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Ok(None);
        }
        self.read_bounded(WORKSPACE, digest)
    }

    pub fn request_object(&self, identity: &Identity) -> Result<Option<Vec<u8>>, Refusal> {
        self.read_bounded(REQUESTS, &json_name(identity))
    }

    /// A host-requested drain: the hook parks the next effect after a quiescent manifest.
    #[must_use]
    pub fn drain_requested(&self) -> bool {
        self.path.join(DRAIN_REQUEST).exists()
    }

    pub fn acknowledge_drain(&self, manifest: &Identity) -> Result<(), Refusal> {
        let acknowledgement = DrainAcknowledgement {
            schema: super::DRAIN_SCHEMA.to_string(),
            manifest: manifest.clone(),
        };
        let bytes = serde_json::to_vec(&acknowledgement).map_err(|_| Refusal::Encoding)?;
        self.write_atomic(&self.path.clone(), DRAINED, &bytes)
    }

    pub fn drain_acknowledgement(&self) -> Result<Option<DrainAcknowledgement>, Refusal> {
        match std::fs::read(self.path.join(DRAINED)) {
            Ok(bytes) => serde_json::from_slice(&bytes).map(Some).map_err(|_| Refusal::Schema),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err(Refusal::Io),
        }
    }

    fn read_bounded(&self, directory: &str, name: &str) -> Result<Option<Vec<u8>>, Refusal> {
        let file = match std::fs::File::open(self.path.join(directory).join(name)) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Refusal::Io),
        };
        let mut bytes = Vec::new();
        file.take(super::MAXIMUM_OBJECT_BYTES.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| Refusal::Io)?;
        if u64::try_from(bytes.len()).map_err(|_| Refusal::Io)? > super::MAXIMUM_OBJECT_BYTES {
            return Err(Refusal::ObjectMismatch);
        }
        Ok(Some(bytes))
    }

    /// Write, fsync, rename, fsync the directory, then read back and compare.
    fn write_verified(&self, directory: &str, name: &str, bytes: &[u8]) -> Result<(), Refusal> {
        let directory = self.path.join(directory);
        if let Ok(existing) = std::fs::read(directory.join(name))
            && existing == bytes
        {
            return Ok(());
        }
        self.write_atomic(&directory, name, bytes)?;
        let written = std::fs::read(directory.join(name)).map_err(|_| Refusal::Io)?;
        if written != bytes {
            return Err(Refusal::ObjectMismatch);
        }
        Ok(())
    }

    fn write_atomic(&self, directory: &Path, name: &str, bytes: &[u8]) -> Result<(), Refusal> {
        std::fs::create_dir_all(directory).map_err(|_| Refusal::Io)?;
        let temporary = directory.join(format!(".{name}.{}.tmp", std::process::id()));
        {
            let mut file = std::fs::File::create(&temporary).map_err(|_| Refusal::Io)?;
            file.write_all(bytes).map_err(|_| Refusal::Io)?;
            file.sync_all().map_err(|_| Refusal::Io)?;
        }
        std::fs::rename(&temporary, directory.join(name)).map_err(|_| Refusal::Io)?;
        std::fs::File::open(directory).and_then(|directory| directory.sync_all()).map_err(|_| Refusal::Io)
    }
}

fn json_name(identity: &Identity) -> String {
    format!("{}{JSON_SUFFIX}", super::core::identity_hex(identity))
}

/// Facts of the running binary. The revision is the executor/restore source digest, so a
/// separately built binary from the same source on another architecture reports the same value.
pub fn local_runtime(environment: Option<String>) -> Result<LocalRuntime, Refusal> {
    static BINARY: std::sync::LazyLock<Option<String>> = std::sync::LazyLock::new(|| {
        let executable = std::env::current_exe().ok()?;
        let mut file = std::fs::File::open(executable).ok()?;
        let mut hasher = blake3::Hasher::new();
        std::io::copy(&mut file, &mut hasher).ok()?;
        Some(hasher.finalize().to_hex().to_string())
    });
    let binary = BINARY.clone().ok_or(Refusal::Runtime)?;
    Ok(LocalRuntime {
        architecture: std::env::consts::ARCH.to_string(),
        runtime: CheckpointRuntime {
            lattice_revision: format!(
                "lattice-{}-runtime-{}",
                env!("CARGO_PKG_VERSION"),
                env!("LATTICE_RUNTIME_IMPLEMENTATION_BLAKE3")
            ),
            lattice_binary_blake3: binary,
            environment,
        },
    })
}

/// Snapshot regular files under `workspace` into the root. Symlinks and special files are refused.
pub fn snapshot_workspace(root: &CheckpointRoot, workspace: &Path) -> Result<Vec<WorkspaceObject>, Refusal> {
    let mut objects = Vec::new();
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let entries = std::fs::read_dir(workspace.join(&relative)).map_err(|_| Refusal::Io)?;
        for entry in entries {
            let entry = entry.map_err(|_| Refusal::Io)?;
            let kind = entry.file_type().map_err(|_| Refusal::Io)?;
            let path = relative.join(entry.file_name());
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() {
                return Err(Refusal::WorkspacePath);
            }
            let name = path.to_str().ok_or(Refusal::WorkspacePath)?.to_string();
            if !super::core::is_workspace_path(&name) || objects.len() >= super::MAXIMUM_WORKSPACE_OBJECTS {
                return Err(Refusal::WorkspacePath);
            }
            let bytes = std::fs::read(entry.path()).map_err(|_| Refusal::Io)?;
            let length = u64::try_from(bytes.len()).map_err(|_| Refusal::Io)?;
            if length > super::MAXIMUM_OBJECT_BYTES {
                return Err(Refusal::WorkspacePath);
            }
            let digest = super::core::content_hex(&bytes);
            root.put_workspace_object(&digest, &bytes)?;
            objects.push(WorkspaceObject {
                path: name,
                blake3: digest,
                bytes: length,
            });
        }
    }
    objects.sort();
    Ok(objects)
}
