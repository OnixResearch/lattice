const SOURCE_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_source_kind");
const SOURCE_LABEL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_source_label");
const CONTENT_HASH_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_content_hash");

const SOURCE_FIRST_RUN: &str = "first_run";
const SOURCE_CHANGED: &str = "changed";
const SOURCE_IMPORTED: &str = "imported";
const SOURCE_PROJECT: &str = "project";
const TRUST_ADMITTED: &str = "admitted";
const TRUST_PENDING: &str = "pending";

type Text = alloc::string::String;

/// Untrusted recipe provenance metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeProvenanceDraft<'a> {
    /// Provenance source kind label.
    pub source_kind: &'a str,
    /// Redacted source label.
    pub source_label: &'a str,
    /// BLAKE3 hash of canonical recipe content.
    pub content_hash: &'a str,
    /// Whether host policy or user confirmation admitted this recipe.
    pub admitted: bool,
}

/// Recipe provenance source kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeSourceKind {
    /// First time this recipe identity is observed.
    FirstRun,
    /// Previously seen recipe identity with changed content.
    Changed,
    /// Imported recipe from an external source.
    Imported,
    /// Project-provided recipe checked into the workspace.
    Project,
}

impl RecipeSourceKind {
    /// Parses a stable recipe source kind label.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, super::RecipeWorkflowError> {
        super::parse_workflow_value(SOURCE_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            SOURCE_FIRST_RUN => Some(Self::FirstRun),
            SOURCE_CHANGED => Some(Self::Changed),
            SOURCE_IMPORTED => Some(Self::Imported),
            SOURCE_PROJECT => Some(Self::Project),
            _ => None,
        }
    }

    /// Returns the stable source kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FirstRun => SOURCE_FIRST_RUN,
            Self::Changed => SOURCE_CHANGED,
            Self::Imported => SOURCE_IMPORTED,
            Self::Project => SOURCE_PROJECT,
        }
    }
}

/// Trust admission status for a recipe.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeTrustStatus {
    /// Recipe may influence runtime inputs within its capability ceilings.
    Admitted,
    /// Recipe has no influence until host admission succeeds.
    Pending,
}

impl RecipeTrustStatus {
    fn from_admitted(admitted: bool) -> Self {
        if admitted { Self::Admitted } else { Self::Pending }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => TRUST_ADMITTED,
            Self::Pending => TRUST_PENDING,
        }
    }
}

/// BLAKE3-identified recipe trust receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeTrustReceipt {
    source_kind: RecipeSourceKind,
    source_label: Text,
    content_hash: Text,
    identity_hash: Text,
    status: RecipeTrustStatus,
}

impl RecipeTrustReceipt {
    /// Builds a trust receipt from recipe provenance metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::RecipeWorkflowError`] when source fields are malformed or
    /// the content hash is not lowercase BLAKE3 hex.
    pub fn new(draft: &RecipeProvenanceDraft<'_>) -> Result<Self, super::RecipeWorkflowError> {
        let source_kind = RecipeSourceKind::parse(draft.source_kind)?;
        let source_label = super::required_text(SOURCE_LABEL_FIELD, draft.source_label)?;
        let content_hash = super::required_text(CONTENT_HASH_FIELD, draft.content_hash)?;
        super::digest::validate_blake3_hex(super::digest::HashInput {
            field: CONTENT_HASH_FIELD.as_str(),
            value: content_hash.as_str(),
        })?;
        let status = RecipeTrustStatus::from_admitted(draft.admitted);
        let identity_hash = super::digest::hash_parts(&[
            source_kind.as_str(),
            source_label.as_str(),
            content_hash.as_str(),
            status.as_str(),
        ]);

        Ok(Self {
            source_kind,
            source_label,
            content_hash,
            identity_hash,
            status,
        })
    }

    /// Returns the source kind.
    #[must_use]
    pub const fn source_kind(&self) -> RecipeSourceKind {
        self.source_kind
    }

    /// Returns the redacted source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns the canonical recipe content hash.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }

    /// Returns the trust receipt identity hash.
    #[must_use]
    pub fn identity_hash(&self) -> &str {
        self.identity_hash.as_str()
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn status(&self) -> RecipeTrustStatus {
        self.status
    }
}

/// Runtime influence status assigned after trust evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeInfluenceStatus {
    /// Recipe may influence prompt, tools, model, memory, and workflow permissions.
    Active,
    /// Recipe is rejected or active only as a no-influence failure state.
    NoInfluence,
}

/// Trust-gated recipe influence flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeInfluence {
    status: RecipeInfluenceStatus,
}

impl RecipeInfluence {
    /// Evaluates whether a recipe trust receipt may influence runtime inputs.
    #[must_use]
    pub const fn evaluate(receipt: &RecipeTrustReceipt) -> Self {
        match receipt.status() {
            RecipeTrustStatus::Admitted => Self::active(),
            RecipeTrustStatus::Pending => Self::no_influence(),
        }
    }

    const fn active() -> Self {
        Self {
            status: RecipeInfluenceStatus::Active,
        }
    }

    const fn no_influence() -> Self {
        Self {
            status: RecipeInfluenceStatus::NoInfluence,
        }
    }

    /// Returns the influence status.
    #[must_use]
    pub const fn status(self) -> RecipeInfluenceStatus {
        self.status
    }

    /// Returns whether prompt influence is allowed.
    #[must_use]
    pub const fn prompt_allowed(self) -> bool {
        self.is_active()
    }

    /// Returns whether tool influence is allowed.
    #[must_use]
    pub const fn tools_allowed(self) -> bool {
        self.is_active()
    }

    /// Returns whether model selection influence is allowed.
    #[must_use]
    pub const fn model_allowed(self) -> bool {
        self.is_active()
    }

    /// Returns whether memory influence is allowed.
    #[must_use]
    pub const fn memory_allowed(self) -> bool {
        self.is_active()
    }

    /// Returns whether workflow permission influence is allowed.
    #[must_use]
    pub const fn workflow_permissions_allowed(self) -> bool {
        self.is_active()
    }

    const fn is_active(self) -> bool {
        match self.status {
            RecipeInfluenceStatus::Active => true,
            RecipeInfluenceStatus::NoInfluence => false,
        }
    }
}
