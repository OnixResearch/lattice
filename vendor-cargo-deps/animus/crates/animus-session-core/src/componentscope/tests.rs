use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::*;
use crate::composition_activation_graph::{
    ActivationAuthorityClass, ActivationComponent, ActivationDeliveryClass,
    ActivationDurabilityClass, ActivationEventRoute, ActivationGraphDraft, ActivationPort,
    ActivationPosture, PortMultiplicity, ValidatedActivationGraph,
};

const CURRENT_GENERATION_VALUE: u64 = 7;
const STALE_GENERATION_VALUE: u64 = 6;
const PROVIDER: &str = "component.provider";
const CONSUMER: &str = "component.consumer";
const CHILD_SCOPE: &str = "scope.child";
const ROOT_SCOPE: &str = "scope.root";
const OTHER_SCOPE: &str = "scope.other";
const REQUIRED_ACTIVATION_NON_CLAIMS: &[&str] = &[
    "not-implementation-correctness",
    "not-runtime-liveness",
    "not-effect-success",
    "not-authority-grant",
    "not-production-readiness",
    "not-release-eligibility",
];
const REQUIRED_SCOPE_NON_CLAIMS: &[&str] = &[
    "not-effect-authority",
    "not-task-cancellation",
    "not-process-death",
    "not-resource-reclamation",
    "not-sandboxing",
    "not-production-readiness",
    "not-release-eligibility",
    "not-external-liveness",
];

#[test]
fn registration_and_explicit_parent_resolution_are_generation_fenced() {
    // r[verify agent_harness_runtime.component_scope.registration]
    // r[verify agent_harness_runtime.component_scope.resolution]
    let graph = graph(false);
    let parent = admit_component_scope(&graph, parent_scope_draft()).expect("parent scope");
    let child =
        admit_child_component_scope(&graph, &parent, child_scope_draft()).expect("child scope");

    let local = resolve_component(
        &child,
        Some(&parent),
        &resolution_request(CHILD_SCOPE, CHILD_SCOPE, CURRENT_GENERATION_VALUE, CONSUMER),
    )
    .expect("local component");
    assert_eq!(local.scope_id(), CHILD_SCOPE);
    assert_eq!(local.component_id(), CONSUMER);
    assert!(!local.grants_effect_authority());

    let inherited = resolve_component(
        &child,
        Some(&parent),
        &resolution_request(CHILD_SCOPE, ROOT_SCOPE, CURRENT_GENERATION_VALUE, PROVIDER),
    )
    .expect("explicit visible parent component");
    assert_eq!(inherited.scope_id(), ROOT_SCOPE);
    assert_eq!(inherited.component_id(), PROVIDER);
    assert!(!inherited.grants_effect_authority());

    let canonical = admit_component_scope(&graph, root_scope_draft()).expect("canonical scope");
    let mut reordered = root_scope_draft();
    reordered.registrations.reverse();
    reordered.non_claims.reverse();
    let reordered = admit_component_scope(&graph, reordered).expect("reordered scope");
    assert_eq!(canonical.identity(), reordered.identity());
    let canonical_close = request_component_scope_close(&canonical, &close_request(ROOT_SCOPE))
        .expect("canonical close");
    let reordered_close = request_component_scope_close(&reordered, &close_request(ROOT_SCOPE))
        .expect("reordered close");
    assert_eq!(
        canonical_close
            .scope()
            .close_plan()
            .expect("canonical plan")
            .release_order(),
        reordered_close
            .scope()
            .close_plan()
            .expect("reordered plan")
            .release_order()
    );
}

#[test]
fn clean_close_converges_and_releases_child_before_dependents() {
    // r[verify agent_harness_runtime.component_scope.teardown]
    // r[verify agent_harness_runtime.component_scope.concurrency]
    let scope = admitted_root_scope();
    let start = request_component_scope_close(&scope, &close_request(ROOT_SCOPE))
        .expect("close must start");
    let plan_identity = start
        .scope()
        .close_plan()
        .expect("close plan")
        .identity()
        .to_string();
    assert!(!start.scope().resolution_open());
    assert!(matches!(
        start.effects(),
        [ScopeEffectRequest::CloseDelivery { .. }]
    ));

    let repeated = request_component_scope_close(start.scope(), &close_request(ROOT_SCOPE))
        .expect("repeated close");
    assert_eq!(
        repeated.scope().close_plan().expect("same plan").identity(),
        plan_identity
    );
    assert_eq!(repeated.class(), ScopeTransitionClass::CloseReused);
    assert!(repeated.effects().is_empty());

    let delivery = reduce_component_scope_observation(
        start.scope(),
        &ScopeObservation::DeliveryClosed {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
            plan_identity: plan_identity.clone(),
        },
    )
    .expect("delivery close");
    assert_eq!(delivery.scope().state(), ComponentScopeState::Quiescing);

    let provider_quiet = quiesce(delivery.scope(), &plan_identity, PROVIDER);
    let consumer_quiet = quiesce(provider_quiet.scope(), &plan_identity, CONSUMER);
    let child_quiet = quiesce(consumer_quiet.scope(), &plan_identity, CHILD_SCOPE);
    assert_eq!(child_quiet.scope().state(), ComponentScopeState::Releasing);
    assert!(matches!(
        child_quiet.effects(),
        [ScopeEffectRequest::ReleaseMember { member_id, .. }] if member_id == CHILD_SCOPE
    ));

    let child_released = release(child_quiet.scope(), &plan_identity, CHILD_SCOPE);
    let consumer_released = release(child_released.scope(), &plan_identity, CONSUMER);
    let provider_released = release(consumer_released.scope(), &plan_identity, PROVIDER);
    let receipt_identity = provider_released
        .scope()
        .terminal_receipt_identity()
        .expect("receipt identity")
        .to_string();
    assert!(matches!(
        provider_released.effects(),
        [ScopeEffectRequest::CommitTerminalReceipt {
            outcome: ScopeTerminalOutcome::Closed,
            ..
        }]
    ));

    let committed = reduce_component_scope_observation(
        provider_released.scope(),
        &ScopeObservation::TerminalReceiptCommitted {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
            plan_identity: plan_identity.clone(),
            receipt_identity,
        },
    )
    .expect("receipt commit");
    assert_eq!(committed.scope().state(), ComponentScopeState::Closed);
    assert!(committed.scope().terminal_receipt_committed());
    let terminal_repeat =
        request_component_scope_close(committed.scope(), &close_request(ROOT_SCOPE))
            .expect("terminal close reuses plan");
    assert_eq!(
        terminal_repeat
            .scope()
            .close_plan()
            .expect("terminal plan")
            .identity(),
        plan_identity
    );
    assert!(terminal_repeat.effects().is_empty());
    let receipt = project_component_scope_receipt(committed.scope()).expect("safe receipt");
    assert_eq!(receipt.outcome, ScopeTerminalOutcome::Closed);
    assert_eq!(receipt.unresolved_cleanup, Vec::new());
}

#[test]
fn stale_ambient_out_of_scope_and_post_close_resolution_fail_closed() {
    // r[verify agent_harness_runtime.component_scope.resolution]
    // r[verify agent_harness_runtime.component_scope.verification]
    let graph = graph(false);
    assert_eq!(
        admit_component_scope(&graph, child_scope_draft()),
        Err(ComponentScopeError::OutOfScope)
    );
    let parent = admit_component_scope(&graph, parent_scope_draft()).expect("parent scope");
    let child =
        admit_child_component_scope(&graph, &parent, child_scope_draft()).expect("child scope");

    let stale = ScopeResolutionRequest {
        requester_scope_id: String::from(CHILD_SCOPE),
        requester_generation: ScopeGeneration::new(STALE_GENERATION_VALUE).expect("stale value"),
        target: ScopeResolutionTarget::Exact {
            scope_id: String::from(CHILD_SCOPE),
            generation: ScopeGeneration::new(STALE_GENERATION_VALUE).expect("stale value"),
        },
        component_id: String::from(CONSUMER),
    };
    assert_eq!(
        resolve_component(&child, Some(&parent), &stale),
        Err(ComponentScopeError::GenerationMismatch)
    );

    let ambient = ScopeResolutionRequest {
        requester_scope_id: String::from(CHILD_SCOPE),
        requester_generation: current_generation(),
        target: ScopeResolutionTarget::Ambient,
        component_id: String::from(PROVIDER),
    };
    assert_eq!(
        resolve_component(&child, Some(&parent), &ambient),
        Err(ComponentScopeError::AmbientResolutionDenied)
    );

    let out_of_scope =
        resolution_request(CHILD_SCOPE, OTHER_SCOPE, CURRENT_GENERATION_VALUE, PROVIDER);
    assert_eq!(
        resolve_component(&child, Some(&parent), &out_of_scope),
        Err(ComponentScopeError::OutOfScope)
    );

    let closing =
        request_component_scope_close(&child, &close_request(CHILD_SCOPE)).expect("close starts");
    assert_eq!(
        resolve_component(
            closing.scope(),
            Some(&parent),
            &resolution_request(CHILD_SCOPE, CHILD_SCOPE, CURRENT_GENERATION_VALUE, CONSUMER,),
        ),
        Err(ComponentScopeError::InvalidState)
    );
}

#[test]
fn wrong_generation_late_delivery_and_duplicate_or_out_of_order_release_are_rejected() {
    // r[verify agent_harness_runtime.component_scope.verification]
    let scope = admitted_root_scope();
    let close =
        request_component_scope_close(&scope, &close_request(ROOT_SCOPE)).expect("close starts");
    let plan = close
        .scope()
        .close_plan()
        .expect("plan")
        .identity()
        .to_string();
    let stale_delivery = ScopeObservation::DeliveryClosed {
        scope_id: String::from(ROOT_SCOPE),
        generation: ScopeGeneration::new(STALE_GENERATION_VALUE).expect("stale value"),
        plan_identity: plan.clone(),
    };
    assert_eq!(
        reduce_component_scope_observation(close.scope(), &stale_delivery),
        Err(ComponentScopeError::GenerationMismatch)
    );

    let delivery = reduce_component_scope_observation(
        close.scope(),
        &ScopeObservation::DeliveryClosed {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
            plan_identity: plan.clone(),
        },
    )
    .expect("delivery closes");
    assert_eq!(
        reduce_component_scope_observation(
            delivery.scope(),
            &ScopeObservation::DeliveryClosed {
                scope_id: String::from(ROOT_SCOPE),
                generation: current_generation(),
                plan_identity: plan.clone(),
            },
        ),
        Err(ComponentScopeError::InvalidObservation)
    );

    let first = quiesce(delivery.scope(), &plan, PROVIDER);
    let second = quiesce(first.scope(), &plan, CONSUMER);
    let third = quiesce(second.scope(), &plan, CHILD_SCOPE);
    assert_eq!(
        reduce_component_scope_observation(
            third.scope(),
            &ScopeObservation::MemberReleased {
                scope_id: String::from(ROOT_SCOPE),
                generation: current_generation(),
                plan_identity: plan.clone(),
                member_id: String::from(PROVIDER),
            },
        ),
        Err(ComponentScopeError::InvalidObservation)
    );
    let released = release(third.scope(), &plan, CHILD_SCOPE);
    assert_eq!(
        reduce_component_scope_observation(
            released.scope(),
            &ScopeObservation::MemberReleased {
                scope_id: String::from(ROOT_SCOPE),
                generation: current_generation(),
                plan_identity: plan,
                member_id: String::from(CHILD_SCOPE),
            },
        ),
        Err(ComponentScopeError::DuplicateObservation)
    );
}

#[test]
fn undeclared_dependencies_and_ownership_cycles_are_rejected() {
    // r[verify agent_harness_runtime.component_scope.teardown]
    // r[verify agent_harness_runtime.component_scope.verification]
    let mut undeclared = root_scope_draft();
    let provider = undeclared
        .registrations
        .iter_mut()
        .find(|registration| registration.member_id == PROVIDER)
        .expect("provider registration");
    provider.dependency_member_ids = alloc::vec![String::from(CONSUMER)];
    assert_eq!(
        admit_component_scope(&graph(false), undeclared),
        Err(ComponentScopeError::UndeclaredDependency)
    );

    let mut cyclic = root_scope_draft();
    let provider = cyclic
        .registrations
        .iter_mut()
        .find(|registration| registration.member_id == PROVIDER)
        .expect("provider registration");
    provider.dependency_member_ids = alloc::vec![String::from(CONSUMER)];
    assert_eq!(
        admit_component_scope(&graph(true), cyclic),
        Err(ComponentScopeError::DependencyCycle)
    );
}

#[test]
fn unresolved_quiescence_stays_visible_and_receipts_reject_private_fields() {
    // r[verify agent_harness_runtime.component_scope.evidence]
    // r[verify agent_harness_runtime.component_scope.verification]
    let scope = admitted_root_scope();
    let close =
        request_component_scope_close(&scope, &close_request(ROOT_SCOPE)).expect("close starts");
    let plan = close
        .scope()
        .close_plan()
        .expect("plan")
        .identity()
        .to_string();
    let delivery = reduce_component_scope_observation(
        close.scope(),
        &ScopeObservation::DeliveryClosed {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
            plan_identity: plan.clone(),
        },
    )
    .expect("delivery closes");
    let provider_quiet = quiesce(delivery.scope(), &plan, PROVIDER);
    let failed = fail_component_scope_cleanup(
        provider_quiet.scope(),
        &ScopeCleanupFailure {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
            plan_identity: plan.clone(),
            kind: ScopeCleanupIssueKind::QuiescenceDeadline,
            subject: String::from(CONSUMER),
        },
    )
    .expect("failed cleanup remains explicit");
    assert_eq!(failed.scope().state(), ComponentScopeState::FailedCleanup);
    assert!(!failed.scope().unresolved_cleanup().is_empty());
    let receipt = project_component_scope_receipt(failed.scope()).expect("failed receipt");
    assert_eq!(receipt.outcome, ScopeTerminalOutcome::FailedCleanup);
    assert!(
        receipt
            .unresolved_cleanup
            .iter()
            .any(|issue| issue.subject == CONSUMER)
    );
    assert_eq!(
        validate_component_scope_evidence_text("service handle=private"),
        Err(ComponentScopeError::UnsafeEvidence)
    );
    assert_eq!(
        validate_component_scope_evidence_text("prompt=private"),
        Err(ComponentScopeError::UnsafeEvidence)
    );
}

fn graph(admit_reverse_dependency: bool) -> ValidatedActivationGraph {
    let mut events = Vec::new();
    if admit_reverse_dependency {
        events.push(ActivationEventRoute {
            event_id: String::from("event.consumer-to-provider"),
            producer_component_id: String::from(CONSUMER),
            consumer_component_ids: alloc::vec![String::from(PROVIDER)],
            delivery: ActivationDeliveryClass::Ordered,
            durability: ActivationDurabilityClass::MetadataOnly,
            lifecycle_owner_component_id: String::from(PROVIDER),
            posture: ActivationPosture::Required,
            bounded_cycle: false,
        });
    }
    ValidatedActivationGraph::validate(ActivationGraphDraft {
        components: alloc::vec![component(PROVIDER), component(CONSUMER)],
        ports: alloc::vec![ActivationPort {
            port_id: String::from("port.provider"),
            owner_component_id: String::from(CONSUMER),
            provider_component_ids: alloc::vec![String::from(PROVIDER)],
            consumer_component_ids: alloc::vec![String::from(CONSUMER)],
            scope_class: String::from("scope"),
            multiplicity: PortMultiplicity::ExactlyOne,
            lifecycle_owner_component_id: String::from(CONSUMER),
            posture: ActivationPosture::Required,
            feature_gate: String::from("always"),
            authority: ActivationAuthorityClass::InjectedOnly,
            focused_check: String::from("component-scope"),
        }],
        events,
        non_claims: REQUIRED_ACTIVATION_NON_CLAIMS
            .iter()
            .copied()
            .map(String::from)
            .collect(),
    })
    .expect("activation graph")
}

fn component(component_id: &str) -> ActivationComponent {
    ActivationComponent {
        component_id: String::from(component_id),
        feature_gate: String::from("always"),
        authority: ActivationAuthorityClass::InjectedOnly,
        posture: ActivationPosture::Required,
    }
}

fn root_scope_draft() -> ComponentScopeDraft {
    ComponentScopeDraft {
        scope_id: String::from(ROOT_SCOPE),
        generation: current_generation(),
        parent: None,
        registrations: alloc::vec![
            ComponentRegistrationDraft {
                member_id: String::from(PROVIDER),
                component_id: String::from(PROVIDER),
                member_kind: ScopeMemberKind::Component,
                visibility: ScopeVisibility::Descendants,
                dependency_member_ids: Vec::new(),
            },
            ComponentRegistrationDraft {
                member_id: String::from(CONSUMER),
                component_id: String::from(CONSUMER),
                member_kind: ScopeMemberKind::Component,
                visibility: ScopeVisibility::LocalOnly,
                dependency_member_ids: alloc::vec![String::from(PROVIDER)],
            },
            ComponentRegistrationDraft {
                member_id: String::from(CHILD_SCOPE),
                component_id: String::from(CONSUMER),
                member_kind: ScopeMemberKind::ChildScope,
                visibility: ScopeVisibility::LocalOnly,
                dependency_member_ids: alloc::vec![String::from(CONSUMER)],
            },
        ],
        non_claims: scope_non_claims(),
    }
}

fn parent_scope_draft() -> ComponentScopeDraft {
    ComponentScopeDraft {
        scope_id: String::from(ROOT_SCOPE),
        generation: current_generation(),
        parent: None,
        registrations: alloc::vec![
            ComponentRegistrationDraft {
                member_id: String::from(PROVIDER),
                component_id: String::from(PROVIDER),
                member_kind: ScopeMemberKind::Component,
                visibility: ScopeVisibility::Descendants,
                dependency_member_ids: Vec::new(),
            },
            ComponentRegistrationDraft {
                member_id: String::from(CHILD_SCOPE),
                component_id: String::from(PROVIDER),
                member_kind: ScopeMemberKind::ChildScope,
                visibility: ScopeVisibility::LocalOnly,
                dependency_member_ids: alloc::vec![String::from(PROVIDER)],
            },
        ],
        non_claims: scope_non_claims(),
    }
}

fn child_scope_draft() -> ComponentScopeDraft {
    ComponentScopeDraft {
        scope_id: String::from(CHILD_SCOPE),
        generation: current_generation(),
        parent: Some(ScopeParent {
            scope_id: String::from(ROOT_SCOPE),
            generation: current_generation(),
        }),
        registrations: alloc::vec![ComponentRegistrationDraft {
            member_id: String::from(CONSUMER),
            component_id: String::from(CONSUMER),
            member_kind: ScopeMemberKind::Component,
            visibility: ScopeVisibility::LocalOnly,
            dependency_member_ids: Vec::new(),
        }],
        non_claims: scope_non_claims(),
    }
}

fn scope_non_claims() -> Vec<String> {
    REQUIRED_SCOPE_NON_CLAIMS
        .iter()
        .copied()
        .map(String::from)
        .collect()
}

fn admitted_root_scope() -> ComponentScope {
    admit_component_scope(&graph(false), root_scope_draft()).expect("root scope")
}

fn current_generation() -> ScopeGeneration {
    ScopeGeneration::new(CURRENT_GENERATION_VALUE).expect("current generation")
}

fn close_request(scope_id: &str) -> ScopeCloseRequest {
    ScopeCloseRequest {
        scope_id: String::from(scope_id),
        generation: current_generation(),
    }
}

fn resolution_request(
    requester_scope_id: &str,
    target_scope_id: &str,
    target_generation: u64,
    component_id: &str,
) -> ScopeResolutionRequest {
    ScopeResolutionRequest {
        requester_scope_id: String::from(requester_scope_id),
        requester_generation: current_generation(),
        target: ScopeResolutionTarget::Exact {
            scope_id: String::from(target_scope_id),
            generation: ScopeGeneration::new(target_generation).expect("target generation"),
        },
        component_id: String::from(component_id),
    }
}

fn quiesce(scope: &ComponentScope, plan_identity: &str, member_id: &str) -> ScopeTransition {
    reduce_component_scope_observation(
        scope,
        &ScopeObservation::MemberQuiescent {
            scope_id: String::from(scope.scope_id()),
            generation: scope.generation(),
            plan_identity: String::from(plan_identity),
            member_id: String::from(member_id),
        },
    )
    .expect("member quiescence")
}

fn release(scope: &ComponentScope, plan_identity: &str, member_id: &str) -> ScopeTransition {
    reduce_component_scope_observation(
        scope,
        &ScopeObservation::MemberReleased {
            scope_id: String::from(scope.scope_id()),
            generation: scope.generation(),
            plan_identity: String::from(plan_identity),
            member_id: String::from(member_id),
        },
    )
    .expect("member release")
}
