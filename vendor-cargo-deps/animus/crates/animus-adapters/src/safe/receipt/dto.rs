const REVIEW_PREFIX: &str = "safe_receipt";
const REVIEW_SEPARATOR: char = ' ';

/// Schema identifier attached to safe receipt DTOs.
pub const SCHEMA: &str = "animus.safe-receipt.v1";

/// Borrowed inputs used to construct a safe receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    pub(super) status: super::Status,
    pub(super) issue_code: super::IssueCode,
    pub(super) operation: super::Operation,
    pub(super) labels: &'a [&'a str],
    pub(super) counts: &'a [super::BoundedCountDraft<'a>],
    pub(super) artifact_hashes: &'a [crate::ArtifactHash],
    pub(super) backend_label: Option<&'a str>,
    pub(super) interchange_label: Option<&'a str>,
    pub(super) redactions: &'a [super::Redaction],
}

impl<'a> Draft<'a> {
    /// Builds a draft with required status, issue, and operation classes.
    #[must_use]
    pub const fn new(status: super::Status, issue_code: super::IssueCode, operation: super::Operation) -> Self {
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
    pub const fn with_counts(mut self, counts: &'a [super::BoundedCountDraft<'a>]) -> Self {
        self.counts = counts;
        self
    }

    /// Adds artifact identity hashes.
    #[must_use]
    pub const fn with_artifact_hashes(mut self, hashes: &'a [crate::ArtifactHash]) -> Self {
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
    pub const fn with_redactions(mut self, redactions: &'a [super::Redaction]) -> Self {
        self.redactions = redactions;
        self
    }
}

/// Safe runtime/provenance receipt containing metadata, counts, hashes, and redactions only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    schema: &'static str,
    status: super::Status,
    issue_code: super::IssueCode,
    operation: super::Operation,
    labels: alloc::vec::Vec<alloc::string::String>,
    counts: alloc::vec::Vec<super::BoundedCount>,
    artifact_hashes: alloc::vec::Vec<crate::ArtifactHash>,
    backend_label: Option<alloc::string::String>,
    interchange_label: Option<alloc::string::String>,
    redactions: alloc::vec::Vec<super::Redaction>,
}

impl Record {
    /// Builds a safe receipt after validating all metadata fields for leaks.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when required collections are empty, metadata is
    /// oversized, unsafe material is detected, or count bounds are exceeded.
    pub fn new(draft: Draft<'_>) -> Result<Self, super::Error> {
        let labels = super::normalize::labels(draft.labels)?;
        let counts = super::normalize::counts(draft.counts)?;
        let hashes = super::normalize::hashes(draft.artifact_hashes)?;
        let redactions = super::normalize::redactions(draft.redactions)?;
        let backend_label = super::normalize::backend_label(draft.backend_label)?;
        let interchange_label = super::normalize::interchange_label(draft.interchange_label)?;

        Ok(Self {
            schema: SCHEMA,
            status: draft.status,
            issue_code: draft.issue_code,
            operation: draft.operation,
            labels,
            counts,
            artifact_hashes: hashes,
            backend_label,
            interchange_label,
            redactions,
        })
    }

    /// Returns the receipt schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> super::Status {
        self.status
    }

    /// Returns the receipt issue code.
    #[must_use]
    pub const fn issue_code(&self) -> super::IssueCode {
        self.issue_code
    }

    /// Returns the effect or mutation operation class.
    #[must_use]
    pub const fn operation(&self) -> super::Operation {
        self.operation
    }

    /// Returns sorted and deduplicated safe labels.
    #[must_use]
    pub fn labels(&self) -> &[alloc::string::String] {
        self.labels.as_slice()
    }

    /// Returns sorted and deduplicated bounded counts.
    #[must_use]
    pub fn counts(&self) -> &[super::BoundedCount] {
        self.counts.as_slice()
    }

    /// Returns sorted and deduplicated artifact hashes.
    #[must_use]
    pub fn artifact_hashes(&self) -> &[crate::ArtifactHash] {
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
    pub fn redactions(&self) -> &[super::Redaction] {
        self.redactions.as_slice()
    }

    /// Renders bounded review output without raw payload fields.
    #[must_use]
    pub fn review_summary(&self) -> alloc::string::String {
        let mut builder = SummaryBuilder::new();
        builder.push_header(SummaryHeader {
            schema: self.schema,
            status: self.status,
            issue_code: self.issue_code,
            operation: self.operation,
        });
        builder.push_labels(&self.labels);
        builder.push_hashes(&self.artifact_hashes);
        builder.push_redactions(&self.redactions);
        builder.into_string()
    }
}

#[derive(Clone, Copy)]
struct SummaryHeader<'a> {
    schema: &'a str,
    status: super::Status,
    issue_code: super::IssueCode,
    operation: super::Operation,
}

struct SummaryBuilder {
    text: alloc::string::String,
}

impl SummaryBuilder {
    fn new() -> Self {
        Self {
            text: alloc::string::String::new(),
        }
    }

    fn push_header(&mut self, header: SummaryHeader<'_>) {
        self.text.push_str(REVIEW_PREFIX);
        self.text.push(REVIEW_SEPARATOR);
        self.text.push_str(header.schema);
        self.text.push(REVIEW_SEPARATOR);
        self.text.push_str(header.status.as_str());
        self.text.push(REVIEW_SEPARATOR);
        self.text.push_str(header.issue_code.as_str());
        self.text.push(REVIEW_SEPARATOR);
        self.text.push_str(header.operation.as_str());
    }

    fn push_labels(&mut self, labels: &[alloc::string::String]) {
        for label in labels {
            self.push_value(label);
        }
    }

    fn push_hashes(&mut self, hashes: &[crate::ArtifactHash]) {
        for hash in hashes {
            self.push_value(hash.as_str());
        }
    }

    fn push_redactions(&mut self, redactions: &[super::Redaction]) {
        for redaction in redactions {
            self.push_value(redaction.as_str());
        }
    }

    fn push_value(&mut self, value: &str) {
        self.text.push(REVIEW_SEPARATOR);
        self.text.push_str(value);
    }

    fn into_string(self) -> alloc::string::String {
        self.text
    }
}
