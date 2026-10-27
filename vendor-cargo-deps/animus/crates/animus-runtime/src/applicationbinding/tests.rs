use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const HASH_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const HASH_1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const HASH_2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const HASH_3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const HASH_4: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const HASH_5: &str = "5555555555555555555555555555555555555555555555555555555555555555";
const HASH_6: &str = "6666666666666666666666666666666666666666666666666666666666666666";
const HASH_7: &str = "7777777777777777777777777777777777777777777777777777777777777777";
const HASH_8: &str = "8888888888888888888888888888888888888888888888888888888888888888";
const HASH_9: &str = "9999999999999999999999999999999999999999999999999999999999999999";
const APPLICATION_SEQUENCE: u64 = 1;
const SESSION_SEQUENCE: u64 = 2;
const EFFECT_SEQUENCE: u64 = 3;
const DECLARED_VIEW_COUNT: usize = 9;

struct Fixture {
    application: PilotIdentity,
    entrypoint: PilotIdentity,
    input_schema: PilotIdentity,
    output_schema: PilotIdentity,
    effect_summary: PilotIdentity,
    operations: Vec<PilotIdentity>,
    hosts: Vec<PilotIdentity>,
    adapter: PilotIdentity,
    resource: PilotIdentity,
    policy: PilotIdentity,
    compatibility: PilotIdentity,
}

fn identity(domain: PilotIdentityDomain, hex: &str) -> PilotIdentity {
    admit_pilot_identity(domain, hex).expect("valid identity")
}

fn fixture(host_identity: &str) -> Fixture {
    Fixture {
        application: identity(PilotIdentityDomain::TargetArtifact, HASH_A),
        entrypoint: identity(PilotIdentityDomain::TargetArtifact, HASH_B),
        input_schema: identity(PilotIdentityDomain::Adapter, HASH_C),
        output_schema: identity(PilotIdentityDomain::Adapter, HASH_D),
        effect_summary: identity(PilotIdentityDomain::AlgebraicEffect, HASH_E),
        operations: vec![
            identity(PilotIdentityDomain::EffectOperation, HASH_F),
            identity(PilotIdentityDomain::EffectOperation, HASH_1),
        ],
        hosts: vec![identity(PilotIdentityDomain::Evaluator, host_identity)],
        adapter: identity(PilotIdentityDomain::Adapter, HASH_2),
        resource: identity(PilotIdentityDomain::Adapter, HASH_3),
        policy: identity(PilotIdentityDomain::Adapter, HASH_4),
        compatibility: identity(PilotIdentityDomain::EvidenceChain, HASH_5),
    }
}

const fn cohort() -> KamaciteContractCohort<'static> {
    KamaciteContractCohort {
        binding_schema: KAMACITE_BINDING_SCHEMA,
        receipt_schema: KAMACITE_BINDING_RECEIPT_SCHEMA,
        producer_revision: KAMACITE_PRODUCER_REVISION,
        schema_blake3: KAMACITE_SCHEMA_BLAKE3,
        receipt_schema_blake3: KAMACITE_RECEIPT_SCHEMA_BLAKE3,
        validation_receipt_blake3: KAMACITE_VALIDATION_RECEIPT_BLAKE3,
        core_blake3: KAMACITE_CORE_BLAKE3,
    }
}

fn draft(fixture: &Fixture) -> PilotBindingDraft<'_> {
    PilotBindingDraft {
        cohort: cohort(),
        application_identity: &fixture.application,
        entrypoint_identity: &fixture.entrypoint,
        input_schema_identity: &fixture.input_schema,
        output_schema_identity: &fixture.output_schema,
        effect_summary_identity: &fixture.effect_summary,
        effect_summary_closed: true,
        effect_operation_identities: fixture.operations.as_slice(),
        host_profile_identities: fixture.hosts.as_slice(),
        adapter_cohort_identity: &fixture.adapter,
        resource_profile_identity: &fixture.resource,
        policy_identity: &fixture.policy,
        compatibility_identity: &fixture.compatibility,
        expected_adapter_cohort_identity: &fixture.adapter,
        expected_resource_profile_identity: &fixture.resource,
        expected_host_profile_identity: &fixture.hosts[0],
        non_claims: &REQUIRED_NON_CLAIMS,
        semantic_claims: &[],
        sensitive: PilotSensitiveDraft::default(),
    }
}

fn session_authority() -> crate::session_authority_profile::SessionAuthorityBinding {
    let components = crate::session_authority_profile::SessionAuthorityComponentClass::ALL
        .iter()
        .copied()
        .map(
            |class| crate::session_authority_profile::SessionAuthorityComponentDraft {
                class,
                identity: HASH_6,
            },
        )
        .collect::<Vec<_>>();
    let profile = crate::session_authority_profile::build_session_authority_profile(
        crate::session_authority_profile::SessionAuthorityProfileDraft {
            components: components.as_slice(),
        },
    )
    .expect("profile");
    let authority = crate::session_authority_profile::build_session_authority_binding(
        crate::session_authority_profile::SessionAuthorityBindingDraft {
            profile: &profile,
            authorization_identity: HASH_7,
            branch_id: "pilot-branch",
            prior_head_hash: HASH_8,
            prior_profile_identity: None,
        },
    )
    .expect("binding");
    assert_eq!(
        authority.profile().profile_identity(),
        profile.profile_identity()
    );
    assert!(!authority.binding_identity().is_empty());
    authority
}

type DeclaredView<'a> = (
    crate::runtime_authority_seal::RuntimeAuthorityViewClass,
    &'a str,
);

fn declared_views<'a>(
    binding: &'a PilotApplicationHostBinding,
    session: &'a crate::session_authority_profile::SessionAuthorityBinding,
) -> [DeclaredView<'a>; DECLARED_VIEW_COUNT] {
    [
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::SessionLineage,
            HASH_A,
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::DurableBinding,
            session.binding_identity(),
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::AuthorityProfile,
            binding.host_profile_identity().hex(),
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::EnforcementObservation,
            HASH_B,
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::OperationSet,
            HASH_C,
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::AdapterCohort,
            binding.adapter_cohort_identity().hex(),
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::ResourceProfile,
            binding.resource_profile_identity().hex(),
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::ResourceBudget,
            HASH_D,
        ),
        (
            crate::runtime_authority_seal::RuntimeAuthorityViewClass::RuntimeServices,
            HASH_E,
        ),
    ]
}

fn expected_views<'a>(
    declared: &'a [DeclaredView<'a>],
) -> Vec<crate::runtime_authority_seal::RuntimeAuthorityExpectedViewDraft<'a>> {
    declared
        .iter()
        .map(
            |(class, value)| crate::runtime_authority_seal::RuntimeAuthorityExpectedViewDraft {
                class: *class,
                identity: value,
            },
        )
        .collect()
}

fn observed_views<'a>(
    declared: &'a [DeclaredView<'a>],
) -> Vec<crate::runtime_authority_seal::RuntimeAuthorityObservedViewDraft<'a>> {
    declared
        .iter()
        .map(
            |(class, value)| crate::runtime_authority_seal::RuntimeAuthorityObservedViewDraft {
                class: *class,
                identity: value,
                status: crate::runtime_authority_seal::RuntimeAuthorityObservationStatus::Current,
            },
        )
        .collect()
}

fn sealed_unit(
    binding: &PilotApplicationHostBinding,
    session: &crate::session_authority_profile::SessionAuthorityBinding,
) -> crate::runtime_authority_seal::RuntimeAuthorityUnit {
    let operation_values = binding
        .effect_operation_identities()
        .iter()
        .map(PilotIdentity::hex)
        .collect::<Vec<_>>();
    let resources = [HASH_9];
    let declared = declared_views(binding, session);
    let expected = expected_views(&declared);
    let observed = observed_views(&declared);
    let unit_draft = crate::runtime_authority_seal::RuntimeAuthorityUnitDraft {
        session_id: "session.pilot",
        dispatch_id: "dispatch.pilot",
        session_lineage_identity: HASH_A,
        durable_binding_identity: session.binding_identity(),
        authority_profile_identity: binding.host_profile_identity().hex(),
        enforcement_observation_identity: HASH_B,
        operation_set_identity: HASH_C,
        adapter_cohort_identity: binding.adapter_cohort_identity().hex(),
        resource_profile_identity: binding.resource_profile_identity().hex(),
        resource_budget_identity: HASH_D,
        runtime_services_identity: HASH_E,
        operation_identities: operation_values.as_slice(),
        resource_identities: &resources,
        expected_views: expected.as_slice(),
        observed_views: observed.as_slice(),
    };
    let plan = crate::runtime_authority_seal::prepare_runtime_authority_seal(
        unit_draft,
        crate::runtime_authority_seal::RuntimeAuthorityReceiptDraft {
            sensitive: crate::runtime_authority_seal::RuntimeAuthoritySensitiveDraft::default(),
            semantic_claims: &[],
        },
    )
    .expect("seal plan");
    let unit = crate::runtime_authority_seal::commit_runtime_authority_seal(
        &plan,
        plan.receipt().receipt_identity.as_str(),
    )
    .expect("sealed unit");
    assert_eq!(
        unit.state(),
        crate::runtime_authority_seal::RuntimeAuthorityUnitState::Sealed
    );
    assert!(unit.seal_receipt_identity().is_some());
    unit
}

fn recheck<'a>(
    binding: &'a PilotApplicationHostBinding,
    session: &'a crate::session_authority_profile::SessionAuthorityBinding,
) -> crate::runtime_authority_seal::RuntimeAuthorityRecheckDraft<'a> {
    let operations = binding
        .effect_operation_identities()
        .iter()
        .map(PilotIdentity::hex)
        .collect::<Vec<_>>();
    let resources = vec![HASH_9];
    let recheck = crate::runtime_authority_seal::RuntimeAuthorityRecheckDraft {
        session_lineage_identity: HASH_A,
        authority_profile_identity: binding.host_profile_identity().hex(),
        durable_binding_identity: session.binding_identity(),
        enforcement_observation_identity: HASH_B,
        operation_set_identity: HASH_C,
        adapter_cohort_identity: binding.adapter_cohort_identity().hex(),
        resource_profile_identity: binding.resource_profile_identity().hex(),
        resource_budget_identity: HASH_D,
        runtime_services_identity: HASH_E,
        operation_identities: Box::leak(operations.into_boxed_slice()),
        resource_identities: Box::leak(resources.into_boxed_slice()),
        admitted: true,
    };
    assert!(!recheck.operation_identities.is_empty());
    assert!(!recheck.resource_identities.is_empty());
    recheck
}

fn enforcement() -> Vec<PilotEnforcementObservation> {
    PilotEnforcementClass::ALL
        .iter()
        .copied()
        .map(|class| PilotEnforcementObservation {
            class,
            admitted: true,
        })
        .collect()
}

fn has_issue(error: &PilotError, code: PilotIssueCode) -> bool {
    error.issues().iter().any(|issue| issue.code == code)
}

#[test]
fn canonical_projection_is_stable_safe_and_kamacite_shaped() {
    let session = session_authority();
    let first_fixture = fixture(session.profile().profile_identity());
    let first = project_application_host_binding(&draft(&first_fixture)).expect("projection");
    let mut reversed_fixture = fixture(session.profile().profile_identity());
    reversed_fixture.operations.reverse();
    let second = project_application_host_binding(&draft(&reversed_fixture)).expect("projection");
    assert_eq!(first.binding_identity(), second.binding_identity());

    let json = render_kamacite_binding_json(&first);
    assert!(json.contains(KAMACITE_BINDING_SCHEMA));
    assert!(json.contains("\"effect_summary_closed\":true"));
    assert!(json.contains("\"alternative_host_profile_identities\":[]"));
    assert!(!json.contains("prompt"));
    assert!(!json.contains("credential"));
    assert!(!json.contains("/home/"));

    let explanation = explain_application_host_binding(&first);
    assert!(explanation.contains("compatibility=exact-frozen-kamacite-cohort"));
    assert!(explanation.contains(NEXT_FOCUSED_CHECK));
    for non_claim in REQUIRED_NON_CLAIMS {
        assert!(explanation.contains(non_claim));
    }
}

#[test]
fn committed_projection_and_existing_seal_precede_fake_backed_effect() {
    let session = session_authority();
    let fixture = fixture(session.profile().profile_identity());
    let binding = project_application_host_binding(&draft(&fixture)).expect("projection");
    let commit = commit_application_host_binding(PilotCommitDraft {
        binding: &binding,
        session_binding: &session,
        application_binding_sequence: APPLICATION_SEQUENCE,
        session_binding_sequence: SESSION_SEQUENCE,
    })
    .expect("commit");
    let sealed = sealed_unit(&binding, &session);
    let admission = prepare_pilot_dispatch(PilotDispatchDraft {
        binding: &binding,
        commit: &commit,
        sealed_unit: &sealed,
        operation: &fixture.operations[0],
        effect_sequence: EFFECT_SEQUENCE,
        enforcement: enforcement().as_slice(),
        recheck: recheck(&binding, &session),
    })
    .expect("dispatch admission");
    assert_eq!(
        admission.running_unit.state(),
        crate::runtime_authority_seal::RuntimeAuthorityUnitState::Running
    );

    let request = crate::dogfood_shell::DogfoodRunRequest {
        prompt_hash: HASH_A.to_owned(),
        policy_profile: "pilot-policy".to_owned(),
        model_capability: "model.fake".to_owned(),
        output_contract: "safe-summary".to_owned(),
        tool_metadata: Some("tool.fake".to_owned()),
        noninteractive: true,
    };
    let services = crate::dogfood_shell::DogfoodFakeServices {
        model_service: crate::dogfood_shell::DogfoodServiceState::Available,
        tool_service: crate::dogfood_shell::DogfoodServiceState::Available,
        allowed_capabilities: vec!["model.fake".to_owned()],
        model_response_summary: "accepted".to_owned(),
        tool_response_summary: Some("accepted".to_owned()),
    };
    assert!(crate::dogfood_shell::run_fake_dogfood_shell(&request, &services).is_ok());
}

#[test]
fn contract_effect_and_boundary_failures_are_stable() {
    let session = session_authority();
    let mut fixture = fixture(session.profile().profile_identity());

    let mut stale = draft(&fixture);
    stale.cohort.schema_blake3 = HASH_A;
    let error = project_application_host_binding(&stale).expect_err("stale");
    assert!(has_issue(&error, PilotIssueCode::StaleKamaciteContract));

    let mut missing_host = draft(&fixture);
    missing_host.host_profile_identities = &[];
    let error = project_application_host_binding(&missing_host).expect_err("missing host");
    assert!(has_issue(&error, PilotIssueCode::MissingHostProfile));

    fixture
        .hosts
        .push(identity(PilotIdentityDomain::Evaluator, HASH_9));
    let error = project_application_host_binding(&draft(&fixture)).expect_err("multiple hosts");
    assert!(has_issue(&error, PilotIssueCode::AmbiguousHostProfile));
    fixture.hosts.pop();

    let mut open = draft(&fixture);
    open.effect_summary_closed = false;
    let error = project_application_host_binding(&open).expect_err("open effects");
    assert!(has_issue(&error, PilotIssueCode::OpenEffectSummary));

    let duplicate_operations = vec![fixture.operations[0].clone(), fixture.operations[0].clone()];
    let mut duplicate = draft(&fixture);
    duplicate.effect_operation_identities = duplicate_operations.as_slice();
    let error = project_application_host_binding(&duplicate).expect_err("duplicate");
    assert!(has_issue(&error, PilotIssueCode::DuplicateEffectOperation));

    let overbound_operations = (0..=super::data::MAX_EFFECT_OPERATIONS)
        .map(|_| fixture.operations[0].clone())
        .collect::<Vec<_>>();
    let mut overbound_draft = draft(&fixture);
    overbound_draft.effect_operation_identities = overbound_operations.as_slice();
    let error = project_application_host_binding(&overbound_draft).expect_err("overbound");
    assert!(has_issue(&error, PilotIssueCode::TooManyEffectOperations));

    let wrong_domain_operations = vec![fixture.input_schema.clone()];
    let mut wrong_domain = draft(&fixture);
    wrong_domain.effect_operation_identities = wrong_domain_operations.as_slice();
    let error = project_application_host_binding(&wrong_domain).expect_err("wrong domain");
    assert!(has_issue(&error, PilotIssueCode::WrongIdentityDomain));

    let mut wrong_cohort = draft(&fixture);
    wrong_cohort.expected_adapter_cohort_identity = &fixture.policy;
    let error = project_application_host_binding(&wrong_cohort).expect_err("cohort");
    assert!(has_issue(&error, PilotIssueCode::CohortMismatch));

    let mut unsafe_draft = draft(&fixture);
    unsafe_draft.sensitive.prompt = Some("private prompt");
    let error = project_application_host_binding(&unsafe_draft).expect_err("unsafe");
    assert!(has_issue(&error, PilotIssueCode::UnsafeField));

    let mut claims = draft(&fixture);
    claims.semantic_claims = &["binding authorizes execution"];
    claims.non_claims = &[];
    let error = project_application_host_binding(&claims).expect_err("overclaim");
    assert!(has_issue(&error, PilotIssueCode::AuthorityOverclaim));
    assert!(has_issue(&error, PilotIssueCode::MissingNonClaim));
    let explanation = explain_application_host_binding_error(&error);
    assert!(explanation.contains("APPLICATION_BINDING_AUTHORITY_OVERCLAIM"));
    assert!(!explanation.contains("binding authorizes execution"));
}

#[test]
fn dispatch_denies_early_unknown_widened_and_current_failures() {
    let session = session_authority();
    let fixture = fixture(session.profile().profile_identity());
    let binding = project_application_host_binding(&draft(&fixture)).expect("projection");
    let commit = commit_application_host_binding(PilotCommitDraft {
        binding: &binding,
        session_binding: &session,
        application_binding_sequence: APPLICATION_SEQUENCE,
        session_binding_sequence: SESSION_SEQUENCE,
    })
    .expect("commit");
    let sealed = sealed_unit(&binding, &session);

    let error = prepare_pilot_dispatch(PilotDispatchDraft {
        binding: &binding,
        commit: &commit,
        sealed_unit: &sealed,
        operation: &fixture.operations[0],
        effect_sequence: SESSION_SEQUENCE,
        enforcement: enforcement().as_slice(),
        recheck: recheck(&binding, &session),
    })
    .expect_err("early effect");
    assert!(has_issue(&error, PilotIssueCode::EffectBeforeBinding));

    let unknown = identity(PilotIdentityDomain::EffectOperation, HASH_9);
    let error = prepare_pilot_dispatch(PilotDispatchDraft {
        binding: &binding,
        commit: &commit,
        sealed_unit: &sealed,
        operation: &unknown,
        effect_sequence: EFFECT_SEQUENCE,
        enforcement: enforcement().as_slice(),
        recheck: recheck(&binding, &session),
    })
    .expect_err("unknown operation");
    assert!(has_issue(&error, PilotIssueCode::UndeclaredOperation));
    assert!(has_issue(&error, PilotIssueCode::WidenedSeal));

    let mut failed_enforcement = enforcement();
    failed_enforcement[0].admitted = false;
    let error = prepare_pilot_dispatch(PilotDispatchDraft {
        binding: &binding,
        commit: &commit,
        sealed_unit: &sealed,
        operation: &fixture.operations[0],
        effect_sequence: EFFECT_SEQUENCE,
        enforcement: failed_enforcement.as_slice(),
        recheck: recheck(&binding, &session),
    })
    .expect_err("current denial");
    assert!(has_issue(&error, PilotIssueCode::CurrentEnforcementDenied));

    let mut denied_recheck = recheck(&binding, &session);
    denied_recheck.admitted = false;
    let error = prepare_pilot_dispatch(PilotDispatchDraft {
        binding: &binding,
        commit: &commit,
        sealed_unit: &sealed,
        operation: &fixture.operations[0],
        effect_sequence: EFFECT_SEQUENCE,
        enforcement: enforcement().as_slice(),
        recheck: denied_recheck,
    })
    .expect_err("existing recheck denial");
    assert!(has_issue(&error, PilotIssueCode::CurrentEnforcementDenied));
}
