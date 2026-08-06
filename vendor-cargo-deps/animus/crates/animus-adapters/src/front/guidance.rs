#![allow(unknown_lints)]
#![allow(non_trait_imports, path_segment_repetition, underscore_in_module_filename)]

#[allow(
    assertion_density,
    non_trait_imports,
    path_segment_repetition,
    raw_arithmetic_overflow
)]
mod assembly;
#[path = "guidance/skill/metadata.rs"]
mod skill_metadata;
#[path = "guidance/skill/package.rs"]
mod skill_package;
#[allow(non_trait_imports, path_segment_repetition)]
mod source;

#[cfg(test)]
mod tests;

use alloc::string::String;
use alloc::vec::Vec;

pub use assembly::assemble_guidance_sources;
pub use skill_metadata::SkillDiscoveryDraft;
pub use skill_metadata::SkillDiscoveryMetadata;
pub use skill_metadata::SkillInvocationMode;
pub use skill_metadata::SkillInvocationPolicy;
pub use skill_metadata::SkillLoadDecision;
pub use skill_metadata::SkillLoadDenial;
pub use skill_metadata::SkillLoadRequest;
pub use skill_metadata::decide_skill_loading;
pub use skill_package::SkillPackageDescriptor;
pub use skill_package::SkillPackageDraft;
pub use skill_package::SkillScriptDenial;
pub use skill_package::admit_runtime_core_skill_script;
pub use source::GuidanceAssembly;
pub use source::GuidanceBudget;
pub use source::GuidanceDisposition;
pub use source::GuidanceLedgerEntry;
pub use source::GuidanceLedgerReason;
pub use source::GuidanceOverride;
pub use source::GuidancePrecedence;
pub use source::GuidanceSource;
pub use source::GuidanceSourceDraft;

/// Descriptor-side effects that skill manifest validation explicitly does not perform.
pub const PROHIBITED_SKILL_MANIFEST_EFFECTS: &[&str] = crate::skill::PROHIBITED_MANIFEST_EFFECTS;
/// Character length of a lowercase BLAKE3 skill content identity.
pub const SKILL_CONTENT_HASH_HEX_CHARS: usize = crate::skill::CONTENT_HASH_HEX_CHARS;
/// Descriptor-side effects that generic resource discovery explicitly does not perform.
pub const PROHIBITED_PROMPT_RESOURCE_EFFECTS: &[&str] = crate::resource::PROHIBITED_DISCOVERY_EFFECTS;
/// Borrowed full-body content explicitly supplied by a host load decision.
pub type SkillBodyDraft<'a> = crate::skill::BodyDraft<'a>;
/// Progressive skill disclosure context for prompt construction.
pub type SkillDisclosureContext = crate::skill::Context;
/// Inputs for skill progressive disclosure.
pub type SkillDisclosureRequest<'a> = crate::skill::Request<'a>;
/// Loaded skill body selected for prompt construction.
pub type SkillLoadedBody = crate::skill::LoadedBody;
/// Validated skill manifest descriptor.
pub type SkillManifest = crate::skill::Descriptor;
/// Borrowed host-supplied skill manifest fields before validation.
pub type SkillManifestDraft<'a> = crate::skill::Draft<'a>;
/// Skill manifest validation error.
pub type SkillManifestError = crate::skill::Error;
/// Host trust status for a skill manifest source.
pub type SkillManifestTrustStatus = crate::skill::TrustStatus;
/// Skill resource advertisement catalog.
pub type SkillResourceCatalog = crate::skill::Catalog;
/// Safe diagnostic emitted while building skill resource summaries.
pub type SkillResourceDiagnostic = crate::skill::Diagnostic;
/// Skill catalog issue class.
pub type SkillResourceIssue = crate::skill::Issue;
/// Model-visible skill summary that omits full body content.
pub type SkillResourceSummary = crate::skill::Summary;
/// Deterministic prompt resource catalog.
pub type PromptResourceCatalog = crate::resource::Catalog;
/// Borrowed full-content payload explicitly supplied by a host load decision.
pub type PromptResourceContentDraft<'a> = crate::resource::ContentDraft<'a>;
/// Progressive prompt resource context.
pub type PromptResourceContext = crate::resource::Context;
/// Safe diagnostic emitted while building resource summaries.
pub type PromptResourceDiagnostic = crate::resource::Diagnostic;
/// Validated host-provided resource descriptor.
pub type PromptResourceDescriptor = crate::resource::Descriptor;
/// Borrowed host-supplied resource fields before validation.
pub type PromptResourceDraft<'a> = crate::resource::Draft<'a>;
/// Prompt resource validation error.
pub type PromptResourceError = crate::resource::Error;
/// Resource catalog issue class.
pub type PromptResourceIssue = crate::resource::Issue;
/// Resource descriptor kind.
pub type PromptResourceKind = crate::resource::Kind;
/// Inputs for prompt resource progressive disclosure.
pub type PromptResourceRequest<'a> = crate::resource::Request<'a>;
/// Resource source scope.
pub type PromptResourceScope = crate::resource::Scope;
/// Model-visible resource summary that omits full content.
pub type PromptResourceSummary = crate::resource::Summary;
/// Host trust status for a resource descriptor.
pub type PromptResourceTrustStatus = crate::resource::TrustStatus;

/// Builds deterministic resource summaries from already validated skill manifests.
#[must_use]
pub fn advertise_skill_resources(manifests: &[SkillManifest]) -> SkillResourceCatalog {
    crate::skill::advertise(manifests)
}

/// Builds progressive skill disclosure context without loading unselected bodies.
#[must_use]
pub fn build_skill_disclosure_context(request: SkillDisclosureRequest<'_>) -> SkillDisclosureContext {
    crate::skill::build_context(request)
}

/// Builds deterministic summaries from already validated prompt resources.
#[must_use]
pub fn advertise_prompt_resources(descriptors: &[PromptResourceDescriptor]) -> PromptResourceCatalog {
    crate::resource::advertise(descriptors)
}

/// Builds progressive prompt resource context without loading unselected bodies.
#[must_use]
pub fn build_prompt_resource_context(request: PromptResourceRequest<'_>) -> PromptResourceContext {
    crate::resource::build_context(request)
}

const GUIDANCE_NAME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("guidance_name");
const GUIDANCE_ORIGIN_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("guidance_origin");
const SKILL_SOURCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_source");
const SKILL_DEPENDENCY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_dependency");
const SKILL_SCRIPT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_script");
const SKILL_REFERENCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_reference");
const SKILL_ASSET_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("skill_asset");
const MAX_GUIDANCE_SOURCES: usize = 32;
const MAX_SKILL_DESCRIPTOR_ITEMS: usize = 32;

const _: () = assert!(MAX_GUIDANCE_SOURCES > 0);
const _: () = assert!(MAX_SKILL_DESCRIPTOR_ITEMS > 0);

/// Guidance and skill context-budget validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuidanceError {
    /// A text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 content hash was invalid.
    InvalidHash(crate::artifact::Error),
    /// A bounded resource list exceeded the allowed item count.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A guidance source declared zero budget units.
    ZeroBudgetCost {
        /// Source associated with the invalid cost.
        source: String,
    },
    /// Two non-redacted guidance sources share the same precedence key.
    DuplicateGuidancePrecedence {
        /// Duplicated source scope.
        scope: PromptResourceScope,
        /// Duplicated precedence rank.
        precedence: GuidancePrecedence,
    },
}

impl core::fmt::Display for GuidanceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(formatter, "{field} supports at most {max_items} items; got {actual_items}"),
            Self::ZeroBudgetCost { source } => {
                write!(formatter, "{source} must use non-zero budget units")
            }
            Self::DuplicateGuidancePrecedence { scope, precedence } => {
                write!(formatter, "guidance scope {} has duplicate precedence {}", scope.as_str(), precedence.rank())
            }
        }
    }
}

impl From<crate::AdapterContractError> for GuidanceError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

fn normalize(field: crate::validation::FieldName, value: &str) -> Result<String, GuidanceError> {
    crate::validation::normalized_adapter_field(field, value).map_err(GuidanceError::Contract)
}

fn normalize_list(
    field: crate::validation::FieldName,
    values: &[&str],
    max_items: usize,
) -> Result<Vec<String>, GuidanceError> {
    if values.len() > max_items {
        return Err(GuidanceError::TooManyItems {
            field: field.as_str(),
            max_items,
            actual_items: values.len(),
        });
    }
    let mut normalized = Vec::with_capacity(values.len());
    for value in values {
        normalized.push(normalize(field, value)?);
    }
    Ok(normalized)
}

fn contains_dependency(available_dependencies: &[&str], required: &str) -> bool {
    available_dependencies.contains(&required)
}
