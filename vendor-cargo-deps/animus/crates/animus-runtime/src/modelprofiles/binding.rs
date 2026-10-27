//! Exact model dispatch, transition, and returned-tool-call bindings.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use super::composition::ModelInvocationPlan;
use super::types::{
    ModelProfileEnablementPolicy, ModelRuntimeProfileError, ModelRuntimeProfileIssue,
    ModelRuntimeTarget,
};
use super::validation::{ModelProfileField, blake3_parts, error, lowercase_blake3, safe_label};

const FIELD_DISPATCH: &str = "model_dispatch";
const FIELD_TRANSITION: &str = "invocation_transition";
const FIELD_DIALECT_TRANSITION: &str = "dialect_transition";
const FIELD_TOOL_CALL: &str = "model_tool_call";
const DIALECT_TRANSITION_HASH_DOMAIN: &str = "animus.model-dialect-transition.v1";

/// Model dispatch admitted with an explicit per-model profile posture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileBoundModelDispatchPlan {
    dispatch: crate::ModelDispatchPlan,
    profile_enablement: ModelProfileEnablementPolicy,
    invocation_plan_hash: Option<String>,
}

impl ProfileBoundModelDispatchPlan {
    /// Returns the underlying admitted provider-neutral dispatch plan.
    #[must_use]
    pub const fn dispatch(&self) -> &crate::ModelDispatchPlan {
        &self.dispatch
    }

    /// Returns the explicit profile enablement posture.
    #[must_use]
    pub const fn profile_enablement(&self) -> ModelProfileEnablementPolicy {
        self.profile_enablement
    }

    /// Returns the required invocation plan identity when profiles are enabled.
    #[must_use]
    pub fn invocation_plan_hash(&self) -> Option<&str> {
        self.invocation_plan_hash.as_deref()
    }
}

/// Binds existing model-dispatch admission to explicit profile enablement.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] when provider-neutral dispatch is denied,
/// required mode lacks a composed plan, or compatibility-disabled mode is given
/// a profile plan implicitly.
///
/// # Panics
///
/// Panics if the constructed binding contradicts the already validated dispatch
/// or explicit profile-enablement invariant.
// r[impl agent_harness_runtime.model_runtime_profiles.dispatch_binding]
pub fn bind_model_dispatch_plan(
    dispatch: &crate::ModelDispatchPlan,
    enablement: ModelProfileEnablementPolicy,
    invocation_plan: Option<&ModelInvocationPlan>,
) -> Result<ProfileBoundModelDispatchPlan, ModelRuntimeProfileError> {
    if !dispatch.is_admitted() {
        return Err(error(
            ModelRuntimeProfileIssue::DispatchDenied,
            FIELD_DISPATCH,
            "provider-neutral model dispatch was not admitted",
        ));
    }
    let invocation_plan_hash = match (enablement, invocation_plan) {
        (ModelProfileEnablementPolicy::Required, Some(plan)) => Some(plan.plan_hash().to_string()),
        (ModelProfileEnablementPolicy::Required, None) => {
            return Err(error(
                ModelRuntimeProfileIssue::MissingProfile,
                FIELD_DISPATCH,
                "required profile mode lacks a composed invocation plan",
            ));
        }
        (ModelProfileEnablementPolicy::CompatibilityDisabled, None) => None,
        (ModelProfileEnablementPolicy::CompatibilityDisabled, Some(_)) => {
            return Err(error(
                ModelRuntimeProfileIssue::StaleBinding,
                FIELD_DISPATCH,
                "compatibility-disabled mode cannot infer or carry a profile plan",
            ));
        }
    };
    let plan = ProfileBoundModelDispatchPlan {
        dispatch: dispatch.clone(),
        profile_enablement: enablement,
        invocation_plan_hash,
    };
    assert!(plan.dispatch.is_admitted());
    assert_eq!(
        plan.invocation_plan_hash.is_some(),
        enablement == ModelProfileEnablementPolicy::Required
    );
    Ok(plan)
}

/// Transition kind at a router, retry, resume, or replay boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelInvocationTransitionKind {
    /// Router selected a different fallback target.
    RouterFallback,
    /// A retriable attempt remains on the same exact target.
    SameTargetRetry,
    /// Runtime is resuming a prior invocation.
    Resume,
    /// Runtime is replaying a prior invocation.
    Replay,
}

/// Explicit identity snapshot considered for plan reuse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelInvocationIdentitySnapshot<'a> {
    /// Target identity.
    pub target_hash: &'a str,
    /// Profile content identity.
    pub profile_hash: &'a str,
    /// Prompt plan identity.
    pub prompt_plan_hash: &'a str,
    /// Skill plan identity.
    pub skill_plan_hash: &'a str,
    /// Effective memory plan identity.
    pub memory_plan_hash: &'a str,
    /// Capability plan identity.
    pub capability_plan_hash: &'a str,
    /// Tool plan identity.
    pub tool_plan_hash: &'a str,
    /// Tool catalog identity.
    pub tool_catalog_identity: &'a str,
    /// Active policy identity.
    pub policy_hash: &'a str,
    /// Immutable invocation plan identity.
    pub plan_hash: &'a str,
}

impl<'a> From<&'a ModelInvocationPlan> for ModelInvocationIdentitySnapshot<'a> {
    fn from(plan: &'a ModelInvocationPlan) -> Self {
        Self {
            target_hash: plan.target().target_hash(),
            profile_hash: plan.profile_hash(),
            prompt_plan_hash: plan.prompt_plan_hash(),
            skill_plan_hash: plan.skill_plan_hash(),
            memory_plan_hash: plan.memory_plan_hash(),
            capability_plan_hash: plan.capability_plan_hash(),
            tool_plan_hash: plan.tool_plan_hash(),
            tool_catalog_identity: plan.tool_catalog_identity(),
            policy_hash: plan.policy_hash(),
            plan_hash: plan.plan_hash(),
        }
    }
}

/// Router/retry/resume/replay planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelInvocationTransitionRequest<'a> {
    /// Transition class.
    pub kind: ModelInvocationTransitionKind,
    /// Current immutable invocation plan.
    pub current_plan: &'a ModelInvocationPlan,
    /// Target identity requested for the next attempt.
    pub next_target_hash: &'a str,
    /// Explicit current identity snapshot supplied by the host.
    pub current_snapshot: ModelInvocationIdentitySnapshot<'a>,
}

/// Safe transition decision before provider effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelInvocationTransitionDecision {
    /// Reuse the exact immutable same-target plan.
    ReuseExactPlan {
        /// Exact plan identity.
        plan_hash: String,
    },
    /// Resolve and compose a fresh profile for a different target.
    RecomposeForTarget {
        /// New target identity that must be resolved.
        target_hash: String,
    },
}

/// Explicit provider-transport and model-dialect compatibility declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelTransportDialectAdmissionDraft<'a> {
    /// Exact versioned provider-transport identity.
    pub provider_transport_id: &'a str,
    /// Exact versioned model-facing dialect identity.
    pub model_dialect_id: &'a str,
}

/// Provider transition input for exact dialect retention.
#[derive(Clone, Copy, Debug)]
pub struct ModelDialectTransitionRequest<'a> {
    /// Current validated target.
    pub current_target: &'a ModelRuntimeTarget,
    /// Explicit next validated target.
    pub next_target: &'a ModelRuntimeTarget,
    /// Exact admitted transport and dialect pairs.
    pub admitted_pairs: &'a [ModelTransportDialectAdmissionDraft<'a>],
}

/// Admitted provider transition that retains one exact dialect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelDialectTransitionPlan {
    provider_transport_id: String,
    model_dialect_id: String,
    transition_hash: String,
}

impl ModelDialectTransitionPlan {
    /// Returns the next provider-transport identity.
    #[must_use]
    pub fn provider_transport_id(&self) -> &str {
        self.provider_transport_id.as_str()
    }

    /// Returns the retained model-facing dialect identity.
    #[must_use]
    pub fn model_dialect_id(&self) -> &str {
        self.model_dialect_id.as_str()
    }

    /// Returns the deterministic transition identity.
    #[must_use]
    pub fn transition_hash(&self) -> &str {
        self.transition_hash.as_str()
    }
}

/// Admits a provider transition only when the next transport declares the exact current dialect.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] when the transition substitutes a dialect,
/// contains malformed compatibility facts, or lacks an exact admitted pair.
///
/// # Panics
///
/// Panics if deterministic transition identity construction returns empty text.
// r[impl animus.dialect.transition]
#[cfg_attr(
    animus_octet,
    allow(
        tigerstyle::numeric_units,
        reason = "provider transport identifiers are labels, not numeric ports or quantities"
    )
)]
pub fn evaluate_model_dialect_transition(
    request: ModelDialectTransitionRequest<'_>,
) -> Result<ModelDialectTransitionPlan, ModelRuntimeProfileError> {
    if request.current_target.model_dialect_id() != request.next_target.model_dialect_id() {
        return Err(error(
            ModelRuntimeProfileIssue::RecompositionRequired,
            FIELD_DIALECT_TRANSITION,
            "provider transition cannot substitute a model-facing dialect",
        ));
    }
    let next_transport = request.next_target.provider_transport_id();
    let dialect = request.next_target.model_dialect_id();
    let mut is_exact_pair_admitted = false;
    for pair in request.admitted_pairs {
        let pair_transport = safe_label(
            ModelProfileField("provider_transport_id"),
            pair.provider_transport_id,
        )?;
        let pair_dialect =
            safe_label(ModelProfileField("model_dialect_id"), pair.model_dialect_id)?;
        if pair_transport == next_transport && pair_dialect == dialect {
            is_exact_pair_admitted = true;
        }
    }
    if !is_exact_pair_admitted {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_DIALECT_TRANSITION,
            "next provider transport does not admit the exact model-facing dialect",
        ));
    }
    let transition_hash = blake3_parts(
        DIALECT_TRANSITION_HASH_DOMAIN,
        &[
            request.current_target.target_hash().to_string(),
            request.next_target.target_hash().to_string(),
            next_transport.to_string(),
            dialect.to_string(),
        ],
    );
    let plan = ModelDialectTransitionPlan {
        provider_transport_id: next_transport.to_string(),
        model_dialect_id: dialect.to_string(),
        transition_hash,
    };
    assert_eq!(plan.provider_transport_id(), next_transport);
    assert_eq!(plan.model_dialect_id(), dialect);
    assert!(!plan.transition_hash().is_empty());
    Ok(plan)
}

/// Evaluates fallback and same-target reuse without carrying model-specific state.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] for malformed or stale identity snapshots,
/// same-target fallback, or cross-target retry/resume/replay attempts.
///
/// # Panics
///
/// Panics if a successfully validated snapshot no longer matches its immutable
/// plan or if validated target identity unexpectedly becomes empty.
// r[impl agent_harness_runtime.model_runtime_profiles.dispatch_binding]
pub fn evaluate_model_invocation_transition(
    request: ModelInvocationTransitionRequest<'_>,
) -> Result<ModelInvocationTransitionDecision, ModelRuntimeProfileError> {
    validate_snapshot_hashes(&request.current_snapshot)?;
    lowercase_blake3(
        ModelProfileField("next_target_hash"),
        request.next_target_hash,
    )?;
    if !snapshot_matches_plan(&request.current_snapshot, request.current_plan) {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_TRANSITION,
            "current profile, prompt, skill, capability, tool, policy, or plan identity changed",
        ));
    }
    assert!(snapshot_matches_plan(
        &request.current_snapshot,
        request.current_plan
    ));
    assert!(!request.next_target_hash.is_empty());
    match request.kind {
        ModelInvocationTransitionKind::RouterFallback => {
            if request.next_target_hash == request.current_plan.target().target_hash() {
                return Err(error(
                    ModelRuntimeProfileIssue::RecompositionRequired,
                    FIELD_TRANSITION,
                    "router fallback must identify a different target",
                ));
            }
            Ok(ModelInvocationTransitionDecision::RecomposeForTarget {
                target_hash: request.next_target_hash.to_string(),
            })
        }
        ModelInvocationTransitionKind::SameTargetRetry
        | ModelInvocationTransitionKind::Resume
        | ModelInvocationTransitionKind::Replay => {
            if request.next_target_hash != request.current_plan.target().target_hash() {
                return Err(error(
                    ModelRuntimeProfileIssue::RecompositionRequired,
                    FIELD_TRANSITION,
                    "cross-target retry, resume, or replay requires fresh profile composition",
                ));
            }
            Ok(ModelInvocationTransitionDecision::ReuseExactPlan {
                plan_hash: request.current_plan.plan_hash().to_string(),
            })
        }
    }
}

fn validate_snapshot_hashes(
    snapshot: &ModelInvocationIdentitySnapshot<'_>,
) -> Result<(), ModelRuntimeProfileError> {
    for (field, value) in [
        ("target_hash", snapshot.target_hash),
        ("profile_hash", snapshot.profile_hash),
        ("prompt_plan_hash", snapshot.prompt_plan_hash),
        ("skill_plan_hash", snapshot.skill_plan_hash),
        ("memory_plan_hash", snapshot.memory_plan_hash),
        ("capability_plan_hash", snapshot.capability_plan_hash),
        ("tool_plan_hash", snapshot.tool_plan_hash),
        ("tool_catalog_identity", snapshot.tool_catalog_identity),
        ("policy_hash", snapshot.policy_hash),
        ("plan_hash", snapshot.plan_hash),
    ] {
        lowercase_blake3(ModelProfileField(field), value)?;
    }
    Ok(())
}

fn snapshot_matches_plan(
    snapshot: &ModelInvocationIdentitySnapshot<'_>,
    plan: &ModelInvocationPlan,
) -> bool {
    snapshot.target_hash == plan.target().target_hash()
        && snapshot.profile_hash == plan.profile_hash()
        && snapshot.prompt_plan_hash == plan.prompt_plan_hash()
        && snapshot.skill_plan_hash == plan.skill_plan_hash()
        && snapshot.memory_plan_hash == plan.memory_plan_hash()
        && snapshot.capability_plan_hash == plan.capability_plan_hash()
        && snapshot.tool_plan_hash == plan.tool_plan_hash()
        && snapshot.tool_catalog_identity == plan.tool_catalog_identity()
        && snapshot.policy_hash == plan.policy_hash()
        && snapshot.plan_hash == plan.plan_hash()
}

/// Returned model tool-call binding fields supplied before executor-kit admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelToolCallBindingDraft<'a> {
    /// Exact tool name returned by the model.
    pub tool_name: &'a str,
    /// Invocation plan identity returned with the call.
    pub invocation_plan_hash: &'a str,
    /// Target identity returned with the call.
    pub target_hash: &'a str,
    /// Profile identity returned with the call.
    pub profile_hash: &'a str,
    /// Prompt plan identity returned with the call.
    pub prompt_plan_hash: &'a str,
    /// Skill plan identity returned with the call.
    pub skill_plan_hash: &'a str,
    /// Effective memory plan identity returned with the call.
    pub memory_plan_hash: &'a str,
    /// Capability plan identity returned with the call.
    pub capability_plan_hash: &'a str,
    /// Tool plan identity returned with the call.
    pub tool_plan_hash: &'a str,
    /// Tool catalog identity returned with the call.
    pub tool_catalog_identity: &'a str,
    /// Active policy identity returned with the call.
    pub policy_hash: &'a str,
    /// Whether the call was parsed from prompt-emulated syntax.
    pub prompt_emulated: bool,
}

/// Validated exact returned-tool-call binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelToolCallBinding {
    tool_name: String,
    invocation_plan_hash: String,
}

impl ModelToolCallBinding {
    /// Returns the exact advertised tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the exact invocation plan identity.
    #[must_use]
    pub fn invocation_plan_hash(&self) -> &str {
        self.invocation_plan_hash.as_str()
    }
}

/// Validates a returned model tool call against every bound plan identity.
///
/// # Errors
///
/// Returns [`ModelRuntimeProfileError`] for prompt-emulated, unadvertised,
/// malformed, stale, cross-profile, cross-target, cross-catalog, or cross-policy
/// calls. Existing tool executor-kit admission is still required afterward.
///
/// # Panics
///
/// Panics if a call accepted by every validation branch is nevertheless marked
/// prompt-emulated or absent from the immutable advertised declaration set.
// r[impl agent_harness_runtime.model_runtime_profiles.dispatch_binding]
pub fn validate_model_tool_call_binding(
    plan: &ModelInvocationPlan,
    draft: ModelToolCallBindingDraft<'_>,
) -> Result<ModelToolCallBinding, ModelRuntimeProfileError> {
    let tool_name = safe_label(ModelProfileField("tool_name"), draft.tool_name)?;
    if draft.prompt_emulated {
        return Err(error(
            ModelRuntimeProfileIssue::ToolDenied,
            FIELD_TOOL_CALL,
            "prompt-emulated tool calls are not canonical native calls",
        ));
    }
    let candidate = ModelInvocationIdentitySnapshot {
        target_hash: draft.target_hash,
        profile_hash: draft.profile_hash,
        prompt_plan_hash: draft.prompt_plan_hash,
        skill_plan_hash: draft.skill_plan_hash,
        memory_plan_hash: draft.memory_plan_hash,
        capability_plan_hash: draft.capability_plan_hash,
        tool_plan_hash: draft.tool_plan_hash,
        tool_catalog_identity: draft.tool_catalog_identity,
        policy_hash: draft.policy_hash,
        plan_hash: draft.invocation_plan_hash,
    };
    validate_snapshot_hashes(&candidate)?;
    if !snapshot_matches_plan(&candidate, plan) {
        return Err(error(
            ModelRuntimeProfileIssue::StaleBinding,
            FIELD_TOOL_CALL,
            "returned tool call does not match the exact invocation plan",
        ));
    }
    let is_tool_advertised = plan
        .tools()
        .iter()
        .any(|declaration| declaration.definition().name() == tool_name);
    if !is_tool_advertised {
        return Err(error(
            ModelRuntimeProfileIssue::ToolDenied,
            FIELD_TOOL_CALL,
            "returned tool was not advertised by this invocation plan",
        ));
    }
    assert!(!draft.prompt_emulated);
    assert!(is_tool_advertised);
    Ok(ModelToolCallBinding {
        tool_name,
        invocation_plan_hash: draft.invocation_plan_hash.to_string(),
    })
}
