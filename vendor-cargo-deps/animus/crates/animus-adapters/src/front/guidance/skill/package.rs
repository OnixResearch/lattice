#![allow(unknown_lints)]
#![allow(non_trait_imports, path_segment_repetition, underscore_in_module_filename)]

use alloc::string::String;
use alloc::vec::Vec;

use super::GuidanceError;
use super::MAX_SKILL_DESCRIPTOR_ITEMS;
use super::SKILL_ASSET_FIELD;
use super::SKILL_DEPENDENCY_FIELD;
use super::SKILL_REFERENCE_FIELD;
use super::SKILL_SCRIPT_FIELD;
use super::SkillDiscoveryMetadata;
use super::SkillManifestTrustStatus;
use super::normalize_list;

/// Validated skill package descriptor with trust-gated resources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillPackageDescriptor {
    metadata: SkillDiscoveryMetadata,
    instructions_hash: crate::artifact::Hash,
    optional_scripts: Vec<String>,
    references: Vec<String>,
    assets: Vec<String>,
    dependencies: Vec<String>,
    trust_status: SkillManifestTrustStatus,
}

impl SkillPackageDescriptor {
    /// Validates a skill package descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`GuidanceError`] when hashes or resource labels are invalid, or
    /// a resource list exceeds the descriptor bound.
    // r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.skill_package]
    pub fn validate(draft: &SkillPackageDraft<'_>) -> Result<Self, GuidanceError> {
        Ok(Self {
            metadata: draft.metadata.clone(),
            instructions_hash: crate::artifact::Hash::new(draft.instructions_hash)
                .map_err(GuidanceError::InvalidHash)?,
            optional_scripts: normalize_list(SKILL_SCRIPT_FIELD, draft.optional_scripts, MAX_SKILL_DESCRIPTOR_ITEMS)?,
            references: normalize_list(SKILL_REFERENCE_FIELD, draft.references, MAX_SKILL_DESCRIPTOR_ITEMS)?,
            assets: normalize_list(SKILL_ASSET_FIELD, draft.assets, MAX_SKILL_DESCRIPTOR_ITEMS)?,
            dependencies: normalize_list(SKILL_DEPENDENCY_FIELD, draft.dependencies, MAX_SKILL_DESCRIPTOR_ITEMS)?,
            trust_status: draft.trust_status,
        })
    }

    /// Returns metadata advertised before full loading.
    #[must_use]
    pub const fn metadata(&self) -> &SkillDiscoveryMetadata {
        &self.metadata
    }

    /// Returns the full instructions content hash.
    #[must_use]
    pub const fn instructions_hash(&self) -> &crate::artifact::Hash {
        &self.instructions_hash
    }

    /// Returns optional script resource labels.
    #[must_use]
    pub fn optional_scripts(&self) -> &[String] {
        self.optional_scripts.as_slice()
    }

    /// Returns reference resource labels.
    #[must_use]
    pub fn references(&self) -> &[String] {
        self.references.as_slice()
    }

    /// Returns asset resource labels.
    #[must_use]
    pub fn assets(&self) -> &[String] {
        self.assets.as_slice()
    }

    /// Returns declared dependency resource labels.
    #[must_use]
    pub fn dependencies(&self) -> &[String] {
        self.dependencies.as_slice()
    }

    /// Returns package trust status.
    #[must_use]
    pub const fn trust_status(&self) -> SkillManifestTrustStatus {
        self.trust_status
    }
}

/// Borrowed skill package descriptor fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillPackageDraft<'a> {
    /// Metadata already admitted for progressive disclosure.
    pub metadata: &'a SkillDiscoveryMetadata,
    /// Lowercase BLAKE3 identity for the full instructions body.
    pub instructions_hash: &'a str,
    /// Optional script resource labels.
    pub optional_scripts: &'a [&'a str],
    /// Reference resource labels.
    pub references: &'a [&'a str],
    /// Asset resource labels.
    pub assets: &'a [&'a str],
    /// Dependency resource labels.
    pub dependencies: &'a [&'a str],
    /// Host trust status for the package source.
    pub trust_status: SkillManifestTrustStatus,
}

/// Runtime-core denial reason for skill script execution attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillScriptDenial {
    /// Package has script resources but the source is not trusted.
    UntrustedPackage,
    /// Runtime core never executes package scripts directly.
    RuntimeCoreDoesNotExecuteScripts,
}

/// Denies direct runtime-core execution of skill package scripts.
///
/// # Errors
///
/// Always returns a denial; trusted scripts must be routed through host shell,
/// hook, and tool policy rather than the reusable runtime core.
// r[impl agent_harness_runtime.openai_codex_guidance_skill_budget.skill_package]
pub fn admit_runtime_core_skill_script(package: &SkillPackageDescriptor) -> Result<(), SkillScriptDenial> {
    if !package.optional_scripts().is_empty() && package.trust_status() != SkillManifestTrustStatus::Trusted {
        return Err(SkillScriptDenial::UntrustedPackage);
    }
    Err(SkillScriptDenial::RuntimeCoreDoesNotExecuteScripts)
}
