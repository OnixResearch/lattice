use alloc::vec::Vec;

use super::*;

const SOURCE_IDENTITY: &str = "b3:0000000000000000000000000000000000000000000000000000000000000000";
const PROFILE_ID: &str = "profile:native-eval-v1";
const POLICY_TASKS: u32 = 32;
const POLICY_RUNS: u32 = 16;
const POLICY_REWARDS: u32 = 8;
const POLICY_ARTIFACTS: u32 = 4;
const PAIR: u32 = 2;
const RECIPE_KIND_COUNT: usize = 5;
const LAZY_SEED: u64 = 7;
const ONE_VALUE: f64 = 1.0;
const OUT_OF_DOMAIN_VALUE: f64 = 2.0;

fn policy_with(
    recipes: &[RecipeKind],
    missing_reward: MissingRewardPolicy,
    mismatch: ResumeMismatchPolicy,
) -> EvaluationPolicy {
    validate_evaluation_policy(
        &EvaluationPolicyDraft {
            schema: EVALUATION_POLICY_SCHEMA,
            source_identity: SOURCE_IDENTITY,
            profile_id: PROFILE_ID,
            max_tasks: POLICY_TASKS,
            max_runs: POLICY_RUNS,
            max_reward_dimensions: POLICY_REWARDS,
            max_artifacts: POLICY_ARTIFACTS,
            network_default: EvaluationNetworkPosture::None,
            secret_values_allowed: false,
            transcript_bodies_allowed: false,
            missing_reward,
            resume_mismatch: mismatch,
            supported_recipes: recipes,
            evidence_redaction: "metadata_only",
        },
        SOURCE_IDENTITY,
    )
    .expect("valid policy")
}

fn all_recipes() -> [RecipeKind; RECIPE_KIND_COUNT] {
    [
        RecipeKind::SingleAgent,
        RecipeKind::BestOfN,
        RecipeKind::UserSimulation,
        RecipeKind::IsolatedAgenticJudge,
        RecipeKind::SharedAdvisoryJudge,
    ]
}

fn bindings() -> RecipeBindings<'static> {
    RecipeBindings {
        task_identity: "task:one",
        role_identity: "role:solver",
        model_identity: "model:one",
        harness_identity: "harness:one",
        tool_identity: "tools:one",
        environment_identity: "environment:solver",
        verifier_identity: "role:verifier",
        verifier_environment_identity: "environment:verifier",
        rubric_identity: "rubric:one",
        resource_identity: "resources:one",
        cancellation_identity: "cancel:one",
        deadline_identity: "deadline:one",
        redaction_identity: "redaction:one",
        artifact_manifest_identity: "artifacts:one",
    }
}

fn task(identity: &'static str) -> TaskDescriptor<'static> {
    TaskDescriptor {
        task_identity: identity,
        data_identity: "data:bounded",
        behavior_identity: "behavior:v1",
        scorer_identity: "scorer:v1",
        policy_identity: "task-policy:v1",
        version: "v1",
    }
}

#[test]
fn exact_and_lazy_tasksets_are_deterministic_and_bounded() {
    let recipes = all_recipes();
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    let tasks = [task("task:one"), task("task:two")];
    let members = ["task:one", "task:two"];
    let exact = plan_taskset(
        &TasksetDescriptor {
            taskset_identity: "taskset:exact",
            version: "v1",
            ordering: TaskOrdering::Stable,
            source: TasksetSource::Exact { members: &members },
        },
        &tasks,
        &policy,
    )
    .expect("exact taskset");
    let lazy = plan_taskset(
        &TasksetDescriptor {
            taskset_identity: "taskset:lazy",
            version: "v1",
            ordering: TaskOrdering::Stable,
            source: TasksetSource::Lazy {
                generator_identity: "generator:one",
                generator_version: "v1",
                seed: LAZY_SEED,
                finiteness: TasksetFiniteness::Infinite,
                selection_bound: 1,
            },
        },
        &tasks,
        &policy,
    )
    .expect("bounded lazy taskset");
    assert_eq!(
        exact.tasks.len(),
        usize::try_from(PAIR).expect("pair fits usize")
    );
    assert_eq!(lazy.tasks.len(), 1);
    assert_ne!(exact.identity, lazy.identity);
}

#[test]
fn environment_and_every_recipe_have_explicit_order_and_authority_posture() {
    let environment = plan_environment(&EnvironmentPlanRequest {
        solver_environment_identity: "environment:solver",
        verifier_environment_identity: Some("environment:verifier"),
        resource_identity: "resources:one",
        network_identity: "network:none",
        require_verifier_isolation: true,
    })
    .expect("environment plan");
    assert!(environment.verifier_isolated);

    let recipes = all_recipes();
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    for kind in recipes {
        let request = RecipeRequest {
            kind,
            bindings: bindings(),
            candidate_count: PAIR,
            artifact_count: 1,
            simulated_user_trainable: false,
            simulated_user_assistant_tools: false,
        };
        let plan = plan_recipe(&request, &policy).expect("recipe plan");
        assert!(!plan.seats.is_empty());
        assert!(!plan.actions.is_empty());
        for (index, action) in plan.actions.iter().enumerate() {
            assert_eq!(
                usize::try_from(action.order).expect("action order fits usize"),
                index
            );
        }
        if kind == RecipeKind::SharedAdvisoryJudge {
            assert!(plan.advisory);
            assert!(!plan.verifier_isolated);
        }
        if kind == RecipeKind::IsolatedAgenticJudge {
            assert!(plan.verifier_isolated);
            assert_eq!(plan.actions[1].kind, RecipeActionKind::HandoffArtifacts);
        }
        if kind == RecipeKind::BestOfN {
            let observations = plan
                .seats
                .iter()
                .map(|seat| PassAtNObservation {
                    run_identity: seat.run_identity.as_str(),
                    passed: true,
                })
                .collect::<Vec<_>>();
            let reduction = reduce_pass_at_n(&plan, &observations, 1).expect("pass-at-N");
            assert!(reduction.passed);
            assert_eq!(reduction.passed_count, PAIR);
        }
    }
}

#[test]
fn episode_keeps_the_ledger_as_trace_authority() {
    let recipes = [RecipeKind::SingleAgent];
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    let plan = plan_recipe(
        &RecipeRequest {
            kind: RecipeKind::SingleAgent,
            bindings: bindings(),
            candidate_count: 1,
            artifact_count: 0,
            simulated_user_trainable: false,
            simulated_user_assistant_tools: false,
        },
        &policy,
    )
    .expect("single plan");
    let seat = &plan.seats[0];
    let observations = [EpisodeRunObservation {
        run_identity: &seat.run_identity,
        seat_identity: &seat.seat_identity,
        ledger: LedgerTraceReference {
            session: "session:one",
            branch: "branch:one",
            leaf: "leaf:one",
            subject: "subject:one",
            role: "role:solver",
            run: &seat.run_identity,
            policy: "ledger-policy:one",
        },
        terminal: TerminalObservationClass::Completed,
        safe_summary_identity: Some("summary:one"),
    }];
    let episode = reduce_episode(&plan, &observations).expect("episode");
    assert_eq!(episode.completed_count, 1);
    assert_eq!(episode.runs[0].run_identity, seat.run_identity);
}

#[test]
fn finite_rewards_preserve_order_attribution_and_advisory_status() {
    let recipes = [RecipeKind::SingleAgent];
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    let dimensions = [RewardDimension {
        name: "correctness",
        minimum: 0.0,
        maximum: ONE_VALUE,
        weight: ONE_VALUE,
        scorer_identity: "scorer:one",
        verifier_identity: "verifier:one",
        rubric_identity: "rubric:one",
        subject_identity: "subject:one",
        trajectory_identity: "trajectory:one",
    }];
    let contributions = [RewardContribution {
        dimension: "correctness",
        value: Some(ONE_VALUE),
        scorer_identity: "scorer:one",
        verifier_identity: "verifier:one",
        rubric_identity: "rubric:one",
        subject_identity: "subject:one",
        trajectory_identity: "trajectory:one",
        evidence_identity: "evidence:one",
        judge_produced: true,
        claim: RewardClaim::Evidence,
    }];
    let result = reduce_reward_dimensions(&dimensions, &contributions, &policy).expect("reward");
    assert_eq!(result.aggregate, Some(ONE_VALUE));
    assert!(result.dimensions[0].advisory);
    assert_eq!(result.contribution_count, 1);
}

#[test]
fn resume_requires_a_full_exact_terminal_match() {
    let recipes = [RecipeKind::SingleAgent];
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::RedoWholeTask,
    );
    let material = resume_material();
    let key = build_resume_key(&material).expect("resume key");
    let exact = [ResumeEvidence {
        resume_key: &key,
        evidence_identity: "evidence:resume",
        output_evidence_identity: "output:evidence",
        state: ResumeEvidenceState::CompleteTerminalSuccess,
    }];
    let keep = evaluate_resume(&material, &exact, &policy).expect("keep");
    assert_eq!(keep.action, ResumeAction::Keep);

    let stale_key = hash_identity("test.stale.v1", &["stale"]);
    let stale = [ResumeEvidence {
        resume_key: &stale_key,
        evidence_identity: exact[0].evidence_identity,
        output_evidence_identity: exact[0].output_evidence_identity,
        state: exact[0].state,
    }];
    let redo = evaluate_resume(&material, &stale, &policy).expect("redo");
    assert_eq!(redo.action, ResumeAction::RedoWholeTask);

    let duplicate = [exact[0], exact[0]];
    assert_eq!(
        evaluate_resume(&material, &duplicate, &policy),
        Err(ResumeError::Conflict)
    );
    let torn = [ResumeEvidence {
        state: ResumeEvidenceState::Torn,
        ..exact[0]
    }];
    let blocked = evaluate_resume(&material, &torn, &policy).expect("torn blocks");
    assert_eq!(blocked.action, ResumeAction::Block);
}

#[test]
fn receipt_is_metadata_only_and_rejects_overclaims() {
    let labels = ["terminal_observed"];
    let claims = [EvaluationReceiptClaim::BoundedObservation];
    let draft = EvaluationReceiptDraft {
        evaluation_identity: "evaluation:one",
        taskset_identity: "taskset:one",
        recipe_identity: "recipe:one",
        episode_identity: Some("episode:one"),
        reward_identity: Some("reward:one"),
        resume_identity: Some("resume:one"),
        policy_identity: "policy:one",
        status: EvaluationReceiptStatus::Observed,
        task_count: 1,
        run_count: 1,
        completed_count: 1,
        redaction_count: 1,
        diagnostic_labels: &labels,
        claims: &claims,
    };
    let first = project_evaluation_receipt(&draft).expect("receipt");
    let second = project_evaluation_receipt(&draft).expect("same receipt");
    assert_eq!(first.identity, second.identity);
    assert!(first.non_claims.contains(&"not-authoritative-completion"));

    let overclaims = [EvaluationReceiptClaim::VerifierSoundness];
    assert_eq!(
        project_evaluation_receipt(&EvaluationReceiptDraft {
            claims: &overclaims,
            ..draft
        }),
        Err(ReceiptError::Overclaim)
    );
}

#[test]
fn unsafe_policy_task_and_recipe_fail_closed() {
    let recipes = [RecipeKind::SingleAgent];
    let unsafe_policy = EvaluationPolicyDraft {
        schema: EVALUATION_POLICY_SCHEMA,
        source_identity: SOURCE_IDENTITY,
        profile_id: PROFILE_ID,
        max_tasks: POLICY_TASKS,
        max_runs: POLICY_RUNS,
        max_reward_dimensions: POLICY_REWARDS,
        max_artifacts: POLICY_ARTIFACTS,
        network_default: EvaluationNetworkPosture::Public,
        secret_values_allowed: false,
        transcript_bodies_allowed: false,
        missing_reward: MissingRewardPolicy::Error,
        resume_mismatch: ResumeMismatchPolicy::Rerun,
        supported_recipes: &recipes,
        evidence_redaction: "metadata_only",
    };
    assert_eq!(
        validate_evaluation_policy(&unsafe_policy, SOURCE_IDENTITY),
        Err(PolicyError::UnsafeDefault)
    );
    let supported_recipes = all_recipes();
    let policy = policy_with(
        &supported_recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    let duplicate_tasks = [task("task:one"), task("task:one")];
    assert_eq!(
        plan_taskset(
            &TasksetDescriptor {
                taskset_identity: "taskset:bad",
                version: "v1",
                ordering: TaskOrdering::Stable,
                source: TasksetSource::Lazy {
                    generator_identity: "generator:one",
                    generator_version: "v1",
                    seed: 1,
                    finiteness: TasksetFiniteness::Infinite,
                    selection_bound: PAIR,
                },
            },
            &duplicate_tasks,
            &policy,
        ),
        Err(TasksetError::Membership)
    );
    let one_task = [task("task:one")];
    let globally_shuffled = TasksetDescriptor {
        taskset_identity: "taskset:shuffled",
        version: "v1",
        ordering: TaskOrdering::GlobalShuffle,
        source: TasksetSource::Lazy {
            generator_identity: "generator:one",
            generator_version: "v1",
            seed: 1,
            finiteness: TasksetFiniteness::Infinite,
            selection_bound: 1,
        },
    };
    assert_eq!(
        plan_taskset(&globally_shuffled, &one_task, &policy),
        Err(TasksetError::Ordering)
    );
    let missing_bound = TasksetDescriptor {
        ordering: TaskOrdering::Stable,
        source: TasksetSource::Lazy {
            generator_identity: "generator:one",
            generator_version: "v1",
            seed: 1,
            finiteness: TasksetFiniteness::Infinite,
            selection_bound: 0,
        },
        ..globally_shuffled
    };
    assert_eq!(
        plan_taskset(&missing_bound, &one_task, &policy),
        Err(TasksetError::Bound)
    );
    assert_eq!(
        plan_recipe(
            &RecipeRequest {
                kind: RecipeKind::UserSimulation,
                bindings: bindings(),
                candidate_count: 1,
                artifact_count: 0,
                simulated_user_trainable: true,
                simulated_user_assistant_tools: false,
            },
            &policy,
        ),
        Err(RecipeError::UserAuthority)
    );
}

#[test]
fn malformed_reward_fails_closed() {
    let recipes = [RecipeKind::SingleAgent];
    let policy = policy_with(
        &recipes,
        MissingRewardPolicy::Error,
        ResumeMismatchPolicy::Rerun,
    );
    let dimensions = [RewardDimension {
        name: "score",
        minimum: 0.0,
        maximum: ONE_VALUE,
        weight: ONE_VALUE,
        scorer_identity: "scorer:one",
        verifier_identity: "verifier:one",
        rubric_identity: "rubric:one",
        subject_identity: "subject:one",
        trajectory_identity: "trajectory:one",
    }];
    let contribution = [RewardContribution {
        dimension: "score",
        value: Some(f64::NAN),
        scorer_identity: "scorer:one",
        verifier_identity: "verifier:one",
        rubric_identity: "rubric:one",
        subject_identity: "subject:one",
        trajectory_identity: "trajectory:one",
        evidence_identity: "evidence:one",
        judge_produced: false,
        claim: RewardClaim::Evidence,
    }];
    assert_eq!(
        reduce_reward_dimensions(&dimensions, &contribution, &policy),
        Err(RewardError::NonFinite)
    );
    let out_of_domain = [RewardContribution {
        value: Some(OUT_OF_DOMAIN_VALUE),
        ..contribution[0]
    }];
    assert_eq!(
        reduce_reward_dimensions(&dimensions, &out_of_domain, &policy),
        Err(RewardError::OutOfDomain)
    );
    let overclaim = [RewardContribution {
        value: Some(ONE_VALUE),
        claim: RewardClaim::Completion,
        ..contribution[0]
    }];
    assert_eq!(
        reduce_reward_dimensions(&dimensions, &overclaim, &policy),
        Err(RewardError::Overclaim)
    );
    let duplicate = [out_of_domain[0], out_of_domain[0]];
    assert_eq!(
        reduce_reward_dimensions(&dimensions, &duplicate, &policy),
        Err(RewardError::Dimension)
    );
}

#[test]
fn leaking_receipt_fails_closed() {
    let leaked_labels = ["raw_prompt"];
    let claims = [EvaluationReceiptClaim::BoundedObservation];
    assert_eq!(
        project_evaluation_receipt(&EvaluationReceiptDraft {
            evaluation_identity: "evaluation:one",
            taskset_identity: "taskset:one",
            recipe_identity: "recipe:one",
            episode_identity: None,
            reward_identity: None,
            resume_identity: None,
            policy_identity: "policy:one",
            status: EvaluationReceiptStatus::Failed,
            task_count: 1,
            run_count: 1,
            completed_count: 0,
            redaction_count: 1,
            diagnostic_labels: &leaked_labels,
            claims: &claims,
        }),
        Err(ReceiptError::Leak)
    );
}

fn resume_material() -> ResumeMaterial<'static> {
    ResumeMaterial {
        task_data_identity: "task-data:one",
        taskset_identity: "taskset:one",
        recipe_identity: "recipe:one",
        model_identity: "model:one",
        harness_identity: "harness:one",
        environment_identity: "environment:one",
        tool_identity: "tools:one",
        verifier_identity: "verifier:one",
        rubric_identity: "rubric:one",
        reward_identity: "reward:one",
        sampling_identity: "sampling:one",
        redaction_identity: "redaction:one",
        output_identity: "output:one",
        policy_identity: "policy:one",
    }
}
