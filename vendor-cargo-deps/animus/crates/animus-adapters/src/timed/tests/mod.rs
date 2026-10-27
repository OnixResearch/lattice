use super::*;

const SCHEDULE_ID: &str = "daily-review";
const RECIPE_ID: &str = "goose.recipe.workflow";
const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const CALENDAR: &str = "utc";
const WORKSPACE: &str = "readonly-workspace";
const RETRY_LIMIT: u32 = 2;
const RETENTION_COUNT: u32 = 7;
const CURRENT_TICK: u64 = 10;
const INTERVAL_TICKS: u64 = 5;
const NEXT_TICK: u64 = 15;
const ZERO_INTERVAL: u64 = 0;
const FIRST_COUNT: u32 = 1;

fn admitted_recipe() -> crate::RecipeTrustReceipt {
    crate::RecipeTrustReceipt::new(&crate::RecipeProvenanceDraft {
        source_kind: "project",
        source_label: "config/recipe-workflows.ncl",
        content_hash: HASH_A,
        admitted: true,
    })
    .expect("trust receipt should parse")
}

fn pending_recipe() -> crate::RecipeTrustReceipt {
    crate::RecipeTrustReceipt::new(&crate::RecipeProvenanceDraft {
        source_kind: "project",
        source_label: "config/recipe-workflows.ncl",
        content_hash: HASH_A,
        admitted: false,
    })
    .expect("trust receipt should parse")
}

fn schedule_draft(receipt: &crate::RecipeTrustReceipt) -> ScheduledWorkflowDraft<'_> {
    ScheduledWorkflowDraft {
        schedule_id: SCHEDULE_ID,
        recipe_id: RECIPE_ID,
        recipe_version_hash: HASH_A,
        parameter_hashes: &[HASH_B],
        cadence: CADENCE_DAILY,
        calendar_policy: CALENDAR,
        execution_mode: EXECUTION_BACKGROUND,
        concurrency_policy: CONCURRENCY_SERIALIZE,
        workspace_scope: WORKSPACE,
        retry_limit: RETRY_LIMIT,
        retention_count: RETENTION_COUNT,
        provenance_hash: HASH_C,
        recipe_trust: Some(receipt),
    }
}

const fn admitted_policy() -> ScheduledRunPolicyAdmission {
    ScheduledRunPolicyAdmission {
        recipe_trusted: true,
        capability_allowed: true,
        sandbox_allowed: true,
    }
}

const fn admitted_runtime() -> ScheduledRunRuntimeAdmission {
    ScheduledRunRuntimeAdmission {
        queue_admitted: true,
        cancellation_handle: true,
        policy_current: true,
    }
}

const fn admitted_run() -> ScheduledRunAdmissionDraft {
    ScheduledRunAdmissionDraft {
        policy: admitted_policy(),
        runtime: admitted_runtime(),
    }
}

#[test]
fn schedule_descriptor_accepts_read_only_recurring_and_run_now() {
    let receipt = admitted_recipe();
    let schedule =
        ScheduledWorkflow::validate(&schedule_draft(&receipt)).expect("schedule should validate");
    let run_now = ScheduledWorkflow::validate(&ScheduledWorkflowDraft {
        cadence: CADENCE_RUN_NOW,
        execution_mode: EXECUTION_FOREGROUND,
        ..schedule_draft(&receipt)
    })
    .expect("run-now schedule should validate");

    assert_eq!(schedule.schedule_id(), SCHEDULE_ID);
    assert_eq!(schedule.cadence(), ScheduleCadence::Daily);
    assert_eq!(run_now.cadence(), ScheduleCadence::RunNow);
}

#[test]
fn schedule_descriptor_rejects_invalid_cadence_missing_recipe_and_untrusted_recipe() {
    let admitted = admitted_recipe();
    let pending = pending_recipe();

    assert!(matches!(
        ScheduledWorkflow::validate(&ScheduledWorkflowDraft {
            cadence: "sometimes",
            ..schedule_draft(&admitted)
        }),
        Err(ScheduleError::UnsupportedValue { .. })
    ));
    assert!(matches!(
        ScheduledWorkflow::validate(&ScheduledWorkflowDraft {
            recipe_id: "",
            ..schedule_draft(&admitted)
        }),
        Err(ScheduleError::Contract(
            crate::AdapterContractError::EmptyField { .. }
        ))
    ));
    assert_eq!(
        ScheduledWorkflow::validate(&schedule_draft(&pending)),
        Err(ScheduleError::UntrustedRecipe)
    );
}

#[test]
fn next_run_planning_uses_explicit_ticks_only() {
    let receipt = admitted_recipe();
    let schedule =
        ScheduledWorkflow::validate(&schedule_draft(&receipt)).expect("schedule should validate");
    let plan = plan_next_run(NextRunInput {
        schedule: &schedule,
        current_tick: CURRENT_TICK,
        interval_ticks: INTERVAL_TICKS,
    })
    .expect("plan should compute");

    assert_eq!(plan.next_tick(), NEXT_TICK);
    assert_eq!(
        plan_next_run(NextRunInput {
            schedule: &schedule,
            current_tick: CURRENT_TICK,
            interval_ticks: ZERO_INTERVAL,
        }),
        Err(ScheduleError::InvalidPlanningInput)
    );
    assert!(alloc::format!("{PROHIBITED_SCHEDULE_EFFECTS:?}").contains("cron edit"));
}

#[test]
fn run_admission_denies_missing_queue_untrusted_policy_and_cancellation() {
    assert_eq!(
        admit_scheduled_run(admitted_run()),
        ScheduledRunAdmissionStatus::Admitted
    );
    assert_eq!(
        admit_scheduled_run(ScheduledRunAdmissionDraft {
            runtime: ScheduledRunRuntimeAdmission {
                queue_admitted: false,
                ..admitted_runtime()
            },
            ..admitted_run()
        }),
        ScheduledRunAdmissionStatus::Denied {
            reason: ScheduledRunDenial::QueueService,
        }
    );
    assert_eq!(
        admit_scheduled_run(ScheduledRunAdmissionDraft {
            runtime: ScheduledRunRuntimeAdmission {
                cancellation_handle: false,
                ..admitted_runtime()
            },
            ..admitted_run()
        }),
        ScheduledRunAdmissionStatus::Denied {
            reason: ScheduledRunDenial::CancellationHandle,
        }
    );
    assert_eq!(
        admit_scheduled_run(ScheduledRunAdmissionDraft {
            policy: ScheduledRunPolicyAdmission {
                recipe_trusted: false,
                ..admitted_policy()
            },
            ..admitted_run()
        }),
        ScheduledRunAdmissionStatus::Denied {
            reason: ScheduledRunDenial::UntrustedRecipe,
        }
    );
}

#[test]
fn concurrency_conflicts_are_serialized_isolated_or_denied() {
    let overlap = ScheduledConcurrencyInput {
        policy: ScheduleConcurrencyPolicy::Serialize,
        active_run: true,
        active_run_writes: true,
        new_run_writes: true,
    };

    assert_eq!(
        evaluate_scheduled_concurrency(overlap),
        ScheduledConcurrencyDecision::Serialize
    );
    assert_eq!(
        evaluate_scheduled_concurrency(ScheduledConcurrencyInput {
            policy: ScheduleConcurrencyPolicy::Isolate,
            ..overlap
        }),
        ScheduledConcurrencyDecision::Isolate
    );
    assert_eq!(
        evaluate_scheduled_concurrency(ScheduledConcurrencyInput {
            policy: ScheduleConcurrencyPolicy::Deny,
            ..overlap
        }),
        ScheduledConcurrencyDecision::Deny
    );
}

#[test]
fn run_records_are_redacted_hash_and_count_records() {
    let record = ScheduledRunRecord::new(&ScheduledRunRecordDraft {
        schedule_hash: HASH_A,
        recipe_hash: HASH_B,
        parameter_hashes: &[HASH_C],
        queue_job_hash: HASH_A,
        ledger_hash: HASH_B,
        retry_count: RETRY_LIMIT,
        outcome: "success",
        evidence_hashes: &[HASH_C],
        redactions: &["prompt_hashed", "secret_parameter_omitted"],
    })
    .expect("record should parse");

    assert_eq!(record.parameter_count(), FIRST_COUNT);
    assert_eq!(record.evidence_count(), FIRST_COUNT);
    assert_eq!(
        validate_scheduled_record_text("hash counts redactions"),
        Ok(())
    );
    assert_eq!(
        validate_scheduled_record_text("prompt: hi secret /home/user tool output"),
        Err(ScheduleError::RecordLeak)
    );
}

#[test]
fn controls_require_host_admission_and_confirmation() {
    let request = ScheduleControlRequest::validate(&ScheduleControlDraft {
        schedule_id: SCHEDULE_ID,
        action: CONTROL_RUN_NOW,
        host_admitted: true,
        confirmed: true,
    })
    .expect("control should validate");

    assert_eq!(request.action(), ScheduleControlAction::RunNow);
    assert_eq!(
        ScheduleControlRequest::validate(&ScheduleControlDraft {
            schedule_id: SCHEDULE_ID,
            action: CONTROL_EDIT,
            host_admitted: false,
            confirmed: true,
        }),
        Err(ScheduleError::ControlDenied)
    );
    assert_eq!(
        ScheduleControlRequest::validate(&ScheduleControlDraft {
            schedule_id: SCHEDULE_ID,
            action: CONTROL_CANCEL,
            host_admitted: true,
            confirmed: false,
        }),
        Err(ScheduleError::ControlDenied)
    );
}
