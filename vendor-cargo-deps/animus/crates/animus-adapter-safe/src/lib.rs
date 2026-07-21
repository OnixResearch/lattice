//! Independent no-std safe receipt contract family for Animus adapter boundaries.
//!
//! This crate contains implementation-free DTOs, validation helpers, leak
//! classes, bounded counts, and BLAKE3 artifact identities for safe receipts.
//! Hosts still own storage, clocks, filesystems, processes, networks,
//! credentials, terminal state, and receipt persistence.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::ToString;

const FIELD_ARTIFACT_HASH: &str = "artifact_hash";
const HASH_LENGTH_REASON: &str = "artifact hash must use lowercase BLAKE3 hex";
const BLAKE3_HASH_BYTES: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const HEX_RADIX: usize = 16;
const HEX_NIBBLE_SHIFT: u8 = 4;
const HEX_LOW_NIBBLE_MASK: u8 = 0x0f;
const LOWER_HEX: &[u8; HEX_RADIX] = b"0123456789abcdef";
const REVIEW_PREFIX: &str = "safe_receipt";
const REVIEW_SEPARATOR: char = ' ';
const MAX_COLLECTION_ITEMS: usize = 32;
const NUL_CHARACTER: char = '\0';
const TOKEN_PREFIX: &str = "sk-";
const GITHUB_TOKEN_PREFIX: &str = "ghp_";
const TOKEN_ASSIGNMENT_MARKER: &str = "token=";
const API_KEY_MARKER: &str = "api_key";
const API_KEY_DASH_MARKER: &str = "api-key";
const BEARER_MARKER: &str = "bearer ";
const RAW_PROMPT_MARKER: &str = "prompt:";
const SYSTEM_PROMPT_MARKER: &str = "system prompt";
const USER_PROMPT_MARKER: &str = "user:";
const PROVIDER_CHOICES_MARKER: &str = "\"choices\"";
const PROVIDER_MESSAGES_MARKER: &str = "\"messages\"";
const PROVIDER_MODEL_MARKER: &str = "\"model\"";
const TOOL_INPUT_MARKER: &str = "tool_input:";
const TOOL_INPUT_TEXT_MARKER: &str = "tool input:";
const TOOL_OUTPUT_MARKER: &str = "tool_output:";
const TOOL_OUTPUT_TEXT_MARKER: &str = "tool output:";
const MODEL_OUTPUT_MARKER: &str = "model_output:";
const MODEL_OUTPUT_TEXT_MARKER: &str = "model output:";
const ASSISTANT_OUTPUT_MARKER: &str = "assistant:";
const ROOT_PATH_MARKER: &str = "/";
const HOME_PATH_MARKER: &str = "/home/";
const TMP_PATH_MARKER: &str = "/tmp/";
const ETC_PATH_MARKER: &str = "/etc/";
const USER_PATH_MARKER: &str = "\\users\\";
const JSON_OBJECT_PREFIX: &str = "{";
const LABELS_FIELD: &str = "receipt_labels";
const COUNTS_FIELD: &str = "receipt_counts";
const HASHES_FIELD: &str = "receipt_artifact_hashes";
const REDACTIONS_FIELD: &str = "receipt_redactions";
const BACKEND_LABEL_FIELD: MetadataField = MetadataField::new("receipt_backend_label");
const INTERCHANGE_LABEL_FIELD: MetadataField = MetadataField::new("receipt_interchange_label");
const LABEL_FIELD: MetadataField = MetadataField::new("receipt_label");
const COUNT_LABEL_FIELD: MetadataField = MetadataField::new("receipt_count_label");

const PROMPT_MARKERS: &[&str] = &[RAW_PROMPT_MARKER, SYSTEM_PROMPT_MARKER, USER_PROMPT_MARKER];
const PROVIDER_MARKERS: &[&str] = &[
    PROVIDER_CHOICES_MARKER,
    PROVIDER_MESSAGES_MARKER,
    PROVIDER_MODEL_MARKER,
];
const CREDENTIAL_MARKERS: &[&str] = &[
    TOKEN_PREFIX,
    GITHUB_TOKEN_PREFIX,
    TOKEN_ASSIGNMENT_MARKER,
    API_KEY_MARKER,
    API_KEY_DASH_MARKER,
    BEARER_MARKER,
];
const PATH_MARKERS: &[&str] = &[
    HOME_PATH_MARKER,
    TMP_PATH_MARKER,
    ETC_PATH_MARKER,
    USER_PATH_MARKER,
];
const TOOL_INPUT_MARKERS: &[&str] = &[TOOL_INPUT_MARKER, TOOL_INPUT_TEXT_MARKER];
const TOOL_OUTPUT_MARKERS: &[&str] = &[TOOL_OUTPUT_MARKER, TOOL_OUTPUT_TEXT_MARKER];
const MODEL_OUTPUT_MARKERS: &[&str] = &[
    MODEL_OUTPUT_MARKER,
    MODEL_OUTPUT_TEXT_MARKER,
    ASSISTANT_OUTPUT_MARKER,
];

/// Schema identifier attached to safe receipt DTOs.
pub const SCHEMA: &str = "animus.safe-receipt.v1";
/// Largest accepted safe metadata label length.
pub const MAX_LABEL_CHARS: usize = 256;
/// Largest accepted bounded count value in a safe receipt.
pub const MAX_COUNT: u32 = 1_000_000;
/// Character length of a lowercase BLAKE3 artifact hash in hex form.
pub const BLAKE3_HASH_HEX_CHARS: usize = BLAKE3_HASH_BYTES.saturating_mul(HEX_CHARS_PER_BYTE);

/// Validated lowercase BLAKE3 identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactHash {
    value: alloc::string::String,
}

impl ArtifactHash {
    /// Builds a hash from validated lowercase BLAKE3 hex text.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when the value is not lowercase BLAKE3 hex.
    pub fn new(value: &str) -> Result<Self, Error> {
        validate_hash(value)?;
        Ok(Self {
            value: value.to_string(),
        })
    }

    /// Builds a hash from raw BLAKE3 bytes.
    ///
    /// # Panics
    ///
    /// Panics if the fixed-size BLAKE3 byte-to-hex encoder violates its nibble
    /// table or output-length invariants.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; BLAKE3_HASH_BYTES]) -> Self {
        let mut value = alloc::string::String::with_capacity(BLAKE3_HASH_HEX_CHARS);
        for byte in bytes {
            let high = usize::from(byte >> HEX_NIBBLE_SHIFT);
            let low = usize::from(byte & HEX_LOW_NIBBLE_MASK);
            assert!(high < HEX_RADIX, "high hex nibble must fit hex table");
            assert!(low < HEX_RADIX, "low hex nibble must fit hex table");
            value.push(char::from(LOWER_HEX[high]));
            value.push(char::from(LOWER_HEX[low]));
        }
        assert_eq!(
            value.len(),
            BLAKE3_HASH_HEX_CHARS,
            "BLAKE3 hex encoding must produce the configured length"
        );
        Self { value }
    }

    /// Returns the validated hash text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl core::fmt::Display for ArtifactHash {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl core::str::FromStr for ArtifactHash {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Status class recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Status {
    /// The observed operation completed.
    Completed,
    /// The observed operation was blocked before unsafe work happened.
    Blocked,
    /// The observed operation was denied by policy or authorization.
    Denied,
    /// The observed operation failed at a typed adapter boundary.
    Failed,
    /// The observed operation was accepted only after redaction.
    Redacted,
}

impl Status {
    /// Returns a stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Blocked => "blocked",
            Self::Denied => "denied",
            Self::Failed => "failed",
            Self::Redacted => "redacted",
        }
    }
}

/// Issue code recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum IssueCode {
    /// No issue was observed.
    None,
    /// Model dispatch evidence.
    ModelDispatch,
    /// Tool execution evidence.
    ToolExecution,
    /// Authorization denial evidence.
    AuthorizationDenied,
    /// Runtime block evidence.
    RuntimeBlocked,
    /// Final response evidence.
    FinalResponse,
    /// Adapter error evidence.
    AdapterError,
    /// Provenance observation evidence.
    ProvenanceObservation,
    /// Unsafe material was rejected or redacted.
    LeakRejected,
}

impl IssueCode {
    /// Returns a stable issue-code label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::AuthorizationDenied => "authorization_denied",
            Self::RuntimeBlocked => "runtime_blocked",
            Self::FinalResponse => "final_response",
            Self::AdapterError => "adapter_error",
            Self::ProvenanceObservation => "provenance_observation",
            Self::LeakRejected => "leak_rejected",
        }
    }
}

/// Effect or mutation class observed by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Operation {
    /// Model or provider dispatch.
    ModelDispatch,
    /// Tool execution.
    ToolExecution,
    /// Authorization or capability decision.
    AuthorizationDecision,
    /// Runtime policy block.
    RuntimeBlock,
    /// Final response emission.
    FinalResponse,
    /// Adapter boundary failure.
    AdapterBoundary,
    /// Provenance observation.
    ProvenanceObservation,
    /// Host-owned mutation.
    Mutation,
}

impl Operation {
    /// Returns a stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ModelDispatch => "model_dispatch",
            Self::ToolExecution => "tool_execution",
            Self::AuthorizationDecision => "authorization_decision",
            Self::RuntimeBlock => "runtime_block",
            Self::FinalResponse => "final_response",
            Self::AdapterBoundary => "adapter_boundary",
            Self::ProvenanceObservation => "provenance_observation",
            Self::Mutation => "mutation",
        }
    }
}

/// Redaction marker recorded by a safe receipt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Redaction {
    /// Raw prompt text was omitted.
    PromptOmitted,
    /// Provider JSON or response bodies were omitted.
    ProviderPayloadOmitted,
    /// Credential-looking values were omitted.
    CredentialOmitted,
    /// Absolute paths were omitted or reduced to safe labels.
    AbsolutePathOmitted,
    /// Raw tool input was omitted.
    ToolInputOmitted,
    /// Raw tool output was omitted.
    ToolOutputOmitted,
    /// Raw model output was omitted.
    ModelOutputOmitted,
    /// Output was bounded to counts, labels, or hashes.
    OutputBounded,
    /// Payload identity was represented by BLAKE3 hash only.
    PayloadHashOnly,
}

impl Redaction {
    /// Returns a stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PromptOmitted => "prompt_omitted",
            Self::ProviderPayloadOmitted => "provider_payload_omitted",
            Self::CredentialOmitted => "credential_omitted",
            Self::AbsolutePathOmitted => "absolute_path_omitted",
            Self::ToolInputOmitted => "tool_input_omitted",
            Self::ToolOutputOmitted => "tool_output_omitted",
            Self::ModelOutputOmitted => "model_output_omitted",
            Self::OutputBounded => "output_bounded",
            Self::PayloadHashOnly => "payload_hash_only",
        }
    }
}

/// Unsafe material class rejected by safe receipt validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeakClass {
    /// Raw prompt text was detected.
    RawPrompt,
    /// Provider JSON was detected.
    ProviderJson,
    /// Credential-looking text was detected.
    Credential,
    /// Absolute path text was detected.
    AbsolutePath,
    /// Raw tool input was detected.
    RawToolInput,
    /// Raw tool output was detected.
    RawToolOutput,
    /// Raw model output was detected.
    RawModelOutput,
    /// Unbounded or oversized output was detected.
    UnboundedOutput,
}

impl LeakClass {
    /// Returns a stable leak-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::ProviderJson => "provider_json",
            Self::Credential => "credential",
            Self::AbsolutePath => "absolute_path",
            Self::RawToolInput => "raw_tool_input",
            Self::RawToolOutput => "raw_tool_output",
            Self::RawModelOutput => "raw_model_output",
            Self::UnboundedOutput => "unbounded_output",
        }
    }
}

/// Errors raised by safe receipt validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary collection field name.
        field: &'static str,
    },
    /// A collection exceeded the safe receipt item limit.
    CollectionTooLarge {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A safe metadata field was empty after trimming.
    EmptyField {
        /// Boundary field name.
        field: &'static str,
    },
    /// A safe metadata field contained a NUL character.
    NulCharacter {
        /// Boundary field name.
        field: &'static str,
        /// Byte index where the NUL character starts.
        byte_index: usize,
    },
    /// Unsafe material was detected in safe metadata.
    UnsafeMetadata {
        /// Boundary field name.
        field: &'static str,
        /// Detected unsafe material class.
        class: LeakClass,
    },
    /// A bounded count exceeded [`MAX_COUNT`].
    CountTooLarge {
        /// Boundary field name.
        field: &'static str,
        /// Maximum accepted count.
        max: u32,
        /// Actual count.
        actual: u32,
    },
    /// A BLAKE3 artifact hash was malformed.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
        /// Validation reason.
        reason: &'static str,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyCollection { field } | Self::EmptyField { field } => {
                write!(formatter, "{field} must not be empty")
            }
            Self::CollectionTooLarge {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} must contain at most {max_items} items; got {actual_items}"
            ),
            Self::NulCharacter { field, byte_index } => write!(
                formatter,
                "{field} contains a NUL character at byte index {byte_index}"
            ),
            Self::UnsafeMetadata { field, class } => {
                write!(
                    formatter,
                    "{field} contains unsafe {} material",
                    class.as_str()
                )
            }
            Self::CountTooLarge { field, max, actual } => {
                write!(formatter, "{field} must be at most {max}; got {actual}")
            }
            Self::InvalidHash { field, reason } => write!(formatter, "{field}: {reason}"),
        }
    }
}

/// Borrowed bounded-count input for a safe receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedCountDraft<'a> {
    label: &'a str,
    value: u32,
}

impl<'a> BoundedCountDraft<'a> {
    /// Builds a bounded-count draft.
    #[must_use]
    pub const fn new(label: &'a str, value: u32) -> Self {
        Self { label, value }
    }
}

/// Bounded count retained by a safe receipt.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BoundedCount {
    label: alloc::string::String,
    value: u32,
}

impl BoundedCount {
    fn new(label: alloc::string::String, value: u32) -> Self {
        Self { label, value }
    }

    /// Returns the safe count label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the bounded count value.
    #[must_use]
    pub const fn value(&self) -> u32 {
        self.value
    }
}

/// Borrowed inputs used to construct a safe receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    status: Status,
    issue_code: IssueCode,
    operation: Operation,
    labels: &'a [&'a str],
    counts: &'a [BoundedCountDraft<'a>],
    artifact_hashes: &'a [ArtifactHash],
    backend_label: Option<&'a str>,
    interchange_label: Option<&'a str>,
    redactions: &'a [Redaction],
}

impl<'a> Draft<'a> {
    /// Builds a draft with required status, issue, and operation classes.
    #[must_use]
    pub const fn new(status: Status, issue_code: IssueCode, operation: Operation) -> Self {
        Self {
            status,
            issue_code,
            operation,
            labels: &[],
            counts: &[],
            artifact_hashes: &[],
            backend_label: None,
            interchange_label: None,
            redactions: &[],
        }
    }

    /// Adds safe labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: &'a [&'a str]) -> Self {
        self.labels = labels;
        self
    }

    /// Adds bounded counts.
    #[must_use]
    pub const fn with_counts(mut self, counts: &'a [BoundedCountDraft<'a>]) -> Self {
        self.counts = counts;
        self
    }

    /// Adds artifact identity hashes.
    #[must_use]
    pub const fn with_artifact_hashes(mut self, hashes: &'a [ArtifactHash]) -> Self {
        self.artifact_hashes = hashes;
        self
    }

    /// Adds a backend label such as a provider class or local runtime class.
    #[must_use]
    pub const fn with_backend_label(mut self, backend_label: &'a str) -> Self {
        self.backend_label = Some(backend_label);
        self
    }

    /// Adds an interchange label such as `preserves` or `jsonl`.
    #[must_use]
    pub const fn with_interchange_label(mut self, interchange_label: &'a str) -> Self {
        self.interchange_label = Some(interchange_label);
        self
    }

    /// Adds redaction markers.
    #[must_use]
    pub const fn with_redactions(mut self, redactions: &'a [Redaction]) -> Self {
        self.redactions = redactions;
        self
    }
}

/// Safe runtime/provenance receipt containing metadata, counts, hashes, and redactions only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    schema: &'static str,
    status: Status,
    issue_code: IssueCode,
    operation: Operation,
    labels: alloc::vec::Vec<alloc::string::String>,
    counts: alloc::vec::Vec<BoundedCount>,
    artifact_hashes: alloc::vec::Vec<ArtifactHash>,
    backend_label: Option<alloc::string::String>,
    interchange_label: Option<alloc::string::String>,
    redactions: alloc::vec::Vec<Redaction>,
}

impl Record {
    /// Builds a safe receipt after validating all metadata fields for leaks.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] when metadata is oversized or unsafe.
    pub fn new(draft: Draft<'_>) -> Result<Self, Error> {
        Ok(Self {
            schema: SCHEMA,
            status: draft.status,
            issue_code: draft.issue_code,
            operation: draft.operation,
            labels: labels(draft.labels)?,
            counts: counts(draft.counts)?,
            artifact_hashes: hashes(draft.artifact_hashes)?,
            backend_label: optional_label(BACKEND_LABEL_FIELD, draft.backend_label)?,
            interchange_label: optional_label(INTERCHANGE_LABEL_FIELD, draft.interchange_label)?,
            redactions: redactions(draft.redactions)?,
        })
    }

    /// Returns the receipt schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> Status {
        self.status
    }

    /// Returns the receipt issue code.
    #[must_use]
    pub const fn issue_code(&self) -> IssueCode {
        self.issue_code
    }

    /// Returns the effect or mutation operation class.
    #[must_use]
    pub const fn operation(&self) -> Operation {
        self.operation
    }

    /// Returns sorted and deduplicated safe labels.
    #[must_use]
    pub fn labels(&self) -> &[alloc::string::String] {
        self.labels.as_slice()
    }

    /// Returns sorted and deduplicated bounded counts.
    #[must_use]
    pub fn counts(&self) -> &[BoundedCount] {
        self.counts.as_slice()
    }

    /// Returns sorted and deduplicated artifact hashes.
    #[must_use]
    pub fn artifact_hashes(&self) -> &[ArtifactHash] {
        self.artifact_hashes.as_slice()
    }

    /// Returns the optional backend label.
    #[must_use]
    pub fn backend_label(&self) -> Option<&str> {
        self.backend_label.as_deref()
    }

    /// Returns the optional interchange label.
    #[must_use]
    pub fn interchange_label(&self) -> Option<&str> {
        self.interchange_label.as_deref()
    }

    /// Returns sorted and deduplicated redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[Redaction] {
        self.redactions.as_slice()
    }

    /// Renders bounded review output without raw payload fields.
    #[must_use]
    pub fn review_summary(&self) -> alloc::string::String {
        let mut text = alloc::string::String::new();
        push_review_value(&mut text, REVIEW_PREFIX);
        push_review_value(&mut text, self.schema);
        push_review_value(&mut text, self.status.as_str());
        push_review_value(&mut text, self.issue_code.as_str());
        push_review_value(&mut text, self.operation.as_str());
        for label in &self.labels {
            push_review_value(&mut text, label);
        }
        for hash in &self.artifact_hashes {
            push_review_value(&mut text, hash.as_str());
        }
        for redaction in &self.redactions {
            push_review_value(&mut text, redaction.as_str());
        }
        text
    }
}

fn labels(values: &[&str]) -> Result<alloc::vec::Vec<alloc::string::String>, Error> {
    ensure_required_collection(LABELS_FIELD, values.len())?;
    let mut labels = values
        .iter()
        .map(|value| required_label(LABEL_FIELD, value))
        .collect::<Result<alloc::vec::Vec<_>, _>>()?;
    labels.sort();
    labels.dedup();
    Ok(labels)
}

fn counts(values: &[BoundedCountDraft<'_>]) -> Result<alloc::vec::Vec<BoundedCount>, Error> {
    ensure_optional_collection(COUNTS_FIELD, values.len())?;
    let mut counts = values
        .iter()
        .map(count)
        .collect::<Result<alloc::vec::Vec<_>, _>>()?;
    counts.sort();
    counts.dedup();
    Ok(counts)
}

fn hashes(values: &[ArtifactHash]) -> Result<alloc::vec::Vec<ArtifactHash>, Error> {
    ensure_optional_collection(HASHES_FIELD, values.len())?;
    let mut hashes = values.to_vec();
    hashes.sort();
    hashes.dedup();
    Ok(hashes)
}

fn redactions(values: &[Redaction]) -> Result<alloc::vec::Vec<Redaction>, Error> {
    ensure_required_collection(REDACTIONS_FIELD, values.len())?;
    let mut redactions = values.to_vec();
    redactions.sort();
    redactions.dedup();
    Ok(redactions)
}

fn count(value: &BoundedCountDraft<'_>) -> Result<BoundedCount, Error> {
    let label = required_label(COUNT_LABEL_FIELD, value.label)?;
    if value.value > MAX_COUNT {
        return Err(Error::CountTooLarge {
            field: COUNTS_FIELD,
            max: MAX_COUNT,
            actual: value.value,
        });
    }
    Ok(BoundedCount::new(label, value.value))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MetadataField {
    name: &'static str,
}

impl MetadataField {
    const fn new(name: &'static str) -> Self {
        Self { name }
    }

    const fn name(self) -> &'static str {
        self.name
    }
}

fn ensure_required_collection(field: &'static str, actual_items: usize) -> Result<(), Error> {
    ensure_optional_collection(field, actual_items)?;
    if actual_items == 0 {
        return Err(Error::EmptyCollection { field });
    }
    Ok(())
}

fn ensure_optional_collection(field: &'static str, actual_items: usize) -> Result<(), Error> {
    if actual_items > MAX_COLLECTION_ITEMS {
        return Err(Error::CollectionTooLarge {
            field,
            max_items: MAX_COLLECTION_ITEMS,
            actual_items,
        });
    }
    Ok(())
}

fn optional_label(
    field: MetadataField,
    value: Option<&str>,
) -> Result<Option<alloc::string::String>, Error> {
    value.map(|label| required_label(field, label)).transpose()
}

fn required_label(field: MetadataField, value: &str) -> Result<alloc::string::String, Error> {
    assert!(
        !field.name().is_empty(),
        "metadata field name must not be empty"
    );
    assert!(
        value.len() >= value.chars().count(),
        "label byte length must cover character count"
    );
    reject_oversized(field, value)?;
    reject_nul(field, value)?;
    let normalized = value.trim();
    reject_empty(field, normalized)?;
    reject_unsafe(field, normalized)?;
    Ok(normalized.to_string())
}

fn reject_oversized(field: MetadataField, value: &str) -> Result<(), Error> {
    let actual_chars = value.chars().count();
    if actual_chars > MAX_LABEL_CHARS {
        return Err(Error::UnsafeMetadata {
            field: field.name(),
            class: LeakClass::UnboundedOutput,
        });
    }
    Ok(())
}

fn reject_nul(field: MetadataField, value: &str) -> Result<(), Error> {
    if let Some((byte_index, _character)) = value
        .char_indices()
        .find(|(_byte_index, character)| *character == NUL_CHARACTER)
    {
        return Err(Error::NulCharacter {
            field: field.name(),
            byte_index,
        });
    }
    Ok(())
}

fn reject_empty(field: MetadataField, value: &str) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::EmptyField {
            field: field.name(),
        });
    }
    Ok(())
}

fn reject_unsafe(field: MetadataField, value: &str) -> Result<(), Error> {
    if let Some(class) = leak_class(value) {
        return Err(Error::UnsafeMetadata {
            field: field.name(),
            class,
        });
    }
    Ok(())
}

fn leak_class(value: &str) -> Option<LeakClass> {
    assert!(!value.is_empty(), "safe metadata value must not be empty");
    assert!(
        value.len() >= value.chars().count(),
        "byte length covers chars"
    );
    let lower = value.to_ascii_lowercase();
    let trimmed = lower.trim();
    if contains_any(trimmed, PROMPT_MARKERS) {
        return Some(LeakClass::RawPrompt);
    }
    if is_provider_json(trimmed) {
        return Some(LeakClass::ProviderJson);
    }
    leak_class_after_provider(trimmed)
}

fn leak_class_after_provider(value: &str) -> Option<LeakClass> {
    if contains_any(value, CREDENTIAL_MARKERS) {
        return Some(LeakClass::Credential);
    }
    if is_path_like(value) {
        return Some(LeakClass::AbsolutePath);
    }
    leak_class_after_path(value)
}

fn leak_class_after_path(value: &str) -> Option<LeakClass> {
    if contains_any(value, TOOL_INPUT_MARKERS) {
        return Some(LeakClass::RawToolInput);
    }
    if contains_any(value, TOOL_OUTPUT_MARKERS) {
        return Some(LeakClass::RawToolOutput);
    }
    if contains_any(value, MODEL_OUTPUT_MARKERS) {
        return Some(LeakClass::RawModelOutput);
    }
    None
}

fn is_provider_json(value: &str) -> bool {
    if !value.starts_with(JSON_OBJECT_PREFIX) {
        return false;
    }
    contains_any(value, PROVIDER_MARKERS)
}

fn is_path_like(value: &str) -> bool {
    if value.starts_with(ROOT_PATH_MARKER) {
        return true;
    }
    contains_any(value, PATH_MARKERS)
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    assert!(!markers.is_empty(), "marker list must not be empty");
    assert!(
        value.len() >= value.chars().count(),
        "byte length covers chars"
    );
    markers.iter().any(|marker| value.contains(marker))
}

fn validate_hash(value: &str) -> Result<(), Error> {
    let actual_chars = value.chars().count();
    assert!(
        actual_chars <= value.len(),
        "hash char count must fit byte length"
    );
    if actual_chars != BLAKE3_HASH_HEX_CHARS
        || !value
            .chars()
            .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    {
        return Err(Error::InvalidHash {
            field: FIELD_ARTIFACT_HASH,
            reason: HASH_LENGTH_REASON,
        });
    }
    Ok(())
}

fn push_review_value(text: &mut alloc::string::String, value: &str) {
    if !text.is_empty() {
        text.push(REVIEW_SEPARATOR);
    }
    text.push_str(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_TEXT: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const SAFE_LABEL: &str = "runtime_started";
    const SAFE_BACKEND: &str = "local";
    const COUNT_LABEL: &str = "events";
    const COUNT_VALUE: u32 = 1;
    const UNSAFE_CREDENTIAL: &str = "token=secret";
    const PRIVATE_PATH: &str = "/home/example/private.txt";

    #[test]
    fn positive_safe_receipt_accepts_bounded_metadata() -> Result<(), Error> {
        let hash = ArtifactHash::new(HASH_TEXT)?;
        let counts = [BoundedCountDraft::new(COUNT_LABEL, COUNT_VALUE)];
        let receipt = Record::new(
            Draft::new(
                Status::Completed,
                IssueCode::None,
                Operation::ProvenanceObservation,
            )
            .with_labels(&[SAFE_LABEL])
            .with_counts(&counts)
            .with_artifact_hashes(&[hash])
            .with_backend_label(SAFE_BACKEND)
            .with_redactions(&[Redaction::PayloadHashOnly]),
        )?;
        assert_eq!(receipt.schema(), SCHEMA);
        assert_eq!(receipt.counts()[0].value(), COUNT_VALUE);
        assert!(receipt.review_summary().contains(SAFE_LABEL));
        Ok(())
    }

    #[test]
    fn negative_safe_receipt_rejects_leaks_and_bad_hashes() {
        assert!(matches!(
            Record::new(
                Draft::new(
                    Status::Blocked,
                    IssueCode::LeakRejected,
                    Operation::RuntimeBlock
                )
                .with_labels(&[UNSAFE_CREDENTIAL])
                .with_redactions(&[Redaction::CredentialOmitted]),
            ),
            Err(Error::UnsafeMetadata {
                class: LeakClass::Credential,
                ..
            })
        ));
        assert!(matches!(
            Record::new(
                Draft::new(
                    Status::Blocked,
                    IssueCode::LeakRejected,
                    Operation::RuntimeBlock
                )
                .with_labels(&[PRIVATE_PATH])
                .with_redactions(&[Redaction::AbsolutePathOmitted]),
            ),
            Err(Error::UnsafeMetadata {
                class: LeakClass::AbsolutePath,
                ..
            })
        ));
        assert!(matches!(
            ArtifactHash::new("not-a-blake3-hex"),
            Err(Error::InvalidHash { .. })
        ));
    }
}
