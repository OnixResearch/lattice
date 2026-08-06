//! Pure per-model prompt, skill, memory, and native-tool composition.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::types::MODEL_INVOCATION_PLAN_HASH_DOMAIN;
use super::types::ModelMemoryAccessMode;
use super::types::ModelNamedResourceReference;
use super::types::ModelPromptSourceReference;
use super::types::ModelRuntimeProfileError;
use super::types::ModelRuntimeProfileIssue;
use super::types::ModelRuntimeTarget;
use super::types::ModelSkillReference;
use super::types::ModelSkillResourcePosture;
use super::types::NativeToolCallingMode;
use super::types::ResolvedModelRuntimeProfile;
use super::validation::ModelProfileField;
use super::validation::blake3_parts;
use super::validation::error;
use super::validation::lowercase_blake3;
use super::validation::safe_label;

const CAPABILITY_PLAN_HASH_DOMAIN: &str = "animus.model-capability-plan.v1";
const MEMORY_PLAN_HASH_DOMAIN: &str = "animus.model-memory-plan.v1";
const PROMPT_PLAN_HASH_DOMAIN: &str = "animus.model-prompt-plan.v1";
const SKILL_PLAN_HASH_DOMAIN: &str = "animus.model-skill-plan.v1";
const TOOL_PLAN_HASH_DOMAIN: &str = "animus.model-tool-plan.v1";
const FIELD_COMPOSITION: &str = "composition";
const FIELD_AUTHORITY: &str = "authority";
const FIELD_PROMPT: &str = "prompt";
const FIELD_MEMORY: &str = "memory";
const FIELD_SKILL: &str = "skill";
const FIELD_TOOL: &str = "tool";
const FIELD_CATALOG_IDENTITY: &str = "tool_catalog_identity";
const NO_REASON_LABEL: &str = "none";

/// One independent narrowing ceiling used during composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelResourceCeiling<'a> {
    /// Stable ceiling label, such as host-policy, grant, agent, or project.
    pub ceiling_label: &'a str,
    /// Exact `provider:model` targets admitted by this ceiling.
    pub model_targets: &'a [&'a str],
    /// Prompt-mode references admitted by this ceiling.
    pub prompt_modes: &'a [&'a str],
    /// Prompt-source labels admitted by this ceiling.
    pub prompt_sources: &'a [&'a str],
    /// Skill names admitted by this ceiling.
    pub skills: &'a [&'a str],
    /// Existing memory-policy identities admitted by this ceiling.
    pub memory_policy_hashes: &'a [&'a str],
    /// Tool names admitted by this ceiling.
    pub tools: &'a [&'a str],
    /// Tool capability labels admitted by this ceiling.
    pub tool_capabilities: &'a [&'a str],
}

/// Active policy and verified-grant inputs for pure composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelInvocationAuthority<'a> {
    /// Independent ceilings whose exact intersection is effective.
    pub ceilings: &'a [ModelResourceCeiling<'a>],
    /// Tool names with current approval evidence.
    pub approved_tools: &'a [&'a str],
    /// Active BLAKE3 policy identity.
    pub policy_hash: &'a str,
}

/// Existing skill validation and selection inputs supplied by the host.
#[derive(Clone, Copy, Debug)]
pub struct ModelSkillCompositionResource<'a> {
    /// Validated skill manifest.
    pub manifest: &'a animus_adapters::SkillManifest,
    /// Body loaded through existing progressive disclosure, if selected.
    pub selected_body: Option<&'a animus_adapters::SkillLoadedBody>,
    /// Profile identity bound by the selection decision.
    pub selection_profile_hash: Option<&'a str>,
    /// Target identity bound by the selection decision.
    pub selection_target_hash: Option<&'a str>,
    /// Existing invocation policy admitted the selected body.
    pub invocation_admitted: bool,
    /// Existing dependency validation admitted the selected body.
    pub dependencies_admitted: bool,
    /// Existing skill budget admitted the selected body.
    pub budget_admitted: bool,
}

/// Borrowed inputs for one pure invocation composition decision.
#[derive(Clone, Copy)]
pub struct ModelInvocationCompositionInput<'a> {
    /// Resolved profile and exact target binding.
    pub resolved: &'a ResolvedModelRuntimeProfile,
    /// Selected model capability matrix.
    pub capabilities: &'a crate::ProviderModelCapabilityMatrix,
    /// Existing capability-negotiation decision.
    pub capability_decision: &'a crate::ModelCapabilityNegotiationDecision,
    /// Validated prompt sources supplied by the host.
    pub prompt_sources: &'a [animus_adapters::PromptSource],
    /// Existing deterministic prompt assembly policy.
    pub prompt_policy: &'a animus_adapters::PromptAssemblyPolicy,
    /// Existing validated skill resources and explicit selections.
    pub skills: &'a [ModelSkillCompositionResource<'a>],
    /// Existing validated opt-in memory policy, when profile access is enabled.
    pub memory_policy: Option<&'a animus_adapters::OptInMemoryPolicy>,
    /// Trusted executor-bound tool catalog.
    pub tool_catalog: &'a crate::tool_executor_kit::ToolCatalog,
    /// Explicit executor readiness observations.
    pub executor_bindings: &'a [crate::tool_executor_kit::ToolExecutorBinding],
    /// BLAKE3 identity of the trusted tool catalog snapshot.
    pub tool_catalog_identity: &'a str,
    /// Active authority ceilings and policy identity.
    pub authority: ModelInvocationAuthority<'a>,
}

/// Omitted-resource class recorded without raw content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelInvocationOmissionClass {
    /// Optional prompt source was not admitted.
    Prompt,
    /// Optional skill resource or selected body was not admitted.
    Skill,
    /// Optional tool declaration was not admitted.
    Tool,
    /// Native tool calls were unavailable in optional mode.
    NativeToolCalling,
}

impl ModelInvocationOmissionClass {
    /// Returns the stable omission class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::Skill => "skill",
            Self::Tool => "tool",
            Self::NativeToolCalling => "native_tool_calling",
        }
    }
}

/// Safe bounded composition diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInvocationDiagnostic {
    class: ModelInvocationOmissionClass,
    resource_label: String,
    reason: String,
}

impl ModelInvocationDiagnostic {
    /// Returns the omitted-resource class.
    #[must_use]
    pub const fn class(&self) -> ModelInvocationOmissionClass {
        self.class
    }

    /// Returns the safe resource label.
    #[must_use]
    pub fn resource_label(&self) -> &str {
        self.resource_label.as_str()
    }

    /// Returns the stable omission reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }
}

/// One admitted per-model skill summary and optional transient body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelSkillPlanEntry {
    name: String,
    description: String,
    content_identity: String,
    posture: ModelSkillResourcePosture,
    selected_body: Option<animus_adapters::SkillLoadedBody>,
}

impl ModelSkillPlanEntry {
    /// Returns the exact skill name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the bounded model-visible summary.
    #[must_use]
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Returns the BLAKE3 full-body identity.
    #[must_use]
    pub fn content_identity(&self) -> &str {
        self.content_identity.as_str()
    }

    /// Returns the configured disclosure posture.
    #[must_use]
    pub const fn posture(&self) -> ModelSkillResourcePosture {
        self.posture
    }

    /// Returns a separately admitted transient body, when selected.
    #[must_use]
    pub const fn selected_body(&self) -> Option<&animus_adapters::SkillLoadedBody> {
        self.selected_body.as_ref()
    }
}

/// One canonical model-visible tool declaration with an exact executor binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelToolDeclaration {
    definition: animus_adapters::tool::Definition,
    executor_binding: String,
}

impl ModelToolDeclaration {
    /// Returns validated implementation-free tool metadata.
    #[must_use]
    pub const fn definition(&self) -> &animus_adapters::tool::Definition {
        &self.definition
    }

    /// Returns the exact executor binding label.
    #[must_use]
    pub fn executor_binding(&self) -> &str {
        self.executor_binding.as_str()
    }
}

/// Fully composed immutable model invocation plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInvocationPlan {
    target: ModelRuntimeTarget,
    profile_id: String,
    profile_hash: String,
    resolution_hash: String,
    prompt_mode_ref: String,
    prompt_assembly: animus_adapters::PromptAssembly,
    skills: Vec<ModelSkillPlanEntry>,
    memory_access_mode: ModelMemoryAccessMode,
    memory_policy_hash: String,
    memory_use_enabled: bool,
    memory_generation_enabled: bool,
    memory_plan_hash: String,
    tools: Vec<ModelToolDeclaration>,
    diagnostics: Vec<ModelInvocationDiagnostic>,
    tool_calling_mode: NativeToolCallingMode,
    max_tool_calls: u32,
    max_parallel_tool_calls: u16,
    prompt_plan_hash: String,
    skill_plan_hash: String,
    capability_plan_hash: String,
    tool_plan_hash: String,
    tool_catalog_identity: String,
    policy_hash: String,
    plan_hash: String,
}

impl ModelInvocationPlan {
    /// Returns the exact selected target.
    #[must_use]
    pub const fn target(&self) -> &ModelRuntimeTarget {
        &self.target
    }

    /// Returns the selected profile ID.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the selected profile content identity.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }

    /// Returns the profile-resolution identity.
    #[must_use]
    pub fn resolution_hash(&self) -> &str {
        self.resolution_hash.as_str()
    }

    /// Returns the admitted prompt-mode reference.
    #[must_use]
    pub fn prompt_mode_ref(&self) -> &str {
        self.prompt_mode_ref.as_str()
    }

    /// Returns existing deterministic prompt assembly metadata.
    #[must_use]
    pub const fn prompt_assembly(&self) -> &animus_adapters::PromptAssembly {
        &self.prompt_assembly
    }

    /// Returns per-model admitted skills.
    #[must_use]
    pub fn skills(&self) -> &[ModelSkillPlanEntry] {
        self.skills.as_slice()
    }

    /// Returns effective model-specific memory access posture.
    #[must_use]
    pub const fn memory_access_mode(&self) -> ModelMemoryAccessMode {
        self.memory_access_mode
    }

    /// Returns the exact existing memory-policy identity.
    #[must_use]
    pub fn memory_policy_hash(&self) -> &str {
        self.memory_policy_hash.as_str()
    }

    /// Returns whether existing policy and all ceilings admit memory use.
    #[must_use]
    pub const fn memory_use_enabled(&self) -> bool {
        self.memory_use_enabled
    }

    /// Returns whether existing policy and all ceilings admit memory generation.
    #[must_use]
    pub const fn memory_generation_enabled(&self) -> bool {
        self.memory_generation_enabled
    }

    /// Returns the deterministic effective memory-plan identity.
    #[must_use]
    pub fn memory_plan_hash(&self) -> &str {
        self.memory_plan_hash.as_str()
    }

    /// Returns canonical model-visible tool declarations.
    #[must_use]
    pub fn tools(&self) -> &[ModelToolDeclaration] {
        self.tools.as_slice()
    }

    /// Returns bounded omission diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ModelInvocationDiagnostic] {
        self.diagnostics.as_slice()
    }

    /// Returns explicit native tool-calling mode.
    #[must_use]
    pub const fn tool_calling_mode(&self) -> NativeToolCallingMode {
        self.tool_calling_mode
    }

    /// Returns the named tool-call budget ceiling.
    #[must_use]
    pub const fn max_tool_calls(&self) -> u32 {
        self.max_tool_calls
    }

    /// Returns the named parallel tool-call budget ceiling.
    #[must_use]
    pub const fn max_parallel_tool_calls(&self) -> u16 {
        self.max_parallel_tool_calls
    }

    /// Returns the deterministic prompt plan identity.
    #[must_use]
    pub fn prompt_plan_hash(&self) -> &str {
        self.prompt_plan_hash.as_str()
    }

    /// Returns the deterministic skill plan identity.
    #[must_use]
    pub fn skill_plan_hash(&self) -> &str {
        self.skill_plan_hash.as_str()
    }

    /// Returns the bound capability decision identity.
    #[must_use]
    pub fn capability_plan_hash(&self) -> &str {
        self.capability_plan_hash.as_str()
    }

    /// Returns the deterministic tool plan identity.
    #[must_use]
    pub fn tool_plan_hash(&self) -> &str {
        self.tool_plan_hash.as_str()
    }

    /// Returns the trusted tool catalog identity.
    #[must_use]
    pub fn tool_catalog_identity(&self) -> &str {
        self.tool_catalog_identity.as_str()
    }

    /// Returns the active policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the immutable invocation plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }
}

struct PromptComposition {
    assembly: animus_adapters::PromptAssembly,
    diagnostics: Vec<ModelInvocationDiagnostic>,
    plan_hash: String,
}

struct SkillComposition {
    entries: Vec<ModelSkillPlanEntry>,
    diagnostics: Vec<ModelInvocationDiagnostic>,
    plan_hash: String,
}

struct MemoryComposition {
    access_mode: ModelMemoryAccessMode,
    policy_hash: String,
    use_enabled: bool,
    generation_enabled: bool,
    plan_hash: String,
}

#[derive(Clone, Copy)]
struct MemoryCompositionInput<'a> {
    access_mode: ModelMemoryAccessMode,
    policy_hash: &'a str,
    is_use_enabled: bool,
    is_generation_enabled: bool,
}

struct SkillBodySelection {
    body: Option<animus_adapters::SkillLoadedBody>,
    diagnostic: Option<ModelInvocationDiagnostic>,
}

#[derive(Clone, Copy)]
struct ModelInvocationDiagnosticDraft<'a> {
    class: ModelInvocationOmissionClass,
    resource_label: &'a str,
    reason: &'static str,
}

struct ToolComposition {
    declarations: Vec<ModelToolDeclaration>,
    diagnostics: Vec<ModelInvocationDiagnostic>,
    plan_hash: String,
}

/// Composes one immutable invocation plan entirely from explicit in-memory inputs.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] before provider or tool effects when any
/// required target, prompt, skill, tool, authority, budget, or identity binding
/// is absent, denied, stale, or incompatible.
///
/// # Panics
///
/// Panics if the constructed immutable plan does not preserve the validated
/// target or profile identity supplied by the resolver.
// r[impl agent_harness_runtime.model_runtime_profiles.composition]
// r[impl agent_harness_runtime.model_runtime_profiles.skills]
// r[impl agent_harness_runtime.model_runtime_profiles.memory]
// r[impl agent_harness_runtime.model_runtime_profiles.prompts]
// r[impl agent_harness_runtime.model_runtime_profiles.tools]
pub fn compose_model_invocation_plan(
    input: ModelInvocationCompositionInput<'_>,
) -> Result<ModelInvocationPlan, ModelRuntimeProfileError> {
    validate_composition_boundary(&input)?;
    let mut prompt = compose_prompts(&input)?;
    let mut skills = compose_skills(&input)?;
    let memory = compose_memory(&input)?;
    let mut tools = compose_tools(&input)?;
    let capability_plan_hash = capability_plan_hash(&input);
    let policy_hash = input.authority.policy_hash.to_string();
    let plan_hash = invocation_plan_hash(InvocationPlanHashInput {
        resolved: input.resolved,
        prompt_plan_hash: prompt.plan_hash.as_str(),
        skill_plan_hash: skills.plan_hash.as_str(),
        memory_plan_hash: memory.plan_hash.as_str(),
        capability_plan_hash: capability_plan_hash.as_str(),
        tool_plan_hash: tools.plan_hash.as_str(),
        tool_catalog_identity: input.tool_catalog_identity,
        policy_hash: policy_hash.as_str(),
    });
    let mut diagnostics = Vec::with_capacity(
        prompt
            .diagnostics
            .len()
            .saturating_add(skills.diagnostics.len())
            .saturating_add(tools.diagnostics.len()),
    );
    diagnostics.append(&mut prompt.diagnostics);
    diagnostics.append(&mut skills.diagnostics);
    diagnostics.append(&mut tools.diagnostics);
    let plan = ModelInvocationPlan {
        target: input.resolved.target().clone(),
        profile_id: input.resolved.profile().profile_id().to_string(),
        profile_hash: input.resolved.profile().profile_hash().to_string(),
        resolution_hash: input.resolved.resolution_hash().to_string(),
        prompt_mode_ref: input.resolved.profile().prompt_mode_ref().to_string(),
        prompt_assembly: prompt.assembly,
        skills: skills.entries,
        memory_access_mode: memory.access_mode,
        memory_policy_hash: memory.policy_hash,
        memory_use_enabled: memory.use_enabled,
        memory_generation_enabled: memory.generation_enabled,
        memory_plan_hash: memory.plan_hash,
        tools: tools.declarations,
        diagnostics,
        tool_calling_mode: input.resolved.profile().tool_calling_mode(),
        max_tool_calls: input.resolved.tool_budget().max_tool_calls(),
        max_parallel_tool_calls: input.resolved.tool_budget().max_parallel_tool_calls(),
        prompt_plan_hash: prompt.plan_hash,
        skill_plan_hash: skills.plan_hash,
        capability_plan_hash,
        tool_plan_hash: tools.plan_hash,
        tool_catalog_identity: input.tool_catalog_identity.to_string(),
        policy_hash,
        plan_hash,
    };
    assert_eq!(plan.target(), input.resolved.target());
    assert_eq!(plan.profile_hash(), input.resolved.profile().profile_hash());
    Ok(plan)
}

fn validate_composition_boundary(input: &ModelInvocationCompositionInput<'_>) -> Result<(), ModelRuntimeProfileError> {
    lowercase_blake3(ModelProfileField(FIELD_CATALOG_IDENTITY), input.tool_catalog_identity)?;
    lowercase_blake3(ModelProfileField("authority_policy_hash"), input.authority.policy_hash)?;
    if input.authority.ceilings.is_empty() {
        return Err(error(
            ModelRuntimeProfileIssue::AuthorityDenied,
            FIELD_AUTHORITY,
            "composition requires at least one explicit authority ceiling",
        ));
    }
    validate_ceiling_labels(input.authority.ceilings)?;
    let profile = input.resolved.profile();
    if profile.policy_hash() != input.authority.policy_hash {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_AUTHORITY,
            "profile policy identity differs from active authority policy",
        ));
    }
    validate_target_capability(input)?;
    validate_target_authority(input)?;
    validate_prompt_mode_authority(input)?;
    assert!(!input.authority.ceilings.is_empty());
    assert_eq!(profile.policy_hash(), input.authority.policy_hash);
    Ok(())
}

fn validate_ceiling_labels(ceilings: &[ModelResourceCeiling<'_>]) -> Result<(), ModelRuntimeProfileError> {
    for ceiling in ceilings {
        safe_label(ModelProfileField("ceiling_label"), ceiling.ceiling_label)?;
        for policy_hash in ceiling.memory_policy_hashes {
            lowercase_blake3(ModelProfileField("ceiling_memory_policy_hash"), policy_hash)?;
        }
    }
    Ok(())
}

fn validate_target_capability(input: &ModelInvocationCompositionInput<'_>) -> Result<(), ModelRuntimeProfileError> {
    let target = input.resolved.target();
    let matrix = input.capabilities;
    let decision = input.capability_decision;
    let is_target_exact = matrix.provider_id == target.provider_id()
        && matrix.model_id == target.model_id()
        && decision.provider_id == target.provider_id()
        && decision.model_id == target.model_id();
    let is_capability_admitted = matches!(
        decision.status,
        crate::ModelCapabilityDecisionStatus::Admitted | crate::ModelCapabilityDecisionStatus::Downgraded
    );
    if !is_target_exact || !is_capability_admitted {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_COMPOSITION,
            "model capability decision is denied or bound to another target",
        ));
    }
    if input.resolved.context_budget().max_context_units() > matrix.context_budget {
        return Err(error(
            ModelRuntimeProfileIssue::InvalidBudget,
            FIELD_COMPOSITION,
            "named context budget exceeds selected model capability",
        ));
    }
    assert!(is_target_exact);
    assert!(is_capability_admitted);
    Ok(())
}

fn validate_target_authority(input: &ModelInvocationCompositionInput<'_>) -> Result<(), ModelRuntimeProfileError> {
    let target_label = format!("{}:{}", input.resolved.target().provider_id(), input.resolved.target().model_id());
    if !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.model_targets, &target_label) {
        return Err(error(
            ModelRuntimeProfileIssue::AuthorityDenied,
            FIELD_AUTHORITY,
            "one or more authority ceilings deny the selected model target",
        ));
    }
    Ok(())
}

fn validate_prompt_mode_authority(input: &ModelInvocationCompositionInput<'_>) -> Result<(), ModelRuntimeProfileError> {
    let mode = input.resolved.profile().prompt_mode_ref();
    if !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.prompt_modes, mode) {
        return Err(error(
            ModelRuntimeProfileIssue::AuthorityDenied,
            FIELD_PROMPT,
            "one or more authority ceilings deny the prompt mode",
        ));
    }
    Ok(())
}

fn compose_prompts(input: &ModelInvocationCompositionInput<'_>) -> Result<PromptComposition, ModelRuntimeProfileError> {
    validate_host_prompt_labels(input.prompt_sources)?;
    let mut selected = Vec::with_capacity(input.prompt_sources.len());
    let mut diagnostics =
        Vec::with_capacity(input.prompt_sources.len().saturating_add(input.resolved.profile().prompt_sources().len()));
    for source in input.prompt_sources {
        let profile_ref = input
            .resolved
            .profile()
            .prompt_sources()
            .iter()
            .find(|reference| reference.label() == source.label());
        let is_mandatory = source.requirement() == animus_adapters::PromptSourceRequirement::Required;
        if profile_ref.is_none() && !is_mandatory {
            continue;
        }
        let is_allowed = allowed_in_all(input.authority.ceilings, |ceiling| ceiling.prompt_sources, source.label());
        let is_required = is_mandatory || profile_ref.is_some_and(ModelPromptSourceReference::required);
        if !is_allowed {
            if is_required {
                return Err(error(
                    ModelRuntimeProfileIssue::PromptDenied,
                    FIELD_PROMPT,
                    "required prompt source is outside an authority ceiling",
                ));
            }
            diagnostics.push(diagnostic(ModelInvocationDiagnosticDraft {
                class: ModelInvocationOmissionClass::Prompt,
                resource_label: source.label(),
                reason: "authority_denied",
            }));
            continue;
        }
        selected.push(source.clone());
    }
    diagnostics.extend(record_missing_prompt_refs(input)?);
    let assembly =
        animus_adapters::assemble_prompt_sources(selected.as_slice(), input.prompt_policy).map_err(|_| {
            error(
                ModelRuntimeProfileIssue::PromptDenied,
                FIELD_PROMPT,
                "existing prompt assembly rejected selected sources",
            )
        })?;
    validate_required_prompt_outcomes(input, &assembly)?;
    validate_prompt_budget(input.resolved, &assembly)?;
    let plan_hash = prompt_plan_hash(&assembly, input.resolved.profile().prompt_mode_ref());
    assert!(!plan_hash.is_empty());
    assert!(assembly.ledger().len() <= input.prompt_sources.len());
    Ok(PromptComposition {
        assembly,
        diagnostics,
        plan_hash,
    })
}

fn validate_host_prompt_labels(sources: &[animus_adapters::PromptSource]) -> Result<(), ModelRuntimeProfileError> {
    for (index, source) in sources.iter().enumerate() {
        if sources[..index].iter().any(|prior| prior.label() == source.label()) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_PROMPT,
                "host supplied duplicate prompt-source labels",
            ));
        }
    }
    Ok(())
}

fn record_missing_prompt_refs(
    input: &ModelInvocationCompositionInput<'_>,
) -> Result<Vec<ModelInvocationDiagnostic>, ModelRuntimeProfileError> {
    let profile_prompt_sources = input.resolved.profile().prompt_sources();
    let mut diagnostics = Vec::with_capacity(profile_prompt_sources.len());
    for reference in profile_prompt_sources {
        if input.prompt_sources.iter().any(|source| source.label() == reference.label()) {
            continue;
        }
        if reference.required() {
            return Err(error(
                ModelRuntimeProfileIssue::PromptDenied,
                FIELD_PROMPT,
                "required profile prompt source is missing",
            ));
        }
        diagnostics.push(diagnostic(ModelInvocationDiagnosticDraft {
            class: ModelInvocationOmissionClass::Prompt,
            resource_label: reference.label(),
            reason: "missing_optional_source",
        }));
    }
    assert!(diagnostics.len() <= profile_prompt_sources.len());
    assert!(diagnostics.iter().all(|entry| entry.class() == ModelInvocationOmissionClass::Prompt));
    Ok(diagnostics)
}

fn validate_required_prompt_outcomes(
    input: &ModelInvocationCompositionInput<'_>,
    assembly: &animus_adapters::PromptAssembly,
) -> Result<(), ModelRuntimeProfileError> {
    for source in input.prompt_sources {
        let is_profile_required = input
            .resolved
            .profile()
            .prompt_sources()
            .iter()
            .find(|reference| reference.label() == source.label())
            .is_some_and(ModelPromptSourceReference::required);
        let is_source_required =
            source.requirement() == animus_adapters::PromptSourceRequirement::Required || is_profile_required;
        if !is_source_required {
            continue;
        }
        let is_source_included = assembly.ledger().iter().any(|entry| {
            entry.source_label() == source.label()
                && entry.state() == animus_adapters::PromptAssemblyLedgerState::Included
        });
        if !is_source_included {
            return Err(error(
                ModelRuntimeProfileIssue::PromptDenied,
                FIELD_PROMPT,
                "required prompt source was not included completely",
            ));
        }
    }
    assert!(assembly.ledger().len() <= input.prompt_sources.len());
    assert!(assembly.ledger().iter().all(|entry| !entry.source_label().is_empty()));
    Ok(())
}

fn validate_prompt_budget(
    resolved: &ResolvedModelRuntimeProfile,
    assembly: &animus_adapters::PromptAssembly,
) -> Result<(), ModelRuntimeProfileError> {
    let consumed_units = assembly
        .ledger()
        .iter()
        .fold(u32::default(), |total, entry| total.saturating_add(entry.consumed_units()));
    if consumed_units > resolved.context_budget().max_context_units() {
        return Err(error(
            ModelRuntimeProfileIssue::InvalidBudget,
            FIELD_PROMPT,
            "assembled prompt exceeds the named profile context budget",
        ));
    }
    Ok(())
}

fn compose_memory(input: &ModelInvocationCompositionInput<'_>) -> Result<MemoryComposition, ModelRuntimeProfileError> {
    let profile = input.resolved.profile();
    let access_mode = profile.memory_access_mode();
    let policy_hash = profile.memory_policy_hash();
    assert!(!profile.profile_hash().is_empty());
    assert!(!policy_hash.is_empty());
    match (access_mode, input.memory_policy) {
        (ModelMemoryAccessMode::Disabled, None) => Ok(memory_composition(MemoryCompositionInput {
            access_mode,
            policy_hash,
            is_use_enabled: false,
            is_generation_enabled: false,
        })),
        (ModelMemoryAccessMode::Disabled, Some(_)) => Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_MEMORY,
            "memory-disabled profile cannot carry an active memory policy",
        )),
        (ModelMemoryAccessMode::ReadOnly | ModelMemoryAccessMode::ReadWrite, None) => Err(error(
            ModelRuntimeProfileIssue::MemoryDenied,
            FIELD_MEMORY,
            "memory-enabled profile lacks an existing validated memory policy",
        )),
        (ModelMemoryAccessMode::ReadOnly | ModelMemoryAccessMode::ReadWrite, Some(policy)) => {
            if policy.policy_hash().as_str() != policy_hash {
                return Err(error(
                    ModelRuntimeProfileIssue::StaleBinding,
                    FIELD_MEMORY,
                    "selected memory policy identity differs from the profile binding",
                ));
            }
            if !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.memory_policy_hashes, policy_hash) {
                return Err(error(
                    ModelRuntimeProfileIssue::MemoryDenied,
                    FIELD_MEMORY,
                    "one or more authority ceilings deny the selected memory policy",
                ));
            }
            if !policy.memory_use_enabled() {
                return Err(error(
                    ModelRuntimeProfileIssue::MemoryDenied,
                    FIELD_MEMORY,
                    "existing memory policy does not admit memory use",
                ));
            }
            let is_generation_enabled = access_mode == ModelMemoryAccessMode::ReadWrite;
            if is_generation_enabled && !policy.memory_generation_enabled() {
                return Err(error(
                    ModelRuntimeProfileIssue::MemoryDenied,
                    FIELD_MEMORY,
                    "existing memory policy does not admit memory generation",
                ));
            }
            Ok(memory_composition(MemoryCompositionInput {
                access_mode,
                policy_hash,
                is_use_enabled: true,
                is_generation_enabled,
            }))
        }
    }
}

fn memory_composition(input: MemoryCompositionInput<'_>) -> MemoryComposition {
    assert!(!input.policy_hash.is_empty());
    assert!(!input.access_mode.as_str().is_empty());
    let plan_hash = blake3_parts(MEMORY_PLAN_HASH_DOMAIN, &[
        input.access_mode.as_str().to_string(),
        input.policy_hash.to_string(),
        input.is_use_enabled.to_string(),
        input.is_generation_enabled.to_string(),
    ]);
    MemoryComposition {
        access_mode: input.access_mode,
        policy_hash: input.policy_hash.to_string(),
        use_enabled: input.is_use_enabled,
        generation_enabled: input.is_generation_enabled,
        plan_hash,
    }
}

fn compose_skills(input: &ModelInvocationCompositionInput<'_>) -> Result<SkillComposition, ModelRuntimeProfileError> {
    validate_host_skill_names(input.skills)?;
    let profile_skills = input.resolved.profile().skills();
    let mut entries = Vec::with_capacity(profile_skills.len());
    let mut diagnostics = Vec::with_capacity(profile_skills.len());
    for reference in profile_skills {
        let Some(resource) = input.skills.iter().find(|resource| resource.manifest.name() == reference.name()) else {
            if let Some(entry) = handle_missing_skill(reference)? {
                diagnostics.push(entry);
            }
            continue;
        };
        if !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.skills, reference.name()) {
            if let Some(entry) = handle_denied_skill(reference, "authority_denied")? {
                diagnostics.push(entry);
            }
            continue;
        }
        if resource.manifest.trust_status() != animus_adapters::SkillManifestTrustStatus::Trusted {
            if let Some(entry) = handle_denied_skill(reference, "untrusted_manifest")? {
                diagnostics.push(entry);
            }
            continue;
        }
        let selection = selected_skill_body(input.resolved, reference, resource)?;
        if let Some(entry) = selection.diagnostic {
            diagnostics.push(entry);
        }
        entries.push(ModelSkillPlanEntry {
            name: resource.manifest.name().to_string(),
            description: resource.manifest.description().to_string(),
            content_identity: resource.manifest.content_identity().as_str().to_string(),
            posture: reference.posture(),
            selected_body: selection.body,
        });
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let plan_hash = skill_plan_hash(entries.as_slice());
    assert!(entries.len() <= profile_skills.len());
    assert!(!plan_hash.is_empty());
    Ok(SkillComposition {
        entries,
        diagnostics,
        plan_hash,
    })
}

fn validate_host_skill_names(resources: &[ModelSkillCompositionResource<'_>]) -> Result<(), ModelRuntimeProfileError> {
    for (index, resource) in resources.iter().enumerate() {
        if resources[..index].iter().any(|prior| prior.manifest.name() == resource.manifest.name()) {
            return Err(error(
                ModelRuntimeProfileIssue::DuplicateIdentity,
                FIELD_SKILL,
                "host supplied duplicate skill resources",
            ));
        }
    }
    Ok(())
}

fn handle_missing_skill(
    reference: &ModelSkillReference,
) -> Result<Option<ModelInvocationDiagnostic>, ModelRuntimeProfileError> {
    if reference.posture() == ModelSkillResourcePosture::RequiredSelected {
        return Err(error(ModelRuntimeProfileIssue::SkillDenied, FIELD_SKILL, "required selected skill is missing"));
    }
    Ok(Some(diagnostic(ModelInvocationDiagnosticDraft {
        class: ModelInvocationOmissionClass::Skill,
        resource_label: reference.name(),
        reason: "missing_optional_skill",
    })))
}

fn handle_denied_skill(
    reference: &ModelSkillReference,
    reason: &'static str,
) -> Result<Option<ModelInvocationDiagnostic>, ModelRuntimeProfileError> {
    if reference.posture() == ModelSkillResourcePosture::RequiredSelected {
        return Err(error(
            ModelRuntimeProfileIssue::SkillDenied,
            FIELD_SKILL,
            "required selected skill failed trust or authority admission",
        ));
    }
    Ok(Some(diagnostic(ModelInvocationDiagnosticDraft {
        class: ModelInvocationOmissionClass::Skill,
        resource_label: reference.name(),
        reason,
    })))
}

fn selected_skill_body(
    resolved: &ResolvedModelRuntimeProfile,
    reference: &ModelSkillReference,
    resource: &ModelSkillCompositionResource<'_>,
) -> Result<SkillBodySelection, ModelRuntimeProfileError> {
    assert!(!reference.name().is_empty());
    assert!(!resolved.profile().profile_hash().is_empty());
    if reference.posture() == ModelSkillResourcePosture::MetadataOnly {
        return Ok(SkillBodySelection {
            body: None,
            diagnostic: None,
        });
    }
    let Some(body) = resource.selected_body else {
        return missing_selected_body(reference);
    };
    validate_selected_body_binding(resolved, resource, body)?;
    let is_every_existing_check_admitted =
        resource.invocation_admitted && resource.dependencies_admitted && resource.budget_admitted;
    if !is_every_existing_check_admitted {
        return denied_selected_body(reference);
    }
    Ok(SkillBodySelection {
        body: Some(body.clone()),
        diagnostic: None,
    })
}

fn missing_selected_body(reference: &ModelSkillReference) -> Result<SkillBodySelection, ModelRuntimeProfileError> {
    if reference.posture() == ModelSkillResourcePosture::RequiredSelected {
        return Err(error(
            ModelRuntimeProfileIssue::SkillDenied,
            FIELD_SKILL,
            "required skill body was not separately selected",
        ));
    }
    Ok(SkillBodySelection {
        body: None,
        diagnostic: Some(diagnostic(ModelInvocationDiagnosticDraft {
            class: ModelInvocationOmissionClass::Skill,
            resource_label: reference.name(),
            reason: "body_not_selected",
        })),
    })
}

fn denied_selected_body(reference: &ModelSkillReference) -> Result<SkillBodySelection, ModelRuntimeProfileError> {
    if reference.posture() == ModelSkillResourcePosture::RequiredSelected {
        return Err(error(
            ModelRuntimeProfileIssue::SkillDenied,
            FIELD_SKILL,
            "required skill body failed existing invocation, dependency, or budget checks",
        ));
    }
    Ok(SkillBodySelection {
        body: None,
        diagnostic: Some(diagnostic(ModelInvocationDiagnosticDraft {
            class: ModelInvocationOmissionClass::Skill,
            resource_label: reference.name(),
            reason: "body_admission_denied",
        })),
    })
}

fn validate_selected_body_binding(
    resolved: &ResolvedModelRuntimeProfile,
    resource: &ModelSkillCompositionResource<'_>,
    body: &animus_adapters::SkillLoadedBody,
) -> Result<(), ModelRuntimeProfileError> {
    let is_identity_exact =
        body.name() == resource.manifest.name() && body.content_identity() == resource.manifest.content_identity();
    let is_plan_binding_exact = resource.selection_profile_hash == Some(resolved.profile().profile_hash())
        && resource.selection_target_hash == Some(resolved.target().target_hash());
    if !is_identity_exact || !is_plan_binding_exact {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_SKILL,
            "selected skill body belongs to another manifest, profile, or target",
        ));
    }
    assert!(is_identity_exact);
    assert!(is_plan_binding_exact);
    Ok(())
}

fn compose_tools(input: &ModelInvocationCompositionInput<'_>) -> Result<ToolComposition, ModelRuntimeProfileError> {
    assert!(!input.resolved.profile().profile_hash().is_empty());
    assert!(!input.tool_catalog_identity.is_empty());
    let mode = input.resolved.profile().tool_calling_mode();
    if mode == NativeToolCallingMode::Disabled {
        return Ok(ToolComposition {
            declarations: Vec::new(),
            diagnostics: Vec::new(),
            plan_hash: tool_plan_hash(input.tool_catalog_identity, &[]),
        });
    }
    if !input.capabilities.tool_calls {
        let is_required_tool_requested =
            input.resolved.expanded_tools().iter().any(ModelNamedResourceReference::required);
        if mode == NativeToolCallingMode::CapabilityRequired || is_required_tool_requested {
            return Err(error(
                ModelRuntimeProfileIssue::ToolDenied,
                FIELD_TOOL,
                "selected model lacks required native tool-call capability",
            ));
        }
        return Ok(ToolComposition {
            declarations: Vec::new(),
            diagnostics: vec![diagnostic(ModelInvocationDiagnosticDraft {
                class: ModelInvocationOmissionClass::NativeToolCalling,
                resource_label: "native-tools",
                reason: "model_capability_unavailable",
            })],
            plan_hash: tool_plan_hash(input.tool_catalog_identity, &[]),
        });
    }
    compose_supported_tools(input)
}

fn compose_supported_tools(
    input: &ModelInvocationCompositionInput<'_>,
) -> Result<ToolComposition, ModelRuntimeProfileError> {
    let reference_count = input.resolved.expanded_tools().len();
    let mut declarations = Vec::with_capacity(reference_count);
    let mut diagnostics = Vec::with_capacity(reference_count);
    for reference in input.resolved.expanded_tools() {
        let definition = input
            .tool_catalog
            .definitions()
            .iter()
            .find(|definition| definition.metadata().name() == reference.name());
        let Some(definition) = definition else {
            if let Some(entry) = handle_denied_tool(reference, "missing_tool")? {
                diagnostics.push(entry);
            }
            continue;
        };
        if let Some(reason) = tool_denial_reason(input, definition) {
            if let Some(entry) = handle_denied_tool(reference, reason)? {
                diagnostics.push(entry);
            }
            continue;
        }
        declarations.push(ModelToolDeclaration {
            definition: definition.metadata().clone(),
            executor_binding: definition.executor_binding().to_string(),
        });
    }
    declarations.sort_by(|left, right| left.definition.name().cmp(right.definition.name()));
    let is_capability_mode_required =
        input.resolved.profile().tool_calling_mode() == NativeToolCallingMode::CapabilityRequired;
    if is_capability_mode_required && declarations.is_empty() {
        return Err(error(
            ModelRuntimeProfileIssue::ToolDenied,
            FIELD_TOOL,
            "capability-required mode has no admitted tool declarations",
        ));
    }
    let plan_hash = tool_plan_hash(input.tool_catalog_identity, declarations.as_slice());
    assert!(declarations.len() <= reference_count);
    assert!(!plan_hash.is_empty());
    Ok(ToolComposition {
        declarations,
        diagnostics,
        plan_hash,
    })
}

fn tool_denial_reason(
    input: &ModelInvocationCompositionInput<'_>,
    definition: &crate::tool_executor_kit::ToolDefinition,
) -> Option<&'static str> {
    assert!(!definition.metadata().name().is_empty());
    assert!(!definition.executor_binding().is_empty());
    if definition.metadata().status() != animus_adapters::tool::DefinitionStatus::Active {
        return Some("inactive_or_untrusted_tool");
    }
    if !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.tools, definition.metadata().name()) {
        return Some("tool_authority_denied");
    }
    if definition.metadata().capability_labels().iter().any(|capability| {
        !allowed_in_all(input.authority.ceilings, |ceiling| ceiling.tool_capabilities, capability.as_str())
    }) {
        return Some("capability_authority_denied");
    }
    if !executor_is_ready(input, definition.executor_binding()) {
        return Some("executor_unavailable");
    }
    match definition.approval_posture() {
        animus_adapters::tool::ApprovalPosture::PreApproved => None,
        animus_adapters::tool::ApprovalPosture::RequiresApproval => {
            (!input.authority.approved_tools.contains(&definition.metadata().name())).then_some("approval_missing")
        }
        animus_adapters::tool::ApprovalPosture::Denied => Some("approval_denied"),
    }
}

fn executor_is_ready(input: &ModelInvocationCompositionInput<'_>, binding: &str) -> bool {
    input.executor_bindings.iter().any(|candidate| {
        candidate.label() == binding && candidate.readiness() == crate::tool_executor_kit::ToolExecutorReadiness::Ready
    })
}

fn handle_denied_tool(
    reference: &ModelNamedResourceReference,
    reason: &'static str,
) -> Result<Option<ModelInvocationDiagnostic>, ModelRuntimeProfileError> {
    if reference.required() {
        return Err(error(
            ModelRuntimeProfileIssue::ToolDenied,
            FIELD_TOOL,
            "required tool failed catalog, capability, approval, or executor admission",
        ));
    }
    Ok(Some(diagnostic(ModelInvocationDiagnosticDraft {
        class: ModelInvocationOmissionClass::Tool,
        resource_label: reference.name(),
        reason,
    })))
}

fn allowed_in_all<'a, F>(ceilings: &'a [ModelResourceCeiling<'a>], select: F, value: &str) -> bool
where F: Fn(&ModelResourceCeiling<'a>) -> &'a [&'a str] {
    ceilings.iter().all(|ceiling| select(ceiling).contains(&value))
}

fn diagnostic(draft: ModelInvocationDiagnosticDraft<'_>) -> ModelInvocationDiagnostic {
    ModelInvocationDiagnostic {
        class: draft.class,
        resource_label: draft.resource_label.to_string(),
        reason: draft.reason.to_string(),
    }
}

fn capability_plan_hash(input: &ModelInvocationCompositionInput<'_>) -> String {
    let status = match input.capability_decision.status {
        crate::ModelCapabilityDecisionStatus::Admitted => "admitted",
        crate::ModelCapabilityDecisionStatus::Downgraded => "downgraded",
        crate::ModelCapabilityDecisionStatus::Denied => "denied",
    };
    blake3_parts(CAPABILITY_PLAN_HASH_DOMAIN, &[
        input.resolved.target().target_hash().to_string(),
        input.resolved.target().capability_identity().to_string(),
        input.capabilities.schema_identity.clone(),
        status.to_string(),
    ])
}

fn prompt_plan_hash(assembly: &animus_adapters::PromptAssembly, prompt_mode_ref: &str) -> String {
    let mut parts = vec![prompt_mode_ref.to_string(), assembly.policy_hash().as_str().to_string()];
    for entry in assembly.ledger() {
        parts.push(format!(
            "{}:{}:{}:{}:{}",
            entry.source_label(),
            prompt_state_label(entry.state()),
            entry.consumed_units(),
            entry.body_hash().as_str(),
            entry.reason().map_or(NO_REASON_LABEL, prompt_reason_label)
        ));
    }
    blake3_parts(PROMPT_PLAN_HASH_DOMAIN, parts.as_slice())
}

fn prompt_state_label(state: animus_adapters::PromptAssemblyLedgerState) -> &'static str {
    match state {
        animus_adapters::PromptAssemblyLedgerState::Included => "included",
        animus_adapters::PromptAssemblyLedgerState::Skipped => "skipped",
        animus_adapters::PromptAssemblyLedgerState::Truncated => "truncated",
        animus_adapters::PromptAssemblyLedgerState::Redacted => "redacted",
        animus_adapters::PromptAssemblyLedgerState::Denied => "denied",
        animus_adapters::PromptAssemblyLedgerState::Conflicting => "conflicting",
        animus_adapters::PromptAssemblyLedgerState::Unsupported => "unsupported",
    }
}

fn prompt_reason_label(reason: animus_adapters::PromptAssemblyLedgerReason) -> &'static str {
    match reason {
        animus_adapters::PromptAssemblyLedgerReason::OverBudget => "over_budget",
        animus_adapters::PromptAssemblyLedgerReason::RedactedByPolicy => "redacted_by_policy",
        animus_adapters::PromptAssemblyLedgerReason::UntrustedSource => "untrusted_source",
        animus_adapters::PromptAssemblyLedgerReason::DeniedByAdmission => "denied_by_admission",
        animus_adapters::PromptAssemblyLedgerReason::RequiredSourceUnavailable => "required_source_unavailable",
        animus_adapters::PromptAssemblyLedgerReason::SourceConflict => "source_conflict",
        animus_adapters::PromptAssemblyLedgerReason::DiscoveryUnavailable => "discovery_unavailable",
        animus_adapters::PromptAssemblyLedgerReason::UnsupportedSourceKind => "unsupported_source_kind",
    }
}

fn skill_plan_hash(entries: &[ModelSkillPlanEntry]) -> String {
    let mut parts = Vec::with_capacity(entries.len());
    for entry in entries {
        parts.push(format!(
            "{}:{}:{}:{}",
            entry.name,
            entry.content_identity,
            entry.posture.as_str(),
            entry.selected_body.is_some()
        ));
    }
    blake3_parts(SKILL_PLAN_HASH_DOMAIN, parts.as_slice())
}

fn tool_plan_hash(catalog_identity: &str, declarations: &[ModelToolDeclaration]) -> String {
    assert!(!catalog_identity.is_empty());
    assert!(declarations.iter().all(|declaration| !declaration.definition.name().is_empty()));
    let mut parts = vec![catalog_identity.to_string()];
    for declaration in declarations {
        let capabilities =
            declaration.definition.capability_labels().iter().map(String::as_str).collect::<Vec<_>>().join(",");
        parts.push(format!(
            "{}:{}:{}:{}",
            declaration.definition.name(),
            declaration.definition.parameter_schema_identity(),
            declaration.executor_binding,
            capabilities
        ));
    }
    blake3_parts(TOOL_PLAN_HASH_DOMAIN, parts.as_slice())
}

#[derive(Clone, Copy)]
struct InvocationPlanHashInput<'a> {
    resolved: &'a ResolvedModelRuntimeProfile,
    prompt_plan_hash: &'a str,
    skill_plan_hash: &'a str,
    memory_plan_hash: &'a str,
    capability_plan_hash: &'a str,
    tool_plan_hash: &'a str,
    tool_catalog_identity: &'a str,
    policy_hash: &'a str,
}

fn invocation_plan_hash(input: InvocationPlanHashInput<'_>) -> String {
    blake3_parts(MODEL_INVOCATION_PLAN_HASH_DOMAIN, &[
        input.resolved.resolution_hash().to_string(),
        input.resolved.target().target_hash().to_string(),
        input.resolved.profile().profile_hash().to_string(),
        input.prompt_plan_hash.to_string(),
        input.skill_plan_hash.to_string(),
        input.memory_plan_hash.to_string(),
        input.capability_plan_hash.to_string(),
        input.tool_plan_hash.to_string(),
        input.tool_catalog_identity.to_string(),
        input.policy_hash.to_string(),
    ])
}
