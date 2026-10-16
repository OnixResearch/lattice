#![allow(
    unknown_lints,
    assertion_density,
    reason = "fallible record admission uses explicit typed diagnostics instead of process-terminating assertions"
)]

use super::model::*;

pub(super) const CONTENT_DOMAIN: &str = "lattice.operation-data-content.v1";
const LOCATOR_DOMAIN: &str = "lattice.operation-data-locator.v1";
const RECORD_DOMAIN: &str = "lattice.operation-data-record.v1";
const VERSION_DOMAIN: &str = "lattice.operation-data-version.v1";
const STORAGE_BACKEND: &str = "capability-file-v1";
const LOCATOR_PREFIX: &str = "payloads/";
const LOCATOR_SUFFIX: &str = ".payload";
const PRIVATE_PATH_PREFIXES: [&str; 3] = ["/home/", "/root/", "/Users/"];
const SECRET_KEY_FRAGMENTS: [&str; 8] = [
    "authorization",
    "credential",
    "password",
    "secret",
    "token",
    "api_key",
    "apikey",
    "private_key",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Limit(&'static str),
    Secret,
    Empty,
    Ambiguous,
    Stale,
    Conflict,
    Encoding,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) => write!(formatter, "invalid operation data: {message}"),
            Self::Limit(message) => write!(formatter, "operation data limit exceeded: {message}"),
            Self::Secret => write!(formatter, "operation data contains prohibited material"),
            Self::Empty => write!(formatter, "operation data selection is empty"),
            Self::Ambiguous => write!(formatter, "operation data selection is ambiguous"),
            Self::Stale => write!(formatter, "operation data selection changed before dispatch"),
            Self::Conflict => write!(formatter, "operation data publication conflicts with existing data"),
            Self::Encoding => write!(formatter, "operation data encoding failed"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct AdmissionPolicy<'a> {
    pub allow_sensitive: bool,
    pub prohibited_fragments: &'a [String],
}

struct AdmittedPayload {
    record_class: RecordClass,
    descriptor: PayloadDescriptor,
    bytes: Vec<u8>,
}

// r[impl lattice.operation_data.records.resource]
// r[impl lattice.operation_data.records.file]
// r[impl lattice.operation_data.confidentiality.safe]
pub fn admit(draft: &RecordDraft, policy: &AdmissionPolicy<'_>) -> Result<(RecordMetadata, Vec<u8>), Error> {
    validate_draft(draft, policy)?;
    let admitted = admit_payload(&draft.payload, policy)?;
    let metadata = build_metadata(draft, &admitted)?;
    validate_metadata(&metadata)?;
    Ok((metadata, admitted.bytes))
}

fn admit_payload(payload: &PayloadDraft, policy: &AdmissionPolicy<'_>) -> Result<AdmittedPayload, Error> {
    let (record_class, bytes, schema_identity, media_class, mut selectable_paths) = match payload {
        PayloadDraft::Resource {
            schema_identity,
            selectable_paths,
            value,
        } => (
            RecordClass::Resource,
            canonical_json_bytes(value)?,
            Some(schema_identity.clone()),
            None,
            selectable_paths.clone(),
        ),
        PayloadDraft::File { media_class, bytes } => {
            (RecordClass::File, bytes.clone(), None, Some(media_class.clone()), Vec::new())
        }
    };
    selectable_paths.sort();
    selectable_paths.dedup();
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(Error::Limit("payload bytes"));
    }
    if contains_prohibited_bytes(&bytes, policy.prohibited_fragments) {
        return Err(Error::Secret);
    }
    let content_identity = crate::operation::identity::hash_bytes(CONTENT_DOMAIN, &bytes);
    let content_suffix =
        content_identity.strip_prefix(crate::operation::model::BLAKE3_PREFIX).ok_or(Error::Encoding)?;
    let safe_locator = format!("{LOCATOR_PREFIX}{content_suffix}{LOCATOR_SUFFIX}");
    let safe_locator_identity = crate::operation::identity::hash_bytes(LOCATOR_DOMAIN, safe_locator.as_bytes());
    let descriptor = PayloadDescriptor {
        content_identity,
        byte_length_bytes: u64::try_from(bytes.len()).map_err(|_| Error::Limit("payload bytes"))?,
        storage_backend: STORAGE_BACKEND.to_string(),
        safe_locator,
        safe_locator_identity,
        schema_identity,
        media_class,
        selectable_paths,
    };
    Ok(AdmittedPayload {
        record_class,
        descriptor,
        bytes,
    })
}

fn build_metadata(draft: &RecordDraft, admitted: &AdmittedPayload) -> Result<RecordMetadata, Error> {
    let payload = &admitted.descriptor;
    let record_identity = hash_value(RECORD_DOMAIN, &RecordIdentityMaterial {
        logical_key: &draft.logical_key,
        record_class: admitted.record_class,
        operation_type: &draft.producer.operation_type,
        type_revision: &draft.producer.type_revision,
        instance: &draft.producer.instance,
        method: &draft.producer.method,
        output_name: &draft.output_name,
        output_contract_identity: &draft.output_contract_identity,
        schema_identity: &payload.schema_identity,
        media_class: &payload.media_class,
        selectable_paths: &payload.selectable_paths,
    })?;
    let mut required_non_claims = draft.required_non_claims.clone();
    required_non_claims.sort();
    required_non_claims.dedup();
    let version_identity = hash_value(VERSION_DOMAIN, &VersionIdentityMaterial {
        record_identity: &record_identity,
        sequence: draft.sequence,
        producer: &draft.producer,
        tags: &draft.tags,
        sensitivity: draft.sensitivity,
        lifetime: &draft.lifetime,
        retention: &draft.retention,
        outcome: &draft.outcome,
        payload,
        required_non_claims: &required_non_claims,
    })?;
    Ok(RecordMetadata {
        schema: RECORD_SCHEMA.to_string(),
        record_identity,
        version_identity,
        record_class: admitted.record_class,
        logical_key: draft.logical_key.clone(),
        sequence: draft.sequence,
        producer: draft.producer.clone(),
        output_name: draft.output_name.clone(),
        output_contract_identity: draft.output_contract_identity.clone(),
        tags: draft.tags.clone(),
        sensitivity: draft.sensitivity,
        lifetime: draft.lifetime.clone(),
        retention: draft.retention.clone(),
        outcome: draft.outcome.clone(),
        payload: payload.clone(),
        required_non_claims,
    })
}

// r[impl lattice.operation_data.versions.conflict]
pub fn publication_decision(existing: Option<&RecordMetadata>, candidate: &RecordMetadata) -> PublicationDecision {
    match existing {
        None => PublicationDecision::Append,
        Some(current) if current == candidate => PublicationDecision::Idempotent,
        Some(_) => PublicationDecision::Conflict,
    }
}

pub fn canonical_json_bytes(value: &serde_json::Value) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(&canonicalize(value)).map_err(|_| Error::Encoding)
}

fn canonicalize(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(values) => serde_json::Value::Array(values.iter().map(canonicalize).collect()),
        serde_json::Value::Object(values) => {
            let ordered: std::collections::BTreeMap<String, serde_json::Value> =
                values.iter().map(|(key, value)| (key.clone(), canonicalize(value))).collect();
            serde_json::Value::Object(ordered.into_iter().collect())
        }
        scalar => scalar.clone(),
    }
}

include!("core/identity.rs");
include!("core/validation.rs");
