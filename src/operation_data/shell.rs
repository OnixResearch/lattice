use std::io::Read as _;
use std::io::Write as _;

use super::model::*;

const PAYLOAD_DIRECTORY: &str = "payloads";
const MUTATION_LOCK_FILE: &str = ".operation-data-mutation.lock";
const MUTATION_LOCK_CONTENT: &[u8] = b"active\n";
const READ_BUFFER_BYTES: usize = 65_536;

#[derive(Debug)]
pub enum Error {
    Core(super::core::Error),
    Store(crate::store::StoreError),
    Io(&'static str),
    Conflict(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(error) => write!(formatter, "{error}"),
            Self::Store(error) => write!(formatter, "operation data store failed: {error}"),
            Self::Io(message) => write!(formatter, "operation data I/O failed: {message}"),
            Self::Conflict(message) => write!(formatter, "operation data conflict: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::core::Error> for Error {
    fn from(error: super::core::Error) -> Self {
        Self::Core(error)
    }
}

impl From<crate::store::StoreError> for Error {
    fn from(error: crate::store::StoreError) -> Self {
        Self::Store(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationReceipt {
    pub version_identity: String,
    pub content_identity: String,
    pub outcome: PublicationOutcome,
    pub payload_bytes: u64,
    pub payload_content_omitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeletionReceipt {
    pub plan_identity: String,
    pub deleted_versions: Vec<String>,
    pub missing_versions: Vec<String>,
    pub payload_content_omitted: bool,
    pub secure_erasure_claimed: bool,
}

pub struct Plane {
    store: crate::store::Store,
    root: cap_std::fs::Dir,
    mutation: std::sync::Mutex<()>,
}

struct MutationFileGuard<'a> {
    root: &'a cap_std::fs::Dir,
}

impl Drop for MutationFileGuard<'_> {
    fn drop(&mut self) {
        let _ = self.root.remove_file(MUTATION_LOCK_FILE);
        let _ = self.root.open(".").and_then(|opened| opened.sync_all());
    }
}

impl Plane {
    pub fn open(store_path: &std::path::Path, payload_root: &std::path::Path) -> Result<Self, Error> {
        validate_root(payload_root)?;
        let root = cap_std::fs::Dir::open_ambient_dir(payload_root, cap_std::ambient_authority())
            .map_err(|_| Error::Io("payload root authority"))?;
        root.create_dir_all(PAYLOAD_DIRECTORY).map_err(|_| Error::Io("payload directory create"))?;
        let store = crate::store::Store::new(store_path)?;
        Ok(Self {
            store,
            root,
            mutation: std::sync::Mutex::new(()),
        })
    }

    pub fn publish(
        &self,
        draft: &RecordDraft,
        policy: &super::core::AdmissionPolicy<'_>,
    ) -> Result<(RecordMetadata, PublicationReceipt), Error> {
        let (metadata, bytes) = super::core::admit(draft, policy)?;
        let _local_guard = self.mutation.lock().map_err(|_| Error::Conflict("mutation owner poisoned"))?;
        let _file_guard = self.acquire_mutation_file()?;
        self.publish_admitted(&metadata, &bytes)?;
        let existing = self.store.load_operation_data_metadata(&metadata.version_identity)?;
        let outcome = match super::core::publication_decision(existing.as_ref(), &metadata) {
            PublicationDecision::Append => {
                self.store.append_operation_data_metadata(&metadata)?;
                PublicationOutcome::Appended
            }
            PublicationDecision::Idempotent => PublicationOutcome::Idempotent,
            PublicationDecision::Conflict => return Err(Error::Conflict("version metadata differs")),
        };
        Ok((metadata.clone(), PublicationReceipt {
            version_identity: metadata.version_identity.clone(),
            content_identity: metadata.payload.content_identity.clone(),
            outcome,
            payload_bytes: metadata.payload.byte_length_bytes,
            payload_content_omitted: true,
        }))
    }

    pub fn load_metadata(&self, version_identity: &str) -> Result<Option<RecordMetadata>, Error> {
        Ok(self.store.load_operation_data_metadata(version_identity)?)
    }

    pub fn list_metadata(&self) -> Result<Vec<RecordMetadata>, Error> {
        Ok(self.store.list_operation_data_metadata()?)
    }

    pub fn read_payload(&self, metadata: &RecordMetadata) -> Result<Vec<u8>, Error> {
        super::core::validate_metadata(metadata)?;
        let file = self.root.open(&metadata.payload.safe_locator).map_err(|_| Error::Io("payload open"))?;
        let expected =
            usize::try_from(metadata.payload.byte_length_bytes).map_err(|_| Error::Io("payload byte length"))?;
        let mut bytes = Vec::with_capacity(expected.min(READ_BUFFER_BYTES));
        file.take(metadata.payload.byte_length_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Io("payload read"))?;
        if bytes.len() != expected {
            return Err(Error::Conflict("payload byte length differs"));
        }
        let observed = crate::operation::identity::hash_bytes(super::core::CONTENT_DOMAIN, &bytes);
        if observed != metadata.payload.content_identity {
            return Err(Error::Conflict("payload content identity differs"));
        }
        Ok(bytes)
    }

    pub fn export_payload_create_only(
        &self,
        metadata: &RecordMetadata,
        destination: &std::path::Path,
    ) -> Result<(), Error> {
        let bytes = self.read_payload(metadata)?;
        let parent = destination.parent().ok_or(Error::Io("payload export parent"))?;
        validate_root(parent)?;
        let name = destination.file_name().and_then(std::ffi::OsStr::to_str).ok_or(Error::Io("payload export name"))?;
        let directory = cap_std::fs::Dir::open_ambient_dir(parent, cap_std::ambient_authority())
            .map_err(|_| Error::Io("payload export authority"))?;
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = directory.open_with(name, &options).map_err(|_| Error::Io("payload export create"))?;
        file.write_all(&bytes).map_err(|_| Error::Io("payload export write"))?;
        file.sync_all().map_err(|_| Error::Io("payload export sync"))?;
        sync_directory(&directory)
    }

    pub fn query(&self, query: &Query, authority: QueryAuthority) -> Result<Vec<QueryResult>, Error> {
        let records = self.list_metadata()?;
        let candidates = records.into_iter().map(|metadata| (metadata, None)).collect::<Vec<_>>();
        let mut metadata_query = query.clone();
        metadata_query.field_path = None;
        let selected = super::selection::reduce_query(&candidates, &metadata_query, authority)?;
        let Some(path) = &query.field_path else {
            return Ok(selected);
        };
        let mut results = Vec::with_capacity(selected.len());
        for mut result in selected {
            if result.metadata.record_class != RecordClass::Resource {
                return Err(Error::Core(super::core::Error::Invalid("file payload is opaque")));
            }
            let bytes = self.read_payload(&result.metadata)?;
            let value: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|_| Error::Conflict("resource payload is invalid"))?;
            result.selected_value = Some(super::selection::select_declared_path(&result.metadata, &value, path)?);
            results.push(result);
        }
        Ok(results)
    }

    pub fn resolve_exact_query(&self, query: &Query, authority: QueryAuthority) -> Result<ResolvedInput, Error> {
        let results = self.query(query, authority)?;
        Ok(super::selection::resolve_exact(&results)?)
    }

    pub fn resolve_latest_query(&self, query: &Query, authority: QueryAuthority) -> Result<ResolvedInput, Error> {
        let mut bounded = query.clone();
        bounded.maximum_result_count = MAX_QUERY_RESULTS;
        let results = self.query(&bounded, authority)?;
        Ok(super::selection::resolve_latest(&results)?)
    }

    pub fn recheck_resolved_input(&self, expected: &ResolvedInput, authority: QueryAuthority) -> Result<(), Error> {
        let metadata = self.load_metadata(&expected.version_identity)?.ok_or(Error::Core(super::core::Error::Stale))?;
        if metadata.sensitivity == SensitivityClass::Sensitive && !authority.allow_sensitive {
            return Err(Error::Core(super::core::Error::Secret));
        }
        let _ = self.read_payload(&metadata)?;
        let current = ResolvedInput {
            version_identity: metadata.version_identity,
            content_identity: metadata.payload.content_identity,
        };
        Ok(super::selection::recheck(expected, &current)?)
    }
}

include!("shell/mutation.rs");

pub fn publish_projection_create_only(
    destination: &std::path::Path,
    projection: &RecordedOnlyProjection,
) -> Result<(), Error> {
    super::lifecycle::validate_recorded_only_projection(projection)?;
    if !destination.is_absolute() {
        return Err(Error::Conflict("projection destination is invalid"));
    }
    let parent = destination.parent().ok_or(Error::Io("projection parent"))?;
    validate_root(parent)?;
    let name = destination.file_name().and_then(std::ffi::OsStr::to_str).ok_or(Error::Io("projection file name"))?;
    let directory = cap_std::fs::Dir::open_ambient_dir(parent, cap_std::ambient_authority())
        .map_err(|_| Error::Io("projection directory authority"))?;
    let bytes = serde_json::to_vec_pretty(projection).map_err(|_| Error::Io("projection encode"))?;
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = directory.open_with(name, &options).map_err(|_| Error::Io("projection create"))?;
    file.write_all(&bytes).map_err(|_| Error::Io("projection write"))?;
    file.sync_all().map_err(|_| Error::Io("projection sync"))?;
    sync_directory(&directory)
}

fn validate_root(root: &std::path::Path) -> Result<(), Error> {
    if !root.is_absolute() {
        return Err(Error::Io("root must be absolute"));
    }
    let metadata = std::fs::symlink_metadata(root).map_err(|_| Error::Io("root observation"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Error::Io("root must be a non-symlink directory"));
    }
    Ok(())
}

fn sync_directory(directory: &cap_std::fs::Dir) -> Result<(), Error> {
    directory.open(".").and_then(|opened| opened.sync_all()).map_err(|_| Error::Io("directory sync"))
}
