// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
/// BLAKE3 fingerprint for quality record content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityFingerprint {
    /// Hash algorithm name.
    pub algorithm: String,
    /// Lowercase hexadecimal digest.
    pub value_hex: String,
}

impl QualityFingerprint {
    /// Builds a quality fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Record kind for Animus-owned quality envelopes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityRecordKind {
    /// Review annotation or concern.
    Annotation,
    /// Explicit resolution record.
    Resolve,
    /// Explicit waiver record.
    Waiver,
}

impl QualityRecordKind {
    /// Stable record-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Annotation => "annotation",
            Self::Resolve => "resolve",
            Self::Waiver => "waiver",
        }
    }
}

/// Optional issuer classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityIssuerClassification {
    /// Human issuer.
    Human,
    /// Agent issuer.
    Agent,
    /// Continuous-integration issuer.
    Ci,
    /// Unknown issuer class.
    Unknown,
}

impl QualityIssuerClassification {
    /// Stable issuer classification label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Agent => "agent",
            Self::Ci => "ci",
            Self::Unknown => "unknown",
        }
    }
}

/// VCS-neutral one-indexed source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualitySpan {
    /// One-indexed starting line.
    pub start_line: u32,
    /// One-indexed ending line.
    pub end_line: u32,
    /// One-indexed starting column.
    pub start_column: u32,
    /// One-indexed ending column.
    pub end_column: u32,
    /// Optional BLAKE3 hash of full covered lines.
    pub content_hash: Option<QualityFingerprint>,
}

/// Safe metadata key/value field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityMetadataField {
    /// Safe metadata key.
    pub key: String,
    /// Safe metadata value.
    pub value: String,
}

/// Annotation-like body shared by annotation, resolve, and waiver records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAnnotationBody {
    /// Kind label, such as concern, suggestion, resolve, or waiver.
    pub kind: String,
    /// Bounded safe summary.
    pub summary: String,
    /// Optional source span.
    pub span: Option<QualitySpan>,
    /// Optional bounded suggested fix.
    pub suggested_fix: Option<String>,
    /// Threaded reference record IDs.
    pub references: Vec<String>,
    /// Superseded record IDs.
    pub supersedes: Vec<String>,
    /// Safe tags.
    pub tags: Vec<String>,
    /// Safe structured metadata.
    pub metadata: Vec<QualityMetadataField>,
}

/// Typed quality record body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityRecordBody {
    /// Annotation body.
    Annotation(QualityAnnotationBody),
    /// Resolve body.
    Resolve(QualityAnnotationBody),
    /// Waiver body.
    Waiver(QualityAnnotationBody),
}

impl QualityRecordBody {
    /// Returns the matching record kind.
    #[must_use]
    pub const fn kind(&self) -> QualityRecordKind {
        match self {
            Self::Annotation(_) => QualityRecordKind::Annotation,
            Self::Resolve(_) => QualityRecordKind::Resolve,
            Self::Waiver(_) => QualityRecordKind::Waiver,
        }
    }

    /// Returns the annotation-like fields.
    #[must_use]
    pub const fn annotation(&self) -> &QualityAnnotationBody {
        match self {
            Self::Annotation(body) | Self::Resolve(body) | Self::Waiver(body) => body,
        }
    }
}

/// Animus-owned Metabox-style quality record envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityRecordEnvelope {
    /// Envelope version.
    pub version: u32,
    /// Record type.
    pub record_type: QualityRecordKind,
    /// Stable subject label or URI.
    pub subject: String,
    /// Issuer URI.
    pub issuer: String,
    /// Optional issuer classification.
    pub issuer_classification: Option<QualityIssuerClassification>,
    /// Shell-supplied creation label.
    pub created_at_label: String,
    /// Content-addressed BLAKE3 record ID.
    pub id: QualityFingerprint,
    /// Typed body.
    pub body: QualityRecordBody,
}

/// Sidecar export projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualitySidecarExport {
    /// Deterministic record ID.
    pub id: QualityFingerprint,
    /// Safe subject.
    pub subject: String,
    /// Safe kind label.
    pub kind: String,
    /// Safe summary.
    pub summary: String,
    /// Whether sensitive content was omitted.
    pub redacted: bool,
    /// Safe references.
    pub references: Vec<String>,
    /// Safe superseded IDs.
    pub supersedes: Vec<String>,
    /// Safe tags.
    pub tags: Vec<String>,
}

/// Unknown sidecar passthrough handle that hides the raw body from core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityUnknownSidecarRecord {
    /// BLAKE3 hash of the raw unknown sidecar line.
    pub raw_hash: QualityFingerprint,
    /// Whether adapter can preserve the line during a rewrite.
    pub preserves_roundtrip: bool,
    /// Whether raw body is exposed to reusable core.
    pub exposes_raw_body_to_core: bool,
}

/// Dependency requirement state for default builds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityDependencyRequirement {
    /// Dependency is not required.
    NotRequired,
    /// Dependency is required.
    Required,
}

/// Dependency isolation report for default builds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityDependencyIsolationReport {
    /// Qualifier crate requirement.
    pub qualifier_crate: QualityDependencyRequirement,
    /// Sidecar discovery requirement.
    pub sidecar_discovery: QualityDependencyRequirement,
    /// Git diff library requirement.
    pub git_diff: QualityDependencyRequirement,
    /// Sidecar compaction code requirement.
    pub sidecar_compaction: QualityDependencyRequirement,
}

/// Active lifecycle report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityLifecycleReport {
    /// Active record IDs after supersession filtering.
    pub active_ids: Vec<String>,
    /// Records used only as thread context.
    pub referenced_context_ids: Vec<String>,
    /// Diagnostics found during lifecycle validation.
    pub diagnostics: Vec<QualityDiagnostic>,
}

/// Agent edit/drift review report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityEditReport {
    /// Record IDs surfaced for review.
    pub surfaced_record_ids: Vec<String>,
    /// Whether a resolve record was emitted automatically.
    pub emitted_resolve_record: bool,
}

/// Span freshness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityFreshness {
    /// Recorded hash matches current supplied lines.
    Fresh,
    /// Recorded hash differs from current supplied lines.
    Drifted,
    /// Span cannot be satisfied by supplied lines.
    Missing,
    /// No recorded hash was provided.
    NoHash,
}

/// Span freshness report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualitySpanFreshnessReport {
    /// Freshness state.
    pub state: QualityFreshness,
    /// Recorded hash if supplied.
    pub expected_hash: Option<QualityFingerprint>,
    /// Current computed hash if available.
    pub actual_hash: Option<QualityFingerprint>,
    /// Validation diagnostics.
    pub diagnostics: Vec<QualityDiagnostic>,
}

/// Agents-guidance page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAgentsGuidancePage {
    /// Topic name.
    pub topic: String,
    /// Markdown body.
    pub markdown: String,
    /// Available topic names.
    pub available_topics: Vec<String>,
    /// Whether guidance resolution uses network access.
    pub reads_network: bool,
}

/// Agents-guidance lookup error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityAgentsGuidanceError {
    /// Unknown topic.
    pub unknown_topic: String,
    /// Available topic names.
    pub available_topics: Vec<String>,
}

/// Quality diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QualityDiagnosticKind {
    /// Unsupported version.
    UnsupportedVersion,
    /// Missing or malformed field.
    InvalidField,
    /// ID is invalid or mismatched.
    InvalidId,
    /// Body contains unsafe material.
    UnsafeBody,
    /// Sidecar boundary rejected raw core exposure.
    SidecarBoundary,
    /// Lifecycle transition is invalid.
    InvalidLifecycle,
    /// Span is invalid.
    InvalidSpan,
}

/// Quality diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityDiagnostic {
    /// Diagnostic class.
    pub kind: QualityDiagnosticKind,
    /// Stable field label.
    pub field: String,
    /// Safe detail text.
    pub detail: String,
}

impl QualityDiagnostic {
    /// Builds a quality diagnostic.
    #[must_use]
    pub fn new(kind: QualityDiagnosticKind, field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Quality validation result.
pub type QualityValidationResult = Result<(), Vec<QualityDiagnostic>>;
