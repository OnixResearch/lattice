#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition
    )
)]

use super::*;
use crate::modelprofiles::tests::support::role_composed_plan;
use crate::modelprofiles::tests::support::role_composed_plan_with_admission;
use crate::modelprofiles::tests::support::tool_call_draft;

const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const POLICY_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SUBJECT_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const TASK_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const ACTOR_HASH: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const PARENT_HASH: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const RUN_HASH: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const SESSION_HASH: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const WORKSPACE_HASH: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const LINEAGE_HASH: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const MEMORY_HASH: &str = "9999999999999999999999999999999999999999999999999999999999999999";
const AGENT_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const GRANT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const FRESHNESS_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const OUTPUT_CONTRACT_HASH: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const EVIDENCE_HASH: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const ALTERNATE_HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const VERIFIER_HASH: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const RUBRIC_HASH: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";

const ROLE_SPECS: [(&str, AgentRoleClass); 6] = [
    ("implementer", AgentRoleClass::Implementation),
    ("reviewer", AgentRoleClass::Review),
    ("verifier", AgentRoleClass::Verification),
    ("judge", AgentRoleClass::Judgment),
    ("coordinator", AgentRoleClass::Coordination),
    ("observer", AgentRoleClass::Observation),
];
const KNOWN_CAPABILITIES: &[&str] = &["context_read", "mutation_intent", "tool_use"];
const CONTEXT_CAPABILITY: &[&str] = &["context_read"];
const REVIEW_CAPABILITIES: &[&str] = &["context_read", "tool_use"];
const IMPLEMENTER_CAPABILITIES: &[&str] = &["context_read", "mutation_intent", "tool_use"];
const MODEL_PROFILES: &[&str] = &["mock-tools-profile"];
const MODEL_TARGETS: &[&str] = &["mock:mock-tools"];
const PROMPT_MODES: &[&str] = &["standard"];
const PROMPT_SOURCES: &[&str] = &["host-policy", "model-guidance"];
const SKILLS: &[&str] = &["proof-search"];
const MEMORY_POLICIES: &[&str] = &[MEMORY_HASH];
const NO_TOOLS: &[&str] = &[];
const TOOLSETS: &[&str] = &["bounded-tools"];
const TOOL_CAPABILITIES: &[&str] = &["observe_read"];
const WORKSPACE_SCOPES: &[&str] = &["workspace:project"];
const INPUT_CONTRACTS: &[&str] = &["bounded-role-input"];
const OUTPUT_CONTRACTS: &[&str] = &["bounded-role-output"];
const EVIDENCE_CONTRACTS: &[&str] = &["metadata-only-evidence"];
const TOOL_NAMES: &[&str] = &["inspect_context"];
const EMPTY_LABELS: &[&str] = &[];
const SUBJECT_BOUND_DIMENSIONS: &[AgentRoleSeparationDimension] = &[AgentRoleSeparationDimension::SubjectImmutable];
const INDEPENDENT_DIMENSIONS: &[AgentRoleSeparationDimension] = &[
    AgentRoleSeparationDimension::Assignment,
    AgentRoleSeparationDimension::Actor,
    AgentRoleSeparationDimension::RunThread,
    AgentRoleSeparationDimension::Session,
    AgentRoleSeparationDimension::ModelProfile,
    AgentRoleSeparationDimension::Workspace,
    AgentRoleSeparationDimension::MutationLineage,
    AgentRoleSeparationDimension::CleanContext,
    AgentRoleSeparationDimension::SubjectImmutable,
];
const INDEPENDENT_VERIFIER_DIMENSIONS: &[AgentRoleSeparationDimension] = &[
    AgentRoleSeparationDimension::Assignment,
    AgentRoleSeparationDimension::Actor,
    AgentRoleSeparationDimension::RunThread,
    AgentRoleSeparationDimension::Session,
    AgentRoleSeparationDimension::ModelProfile,
    AgentRoleSeparationDimension::Workspace,
    AgentRoleSeparationDimension::MutationLineage,
    AgentRoleSeparationDimension::CleanContext,
    AgentRoleSeparationDimension::SubjectImmutable,
    AgentRoleSeparationDimension::VerifierImmutable,
];
const INDEPENDENT_JUDGE_DIMENSIONS: &[AgentRoleSeparationDimension] = &[
    AgentRoleSeparationDimension::Assignment,
    AgentRoleSeparationDimension::Actor,
    AgentRoleSeparationDimension::RunThread,
    AgentRoleSeparationDimension::Session,
    AgentRoleSeparationDimension::ModelProfile,
    AgentRoleSeparationDimension::Workspace,
    AgentRoleSeparationDimension::MutationLineage,
    AgentRoleSeparationDimension::CleanContext,
    AgentRoleSeparationDimension::SubjectImmutable,
    AgentRoleSeparationDimension::RubricImmutable,
];
const ALL_STATE_DIMENSIONS: &[AgentRoleSeparationDimension] = &[
    AgentRoleSeparationDimension::CleanContext,
    AgentRoleSeparationDimension::SubjectImmutable,
    AgentRoleSeparationDimension::VerifierImmutable,
    AgentRoleSeparationDimension::RubricImmutable,
];
const STATE_DIMENSIONS_WITHOUT_CLEAN_CONTEXT: &[AgentRoleSeparationDimension] = &[
    AgentRoleSeparationDimension::SubjectImmutable,
    AgentRoleSeparationDimension::VerifierImmutable,
    AgentRoleSeparationDimension::RubricImmutable,
];
const SEPARATION_POLICY_COUNT: usize = 4;
const ACTION_LIMIT: u32 = 32;
const OUTPUT_LIMIT: u16 = 8;
const CONTEXT_LIMIT: u32 = 4_096;

struct AdmittedFixture {
    catalog: AgentRoleCatalog,
    model_plan: AgentRoleBoundModelPlan,
    separation: AgentRoleSeparationDecision,
    assignment: AgentRoleAssignment,
    capabilities: AgentRoleCapabilityResolution,
    role_plan: AgentRolePlan,
}

#[test]
fn catalog_validates_baseline_and_specialized_roles() {
    let catalog = catalog();
    let implementer = catalog.profile("implementer").expect("implementer");
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let verifier = catalog.profile("verifier").expect("verifier");
    let judge = catalog.profile("judge").expect("judge");

    assert_eq!(catalog.profiles().len(), ROLE_SPECS.len());
    assert_eq!(implementer.class(), AgentRoleClass::Implementation);
    assert_eq!(reviewer.class(), AgentRoleClass::Review);
    assert_eq!(verifier.class(), AgentRoleClass::Verification);
    assert_eq!(judge.class(), AgentRoleClass::Judgment);
    assert_eq!(implementer.mutation_posture(), AgentRoleMutationPosture::ScopedIntent);
    assert_eq!(reviewer.mutation_posture(), AgentRoleMutationPosture::SubjectReadOnly);
    assert_ne!(implementer.profile_hash(), reviewer.profile_hash());
    assert_eq!(reviewer.expanded_tools(), &[String::from("inspect_context")]);
}

#[test]
fn catalog_rejects_stale_profile_identity() {
    let error = catalog_with_hash_override(Some(ZERO_HASH)).expect_err("stale role identity must fail closed");

    assert_eq!(error.issue(), AgentRoleIssue::StaleIdentity);
    assert_eq!(error.field(), "agent_role_profile_hash");
    assert!(error.detail().contains("stale"));
}

#[test]
fn role_model_composition_adds_an_independent_resource_ceiling() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let bound = role_composed_plan(reviewer).expect("role-bound model plan");

    assert_eq!(bound.role_profile_hash(), reviewer.profile_hash());
    assert_eq!(bound.plan().profile_hash().len(), ZERO_HASH.len());
    assert_eq!(bound.plan().tools().len(), 1);
    assert_ne!(bound.binding_hash(), bound.plan().plan_hash());
}

#[test]
fn separation_checks_explicit_dimensions_and_detects_same_actor() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let policy = catalog.separation_policy(reviewer.separation_policy_ref()).expect("review separation policy");
    let passed = evaluate_agent_role_separation(policy, &separation_observation(reviewer, false))
        .expect("valid separation observation");
    let failed = evaluate_agent_role_separation(policy, &separation_observation(reviewer, true))
        .expect("well-formed failed separation");
    let alternate_origin = evaluate_agent_role_separation(policy, &AgentRoleSeparationObservation {
        subject_actor_hash: ALTERNATE_HASH,
        ..separation_observation(reviewer, false)
    })
    .expect("alternate separated origin");

    assert!(passed.passed());
    assert!(passed.failed_dimensions().is_empty());
    assert!(!failed.passed());
    assert!(alternate_origin.passed());
    assert_ne!(passed.decision_hash(), alternate_origin.decision_hash());
    assert!(failed.failed_dimensions().contains(&AgentRoleSeparationDimension::Actor));
}

#[test]
fn exact_activation_authority_admits_assignment() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");

    assert_eq!(fixture.assignment.role_id(), reviewer.role_id());
    assert_eq!(fixture.assignment.role_profile_hash(), reviewer.profile_hash());
    assert_eq!(fixture.assignment.subject_hash(), SUBJECT_HASH);
    assert_eq!(fixture.assignment.separation_hash(), fixture.separation.decision_hash());
    assert_eq!(fixture.assignment.model_plan_hash(), fixture.model_plan.binding_hash());
}

#[test]
fn cross_subject_activation_authority_is_rejected() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let model_plan = role_composed_plan(reviewer).expect("role-bound model plan");
    let separation = separation_for(&catalog, reviewer);
    let correct_resource =
        agent_role_activation_resource(activation_resource_input(reviewer, &model_plan, &separation))
            .expect("correct activation resource");
    let wrong_resource = format!("role:{}:{}:{}", reviewer.role_id(), ALTERNATE_HASH, FRESHNESS_HASH);
    let (role_assignment_request, role_assignment_receipt) =
        authorized_role_request(correct_resource.as_str(), AGENT_ROLE_ASSIGN_ABILITY);
    let (activation_request, activation_receipt) =
        authorized_role_request(wrong_resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let error = admit_agent_role_assignment(assignment_request(
        reviewer,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &role_assignment_receipt,
            &activation_request,
            &activation_receipt,
        ),
    ))
    .expect_err("cross-subject authority must fail");

    assert_eq!(error.issue(), AgentRoleIssue::AuthorityDenied);
    assert!(error.detail().contains("exact allowed"));
}

#[test]
fn capability_resolution_never_unions_independent_ceilings() {
    let fixture = admitted_fixture("implementer");
    let implementer = fixture.catalog.profile("implementer").expect("implementer");
    let role_ceiling = typed_role_ceiling(implementer);
    let context_only = vec![context_capability()];
    let resolution = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment: &fixture.assignment,
        profile: implementer,
        requested: role_ceiling.as_slice(),
        role_ceiling: role_ceiling.as_slice(),
        ceilings: AgentRoleCapabilityCeilings {
            verified_grants: role_ceiling.as_slice(),
            host_policy: context_only.as_slice(),
            parent_delegation: role_ceiling.as_slice(),
            task_scope: role_ceiling.as_slice(),
            agent_definition: role_ceiling.as_slice(),
            model_profile: role_ceiling.as_slice(),
            workspace: role_ceiling.as_slice(),
            tool_executor: role_ceiling.as_slice(),
            approval: role_ceiling.as_slice(),
            sandbox: role_ceiling.as_slice(),
            cancellation: role_ceiling.as_slice(),
            timeout: role_ceiling.as_slice(),
            budget: role_ceiling.as_slice(),
        },
    })
    .expect("required context capability remains admitted");
    let mutation = mutation_capability();
    let error = admit_agent_role_capability_use(&fixture.assignment, &resolution, &mutation)
        .expect_err("one denied ceiling must deny use");

    assert_eq!(resolution.effective(), context_only.as_slice());
    assert!(resolution.denied().iter().any(|denial| {
        denial.capability() == &mutation && denial.source() == AgentRoleCapabilitySource::HostPolicy
    }));
    assert_eq!(error.issue(), AgentRoleIssue::CapabilityDenied);
}

#[test]
fn approval_sandbox_cancellation_timeout_budget_and_parent_are_independent_ceilings() {
    let fixture = admitted_fixture("implementer");
    let implementer = fixture.catalog.profile("implementer").expect("implementer");
    let role_ceiling = typed_role_ceiling(implementer);
    let context_only = vec![context_capability()];
    let mutation = mutation_capability();
    let base = uniform_ceilings(role_ceiling.as_slice());
    let cases = [
        (AgentRoleCapabilitySource::ParentDelegation, AgentRoleCapabilityCeilings {
            parent_delegation: context_only.as_slice(),
            ..base
        }),
        (AgentRoleCapabilitySource::Approval, AgentRoleCapabilityCeilings {
            approval: context_only.as_slice(),
            ..base
        }),
        (AgentRoleCapabilitySource::Sandbox, AgentRoleCapabilityCeilings {
            sandbox: context_only.as_slice(),
            ..base
        }),
        (AgentRoleCapabilitySource::Cancellation, AgentRoleCapabilityCeilings {
            cancellation: context_only.as_slice(),
            ..base
        }),
        (AgentRoleCapabilitySource::Timeout, AgentRoleCapabilityCeilings {
            timeout: context_only.as_slice(),
            ..base
        }),
        (AgentRoleCapabilitySource::Budget, AgentRoleCapabilityCeilings {
            budget: context_only.as_slice(),
            ..base
        }),
    ];
    for (source, ceilings) in cases {
        let resolution = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
            assignment: &fixture.assignment,
            profile: implementer,
            requested: role_ceiling.as_slice(),
            role_ceiling: role_ceiling.as_slice(),
            ceilings,
        })
        .expect("required context remains admitted");
        assert!(
            resolution
                .denied()
                .iter()
                .any(|denial| { denial.capability() == &mutation && denial.source() == source })
        );
        assert_eq!(resolution.effective(), context_only.as_slice());
    }
}

#[test]
fn role_plan_resume_requires_every_exact_identity() {
    let fixture = admitted_fixture("reviewer");
    let snapshot = AgentRolePlanSnapshot::from(&fixture.role_plan);
    let reused = evaluate_agent_role_transition(&AgentRoleTransitionRequest {
        kind: AgentRoleTransitionKind::Resume,
        current_plan: &fixture.role_plan,
        current_snapshot: snapshot,
        next_role_profile_hash: None,
        next_model_target_hash: None,
    })
    .expect("exact plan may resume");
    let stale_snapshot = AgentRolePlanSnapshot {
        policy_hash: ALTERNATE_HASH,
        ..snapshot
    };
    let stale_error = evaluate_agent_role_transition(&AgentRoleTransitionRequest {
        kind: AgentRoleTransitionKind::Replay,
        current_plan: &fixture.role_plan,
        current_snapshot: stale_snapshot,
        next_role_profile_hash: None,
        next_model_target_hash: None,
    })
    .expect_err("stale replay must replan");

    assert!(matches!(reused, AgentRoleTransitionDecision::ReuseExact { .. }));
    assert_eq!(stale_error.issue(), AgentRoleIssue::TransitionDenied);
}

#[test]
fn role_and_model_changes_require_recomposition() {
    let fixture = admitted_fixture("reviewer");
    let judge_hash = fixture.catalog.profile("judge").expect("judge").profile_hash();
    let snapshot = AgentRolePlanSnapshot::from(&fixture.role_plan);
    let role_change = evaluate_agent_role_transition(&AgentRoleTransitionRequest {
        kind: AgentRoleTransitionKind::RoleChange,
        current_plan: &fixture.role_plan,
        current_snapshot: snapshot,
        next_role_profile_hash: Some(judge_hash),
        next_model_target_hash: None,
    })
    .expect("role change decision");
    let fallback = evaluate_agent_role_transition(&AgentRoleTransitionRequest {
        kind: AgentRoleTransitionKind::ModelFallback,
        current_plan: &fixture.role_plan,
        current_snapshot: snapshot,
        next_role_profile_hash: None,
        next_model_target_hash: Some(ALTERNATE_HASH),
    })
    .expect("model fallback decision");
    let cancelled = evaluate_agent_role_transition(&AgentRoleTransitionRequest {
        kind: AgentRoleTransitionKind::Cancel,
        current_plan: &fixture.role_plan,
        current_snapshot: snapshot,
        next_role_profile_hash: None,
        next_model_target_hash: None,
    })
    .expect("cancellation decision");

    assert!(matches!(role_change, AgentRoleTransitionDecision::RecomposeRole { .. }));
    assert!(matches!(fallback, AgentRoleTransitionDecision::RecomposeModel { .. }));
    assert!(matches!(cancelled, AgentRoleTransitionDecision::Cancelled { .. }));
}

#[test]
fn handoff_preserves_only_bounded_metadata() {
    let fixture = admitted_fixture("implementer");
    let reviewer_hash = fixture.catalog.profile("reviewer").expect("reviewer").profile_hash();
    let evidence_refs = ["candidate-identity", "check-request"];
    let unresolved_intents = ["verify-output"];
    let requested_actions = ["review-candidate"];
    let handoff_resource = agent_role_handoff_resource(AgentRoleHandoffResourceInput {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: &evidence_refs,
        unresolved_intents: &unresolved_intents,
        requested_actions: &requested_actions,
    })
    .expect("handoff resource");
    let (handoff_request, handoff_receipt) =
        authorized_role_request(handoff_resource.as_str(), AGENT_ROLE_HANDOFF_ABILITY);
    let handoff = validate_agent_role_handoff(AgentRoleHandoffDraft {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: &evidence_refs,
        unresolved_intents: &unresolved_intents,
        requested_actions: &requested_actions,
        handoff_request: &handoff_request,
        handoff_receipt: &handoff_receipt,
        sensitive: AgentRoleHandoffSensitiveDraft::default(),
    })
    .expect("bounded handoff");
    let empty_resource = agent_role_handoff_resource(AgentRoleHandoffResourceInput {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: EMPTY_LABELS,
        unresolved_intents: EMPTY_LABELS,
        requested_actions: EMPTY_LABELS,
    })
    .expect("empty handoff resource");
    let (empty_request, empty_receipt) = authorized_role_request(empty_resource.as_str(), AGENT_ROLE_HANDOFF_ABILITY);
    let leak = validate_agent_role_handoff(AgentRoleHandoffDraft {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: EMPTY_LABELS,
        unresolved_intents: EMPTY_LABELS,
        requested_actions: EMPTY_LABELS,
        handoff_request: &empty_request,
        handoff_receipt: &empty_receipt,
        sensitive: AgentRoleHandoffSensitiveDraft {
            inherited_grant: Some("ambient-admin"),
            ..AgentRoleHandoffSensitiveDraft::default()
        },
    })
    .expect_err("handoff cannot inherit grants");
    let (denied_request, denied_receipt) = denied_role_request(handoff_resource.as_str(), AGENT_ROLE_HANDOFF_ABILITY);
    let denied = validate_agent_role_handoff(AgentRoleHandoffDraft {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: &evidence_refs,
        unresolved_intents: &unresolved_intents,
        requested_actions: &requested_actions,
        handoff_request: &denied_request,
        handoff_receipt: &denied_receipt,
        sensitive: AgentRoleHandoffSensitiveDraft::default(),
    })
    .expect_err("handoff requires its own grant");

    assert_eq!(handoff.subject_hash(), SUBJECT_HASH);
    assert_eq!(handoff.evidence_refs().len(), 2);
    assert_eq!(handoff.redaction_marker(), "raw-role-context-omitted");
    assert_eq!(leak.issue(), AgentRoleIssue::ReceiptLeak);
    assert_eq!(denied.issue(), AgentRoleIssue::AuthorityDenied);
}

#[test]
fn review_output_is_advisory_and_separately_authorized() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let finding_resource_input = output_resource_input(&fixture, reviewer, AgentRoleOutputEffect::Finding);
    let finding_resource = agent_role_output_resource(finding_resource_input).expect("finding resource");
    let (output_request, output_receipt) =
        authorized_role_request(finding_resource.as_str(), AGENT_ROLE_EMIT_REVIEW_ABILITY);
    let output = admit_agent_role_output(output_draft(finding_resource_input, &output_request, &output_receipt))
        .expect("advisory finding");
    let select_resource_input = output_resource_input(&fixture, reviewer, AgentRoleOutputEffect::Select);
    let select_resource = agent_role_output_resource(select_resource_input).expect("invalid class resource");
    let (select_request, select_receipt) =
        authorized_role_request(select_resource.as_str(), AGENT_ROLE_EMIT_REVIEW_ABILITY);
    let class_error = admit_agent_role_output(output_draft(select_resource_input, &select_request, &select_receipt))
        .expect_err("reviewer cannot select");

    assert_eq!(output.output_kind(), AgentRoleOutputKind::ReviewReport);
    assert_eq!(output.effect(), AgentRoleOutputEffect::Finding);
    assert_eq!(output.redaction_marker(), "raw-role-output-omitted");
    assert_eq!(class_error.issue(), AgentRoleIssue::OutputDenied);
}

#[test]
fn judgment_cannot_override_deterministic_denial() {
    let fixture = admitted_fixture("judge");
    let judge = fixture.catalog.profile("judge").expect("judge");
    let resource_input = AgentRoleOutputResourceInput {
        rubric_hash: Some(RUBRIC_HASH),
        admission: AgentRoleOutputAdmission {
            deterministic_policy: AgentRoleDeterministicPolicyState::Denied,
            rubric: AgentRoleAdmissionState::Admitted,
            ..AgentRoleOutputAdmission::default()
        },
        ..output_resource_input(&fixture, judge, AgentRoleOutputEffect::Select)
    };
    let resource = agent_role_output_resource(resource_input).expect("judgment resource");
    let (output_request, output_receipt) = authorized_role_request(resource.as_str(), AGENT_ROLE_SELECT_ABILITY);
    let error = admit_agent_role_output(output_draft(resource_input, &output_request, &output_receipt))
        .expect_err("judge must not suppress deterministic denial");

    assert_eq!(error.issue(), AgentRoleIssue::OutputDenied);
    assert!(error.detail().contains("deterministic denial"));
}

#[test]
fn assignment_requires_distinct_assign_and_activate_authority() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let model_plan = role_composed_plan(reviewer).expect("role-bound model plan");
    let separation = separation_for(&catalog, reviewer);
    let resource = agent_role_activation_resource(activation_resource_input(reviewer, &model_plan, &separation))
        .expect("activation resource");
    let (wrong_assignment_request, wrong_assignment_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let (activation_request, activation_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let error = admit_agent_role_assignment(assignment_request(
        reviewer,
        &model_plan,
        &separation,
        assignment_authorizations(
            &wrong_assignment_request,
            &wrong_assignment_receipt,
            &activation_request,
            &activation_receipt,
        ),
    ))
    .expect_err("activation authority cannot substitute for assignment authority");

    assert_eq!(error.issue(), AgentRoleIssue::AuthorityDenied);
    assert!(error.detail().contains("role-assignment"));
}

#[test]
fn equal_router_and_semantic_judge_labels_do_not_grant_semantic_authority() {
    let router_role_label = "judge";
    let catalog = catalog();
    let judge = catalog.profile("judge").expect("judge");
    let model_plan = role_composed_plan(judge).expect("role-bound model plan");
    let separation = separation_for(&catalog, judge);
    let resource = agent_role_activation_resource(activation_resource_input(judge, &model_plan, &separation))
        .expect("activation resource");
    let (role_assignment_request, assignment_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ASSIGN_ABILITY);
    let (wrong_activation_request, wrong_activation_receipt) = authorized_request_for_action(
        animus_adapters::GuardedAction::Model,
        resource.as_str(),
        AGENT_ROLE_ACTIVATE_ABILITY,
    );
    let error = admit_agent_role_assignment(assignment_request(
        judge,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &assignment_receipt,
            &wrong_activation_request,
            &wrong_activation_receipt,
        ),
    ))
    .expect_err("an equal router label cannot substitute activation authority");

    assert_eq!(router_role_label, judge.role_id());
    assert_eq!(error.issue(), AgentRoleIssue::AuthorityDenied);
}

#[test]
fn assignment_rejects_missing_grant_lineage_drift_and_malformed_contract() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let model_plan = role_composed_plan(reviewer).expect("role-bound model plan");
    let separation = separation_for(&catalog, reviewer);
    let resource = agent_role_activation_resource(activation_resource_input(reviewer, &model_plan, &separation))
        .expect("activation resource");
    let (role_assignment_request, role_assignment_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ASSIGN_ABILITY);
    let (activation_request, activation_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let (denied_activation_request, denied_activation_receipt) =
        denied_role_request(resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let missing_grant = admit_agent_role_assignment(assignment_request(
        reviewer,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &role_assignment_receipt,
            &denied_activation_request,
            &denied_activation_receipt,
        ),
    ))
    .expect_err("missing activation grant must fail");
    let mut stale_lineage = assignment_request(
        reviewer,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &role_assignment_receipt,
            &activation_request,
            &activation_receipt,
        ),
    );
    stale_lineage.parent_lineage_hash = ALTERNATE_HASH;
    let stale_error =
        admit_agent_role_assignment(stale_lineage).expect_err("lineage drift must invalidate authorization");
    let mut malformed_contract = assignment_request(
        reviewer,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &role_assignment_receipt,
            &activation_request,
            &activation_receipt,
        ),
    );
    malformed_contract.output_contract_hash = "not-a-hash";
    let malformed_error =
        admit_agent_role_assignment(malformed_contract).expect_err("malformed output contract must fail");

    assert_eq!(missing_grant.issue(), AgentRoleIssue::AuthorityDenied);
    assert_eq!(stale_error.issue(), AgentRoleIssue::AuthorityDenied);
    assert_eq!(malformed_error.issue(), AgentRoleIssue::InvalidHash);
}

#[test]
fn required_capabilities_and_wildcards_fail_closed() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let role_ceiling = typed_role_ceiling(reviewer);
    let mutation_only = vec![mutation_capability()];
    let missing_required = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment: &fixture.assignment,
        profile: reviewer,
        requested: role_ceiling.as_slice(),
        role_ceiling: role_ceiling.as_slice(),
        ceilings: AgentRoleCapabilityCeilings {
            verified_grants: role_ceiling.as_slice(),
            host_policy: mutation_only.as_slice(),
            parent_delegation: role_ceiling.as_slice(),
            task_scope: role_ceiling.as_slice(),
            agent_definition: role_ceiling.as_slice(),
            model_profile: role_ceiling.as_slice(),
            workspace: role_ceiling.as_slice(),
            tool_executor: role_ceiling.as_slice(),
            approval: role_ceiling.as_slice(),
            sandbox: role_ceiling.as_slice(),
            cancellation: role_ceiling.as_slice(),
            timeout: role_ceiling.as_slice(),
            budget: role_ceiling.as_slice(),
        },
    })
    .expect_err("required context capability cannot be minted by role");
    let wildcard = vec![crate::TypedSubagentCapability::ContextRead {
        stable_label: "all".to_string(),
        context_label: "bounded-context".to_string(),
    }];
    let wildcard_error = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment: &fixture.assignment,
        profile: reviewer,
        requested: wildcard.as_slice(),
        role_ceiling: role_ceiling.as_slice(),
        ceilings: uniform_ceilings(role_ceiling.as_slice()),
    })
    .expect_err("wildcard capability must fail closed");

    assert_eq!(missing_required.issue(), AgentRoleIssue::MissingRequiredCapability);
    assert_eq!(wildcard_error.issue(), AgentRoleIssue::DuplicateIdentity);
}

#[test]
fn hidden_capability_aliases_and_duplicate_stable_labels_fail_closed() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let role_ceiling = typed_role_ceiling(reviewer);
    let hidden_alias = crate::TypedSubagentCapability::ContextRead {
        stable_label: "context-read".to_string(),
        context_label: "bounded-context".to_string(),
    };
    let alias_request = vec![context_capability(), hidden_alias.clone()];
    let alias_resolution = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment: &fixture.assignment,
        profile: reviewer,
        requested: alias_request.as_slice(),
        role_ceiling: role_ceiling.as_slice(),
        ceilings: uniform_ceilings(role_ceiling.as_slice()),
    })
    .expect("exact context capability remains admitted");
    let duplicate_labels = vec![context_capability(), crate::TypedSubagentCapability::ToolUse {
        stable_label: "context_read".to_string(),
        tool_label: "inspect_context".to_string(),
    }];
    let duplicate_error = resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment: &fixture.assignment,
        profile: reviewer,
        requested: duplicate_labels.as_slice(),
        role_ceiling: role_ceiling.as_slice(),
        ceilings: uniform_ceilings(role_ceiling.as_slice()),
    })
    .expect_err("duplicate normalized labels must fail");

    assert!(alias_resolution.denied().iter().any(|denial| {
        denial.capability() == &hidden_alias && denial.source() == AgentRoleCapabilitySource::SemanticRole
    }));
    assert_eq!(duplicate_error.issue(), AgentRoleIssue::DuplicateIdentity);
}

#[test]
fn role_model_composition_rejects_unadmitted_output_budget_and_workspace() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let wrong_output =
        role_composed_plan_with_admission(reviewer, &["other-output"], &[reviewer.budget_policy_ref()], &[
            "workspace:project",
        ])
        .expect_err("output contract mismatch must block composition");
    let wrong_budget =
        role_composed_plan_with_admission(reviewer, &[reviewer.output_contract()], &["other-budget"], &[
            "workspace:project",
        ])
        .expect_err("budget mismatch must block composition");
    let wrong_workspace =
        role_composed_plan_with_admission(reviewer, &[reviewer.output_contract()], &[reviewer.budget_policy_ref()], &[
            "workspace:other",
        ])
        .expect_err("workspace mismatch must block composition");

    assert_eq!(wrong_output.issue(), AgentRoleIssue::ResourceDenied);
    assert_eq!(wrong_budget.issue(), AgentRoleIssue::ResourceDenied);
    assert_eq!(wrong_workspace.issue(), AgentRoleIssue::ResourceDenied);
}

#[test]
fn implementer_and_reviewer_plans_remain_distinct() {
    let implementer = admitted_fixture("implementer");
    let reviewer = admitted_fixture("reviewer");

    assert_ne!(implementer.model_plan.binding_hash(), reviewer.model_plan.binding_hash());
    assert_ne!(implementer.role_plan.plan_hash(), reviewer.role_plan.plan_hash());
    assert_ne!(implementer.capabilities.effective(), reviewer.capabilities.effective());
    assert!(reviewer.capabilities.effective().iter().all(|capability| capability != &mutation_capability()));
}

#[test]
fn separation_detects_lineage_context_verifier_and_rubric_drift() {
    let catalog = catalog();
    let reviewer = catalog.profile("reviewer").expect("reviewer");
    let review_policy = catalog.separation_policy(reviewer.separation_policy_ref()).expect("review policy");
    let lineage_failure = evaluate_agent_role_separation(review_policy, &AgentRoleSeparationObservation {
        candidate_mutation_lineage_hash: PARENT_HASH,
        subject_mutation_lineage_hash: PARENT_HASH,
        ..separation_observation(reviewer, false)
    })
    .expect("well-formed lineage failure");
    let context_failure = evaluate_agent_role_separation(review_policy, &AgentRoleSeparationObservation {
        satisfied_state_dimensions: STATE_DIMENSIONS_WITHOUT_CLEAN_CONTEXT,
        ..separation_observation(reviewer, false)
    })
    .expect("well-formed context failure");
    let verifier = catalog.profile("verifier").expect("verifier");
    let verifier_policy = catalog.separation_policy(verifier.separation_policy_ref()).expect("verifier policy");
    let verifier_error = evaluate_agent_role_separation(verifier_policy, &AgentRoleSeparationObservation {
        observed_verifier_hash: Some(ALTERNATE_HASH),
        ..separation_observation(verifier, false)
    })
    .expect_err("verifier identity drift must fail");
    let judge = catalog.profile("judge").expect("judge");
    let judge_policy = catalog.separation_policy(judge.separation_policy_ref()).expect("judge policy");
    let rubric_error = evaluate_agent_role_separation(judge_policy, &AgentRoleSeparationObservation {
        observed_rubric_hash: Some(ALTERNATE_HASH),
        ..separation_observation(judge, false)
    })
    .expect_err("rubric identity drift must fail");

    assert!(lineage_failure.failed_dimensions().contains(&AgentRoleSeparationDimension::MutationLineage));
    assert!(context_failure.failed_dimensions().contains(&AgentRoleSeparationDimension::CleanContext));
    assert_eq!(verifier_error.issue(), AgentRoleIssue::BindingMismatch);
    assert_eq!(rubric_error.issue(), AgentRoleIssue::BindingMismatch);
}

#[test]
fn implementer_and_reviewer_outputs_reject_overreach_and_raw_diffs() {
    let implementer = admitted_fixture("implementer");
    let implementer_profile = implementer.catalog.profile("implementer").expect("implementer");
    let candidate_input = output_resource_input(&implementer, implementer_profile, AgentRoleOutputEffect::Candidate);
    let candidate_resource = agent_role_output_resource(candidate_input).expect("candidate resource");
    let (candidate_request, candidate_receipt) =
        authorized_role_request(candidate_resource.as_str(), AGENT_ROLE_EMIT_CANDIDATE_ABILITY);
    let completion_claims = [AgentRoleOutputClaim::AuthoritativeCompletion];
    let mut completion_claim = output_draft(candidate_input, &candidate_request, &candidate_receipt);
    completion_claim.claims = &completion_claims;
    let completion_error = admit_agent_role_output(completion_claim).expect_err("implementer cannot attest completion");
    let reviewer = admitted_fixture("reviewer");
    let reviewer_profile = reviewer.catalog.profile("reviewer").expect("reviewer");
    let finding_input = output_resource_input(&reviewer, reviewer_profile, AgentRoleOutputEffect::Finding);
    let finding_resource = agent_role_output_resource(finding_input).expect("finding resource");
    let (finding_request, finding_receipt) =
        authorized_role_request(finding_resource.as_str(), AGENT_ROLE_EMIT_REVIEW_ABILITY);
    let mutation_claims = [AgentRoleOutputClaim::SubjectMutation];
    let mut mutation = output_draft(finding_input, &finding_request, &finding_receipt);
    mutation.claims = &mutation_claims;
    let mutation_error = admit_agent_role_output(mutation).expect_err("reviewer cannot mutate subject");
    let authority_claims = [AgentRoleOutputClaim::AuthorityGrant];
    let mut authority = output_draft(finding_input, &finding_request, &finding_receipt);
    authority.claims = &authority_claims;
    let authority_error = admit_agent_role_output(authority).expect_err("reviewer cannot grant authority");
    let raw_diff_leak = [AgentRoleOutputLeak::RawDiffOrFile("raw patch body")];
    let mut raw_diff = output_draft(finding_input, &finding_request, &finding_receipt);
    raw_diff.sensitive = &raw_diff_leak;
    let raw_diff_error = admit_agent_role_output(raw_diff).expect_err("raw diff cannot enter review output");

    assert_eq!(completion_error.issue(), AgentRoleIssue::OutputDenied);
    assert_eq!(mutation_error.issue(), AgentRoleIssue::OutputDenied);
    assert_eq!(authority_error.issue(), AgentRoleIssue::OutputDenied);
    assert_eq!(raw_diff_error.issue(), AgentRoleIssue::ReceiptLeak);
}

#[test]
fn verification_requires_admitted_named_host_evidence_and_no_soundness_overclaim() {
    let fixture = admitted_fixture("verifier");
    let verifier = fixture.catalog.profile("verifier").expect("verifier");
    let missing_evidence_input = AgentRoleOutputResourceInput {
        verifier_hash: Some(VERIFIER_HASH),
        admission: AgentRoleOutputAdmission {
            verifier: AgentRoleAdmissionState::Admitted,
            ..AgentRoleOutputAdmission::default()
        },
        ..output_resource_input(&fixture, verifier, AgentRoleOutputEffect::CheckEvidence)
    };
    let missing_resource = agent_role_output_resource(missing_evidence_input).expect("verification resource");
    let (missing_request, missing_receipt) =
        authorized_role_request(missing_resource.as_str(), AGENT_ROLE_EMIT_VERIFICATION_ABILITY);
    let missing_error =
        admit_agent_role_output(output_draft(missing_evidence_input, &missing_request, &missing_receipt))
            .expect_err("fabricated verification evidence must fail");
    let admitted_input = AgentRoleOutputResourceInput {
        admission: AgentRoleOutputAdmission {
            host_evidence: AgentRoleAdmissionState::Admitted,
            ..missing_evidence_input.admission
        },
        ..missing_evidence_input
    };
    let admitted_resource = agent_role_output_resource(admitted_input).expect("admitted verification resource");
    let (admitted_request, admitted_receipt) =
        authorized_role_request(admitted_resource.as_str(), AGENT_ROLE_EMIT_VERIFICATION_ABILITY);
    let output = admit_agent_role_output(output_draft(admitted_input, &admitted_request, &admitted_receipt))
        .expect("named host evidence may be reported");
    let soundness_claims = [AgentRoleOutputClaim::VerifierSoundness];
    let mut soundness_claim = output_draft(admitted_input, &admitted_request, &admitted_receipt);
    soundness_claim.claims = &soundness_claims;
    let soundness_error = admit_agent_role_output(soundness_claim).expect_err("verifier soundness overclaim must fail");

    assert_eq!(missing_error.issue(), AgentRoleIssue::MissingReference);
    assert_eq!(output.effect(), AgentRoleOutputEffect::CheckEvidence);
    assert_eq!(soundness_error.issue(), AgentRoleIssue::OutputDenied);
}

#[test]
fn judgment_requires_admitted_rubric_and_rejects_hidden_rationale() {
    let fixture = admitted_fixture("judge");
    let judge = fixture.catalog.profile("judge").expect("judge");
    let missing_rubric_input = output_resource_input(&fixture, judge, AgentRoleOutputEffect::Rank);
    let missing_resource = agent_role_output_resource(missing_rubric_input).expect("missing-rubric resource");
    let (missing_request, missing_receipt) =
        authorized_role_request(missing_resource.as_str(), AGENT_ROLE_EMIT_JUDGMENT_ABILITY);
    let missing_error = admit_agent_role_output(output_draft(missing_rubric_input, &missing_request, &missing_receipt))
        .expect_err("judgment without rubric must fail");
    let admitted_input = AgentRoleOutputResourceInput {
        rubric_hash: Some(RUBRIC_HASH),
        admission: AgentRoleOutputAdmission {
            rubric: AgentRoleAdmissionState::Admitted,
            ..missing_rubric_input.admission
        },
        ..missing_rubric_input
    };
    let admitted_resource = agent_role_output_resource(admitted_input).expect("judgment resource");
    let (admitted_request, admitted_receipt) =
        authorized_role_request(admitted_resource.as_str(), AGENT_ROLE_EMIT_JUDGMENT_ABILITY);
    let rationale_leak = [AgentRoleOutputLeak::JudgeRationale("private weighted rationale")];
    let mut hidden_rationale = output_draft(admitted_input, &admitted_request, &admitted_receipt);
    hidden_rationale.sensitive = &rationale_leak;
    let rationale_error = admit_agent_role_output(hidden_rationale).expect_err("hidden rationale cannot be persisted");

    assert_eq!(missing_error.issue(), AgentRoleIssue::MissingReference);
    assert_eq!(rationale_error.issue(), AgentRoleIssue::ReceiptLeak);
}

#[test]
fn judgment_abstention_is_explicit_and_never_forced() {
    let fixture = admitted_fixture("judge");
    let judge = fixture.catalog.profile("judge").expect("judge");
    let abstained_input = AgentRoleOutputResourceInput {
        status: AgentRoleOutputStatus::Abstained,
        ..output_resource_input(&fixture, judge, AgentRoleOutputEffect::Abstain)
    };
    let abstained_resource = agent_role_output_resource(abstained_input).expect("abstention resource");
    let (abstained_request, abstained_receipt) =
        authorized_role_request(abstained_resource.as_str(), AGENT_ROLE_EMIT_JUDGMENT_ABILITY);
    let output = admit_agent_role_output(output_draft(abstained_input, &abstained_request, &abstained_receipt))
        .expect("explicit abstention");
    let inconsistent_input = output_resource_input(&fixture, judge, AgentRoleOutputEffect::Abstain);
    let inconsistent_resource = agent_role_output_resource(inconsistent_input).expect("inconsistent resource");
    let (inconsistent_request, inconsistent_receipt) =
        authorized_role_request(inconsistent_resource.as_str(), AGENT_ROLE_EMIT_JUDGMENT_ABILITY);
    let error = admit_agent_role_output(output_draft(inconsistent_input, &inconsistent_request, &inconsistent_receipt))
        .expect_err("abstention status and effect must agree");

    assert_eq!(output.status(), AgentRoleOutputStatus::Abstained);
    assert_eq!(output.effect(), AgentRoleOutputEffect::Abstain);
    assert_eq!(error.issue(), AgentRoleIssue::OutputDenied);
}

#[test]
fn coordination_and_observation_outputs_remain_bounded() {
    for (role_id, effect, ability) in [
        ("coordinator", AgentRoleOutputEffect::Coordinate, AGENT_ROLE_EMIT_COORDINATION_ABILITY),
        ("observer", AgentRoleOutputEffect::Observe, AGENT_ROLE_EMIT_OBSERVATION_ABILITY),
    ] {
        let fixture = admitted_fixture(role_id);
        let profile = fixture.catalog.profile(role_id).expect("extension role");
        let resource_input = output_resource_input(&fixture, profile, effect);
        let resource = agent_role_output_resource(resource_input).expect("output resource");
        let (request, receipt) = authorized_role_request(resource.as_str(), ability);
        let output = admit_agent_role_output(output_draft(resource_input, &request, &receipt))
            .expect("bounded extension output");
        assert_eq!(output.effect(), effect);
        assert_eq!(output.subject_hash(), SUBJECT_HASH);
    }
}

#[test]
fn output_influence_requires_its_own_grant() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let resource_input = output_resource_input(&fixture, reviewer, AgentRoleOutputEffect::Finding);
    let resource = agent_role_output_resource(resource_input).expect("finding resource");
    let (request, receipt) = denied_role_request(resource.as_str(), AGENT_ROLE_EMIT_REVIEW_ABILITY);
    let error = admit_agent_role_output(output_draft(resource_input, &request, &receipt))
        .expect_err("activation cannot mint review-output influence");

    assert_eq!(error.issue(), AgentRoleIssue::AuthorityDenied);
}

#[test]
fn role_outputs_preserve_typed_non_success_statuses() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    for status in [
        AgentRoleOutputStatus::Denied,
        AgentRoleOutputStatus::Unavailable,
        AgentRoleOutputStatus::TimedOut,
        AgentRoleOutputStatus::Cancelled,
        AgentRoleOutputStatus::Malformed,
        AgentRoleOutputStatus::Conflict,
    ] {
        let resource_input = AgentRoleOutputResourceInput {
            status,
            ..output_resource_input(&fixture, reviewer, AgentRoleOutputEffect::Finding)
        };
        let resource = agent_role_output_resource(resource_input).expect("status resource");
        let (request, receipt) = authorized_role_request(resource.as_str(), AGENT_ROLE_EMIT_REVIEW_ABILITY);
        let output = admit_agent_role_output(output_draft(resource_input, &request, &receipt))
            .expect("typed non-success output");
        assert_eq!(output.status(), status);
        assert_eq!(output.effect(), AgentRoleOutputEffect::Finding);
    }
}

#[test]
fn handoff_rejects_overbound_metadata() {
    const OVERBOUND_HANDOFF_REFERENCES: usize = 65;
    let fixture = admitted_fixture("implementer");
    let reviewer_hash = fixture.catalog.profile("reviewer").expect("reviewer").profile_hash();
    let evidence_refs = vec!["bounded-label"; OVERBOUND_HANDOFF_REFERENCES];
    let error = agent_role_handoff_resource(AgentRoleHandoffResourceInput {
        current_plan: &fixture.role_plan,
        target_role_profile_hash: reviewer_hash,
        evidence_refs: evidence_refs.as_slice(),
        unresolved_intents: EMPTY_LABELS,
        requested_actions: EMPTY_LABELS,
    })
    .expect_err("over-bound handoff metadata must fail");

    assert_eq!(error.issue(), AgentRoleIssue::InvalidCollection);
}

#[test]
fn native_tool_actions_bind_model_and_role_plans() {
    let fixture = admitted_fixture("reviewer");
    let model_call =
        crate::validate_model_tool_call_binding(fixture.model_plan.plan(), tool_call_draft(fixture.model_plan.plan()))
            .expect("validated native model tool call");
    let capability = tool_capability();
    let binding = bind_agent_role_tool_action(AgentRoleToolActionInput {
        assignment: &fixture.assignment,
        role_plan: &fixture.role_plan,
        model_plan: &fixture.model_plan,
        capabilities: &fixture.capabilities,
        capability: &capability,
        model_call: &model_call,
    })
    .expect("role-bound native tool call");
    let other = admitted_fixture("implementer");
    let stale_error = bind_agent_role_tool_action(AgentRoleToolActionInput {
        assignment: &fixture.assignment,
        role_plan: &fixture.role_plan,
        model_plan: &other.model_plan,
        capabilities: &fixture.capabilities,
        capability: &capability,
        model_call: &model_call,
    })
    .expect_err("tool call cannot cross role/model plan");

    assert_eq!(binding.tool_name(), "inspect_context");
    assert_eq!(binding.role_plan_hash(), fixture.role_plan.plan_hash());
    assert_eq!(stale_error.issue(), AgentRoleIssue::BindingMismatch);
}

#[test]
fn role_receipt_is_metadata_only_and_rejects_raw_material() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let input = AgentRoleReceiptInput {
        kind: AgentRoleReceiptKind::RolePlan,
        assignment: &fixture.assignment,
        profile: reviewer,
        capability_resolution: Some(&fixture.capabilities),
        separation: Some(&fixture.separation),
        role_plan: Some(&fixture.role_plan),
        handoff: None,
        output: None,
    };
    let receipt =
        project_agent_role_receipt(input, AgentRoleReceiptSensitiveDraft::default()).expect("metadata receipt");
    let error = project_agent_role_receipt(input, AgentRoleReceiptSensitiveDraft {
        hidden_reasoning: Some("private chain"),
        ..AgentRoleReceiptSensitiveDraft::default()
    })
    .expect_err("reasoning cannot enter durable receipt");

    assert_eq!(receipt.kind(), AgentRoleReceiptKind::RolePlan);
    assert_eq!(receipt.assignment_hash(), fixture.assignment.assignment_hash());
    assert_eq!(
        receipt.effective_capability_count(),
        u32::try_from(REVIEW_CAPABILITIES.len()).expect("bounded review capability count")
    );
    assert_eq!(receipt.redaction_marker(), "raw-role-context-and-output-omitted");
    assert_eq!(error.issue(), AgentRoleIssue::ReceiptLeak);
    assert!(!format!("{receipt:?}").contains("private chain"));
}

#[test]
fn role_receipt_rejects_every_sensitive_material_class() {
    let fixture = admitted_fixture("reviewer");
    let reviewer = fixture.catalog.profile("reviewer").expect("reviewer");
    let input = AgentRoleReceiptInput {
        kind: AgentRoleReceiptKind::RolePlan,
        assignment: &fixture.assignment,
        profile: reviewer,
        capability_resolution: Some(&fixture.capabilities),
        separation: Some(&fixture.separation),
        role_plan: Some(&fixture.role_plan),
        handoff: None,
        output: None,
    };
    let sensitive_cases = [
        AgentRoleReceiptSensitiveDraft {
            raw_prompt_or_instruction: Some("raw prompt"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            hidden_reasoning: Some("reasoning"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            provider_payload: Some("provider body"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            credential_or_token: Some("token"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            private_path: Some("private path"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            verifier_body: Some("verifier body"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            hidden_test_or_oracle: Some("hidden oracle"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            raw_diff_or_file: Some("raw diff"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            tool_body: Some("tool body"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            judge_rationale: Some("judge rationale"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
        AgentRoleReceiptSensitiveDraft {
            transcript: Some("transcript"),
            ..AgentRoleReceiptSensitiveDraft::default()
        },
    ];

    for sensitive in sensitive_cases {
        let error =
            project_agent_role_receipt(input, sensitive).expect_err("sensitive role receipt material must fail");
        assert_eq!(error.issue(), AgentRoleIssue::ReceiptLeak);
    }
}

fn catalog() -> AgentRoleCatalog {
    catalog_with_hash_override(None).expect("valid role catalog")
}

fn catalog_with_hash_override(first_profile_hash: Option<&str>) -> Result<AgentRoleCatalog, AgentRoleError> {
    let provisional = ROLE_SPECS
        .iter()
        .map(|(role_id, class)| role_profile_draft(role_id, *class, ZERO_HASH))
        .collect::<Vec<_>>();
    let identities = provisional
        .iter()
        .map(|profile| blake3_agent_role_profile_identity(profile).expect("profile identity"))
        .collect::<Vec<_>>();
    let profiles = ROLE_SPECS
        .iter()
        .enumerate()
        .map(|(index, (role_id, class))| {
            let profile_hash = if index == usize::default() {
                first_profile_hash.unwrap_or(identities[index].as_str())
            } else {
                identities[index].as_str()
            };
            role_profile_draft(role_id, *class, profile_hash)
        })
        .collect::<Vec<_>>();
    let budgets = [AgentRoleBudgetPolicyDraft {
        policy_id: "bounded-role",
        max_actions: ACTION_LIMIT,
        max_outputs: OUTPUT_LIMIT,
        max_context_units: CONTEXT_LIMIT,
    }];
    let toolsets = [AgentRoleToolsetDraft {
        toolset_id: "bounded-tools",
        tool_names: TOOL_NAMES,
    }];
    let separation_policies = separation_policy_drafts();
    AgentRoleCatalog::validate(&AgentRoleCatalogDraft {
        schema: AGENT_ROLE_CATALOG_SCHEMA,
        known_capabilities: KNOWN_CAPABILITIES,
        input_contracts: INPUT_CONTRACTS,
        output_contracts: OUTPUT_CONTRACTS,
        evidence_contracts: EVIDENCE_CONTRACTS,
        budget_policies: &budgets,
        toolsets: &toolsets,
        separation_policies: &separation_policies,
        profiles: profiles.as_slice(),
    })
}

fn role_profile_draft<'a>(role_id: &'a str, class: AgentRoleClass, profile_hash: &'a str) -> AgentRoleProfileDraft<'a> {
    let (purpose, mutation_posture, context_posture, evidence_posture, output_kind) = match class {
        AgentRoleClass::Implementation => (
            "Produce bounded implementation candidates without completion authority.",
            AgentRoleMutationPosture::ScopedIntent,
            AgentRoleContextPosture::CleanRecompose,
            AgentRoleEvidencePosture::Candidate,
            AgentRoleOutputKind::ImplementationCandidate,
        ),
        AgentRoleClass::Review => (
            "Review an immutable subject and emit advisory findings.",
            AgentRoleMutationPosture::SubjectReadOnly,
            AgentRoleContextPosture::CleanRecompose,
            AgentRoleEvidencePosture::Findings,
            AgentRoleOutputKind::ReviewReport,
        ),
        AgentRoleClass::Verification => (
            "Run named checks over an immutable subject and report supplied evidence.",
            AgentRoleMutationPosture::VerificationServiceOnly,
            AgentRoleContextPosture::CleanRecompose,
            AgentRoleEvidencePosture::CheckEvidence,
            AgentRoleOutputKind::VerificationReport,
        ),
        AgentRoleClass::Judgment => (
            "Apply a pinned rubric without granting authority or overriding denials.",
            AgentRoleMutationPosture::None,
            AgentRoleContextPosture::CleanRecompose,
            AgentRoleEvidencePosture::Judgment,
            AgentRoleOutputKind::JudgmentReport,
        ),
        AgentRoleClass::Coordination => (
            "Coordinate bounded metadata-only work and handoffs.",
            AgentRoleMutationPosture::None,
            AgentRoleContextPosture::BoundedShared,
            AgentRoleEvidencePosture::Coordination,
            AgentRoleOutputKind::CoordinationReport,
        ),
        AgentRoleClass::Observation => (
            "Observe bounded admitted records without mutation authority.",
            AgentRoleMutationPosture::None,
            AgentRoleContextPosture::ReadOnlyObservation,
            AgentRoleEvidencePosture::Observation,
            AgentRoleOutputKind::ObservationReport,
        ),
    };
    let capability_ceiling = match class {
        AgentRoleClass::Implementation => IMPLEMENTER_CAPABILITIES,
        AgentRoleClass::Review => REVIEW_CAPABILITIES,
        AgentRoleClass::Verification
        | AgentRoleClass::Judgment
        | AgentRoleClass::Coordination
        | AgentRoleClass::Observation => CONTEXT_CAPABILITY,
    };
    AgentRoleProfileDraft {
        role_id,
        class,
        purpose,
        source_scope: crate::ModelProfileSourceScope::Host,
        trust_status: crate::ModelProfileTrustStatus::Trusted,
        admission_status: crate::ModelProfileAdmissionStatus::Admitted,
        project_profile_admitted: false,
        required_capabilities: CONTEXT_CAPABILITY,
        capability_ceiling,
        eligible_model_profiles: MODEL_PROFILES,
        model_targets: MODEL_TARGETS,
        prompt_modes: PROMPT_MODES,
        prompt_sources: PROMPT_SOURCES,
        skills: SKILLS,
        memory_policy_hashes: MEMORY_POLICIES,
        tools: NO_TOOLS,
        toolsets: TOOLSETS,
        tool_capabilities: TOOL_CAPABILITIES,
        workspace_scopes: WORKSPACE_SCOPES,
        mutation_posture,
        context_posture,
        evidence_posture,
        output_kind,
        input_contract: INPUT_CONTRACTS[0],
        output_contract: OUTPUT_CONTRACTS[0],
        evidence_contract: EVIDENCE_CONTRACTS[0],
        separation_policy_ref: separation_policy_id(class),
        budget_policy_ref: "bounded-role",
        policy_hash: POLICY_HASH,
        profile_hash,
    }
}

const fn separation_policy_id(class: AgentRoleClass) -> &'static str {
    match class {
        AgentRoleClass::Implementation | AgentRoleClass::Coordination | AgentRoleClass::Observation => "subject-bound",
        AgentRoleClass::Review => "independent-review",
        AgentRoleClass::Verification => "independent-verifier",
        AgentRoleClass::Judgment => "independent-judge",
    }
}

fn separation_policy_drafts() -> [AgentRoleSeparationPolicyDraft<'static>; SEPARATION_POLICY_COUNT] {
    [
        separation_policy("subject-bound", SUBJECT_BOUND_DIMENSIONS),
        separation_policy("independent-review", INDEPENDENT_DIMENSIONS),
        separation_policy("independent-verifier", INDEPENDENT_VERIFIER_DIMENSIONS),
        separation_policy("independent-judge", INDEPENDENT_JUDGE_DIMENSIONS),
    ]
}

const fn separation_policy(
    policy_id: &'static str,
    required_dimensions: &'static [AgentRoleSeparationDimension],
) -> AgentRoleSeparationPolicyDraft<'static> {
    AgentRoleSeparationPolicyDraft {
        policy_id,
        required_dimensions,
    }
}

fn separation_observation(profile: &AgentRoleProfile, use_same_actor: bool) -> AgentRoleSeparationObservation<'static> {
    let expected_verifier_hash = (profile.class() == AgentRoleClass::Verification).then_some(VERIFIER_HASH);
    let expected_rubric_hash = (profile.class() == AgentRoleClass::Judgment).then_some(RUBRIC_HASH);
    AgentRoleSeparationObservation {
        candidate_assignment_hash: FRESHNESS_HASH,
        subject_assignment_hash: PARENT_HASH,
        candidate_actor_hash: if use_same_actor { PARENT_HASH } else { ACTOR_HASH },
        subject_actor_hash: PARENT_HASH,
        candidate_run_thread_hash: RUN_HASH,
        subject_run_thread_hash: PARENT_HASH,
        candidate_session_hash: SESSION_HASH,
        subject_session_hash: PARENT_HASH,
        candidate_model_profile_hash: AGENT_HASH,
        subject_model_profile_hash: PARENT_HASH,
        candidate_workspace_hash: WORKSPACE_HASH,
        subject_workspace_hash: PARENT_HASH,
        candidate_mutation_lineage_hash: LINEAGE_HASH,
        subject_mutation_lineage_hash: PARENT_HASH,
        subject_hash: SUBJECT_HASH,
        observed_subject_hash: SUBJECT_HASH,
        expected_verifier_hash,
        observed_verifier_hash: expected_verifier_hash,
        expected_rubric_hash,
        observed_rubric_hash: expected_rubric_hash,
        policy_hash: POLICY_HASH,
        observed_policy_hash: POLICY_HASH,
        satisfied_state_dimensions: ALL_STATE_DIMENSIONS,
    }
}

fn separation_for(catalog: &AgentRoleCatalog, profile: &AgentRoleProfile) -> AgentRoleSeparationDecision {
    let policy = catalog.separation_policy(profile.separation_policy_ref()).expect("separation policy");
    evaluate_agent_role_separation(policy, &separation_observation(profile, false)).expect("separation decision")
}

fn admitted_fixture(role_id: &str) -> AdmittedFixture {
    let catalog = catalog();
    let profile = catalog.profile(role_id).expect("role profile");
    let model_plan = role_composed_plan(profile).expect("role-bound model plan");
    let separation = separation_for(&catalog, profile);
    let resource = agent_role_activation_resource(activation_resource_input(profile, &model_plan, &separation))
        .expect("activation resource");
    let (role_assignment_request, role_assignment_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ASSIGN_ABILITY);
    let (activation_request, activation_receipt) =
        authorized_role_request(resource.as_str(), AGENT_ROLE_ACTIVATE_ABILITY);
    let assignment = admit_agent_role_assignment(assignment_request(
        profile,
        &model_plan,
        &separation,
        assignment_authorizations(
            &role_assignment_request,
            &role_assignment_receipt,
            &activation_request,
            &activation_receipt,
        ),
    ))
    .expect("admitted role assignment");
    let role_ceiling = typed_role_ceiling(profile);
    let capabilities =
        resolve_with_uniform_ceilings(&assignment, profile, role_ceiling.as_slice()).expect("role capabilities");
    let role_plan = build_agent_role_plan(AgentRolePlanInput {
        assignment: &assignment,
        profile,
        model_plan: &model_plan,
        capabilities: &capabilities,
        separation: &separation,
    })
    .expect("immutable role plan");
    AdmittedFixture {
        catalog,
        model_plan,
        separation,
        assignment,
        capabilities,
        role_plan,
    }
}

fn activation_resource_input<'a>(
    profile: &'a AgentRoleProfile,
    model_plan: &'a AgentRoleBoundModelPlan,
    separation: &'a AgentRoleSeparationDecision,
) -> AgentRoleActivationResourceInput<'a> {
    AgentRoleActivationResourceInput {
        profile,
        model_plan,
        separation,
        subject_hash: SUBJECT_HASH,
        task_hash: TASK_HASH,
        actor_hash: ACTOR_HASH,
        parent_lineage_hash: PARENT_HASH,
        run_thread_hash: RUN_HASH,
        session_hash: SESSION_HASH,
        workspace_hash: WORKSPACE_HASH,
        mutation_lineage_hash: LINEAGE_HASH,
        agent_definition_hash: AGENT_HASH,
        grant_set_hash: GRANT_HASH,
        freshness_hash: FRESHNESS_HASH,
        output_contract_hash: OUTPUT_CONTRACT_HASH,
    }
}

#[derive(Clone, Copy)]
struct AssignmentAuthorizations<'a> {
    role_assignment_request: &'a animus_adapters::CapabilityRequest,
    role_assignment_receipt: &'a animus_adapters::AuthorizationReceipt,
    activation_request: &'a animus_adapters::CapabilityRequest,
    activation_receipt: &'a animus_adapters::AuthorizationReceipt,
}

const fn assignment_authorizations<'a>(
    role_assignment_request: &'a animus_adapters::CapabilityRequest,
    role_assignment_receipt: &'a animus_adapters::AuthorizationReceipt,
    activation_request: &'a animus_adapters::CapabilityRequest,
    activation_receipt: &'a animus_adapters::AuthorizationReceipt,
) -> AssignmentAuthorizations<'a> {
    AssignmentAuthorizations {
        role_assignment_request,
        role_assignment_receipt,
        activation_request,
        activation_receipt,
    }
}

fn assignment_request<'a>(
    profile: &'a AgentRoleProfile,
    model_plan: &'a AgentRoleBoundModelPlan,
    separation: &'a AgentRoleSeparationDecision,
    authorizations: AssignmentAuthorizations<'a>,
) -> AgentRoleAssignmentRequest<'a> {
    AgentRoleAssignmentRequest {
        profile,
        model_plan,
        separation,
        subject_hash: SUBJECT_HASH,
        task_hash: TASK_HASH,
        actor_hash: ACTOR_HASH,
        parent_lineage_hash: PARENT_HASH,
        run_thread_hash: RUN_HASH,
        session_hash: SESSION_HASH,
        workspace_hash: WORKSPACE_HASH,
        mutation_lineage_hash: LINEAGE_HASH,
        agent_definition_hash: AGENT_HASH,
        grant_set_hash: GRANT_HASH,
        freshness_hash: FRESHNESS_HASH,
        output_contract_hash: OUTPUT_CONTRACT_HASH,
        assignment_request: authorizations.role_assignment_request,
        assignment_receipt: authorizations.role_assignment_receipt,
        activation_request: authorizations.activation_request,
        activation_receipt: authorizations.activation_receipt,
    }
}

fn authorized_role_request(
    resource: &str,
    ability: impl AsRef<str>,
) -> (animus_adapters::CapabilityRequest, animus_adapters::AuthorizationReceipt) {
    authorized_request_for_action(animus_adapters::GuardedAction::AgentRole, resource, ability)
}

fn authorized_request_for_action(
    action: animus_adapters::GuardedAction,
    resource: &str,
    ability: impl AsRef<str>,
) -> (animus_adapters::CapabilityRequest, animus_adapters::AuthorizationReceipt) {
    let prefix = "role:";
    let ability = ability.as_ref();
    let request = animus_adapters::CapabilityRequest::new(action, AGENT_ROLE_CAPABILITY_CONTRACT, resource, ability)
        .expect("bounded capability request");
    let contract =
        animus_adapters::CapabilityPolicyContract::new(action, AGENT_ROLE_CAPABILITY_CONTRACT, prefix, ability)
            .expect("bounded capability contract");
    let grant =
        animus_adapters::UcanGrant::new(AGENT_ROLE_CAPABILITY_CONTRACT, prefix, ability).expect("bounded UCAN grant");
    let receipt = animus_adapters::authorize_capability(&animus_adapters::AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[grant],
        caveats: &[],
        sandbox: &animus_adapters::SandboxConstraint::NotRequired,
    });
    (request, receipt)
}

fn denied_role_request(
    resource: &str,
    ability: impl AsRef<str>,
) -> (animus_adapters::CapabilityRequest, animus_adapters::AuthorizationReceipt) {
    let prefix = "role:";
    let ability = ability.as_ref();
    let request = animus_adapters::CapabilityRequest::new(
        animus_adapters::GuardedAction::AgentRole,
        AGENT_ROLE_CAPABILITY_CONTRACT,
        resource,
        ability,
    )
    .expect("role capability request");
    let contract = animus_adapters::CapabilityPolicyContract::new(
        animus_adapters::GuardedAction::AgentRole,
        AGENT_ROLE_CAPABILITY_CONTRACT,
        prefix,
        ability,
    )
    .expect("role capability contract");
    let receipt = animus_adapters::authorize_capability(&animus_adapters::AuthorizationContext {
        request: &request,
        contract: &contract,
        grants: &[],
        caveats: &[],
        sandbox: &animus_adapters::SandboxConstraint::NotRequired,
    });
    (request, receipt)
}

fn typed_role_ceiling(profile: &AgentRoleProfile) -> Vec<crate::TypedSubagentCapability> {
    profile
        .capability_ceiling()
        .iter()
        .map(|label| match label.as_str() {
            "context_read" => context_capability(),
            "mutation_intent" => mutation_capability(),
            "tool_use" => tool_capability(),
            other => panic!("unsupported test capability {other}"),
        })
        .collect()
}

fn context_capability() -> crate::TypedSubagentCapability {
    crate::TypedSubagentCapability::ContextRead {
        stable_label: "context_read".to_string(),
        context_label: "bounded-context".to_string(),
    }
}

fn mutation_capability() -> crate::TypedSubagentCapability {
    crate::TypedSubagentCapability::MutationIntent {
        stable_label: "mutation_intent".to_string(),
        mutation_kind: "candidate-edit".to_string(),
    }
}

fn tool_capability() -> crate::TypedSubagentCapability {
    crate::TypedSubagentCapability::ToolUse {
        stable_label: "tool_use".to_string(),
        tool_label: "inspect_context".to_string(),
    }
}

fn uniform_ceilings(capabilities: &[crate::TypedSubagentCapability]) -> AgentRoleCapabilityCeilings<'_> {
    AgentRoleCapabilityCeilings {
        verified_grants: capabilities,
        host_policy: capabilities,
        parent_delegation: capabilities,
        task_scope: capabilities,
        agent_definition: capabilities,
        model_profile: capabilities,
        workspace: capabilities,
        tool_executor: capabilities,
        approval: capabilities,
        sandbox: capabilities,
        cancellation: capabilities,
        timeout: capabilities,
        budget: capabilities,
    }
}

fn resolve_with_uniform_ceilings(
    assignment: &AgentRoleAssignment,
    profile: &AgentRoleProfile,
    capabilities: &[crate::TypedSubagentCapability],
) -> Result<AgentRoleCapabilityResolution, AgentRoleError> {
    resolve_agent_role_capabilities(AgentRoleCapabilityResolutionInput {
        assignment,
        profile,
        requested: capabilities,
        role_ceiling: capabilities,
        ceilings: uniform_ceilings(capabilities),
    })
}

fn output_resource_input<'a>(
    fixture: &'a AdmittedFixture,
    profile: &'a AgentRoleProfile,
    effect: AgentRoleOutputEffect,
) -> AgentRoleOutputResourceInput<'a> {
    AgentRoleOutputResourceInput {
        assignment: &fixture.assignment,
        profile,
        role_plan: &fixture.role_plan,
        capability_resolution: &fixture.capabilities,
        separation: &fixture.separation,
        status: AgentRoleOutputStatus::Admitted,
        effect,
        evidence_hash: EVIDENCE_HASH,
        verifier_hash: None,
        rubric_hash: None,
        admission: AgentRoleOutputAdmission::default(),
    }
}

fn output_draft<'a>(
    resource: AgentRoleOutputResourceInput<'a>,
    output_request: &'a animus_adapters::CapabilityRequest,
    output_receipt: &'a animus_adapters::AuthorizationReceipt,
) -> AgentRoleOutputDraft<'a> {
    AgentRoleOutputDraft {
        assignment: resource.assignment,
        profile: resource.profile,
        role_plan: resource.role_plan,
        capability_resolution: resource.capability_resolution,
        separation: resource.separation,
        status: resource.status,
        effect: resource.effect,
        evidence_hash: resource.evidence_hash,
        verifier_hash: resource.verifier_hash,
        rubric_hash: resource.rubric_hash,
        admission: resource.admission,
        claims: &[],
        output_request,
        output_receipt,
        sensitive: &[],
    }
}
