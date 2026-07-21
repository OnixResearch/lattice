/// Version-control-neutral change-set selector kinds for review runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiffReviewSelectorKind {
    /// Review current working-copy changes.
    WorkingCopy,
    /// Review one commit or revision identifier.
    Commit,
    /// Review changes between two branch or revision labels.
    BranchComparison,
    /// Review an externally supplied patch payload.
    ExternalPatch,
}

impl DiffReviewSelectorKind {
    /// Returns the stable selector label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WorkingCopy => "working_copy",
            Self::Commit => "commit",
            Self::BranchComparison => "branch_comparison",
            Self::ExternalPatch => "external_patch",
        }
    }
}

/// Borrowed selector shape before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewSelectorDraft<'a> {
    /// Selector kind requested by the caller.
    pub kind: DiffReviewSelectorKind,
    /// Primary revision, branch, or external patch label when required.
    pub primary_ref: Option<&'a str>,
    /// Secondary revision or branch label when required.
    pub secondary_ref: Option<&'a str>,
}

/// Validated change-set selector for a review run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewSelector {
    kind: DiffReviewSelectorKind,
    primary_ref: Option<super::Text>,
    secondary_ref: Option<super::Text>,
}

impl DiffReviewSelector {
    /// Validates a borrowed selector without reading a repository.
    ///
    /// # Errors
    ///
    /// Returns [`super::DiffReviewError`] when required selector references are
    /// missing, extra references are present, or text fields are malformed.
    pub fn validate(draft: &DiffReviewSelectorDraft<'_>) -> Result<Self, super::DiffReviewError> {
        let primary_ref =
            super::check::optional_text(super::SELECTOR_REF_FIELD, draft.primary_ref)?;
        let secondary_ref =
            super::check::optional_text(super::SELECTOR_REF_FIELD, draft.secondary_ref)?;
        validate_selector_shape(draft.kind, primary_ref.as_ref(), secondary_ref.as_ref())?;
        Ok(Self {
            kind: draft.kind,
            primary_ref,
            secondary_ref,
        })
    }

    /// Returns the selector kind.
    #[must_use]
    pub const fn kind(&self) -> DiffReviewSelectorKind {
        self.kind
    }

    /// Returns the normalized primary reference when present.
    #[must_use]
    pub fn primary_ref(&self) -> Option<&str> {
        self.primary_ref.as_deref()
    }

    /// Returns the normalized secondary reference when present.
    #[must_use]
    pub fn secondary_ref(&self) -> Option<&str> {
        self.secondary_ref.as_deref()
    }
}

/// Borrowed review run request before adapter-capability checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiffReviewRequestDraft<'a> {
    /// Stable review run identity supplied by the host shell.
    pub run_id: &'a str,
    /// Requested change-set selector.
    pub selector: DiffReviewSelectorDraft<'a>,
    /// Selector kinds supported by the active adapter.
    pub adapter_capabilities: &'a [DiffReviewSelectorKind],
    /// Bounded source labels included in the review scope.
    pub target_labels: &'a [&'a str],
}

/// Validated dedicated diff-review run request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReviewRequest {
    run_id: super::Text,
    selector: DiffReviewSelector,
    target_labels: super::Collection<super::Text>,
}

impl DiffReviewRequest {
    /// Validates a review request before model or tool calls start.
    ///
    /// # Errors
    ///
    /// Returns [`super::DiffReviewError`] when text fields are malformed, the selector
    /// shape is invalid, the active adapter lacks the requested selector
    /// capability, or the request lacks a bounded review target.
    pub fn validate(draft: &DiffReviewRequestDraft<'_>) -> Result<Self, super::DiffReviewError> {
        let run_id = super::check::required_text(super::RUN_ID_FIELD, draft.run_id)?;
        let selector = DiffReviewSelector::validate(&draft.selector)?;
        if !draft.adapter_capabilities.contains(&selector.kind()) {
            return Err(super::DiffReviewError::UnsupportedSelector {
                kind: selector.kind(),
            });
        }
        let target_labels = super::check::text_collection(
            super::TARGET_LABEL_FIELD,
            draft.target_labels,
            super::check::CollectionPolicy {
                required: true,
                max_items: super::MAX_TARGET_LABELS,
            },
        )?;
        Ok(Self {
            run_id,
            selector,
            target_labels,
        })
    }

    /// Returns the stable review run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns the validated change-set selector.
    #[must_use]
    pub const fn selector(&self) -> &DiffReviewSelector {
        &self.selector
    }

    /// Returns bounded source labels included in the review scope.
    #[must_use]
    pub fn target_labels(&self) -> &[super::Text] {
        self.target_labels.as_slice()
    }
}

fn validate_selector_shape(
    kind: DiffReviewSelectorKind,
    primary_ref: Option<&super::Text>,
    secondary_ref: Option<&super::Text>,
) -> Result<(), super::DiffReviewError> {
    let is_valid = match kind {
        DiffReviewSelectorKind::WorkingCopy => primary_ref.is_none() && secondary_ref.is_none(),
        DiffReviewSelectorKind::Commit | DiffReviewSelectorKind::ExternalPatch => {
            primary_ref.is_some() && secondary_ref.is_none()
        }
        DiffReviewSelectorKind::BranchComparison => {
            primary_ref.is_some() && secondary_ref.is_some()
        }
    };
    if is_valid {
        Ok(())
    } else {
        Err(super::DiffReviewError::InvalidSelectorShape { kind })
    }
}
