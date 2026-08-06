use super::CAPABILITY_FIELD;
use super::CapabilitySetCheck;
use super::Collection;
use super::SubagentError;
use super::Text;
use super::contains_all;
use super::text_collection;

/// Capability intersection input for a subagent request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityInput<'a> {
    /// Capabilities allowed by parent policy.
    pub parent_policy: &'a [&'a str],
    /// Capabilities allowed by recipe or profile ceilings.
    pub recipe_or_profile: &'a [&'a str],
    /// Capabilities requested by the subagent delegation.
    pub request: &'a [&'a str],
    /// Capabilities allowed by workspace scope.
    pub workspace_scope: &'a [&'a str],
    /// Capabilities admitted by host policy.
    pub host_admission: &'a [&'a str],
}

/// Denied subagent capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityDenial {
    capability: Text,
}

impl SubagentCapabilityDenial {
    /// Returns the denied capability label.
    #[must_use]
    pub fn capability(&self) -> &str {
        self.capability.as_str()
    }
}

/// Effective subagent capability intersection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityResolution {
    effective: Collection<Text>,
    denied: Collection<SubagentCapabilityDenial>,
}

impl SubagentCapabilityResolution {
    /// Returns capabilities admitted by every input set.
    #[must_use]
    pub fn effective(&self) -> &[Text] {
        self.effective.as_slice()
    }

    /// Returns requested capabilities denied by at least one input set.
    #[must_use]
    pub fn denied(&self) -> &[SubagentCapabilityDenial] {
        self.denied.as_slice()
    }
}

/// Resolves effective subagent capabilities as a pure intersection.
///
/// # Errors
///
/// Returns [`SubagentError`] when a capability label is malformed or duplicated.
///
/// # Panics
///
/// Panics if normalized required capability requests become empty after passing
/// required collection validation.
pub fn resolve_subagent_capabilities(
    input: SubagentCapabilityInput<'_>,
) -> Result<SubagentCapabilityResolution, SubagentError> {
    let parent_policy = text_collection(CAPABILITY_FIELD, input.parent_policy, true)?;
    let recipe_or_profile = text_collection(CAPABILITY_FIELD, input.recipe_or_profile, true)?;
    let request = text_collection(CAPABILITY_FIELD, input.request, true)?;
    assert!(!request.is_empty(), "capability request is required");
    assert!(request.len() <= input.request.len(), "normalized capabilities are bounded");
    let workspace_scope = text_collection(CAPABILITY_FIELD, input.workspace_scope, true)?;
    let host_admission = text_collection(CAPABILITY_FIELD, input.host_admission, true)?;
    let request_count = request.len();
    let mut effective = Collection::with_capacity(request_count);
    let mut denied = Collection::with_capacity(request_count);
    for capability in request {
        if contains_all(CapabilitySetCheck {
            capability: capability.as_str(),
            parent_policy: &parent_policy,
            recipe_or_profile: &recipe_or_profile,
            workspace_scope: &workspace_scope,
            host_admission: &host_admission,
        }) {
            effective.push(capability);
        } else {
            denied.push(SubagentCapabilityDenial { capability });
        }
    }
    Ok(SubagentCapabilityResolution { effective, denied })
}
