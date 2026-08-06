#![allow(unknown_lints)]
#![allow(non_trait_imports, path_segment_repetition, underscore_in_module_filename)]

use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU32;

use super::GuidanceError;
use super::MAX_SKILL_DESCRIPTOR_ITEMS;
use super::SKILL_DEPENDENCY_FIELD;
use super::SKILL_SOURCE_FIELD;
use super::SkillManifest;
use super::contains_dependency;
use super::normalize;
use super::normalize_list;

/// Skill invocation policy for deferred full-instruction loading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillInvocationPolicy {
    /// Explicit selection is required before full skill loading.
    ExplicitOnly,
    /// Host policy may approve implicit loading.
    ImplicitAllowed,
    /// Skill metadata may be advertised but full loading is disabled.
    Disabled,
}

impl SkillInvocationPolicy {
    /// Returns true when implicit loading can be policy-approved.
    #[must_use]
    pub const fn allows_implicit(self) -> bool {
        matches!(self, Self::ImplicitAllowed)
    }
}

/// Borrowed skill discovery metadata fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillDiscoveryDraft<'a> {
    /// Already validated skill manifest.
    pub manifest: &'a SkillManifest,
    /// Safe source label for this skill descriptor.
    pub source: &'a str,
    /// Invocation policy for deferred full loading.
    pub invocation_policy: SkillInvocationPolicy,
    /// Dependency hints required before full loading.
    pub dependency_hints: &'a [&'a str],
    /// Prompt budget cost for metadata-only discovery.
    pub context_budget_cost: u32,
}

/// Model-visible skill metadata used before full instructions are loaded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillDiscoveryMetadata {
    name: String,
    description: String,
    source: String,
    invocation_policy: SkillInvocationPolicy,
    dependency_hints: Vec<String>,
    context_budget_cost: NonZeroU32,
}

impl SkillDiscoveryMetadata {
    /// Builds bounded metadata from an already validated skill manifest.
    ///
    /// # Errors
    ///
    /// Returns [`GuidanceError`] when source/dependency text is empty or too long,
    /// too many dependency hints are supplied, or the budget cost is zero.
    // r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.skill_metadata]
    pub fn validate(draft: &SkillDiscoveryDraft<'_>) -> Result<Self, GuidanceError> {
        let context_budget_cost =
            NonZeroU32::new(draft.context_budget_cost).ok_or_else(|| GuidanceError::ZeroBudgetCost {
                source: String::from(draft.manifest.name()),
            })?;
        Ok(Self {
            name: String::from(draft.manifest.name()),
            description: String::from(draft.manifest.description()),
            source: normalize(SKILL_SOURCE_FIELD, draft.source)?,
            invocation_policy: draft.invocation_policy,
            dependency_hints: normalize_list(
                SKILL_DEPENDENCY_FIELD,
                draft.dependency_hints,
                MAX_SKILL_DESCRIPTOR_ITEMS,
            )?,
            context_budget_cost,
        })
    }

    /// Returns the skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns metadata-only skill description.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the safe source label.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the invocation policy.
    #[must_use]
    pub const fn invocation_policy(&self) -> SkillInvocationPolicy {
        self.invocation_policy
    }

    /// Returns dependency hints required before loading.
    #[must_use]
    pub fn dependency_hints(&self) -> &[String] {
        self.dependency_hints.as_slice()
    }

    /// Returns initial prompt budget cost for metadata.
    #[must_use]
    pub const fn context_budget_cost(&self) -> NonZeroU32 {
        self.context_budget_cost
    }
}

/// Full-skill loading request mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillInvocationMode {
    /// User or host explicitly selected the skill.
    Explicit,
    /// Host policy is considering an implicit selection.
    Implicit,
}

/// Borrowed request for deferred full-skill loading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillLoadRequest<'a> {
    /// Requested skill name.
    pub name: &'a str,
    /// Invocation mode being evaluated.
    pub mode: SkillInvocationMode,
    /// Dependency identifiers currently available to the host.
    pub available_dependencies: &'a [&'a str],
}

/// Deferred full-skill loading decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillLoadDecision {
    /// Full instructions may be loaded by the host.
    Load,
    /// Full loading is denied with an explicit reason.
    Deny(SkillLoadDenial),
}

/// Deferred full-skill loading denial reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SkillLoadDenial {
    /// No advertised skill has the requested name.
    NotFound,
    /// More than one advertised skill has the requested name.
    AmbiguousName,
    /// Skill is disabled.
    Disabled,
    /// Implicit loading was not allowed by skill policy.
    ImplicitInvocationDenied,
    /// A required dependency hint is unavailable.
    MissingDependency {
        /// Missing dependency identifier.
        dependency: String,
    },
}

/// Decides whether full skill instructions may be loaded.
#[must_use]
// r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.deferred_loading]
pub fn decide_skill_loading(metadata: &[SkillDiscoveryMetadata], request: SkillLoadRequest<'_>) -> SkillLoadDecision {
    let candidate = match unique_skill_candidate(metadata, request.name) {
        Ok(candidate) => candidate,
        Err(denial) => return SkillLoadDecision::Deny(denial),
    };
    skill_policy_denial(candidate, request).map_or(SkillLoadDecision::Load, SkillLoadDecision::Deny)
}

fn unique_skill_candidate<'a>(
    metadata: &'a [SkillDiscoveryMetadata],
    name: &str,
) -> Result<&'a SkillDiscoveryMetadata, SkillLoadDenial> {
    let mut candidates = metadata.iter().filter(|candidate| candidate.name() == name);
    let Some(first_candidate) = candidates.next() else {
        return Err(SkillLoadDenial::NotFound);
    };
    if candidates.next().is_some() {
        return Err(SkillLoadDenial::AmbiguousName);
    }
    Ok(first_candidate)
}

fn skill_policy_denial(candidate: &SkillDiscoveryMetadata, request: SkillLoadRequest<'_>) -> Option<SkillLoadDenial> {
    if candidate.invocation_policy() == SkillInvocationPolicy::Disabled {
        return Some(SkillLoadDenial::Disabled);
    }
    if request.mode == SkillInvocationMode::Implicit && !candidate.invocation_policy().allows_implicit() {
        return Some(SkillLoadDenial::ImplicitInvocationDenied);
    }
    missing_dependency(candidate, request.available_dependencies).map(|dependency| SkillLoadDenial::MissingDependency {
        dependency: String::from(dependency),
    })
}

fn missing_dependency<'a>(candidate: &'a SkillDiscoveryMetadata, available_dependencies: &[&str]) -> Option<&'a str> {
    candidate
        .dependency_hints()
        .iter()
        .map(String::as_str)
        .find(|dependency| !contains_dependency(available_dependencies, dependency))
}
