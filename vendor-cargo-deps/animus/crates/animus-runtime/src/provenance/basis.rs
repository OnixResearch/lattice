use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const PROVENANCE_ARTIFACT_SCHEMA: &str = "animus.provenance.artifact.v1";
const PROVENANCE_RECEIPT_SCHEMA: &str = "animus.provenance.receipt.v1";
const PROVENANCE_TOOL_EFFECT_SCHEMA: &str = "animus.provenance.tool-effect.v1";
const TOOLPATH_PROJECTION_SCHEMA: &str = "animus.provenance.toolpath-projection.v1";
const PROVENANCE_ARTIFACT_SCHEMA_VERSION: u16 = 1;
const HASH_HEX_LENGTH: usize = 64;
const MAX_SAFE_FIELD_CHARS: usize = 160;
const MAX_RECEIPT_COUNT: u64 = 1_000_000;
const HASH_PART_SEPARATOR: char = '|';
const HASH_LENGTH_SEPARATOR: char = ':';
const PUBLIC_REDACTION_MARKER: &str = "public";
const METADATA_ONLY_REDACTION_MARKER: &str = "metadata-only";
const REDACTED_PAYLOAD_MARKER: &str = "redacted-payload";
const SECRET_REDACTION_MARKER: &str = "secret";
const TOOLPATH_REDACTED_PROMPT_MARKER: &str = "redacted:prompt-or-output";
const TOOLPATH_REDACTED_ARTIFACT_MARKER: &str = "redacted:artifact-payload";
const UNKNOWN_SAFE_FIELD: &str = "unknown";
const COMPLETED_STATUS: &str = "completed";
const TRUNCATED_STATUS: &str = "truncated";
const UNAVAILABLE_STATUS: &str = "unavailable";
const DENIED_STATUS: &str = "denied";
const CANCELLED_STATUS: &str = "cancelled";
const EMPTY_OPTION_MARKER: &str = "none";

const UNSAFE_MARKERS: &[UnsafeMarker] = &[
    UnsafeMarker {
        needle: "raw_prompt:",
        class: "raw_prompt",
    },
    UnsafeMarker {
        needle: "provider_json",
        class: "provider_payload",
    },
    UnsafeMarker {
        needle: "provider payload",
        class: "provider_payload",
    },
    UnsafeMarker {
        needle: "provider_payload:",
        class: "provider_payload",
    },
    UnsafeMarker {
        needle: "tool_input:",
        class: "raw_tool_input",
    },
    UnsafeMarker {
        needle: "tool_output:",
        class: "raw_tool_output",
    },
    UnsafeMarker {
        needle: "model_output:",
        class: "raw_model_output",
    },
    UnsafeMarker {
        needle: "unbounded_output:",
        class: "unbounded_output",
    },
    UnsafeMarker {
        needle: "unified diff",
        class: "raw_diff",
    },
    UnsafeMarker {
        needle: "authorization:",
        class: "credential",
    },
    UnsafeMarker {
        needle: "api_key",
        class: "credential",
    },
    UnsafeMarker {
        needle: "secret=",
        class: "credential",
    },
    UnsafeMarker {
        needle: "sk-",
        class: "credential",
    },
    UnsafeMarker {
        needle: "/home/",
        class: "full_path",
    },
];

type Text = String;
type Collection<T> = Vec<T>;

#[derive(Clone, Copy)]
struct UnsafeMarker {
    needle: &'static str,
    class: &'static str,
}

/// Provenance validation error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceErrorKind {
    /// A required identifier or label was empty.
    EmptyField,
    /// A safe metadata field exceeded the bounded size policy.
    FieldTooLong,
    /// A metadata field contained raw or unsafe material.
    UnsafeMetadata,
    /// A hash field was not a BLAKE3 hexadecimal digest.
    InvalidHash,
    /// Replay encountered an adapter-owned or unsupported entry.
    UnsupportedEntry,
    /// A graph entry id appeared more than once.
    DuplicateEntry,
    /// A graph parent reference did not resolve.
    MissingParent,
    /// Graph ancestry contained a cycle.
    Cycle,
    /// Tool or effect evidence was requested before a completed outcome existed.
    IncompleteEffect,
    /// A requested compatibility shape is not available in the default policy.
    UnsupportedInterchange,
    /// Line range or hunk data was malformed.
    InvalidRange,
    /// Line ranges or hunks were not sorted as required.
    UnsortedRange,
    /// Line arithmetic overflowed or underflowed.
    ArithmeticOverflow,
    /// Required provenance backend was not injected or available.
    UnavailableBackend,
    /// Injected provenance backend does not support the requested operation.
    UnsupportedOperation,
    /// Mutating-tool checkpoint policy was malformed.
    InvalidPolicy,
    /// Git-specific terminology escaped into reusable core DTO fields.
    GitTermInCoreDto,
    /// Optional Git AI compatibility data was malformed.
    InvalidCompatibility,
}

/// Typed provenance validation error with safe diagnostics only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceError {
    kind: ProvenanceErrorKind,
    field: &'static str,
    subject: Text,
    reason: Text,
}

impl ProvenanceError {
    fn new(
        kind: ProvenanceErrorKind,
        field: &'static str,
        subject: impl Into<Text>,
        reason: impl Into<Text>,
    ) -> Self {
        Self {
            kind,
            field,
            subject: subject.into(),
            reason: reason.into(),
        }
    }

    /// Returns the stable error class.
    #[must_use]
    pub const fn kind(&self) -> ProvenanceErrorKind {
        self.kind
    }

    /// Returns the field that failed validation.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a safe subject label or hash.
    #[must_use]
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Returns a safe diagnostic reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "provenance field `{}` failed ({:?}) for `{}`: {}",
            self.field, self.kind, self.subject, self.reason
        )
    }
}

impl std::error::Error for ProvenanceError {}
