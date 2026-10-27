#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::function_length,
        tigerstyle::renamed_imports,
        reason = "dense table-driven stage fixtures keep the established kit alias and grouped outcome assertions"
    )
)]

use super::*;
use crate::tool_executor_kit as kit;

const TOOL_NAME: &str = "lookup";
const TOOL_SCHEMA: &str = "schema.lookup.v1";
const TOOL_CAPABILITY: &str = "capability.lookup";
const EXECUTOR_BINDING: &str = "executor.lookup";
const ARGUMENT_NAME: &str = "key";
const ARGUMENT_VALUE: &str = "alpha";
const OTHER_ARGUMENT_VALUE: &str = "beta";
const OUTPUT_VISIBLE_CHARS: u32 = 64;
const OUTPUT_BYTES: u64 = 256;
const RAW_OUTPUT_BYTES: u64 = 12;
const RAW_OUTPUT_CHARS: u64 = 12;
const NARROWED_OUTPUT_BYTES: u64 = 8;
const NARROWED_OUTPUT_CHARS: u64 = 8;
const WIDENED_OUTPUT_CHARS: u32 = OUTPUT_VISIBLE_CHARS.saturating_add(1);

struct Fixture {
    catalog: kit::ToolCatalog,
    capabilities: crate::RuntimeCapabilityRegistry,
    executors: Vec<kit::ToolExecutorBinding>,
    request: animus_adapters::ToolRequest,
}

impl Fixture {
    fn new() -> Self {
        Self {
            catalog: catalog(),
            capabilities: capabilities(),
            executors: vec![
                kit::ToolExecutorBinding::new(EXECUTOR_BINDING, kit::ToolExecutorReadiness::Ready)
                    .expect("executor"),
            ],
            request: request(ARGUMENT_VALUE),
        }
    }

    fn tool_plan(&self) -> kit::ToolExecutionPlan {
        kit::plan_tool_execution(kit::ToolExecutionPlanRequest {
            request: &self.request,
            catalog: &self.catalog,
            capabilities: &self.capabilities,
            executors: self.executors.as_slice(),
            admission: kit::ToolExecutorAdmission::admitted(),
        })
    }

    fn stage_plan(&self, handlers: &[ToolExecutionStageHandler]) -> ToolExecutionStagePlan {
        let tool_plan = self.tool_plan();
        plan_tool_execution_stages(ToolExecutionStagePlanDraft {
            phase_order: &TOOL_EXECUTION_STAGE_PHASES,
            handlers,
            tool_plan: &tool_plan,
            request: &self.request,
            authority_seal_identity: hash("authority"),
            commit_owner_identity: hash("commit-owner"),
        })
        .expect("stage plan")
    }
}

#[test]
fn zero_handler_plan_keeps_every_fixed_barrier_and_commits_before_observation() {
    // r[verify agent_harness_runtime.tool_execution_stages.plan]
    // r[verify agent_harness_runtime.tool_execution_stages.commit]
    let fixture = Fixture::new();
    let plan = fixture.stage_plan(&[]);
    let tool_plan = fixture.tool_plan();
    let state = complete_to_final_validation(&plan, &tool_plan);

    assert_eq!(state.status(), ToolExecutionStageStatus::AwaitingCommit);
    assert_eq!(
        state.next_phase(),
        Some(ToolExecutionStagePhase::DurableCommit)
    );
    let premature = exact_tool_execution_stage_candidate(
        &plan,
        &state,
        ToolExecutionStagePhase::PostCommitObservation,
        ToolExecutionStageDisposition::Admitted,
    );
    assert_eq!(
        apply_tool_post_commit_candidate(&plan, &state, &premature)
            .expect_err("observation requires commit")
            .issue(),
        ToolExecutionStageIssue::CommitRequired
    );

    let committed = observe_tool_stage_durable_commit(&plan, &state, &commit_observation(&plan))
        .expect("durable commit");
    let post = exact_tool_execution_stage_candidate(
        &plan,
        &committed,
        ToolExecutionStagePhase::PostCommitObservation,
        ToolExecutionStageDisposition::Admitted,
    );
    let observed = apply_tool_post_commit_candidate(&plan, &committed, &post).expect("post commit");
    let receipt = project_tool_execution_stage_receipt(
        &plan,
        &observed,
        ToolExecutionStageSensitiveDraft::default(),
    )
    .expect("safe receipt");
    assert_eq!(observed.status(), ToolExecutionStageStatus::Observed);
    assert_eq!(receipt.handler_count(), 0);
    assert_eq!(receipt.stage_plan_identity(), plan.plan_identity());
    assert_eq!(receipt.request_identity(), plan.request_identity());
    assert_eq!(
        receipt.authority_seal_identity(),
        plan.authority_seal_identity()
    );
    assert_eq!(
        receipt.commit_owner_identity(),
        plan.commit_owner_identity()
    );
    assert!(receipt.result_identity().is_some());
    assert!(receipt.executor_receipt_identity().is_some());
    assert!(receipt.durable_head_identity().is_some());
    assert!(receipt.durable_commit_identity().is_some());
    assert_eq!(
        receipt.redaction_marker(),
        TOOL_EXECUTION_STAGE_REDACTION_MARKER
    );
    assert_eq!(receipt.status(), ToolExecutionStageStatus::Observed);
}

#[test]
fn ordered_handlers_are_bound_and_output_can_only_narrow() {
    // r[verify agent_harness_runtime.tool_execution_stages.binding]
    // r[verify agent_harness_runtime.tool_execution_stages.output]
    let handlers = handlers();
    let fixture = Fixture::new();
    let plan = fixture.stage_plan(handlers.as_slice());
    assert_eq!(plan.handlers(), handlers.as_slice());
    assert_ne!(
        tool_execution_stage_handler_set_identity(&plan, ToolExecutionStagePhase::PreAdmission),
        tool_execution_stage_handler_set_identity(&plan, ToolExecutionStagePhase::PreExecution)
    );
    let tool_plan = fixture.tool_plan();
    let mut state = start_tool_execution_stages(&plan);
    state = apply_tool_pre_admission_candidate(
        &plan,
        &state,
        &exact_tool_execution_stage_candidate(
            &plan,
            &state,
            ToolExecutionStagePhase::PreAdmission,
            ToolExecutionStageDisposition::Admitted,
        ),
    )
    .expect("pre admission");
    state = apply_tool_final_admission(&plan, &state, &tool_plan).expect("final admission");
    state = apply_tool_pre_execution_candidate(
        &plan,
        &state,
        &exact_tool_execution_stage_candidate(
            &plan,
            &state,
            ToolExecutionStagePhase::PreExecution,
            ToolExecutionStageDisposition::Admitted,
        ),
    )
    .expect("pre execution");
    state =
        observe_tool_stage_execution(&plan, &state, &raw_observation(&plan)).expect("execution");
    let candidate = exact_tool_execution_stage_candidate(
        &plan,
        &state,
        ToolExecutionStagePhase::OutputNarrowing,
        ToolExecutionStageDisposition::Admitted,
    );
    let narrowed = output_observation(candidate);
    let state = apply_tool_output_narrowing(&plan, &state, &narrowed).expect("narrowing");
    assert_eq!(
        state.next_phase(),
        Some(ToolExecutionStagePhase::FinalValidation)
    );
}

#[test]
fn reordered_phases_duplicate_handlers_and_fixed_phase_handlers_fail_closed() {
    // r[verify agent_harness_runtime.tool_execution_stages.verification]
    let fixture = Fixture::new();
    let tool_plan = fixture.tool_plan();
    let mut reordered = TOOL_EXECUTION_STAGE_PHASES;
    reordered.swap(0, 1);
    assert_eq!(
        plan_tool_execution_stages(ToolExecutionStagePlanDraft {
            phase_order: &reordered,
            handlers: &[],
            tool_plan: &tool_plan,
            request: &fixture.request,
            authority_seal_identity: hash("authority"),
            commit_owner_identity: hash("commit-owner"),
        })
        .expect_err("phase reorder")
        .issue(),
        ToolExecutionStageIssue::InvalidPhaseOrder
    );
    assert_eq!(
        validate_tool_execution_stage_handler(ToolExecutionStageHandlerDraft {
            phase: ToolExecutionStagePhase::Execution,
            handler_identity: hash("handler"),
            cohort_identity: hash("cohort"),
        })
        .expect_err("fixed handler")
        .issue(),
        ToolExecutionStageIssue::InvalidHandlerPhase
    );
    let handler = handler(ToolExecutionStagePhase::PreAdmission, "duplicate");
    assert_eq!(
        plan_tool_execution_stages(ToolExecutionStagePlanDraft {
            phase_order: &TOOL_EXECUTION_STAGE_PHASES,
            handlers: &[handler.clone(), handler],
            tool_plan: &tool_plan,
            request: &fixture.request,
            authority_seal_identity: hash("authority"),
            commit_owner_identity: hash("commit-owner"),
        })
        .expect_err("duplicate")
        .issue(),
        ToolExecutionStageIssue::DuplicateHandler
    );
}

#[test]
fn argument_handler_authority_and_capability_substitution_fail_before_execution() {
    // r[verify agent_harness_runtime.tool_execution_stages.binding]
    let fixture = Fixture::new();
    let tool_plan = fixture.tool_plan();
    let substituted_request = request(OTHER_ARGUMENT_VALUE);
    assert_eq!(
        plan_tool_execution_stages(ToolExecutionStagePlanDraft {
            phase_order: &TOOL_EXECUTION_STAGE_PHASES,
            handlers: &[],
            tool_plan: &tool_plan,
            request: &substituted_request,
            authority_seal_identity: hash("authority"),
            commit_owner_identity: hash("commit-owner"),
        })
        .expect_err("arguments changed")
        .issue(),
        ToolExecutionStageIssue::PlanBindingMismatch
    );

    let plan = fixture.stage_plan(&[]);
    let state = start_tool_execution_stages(&plan);
    let extra_capabilities = vec![
        String::from(TOOL_CAPABILITY),
        String::from("capability.added"),
    ];
    let candidate = candidate_with(
        &plan,
        &state,
        CandidateChanges {
            authority: Some(hash("other-authority")),
            capabilities: Some(extra_capabilities),
            ..CandidateChanges::default()
        },
    );
    assert!(matches!(
        apply_tool_pre_admission_candidate(&plan, &state, &candidate)
            .expect_err("authority or capability changed")
            .issue(),
        ToolExecutionStageIssue::AuthorityWidening | ToolExecutionStageIssue::CapabilityWidening
    ));
}

#[test]
fn sticky_denial_bound_and_mutation_widening_never_reach_executor() {
    // r[verify agent_harness_runtime.tool_execution_stages.monotonic]
    let fixture = Fixture::new();
    let plan = fixture.stage_plan(&[]);
    let state = start_tool_execution_stages(&plan);
    let denial = exact_tool_execution_stage_candidate(
        &plan,
        &state,
        ToolExecutionStagePhase::PreAdmission,
        ToolExecutionStageDisposition::Denied,
    );
    let denied = apply_tool_pre_admission_candidate(&plan, &state, &denial).expect("sticky denial");
    let reversal = exact_tool_execution_stage_candidate(
        &plan,
        &denied,
        ToolExecutionStagePhase::PreAdmission,
        ToolExecutionStageDisposition::Admitted,
    );
    assert_eq!(
        apply_tool_pre_admission_candidate(&plan, &denied, &reversal)
            .expect_err("denial reversal")
            .issue(),
        ToolExecutionStageIssue::StickyDenial
    );

    let bound_widening = candidate_with(
        &plan,
        &state,
        CandidateChanges {
            max_visible_chars: Some(WIDENED_OUTPUT_CHARS),
            ..CandidateChanges::default()
        },
    );
    assert_eq!(
        apply_tool_pre_admission_candidate(&plan, &state, &bound_widening)
            .expect_err("bound widening")
            .issue(),
        ToolExecutionStageIssue::BoundWidening
    );
    let mutation_widening = candidate_with(
        &plan,
        &state,
        CandidateChanges {
            mutation_posture: Some(kit::ToolMutationPosture::DeclaredTargets),
            ..CandidateChanges::default()
        },
    );
    assert_eq!(
        apply_tool_pre_admission_candidate(&plan, &state, &mutation_widening)
            .expect_err("mutation widening")
            .issue(),
        ToolExecutionStageIssue::MutationWidening
    );
}

#[test]
fn output_growth_failure_to_success_and_redaction_removal_are_rejected() {
    // r[verify agent_harness_runtime.tool_execution_stages.output]
    let fixture = Fixture::new();
    let plan = fixture.stage_plan(&[]);
    let tool_plan = fixture.tool_plan();
    let state = to_output_phase(
        &plan,
        &tool_plan,
        ToolExecutionStageOutcomeClass::Failed,
        true,
    );
    let changed_class = candidate_with_phase(
        &plan,
        &state,
        ToolExecutionStagePhase::OutputNarrowing,
        CandidateChanges {
            outcome_class: Some(ToolExecutionStageOutcomeClass::Succeeded),
            required_redaction: Some(false),
            ..CandidateChanges::default()
        },
    );
    let observation = output_observation(changed_class);
    assert!(matches!(
        apply_tool_output_narrowing(&plan, &state, &observation)
            .expect_err("class and redaction widening")
            .issue(),
        ToolExecutionStageIssue::OutcomeWidening | ToolExecutionStageIssue::RedactionRemoved
    ));

    let state = to_output_phase(
        &plan,
        &tool_plan,
        ToolExecutionStageOutcomeClass::Succeeded,
        false,
    );
    let candidate = exact_tool_execution_stage_candidate(
        &plan,
        &state,
        ToolExecutionStagePhase::OutputNarrowing,
        ToolExecutionStageDisposition::Admitted,
    );
    let mut growth = output_observation(candidate);
    growth.output_byte_count = RAW_OUTPUT_BYTES.saturating_add(1);
    assert_eq!(
        apply_tool_output_narrowing(&plan, &state, &growth)
            .expect_err("output grew")
            .issue(),
        ToolExecutionStageIssue::BoundWidening
    );
}

#[test]
fn commit_mismatch_and_public_receipt_leaks_fail_closed() {
    // r[verify agent_harness_runtime.tool_execution_stages.commit]
    // r[verify agent_harness_runtime.tool_execution_stages.verification]
    let fixture = Fixture::new();
    let plan = fixture.stage_plan(&[]);
    let state = complete_to_final_validation(&plan, &fixture.tool_plan());
    let mut mismatched_commit = commit_observation(&plan);
    mismatched_commit.result_identity = hash("stale-result").to_string();
    assert_eq!(
        observe_tool_stage_durable_commit(&plan, &state, &mismatched_commit)
            .expect_err("stale result")
            .issue(),
        ToolExecutionStageIssue::CommitMismatch
    );

    let denied = apply_tool_pre_admission_candidate(
        &plan,
        &start_tool_execution_stages(&plan),
        &exact_tool_execution_stage_candidate(
            &plan,
            &start_tool_execution_stages(&plan),
            ToolExecutionStagePhase::PreAdmission,
            ToolExecutionStageDisposition::Denied,
        ),
    )
    .expect("denied");
    assert_eq!(
        project_tool_execution_stage_receipt(
            &plan,
            &denied,
            ToolExecutionStageSensitiveDraft {
                raw_output: Some("private output"),
                ..ToolExecutionStageSensitiveDraft::default()
            },
        )
        .expect_err("receipt leak")
        .issue(),
        ToolExecutionStageIssue::UnsafeEvidence
    );
}

fn complete_to_final_validation(
    plan: &ToolExecutionStagePlan,
    tool_plan: &kit::ToolExecutionPlan,
) -> ToolExecutionStageState {
    let state = to_output_phase(
        plan,
        tool_plan,
        ToolExecutionStageOutcomeClass::Succeeded,
        false,
    );
    let candidate = exact_tool_execution_stage_candidate(
        plan,
        &state,
        ToolExecutionStagePhase::OutputNarrowing,
        ToolExecutionStageDisposition::Admitted,
    );
    let narrowed = apply_tool_output_narrowing(plan, &state, &output_observation(candidate))
        .expect("output narrowing");
    observe_tool_final_validation(
        plan,
        &narrowed,
        &ToolExecutionStageFinalObservation {
            stage_plan_identity: String::from(plan.plan_identity()),
            narrowed_output_identity: hash("narrowed").to_string(),
            result_identity: hash("result").to_string(),
            executor_receipt_identity: hash("executor-receipt").to_string(),
            outcome_class: ToolExecutionStageOutcomeClass::Succeeded,
            truncated: false,
        },
    )
    .expect("final validation")
}

fn to_output_phase(
    plan: &ToolExecutionStagePlan,
    tool_plan: &kit::ToolExecutionPlan,
    outcome_class: ToolExecutionStageOutcomeClass,
    required_redaction: bool,
) -> ToolExecutionStageState {
    let mut state = start_tool_execution_stages(plan);
    state = apply_tool_pre_admission_candidate(
        plan,
        &state,
        &exact_tool_execution_stage_candidate(
            plan,
            &state,
            ToolExecutionStagePhase::PreAdmission,
            ToolExecutionStageDisposition::Admitted,
        ),
    )
    .expect("pre admission");
    state = apply_tool_final_admission(plan, &state, tool_plan).expect("final admission");
    state = apply_tool_pre_execution_candidate(
        plan,
        &state,
        &exact_tool_execution_stage_candidate(
            plan,
            &state,
            ToolExecutionStagePhase::PreExecution,
            ToolExecutionStageDisposition::Admitted,
        ),
    )
    .expect("pre execution");
    let mut observation = raw_observation(plan);
    observation.outcome_class = outcome_class;
    observation.required_redaction = required_redaction;
    observe_tool_stage_execution(plan, &state, &observation).expect("execution")
}

fn raw_observation(plan: &ToolExecutionStagePlan) -> ToolExecutionStageRawObservation {
    ToolExecutionStageRawObservation {
        stage_plan_identity: String::from(plan.plan_identity()),
        request_identity: String::from(plan.request_identity()),
        output_identity: hash("raw-output").to_string(),
        outcome_class: ToolExecutionStageOutcomeClass::Succeeded,
        output_byte_count: RAW_OUTPUT_BYTES,
        output_char_count: RAW_OUTPUT_CHARS,
        required_redaction: false,
    }
}

fn output_observation(
    candidate: ToolExecutionStageCandidate,
) -> ToolExecutionStageOutputObservation {
    ToolExecutionStageOutputObservation {
        candidate,
        input_output_identity: hash("raw-output").to_string(),
        narrowed_output_identity: hash("narrowed").to_string(),
        output_byte_count: NARROWED_OUTPUT_BYTES,
        output_char_count: NARROWED_OUTPUT_CHARS,
        truncated: false,
    }
}

fn commit_observation(plan: &ToolExecutionStagePlan) -> ToolExecutionStageCommitObservation {
    ToolExecutionStageCommitObservation {
        stage_plan_identity: String::from(plan.plan_identity()),
        result_identity: hash("result").to_string(),
        commit_owner_identity: String::from(plan.commit_owner_identity()),
        durable_head_identity: hash("durable-head").to_string(),
        complete: true,
        synchronized: true,
    }
}

#[derive(Default)]
struct CandidateChanges {
    authority: Option<&'static str>,
    capabilities: Option<Vec<String>>,
    mutation_posture: Option<kit::ToolMutationPosture>,
    max_visible_chars: Option<u32>,
    required_redaction: Option<bool>,
    outcome_class: Option<ToolExecutionStageOutcomeClass>,
}

fn candidate_with(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    changes: CandidateChanges,
) -> ToolExecutionStageCandidate {
    candidate_with_phase(plan, state, ToolExecutionStagePhase::PreAdmission, changes)
}

fn candidate_with_phase(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    phase: ToolExecutionStagePhase,
    changes: CandidateChanges,
) -> ToolExecutionStageCandidate {
    let handler_set_identity = tool_execution_stage_handler_set_identity(plan, phase);
    let capabilities = changes
        .capabilities
        .unwrap_or_else(|| plan.capability_labels().to_vec());
    build_tool_execution_stage_candidate(ToolExecutionStageCandidateDraft {
        phase,
        stage_plan_identity: plan.plan_identity(),
        handler_set_identity: handler_set_identity.as_str(),
        request_identity: plan.request_identity(),
        schema_identity: plan.schema_identity(),
        executor_binding: plan.executor_binding(),
        authority_seal_identity: changes
            .authority
            .unwrap_or_else(|| plan.authority_seal_identity()),
        capability_labels: capabilities.as_slice(),
        mutation_targets: plan.mutation_targets(),
        mutation_posture: changes
            .mutation_posture
            .map_or_else(|| plan.mutation_posture(), Some),
        max_visible_chars: changes.max_visible_chars.map_or_else(
            || {
                plan.output_policy()
                    .map(kit::ToolExecutorOutputPolicy::max_visible_chars)
            },
            Some,
        ),
        max_output_bytes: plan
            .output_policy()
            .map(kit::ToolExecutorOutputPolicy::max_output_bytes),
        required_redaction: changes
            .required_redaction
            .unwrap_or_else(|| state.required_redaction()),
        disposition: ToolExecutionStageDisposition::Admitted,
        outcome_class: changes
            .outcome_class
            .unwrap_or_else(|| state.outcome_class()),
    })
    .expect("candidate")
}

fn handlers() -> Vec<ToolExecutionStageHandler> {
    [
        (ToolExecutionStagePhase::PreAdmission, "pre-one"),
        (ToolExecutionStagePhase::PreExecution, "wrapper-one"),
        (ToolExecutionStagePhase::OutputNarrowing, "redact-one"),
        (
            ToolExecutionStagePhase::PostCommitObservation,
            "observe-one",
        ),
    ]
    .into_iter()
    .map(|(phase, label)| handler(phase, label))
    .collect()
}

fn handler(phase: ToolExecutionStagePhase, label: &str) -> ToolExecutionStageHandler {
    validate_tool_execution_stage_handler(ToolExecutionStageHandlerDraft {
        phase,
        handler_identity: hash(label),
        cohort_identity: hash("cohort"),
    })
    .expect("handler")
}

fn catalog() -> kit::ToolCatalog {
    let metadata =
        animus_adapters::tool::Definition::validate(&animus_adapters::tool::DefinitionDraft {
            name: Some(TOOL_NAME),
            label: Some("Lookup"),
            description: Some("Reads one deterministic fixture."),
            parameter_schema_identity: Some(TOOL_SCHEMA),
            prompt_snippet: Some("Use lookup for fixture reads."),
            prompt_guidelines: &["Supply one key."],
            source_identity: Some("tool-stage-tests"),
            provenance: Some("maintained-test"),
            capability_labels: &[TOOL_CAPABILITY],
            result_metadata_shape: Some("bounded-text.v1"),
            status: animus_adapters::tool::DefinitionStatus::Active,
        })
        .expect("metadata");
    let output_policy =
        kit::ToolExecutorOutputPolicy::new(OUTPUT_VISIBLE_CHARS, OUTPUT_BYTES, false)
            .expect("output policy");
    let definition = kit::ToolDefinition::from_draft(kit::ToolDefinitionDraft {
        metadata,
        executor_binding: EXECUTOR_BINDING,
        argument_names: &[ARGUMENT_NAME],
        mutation_posture: kit::ToolMutationPosture::ReadOnly,
        output_policy,
        approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved,
    })
    .expect("definition");
    kit::ToolCatalog::new(vec![definition]).expect("catalog")
}

fn capabilities() -> crate::RuntimeCapabilityRegistry {
    crate::RuntimeCapabilityRegistry::from_drafts(&[crate::RuntimeCapabilityDescriptorDraft {
        label: TOOL_CAPABILITY,
        service_kind: "tool",
        scope: None,
    }])
    .expect("capabilities")
}

fn request(value: &str) -> animus_adapters::ToolRequest {
    let arguments = animus_adapters::tool::PreparedArguments::prepare(
        &animus_adapters::tool::ArgumentPreparationRequest {
            current_schema_identity: TOOL_SCHEMA,
            allowed_field_names: &[ARGUMENT_NAME],
            compatibility_rules: &[],
            fields: &[animus_adapters::tool::ArgumentFieldDraft {
                name: Some(ARGUMENT_NAME),
                value: Some(value),
            }],
        },
    )
    .expect("arguments");
    animus_adapters::ToolRequest::from_draft(&animus_adapters::tool::RequestDraft {
        name: Some(TOOL_NAME),
        arguments: Some(&arguments),
        correlation_id: Some("tool-call-1"),
        output_policy: animus_adapters::tool::OutputPolicy {
            visible_char_budget: OUTPUT_VISIBLE_CHARS,
        },
        mutation_targets: &[],
        required_capability_labels: &[TOOL_CAPABILITY],
        approval_posture: animus_adapters::tool::ApprovalPosture::PreApproved.as_str(),
    })
    .expect("request")
}

fn hash(label: &str) -> &'static str {
    Box::leak(
        blake3::hash(label.as_bytes())
            .to_hex()
            .to_string()
            .into_boxed_str(),
    )
}
