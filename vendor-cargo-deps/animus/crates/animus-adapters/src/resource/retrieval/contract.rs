/// Retrieval-capable resource class used by budget policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Capability {
    /// Web search over a search provider.
    WebSearch,
    /// Browser automation or page inspection.
    BrowserAutomation,
    /// Documentation lookup such as Context7.
    DocumentationLookup,
    /// Repository browsing or code search.
    RepositoryBrowsing,
    /// Multi-lens analysis over several retrieval paths.
    MultiLensAnalysis,
    /// Property or fact discovery workflow.
    PropertyDiscovery,
    /// Resource does not perform retrieval.
    NonRetrieval,
}

impl Capability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WebSearch => "web_search",
            Self::BrowserAutomation => "browser_automation",
            Self::DocumentationLookup => "documentation_lookup",
            Self::RepositoryBrowsing => "repository_browsing",
            Self::MultiLensAnalysis => "multi_lens_analysis",
            Self::PropertyDiscovery => "property_discovery",
            Self::NonRetrieval => "non_retrieval",
        }
    }

    /// Returns true when policy may require a retrieval budget.
    #[must_use]
    pub const fn is_retrieval_capable(self) -> bool {
        match self {
            Self::WebSearch
            | Self::BrowserAutomation
            | Self::DocumentationLookup
            | Self::RepositoryBrowsing
            | Self::MultiLensAnalysis
            | Self::PropertyDiscovery => true,
            Self::NonRetrieval => false,
        }
    }
}

/// Whether host policy requires budgets for retrieval-capable resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Requirement {
    /// Budget descriptors are optional.
    Optional,
    /// Retrieval-capable resources need a budget descriptor before advertisement.
    Required,
}

/// Host retrieval policy applied to one resource descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    requirement: Requirement,
}

impl Policy {
    /// Builds a retrieval resource policy.
    #[must_use]
    pub const fn new(requirement: Requirement) -> Self {
        Self { requirement }
    }

    /// Returns the budget requirement.
    #[must_use]
    pub const fn requirement(self) -> Requirement {
        self.requirement
    }
}

/// Borrowed retrieval budget fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Purpose of the retrieval.
    pub purpose: Option<&'a str>,
    /// First retrieval action guidance.
    pub first_action: Option<&'a str>,
    /// Conditions that justify repeat retrieval.
    pub repeat_conditions: &'a [&'a str],
    /// Conditions that stop retrieval.
    pub stop_conditions: &'a [&'a str],
    /// Behavior when evidence is missing.
    pub missing_evidence_behavior: Option<&'a str>,
    /// Citation or source-naming expectation.
    pub citation_expectation: Option<&'a str>,
    /// Progress-note guidance.
    pub progress_guidance: Option<&'a str>,
    /// Safe provenance label for the budget.
    pub provenance: Option<&'a str>,
    /// Optional host-configured retrieval count limit.
    pub max_retrievals: Option<u32>,
}

/// Validated retrieval budget descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Budget {
    purpose: alloc::string::String,
    first_action: alloc::string::String,
    repeat_conditions: alloc::vec::Vec<alloc::string::String>,
    stop_conditions: alloc::vec::Vec<alloc::string::String>,
    missing_evidence_behavior: alloc::string::String,
    citation_expectation: alloc::string::String,
    progress_guidance: alloc::string::String,
    provenance: alloc::string::String,
    max_retrievals: Option<core::num::NonZeroU32>,
}

impl Budget {
    /// Validates a retrieval budget descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`super::failure::Error`] when required text is missing,
    /// condition lists are empty or over limit, or the optional limit is zero.
    pub fn validate(draft: &Draft<'_>) -> Result<Self, super::failure::Error> {
        let repeat_conditions = super::check::conditions(super::REPEAT_CONDITION_FIELD, draft.repeat_conditions)?;
        let stop_conditions = super::check::conditions(super::STOP_CONDITION_FIELD, draft.stop_conditions)?;
        Ok(Self {
            purpose: super::check::required_text(super::PURPOSE_FIELD, draft.purpose)?,
            first_action: super::check::required_text(super::FIRST_ACTION_FIELD, draft.first_action)?,
            repeat_conditions,
            stop_conditions,
            missing_evidence_behavior: super::check::required_text(
                super::MISSING_BEHAVIOR_FIELD,
                draft.missing_evidence_behavior,
            )?,
            citation_expectation: super::check::required_text(super::CITATION_FIELD, draft.citation_expectation)?,
            progress_guidance: super::check::required_text(super::PROGRESS_FIELD, draft.progress_guidance)?,
            provenance: super::check::required_text(super::PROVENANCE_FIELD, draft.provenance)?,
            max_retrievals: super::check::limit(draft.max_retrievals)?,
        })
    }

    /// Returns the retrieval purpose.
    #[must_use]
    pub fn purpose(&self) -> &str {
        self.purpose.as_str()
    }

    /// Returns first-action guidance.
    #[must_use]
    pub fn first_action(&self) -> &str {
        self.first_action.as_str()
    }

    /// Returns repeat retrieval conditions.
    #[must_use]
    pub fn repeat_conditions(&self) -> &[alloc::string::String] {
        self.repeat_conditions.as_slice()
    }

    /// Returns stop conditions.
    #[must_use]
    pub fn stop_conditions(&self) -> &[alloc::string::String] {
        self.stop_conditions.as_slice()
    }

    /// Returns missing-evidence behavior.
    #[must_use]
    pub fn missing_evidence_behavior(&self) -> &str {
        self.missing_evidence_behavior.as_str()
    }

    /// Returns citation or source-naming expectation.
    #[must_use]
    pub fn citation_expectation(&self) -> &str {
        self.citation_expectation.as_str()
    }

    /// Returns progress-note guidance.
    #[must_use]
    pub fn progress_guidance(&self) -> &str {
        self.progress_guidance.as_str()
    }

    /// Returns safe provenance.
    #[must_use]
    pub fn provenance(&self) -> &str {
        self.provenance.as_str()
    }

    /// Returns optional retrieval count limit.
    #[must_use]
    pub const fn max_retrievals(&self) -> Option<core::num::NonZeroU32> {
        self.max_retrievals
    }

    /// Builds bounded model-visible prompt guidance for this budget.
    #[must_use]
    pub fn guidance_summary(&self) -> alloc::string::String {
        let mut summary = alloc::string::String::new();
        summary.push_str("first: ");
        summary.push_str(self.first_action());
        summary.push_str("; repeat: ");
        summary.push_str(self.repeat_conditions[0].as_str());
        summary.push_str("; stop: ");
        summary.push_str(self.stop_conditions[0].as_str());
        summary.push_str("; missing: ");
        summary.push_str(self.missing_evidence_behavior());
        super::check::truncate_chars(summary, super::MAX_GUIDANCE_CHARS)
    }

    pub(super) fn repeat_allowed_after(&self, performed: u32) -> bool {
        if let Some(limit) = self.max_retrievals {
            return performed < limit.get();
        }
        true
    }
}

/// Admission result for a retrieval-capable resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    /// Resource can be advertised.
    Advertise,
    /// Resource is unavailable because a required budget is missing.
    MissingBudget,
}

/// Applies retrieval policy to a resource and optional budget descriptor.
///
/// # Errors
///
/// Returns [`super::failure::Error`] when policy requires a budget and the
/// retrieval-capable resource lacks one.
pub fn admit(
    capability: Capability,
    budget: Option<&Budget>,
    policy: Policy,
) -> Result<Admission, super::failure::Error> {
    if !capability.is_retrieval_capable() || budget.is_some() {
        return Ok(Admission::Advertise);
    }
    match policy.requirement() {
        Requirement::Optional => Ok(Admission::Advertise),
        Requirement::Required => Err(super::failure::Error::MissingBudget { capability }),
    }
}

impl Default for Draft<'_> {
    fn default() -> Self {
        Self {
            purpose: Some("answer with bounded evidence"),
            first_action: Some("make one targeted retrieval first"),
            repeat_conditions: &["missing required fact"],
            stop_conditions: &["required evidence is sufficient"],
            missing_evidence_behavior: Some("name the missing evidence"),
            citation_expectation: Some("name sources or code artifacts"),
            progress_guidance: Some("report one short progress note before repeating"),
            provenance: Some("hermes-retrieval-budget"),
            max_retrievals: Some(super::DEFAULT_MAX_RETRIEVALS),
        }
    }
}
