//! Metadata-only component-scope receipt projection and evidence checks.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake3::Hasher;

use super::core::{update_str, update_u64, validate_hash, validate_label};
use super::data::{
    ComponentScope, ComponentScopeError, ComponentScopeReceipt, SCOPE_RECEIPT_IDENTITY_DOMAIN,
    SCOPE_RECEIPT_SCHEMA, ScopeMemberKind, ScopeTerminalOutcome, ScopeTransitionClass,
};

const FORBIDDEN_SCOPE_EVIDENCE_MARKERS: &[&str] = &[
    "/home/",
    "api_key",
    "authorization:",
    "command=",
    "credential",
    "endpoint=",
    "private path",
    "prompt=",
    "provider payload",
    "secret",
    "service handle",
    "stderr:",
    "stdout:",
    "tool payload",
];

/// Projects one stable metadata-only terminal receipt.
///
/// The receipt excludes host handles, paths, commands, payloads, prompts,
/// credentials, endpoints, service values, and raw errors.
///
/// # Errors
///
/// Returns a typed error when terminal facts are incomplete, inconsistent, or
/// outside receipt bounds.
pub fn project_component_scope_receipt(
    scope: &ComponentScope,
) -> Result<ComponentScopeReceipt, ComponentScopeError> {
    validate_hash(scope.graph_identity.as_str())?;
    validate_hash(scope.identity.as_str())?;
    let plan = scope
        .close_plan
        .as_ref()
        .ok_or(ComponentScopeError::ReceiptNotReady)?;
    validate_hash(plan.identity.as_str())?;
    let outcome = scope
        .terminal_outcome
        .ok_or(ComponentScopeError::ReceiptNotReady)?;
    validate_terminal_facts(scope, outcome)?;

    let component_count = u16::try_from(
        scope
            .registrations
            .iter()
            .filter(|registration| registration.member_kind == ScopeMemberKind::Component)
            .count(),
    )
    .map_err(|_error| ComponentScopeError::BoundExceeded)?;
    let child_scope_count = u16::try_from(
        scope
            .registrations
            .iter()
            .filter(|registration| registration.member_kind == ScopeMemberKind::ChildScope)
            .count(),
    )
    .map_err(|_error| ComponentScopeError::BoundExceeded)?;
    let requested_effect_count = u32::try_from(scope.requested_effect_count)
        .map_err(|_error| ComponentScopeError::BoundExceeded)?;
    let observed_quiescent_count = u16::try_from(scope.quiescent_member_ids.len())
        .map_err(|_error| ComponentScopeError::BoundExceeded)?;
    let observed_release_count = u16::try_from(scope.released_member_ids.len())
        .map_err(|_error| ComponentScopeError::BoundExceeded)?;
    let transition_classes: Vec<ScopeTransitionClass> = scope
        .transition_classes
        .iter()
        .copied()
        .filter(|class| {
            !matches!(
                class,
                ScopeTransitionClass::TerminalCommitObserved | ScopeTransitionClass::CloseReused
            )
        })
        .collect();
    for issue in &scope.unresolved_cleanup {
        validate_label(issue.subject.as_str())?;
    }
    for non_claim in &scope.non_claims {
        validate_component_scope_evidence_text(non_claim.as_str())?;
    }

    let identity = receipt_identity(
        scope,
        plan.identity.as_str(),
        outcome,
        component_count,
        child_scope_count,
        requested_effect_count,
        observed_quiescent_count,
        observed_release_count,
        transition_classes.as_slice(),
    );
    Ok(ComponentScopeReceipt {
        schema: SCOPE_RECEIPT_SCHEMA,
        scope_id: scope.scope_id.clone(),
        generation: scope.generation,
        graph_identity: scope.graph_identity.clone(),
        plan_identity: plan.identity.clone(),
        outcome,
        component_count,
        child_scope_count,
        requested_effect_count,
        observed_quiescent_count,
        observed_release_count,
        unresolved_cleanup: scope.unresolved_cleanup.clone(),
        transition_classes,
        non_claims: scope.non_claims.clone(),
        identity,
    })
}

/// Rejects unsafe text from scope identities, issues, receipts, and non-claims.
///
/// # Errors
///
/// Returns [`ComponentScopeError::UnsafeEvidence`] for forbidden text.
pub fn validate_component_scope_evidence_text(value: &str) -> Result<(), ComponentScopeError> {
    let normalized = value.to_ascii_lowercase();
    if normalized.starts_with("sk-")
        || FORBIDDEN_SCOPE_EVIDENCE_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
    {
        return Err(ComponentScopeError::UnsafeEvidence);
    }
    Ok(())
}

fn validate_terminal_facts(
    scope: &ComponentScope,
    outcome: ScopeTerminalOutcome,
) -> Result<(), ComponentScopeError> {
    match outcome {
        ScopeTerminalOutcome::Closed
            if scope.quiescent_member_ids.len() == scope.registrations.len()
                && scope.released_member_ids.len() == scope.registrations.len()
                && scope.unresolved_cleanup.is_empty() =>
        {
            Ok(())
        }
        ScopeTerminalOutcome::FailedCleanup if !scope.unresolved_cleanup.is_empty() => Ok(()),
        ScopeTerminalOutcome::Closed | ScopeTerminalOutcome::FailedCleanup => {
            Err(ComponentScopeError::CleanupIncomplete)
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "receipt identity binds each bounded terminal count explicitly"
)]
fn receipt_identity(
    scope: &ComponentScope,
    plan_identity: &str,
    outcome: ScopeTerminalOutcome,
    component_count: u16,
    child_scope_count: u16,
    requested_effect_count: u32,
    observed_quiescent_count: u16,
    observed_release_count: u16,
    transition_classes: &[ScopeTransitionClass],
) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, SCOPE_RECEIPT_IDENTITY_DOMAIN);
    update_str(&mut hasher, scope.scope_id.as_str());
    update_u64(&mut hasher, scope.generation.get());
    update_str(&mut hasher, scope.graph_identity.as_str());
    update_str(&mut hasher, plan_identity);
    update_str(&mut hasher, outcome.as_str());
    update_u64(&mut hasher, u64::from(component_count));
    update_u64(&mut hasher, u64::from(child_scope_count));
    update_u64(&mut hasher, u64::from(requested_effect_count));
    update_u64(&mut hasher, u64::from(observed_quiescent_count));
    update_u64(&mut hasher, u64::from(observed_release_count));
    for issue in &scope.unresolved_cleanup {
        update_str(&mut hasher, issue.kind.as_str());
        update_str(&mut hasher, issue.subject.as_str());
    }
    for class in transition_classes {
        update_str(&mut hasher, class.as_str());
    }
    for non_claim in &scope.non_claims {
        update_str(&mut hasher, non_claim.as_str());
    }
    hasher.finalize().to_hex().to_string()
}
