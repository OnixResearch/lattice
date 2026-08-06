use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const HASH_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const HASH_ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const HASH_ONE: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const HASH_TWO: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const HASH_THREE: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const HASH_FOUR: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const SESSION_ID: &str = "session.authority-fixture";
const BRANCH_ID: &str = "branch.main";
const FORK_BRANCH_ID: &str = "branch.reauthorized";
const EXPECTED_COMPONENT_COUNT: usize = 10;
const EXPECTED_HASH_HEX_LENGTH: usize = 64;
const GENERATED_PROFILE_JSON: &str = include_str!("../../../../config/generated/session-authority-profile.json");

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedAuthorityProfile {
    schema: String,
    components: Vec<GeneratedAuthorityComponent>,
    expected_profile_identity: String,
    redaction_marker: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratedAuthorityComponent {
    class: String,
    identity: String,
}

#[test]
fn profile_identity_is_canonical_across_input_order() {
    // r[verify agent_harness_runtime.session_authority_profile.identity]
    let forward = component_drafts(false, HASH_B);
    let reverse = component_drafts(true, HASH_B);
    let first = build_session_authority_profile(SessionAuthorityProfileDraft {
        components: forward.as_slice(),
    })
    .expect("forward profile");
    let second = build_session_authority_profile(SessionAuthorityProfileDraft {
        components: reverse.as_slice(),
    })
    .expect("reverse profile");
    assert_eq!(first, second);
    assert_eq!(first.components().len(), EXPECTED_COMPONENT_COUNT);
    assert_eq!(first.components()[0].class(), SessionAuthorityComponentClass::PolicyAuthorization);
    assert_eq!(first.profile_identity().len(), EXPECTED_HASH_HEX_LENGTH);
}

#[test]
fn nickel_profile_matches_rust_canonical_identity() {
    // r[verify agent_harness_runtime.session_authority_profile.identity]
    let generated: GeneratedAuthorityProfile =
        serde_json::from_str(GENERATED_PROFILE_JSON).expect("generated authority profile");
    assert_eq!(generated.schema, SESSION_AUTHORITY_PROFILE_SCHEMA);
    assert!(generated.redaction_marker.contains("omitted"));
    let classes = generated
        .components
        .iter()
        .map(|component| {
            SessionAuthorityComponentClass::from_label(component.class.as_str()).expect("known component class")
        })
        .collect::<Vec<_>>();
    let drafts = generated
        .components
        .iter()
        .zip(classes.iter())
        .map(|(component, class)| SessionAuthorityComponentDraft {
            class: *class,
            identity: component.identity.as_str(),
        })
        .collect::<Vec<_>>();
    let profile = build_session_authority_profile(SessionAuthorityProfileDraft {
        components: drafts.as_slice(),
    })
    .expect("Rust profile");
    assert_eq!(profile.profile_identity(), generated.expected_profile_identity);
}

#[test]
fn missing_duplicate_and_unsafe_components_fail_closed() {
    let complete = component_drafts(false, HASH_B);
    let missing = &complete[..complete.len().saturating_sub(1)];
    let missing_error = build_session_authority_profile(SessionAuthorityProfileDraft { components: missing })
        .expect_err("missing component");
    assert_eq!(missing_error.issue(), SessionAuthorityIssue::MissingComponent);

    let mut duplicate = complete.clone();
    duplicate[1].class = SessionAuthorityComponentClass::PolicyAuthorization;
    let duplicate_error = build_session_authority_profile(SessionAuthorityProfileDraft {
        components: duplicate.as_slice(),
    })
    .expect_err("duplicate component");
    assert_eq!(duplicate_error.issue(), SessionAuthorityIssue::DuplicateComponent);

    let mut unsafe_components = complete;
    unsafe_components[0].identity = "/home/private/policy";
    let unsafe_error = build_session_authority_profile(SessionAuthorityProfileDraft {
        components: unsafe_components.as_slice(),
    })
    .expect_err("unsafe identity");
    assert_eq!(unsafe_error.issue(), SessionAuthorityIssue::UnsafeIdentity);
}

#[test]
fn binding_payload_round_trips_and_tampering_fails() {
    // r[verify agent_harness_runtime.session_authority_profile.durable_binding]
    let profile = profile(HASH_B);
    let binding = binding(&profile, None, BRANCH_ID);
    let payload = encode_session_authority_binding(&binding).expect("encode binding");
    let decoded = decode_session_authority_binding(payload.as_str()).expect("decode binding");
    assert_eq!(decoded, binding);
    assert!(!payload.contains("/home/"));
    assert!(!payload.contains("prompt"));

    let tampered = payload.replacen(binding.binding_identity(), HASH_C, 1);
    let tampered_error = decode_session_authority_binding(tampered.as_str()).expect_err("tampered binding identity");
    assert_eq!(tampered_error.issue(), SessionAuthorityIssue::IdentityMismatch);
}

#[test]
fn resume_comparison_distinguishes_exact_drift_and_legacy() {
    // r[verify agent_harness_runtime.session_authority_profile.resume_admission]
    let committed = profile(HASH_B);
    let exact = compare_session_authority_profiles(Some(&committed), &committed).expect("exact comparison");
    assert_eq!(exact.status(), SessionAuthorityResumeStatus::ExactMatch);
    assert!(exact.changed_components().is_empty());

    let changed = profile(HASH_C);
    let drift = compare_session_authority_profiles(Some(&committed), &changed).expect("drift comparison");
    assert_eq!(drift.status(), SessionAuthorityResumeStatus::DriftBlocked);
    assert_eq!(drift.changed_components(), &[SessionAuthorityComponentClass::Permission]);

    let legacy = compare_session_authority_profiles(None, &changed).expect("legacy comparison");
    assert_eq!(legacy.status(), SessionAuthorityResumeStatus::LegacyUnbound);
    assert_eq!(legacy.changed_components().len(), EXPECTED_COMPONENT_COUNT);
}

#[test]
fn every_component_class_is_reported_as_drift() {
    // r[verify agent_harness_runtime.session_authority_profile.verification]
    let committed = profile(HASH_B);
    for (index, expected_class) in SessionAuthorityComponentClass::ALL.iter().enumerate() {
        let mut changed_components = component_drafts(false, HASH_B);
        changed_components[index].identity = HASH_FOUR;
        let changed = build_session_authority_profile(SessionAuthorityProfileDraft {
            components: changed_components.as_slice(),
        })
        .expect("changed profile");
        let comparison = compare_session_authority_profiles(Some(&committed), &changed).expect("drift comparison");
        assert_eq!(comparison.status(), SessionAuthorityResumeStatus::DriftBlocked);
        assert_eq!(comparison.changed_components(), &[*expected_class]);
    }
}

#[test]
fn explicit_rebind_plans_successor_binding_without_rewriting_prior_profile() {
    let committed = profile(HASH_B);
    let requested = profile(HASH_C);
    let comparison = compare_session_authority_profiles(Some(&committed), &requested).expect("drift comparison");
    let plan = plan_session_authority_transition(SessionAuthorityTransitionRequest {
        comparison: &comparison,
        requested_profile: &requested,
        prior_head_hash: HASH_ZERO,
        action: SessionAuthorityTransitionAction::Rebind {
            branch_id: FORK_BRANCH_ID,
            authorization_identity: HASH_ONE,
        },
    })
    .expect("rebind plan");
    assert_eq!(plan.disposition(), SessionAuthorityTransitionDisposition::RebindCommitRequired);
    let planned_binding = plan.binding().expect("binding");
    assert_eq!(planned_binding.profile(), &requested);
    assert_eq!(planned_binding.prior_profile_identity(), Some(committed.profile_identity()));
    assert_eq!(
        committed.component(SessionAuthorityComponentClass::Permission).expect("permission").identity(),
        HASH_B
    );
}

#[test]
fn exact_rebind_and_missing_authorization_fail_closed() {
    let current = profile(HASH_B);
    let comparison = compare_session_authority_profiles(Some(&current), &current).expect("exact comparison");
    let exact_error = plan_session_authority_transition(SessionAuthorityTransitionRequest {
        comparison: &comparison,
        requested_profile: &current,
        prior_head_hash: HASH_ZERO,
        action: SessionAuthorityTransitionAction::Rebind {
            branch_id: FORK_BRANCH_ID,
            authorization_identity: HASH_ONE,
        },
    })
    .expect_err("exact rebind");
    assert_eq!(exact_error.issue(), SessionAuthorityIssue::InPlaceRewrite);

    let drifted = profile(HASH_C);
    let drift = compare_session_authority_profiles(Some(&current), &drifted).expect("drift comparison");
    let authorization_error = plan_session_authority_transition(SessionAuthorityTransitionRequest {
        comparison: &drift,
        requested_profile: &drifted,
        prior_head_hash: HASH_ZERO,
        action: SessionAuthorityTransitionAction::Rebind {
            branch_id: FORK_BRANCH_ID,
            authorization_identity: "missing",
        },
    })
    .expect_err("missing authorization");
    assert_eq!(authorization_error.issue(), SessionAuthorityIssue::UnsafeIdentity);
}

#[test]
fn realized_enforcement_is_current_separate_and_fail_closed() {
    // r[verify agent_harness_runtime.session_authority_profile.realized_enforcement]
    let current_profile = profile(HASH_B);
    let admitted = enforcement(current_profile.profile_identity(), true);
    validate_session_authority_enforcement(&current_profile, &admitted).expect("admitted enforcement");
    assert!(admitted.admitted());
    assert_ne!(admitted.observation_identity(), current_profile.profile_identity());

    let denied = enforcement(current_profile.profile_identity(), false);
    let denied_error =
        validate_session_authority_enforcement(&current_profile, &denied).expect_err("denied enforcement");
    assert_eq!(denied_error.issue(), SessionAuthorityIssue::EnforcementUnavailable);

    let other_profile = profile(HASH_C);
    let mismatch_error =
        validate_session_authority_enforcement(&other_profile, &admitted).expect_err("profile mismatch");
    assert_eq!(mismatch_error.issue(), SessionAuthorityIssue::IdentityMismatch);
}

#[test]
fn receipt_is_metadata_only_and_rejects_private_material() {
    // r[verify agent_harness_runtime.session_authority_profile.receipts]
    let profile = profile(HASH_B);
    let comparison = compare_session_authority_profiles(Some(&profile), &profile).expect("comparison");
    let binding = binding(&profile, None, BRANCH_ID);
    let enforcement = enforcement(profile.profile_identity(), true);
    let receipt = project_session_authority_receipt(SessionAuthorityReceiptDraft {
        session_id: SESSION_ID,
        branch_id: BRANCH_ID,
        head_hash: HASH_ZERO,
        comparison: &comparison,
        binding: Some(&binding),
        enforcement: Some(&enforcement),
        sensitive: SessionAuthoritySensitiveDraft::default(),
    })
    .expect("receipt");
    assert_eq!(receipt.status, SessionAuthorityResumeStatus::ExactMatch);
    assert_eq!(receipt.binding_identity.as_deref(), Some(binding.binding_identity()));
    assert!(!receipt.non_claims.is_empty());

    let leak_error = project_session_authority_receipt(SessionAuthorityReceiptDraft {
        session_id: SESSION_ID,
        branch_id: BRANCH_ID,
        head_hash: HASH_ZERO,
        comparison: &comparison,
        binding: Some(&binding),
        enforcement: None,
        sensitive: SessionAuthoritySensitiveDraft {
            credential: Some("secret"),
            ..SessionAuthoritySensitiveDraft::default()
        },
    })
    .expect_err("credential leak");
    assert_eq!(leak_error.issue(), SessionAuthorityIssue::UnsafeReceipt);
}

fn component_drafts(reverse: bool, permission_identity: &'static str) -> Vec<SessionAuthorityComponentDraft<'static>> {
    let mut components = vec![
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::PolicyAuthorization,
            identity: HASH_A,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::Permission,
            identity: permission_identity,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::SandboxIntent,
            identity: HASH_C,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::ModelRuntime,
            identity: HASH_D,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::ToolCatalog,
            identity: HASH_E,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::CapabilityCeiling,
            identity: HASH_F,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::HookPluginSet,
            identity: HASH_ZERO,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::AgentDefinitionSet,
            identity: HASH_ONE,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::ContextRetrieval,
            identity: HASH_TWO,
        },
        SessionAuthorityComponentDraft {
            class: SessionAuthorityComponentClass::AdapterCohort,
            identity: HASH_THREE,
        },
    ];
    assert_eq!(components.len(), SessionAuthorityComponentClass::ALL.len());
    assert_eq!(
        components
            .iter()
            .filter(|component| component.class == SessionAuthorityComponentClass::Permission)
            .count(),
        1
    );
    if reverse {
        components.reverse();
    }
    components
}

fn profile(permission_identity: &'static str) -> SessionRuntimeAuthorityProfile {
    let components = component_drafts(false, permission_identity);
    build_session_authority_profile(SessionAuthorityProfileDraft {
        components: components.as_slice(),
    })
    .expect("profile")
}

fn binding(
    profile: &SessionRuntimeAuthorityProfile,
    prior_profile_identity: Option<&str>,
    branch_id: &str,
) -> SessionAuthorityBinding {
    build_session_authority_binding(SessionAuthorityBindingDraft {
        profile,
        authorization_identity: HASH_FOUR,
        branch_id,
        prior_head_hash: HASH_ZERO,
        prior_profile_identity,
    })
    .expect("binding")
}

fn enforcement(profile_identity: &str, admitted: bool) -> SessionAuthorityEnforcementObservation {
    build_session_authority_enforcement_observation(SessionAuthorityEnforcementDraft {
        profile_identity,
        authorization_identity: HASH_FOUR,
        adapter_cohort_identity: HASH_THREE,
        capability_observation_identity: HASH_TWO,
        tool_readiness_identity: HASH_ONE,
        provider_readiness_identity: HASH_ZERO,
        sandbox_observation_identity: HASH_F,
        admitted,
    })
    .expect("enforcement")
}
