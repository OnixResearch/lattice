use super::*;

const MAX_CONTRIBUTIONS: u16 = 32;
const MAX_OMISSIONS: u16 = 32;
const CONTEXT_BUDGET: u32 = 12;
const MANDATORY_COST: u32 = 3;
const ADVISORY_COST: u32 = 4;
const HIGH_PRIORITY: i16 = 20;
const MEDIUM_PRIORITY: i16 = 10;
const LOW_PRIORITY: i16 = 2;
const FIRST_ATTEMPT: u16 = 1;
const SECOND_ATTEMPT: u16 = 2;
const REDUCER_BUDGET: u32 = MANDATORY_COST.saturating_add(ADVISORY_COST);
const INSUFFICIENT_BUDGET: u32 = MANDATORY_COST.saturating_sub(1);

fn policy(context_budget: u32) -> ContributionPolicy {
    ContributionPolicy {
        max_contributions: MAX_CONTRIBUTIONS,
        context_budget,
        max_omissions: MAX_OMISSIONS,
    }
}

fn binding(request_id: &str) -> ContributionBinding {
    ContributionBinding {
        target: ModelTargetIdentity::new("target-a"),
        profile: ModelProfileIdentity::new("profile-a"),
        prompt_plan: PromptPlanIdentity::new("prompt-plan-a"),
        invocation: ModelInvocationIdentity::new("invocation-a"),
        policy: PolicyIdentity::new("policy-a"),
        authority: AuthorityIdentity::new("authority-a"),
        provider_request: ProviderRequestIdentity::new(request_id),
    }
}

#[derive(Clone, Copy)]
struct TestContribution<'a> {
    id: &'a str,
    body: &'a str,
    source: ContributionSource,
    posture: ContributionPosture,
    priority: i16,
    cost: u32,
    deduplication_key: &'a str,
}

fn contribution(input: TestContribution<'_>) -> Contribution {
    contribution_with_lifetime(input, ContributionLifetime::CurrentRequest)
}

fn contribution_with_lifetime(
    input: TestContribution<'_>,
    lifetime: ContributionLifetime,
) -> Contribution {
    let payload = blake3_payload_identity(input.body.as_bytes());
    validate_contribution(ContributionDraft {
        contribution_id: input.id,
        source: input.source,
        trust: ContributionTrust::Trusted,
        posture: input.posture,
        priority: input.priority,
        budget_cost: input.cost,
        payload_identity: payload.as_str(),
        deduplication_key: input.deduplication_key,
        supersedes: &[],
        conflict_group: None,
        lifetime,
        sensitivity: ContributionSensitivity::RequestLocal,
        safe_label: input.id,
    })
    .expect("fixture contribution must validate")
}

fn selected_ids(plan: &ContributionPlan) -> Vec<&str> {
    plan.selected
        .iter()
        .map(|entry| entry.contribution_id().as_str())
        .collect()
}

fn lifetime_contribution(id: &str, lifetime: ContributionLifetime) -> Contribution {
    contribution_with_lifetime(
        TestContribution {
            id,
            body: "finite body",
            source: ContributionSource::Runtime,
            posture: ContributionPosture::Mandatory,
            priority: HIGH_PRIORITY,
            cost: MANDATORY_COST,
            deduplication_key: id,
        },
        lifetime,
    )
}

#[test]
fn declared_mandatory_identities_must_be_present() {
    // r[verify agent_harness_runtime.request_context_contributions.reducer.mandatory]
    let mandatory = sensitive_contribution();
    let required = [mandatory.contribution_id().clone()];
    let plan = reduce_required_contributions(ContributionReductionInput {
        candidates: std::slice::from_ref(&mandatory),
        required: &required,
        binding: &binding("request-required"),
        policy: policy(CONTEXT_BUDGET),
    })
    .expect("declared mandatory contribution must reduce");
    assert_eq!(selected_ids(&plan), vec!["sensitive-skill"]);

    let missing = [ContributionIdentity::new("missing-policy")];
    assert_eq!(
        reduce_required_contributions(ContributionReductionInput {
            candidates: std::slice::from_ref(&mandatory),
            required: &missing,
            binding: &binding("request-required"),
            policy: policy(CONTEXT_BUDGET),
        }),
        Err(ContributionIssue::MissingMandatoryContribution)
    );
    let advisory = contribution(TestContribution {
        id: "advisory-policy",
        body: "advisory",
        source: ContributionSource::Prompt,
        posture: ContributionPosture::Advisory,
        priority: LOW_PRIORITY,
        cost: ADVISORY_COST,
        deduplication_key: "advisory-policy",
    });
    let required_advisory = [advisory.contribution_id().clone()];
    assert_eq!(
        reduce_required_contributions(ContributionReductionInput {
            candidates: std::slice::from_ref(&advisory),
            required: &required_advisory,
            binding: &binding("request-required"),
            policy: policy(CONTEXT_BUDGET),
        }),
        Err(ContributionIssue::MissingMandatoryContribution)
    );
}

#[test]
fn finite_lifetimes_expire_only_on_exact_terminal_observations() {
    // r[verify agent_harness_runtime.request_context_contributions.retention.expires]
    let request_id = ProviderRequestIdentity::new("request-lifetime");
    let turn_id = LogicalTurnIdentity::new("turn-lifetime");
    let request = lifetime_contribution("request-context", ContributionLifetime::CurrentRequest);
    assert_eq!(
        advance_contribution_lifetime(
            &request,
            &request_id,
            &turn_id,
            &ContributionTerminalObservation::RequestAdmitted(request_id.clone()),
        ),
        Ok(ContributionLifetimeState::Expired)
    );
    let response = lifetime_contribution(
        "response-context",
        ContributionLifetime::UntilNextProviderResponse,
    );
    let response_observation = ContributionTerminalObservation::ProviderResponse {
        request_id: request_id.clone(),
        response_id: ProviderResponseIdentity::new("response-lifetime"),
    };
    assert_eq!(
        advance_contribution_lifetime(&response, &request_id, &turn_id, &response_observation),
        Ok(ContributionLifetimeState::Expired)
    );
    let turn = lifetime_contribution("turn-context", ContributionLifetime::CurrentLogicalTurn);
    assert_eq!(
        advance_contribution_lifetime(
            &turn,
            &request_id,
            &turn_id,
            &ContributionTerminalObservation::LogicalTurnCompleted(turn_id.clone()),
        ),
        Ok(ContributionLifetimeState::Expired)
    );
    assert_eq!(
        advance_contribution_lifetime(
            &response,
            &request_id,
            &turn_id,
            &ContributionTerminalObservation::ProviderResponse {
                request_id: ProviderRequestIdentity::new("stale-request"),
                response_id: ProviderResponseIdentity::new("response-lifetime"),
            },
        ),
        Err(ContributionIssue::InvocationDrift)
    );
}

#[test]
fn reducer_is_order_independent_and_reserves_mandatory_budget() {
    // r[verify agent_harness_runtime.request_context_contributions.contract.valid]
    // r[verify agent_harness_runtime.request_context_contributions.reducer.order_independent]
    // r[verify agent_harness_runtime.request_context_contributions.reducer.advisory]
    let mandatory = contribution(TestContribution {
        id: "mandatory-policy",
        body: "policy body",
        source: ContributionSource::Prompt,
        posture: ContributionPosture::Mandatory,
        priority: HIGH_PRIORITY,
        cost: MANDATORY_COST,
        deduplication_key: "policy",
    });
    let skill = contribution(TestContribution {
        id: "skill-context",
        body: "skill body",
        source: ContributionSource::Skill,
        posture: ContributionPosture::Advisory,
        priority: MEDIUM_PRIORITY,
        cost: ADVISORY_COST,
        deduplication_key: "skill",
    });
    let memory = contribution(TestContribution {
        id: "memory-context",
        body: "memory body",
        source: ContributionSource::Memory,
        posture: ContributionPosture::Advisory,
        priority: LOW_PRIORITY,
        cost: ADVISORY_COST,
        deduplication_key: "memory",
    });
    let first = reduce_contributions(
        &[mandatory.clone(), memory.clone(), skill.clone()],
        &binding("request-a"),
        policy(REDUCER_BUDGET),
    )
    .expect("valid contribution set must reduce");
    let second = reduce_contributions(
        &[skill, mandatory, memory],
        &binding("request-a"),
        policy(REDUCER_BUDGET),
    )
    .expect("reordered contribution set must reduce");

    assert_eq!(first, second);
    assert_eq!(
        selected_ids(&first),
        vec!["mandatory-policy", "skill-context"]
    );
    assert_eq!(first.selected_budget, REDUCER_BUDGET);
    assert_eq!(first.omissions.len(), 1);
    assert_eq!(first.omissions[0].class, ContributionOmissionClass::Budget);
}

fn supersession_candidates() -> [Contribution; 2] {
    let old = contribution(TestContribution {
        id: "old-runtime",
        body: "old body",
        source: ContributionSource::Runtime,
        posture: ContributionPosture::Advisory,
        priority: LOW_PRIORITY,
        cost: ADVISORY_COST,
        deduplication_key: "old-runtime",
    });
    let payload = blake3_payload_identity(b"replacement body");
    let replacement = validate_contribution(ContributionDraft {
        contribution_id: "replacement-runtime",
        source: ContributionSource::Runtime,
        trust: ContributionTrust::Trusted,
        posture: ContributionPosture::Advisory,
        priority: MEDIUM_PRIORITY,
        budget_cost: ADVISORY_COST,
        payload_identity: payload.as_str(),
        deduplication_key: "replacement-runtime",
        supersedes: &["old-runtime"],
        conflict_group: None,
        lifetime: ContributionLifetime::CurrentLogicalTurn,
        sensitivity: ContributionSensitivity::Public,
        safe_label: "replacement-runtime",
    })
    .expect("replacement must validate");
    debug_assert_eq!(old.contribution_id().as_str(), "old-runtime");
    debug_assert_eq!(
        replacement.contribution_id().as_str(),
        "replacement-runtime"
    );
    [old, replacement]
}

fn deduplication_candidates() -> [Contribution; 2] {
    [
        contribution(TestContribution {
            id: "duplicate-low",
            body: "duplicate body low",
            source: ContributionSource::ProjectContext,
            posture: ContributionPosture::Advisory,
            priority: LOW_PRIORITY,
            cost: ADVISORY_COST,
            deduplication_key: "project-dedup",
        }),
        contribution(TestContribution {
            id: "duplicate-high",
            body: "duplicate body high",
            source: ContributionSource::ProjectContext,
            posture: ContributionPosture::Advisory,
            priority: HIGH_PRIORITY,
            cost: ADVISORY_COST,
            deduplication_key: "project-dedup",
        }),
    ]
}

#[derive(Clone, Copy)]
struct ConflictFixture<'a> {
    id: &'a str,
    body: &'a str,
    priority: i16,
}

fn conflict_candidate(input: ConflictFixture<'_>) -> Contribution {
    let payload = blake3_payload_identity(input.body.as_bytes());
    validate_contribution(ContributionDraft {
        conflict_group: Some("mode"),
        payload_identity: payload.as_str(),
        contribution_id: input.id,
        source: ContributionSource::Mcp,
        trust: ContributionTrust::Untrusted,
        posture: ContributionPosture::Advisory,
        priority: input.priority,
        budget_cost: ADVISORY_COST,
        deduplication_key: input.id,
        supersedes: &[],
        lifetime: ContributionLifetime::UntilNextProviderResponse,
        sensitivity: ContributionSensitivity::Sensitive,
        safe_label: input.id,
    })
    .expect("conflict candidate must validate")
}

fn arbitration_candidates() -> Vec<Contribution> {
    let mut candidates = Vec::new();
    candidates.extend(supersession_candidates());
    candidates.extend(deduplication_candidates());
    candidates.push(conflict_candidate(ConflictFixture {
        id: "conflict-low",
        body: "conflict low",
        priority: LOW_PRIORITY,
    }));
    candidates.push(conflict_candidate(ConflictFixture {
        id: "conflict-high",
        body: "conflict high",
        priority: HIGH_PRIORITY,
    }));
    candidates
}

#[test]
fn supersession_deduplication_and_conflicts_are_explicit() {
    // r[verify agent_harness_runtime.request_context_contributions.evidence.safe]
    let plan = reduce_contributions(
        arbitration_candidates().as_slice(),
        &binding("request-b"),
        policy(CONTEXT_BUDGET),
    )
    .expect("advisory arbitration must reduce");

    assert!(selected_ids(&plan).contains(&"replacement-runtime"));
    assert!(selected_ids(&plan).contains(&"duplicate-high"));
    assert!(selected_ids(&plan).contains(&"conflict-high"));
    assert!(
        plan.omissions
            .iter()
            .any(|entry| entry.class == ContributionOmissionClass::Superseded)
    );
    assert!(
        plan.omissions
            .iter()
            .any(|entry| entry.class == ContributionOmissionClass::Deduplicated)
    );
    assert!(
        plan.omissions
            .iter()
            .any(|entry| entry.class == ContributionOmissionClass::Conflict)
    );
    let receipt = project_contribution_receipt(&plan, ContributionReceiptSensitiveDraft::default())
        .expect("arbitration receipt must project");
    let replacement = receipt
        .selected
        .iter()
        .find(|entry| entry.contribution_id.as_str() == "replacement-runtime")
        .expect("replacement metadata must remain visible");
    assert_eq!(replacement.deduplication_key, "replacement-runtime");
    assert_eq!(replacement.supersedes[0].as_str(), "old-runtime");
    let conflict = receipt
        .selected
        .iter()
        .find(|entry| entry.contribution_id.as_str() == "conflict-high")
        .expect("conflict metadata must remain visible");
    assert_eq!(conflict.conflict_group.as_deref(), Some("mode"));
}

#[test]
fn reducer_rejects_duplicate_mandatory_conflict_budget_and_drift() {
    let policy_payload = blake3_payload_identity(b"mandatory a");
    let mandatory_a = validate_contribution(ContributionDraft {
        contribution_id: "mandatory-a",
        source: ContributionSource::Prompt,
        trust: ContributionTrust::Trusted,
        posture: ContributionPosture::Mandatory,
        priority: HIGH_PRIORITY,
        budget_cost: MANDATORY_COST,
        payload_identity: policy_payload.as_str(),
        deduplication_key: "mandatory-a",
        supersedes: &[],
        conflict_group: Some("mandatory-mode"),
        lifetime: ContributionLifetime::CurrentRequest,
        sensitivity: ContributionSensitivity::RequestLocal,
        safe_label: "mandatory-a",
    })
    .expect("mandatory fixture must validate");
    let runtime_payload = blake3_payload_identity(b"mandatory b");
    let mandatory_b = validate_contribution(ContributionDraft {
        contribution_id: "mandatory-b",
        source: ContributionSource::Runtime,
        trust: ContributionTrust::Trusted,
        posture: ContributionPosture::Mandatory,
        priority: MEDIUM_PRIORITY,
        budget_cost: MANDATORY_COST,
        payload_identity: runtime_payload.as_str(),
        deduplication_key: "mandatory-b",
        supersedes: &[],
        conflict_group: Some("mandatory-mode"),
        lifetime: ContributionLifetime::CurrentRequest,
        sensitivity: ContributionSensitivity::RequestLocal,
        safe_label: "mandatory-b",
    })
    .expect("mandatory fixture must validate");
    assert_eq!(
        reduce_contributions(
            &[mandatory_a.clone(), mandatory_a.clone()],
            &binding("request-c"),
            policy(CONTEXT_BUDGET),
        ),
        Err(ContributionIssue::DuplicateContribution)
    );
    assert_eq!(
        reduce_contributions(
            &[mandatory_a.clone(), mandatory_b],
            &binding("request-c"),
            policy(CONTEXT_BUDGET),
        ),
        Err(ContributionIssue::MandatoryConflict)
    );
    assert_eq!(
        reduce_contributions(
            &[mandatory_a],
            &binding("request-c"),
            policy(INSUFFICIENT_BUDGET),
        ),
        Err(ContributionIssue::MandatoryBudgetExceeded)
    );

    let plan = reduce_contributions(&[], &binding("request-c"), policy(CONTEXT_BUDGET))
        .expect("empty advisory context remains valid");
    assert_eq!(
        validate_invocation_binding(&plan, &binding("request-d")),
        Err(ContributionIssue::InvocationDrift)
    );
}

#[test]
fn malformed_metadata_and_payload_views_fail_closed() {
    // r[verify agent_harness_runtime.request_context_contributions.contract.invalid]
    let payload = blake3_payload_identity(b"body");
    let mut malformed = ContributionDraft {
        contribution_id: "bad/../identity",
        source: ContributionSource::Runtime,
        trust: ContributionTrust::Trusted,
        posture: ContributionPosture::Advisory,
        priority: LOW_PRIORITY,
        budget_cost: ADVISORY_COST,
        payload_identity: payload.as_str(),
        deduplication_key: "runtime",
        supersedes: &[],
        conflict_group: None,
        lifetime: ContributionLifetime::CurrentRequest,
        sensitivity: ContributionSensitivity::RequestLocal,
        safe_label: "runtime",
    };
    assert_eq!(
        validate_contribution(malformed),
        Err(ContributionIssue::MissingIdentity)
    );
    malformed.contribution_id = "runtime-a";
    malformed.payload_identity = "not-a-hash";
    assert_eq!(
        validate_contribution(malformed),
        Err(ContributionIssue::InvalidPayloadIdentity)
    );

    let valid = contribution(TestContribution {
        id: "runtime-a",
        body: "body",
        source: ContributionSource::Runtime,
        posture: ContributionPosture::Advisory,
        priority: LOW_PRIORITY,
        cost: ADVISORY_COST,
        deduplication_key: "runtime",
    });
    let plan = reduce_contributions(
        std::slice::from_ref(&valid),
        &binding("request-e"),
        policy(CONTEXT_BUDGET),
    )
    .expect("valid contribution must reduce");
    let stale_identity = blake3_payload_identity(b"other");
    let payloads = [ContributionPayloadView {
        contribution_id: valid.contribution_id(),
        payload_identity: &stale_identity,
        body: "body",
    }];
    assert_eq!(
        validate_selected_payloads(&plan, &payloads),
        Err(ContributionIssue::InvalidPayloadIdentity)
    );
}

#[derive(Clone)]
struct FakeSkillBodyService {
    result: Result<LoadedSkillBody, SkillBodyServiceError>,
}

impl SkillBodyService for FakeSkillBodyService {
    fn load_body(
        &mut self,
        _request: &SkillBodyLoadRequest,
    ) -> Result<LoadedSkillBody, SkillBodyServiceError> {
        self.result.clone()
    }
}

fn skill_binding(body: &str) -> SkillSelectionBinding {
    SkillSelectionBinding {
        session: SessionIdentity::new("session-a"),
        turn: LogicalTurnIdentity::new("turn-a"),
        target: ModelTargetIdentity::new("target-a"),
        profile: ModelProfileIdentity::new("profile-a"),
        manifest: SkillManifestIdentity::new("manifest-a"),
        body: blake3_payload_identity(body.as_bytes()),
        selection: SkillSelectionIdentity::new("selection-a"),
        contribution: ContributionIdentity::new("skill-selection-a"),
    }
}

fn skill_policy() -> SkillContributionPolicy<'static> {
    SkillContributionPolicy {
        priority: HIGH_PRIORITY,
        budget_cost: MANDATORY_COST,
        safe_label: "reviewed-skill",
    }
}

#[test]
fn skill_turn_requires_exact_next_provider_response() {
    // r[verify agent_harness_runtime.request_context_contributions.skill_turn.consumed]
    // r[verify agent_harness_runtime.request_context_contributions.skill_turn.result]
    let body = "reviewed skill body";
    let selected = begin_skill_turn(skill_binding(body)).expect("binding must validate");
    let mut service = FakeSkillBodyService {
        result: Ok(LoadedSkillBody {
            body_id: blake3_payload_identity(body.as_bytes()),
            body: body.to_string(),
        }),
    };
    let loaded =
        load_skill_turn(&mut service, &selected, skill_policy()).expect("exact body must load");
    assert_eq!(
        loaded.result.lifetime,
        ContributionLifetime::UntilNextProviderResponse
    );
    assert!(loaded.result.next_response_required);
    assert_eq!(
        admit_skill_turn_completion(&loaded.turn),
        Err(ContributionIssue::SkillResponsePending)
    );

    let plan = reduce_contributions(
        std::slice::from_ref(&loaded.contribution),
        &binding("request-skill"),
        policy(CONTEXT_BUDGET),
    )
    .expect("skill contribution must reduce");
    let first = bind_skill_provider_request(
        &loaded.turn,
        &plan,
        &SkillProviderRequestObservation {
            request_id: ProviderRequestIdentity::new("request-skill"),
            target_id: ModelTargetIdentity::new("target-a"),
            profile_id: ModelProfileIdentity::new("profile-a"),
            contribution_plan_id: plan.plan_id.clone(),
            contribution_selected: true,
            attempt: FIRST_ATTEMPT,
        },
    )
    .expect("first exact provider request must bind");
    let retried = bind_skill_provider_request(
        &first,
        &plan,
        &SkillProviderRequestObservation {
            request_id: ProviderRequestIdentity::new("request-skill"),
            target_id: ModelTargetIdentity::new("target-a"),
            profile_id: ModelProfileIdentity::new("profile-a"),
            contribution_plan_id: plan.plan_id.clone(),
            contribution_selected: true,
            attempt: SECOND_ATTEMPT,
        },
    )
    .expect("same-request retry must retain exact binding");
    for outcome in [
        ProviderTerminalOutcome::Completed,
        ProviderTerminalOutcome::Failed,
        ProviderTerminalOutcome::Cancelled,
    ] {
        let terminal = observe_skill_provider_response(
            &retried,
            &SkillProviderResponseObservation {
                request_id: ProviderRequestIdentity::new("request-skill"),
                response_id: ProviderResponseIdentity::new("response-skill"),
                outcome,
            },
        )
        .expect("correlated terminal response must complete the barrier");
        assert_eq!(admit_skill_turn_completion(&terminal), Ok(()));
    }
}

#[test]
fn skill_turn_rejects_stale_body_target_and_service_failure() {
    // r[verify agent_harness_runtime.request_context_contributions.skill_turn.stale]
    let selected = begin_skill_turn(skill_binding("current body")).expect("binding must validate");
    let mut stale_service = FakeSkillBodyService {
        result: Ok(LoadedSkillBody {
            body_id: blake3_payload_identity(b"stale body"),
            body: "stale body".to_string(),
        }),
    };
    assert_eq!(
        load_skill_turn(&mut stale_service, &selected, skill_policy()),
        Err(ContributionIssue::StaleSkillBody)
    );
    let mut failed_service = FakeSkillBodyService {
        result: Err(SkillBodyServiceError::Unavailable),
    };
    assert_eq!(
        load_skill_turn(&mut failed_service, &selected, skill_policy()),
        Err(ContributionIssue::SkillBodyServiceFailed)
    );

    let body = "current body";
    let loaded_body = LoadedSkillBody {
        body_id: blake3_payload_identity(body.as_bytes()),
        body: body.to_string(),
    };
    let (loaded_turn, skill_contribution, _) =
        admit_loaded_skill_body(&selected, &loaded_body, skill_policy())
            .expect("exact body must load");
    let plan = reduce_contributions(
        &[skill_contribution],
        &binding("request-skill"),
        policy(CONTEXT_BUDGET),
    )
    .expect("skill contribution must reduce");
    assert_eq!(
        bind_skill_provider_request(
            &loaded_turn,
            &plan,
            &SkillProviderRequestObservation {
                request_id: ProviderRequestIdentity::new("request-skill"),
                target_id: ModelTargetIdentity::new("target-b"),
                profile_id: ModelProfileIdentity::new("profile-a"),
                contribution_plan_id: plan.plan_id.clone(),
                contribution_selected: true,
                attempt: FIRST_ATTEMPT,
            },
        ),
        Err(ContributionIssue::InvocationDrift)
    );
    assert_eq!(
        admit_skill_turn_completion(&loaded_turn),
        Err(ContributionIssue::SkillResponsePending)
    );
    assert_eq!(
        admit_skill_turn_completion(
            &expire_unconsumed_skill_turn(&loaded_turn)
                .expect("pre-dispatch expiration is explicit")
        ),
        Ok(())
    );
}

fn sensitive_contribution() -> Contribution {
    contribution(TestContribution {
        id: "sensitive-skill",
        body: "secret body",
        source: ContributionSource::Skill,
        posture: ContributionPosture::Mandatory,
        priority: HIGH_PRIORITY,
        cost: MANDATORY_COST,
        deduplication_key: "sensitive-skill",
    })
}

fn sensitive_plan() -> ContributionPlan {
    reduce_contributions(
        &[sensitive_contribution()],
        &binding("request-receipt"),
        policy(CONTEXT_BUDGET),
    )
    .expect("valid contribution must reduce")
}

#[test]
fn retention_rejects_payloads() {
    // r[verify agent_harness_runtime.request_context_contributions.retention.denied]
    let contribution = sensitive_contribution();
    for sink in [
        RetentionSink::Conversation,
        RetentionSink::Compaction,
        RetentionSink::Memory,
        RetentionSink::Quality,
        RetentionSink::Receipt,
    ] {
        assert_eq!(
            admit_payload_retention(&contribution, sink),
            Err(ContributionIssue::RetentionDenied)
        );
    }
}

#[test]
fn receipt_projection_is_metadata_only() {
    let receipt = project_contribution_receipt(
        &sensitive_plan(),
        ContributionReceiptSensitiveDraft::default(),
    )
    .expect("metadata-only receipt must project");

    assert_eq!(validate_contribution_receipt(&receipt), Ok(()));
    assert!(!format!("{receipt:?}").contains("secret body"));
}

#[test]
fn receipt_rejects_sensitive_fields_and_overclaims() {
    // r[verify agent_harness_runtime.request_context_contributions.evidence.rejects]
    let plan = sensitive_plan();
    let receipt = project_contribution_receipt(&plan, ContributionReceiptSensitiveDraft::default())
        .expect("metadata-only receipt must project");
    let leak_values = [
        ContributionReceiptSensitiveDraft {
            prompt_body: Some("raw prompt"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            skill_body: Some("skill body"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            mcp_instructions: Some("server instructions"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            memory_record: Some("memory record"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            path: Some("private path"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            credential: Some("credential"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            provider_data: Some("provider data"),
            ..ContributionReceiptSensitiveDraft::default()
        },
        ContributionReceiptSensitiveDraft {
            hidden_directive: Some("hidden directive"),
            ..ContributionReceiptSensitiveDraft::default()
        },
    ];
    for leak in leak_values {
        assert_eq!(
            project_contribution_receipt(&plan, leak),
            Err(ContributionIssue::ReceiptLeak)
        );
    }

    let mut overclaim = receipt.clone();
    overclaim.claims.push("provider-delivery".to_string());
    assert_eq!(
        validate_contribution_receipt(&overclaim),
        Err(ContributionIssue::ReceiptOverclaim)
    );
    let mut weakened = receipt;
    weakened.non_claims.clear();
    assert_eq!(
        validate_contribution_receipt(&weakened),
        Err(ContributionIssue::MissingNonClaim)
    );
}
