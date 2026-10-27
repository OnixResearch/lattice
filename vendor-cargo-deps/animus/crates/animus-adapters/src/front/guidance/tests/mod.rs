#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    non_trait_imports,
    path_segment_repetition
)]

use super::*;
use core::num::NonZeroU32;

mod assembly;
mod skills;

pub(super) const ROOT_NAME: &str = "root-guidance";
pub(super) const NESTED_NAME: &str = "nested-guidance";
pub(super) const REDACTED_NAME: &str = "redacted-guidance";
pub(super) const ORIGIN_ROOT: &str = "repo-root-agents";
pub(super) const ORIGIN_NESTED: &str = "repo-nested-agents";
pub(super) const ORIGIN_REDACTED: &str = "repo-secret-agents";
pub(super) const VALID_HASH: &str =
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const SECOND_HASH: &str =
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub(super) const THIRD_HASH: &str =
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
pub(super) const ROOT_PRECEDENCE: GuidancePrecedence = GuidancePrecedence::new(10);
pub(super) const NESTED_PRECEDENCE: GuidancePrecedence = GuidancePrecedence::new(20);
pub(super) const REDACTED_PRECEDENCE: GuidancePrecedence = GuidancePrecedence::new(30);
pub(super) const ROOT_UNITS: u32 = 3;
pub(super) const NESTED_UNITS: u32 = 4;
pub(super) const REDACTED_UNITS: u32 = 2;
pub(super) const FULL_BUDGET_UNITS: u32 = 12;
pub(super) const SMALL_BUDGET_UNITS: u32 = 5;
pub(super) const EXPECTED_TWO_ITEMS: usize = 2;
pub(super) const EXPECTED_THREE_ITEMS: usize = 3;
pub(super) const SKILL_NAME: &str = "skill-authoring";
pub(super) const SKILL_DESCRIPTION: &str = "Helps assemble focused agent guidance";
pub(super) const SKILL_VERSION: &str = "1.0.0";
pub(super) const SKILL_CATEGORY: &str = "workflow";
pub(super) const SKILL_TAG: &str = "guidance";
pub(super) const SKILL_SOURCE: &str = "bundle:codex-reference";
pub(super) const SKILL_DEPENDENCY: &str = "model-context";
pub(super) const MISSING_DEPENDENCY: &str = "missing-tool";
pub(super) const SCRIPT_LABEL: &str = "scripts/check.sh";
pub(super) const REFERENCE_LABEL: &str = "references/style.md";
pub(super) const ASSET_LABEL: &str = "assets/logo.svg";
pub(super) const SELECTED_BODY: &str = "# Skill\n\nFull instructions stay deferred.";
pub(super) const METADATA_BUDGET_UNITS: u32 = 1;

pub(super) fn source(
    name: &'static str,
    scope: PromptResourceScope,
    precedence: GuidancePrecedence,
    origin: &'static str,
) -> GuidanceSource {
    source_with(
        name, scope, precedence, origin, VALID_HASH, ROOT_UNITS, false,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "fixture mirrors guidance source fields"
)]
pub(super) fn source_with(
    name: &'static str,
    scope: PromptResourceScope,
    precedence: GuidancePrecedence,
    origin: &'static str,
    content_hash: &'static str,
    budget_units: u32,
    redacted: bool,
) -> GuidanceSource {
    GuidanceSource::validate(&GuidanceSourceDraft {
        name,
        scope,
        precedence,
        override_semantics: GuidanceOverride::Extend,
        origin,
        content_hash,
        budget_units,
        redacted,
    })
    .expect("guidance source should validate")
}

pub(super) fn build_metadata(
    manifest: &SkillManifest,
    invocation_policy: SkillInvocationPolicy,
    dependency_hints: &'static [&'static str],
) -> SkillDiscoveryMetadata {
    SkillDiscoveryMetadata::validate(&SkillDiscoveryDraft {
        manifest,
        source: SKILL_SOURCE,
        invocation_policy,
        dependency_hints,
        context_budget_cost: METADATA_BUDGET_UNITS,
    })
    .expect("skill metadata should validate")
}

pub(super) fn manifest(trust_status: SkillManifestTrustStatus) -> SkillManifest {
    SkillManifest::validate(&SkillManifestDraft {
        name: Some(SKILL_NAME),
        description: Some(SKILL_DESCRIPTION),
        version: Some(SKILL_VERSION),
        category: Some(SKILL_CATEGORY),
        tags: &[SKILL_TAG],
        related_skills: &[],
        provenance: Some(SKILL_SOURCE),
        trust_status: Some(trust_status.as_str()),
        content_identity: Some(VALID_HASH),
    })
    .expect("skill manifest should validate")
}

pub(super) fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("test budgets use non-zero values")
}
