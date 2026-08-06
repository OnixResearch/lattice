/// Evidence kinds that a review finding may attempt to carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewEvidenceKind {
    /// Bounded file-span snippet.
    FileSpan,
    /// Bounded patch hunk snippet.
    PatchHunk,
    /// Bounded externally supplied input snippet.
    ExternalInput,
    /// Full file payload, rejected by default.
    FullFile,
    /// Unbounded log payload, rejected by default.
    UnboundedLog,
    /// Context unrelated to the reviewed change set, rejected by default.
    UnrelatedContext,
}

/// Borrowed review evidence before validation and hashing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewEvidenceDraft<'a> {
    /// Evidence kind.
    pub kind: DiffReviewEvidenceKind,
    /// Source or hunk label.
    pub source_label: &'a str,
    /// Bounded evidence snippet.
    pub snippet: &'a str,
    /// Number of evidence lines represented by the snippet.
    pub line_count: u32,
    /// Number of bytes represented by the snippet.
    pub byte_count: u64,
}

/// Validated bounded evidence for a review finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewEvidence {
    kind: DiffReviewEvidenceKind,
    source_label: super::super::Text,
    snippet: super::super::Text,
    line_count: u32,
    byte_count: u64,
    snippet_hash: super::super::Text,
}

impl DiffReviewEvidence {
    /// Validates and hashes bounded review evidence.
    ///
    /// # Errors
    ///
    /// Returns [`super::super::DiffReviewError`] when evidence attempts to embed full
    /// files, unbounded logs, unrelated context, unsafe raw material, or snippets
    /// that exceed review report bounds.
    pub fn validate(draft: &DiffReviewEvidenceDraft<'_>) -> Result<Self, super::super::DiffReviewError> {
        super::super::check::evidence_kind(draft.kind)?;
        super::super::check::evidence_bounds(super::super::check::EvidenceBounds {
            line_count: draft.line_count,
            byte_count: draft.byte_count,
            snippet: draft.snippet,
        })?;
        let source_label =
            super::super::check::required_safe_text(super::super::SOURCE_LABEL_FIELD, draft.source_label)?;
        let snippet = super::super::check::required_safe_text(super::super::EVIDENCE_SNIPPET_FIELD, draft.snippet)?;
        let snippet_hash = super::super::hash_parts(&[
            super::super::HASH_SCHEMA,
            super::super::EVIDENCE_SNIPPET_FIELD.as_str(),
            source_label.as_str(),
            snippet.as_str(),
        ]);
        Ok(Self {
            kind: draft.kind,
            source_label,
            snippet,
            line_count: draft.line_count,
            byte_count: draft.byte_count,
            snippet_hash,
        })
    }

    /// Returns the evidence kind.
    #[must_use]
    pub const fn kind(&self) -> DiffReviewEvidenceKind {
        self.kind
    }

    /// Returns the evidence source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the bounded snippet.
    #[must_use]
    pub fn snippet(&self) -> &str {
        self.snippet.as_str()
    }

    /// Returns the represented evidence line count.
    #[must_use]
    pub const fn line_count(&self) -> u32 {
        self.line_count
    }

    /// Returns the represented evidence byte count.
    #[must_use]
    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    /// Returns the BLAKE3 snippet identity.
    #[must_use]
    pub fn snippet_hash(&self) -> &str {
        self.snippet_hash.as_str()
    }
}
