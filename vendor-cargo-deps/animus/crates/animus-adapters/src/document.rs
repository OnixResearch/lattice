//! Provider-neutral document-ingest adapter contracts.
//!
//! Hosts may read local files or run optional extractors before constructing these
//! DTOs. Reusable crates only validate already-loaded report metadata, bounded
//! summaries, hashes, diagnostics, redaction labels, and non-claims.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::borrowed_argument_types,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]

use crate::validation::{self, AdapterContractError, FieldName};
use alloc::string::String;
use alloc::vec::Vec;

const SOURCE_LABEL_FIELD: FieldName = FieldName::new("document_source_label");
const FORMAT_FIELD: FieldName = FieldName::new("document_format");
const EXTRACTOR_FIELD: FieldName = FieldName::new("document_extractor_family");
const SECTION_ID_FIELD: FieldName = FieldName::new("document_section_id");
const SECTION_HEADING_FIELD: FieldName = FieldName::new("document_section_heading");
const SECTION_SUMMARY_FIELD: FieldName = FieldName::new("document_section_summary");
const TABLE_ID_FIELD: FieldName = FieldName::new("document_table_id");
const TABLE_CELL_FIELD: FieldName = FieldName::new("document_table_cell_summary");
const METADATA_KEY_FIELD: FieldName = FieldName::new("document_metadata_key");
const METADATA_VALUE_FIELD: FieldName = FieldName::new("document_metadata_value");
const DIAGNOSTIC_FIELD: FieldName = FieldName::new("document_diagnostic");
const TIMEOUT_CLASS_FIELD: FieldName = FieldName::new("document_timeout_class");
const CONTEXT_LABEL_FIELD: FieldName = FieldName::new("document_context_label");
const RECEIPT_BACKEND_LABEL: &str = "document-ingest";
const RECEIPT_INTERCHANGE_LABEL: &str = "safe-dto";
const FIELD_INPUT_BYTES: &str = "input_bytes";
const FIELD_OUTPUT_BYTES: &str = "output_bytes";
const FIELD_DOCUMENT_COUNT: &str = "document_count";
const FIELD_SECTION_COUNT: &str = "section_count";
const FIELD_TABLE_CELL_COUNT: &str = "table_cell_count";
const FIELD_METADATA_COUNT: &str = "metadata_count";
const FIELD_DIAGNOSTIC_COUNT: &str = "diagnostic_count";
const FIELD_NON_CLAIM_COUNT: &str = "non_claim_count";
const FIELD_REDACTION_COUNT: &str = "redaction_count";
const UNSAFE_TRUST_LABEL: &str = "trusted";
const ABSOLUTE_UNIX_PREFIX: &str = "/";
const HOME_PATH_MARKER: &str = "/home/";
const TMP_PATH_MARKER: &str = "/tmp/";
const WINDOWS_USER_MARKER: &str = "\\users\\";
const RAW_CONTENT_MARKER: &str = "raw document";
const RAW_PATH_MARKER: &str = "raw path";
const PROMPT_MARKER: &str = "prompt:";
const TOKEN_MARKER: &str = "token=";
const SECRET_MARKER: &str = "secret";
const PASSWORD_MARKER: &str = "password=";

const ZERO_COUNT: u32 = 0;
const ZERO_ITEMS: usize = 0;
const MIN_CAP: u32 = 1;
const MAX_SECTIONS: usize = 32;
const MAX_TABLE_CELLS: usize = 64;
const MAX_METADATA_PAIRS: usize = 32;
const MAX_DIAGNOSTICS: usize = 16;
const MAX_REDACTIONS: usize = 8;
const REQUIRED_DOCUMENT_NON_CLAIMS: usize = 7;
const REQUIRED_DOCUMENT_REDACTIONS: usize = 2;
const MAX_SECTION_SUMMARY_CHARS: usize = 1_024;
const MAX_TABLE_CELL_CHARS: usize = 512;
const MAX_METADATA_VALUE_CHARS: usize = 512;
const MAX_DIAGNOSTIC_CHARS: usize = 256;

const UNSAFE_TEXT_MARKERS: &[&str] = &[
    HOME_PATH_MARKER,
    TMP_PATH_MARKER,
    WINDOWS_USER_MARKER,
    RAW_CONTENT_MARKER,
    RAW_PATH_MARKER,
    PROMPT_MARKER,
    TOKEN_MARKER,
    SECRET_MARKER,
    PASSWORD_MARKER,
];

const _: () = assert!(MIN_CAP > ZERO_COUNT);
const _: () = assert!(MAX_SECTIONS > ZERO_ITEMS);
const _: () = assert!(MAX_TABLE_CELLS > ZERO_ITEMS);
const _: () = assert!(MAX_METADATA_PAIRS > ZERO_ITEMS);
const _: () = assert!(MAX_DIAGNOSTICS > ZERO_ITEMS);
const _: () = assert!(MAX_REDACTIONS >= REQUIRED_DOCUMENT_REDACTIONS);
const _: () = assert!(REQUIRED_DOCUMENT_NON_CLAIMS > ZERO_ITEMS);
const _: () = assert!(MAX_SECTION_SUMMARY_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_TABLE_CELL_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_METADATA_VALUE_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);
const _: () = assert!(MAX_DIAGNOSTIC_CHARS >= validation::MIN_ADAPTER_FIELD_CHARS);

/// Product behaviors that document-ingest DTO validation explicitly does not perform.
pub const PROHIBITED_DOCUMENT_INGEST_EFFECTS: &[&str] = &[
    "filesystem reads",
    "parser execution",
    "process spawning",
    "environment inspection",
    "clock access",
    "network access",
    "OCR execution",
    "receipt persistence",
];

/// Stable admitted document format labels.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DocumentFormat {
    /// Plain UTF-8 text admitted by the host.
    PlainText,
    /// Markdown text admitted by the host.
    Markdown,
    /// PDF-derived extraction report admitted by the host.
    Pdf,
    /// HTML-derived extraction report admitted by the host.
    Html,
    /// CSV or table-like text admitted by the host.
    Csv,
}

impl DocumentFormat {
    /// Parses a supported admitted document format label.
    ///
    /// # Errors
    ///
    /// Returns [`DocumentIngestValidationIssue`] when the label is empty, unsafe,
    /// or unsupported.
    pub fn parse(value: &str) -> Result<Self, DocumentIngestValidationIssue> {
        let normalized = normalize_label(FORMAT_FIELD, value)?;
        match normalized.as_str() {
            "text" | "plain-text" => Ok(Self::PlainText),
            "markdown" => Ok(Self::Markdown),
            "pdf" => Ok(Self::Pdf),
            "html" => Ok(Self::Html),
            "csv" => Ok(Self::Csv),
            _unsupported => Err(DocumentIngestValidationIssue::UnsupportedFormat),
        }
    }

    /// Returns the stable format label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlainText => "plain-text",
            Self::Markdown => "markdown",
            Self::Pdf => "pdf",
            Self::Html => "html",
            Self::Csv => "csv",
        }
    }
}

/// Extraction result status reported by the host adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentIngestStatus {
    /// Extraction completed with bounded summaries.
    Completed,
    /// Extraction completed after deterministic host truncation.
    Truncated,
    /// The host rejected the format before parser execution.
    Unsupported,
    /// The parser failed and the report must not be admitted as context.
    ParserFailed,
}

impl DocumentIngestStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Truncated => "truncated",
            Self::Unsupported => "unsupported",
            Self::ParserFailed => "parser_failed",
        }
    }
}

/// Output truncation state for extracted document summaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentTruncationStatus {
    /// Output fits host policy.
    None,
    /// Host truncated output to policy.
    Truncated,
    /// Host rejected output instead of admitting context.
    Rejected,
}

impl DocumentTruncationStatus {
    /// Returns the stable truncation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Truncated => "truncated",
            Self::Rejected => "rejected",
        }
    }
}

/// Host-reported process isolation status for extractor execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentIsolationStatus {
    /// Isolation was not requested by host policy.
    NotRequested,
    /// Isolation was requested and ran according to host reporting.
    RequestedAndRan,
    /// Isolation was requested but unavailable; report is fail-closed.
    RequestedUnavailable,
}

impl DocumentIsolationStatus {
    /// Returns the stable process-isolation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotRequested => "not_requested",
            Self::RequestedAndRan => "requested_and_ran",
            Self::RequestedUnavailable => "requested_unavailable",
        }
    }
}

/// Trust label for extracted document content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentContentTrust {
    /// Content is untrusted context only.
    UntrustedContext,
    /// Content claims trusted semantics and must be rejected.
    TrustedContent,
}

impl DocumentContentTrust {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UntrustedContext => "untrusted_context",
            Self::TrustedContent => UNSAFE_TRUST_LABEL,
        }
    }
}

/// Redaction classes required on safe document-ingest reports.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DocumentRedactionLabel {
    /// Host paths were omitted or reduced to safe labels.
    SourcePathOmitted,
    /// Extracted content was bounded to summaries and counts.
    ContentBounded,
    /// Raw source bytes are represented by BLAKE3 identity only.
    PayloadHashOnly,
    /// Host redacted privacy-sensitive document text.
    SensitiveTextRedacted,
}

impl DocumentRedactionLabel {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourcePathOmitted => "source_path_omitted",
            Self::ContentBounded => "content_bounded",
            Self::PayloadHashOnly => "payload_hash_only",
            Self::SensitiveTextRedacted => "sensitive_text_redacted",
        }
    }
}

/// Explicit non-claims that must accompany accepted document context.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DocumentIngestNonClaim {
    /// No parser-soundness proof is implied.
    ParserSoundness,
    /// Extraction may be incomplete.
    ExtractionCompleteness,
    /// OCR coverage is not claimed.
    OcrCoverage,
    /// Extracted content is not semantically true by construction.
    SemanticTruth,
    /// Embedded content is not safe to execute.
    EmbeddedContentSafety,
    /// Release eligibility is not established by document intake.
    ReleaseEligibility,
    /// Downstream task correctness is not established.
    DownstreamTaskCorrectness,
}

impl DocumentIngestNonClaim {
    /// Returns the stable non-claim label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParserSoundness => "no_parser_soundness_claim",
            Self::ExtractionCompleteness => "no_extraction_completeness_claim",
            Self::OcrCoverage => "no_ocr_coverage_claim",
            Self::SemanticTruth => "no_semantic_truth_claim",
            Self::EmbeddedContentSafety => "no_embedded_content_safety_claim",
            Self::ReleaseEligibility => "no_release_eligibility_claim",
            Self::DownstreamTaskCorrectness => "no_downstream_task_correctness_claim",
        }
    }
}

/// Deterministic document-ingest validation issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentIngestValidationIssue {
    /// A required field was empty, too large, or contained a NUL byte.
    InvalidField,
    /// Safe metadata looked like raw content, credentials, or host paths.
    UnsafeMetadata,
    /// Hash fields were malformed or did not match.
    StaleSourceHash,
    /// The format label is unsupported.
    UnsupportedFormat,
    /// Host reported a parser failure.
    ParserFailure,
    /// Counts exceeded declared budgets without truncation or rejection.
    BudgetViolation,
    /// Required summaries or diagnostics were missing.
    MissingRequiredEvidence,
    /// A trusted-content label attempted to overclaim extracted text.
    TrustedContentLabel,
    /// Required non-claims were absent.
    MissingNonClaim,
    /// Required redaction labels were absent.
    MissingRedaction,
    /// A collection exceeded the public DTO bound.
    TooManyItems,
    /// Host-requested isolation did not run.
    IsolationUnavailable,
    /// Safe receipt construction rejected metadata.
    SafeReceiptRejected,
}

impl DocumentIngestValidationIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidField => "invalid_field",
            Self::UnsafeMetadata => "unsafe_metadata",
            Self::StaleSourceHash => "stale_source_hash",
            Self::UnsupportedFormat => "unsupported_format",
            Self::ParserFailure => "parser_failure",
            Self::BudgetViolation => "budget_violation",
            Self::MissingRequiredEvidence => "missing_required_evidence",
            Self::TrustedContentLabel => "trusted_content_label",
            Self::MissingNonClaim => "missing_non_claim",
            Self::MissingRedaction => "missing_redaction",
            Self::TooManyItems => "too_many_items",
            Self::IsolationUnavailable => "isolation_unavailable",
            Self::SafeReceiptRejected => "safe_receipt_rejected",
        }
    }
}

/// Safe diagnostic emitted by document-ingest validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentIngestDiagnostic {
    issue: DocumentIngestValidationIssue,
    field: &'static str,
    message: String,
}

impl DocumentIngestDiagnostic {
    /// Builds a safe document-ingest diagnostic.
    #[must_use]
    pub fn new(issue: DocumentIngestValidationIssue, field: &'static str, message: &str) -> Self {
        Self {
            issue,
            field,
            message: String::from(message),
        }
    }

    /// Returns the issue code.
    #[must_use]
    pub const fn issue(&self) -> DocumentIngestValidationIssue {
        self.issue
    }

    /// Returns the field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the safe diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }
}

/// Borrowed source identity fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentSourceIdentityDraft<'a> {
    /// Safe source label, never a raw host path.
    pub safe_source_label: &'a str,
    /// Expected BLAKE3 hash for the source artifact.
    pub source_hash: &'a str,
    /// Observed BLAKE3 hash from the host extraction boundary.
    pub observed_source_hash: &'a str,
}

/// Validated source artifact identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSourceIdentity {
    safe_source_label: String,
    source_hash: crate::ArtifactHash,
}

impl DocumentSourceIdentity {
    /// Validates source identity and freshness.
    ///
    /// # Errors
    ///
    /// Returns diagnostics when the label is unsafe, either hash is malformed,
    /// or the observed hash differs from the expected source hash.
    pub fn new(draft: DocumentSourceIdentityDraft<'_>) -> Result<Self, DocumentIngestError> {
        let mut diagnostics = Vec::new();
        let safe_source_label = validate_safe_label(
            SOURCE_LABEL_FIELD,
            draft.safe_source_label,
            &mut diagnostics,
        );
        let source_hash =
            validate_hash("document_source_hash", draft.source_hash, &mut diagnostics);
        let observed_hash = validate_hash(
            "document_observed_source_hash",
            draft.observed_source_hash,
            &mut diagnostics,
        );
        if source_hash
            .as_ref()
            .zip(observed_hash.as_ref())
            .is_some_and(|(expected, observed)| expected != observed)
        {
            diagnostics.push(DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::StaleSourceHash,
                "document_source_hash",
                "document source hash does not match observed extraction hash",
            ));
        }
        if diagnostics.is_empty() {
            if let (Some(safe_source_label), Some(source_hash)) = (safe_source_label, source_hash) {
                return Ok(Self {
                    safe_source_label,
                    source_hash,
                });
            }
            diagnostics.push(DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::InvalidField,
                "document_source_identity",
                "document source identity validation ended without required fields",
            ));
        }
        Err(DocumentIngestError { diagnostics })
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn safe_source_label(&self) -> &str {
        self.safe_source_label.as_str()
    }

    /// Returns the source artifact hash.
    #[must_use]
    pub const fn source_hash(&self) -> &crate::ArtifactHash {
        &self.source_hash
    }
}

/// Borrowed document section summary before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentSectionDraft<'a> {
    /// Stable host-local section identifier.
    pub section_id: &'a str,
    /// Safe heading label.
    pub heading: &'a str,
    /// Bounded section summary.
    pub summary: &'a str,
    /// Host-counted original bytes represented by this section summary.
    pub byte_count: u32,
}

/// Validated bounded document section summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSection {
    section_id: String,
    heading: String,
    summary: String,
    byte_count: u32,
}

impl DocumentSection {
    /// Returns the section identifier.
    #[must_use]
    pub fn section_id(&self) -> &str {
        self.section_id.as_str()
    }

    /// Returns the safe heading label.
    #[must_use]
    pub fn heading(&self) -> &str {
        self.heading.as_str()
    }

    /// Returns the bounded summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the host-counted source byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u32 {
        self.byte_count
    }
}

/// Borrowed optional table-cell summary before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentTableCellDraft<'a> {
    /// Stable table identifier.
    pub table_id: &'a str,
    /// Zero-based row index supplied by the host.
    pub row_index: u32,
    /// Zero-based column index supplied by the host.
    pub column_index: u32,
    /// Bounded cell text summary.
    pub text_summary: &'a str,
}

/// Validated optional table-cell summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentTableCell {
    table_id: String,
    row_index: u32,
    column_index: u32,
    text_summary: String,
}

impl DocumentTableCell {
    /// Returns the table identifier.
    #[must_use]
    pub fn table_id(&self) -> &str {
        self.table_id.as_str()
    }

    /// Returns the row index.
    #[must_use]
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// Returns the column index.
    #[must_use]
    pub const fn column_index(&self) -> u32 {
        self.column_index
    }

    /// Returns the bounded text summary.
    #[must_use]
    pub fn text_summary(&self) -> &str {
        self.text_summary.as_str()
    }
}

/// Borrowed safe metadata key/value before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentMetadataDraft<'a> {
    /// Safe metadata key.
    pub key: &'a str,
    /// Safe metadata value.
    pub value: &'a str,
}

/// Validated safe metadata key/value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMetadata {
    key: String,
    value: String,
}

impl DocumentMetadata {
    /// Returns the metadata key.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the metadata value.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Borrowed document-ingest report before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentIngestReportDraft<'a> {
    /// Source artifact identity.
    pub source: DocumentSourceIdentityDraft<'a>,
    /// Admitted format label.
    pub admitted_format: &'a str,
    /// Host extractor family label.
    pub extractor_family: &'a str,
    /// Host extraction status.
    pub status: DocumentIngestStatus,
    /// Output truncation status.
    pub truncation: DocumentTruncationStatus,
    /// Host-reported content trust label.
    pub content_trust: DocumentContentTrust,
    /// Input byte count.
    pub input_bytes: u32,
    /// Input byte policy cap.
    pub input_byte_cap: u32,
    /// Extracted output byte count.
    pub output_bytes: u32,
    /// Extracted output byte policy cap.
    pub output_byte_cap: u32,
    /// Document count included in this report.
    pub document_count: u32,
    /// Document count policy cap.
    pub document_count_cap: u32,
    /// Host timeout class label.
    pub timeout_class: &'a str,
    /// Process-isolation status label.
    pub isolation: DocumentIsolationStatus,
    /// Bounded section summaries.
    pub sections: &'a [DocumentSectionDraft<'a>],
    /// Optional table-cell summaries.
    pub table_cells: &'a [DocumentTableCellDraft<'a>],
    /// Safe metadata pairs.
    pub metadata: &'a [DocumentMetadataDraft<'a>],
    /// Host and validation diagnostics.
    pub diagnostics: &'a [DocumentIngestDiagnostic],
    /// Redaction labels.
    pub redactions: &'a [DocumentRedactionLabel],
    /// Explicit non-claims.
    pub non_claims: &'a [DocumentIngestNonClaim],
}

/// Validated document-ingest report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentIngestReport {
    source: DocumentSourceIdentity,
    admitted_format: DocumentFormat,
    extractor_family: String,
    status: DocumentIngestStatus,
    truncation: DocumentTruncationStatus,
    content_trust: DocumentContentTrust,
    input_bytes: u32,
    input_byte_cap: u32,
    output_bytes: u32,
    output_byte_cap: u32,
    document_count: u32,
    document_count_cap: u32,
    timeout_class: String,
    isolation: DocumentIsolationStatus,
    sections: Vec<DocumentSection>,
    table_cells: Vec<DocumentTableCell>,
    metadata: Vec<DocumentMetadata>,
    diagnostics: Vec<DocumentIngestDiagnostic>,
    redactions: Vec<DocumentRedactionLabel>,
    non_claims: Vec<DocumentIngestNonClaim>,
    receipt: crate::safe::receipt::Record,
}

impl DocumentIngestReport {
    /// Validates a document-ingest report draft.
    ///
    /// # Errors
    ///
    /// Returns deterministic diagnostics when the report is malformed, unsafe,
    /// unsupported, stale, over budget, overclaiming, or missing non-claims.
    pub fn new(draft: DocumentIngestReportDraft<'_>) -> Result<Self, DocumentIngestError> {
        validate_document_ingest_report(draft)
    }

    /// Returns source identity.
    #[must_use]
    pub const fn source(&self) -> &DocumentSourceIdentity {
        &self.source
    }

    /// Returns admitted format.
    #[must_use]
    pub const fn admitted_format(&self) -> DocumentFormat {
        self.admitted_format
    }

    /// Returns extractor family label.
    #[must_use]
    pub fn extractor_family(&self) -> &str {
        self.extractor_family.as_str()
    }

    /// Returns extraction status.
    #[must_use]
    pub const fn status(&self) -> DocumentIngestStatus {
        self.status
    }

    /// Returns truncation status.
    #[must_use]
    pub const fn truncation(&self) -> DocumentTruncationStatus {
        self.truncation
    }

    /// Returns content trust label.
    #[must_use]
    pub const fn content_trust(&self) -> DocumentContentTrust {
        self.content_trust
    }

    /// Returns bounded section summaries.
    #[must_use]
    pub fn sections(&self) -> &[DocumentSection] {
        self.sections.as_slice()
    }

    /// Returns optional table-cell summaries.
    #[must_use]
    pub fn table_cells(&self) -> &[DocumentTableCell] {
        self.table_cells.as_slice()
    }

    /// Returns safe metadata pairs.
    #[must_use]
    pub fn metadata(&self) -> &[DocumentMetadata] {
        self.metadata.as_slice()
    }

    /// Returns diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[DocumentIngestDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns redaction labels.
    #[must_use]
    pub fn redactions(&self) -> &[DocumentRedactionLabel] {
        self.redactions.as_slice()
    }

    /// Returns explicit non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[DocumentIngestNonClaim] {
        self.non_claims.as_slice()
    }

    /// Returns safe receipt metadata.
    #[must_use]
    pub const fn receipt(&self) -> &crate::safe::receipt::Record {
        &self.receipt
    }

    /// Returns a safe context-assembly receipt preserving document non-claims.
    ///
    /// # Errors
    ///
    /// Returns diagnostics when the supplied context label is unsafe or safe
    /// receipt construction rejects metadata.
    pub fn context_receipt(
        &self,
        context_label: &str,
    ) -> Result<crate::safe::receipt::Record, DocumentIngestError> {
        let mut diagnostics = Vec::new();
        let context_label =
            validate_safe_label(CONTEXT_LABEL_FIELD, context_label, &mut diagnostics);
        if !diagnostics.is_empty() {
            return Err(DocumentIngestError { diagnostics });
        }
        let Some(context_label) = context_label else {
            return Err(DocumentIngestError {
                diagnostics: alloc::vec![DocumentIngestDiagnostic::new(
                    DocumentIngestValidationIssue::InvalidField,
                    CONTEXT_LABEL_FIELD.as_str(),
                    "document context label validation ended without required field",
                )],
            });
        };
        build_receipt(
            &ReceiptInput {
                source: &self.source,
                format: self.admitted_format,
                extractor_family: self.extractor_family.as_str(),
                status: self.status,
                truncation: self.truncation,
                content_trust: self.content_trust,
                input_bytes: self.input_bytes,
                output_bytes: self.output_bytes,
                document_count: self.document_count,
                section_count: len_to_u32(self.sections.len()),
                table_cell_count: len_to_u32(self.table_cells.len()),
                metadata_count: len_to_u32(self.metadata.len()),
                diagnostic_count: len_to_u32(self.diagnostics.len()),
                non_claim_count: len_to_u32(self.non_claims.len()),
                redaction_count: len_to_u32(self.redactions.len()),
                timeout_class: self.timeout_class.as_str(),
                isolation: self.isolation,
                redactions: self.redactions.as_slice(),
                non_claims: self.non_claims.as_slice(),
                extra_label: Some(context_label.as_str()),
            },
            crate::safe::receipt::Operation::ProvenanceObservation,
        )
        .map_err(|diagnostic| DocumentIngestError {
            diagnostics: alloc::vec![diagnostic],
        })
    }
}

/// Document-ingest validation error with deterministic diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentIngestError {
    diagnostics: Vec<DocumentIngestDiagnostic>,
}

impl DocumentIngestError {
    /// Returns validation diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[DocumentIngestDiagnostic] {
        self.diagnostics.as_slice()
    }
}

impl core::fmt::Display for DocumentIngestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "document ingest validation failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}

/// Validates an already-loaded document-ingest report DTO.
///
/// # Errors
///
/// Returns deterministic diagnostics when validation fails. This function is
/// pure over the supplied DTO and performs no I/O, parser execution, process
/// spawning, environment inspection, clock access, or network access.
pub fn validate_document_ingest_report(
    draft: DocumentIngestReportDraft<'_>,
) -> Result<DocumentIngestReport, DocumentIngestError> {
    let mut diagnostics = Vec::new();
    let source = match DocumentSourceIdentity::new(draft.source) {
        Ok(identity) => Some(identity),
        Err(error) => {
            diagnostics.extend(error.diagnostics);
            None
        }
    };
    let admitted_format = match DocumentFormat::parse(draft.admitted_format) {
        Ok(format) => Some(format),
        Err(issue) => {
            diagnostics.push(DocumentIngestDiagnostic::new(
                issue,
                FORMAT_FIELD.as_str(),
                "document format is not admitted",
            ));
            None
        }
    };
    let extractor_family =
        validate_safe_label(EXTRACTOR_FIELD, draft.extractor_family, &mut diagnostics);
    let timeout_class =
        validate_safe_label(TIMEOUT_CLASS_FIELD, draft.timeout_class, &mut diagnostics);
    let sections = validate_sections(draft.sections, &mut diagnostics);
    let table_cells = validate_table_cells(draft.table_cells, &mut diagnostics);
    let metadata = validate_metadata(draft.metadata, &mut diagnostics);
    validate_supplied_diagnostics(draft.diagnostics, &mut diagnostics);
    validate_status_and_budget(draft, &mut diagnostics);
    let redactions = validate_redactions(draft.redactions, &mut diagnostics);
    let non_claims = validate_non_claims(draft.non_claims, &mut diagnostics);

    if diagnostics.is_empty() {
        return assemble_document_report(
            draft,
            DocumentAssembleParts {
                source,
                admitted_format,
                extractor_family,
                timeout_class,
                sections,
                table_cells,
                metadata,
                redactions,
                non_claims,
            },
        );
    }
    Err(DocumentIngestError { diagnostics })
}

struct DocumentAssembleParts {
    source: Option<DocumentSourceIdentity>,
    admitted_format: Option<DocumentFormat>,
    extractor_family: Option<String>,
    timeout_class: Option<String>,
    sections: Vec<DocumentSection>,
    table_cells: Vec<DocumentTableCell>,
    metadata: Vec<DocumentMetadata>,
    redactions: Option<Vec<DocumentRedactionLabel>>,
    non_claims: Option<Vec<DocumentIngestNonClaim>>,
}

fn assemble_document_report(
    draft: DocumentIngestReportDraft<'_>,
    parts: DocumentAssembleParts,
) -> Result<DocumentIngestReport, DocumentIngestError> {
    let DocumentAssembleParts {
        source,
        admitted_format,
        extractor_family,
        timeout_class,
        sections,
        table_cells,
        metadata,
        redactions,
        non_claims,
    } = parts;
    let (
        Some(source),
        Some(admitted_format),
        Some(extractor_family),
        Some(timeout_class),
        Some(redactions),
        Some(non_claims),
    ) = (
        source,
        admitted_format,
        extractor_family,
        timeout_class,
        redactions,
        non_claims,
    )
    else {
        return Err(DocumentIngestError {
            diagnostics: alloc::vec![DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::InvalidField,
                "document_report",
                "document report validation ended without required fields",
            )],
        });
    };
    let receipt = build_document_report_receipt(&DocumentReportReceiptInput {
        draft,
        source: &source,
        admitted_format,
        extractor_family: extractor_family.as_str(),
        timeout_class: timeout_class.as_str(),
        section_count: len_to_u32(sections.len()),
        table_cell_count: len_to_u32(table_cells.len()),
        metadata_count: len_to_u32(metadata.len()),
        redactions: redactions.as_slice(),
        non_claims: non_claims.as_slice(),
    })?;
    Ok(DocumentIngestReport {
        source,
        admitted_format,
        extractor_family,
        status: draft.status,
        truncation: draft.truncation,
        content_trust: draft.content_trust,
        input_bytes: draft.input_bytes,
        input_byte_cap: draft.input_byte_cap,
        output_bytes: draft.output_bytes,
        output_byte_cap: draft.output_byte_cap,
        document_count: draft.document_count,
        document_count_cap: draft.document_count_cap,
        timeout_class,
        isolation: draft.isolation,
        sections,
        table_cells,
        metadata,
        diagnostics: draft.diagnostics.to_vec(),
        redactions,
        non_claims,
        receipt,
    })
}

#[derive(Clone, Copy)]
struct DocumentReportReceiptInput<'a> {
    draft: DocumentIngestReportDraft<'a>,
    source: &'a DocumentSourceIdentity,
    admitted_format: DocumentFormat,
    extractor_family: &'a str,
    timeout_class: &'a str,
    section_count: u32,
    table_cell_count: u32,
    metadata_count: u32,
    redactions: &'a [DocumentRedactionLabel],
    non_claims: &'a [DocumentIngestNonClaim],
}

fn build_document_report_receipt(
    input: &DocumentReportReceiptInput<'_>,
) -> Result<crate::safe::receipt::Record, DocumentIngestError> {
    build_receipt(
        &ReceiptInput {
            source: input.source,
            format: input.admitted_format,
            extractor_family: input.extractor_family,
            status: input.draft.status,
            truncation: input.draft.truncation,
            content_trust: input.draft.content_trust,
            input_bytes: input.draft.input_bytes,
            output_bytes: input.draft.output_bytes,
            document_count: input.draft.document_count,
            section_count: input.section_count,
            table_cell_count: input.table_cell_count,
            metadata_count: input.metadata_count,
            diagnostic_count: len_to_u32(input.draft.diagnostics.len()),
            non_claim_count: len_to_u32(input.non_claims.len()),
            redaction_count: len_to_u32(input.redactions.len()),
            timeout_class: input.timeout_class,
            isolation: input.draft.isolation,
            redactions: input.redactions,
            non_claims: input.non_claims,
            extra_label: None,
        },
        crate::safe::receipt::Operation::AdapterBoundary,
    )
    .map_err(|diagnostic| DocumentIngestError {
        diagnostics: alloc::vec![diagnostic],
    })
}

fn validate_sections(
    drafts: &[DocumentSectionDraft<'_>],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Vec<DocumentSection> {
    if drafts.is_empty() {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::MissingRequiredEvidence,
            "document_sections",
            "document ingest report requires at least one bounded section summary",
        ));
    }
    if drafts.len() > MAX_SECTIONS {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TooManyItems,
            "document_sections",
            "document ingest section count exceeds DTO bound",
        ));
    }
    drafts
        .iter()
        .take(MAX_SECTIONS)
        .filter_map(|draft| validate_section(*draft, diagnostics))
        .collect()
}

fn validate_section(
    draft: DocumentSectionDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<DocumentSection> {
    let section_id = validate_safe_label(SECTION_ID_FIELD, draft.section_id, diagnostics);
    let heading = validate_safe_label(SECTION_HEADING_FIELD, draft.heading, diagnostics);
    let summary = validate_bounded_safe_text(
        SECTION_SUMMARY_FIELD,
        draft.summary,
        MAX_SECTION_SUMMARY_CHARS,
        diagnostics,
    );
    if draft.byte_count == ZERO_COUNT {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            FIELD_SECTION_COUNT,
            "document section byte count must be positive",
        ));
    }
    section_id
        .zip(heading)
        .zip(summary)
        .map(|((section_id, heading), summary)| DocumentSection {
            section_id,
            heading,
            summary,
            byte_count: draft.byte_count,
        })
}

fn validate_table_cells(
    drafts: &[DocumentTableCellDraft<'_>],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Vec<DocumentTableCell> {
    if drafts.len() > MAX_TABLE_CELLS {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TooManyItems,
            "document_table_cells",
            "document table-cell count exceeds DTO bound",
        ));
    }
    drafts
        .iter()
        .take(MAX_TABLE_CELLS)
        .filter_map(|draft| validate_table_cell(*draft, diagnostics))
        .collect()
}

fn validate_table_cell(
    draft: DocumentTableCellDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<DocumentTableCell> {
    let table_id = validate_safe_label(TABLE_ID_FIELD, draft.table_id, diagnostics);
    let text_summary = validate_bounded_safe_text(
        TABLE_CELL_FIELD,
        draft.text_summary,
        MAX_TABLE_CELL_CHARS,
        diagnostics,
    );
    table_id
        .zip(text_summary)
        .map(|(table_id, text_summary)| DocumentTableCell {
            table_id,
            row_index: draft.row_index,
            column_index: draft.column_index,
            text_summary,
        })
}

fn validate_metadata(
    drafts: &[DocumentMetadataDraft<'_>],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Vec<DocumentMetadata> {
    if drafts.len() > MAX_METADATA_PAIRS {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TooManyItems,
            "document_metadata",
            "document metadata count exceeds DTO bound",
        ));
    }
    drafts
        .iter()
        .take(MAX_METADATA_PAIRS)
        .filter_map(|draft| validate_metadata_pair(*draft, diagnostics))
        .collect()
}

fn validate_metadata_pair(
    draft: DocumentMetadataDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<DocumentMetadata> {
    let key = validate_safe_label(METADATA_KEY_FIELD, draft.key, diagnostics);
    let value = validate_bounded_safe_text(
        METADATA_VALUE_FIELD,
        draft.value,
        MAX_METADATA_VALUE_CHARS,
        diagnostics,
    );
    key.zip(value)
        .map(|(key, value)| DocumentMetadata { key, value })
}

fn validate_supplied_diagnostics(
    supplied: &[DocumentIngestDiagnostic],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) {
    if supplied.len() > MAX_DIAGNOSTICS {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TooManyItems,
            "document_diagnostics",
            "document diagnostics count exceeds DTO bound",
        ));
    }
    for diagnostic in supplied.iter().take(MAX_DIAGNOSTICS) {
        validate_bounded_safe_text(
            DIAGNOSTIC_FIELD,
            diagnostic.message(),
            MAX_DIAGNOSTIC_CHARS,
            diagnostics,
        );
    }
}

fn validate_status_and_budget(
    draft: DocumentIngestReportDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) {
    if draft.status == DocumentIngestStatus::Unsupported {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::UnsupportedFormat,
            "document_status",
            "unsupported document formats are fail-closed and not admitted as context",
        ));
    }
    if draft.status == DocumentIngestStatus::ParserFailed {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::ParserFailure,
            "document_status",
            "parser failure reports are not admitted as agent context",
        ));
    }
    if draft.status == DocumentIngestStatus::ParserFailed && draft.diagnostics.is_empty() {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::MissingRequiredEvidence,
            "document_diagnostics",
            "parser failures require safe diagnostics",
        ));
    }
    if draft.content_trust == DocumentContentTrust::TrustedContent {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TrustedContentLabel,
            "document_content_trust",
            "document content must remain untrusted context",
        ));
    }
    if draft.isolation == DocumentIsolationStatus::RequestedUnavailable {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::IsolationUnavailable,
            "document_isolation",
            "requested process isolation did not run",
        ));
    }
    validate_caps(draft, diagnostics);
    validate_over_budget(draft, diagnostics);
}

fn validate_caps(
    draft: DocumentIngestReportDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) {
    if draft.input_byte_cap < MIN_CAP
        || draft.output_byte_cap < MIN_CAP
        || draft.document_count_cap < MIN_CAP
    {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            "document_budget",
            "document ingest budget caps must be positive",
        ));
    }
    if draft.document_count == ZERO_COUNT {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            FIELD_DOCUMENT_COUNT,
            "document count must be positive",
        ));
    }
}

fn validate_over_budget(
    draft: DocumentIngestReportDraft<'_>,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) {
    let input_over = draft.input_bytes > draft.input_byte_cap;
    let output_over = draft.output_bytes > draft.output_byte_cap;
    let documents_over = draft.document_count > draft.document_count_cap;
    if (input_over || output_over || documents_over)
        && draft.truncation == DocumentTruncationStatus::None
    {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            "document_budget",
            "over-budget document ingest reports require truncation or rejection status",
        ));
    }
    if draft.status == DocumentIngestStatus::Completed
        && draft.truncation == DocumentTruncationStatus::Rejected
    {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            "document_truncation",
            "completed document reports cannot be marked rejected",
        ));
    }
}

fn validate_redactions(
    values: &[DocumentRedactionLabel],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<Vec<DocumentRedactionLabel>> {
    if values.len() > MAX_REDACTIONS {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::TooManyItems,
            "document_redactions",
            "document redaction count exceeds DTO bound",
        ));
    }
    if !contains_redaction(values, DocumentRedactionLabel::ContentBounded)
        || !contains_redaction(values, DocumentRedactionLabel::PayloadHashOnly)
    {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::MissingRedaction,
            "document_redactions",
            "document ingest reports require bounded-content and payload-hash redactions",
        ));
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.field() == "document_redactions")
    {
        return None;
    }
    let mut redactions = values.to_vec();
    redactions.sort();
    redactions.dedup();
    Some(redactions)
}

fn validate_non_claims(
    values: &[DocumentIngestNonClaim],
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<Vec<DocumentIngestNonClaim>> {
    for required in required_document_non_claims() {
        if !contains_non_claim(values, *required) {
            diagnostics.push(DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::MissingNonClaim,
                "document_non_claims",
                "document ingest report is missing a required non-claim",
            ));
            break;
        }
    }
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.field() == "document_non_claims")
    {
        return None;
    }
    let mut non_claims = values.to_vec();
    non_claims.sort();
    non_claims.dedup();
    Some(non_claims)
}

fn contains_redaction(values: &[DocumentRedactionLabel], target: DocumentRedactionLabel) -> bool {
    values.contains(&target)
}

fn contains_non_claim(values: &[DocumentIngestNonClaim], target: DocumentIngestNonClaim) -> bool {
    values.contains(&target)
}

fn required_document_non_claims() -> &'static [DocumentIngestNonClaim; REQUIRED_DOCUMENT_NON_CLAIMS]
{
    &[
        DocumentIngestNonClaim::ParserSoundness,
        DocumentIngestNonClaim::ExtractionCompleteness,
        DocumentIngestNonClaim::OcrCoverage,
        DocumentIngestNonClaim::SemanticTruth,
        DocumentIngestNonClaim::EmbeddedContentSafety,
        DocumentIngestNonClaim::ReleaseEligibility,
        DocumentIngestNonClaim::DownstreamTaskCorrectness,
    ]
}

struct ReceiptInput<'a> {
    source: &'a DocumentSourceIdentity,
    format: DocumentFormat,
    extractor_family: &'a str,
    status: DocumentIngestStatus,
    truncation: DocumentTruncationStatus,
    content_trust: DocumentContentTrust,
    input_bytes: u32,
    output_bytes: u32,
    document_count: u32,
    section_count: u32,
    table_cell_count: u32,
    metadata_count: u32,
    diagnostic_count: u32,
    non_claim_count: u32,
    redaction_count: u32,
    timeout_class: &'a str,
    isolation: DocumentIsolationStatus,
    redactions: &'a [DocumentRedactionLabel],
    non_claims: &'a [DocumentIngestNonClaim],
    extra_label: Option<&'a str>,
}

fn build_receipt(
    input: &ReceiptInput<'_>,
    operation: crate::safe::receipt::Operation,
) -> Result<crate::safe::receipt::Record, DocumentIngestDiagnostic> {
    let label_storage = receipt_labels(input);
    let label_refs = label_storage.iter().map(String::as_str).collect::<Vec<_>>();
    let counts = [
        crate::safe::receipt::BoundedCountDraft::new(FIELD_INPUT_BYTES, input.input_bytes),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_OUTPUT_BYTES, input.output_bytes),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_DOCUMENT_COUNT, input.document_count),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_SECTION_COUNT, input.section_count),
        crate::safe::receipt::BoundedCountDraft::new(
            FIELD_TABLE_CELL_COUNT,
            input.table_cell_count,
        ),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_METADATA_COUNT, input.metadata_count),
        crate::safe::receipt::BoundedCountDraft::new(
            FIELD_DIAGNOSTIC_COUNT,
            input.diagnostic_count,
        ),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_NON_CLAIM_COUNT, input.non_claim_count),
        crate::safe::receipt::BoundedCountDraft::new(FIELD_REDACTION_COUNT, input.redaction_count),
    ];
    let hashes = [input.source.source_hash().clone()];
    let redactions = receipt_redactions(input.redactions);
    crate::safe::receipt::Record::new(
        crate::safe::receipt::Draft::new(
            crate::safe::receipt::Status::Completed,
            crate::safe::receipt::IssueCode::ProvenanceObservation,
            operation,
        )
        .with_labels(label_refs.as_slice())
        .with_counts(&counts)
        .with_artifact_hashes(&hashes)
        .with_backend_label(RECEIPT_BACKEND_LABEL)
        .with_interchange_label(RECEIPT_INTERCHANGE_LABEL)
        .with_redactions(redactions.as_slice()),
    )
    .map_err(|_error| {
        DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::SafeReceiptRejected,
            "document_receipt",
            "safe receipt rejected document ingest metadata",
        )
    })
}

fn receipt_labels(input: &ReceiptInput<'_>) -> Vec<String> {
    let mut labels = alloc::vec![
        String::from(input.source.safe_source_label()),
        String::from(input.format.as_str()),
        String::from(input.extractor_family),
        String::from(input.status.as_str()),
        String::from(input.truncation.as_str()),
        String::from(input.content_trust.as_str()),
        String::from(input.timeout_class),
        String::from(input.isolation.as_str()),
    ];
    labels.extend(
        input
            .non_claims
            .iter()
            .map(|claim| String::from(claim.as_str())),
    );
    labels.extend(
        input
            .redactions
            .iter()
            .map(|redaction| String::from(redaction.as_str())),
    );
    if let Some(extra_label) = input.extra_label {
        labels.push(String::from(extra_label));
    }
    labels
}

fn receipt_redactions(values: &[DocumentRedactionLabel]) -> Vec<crate::safe::receipt::Redaction> {
    let mut redactions = Vec::new();
    redactions.push(crate::safe::receipt::Redaction::OutputBounded);
    redactions.push(crate::safe::receipt::Redaction::PayloadHashOnly);
    if contains_redaction(values, DocumentRedactionLabel::SourcePathOmitted) {
        redactions.push(crate::safe::receipt::Redaction::AbsolutePathOmitted);
    }
    if contains_redaction(values, DocumentRedactionLabel::SensitiveTextRedacted) {
        redactions.push(crate::safe::receipt::Redaction::ToolOutputOmitted);
    }
    redactions.sort();
    redactions.dedup();
    redactions
}

fn validate_hash(
    field: &'static str,
    value: &str,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<crate::ArtifactHash> {
    match crate::ArtifactHash::new(value) {
        Ok(hash) => Some(hash),
        Err(_error) => {
            diagnostics.push(DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::StaleSourceHash,
                field,
                "document source hash must be lowercase BLAKE3 hex",
            ));
            None
        }
    }
}

fn validate_safe_label(
    field: FieldName,
    value: &str,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<String> {
    match normalize_label(field, value) {
        Ok(label) if is_safe_text(label.as_str()) => Some(label),
        Ok(_unsafe) => {
            diagnostics.push(DocumentIngestDiagnostic::new(
                DocumentIngestValidationIssue::UnsafeMetadata,
                field.as_str(),
                "document metadata contains unsafe raw material marker",
            ));
            None
        }
        Err(issue) => {
            diagnostics.push(DocumentIngestDiagnostic::new(
                issue,
                field.as_str(),
                "document metadata field is invalid",
            ));
            None
        }
    }
}

fn normalize_label(field: FieldName, value: &str) -> Result<String, DocumentIngestValidationIssue> {
    match validation::normalized_adapter_field(field, value) {
        Ok(label) => Ok(label),
        Err(
            AdapterContractError::EmptyField { .. }
            | AdapterContractError::FieldTooLong { .. }
            | AdapterContractError::NulCharacter { .. },
        ) => Err(DocumentIngestValidationIssue::InvalidField),
        Err(_error) => Err(DocumentIngestValidationIssue::InvalidField),
    }
}

fn validate_bounded_safe_text(
    field: FieldName,
    value: &str,
    max_chars: usize,
    diagnostics: &mut Vec<DocumentIngestDiagnostic>,
) -> Option<String> {
    let text = validate_safe_label(field, value, diagnostics)?;
    if text.chars().count() > max_chars {
        diagnostics.push(DocumentIngestDiagnostic::new(
            DocumentIngestValidationIssue::BudgetViolation,
            field.as_str(),
            "document text summary exceeds DTO character budget",
        ));
        return None;
    }
    Some(text)
}

fn is_safe_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if lower.starts_with(ABSOLUTE_UNIX_PREFIX) {
        return false;
    }
    !UNSAFE_TEXT_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

fn len_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(crate::safe::receipt::MAX_COUNT)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SAFE_SOURCE: &str = "artifact:doc:alpha";
    const EXTRACTOR: &str = "fixture-extractor";
    const TIMEOUT: &str = "short";
    const SECTION_ID: &str = "section-1";
    const SECTION_HEADING: &str = "overview";
    const SECTION_SUMMARY: &str = "bounded summary";
    const TABLE_ID: &str = "table-1";
    const TABLE_SUMMARY: &str = "bounded cell";
    const METADATA_KEY: &str = "title";
    const METADATA_VALUE: &str = "safe title";
    const CONTEXT_LABEL: &str = "context:document:alpha";
    const INPUT_BYTES: u32 = 512;
    const INPUT_CAP: u32 = 1_024;
    const OUTPUT_BYTES: u32 = 128;
    const OUTPUT_CAP: u32 = 512;
    const DOCUMENT_COUNT: u32 = 1;
    const DOCUMENT_CAP: u32 = 4;
    const SECTION_BYTES: u32 = 256;

    const EXPECTED_SECTION_COUNT: usize = 1;
    const FIRST_TABLE_INDEX: u32 = 0;

    struct Fixture {
        sections: [DocumentSectionDraft<'static>; EXPECTED_SECTION_COUNT],
        table_cells: [DocumentTableCellDraft<'static>; EXPECTED_SECTION_COUNT],
        metadata: [DocumentMetadataDraft<'static>; EXPECTED_SECTION_COUNT],
        diagnostics: [DocumentIngestDiagnostic; EXPECTED_SECTION_COUNT],
        redactions: [DocumentRedactionLabel; REQUIRED_DOCUMENT_REDACTIONS],
        non_claims: [DocumentIngestNonClaim; REQUIRED_DOCUMENT_NON_CLAIMS],
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                sections: [DocumentSectionDraft {
                    section_id: SECTION_ID,
                    heading: SECTION_HEADING,
                    summary: SECTION_SUMMARY,
                    byte_count: SECTION_BYTES,
                }],
                table_cells: [DocumentTableCellDraft {
                    table_id: TABLE_ID,
                    row_index: FIRST_TABLE_INDEX,
                    column_index: FIRST_TABLE_INDEX,
                    text_summary: TABLE_SUMMARY,
                }],
                metadata: [DocumentMetadataDraft {
                    key: METADATA_KEY,
                    value: METADATA_VALUE,
                }],
                diagnostics: [DocumentIngestDiagnostic::new(
                    DocumentIngestValidationIssue::InvalidField,
                    "fixture",
                    "safe fixture diagnostic",
                )],
                redactions: [
                    DocumentRedactionLabel::ContentBounded,
                    DocumentRedactionLabel::PayloadHashOnly,
                ],
                non_claims: *required_document_non_claims(),
            }
        }

        fn draft(&self) -> DocumentIngestReportDraft<'_> {
            DocumentIngestReportDraft {
                source: DocumentSourceIdentityDraft {
                    safe_source_label: SAFE_SOURCE,
                    source_hash: HASH_A,
                    observed_source_hash: HASH_A,
                },
                admitted_format: "markdown",
                extractor_family: EXTRACTOR,
                status: DocumentIngestStatus::Completed,
                truncation: DocumentTruncationStatus::None,
                content_trust: DocumentContentTrust::UntrustedContext,
                input_bytes: INPUT_BYTES,
                input_byte_cap: INPUT_CAP,
                output_bytes: OUTPUT_BYTES,
                output_byte_cap: OUTPUT_CAP,
                document_count: DOCUMENT_COUNT,
                document_count_cap: DOCUMENT_CAP,
                timeout_class: TIMEOUT,
                isolation: DocumentIsolationStatus::RequestedAndRan,
                sections: &self.sections,
                table_cells: &self.table_cells,
                metadata: &self.metadata,
                diagnostics: &self.diagnostics,
                redactions: &self.redactions,
                non_claims: &self.non_claims,
            }
        }
    }

    #[test]
    fn document_ingest_report_accepts_bounded_positive_fixture() {
        // r[verify document-ingest-adapters.positive]
        // r[verify document-ingest-adapters.report-dto]
        let fixture = Fixture::new();
        let report = validate_document_ingest_report(fixture.draft())
            .expect("valid document ingest report should pass");

        assert_eq!(report.admitted_format(), DocumentFormat::Markdown);
        assert_eq!(report.sections().len(), EXPECTED_SECTION_COUNT);
        assert_eq!(report.receipt().artifact_hashes()[0].as_str(), HASH_A);
        assert!(
            report
                .non_claims()
                .contains(&DocumentIngestNonClaim::SemanticTruth)
        );
        assert!(PROHIBITED_DOCUMENT_INGEST_EFFECTS.contains(&"filesystem reads"));
    }

    #[test]
    fn document_context_receipt_preserves_non_claim_labels() {
        // r[verify document-ingest-adapters.validation]
        let fixture = Fixture::new();
        let report = validate_document_ingest_report(fixture.draft())
            .expect("valid document ingest report should pass");
        let receipt = report
            .context_receipt(CONTEXT_LABEL)
            .expect("context receipt should preserve non-claims");

        assert!(
            receipt
                .labels()
                .iter()
                .any(|label| label == DocumentIngestNonClaim::ParserSoundness.as_str())
        );
        assert!(receipt.labels().iter().any(|label| label == CONTEXT_LABEL));
    }

    #[test]
    fn document_ingest_report_rejects_unsafe_negative_fixtures() {
        // r[verify document-ingest-adapters.negative]
        // r[verify document-ingest-adapters.core-validation]
        let fixture = Fixture::new();
        let mut unsupported = fixture.draft();
        unsupported.admitted_format = "docx-macro";
        assert_issue(
            validate_document_ingest_report(unsupported),
            DocumentIngestValidationIssue::UnsupportedFormat,
        );

        let mut stale = fixture.draft();
        stale.source.observed_source_hash = HASH_B;
        assert_issue(
            validate_document_ingest_report(stale),
            DocumentIngestValidationIssue::StaleSourceHash,
        );

        let mut over_budget = fixture.draft();
        over_budget.output_bytes = OUTPUT_CAP.saturating_add(DOCUMENT_COUNT);
        assert_issue(
            validate_document_ingest_report(over_budget),
            DocumentIngestValidationIssue::BudgetViolation,
        );

        let mut parser_failure = fixture.draft();
        parser_failure.status = DocumentIngestStatus::ParserFailed;
        parser_failure.diagnostics = &[];
        assert_issue(
            validate_document_ingest_report(parser_failure),
            DocumentIngestValidationIssue::ParserFailure,
        );

        let mut missing_non_claim = fixture.draft();
        missing_non_claim.non_claims = &[];
        assert_issue(
            validate_document_ingest_report(missing_non_claim),
            DocumentIngestValidationIssue::MissingNonClaim,
        );

        let mut raw_path = fixture.draft();
        raw_path.source.safe_source_label = "/home/example/private.pdf";
        assert_issue(
            validate_document_ingest_report(raw_path),
            DocumentIngestValidationIssue::UnsafeMetadata,
        );

        let mut trusted = fixture.draft();
        trusted.content_trust = DocumentContentTrust::TrustedContent;
        assert_issue(
            validate_document_ingest_report(trusted),
            DocumentIngestValidationIssue::TrustedContentLabel,
        );
    }

    fn assert_issue(
        result: Result<DocumentIngestReport, DocumentIngestError>,
        issue: DocumentIngestValidationIssue,
    ) {
        let error = result.expect_err("fixture should fail");
        assert!(
            error
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.issue() == issue),
            "expected issue {issue:?}, got {:?}",
            error.diagnostics()
        );
    }
}
