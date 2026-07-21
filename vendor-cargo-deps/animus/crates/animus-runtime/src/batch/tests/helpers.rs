    fn valid_request() -> BatchRequestDraft<'static> {
        BatchRequestDraft {
            run_label: RUN_LABEL,
            source_location: SOURCE_LOCATION,
            output_location: OUTPUT_LOCATION,
            output_format: FORMAT_JSONL,
            concurrency_limit: CONCURRENCY_LIMIT,
            execution_mode: MODE_HEADLESS,
            redaction_policy: REDACTION_HASH_ONLY,
            jobs: &[
                BatchJobDraft {
                    job_id: SECOND_JOB_ID,
                    prompt: SECOND_PROMPT,
                    objective_label: OBJECTIVE_LABEL,
                    output_label: SECOND_OUTPUT_LABEL,
                    model_handle: Some(MODEL_HANDLE),
                    session_handle: Some(SESSION_HANDLE),
                },
                BatchJobDraft {
                    job_id: FIRST_JOB_ID,
                    prompt: FIRST_PROMPT,
                    objective_label: OBJECTIVE_LABEL,
                    output_label: FIRST_OUTPUT_LABEL,
                    model_handle: Some(MODEL_HANDLE),
                    session_handle: Some(SESSION_HANDLE),
                },
            ],
        }
    }

    fn valid_plan() -> BatchPlan {
        BatchPlan::validate(&valid_request()).expect("valid plan should pass")
    }

    fn outcome(job_id: &str, output_label: &str, completion_sequence: u64) -> BatchJobOutcome {
        BatchJobOutcome::validate(&BatchJobOutcomeDraft {
            job_id,
            status: STATUS_SUCCEEDED,
            completion_sequence,
            model_handle: Some(MODEL_HANDLE),
            session_handle: Some(SESSION_HANDLE),
            objective_summary: OBJECTIVE_SUMMARY,
            output_label,
            output_evidence_hash: RECEIPT_REDACTION_MARKER,
            redaction_marker: RECEIPT_REDACTION_MARKER,
        })
        .expect("outcome should validate")
    }

    fn manifest<'a>(
        plan: &'a BatchPlan,
        completed_jobs: &'a [ResumeCompletedJobDraft<'a>],
    ) -> ResumeManifestDraft<'a> {
        ResumeManifestDraft {
            run_id: plan.identity().run_id(),
            plan_hash: plan.identity().plan_hash(),
            output_format: plan.output_format().as_str(),
            redaction_policy: plan.redaction_policy().as_str(),
            completed_jobs,
        }
    }

    fn valid_receipt() -> BatchReceiptDraft<'static> {
        BatchReceiptDraft {
            run_id: "run-hash",
            plan_hash: "plan-hash",
            output_format: FORMAT_JSONL,
            redaction_policy: REDACTION_HASH_ONLY,
            jobs: VALID_RECEIPT_JOBS,
        }
    }
