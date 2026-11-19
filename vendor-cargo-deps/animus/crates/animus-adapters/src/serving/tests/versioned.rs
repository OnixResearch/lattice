//! Version-bound rollout eligibility, admission, and response provenance.

const BASE_MODEL: &str = "blake3:1111111111111111111111111111111111111111111111111111111111111111";
const TOKENIZER: &str = "blake3:2222222222222222222222222222222222222222222222222222222222222222";
const ADAPTER: &str = "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const OTHER_ARTIFACT: &str =
    "blake3:4444444444444444444444444444444444444444444444444444444444444444";
const QUALIFICATION: &str =
    "blake3:5555555555555555555555555555555555555555555555555555555555555555";
const BACKEND: &str = "blake3:6666666666666666666666666666666666666666666666666666666666666666";
const POLICY_GENERATION: u64 = 7;
const PREVIOUS_GENERATION: u64 = 6;
const NEXT_GENERATION: u64 = 8;
const CROSSED_GENERATION: u64 = 3;
const OPERATIONS: [crate::serving::RolloutOperation; 2] = [
    crate::serving::RolloutOperation::TokenLogprobs,
    crate::serving::RolloutOperation::AdapterRouting,
];
const LOGPROBS_ONLY: [crate::serving::RolloutOperation; 1] =
    [crate::serving::RolloutOperation::TokenLogprobs];

type FactsEdit = fn(&mut crate::serving::RolloutDraft<'static>);

fn facts() -> crate::serving::RolloutDraft<'static> {
    crate::serving::RolloutDraft {
        qualification_ref: QUALIFICATION,
        qualified_backend_ref: BACKEND,
        qualified_configuration_ref: super::fixtures::CONFIG_A,
        base_model_ref: BASE_MODEL,
        tokenizer_ref: TOKENIZER,
        adapter_ref: Some(ADAPTER),
        loaded_policy_generation: POLICY_GENERATION,
        activation: crate::serving::Attestation::Observed,
        probability_semantics: crate::serving::ProbabilitySemantics::SampledTokenLogprobs,
        operations: &OPERATIONS,
        qualified_envelope: super::fixtures::envelope(),
    }
}

const fn requirement() -> crate::serving::RolloutRequirementDraft<'static> {
    crate::serving::RolloutRequirementDraft {
        base_model_ref: BASE_MODEL,
        tokenizer_ref: TOKENIZER,
        adapter_ref: Some(ADAPTER),
        policy_generation: POLICY_GENERATION,
        probability_semantics: crate::serving::ProbabilitySemantics::SampledTokenLogprobs,
        operations: &OPERATIONS,
    }
}

fn bound_candidate() -> crate::serving::RouteCandidate {
    super::fixtures::candidate_with(|draft| draft.rollout = Some(requirement()))
}

fn bound_screening() -> crate::serving::Screening {
    let snapshot = super::fixtures::snapshot_with(|service| service.rollout = Some(facts()));
    super::fixtures::screen(&super::fixtures::profile(), &snapshot, &[bound_candidate()])
}

fn reason_with(edit: FactsEdit) -> crate::serving::ExclusionReason {
    let mut draft = facts();
    edit(&mut draft);
    let snapshot = super::fixtures::snapshot_with(|service| service.rollout = Some(draft));
    let screening =
        super::fixtures::screen(&super::fixtures::profile(), &snapshot, &[bound_candidate()]);
    assert!(screening.eligible().is_empty());
    screening.exclusions()[0].reason()
}

#[test]
fn version_bound_route_is_eligible_and_ordinary_routes_ignore_facts() {
    assert_eq!(bound_screening().eligible().len(), 1);
    let without = super::fixtures::snapshot_with(|_| {});
    let profile = super::fixtures::profile();
    let ordinary = super::fixtures::screen(&profile, &without, &[super::fixtures::candidate()]);
    assert_eq!(ordinary.eligible().len(), 1);
    let required = super::fixtures::screen(&profile, &without, &[bound_candidate()]);
    assert_eq!(
        required.exclusions()[0].reason(),
        crate::serving::ExclusionReason::RolloutUnavailable
    );
}

#[test]
fn mismatched_facts_stay_unavailable() {
    let cases: [(FactsEdit, crate::serving::RolloutField); 9] = [
        (
            |draft| draft.adapter_ref = Some(OTHER_ARTIFACT),
            crate::serving::RolloutField::Adapter,
        ),
        (
            |draft| draft.base_model_ref = OTHER_ARTIFACT,
            crate::serving::RolloutField::BaseModel,
        ),
        (
            |draft| draft.tokenizer_ref = OTHER_ARTIFACT,
            crate::serving::RolloutField::Tokenizer,
        ),
        (
            |draft| draft.qualified_configuration_ref = super::fixtures::CONFIG_B,
            crate::serving::RolloutField::QualificationConfiguration,
        ),
        (
            |draft| draft.activation = crate::serving::Attestation::Claimed,
            crate::serving::RolloutField::Activation,
        ),
        (
            |draft| draft.loaded_policy_generation = PREVIOUS_GENERATION,
            crate::serving::RolloutField::PolicyGeneration,
        ),
        (
            |draft| draft.operations = &LOGPROBS_ONLY,
            crate::serving::RolloutField::Operations,
        ),
        (
            |draft| draft.probability_semantics = crate::serving::ProbabilitySemantics::Absent,
            crate::serving::RolloutField::ProbabilitySemantics,
        ),
        (
            |draft| {
                draft.qualified_envelope =
                    crate::serving::Envelope::new(1_024, 256).unwrap_or(draft.qualified_envelope);
            },
            crate::serving::RolloutField::QualifiedEnvelope,
        ),
    ];
    for (edit, field) in cases {
        assert_eq!(
            reason_with(edit),
            crate::serving::ExclusionReason::RolloutMismatch(field)
        );
    }
    assert_eq!(crate::serving::RolloutField::Adapter.as_str(), "adapter");
}

fn bound_verdict(
    grant_generation: Option<u64>,
    current_generation: Option<u64>,
) -> crate::serving::GateVerdict {
    let screening = bound_screening();
    let query = crate::serving::admission_query(
        &screening.eligible()[0],
        &super::fixtures::caller(),
        super::fixtures::REQUEST_ONE,
    )
    .expect("query");
    assert_eq!(query.policy_generation(), Some(POLICY_GENERATION));
    let owner = crate::serving::OwnerReply::granted(
        crate::serving::Grant::new(&crate::serving::GrantDraft {
            policy_generation: grant_generation,
            ..super::fixtures::grant_draft()
        })
        .expect("grant"),
    );
    let mut state = super::fixtures::current();
    state.policy_generation = current_generation;
    assert_eq!(state.readiness, crate::serving::Readiness::Ready);
    crate::serving::gate_dispatch(&crate::serving::GateInput {
        profile: &super::fixtures::profile(),
        query: &query,
        owner: &owner,
        current: crate::serving::CurrentReading::Observed(state),
        existing: super::fixtures::existing(),
    })
}

#[test]
fn admission_preserves_the_required_policy_generation() {
    let generation = Some(POLICY_GENERATION);
    assert!(matches!(
        bound_verdict(generation, generation),
        crate::serving::GateVerdict::Admitted(_)
    ));
    assert_eq!(
        bound_verdict(None, generation),
        crate::serving::GateVerdict::Rejected(
            crate::serving::GateRejection::PolicyGenerationUnestablished
        )
    );
    assert_eq!(
        bound_verdict(Some(NEXT_GENERATION), generation),
        crate::serving::GateVerdict::Rejected(
            crate::serving::GateRejection::PolicyGenerationMismatch
        )
    );
    assert_eq!(
        bound_verdict(generation, Some(NEXT_GENERATION)),
        crate::serving::GateVerdict::Rejected(
            crate::serving::GateRejection::PolicyGenerationMismatch
        )
    );
}

#[test]
fn response_generation_provenance_is_propagated_without_rewriting() {
    let screening = bound_screening();
    let query = crate::serving::admission_query(
        &screening.eligible()[0],
        &super::fixtures::caller(),
        super::fixtures::REQUEST_ONE,
    )
    .expect("query");
    let model = Some(super::fixtures::MODEL_A);
    let with_generation = |policy_generation| crate::serving::ResponseFacts {
        usage: crate::serving::Usage::UNKNOWN,
        policy_generation,
    };
    let matched = crate::serving::response_provenance(
        &query,
        model,
        &with_generation(Some(POLICY_GENERATION)),
    );
    assert_eq!(
        matched.policy_generation,
        crate::serving::PolicyGenerationProvenance::Matched(POLICY_GENERATION)
    );
    assert_eq!(matched.rejection(), None);
    let missing = crate::serving::response_provenance(&query, model, &with_generation(None));
    assert_eq!(
        missing.rejection(),
        Some(crate::serving::ResponseRejection::PolicyGenerationMissing)
    );
    let crossed = crate::serving::response_provenance(
        &query,
        model,
        &with_generation(Some(CROSSED_GENERATION)),
    );
    assert_eq!(
        crossed.policy_generation,
        crate::serving::PolicyGenerationProvenance::Crossed {
            required: POLICY_GENERATION,
            observed: CROSSED_GENERATION
        }
    );
    let no_model = crate::serving::response_provenance(
        &query,
        None,
        &with_generation(Some(POLICY_GENERATION)),
    );
    assert_eq!(
        no_model.rejection(),
        Some(crate::serving::ResponseRejection::ModelMissing)
    );
}

#[test]
fn rollout_operations_are_bounded_and_unique() {
    let repeated = [
        crate::serving::RolloutOperation::TokenLogprobs,
        crate::serving::RolloutOperation::TokenLogprobs,
    ];
    assert_eq!(
        crate::serving::RolloutRequirement::new(&crate::serving::RolloutRequirementDraft {
            operations: &repeated,
            ..requirement()
        }),
        Err(crate::serving::ContractIssue::Duplicate {
            field: "rollout.operations"
        })
    );
    assert_eq!(
        crate::serving::RolloutRequirement::new(&crate::serving::RolloutRequirementDraft {
            policy_generation: 0,
            ..requirement()
        }),
        Err(crate::serving::ContractIssue::ZeroValue {
            field: "rollout.policy_generation"
        })
    );
}
