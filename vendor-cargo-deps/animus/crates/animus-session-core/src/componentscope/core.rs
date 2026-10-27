//! Pure component-scope admission, resolution, and lifecycle reduction.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake3::Hasher;

use super::data::{
    CLOSE_PLAN_IDENTITY_DOMAIN, ComponentLease, ComponentRegistration, ComponentRegistrationDraft,
    ComponentScope, ComponentScopeDraft, ComponentScopeError, ComponentScopeState, HASH_SEPARATOR,
    LEASE_IDENTITY_DOMAIN, MAX_REGISTRATION_DEPENDENCIES, MAX_SCOPE_NON_CLAIMS,
    MAX_SCOPE_REGISTRATIONS, MAX_SCOPE_TRANSITIONS, REGISTRATION_IDENTITY_DOMAIN,
    REQUIRED_SCOPE_NON_CLAIMS, SCOPE_IDENTITY_DOMAIN, ScopeCleanupFailure, ScopeCleanupIssue,
    ScopeCleanupIssueKind, ScopeClosePlan, ScopeCloseRequest, ScopeEffectRequest, ScopeGeneration,
    ScopeMemberKind, ScopeObservation, ScopeResolutionRequest, ScopeResolutionTarget,
    ScopeTerminalOutcome, ScopeTransition, ScopeTransitionClass, ScopeVisibility,
};
use crate::composition_activation_graph::ValidatedActivationGraph;

const TEARDOWN_PRE_RELEASE_EFFECTS_PER_MEMBER: usize = 2;
const ADJACENT_REGISTRATION_PAIR_SIZE: usize = 2;

/// Admits one explicit root component scope against a validated activation graph.
///
/// # Errors
///
/// Returns a typed error for a parent-bearing root, malformed identities,
/// unknown components, undeclared dependencies, duplicates, cycles, or unsafe evidence.
pub fn admit_component_scope(
    graph: &ValidatedActivationGraph,
    draft: ComponentScopeDraft,
) -> Result<ComponentScope, ComponentScopeError> {
    if draft.parent.is_some() {
        return Err(ComponentScopeError::OutOfScope);
    }
    admit_scope(graph, draft)
}

/// Admits one child scope through an exact open parent registration.
///
/// # Errors
///
/// Returns a typed error when the parent binding is stale, the parent is not
/// open, the parent does not own the child registration, or scope admission fails.
pub fn admit_child_component_scope(
    graph: &ValidatedActivationGraph,
    parent: &ComponentScope,
    draft: ComponentScopeDraft,
) -> Result<ComponentScope, ComponentScopeError> {
    if !parent.resolution_open() {
        return Err(ComponentScopeError::InvalidState);
    }
    let parent_binding = draft
        .parent
        .as_ref()
        .ok_or(ComponentScopeError::OutOfScope)?;
    validate_scope_binding(
        parent,
        parent_binding.scope_id.as_str(),
        parent_binding.generation,
    )?;
    let parent_owns_child = parent.registrations.iter().any(|registration| {
        registration.member_kind == ScopeMemberKind::ChildScope
            && registration.member_id == draft.scope_id
    });
    if !parent_owns_child {
        return Err(ComponentScopeError::OutOfScope);
    }
    admit_scope(graph, draft)
}

fn admit_scope(
    graph: &ValidatedActivationGraph,
    draft: ComponentScopeDraft,
) -> Result<ComponentScope, ComponentScopeError> {
    validate_label(draft.scope_id.as_str())?;
    validate_hash(graph.identity())?;
    validate_parent(draft.parent.as_ref())?;
    validate_non_claims(draft.non_claims.as_slice())?;
    if draft.registrations.is_empty() || draft.registrations.len() > MAX_SCOPE_REGISTRATIONS {
        return Err(ComponentScopeError::BoundExceeded);
    }

    let graph_components: BTreeSet<&str> = graph
        .components()
        .iter()
        .map(|component| component.component_id.as_str())
        .collect();
    let mut drafts = draft.registrations;
    drafts.sort_by(|left, right| left.member_id.cmp(&right.member_id));
    let mut non_claims = draft.non_claims;
    non_claims.sort();
    let registrations = validate_registrations(graph, drafts, &graph_components)?;
    let release_order = derive_release_order(registrations.as_slice())?;
    let identity = scope_identity(&ScopeIdentityInput {
        scope_id: draft.scope_id.as_str(),
        generation: draft.generation,
        graph_identity: graph.identity(),
        parent: draft.parent.as_ref(),
        registrations: registrations.as_slice(),
        non_claims: non_claims.as_slice(),
    });

    Ok(ComponentScope {
        scope_id: draft.scope_id,
        generation: draft.generation,
        graph_identity: graph.identity().to_string(),
        parent: draft.parent,
        registrations,
        release_order,
        state: ComponentScopeState::Open,
        close_plan: None,
        quiescent_member_ids: BTreeSet::new(),
        released_member_ids: Vec::new(),
        unresolved_cleanup: Vec::new(),
        transition_classes: alloc::vec![ScopeTransitionClass::ScopeAdmitted],
        requested_effect_count: 0,
        terminal_outcome: None,
        terminal_receipt_identity: None,
        terminal_receipt_committed: false,
        non_claims,
        identity,
    })
}

/// Resolves one exact component from the current scope or one explicit parent.
///
/// The function has no global registry, process current scope, thread-local
/// context, name-only fallback, or effect authority.
///
/// # Errors
///
/// Returns a typed error for ambient, stale, closing, out-of-scope, unknown, or
/// hidden component lookup.
pub fn resolve_component(
    current: &ComponentScope,
    explicit_parent: Option<&ComponentScope>,
    request: &ScopeResolutionRequest,
) -> Result<ComponentLease, ComponentScopeError> {
    if request.requester_scope_id != current.scope_id {
        return Err(ComponentScopeError::ScopeMismatch);
    }
    if request.requester_generation != current.generation {
        return Err(ComponentScopeError::GenerationMismatch);
    }
    validate_label(request.component_id.as_str())?;

    let (target, parent_lookup) = match &request.target {
        ScopeResolutionTarget::Ambient => {
            return Err(ComponentScopeError::AmbientResolutionDenied);
        }
        ScopeResolutionTarget::Exact {
            scope_id,
            generation,
        } if scope_id == &current.scope_id => {
            if generation != &current.generation {
                return Err(ComponentScopeError::GenerationMismatch);
            }
            (current, false)
        }
        ScopeResolutionTarget::Exact {
            scope_id,
            generation,
        } => {
            let parent_binding = current
                .parent
                .as_ref()
                .ok_or(ComponentScopeError::OutOfScope)?;
            if scope_id != &parent_binding.scope_id {
                return Err(ComponentScopeError::OutOfScope);
            }
            if generation != &parent_binding.generation {
                return Err(ComponentScopeError::GenerationMismatch);
            }
            let parent = explicit_parent.ok_or(ComponentScopeError::OutOfScope)?;
            if parent.scope_id != parent_binding.scope_id {
                return Err(ComponentScopeError::ScopeMismatch);
            }
            if parent.generation != parent_binding.generation {
                return Err(ComponentScopeError::GenerationMismatch);
            }
            (parent, true)
        }
    };

    if !target.resolution_open() {
        return Err(ComponentScopeError::InvalidState);
    }
    let registration = target
        .registrations
        .iter()
        .find(|registration| {
            registration.member_kind == ScopeMemberKind::Component
                && registration.component_id == request.component_id
        })
        .ok_or(ComponentScopeError::ComponentNotVisible)?;
    if parent_lookup && registration.visibility != ScopeVisibility::Descendants {
        return Err(ComponentScopeError::ComponentNotVisible);
    }

    let identity = lease_identity(target, registration);
    Ok(ComponentLease {
        scope_id: target.scope_id.clone(),
        generation: target.generation,
        component_id: registration.component_id.clone(),
        registration_identity: registration.identity.clone(),
        identity,
        effect_authority_granted: false,
    })
}

/// Starts or reuses one deterministic generation-bound close plan.
///
/// # Errors
///
/// Returns a typed error when scope or generation identity differs.
pub fn request_component_scope_close(
    scope: &ComponentScope,
    request: &ScopeCloseRequest,
) -> Result<ScopeTransition, ComponentScopeError> {
    validate_scope_binding(scope, request.scope_id.as_str(), request.generation)?;
    if scope.state != ComponentScopeState::Open {
        if scope.close_plan.is_none() {
            return Err(ComponentScopeError::InvalidState);
        }
        return Ok(ScopeTransition::new(
            scope.clone(),
            ScopeTransitionClass::CloseReused,
            Vec::new(),
        ));
    }

    let plan = ScopeClosePlan {
        identity: close_plan_identity(scope),
        generation: scope.generation,
        release_order: scope.release_order.clone(),
    };
    let effect = ScopeEffectRequest::CloseDelivery {
        scope_id: scope.scope_id.clone(),
        generation: scope.generation,
        plan_identity: plan.identity.clone(),
    };
    let mut next = scope.clone();
    next.state = ComponentScopeState::Closing;
    next.close_plan = Some(plan);
    record_transition(
        &mut next,
        ScopeTransitionClass::ResolutionRevoked,
        core::slice::from_ref(&effect),
    )?;
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::ResolutionRevoked,
        alloc::vec![effect],
    ))
}

/// Applies one exact generation-bound host observation.
///
/// # Errors
///
/// Returns a typed error for stale, duplicate, unknown, out-of-order, or forged
/// observations.
pub fn reduce_component_scope_observation(
    scope: &ComponentScope,
    observation: &ScopeObservation,
) -> Result<ScopeTransition, ComponentScopeError> {
    validate_observation_plan(scope, observation)?;
    match observation {
        ScopeObservation::DeliveryClosed { .. } => reduce_delivery_closed(scope),
        ScopeObservation::MemberQuiescent { member_id, .. } => {
            reduce_member_quiescent(scope, member_id.as_str())
        }
        ScopeObservation::MemberReleased { member_id, .. } => {
            reduce_member_released(scope, member_id.as_str())
        }
        ScopeObservation::TerminalReceiptCommitted {
            receipt_identity, ..
        } => reduce_terminal_receipt_committed(scope, receipt_identity.as_str()),
    }
}

/// Ends cleanup with an explicit metadata-only unresolved outcome.
///
/// # Errors
///
/// Returns a typed error for a forged binding, unknown subject, terminal state,
/// or a failure class that does not match the current teardown phase.
pub fn fail_component_scope_cleanup(
    scope: &ComponentScope,
    failure: &ScopeCleanupFailure,
) -> Result<ScopeTransition, ComponentScopeError> {
    validate_scope_binding(scope, failure.scope_id.as_str(), failure.generation)?;
    validate_plan_identity(scope, failure.plan_identity.as_str())?;
    validate_cleanup_failure(scope, failure)?;

    let unresolved_cleanup = unresolved_cleanup_for(scope, failure);
    if unresolved_cleanup.is_empty() {
        return Err(ComponentScopeError::CleanupIncomplete);
    }
    let mut next = scope.clone();
    next.state = ComponentScopeState::FailedCleanup;
    next.unresolved_cleanup = unresolved_cleanup;
    next.terminal_outcome = Some(ScopeTerminalOutcome::FailedCleanup);
    push_transition(&mut next, ScopeTransitionClass::CleanupFailed)?;
    push_transition(&mut next, ScopeTransitionClass::TerminalCommitRequested)?;
    next.requested_effect_count = next
        .requested_effect_count
        .checked_add(1)
        .ok_or(ComponentScopeError::BoundExceeded)?;
    let receipt = super::redaction::project_component_scope_receipt(&next)?;
    next.terminal_receipt_identity = Some(receipt.identity.clone());
    let effect = ScopeEffectRequest::CommitTerminalReceipt {
        scope_id: next.scope_id.clone(),
        generation: next.generation,
        plan_identity: failure.plan_identity.clone(),
        receipt_identity: receipt.identity,
        outcome: ScopeTerminalOutcome::FailedCleanup,
    };
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::CleanupFailed,
        alloc::vec![effect],
    ))
}

fn reduce_delivery_closed(scope: &ComponentScope) -> Result<ScopeTransition, ComponentScopeError> {
    if scope.state != ComponentScopeState::Closing {
        return Err(ComponentScopeError::InvalidObservation);
    }
    let plan = required_plan(scope)?;
    let mut effects = Vec::with_capacity(
        plan.release_order
            .len()
            .saturating_mul(TEARDOWN_PRE_RELEASE_EFFECTS_PER_MEMBER),
    );
    for member_id in &plan.release_order {
        effects.push(ScopeEffectRequest::RequestCancellation {
            scope_id: scope.scope_id.clone(),
            generation: scope.generation,
            plan_identity: plan.identity.clone(),
            member_id: member_id.clone(),
        });
    }
    for member_id in &plan.release_order {
        effects.push(ScopeEffectRequest::ObserveQuiescence {
            scope_id: scope.scope_id.clone(),
            generation: scope.generation,
            plan_identity: plan.identity.clone(),
            member_id: member_id.clone(),
        });
    }
    let mut next = scope.clone();
    next.state = ComponentScopeState::Quiescing;
    record_transition(
        &mut next,
        ScopeTransitionClass::DeliveryClosed,
        effects.as_slice(),
    )?;
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::DeliveryClosed,
        effects,
    ))
}

fn reduce_member_quiescent(
    scope: &ComponentScope,
    member_id: &str,
) -> Result<ScopeTransition, ComponentScopeError> {
    if scope.state != ComponentScopeState::Quiescing {
        return Err(ComponentScopeError::InvalidObservation);
    }
    validate_member(scope, member_id)?;
    if scope.quiescent_member_ids.contains(member_id) {
        return Err(ComponentScopeError::DuplicateObservation);
    }
    let mut next = scope.clone();
    next.quiescent_member_ids.insert(member_id.to_string());
    push_transition(&mut next, ScopeTransitionClass::MemberQuiescent)?;
    if next.quiescent_member_ids.len() != next.registrations.len() {
        return Ok(ScopeTransition::new(
            next,
            ScopeTransitionClass::MemberQuiescent,
            Vec::new(),
        ));
    }

    next.state = ComponentScopeState::Releasing;
    push_transition(&mut next, ScopeTransitionClass::ReleaseStarted)?;
    let plan = required_plan(&next)?;
    let first_member = plan
        .release_order
        .first()
        .ok_or(ComponentScopeError::InvalidState)?;
    let effect = ScopeEffectRequest::ReleaseMember {
        scope_id: next.scope_id.clone(),
        generation: next.generation,
        plan_identity: plan.identity.clone(),
        member_id: first_member.clone(),
    };
    next.requested_effect_count = next
        .requested_effect_count
        .checked_add(1)
        .ok_or(ComponentScopeError::BoundExceeded)?;
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::ReleaseStarted,
        alloc::vec![effect],
    ))
}

fn reduce_member_released(
    scope: &ComponentScope,
    member_id: &str,
) -> Result<ScopeTransition, ComponentScopeError> {
    if scope.state != ComponentScopeState::Releasing || scope.terminal_outcome.is_some() {
        return Err(ComponentScopeError::InvalidObservation);
    }
    validate_member(scope, member_id)?;
    if scope
        .released_member_ids
        .iter()
        .any(|released| released == member_id)
    {
        return Err(ComponentScopeError::DuplicateObservation);
    }
    let plan = required_plan(scope)?;
    let expected = plan
        .release_order
        .get(scope.released_member_ids.len())
        .ok_or(ComponentScopeError::InvalidObservation)?;
    if expected != member_id {
        return Err(ComponentScopeError::InvalidObservation);
    }

    let mut next = scope.clone();
    next.released_member_ids.push(member_id.to_string());
    push_transition(&mut next, ScopeTransitionClass::MemberReleased)?;
    if next.released_member_ids.len() != plan.release_order.len() {
        let next_member = plan
            .release_order
            .get(next.released_member_ids.len())
            .ok_or(ComponentScopeError::InvalidObservation)?;
        let effect = ScopeEffectRequest::ReleaseMember {
            scope_id: next.scope_id.clone(),
            generation: next.generation,
            plan_identity: plan.identity.clone(),
            member_id: next_member.clone(),
        };
        next.requested_effect_count = next
            .requested_effect_count
            .checked_add(1)
            .ok_or(ComponentScopeError::BoundExceeded)?;
        return Ok(ScopeTransition::new(
            next,
            ScopeTransitionClass::MemberReleased,
            alloc::vec![effect],
        ));
    }

    next.terminal_outcome = Some(ScopeTerminalOutcome::Closed);
    push_transition(&mut next, ScopeTransitionClass::TerminalCommitRequested)?;
    next.requested_effect_count = next
        .requested_effect_count
        .checked_add(1)
        .ok_or(ComponentScopeError::BoundExceeded)?;
    let receipt = super::redaction::project_component_scope_receipt(&next)?;
    next.terminal_receipt_identity = Some(receipt.identity.clone());
    let effect = ScopeEffectRequest::CommitTerminalReceipt {
        scope_id: next.scope_id.clone(),
        generation: next.generation,
        plan_identity: plan.identity.clone(),
        receipt_identity: receipt.identity,
        outcome: ScopeTerminalOutcome::Closed,
    };
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::TerminalCommitRequested,
        alloc::vec![effect],
    ))
}

fn reduce_terminal_receipt_committed(
    scope: &ComponentScope,
    receipt_identity: &str,
) -> Result<ScopeTransition, ComponentScopeError> {
    if scope.terminal_receipt_committed {
        return Err(ComponentScopeError::DuplicateObservation);
    }
    let expected = scope
        .terminal_receipt_identity
        .as_deref()
        .ok_or(ComponentScopeError::InvalidObservation)?;
    if expected != receipt_identity {
        return Err(ComponentScopeError::InvalidObservation);
    }
    let outcome = scope
        .terminal_outcome
        .ok_or(ComponentScopeError::InvalidObservation)?;
    let mut next = scope.clone();
    next.terminal_receipt_committed = true;
    if outcome == ScopeTerminalOutcome::Closed {
        next.state = ComponentScopeState::Closed;
    }
    push_transition(&mut next, ScopeTransitionClass::TerminalCommitObserved)?;
    Ok(ScopeTransition::new(
        next,
        ScopeTransitionClass::TerminalCommitObserved,
        Vec::new(),
    ))
}

fn validate_registrations(
    graph: &ValidatedActivationGraph,
    drafts: Vec<ComponentRegistrationDraft>,
    graph_components: &BTreeSet<&str>,
) -> Result<Vec<ComponentRegistration>, ComponentScopeError> {
    let mut members = BTreeMap::new();
    let mut component_members = BTreeSet::new();
    for draft in &drafts {
        validate_registration_shape(draft, graph_components)?;
        if members
            .insert(draft.member_id.clone(), draft.clone())
            .is_some()
        {
            return Err(ComponentScopeError::DuplicateRegistration);
        }
        if draft.member_kind == ScopeMemberKind::Component
            && !component_members.insert(draft.component_id.as_str())
        {
            return Err(ComponentScopeError::DuplicateRegistration);
        }
    }

    let mut registrations = Vec::with_capacity(drafts.len());
    for mut draft in drafts {
        draft.dependency_member_ids.sort();
        if draft
            .dependency_member_ids
            .windows(ADJACENT_REGISTRATION_PAIR_SIZE)
            .any(|pair| pair[0] == pair[1])
        {
            return Err(ComponentScopeError::DuplicateRegistration);
        }
        for dependency_member_id in &draft.dependency_member_ids {
            let dependency = members
                .get(dependency_member_id.as_str())
                .ok_or(ComponentScopeError::UnknownDependency)?;
            if dependency.member_id == draft.member_id {
                return Err(ComponentScopeError::DependencyCycle);
            }
            if !activation_dependency_admitted(graph, &draft, dependency) {
                return Err(ComponentScopeError::UndeclaredDependency);
            }
        }
        let identity = registration_identity(&draft);
        registrations.push(ComponentRegistration {
            member_id: draft.member_id,
            component_id: draft.component_id,
            member_kind: draft.member_kind,
            visibility: draft.visibility,
            dependency_member_ids: draft.dependency_member_ids,
            identity,
        });
    }
    Ok(registrations)
}

fn validate_registration_shape(
    draft: &ComponentRegistrationDraft,
    graph_components: &BTreeSet<&str>,
) -> Result<(), ComponentScopeError> {
    validate_label(draft.member_id.as_str())?;
    validate_label(draft.component_id.as_str())?;
    if !graph_components.contains(draft.component_id.as_str()) {
        return Err(ComponentScopeError::UnknownComponent);
    }
    if draft.dependency_member_ids.len() > MAX_REGISTRATION_DEPENDENCIES {
        return Err(ComponentScopeError::BoundExceeded);
    }
    for dependency in &draft.dependency_member_ids {
        validate_label(dependency.as_str())?;
    }
    match draft.member_kind {
        ScopeMemberKind::Component if draft.member_id != draft.component_id => {
            Err(ComponentScopeError::InvalidIdentity)
        }
        ScopeMemberKind::ChildScope if draft.visibility != ScopeVisibility::LocalOnly => {
            Err(ComponentScopeError::InvalidIdentity)
        }
        ScopeMemberKind::Component | ScopeMemberKind::ChildScope => Ok(()),
    }
}

fn activation_dependency_admitted(
    graph: &ValidatedActivationGraph,
    dependent: &ComponentRegistrationDraft,
    dependency: &ComponentRegistrationDraft,
) -> bool {
    if dependent.member_kind == ScopeMemberKind::ChildScope
        && dependent.component_id == dependency.component_id
    {
        return true;
    }
    let port_dependency = graph.ports().iter().any(|port| {
        let dependent_uses_port = port.owner_component_id == dependent.component_id
            || port
                .consumer_component_ids
                .iter()
                .any(|consumer| consumer == &dependent.component_id);
        let dependency_provides_port = port
            .provider_component_ids
            .iter()
            .any(|provider| provider == &dependency.component_id);
        dependent_uses_port && dependency_provides_port
    });
    let event_dependency = graph.events().iter().any(|event| {
        event.producer_component_id == dependency.component_id
            && event
                .consumer_component_ids
                .iter()
                .any(|consumer| consumer == &dependent.component_id)
    });
    port_dependency || event_dependency
}

fn derive_release_order(
    registrations: &[ComponentRegistration],
) -> Result<Vec<String>, ComponentScopeError> {
    let mut remaining: BTreeSet<&str> = registrations
        .iter()
        .map(ComponentRegistration::member_id)
        .collect();
    let dependencies: BTreeMap<&str, BTreeSet<&str>> = registrations
        .iter()
        .map(|registration| {
            (
                registration.member_id(),
                registration
                    .dependency_member_ids()
                    .iter()
                    .map(String::as_str)
                    .collect(),
            )
        })
        .collect();
    let mut admitted = BTreeSet::new();
    let mut dependency_first = Vec::with_capacity(registrations.len());
    while !remaining.is_empty() {
        let next = remaining.iter().copied().find(|member_id| {
            dependencies
                .get(member_id)
                .is_some_and(|required| required.is_subset(&admitted))
        });
        let member_id = next.ok_or(ComponentScopeError::DependencyCycle)?;
        remaining.remove(member_id);
        admitted.insert(member_id);
        dependency_first.push(member_id.to_string());
    }
    dependency_first.reverse();
    Ok(dependency_first)
}

fn validate_observation_plan(
    scope: &ComponentScope,
    observation: &ScopeObservation,
) -> Result<(), ComponentScopeError> {
    let (scope_id, generation, plan_identity) = match observation {
        ScopeObservation::DeliveryClosed {
            scope_id,
            generation,
            plan_identity,
        }
        | ScopeObservation::MemberQuiescent {
            scope_id,
            generation,
            plan_identity,
            ..
        }
        | ScopeObservation::MemberReleased {
            scope_id,
            generation,
            plan_identity,
            ..
        }
        | ScopeObservation::TerminalReceiptCommitted {
            scope_id,
            generation,
            plan_identity,
            ..
        } => (scope_id.as_str(), *generation, plan_identity.as_str()),
    };
    validate_scope_binding(scope, scope_id, generation)?;
    validate_plan_identity(scope, plan_identity)
}

fn validate_scope_binding(
    scope: &ComponentScope,
    scope_id: &str,
    generation: ScopeGeneration,
) -> Result<(), ComponentScopeError> {
    if scope.scope_id != scope_id {
        return Err(ComponentScopeError::ScopeMismatch);
    }
    if scope.generation != generation {
        return Err(ComponentScopeError::GenerationMismatch);
    }
    Ok(())
}

fn validate_plan_identity(
    scope: &ComponentScope,
    plan_identity: &str,
) -> Result<(), ComponentScopeError> {
    let plan = required_plan(scope)?;
    if plan.identity != plan_identity {
        return Err(ComponentScopeError::InvalidObservation);
    }
    Ok(())
}

fn validate_cleanup_failure(
    scope: &ComponentScope,
    failure: &ScopeCleanupFailure,
) -> Result<(), ComponentScopeError> {
    let state_admits = matches!(
        (scope.state, failure.kind),
        (
            ComponentScopeState::Closing,
            ScopeCleanupIssueKind::DeliveryClose
        ) | (
            ComponentScopeState::Quiescing,
            ScopeCleanupIssueKind::CancellationRequest | ScopeCleanupIssueKind::QuiescenceDeadline
        ) | (
            ComponentScopeState::Releasing,
            ScopeCleanupIssueKind::ReleaseObservation
        )
    );
    if !state_admits || scope.terminal_outcome.is_some() {
        return Err(ComponentScopeError::InvalidState);
    }
    if failure.kind == ScopeCleanupIssueKind::DeliveryClose {
        if failure.subject != scope.scope_id {
            return Err(ComponentScopeError::ScopeMismatch);
        }
    } else {
        validate_member(scope, failure.subject.as_str())?;
    }
    Ok(())
}

fn unresolved_cleanup_for(
    scope: &ComponentScope,
    failure: &ScopeCleanupFailure,
) -> Vec<ScopeCleanupIssue> {
    match scope.state {
        ComponentScopeState::Closing => alloc::vec![ScopeCleanupIssue {
            kind: ScopeCleanupIssueKind::DeliveryClose,
            subject: scope.scope_id.clone(),
        }],
        ComponentScopeState::Quiescing => scope
            .release_order
            .iter()
            .filter(|member_id| !scope.quiescent_member_ids.contains(member_id.as_str()))
            .map(|member_id| ScopeCleanupIssue {
                kind: if member_id == &failure.subject {
                    failure.kind
                } else {
                    ScopeCleanupIssueKind::QuiescenceDeadline
                },
                subject: member_id.clone(),
            })
            .collect(),
        ComponentScopeState::Releasing => scope
            .release_order
            .iter()
            .filter(|member_id| {
                !scope
                    .released_member_ids
                    .iter()
                    .any(|released| released == *member_id)
            })
            .map(|member_id| ScopeCleanupIssue {
                kind: ScopeCleanupIssueKind::ReleaseObservation,
                subject: member_id.clone(),
            })
            .collect(),
        ComponentScopeState::Open
        | ComponentScopeState::Closed
        | ComponentScopeState::FailedCleanup => Vec::new(),
    }
}

fn validate_member(scope: &ComponentScope, member_id: &str) -> Result<(), ComponentScopeError> {
    if scope
        .registrations
        .iter()
        .any(|registration| registration.member_id == member_id)
    {
        Ok(())
    } else {
        Err(ComponentScopeError::UnknownMember)
    }
}

fn required_plan(scope: &ComponentScope) -> Result<&ScopeClosePlan, ComponentScopeError> {
    scope
        .close_plan
        .as_ref()
        .ok_or(ComponentScopeError::InvalidState)
}

fn record_transition(
    scope: &mut ComponentScope,
    class: ScopeTransitionClass,
    effects: &[ScopeEffectRequest],
) -> Result<(), ComponentScopeError> {
    push_transition(scope, class)?;
    scope.requested_effect_count = scope
        .requested_effect_count
        .checked_add(effects.len())
        .ok_or(ComponentScopeError::BoundExceeded)?;
    Ok(())
}

fn push_transition(
    scope: &mut ComponentScope,
    class: ScopeTransitionClass,
) -> Result<(), ComponentScopeError> {
    if scope.transition_classes.len() >= MAX_SCOPE_TRANSITIONS {
        return Err(ComponentScopeError::BoundExceeded);
    }
    scope.transition_classes.push(class);
    Ok(())
}

fn validate_parent(parent: Option<&super::data::ScopeParent>) -> Result<(), ComponentScopeError> {
    if let Some(parent) = parent {
        validate_label(parent.scope_id.as_str())?;
    }
    Ok(())
}

fn validate_non_claims(non_claims: &[String]) -> Result<(), ComponentScopeError> {
    if non_claims.len() > MAX_SCOPE_NON_CLAIMS {
        return Err(ComponentScopeError::BoundExceeded);
    }
    let unique: BTreeSet<&str> = non_claims.iter().map(String::as_str).collect();
    if unique.len() != non_claims.len() {
        return Err(ComponentScopeError::UnsafeEvidence);
    }
    for non_claim in non_claims {
        super::redaction::validate_component_scope_evidence_text(non_claim.as_str())?;
    }
    if REQUIRED_SCOPE_NON_CLAIMS.iter().any(|required| {
        !non_claims
            .iter()
            .any(|non_claim| non_claim.as_str() == *required)
    }) {
        return Err(ComponentScopeError::UnsafeEvidence);
    }
    Ok(())
}

pub(crate) fn validate_label(value: &str) -> Result<(), ComponentScopeError> {
    if value.is_empty()
        || value.len() > super::data::MAX_SCOPE_LABEL_BYTES
        || value
            .bytes()
            .any(|byte| byte == HASH_SEPARATOR || byte.is_ascii_control())
    {
        return Err(ComponentScopeError::InvalidIdentity);
    }
    super::redaction::validate_component_scope_evidence_text(value)
}

pub(crate) fn validate_hash(value: &str) -> Result<(), ComponentScopeError> {
    const BLAKE3_HEX_LENGTH: usize = 64;
    if value.len() != BLAKE3_HEX_LENGTH
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ComponentScopeError::InvalidIdentity);
    }
    Ok(())
}

struct ScopeIdentityInput<'input> {
    scope_id: &'input str,
    generation: ScopeGeneration,
    graph_identity: &'input str,
    parent: Option<&'input super::data::ScopeParent>,
    registrations: &'input [ComponentRegistration],
    non_claims: &'input [String],
}

fn scope_identity(input: &ScopeIdentityInput<'_>) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, SCOPE_IDENTITY_DOMAIN);
    update_str(&mut hasher, input.scope_id);
    update_u64(&mut hasher, input.generation.get());
    update_str(&mut hasher, input.graph_identity);
    if let Some(parent) = input.parent {
        update_str(&mut hasher, parent.scope_id.as_str());
        update_u64(&mut hasher, parent.generation.get());
    }
    for registration in input.registrations {
        update_str(&mut hasher, registration.identity.as_str());
    }
    for non_claim in input.non_claims {
        update_str(&mut hasher, non_claim.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn registration_identity(draft: &ComponentRegistrationDraft) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, REGISTRATION_IDENTITY_DOMAIN);
    update_str(&mut hasher, draft.member_id.as_str());
    update_str(&mut hasher, draft.component_id.as_str());
    update_str(&mut hasher, draft.member_kind.as_str());
    update_str(&mut hasher, draft.visibility.as_str());
    for dependency in &draft.dependency_member_ids {
        update_str(&mut hasher, dependency.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn close_plan_identity(scope: &ComponentScope) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, CLOSE_PLAN_IDENTITY_DOMAIN);
    update_str(&mut hasher, scope.identity.as_str());
    update_u64(&mut hasher, scope.generation.get());
    for member_id in &scope.release_order {
        update_str(&mut hasher, member_id.as_str());
    }
    hasher.finalize().to_hex().to_string()
}

fn lease_identity(scope: &ComponentScope, registration: &ComponentRegistration) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, LEASE_IDENTITY_DOMAIN);
    update_str(&mut hasher, scope.identity.as_str());
    update_u64(&mut hasher, scope.generation.get());
    update_str(&mut hasher, registration.identity.as_str());
    hasher.finalize().to_hex().to_string()
}

pub(crate) fn update_str(hasher: &mut Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(&[HASH_SEPARATOR]);
}

pub(crate) fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
    hasher.update(&[HASH_SEPARATOR]);
}
