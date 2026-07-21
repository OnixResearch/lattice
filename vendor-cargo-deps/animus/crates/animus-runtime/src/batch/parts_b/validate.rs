const LEAK_MARKERS: &[LeakMarker] = &[
    LeakMarker {
        needle: "raw_prompt:",
        marker: "raw_prompt",
    },
    LeakMarker {
        needle: "provider payload",
        marker: "provider_payload",
    },
    LeakMarker {
        needle: "provider_payload",
        marker: "provider_payload",
    },
    LeakMarker {
        needle: "authorization:",
        marker: "credential",
    },
    LeakMarker {
        needle: "api_key",
        marker: "credential",
    },
    LeakMarker {
        needle: "secret=",
        marker: "credential",
    },
    LeakMarker {
        needle: "http://",
        marker: "remote_destination",
    },
    LeakMarker {
        needle: "https://",
        marker: "remote_destination",
    },
    LeakMarker {
        needle: "s3://",
        marker: "remote_destination",
    },
    LeakMarker {
        needle: "/home/",
        marker: "full_path",
    },
    LeakMarker {
        needle: "unbounded_output:",
        marker: "unbounded_output",
    },
];

fn validate_jobs(drafts: &[BatchJobDraft<'_>]) -> Result<Collection<BatchJob>, BatchError> {
    if drafts.is_empty() {
        return Err(BatchError::EmptyJobs {
            field: EMPTY_JOBS_MARKER,
        });
    }
    let mut jobs = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let id = validate_safe_label(JOB_ID_FIELD, draft.job_id)?;
        if jobs
            .iter()
            .any(|job: &BatchJob| job.job_id() == id.as_str())
        {
            return Err(BatchError::DuplicateValue {
                field: JOB_ID_FIELD,
                value: id,
            });
        }
        let prompt = required_text(PROMPT_FIELD, draft.prompt)?;
        let objective_label = validate_safe_label(OBJECTIVE_LABEL_FIELD, draft.objective_label)?;
        let output_label = BatchOutputLabel::new(draft.output_label)?;
        let model_handle = validate_optional_safe_text(MODEL_HANDLE_FIELD, draft.model_handle)?;
        let session_handle =
            validate_optional_safe_text(SESSION_HANDLE_FIELD, draft.session_handle)?;
        let prompt_bytes = prompt.len();
        let job_hash = hash_parts(&[
            BATCH_JOB_SCHEMA,
            id.as_str(),
            prompt.as_str(),
            objective_label.as_str(),
            output_label.as_str(),
            model_handle.as_deref().unwrap_or_default(),
            session_handle.as_deref().unwrap_or_default(),
        ]);
        jobs.push(BatchJob {
            id,
            prompt,
            objective_label,
            output_label,
            model_handle,
            session_handle,
            prompt_bytes,
            job_hash,
        });
    }
    jobs.sort_by(|left, right| left.job_id().cmp(right.job_id()));
    Ok(jobs)
}

fn validate_resume_entries(
    drafts: &[ResumeCompletedJobDraft<'_>],
) -> Result<Collection<ResumeEntry>, BatchError> {
    let mut entries = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let job_id = validate_safe_label(JOB_ID_FIELD, draft.job_id)?;
        if entries
            .iter()
            .any(|entry: &ResumeEntry| entry.job_id == job_id)
        {
            return Err(BatchError::DuplicateValue {
                field: JOB_ID_FIELD,
                value: job_id,
            });
        }
        entries.push(ResumeEntry {
            job_id,
            plan_hash: validate_safe_metadata("entry_plan_hash", draft.plan_hash)?,
            output_format: BatchOutputFormat::parse(draft.output_format)?,
            redaction_policy: BatchRedactionPolicy::parse(draft.redaction_policy)?,
            output_evidence_hash: validate_safe_metadata(
                OUTPUT_EVIDENCE_HASH_FIELD,
                draft.output_evidence_hash,
            )?,
            status: ResumeJobStatus::parse(draft.status)?,
        });
    }
    Ok(entries)
}

fn validate_job_receipts(
    drafts: &[BatchJobReceiptDraft<'_>],
) -> Result<Collection<BatchJobReceipt>, BatchError> {
    if drafts.is_empty() {
        return Err(BatchError::EmptyJobs {
            field: EMPTY_JOBS_MARKER,
        });
    }
    let mut receipts = Collection::with_capacity(drafts.len());
    for draft in drafts {
        let job_id = validate_safe_label(JOB_ID_FIELD, draft.job_id)?;
        if receipts
            .iter()
            .any(|receipt: &BatchJobReceipt| receipt.job_id == job_id)
        {
            return Err(BatchError::DuplicateValue {
                field: JOB_ID_FIELD,
                value: job_id,
            });
        }
        receipts.push(BatchJobReceipt {
            job_id,
            status: BatchJobStatus::parse(draft.status)?,
            prompt_bytes: draft.prompt_bytes,
            model_handle: validate_optional_safe_text(MODEL_HANDLE_FIELD, draft.model_handle)?,
            session_handle: validate_optional_safe_text(
                SESSION_HANDLE_FIELD,
                draft.session_handle,
            )?,
            objective_summary: validate_safe_metadata(
                OBJECTIVE_SUMMARY_FIELD,
                draft.objective_summary,
            )?,
            output_label: BatchOutputLabel::new(draft.output_label)?,
            output_evidence_hash: validate_safe_metadata(
                OUTPUT_EVIDENCE_HASH_FIELD,
                draft.output_evidence_hash,
            )?,
            redaction_marker: validate_safe_metadata(
                REDACTION_MARKER_FIELD,
                draft.redaction_marker,
            )?,
        });
    }
    Ok(receipts)
}

fn reject_unknown_or_duplicate_outcomes(
    plan: &BatchPlan,
    outcomes: &[BatchJobOutcome],
) -> Result<(), BatchError> {
    let mut seen = Collection::with_capacity(outcomes.len());
    for outcome in outcomes {
        if plan
            .jobs()
            .iter()
            .all(|job| job.job_id() != outcome.job_id())
        {
            return Err(BatchError::UnknownJob {
                job_id: Text::from(outcome.job_id()),
            });
        }
        if seen.iter().any(|job_id| job_id == outcome.job_id()) {
            return Err(BatchError::DuplicateValue {
                field: JOB_ID_FIELD,
                value: Text::from(outcome.job_id()),
            });
        }
        seen.push(Text::from(outcome.job_id()));
    }
    Ok(())
}

fn find_outcome<'a>(outcomes: &'a [BatchJobOutcome], job_id: &str) -> Option<&'a BatchJobOutcome> {
    outcomes.iter().find(|outcome| outcome.job_id() == job_id)
}

#[derive(Clone, Copy)]
struct ResumeHeaderInput<'a> {
    plan: &'a BatchPlan,
    run_id: &'a str,
    plan_hash: &'a str,
    output_format: BatchOutputFormat,
    redaction_policy: BatchRedactionPolicy,
}

fn header_stale_reason(input: ResumeHeaderInput<'_>) -> Option<ResumeStaleReason> {
    if input.run_id != input.plan.identity().run_id() {
        return Some(ResumeStaleReason::RunIdMismatch);
    }
    if input.plan_hash != input.plan.identity().plan_hash() {
        return Some(ResumeStaleReason::PlanHashMismatch);
    }
    if input.output_format != input.plan.output_format() {
        return Some(ResumeStaleReason::OutputFormatMismatch);
    }
    if input.redaction_policy != input.plan.redaction_policy() {
        return Some(ResumeStaleReason::RedactionPolicyMismatch);
    }
    None
}

fn entry_stale_reason(
    plan: &BatchPlan,
    job: &BatchJob,
    entries: &[ResumeEntry],
) -> Option<ResumeStaleReason> {
    let Some(entry) = entries
        .iter()
        .find(|candidate| candidate.job_id == job.job_id())
    else {
        return Some(ResumeStaleReason::MissingCompletedEvidence);
    };
    if entry.plan_hash != plan.identity().plan_hash() {
        return Some(ResumeStaleReason::PlanHashMismatch);
    }
    if entry.output_format != plan.output_format() {
        return Some(ResumeStaleReason::OutputFormatMismatch);
    }
    if entry.redaction_policy != plan.redaction_policy() {
        return Some(ResumeStaleReason::RedactionPolicyMismatch);
    }
    if entry.status != ResumeJobStatus::Completed {
        return Some(ResumeStaleReason::JobNotCompleted);
    }
    let expected_evidence = plan.expected_output_evidence_hash(job);
    if entry.output_evidence_hash != expected_evidence {
        return Some(ResumeStaleReason::OutputEvidenceMismatch);
    }
    None
}
