//! Pure generation-fenced component registration, resolution, and teardown.
//!
//! The core consumes explicit activation metadata and host observations. It
//! performs no cancellation, waiting, listener, task, process, filesystem,
//! network, clock, persistence, terminal, provider, tool, or credential effect.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::too_many_parameters,
        tigerstyle::usize_in_public_api,
        reason = "published component-scope compatibility contracts retain their typed stable signatures"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        reason = "fail-closed reducers return typed errors for untrusted observations instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the no_std component uses explicit sibling DTO imports to keep reducer ownership visible"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published component-scope vocabulary uses protocol field names fixed by its versioned wire contract"
    )
)]

mod core;
mod data;
#[path = "receipt.rs"]
mod redaction;

pub use core::{
    admit_child_component_scope, admit_component_scope, fail_component_scope_cleanup,
    reduce_component_scope_observation, request_component_scope_close, resolve_component,
};
pub use data::{
    ComponentLease, ComponentRegistration, ComponentRegistrationDraft, ComponentScope,
    ComponentScopeDraft, ComponentScopeError, ComponentScopeReceipt, ComponentScopeState,
    ScopeCleanupFailure, ScopeCleanupIssue, ScopeCleanupIssueKind, ScopeClosePlan,
    ScopeCloseRequest, ScopeEffectRequest, ScopeGeneration, ScopeMemberKind, ScopeObservation,
    ScopeParent, ScopeResolutionRequest, ScopeResolutionTarget, ScopeTerminalOutcome,
    ScopeTransition, ScopeTransitionClass, ScopeVisibility,
};
pub use redaction::{project_component_scope_receipt, validate_component_scope_evidence_text};

#[cfg(test)]
mod tests;
