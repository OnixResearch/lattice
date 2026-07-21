    #[test]
    fn local_batch_planning_sorts_jobs_and_builds_identity() {
        let plan = valid_plan();

        assert_eq!(plan.identity().label(), RUN_LABEL);
        assert!(!plan.identity().run_id().is_empty());
        assert!(!plan.identity().plan_hash().is_empty());
        assert_eq!(plan.output_format(), BatchOutputFormat::Jsonl);
        assert_eq!(plan.concurrency().limit(), CONCURRENCY_LIMIT);
        assert_eq!(plan.execution_mode(), BatchExecutionMode::Headless);
        assert_eq!(plan.jobs()[0].job_id(), FIRST_JOB_ID);
        assert_eq!(plan.jobs()[1].job_id(), SECOND_JOB_ID);
        assert_eq!(plan.jobs()[0].prompt_bytes(), FIRST_PROMPT.len());
    }
    #[test]
    fn missing_execution_service_fails_without_fallback() {
        let plan = valid_plan();
        let error = execute_batch_plan(&plan, None).expect_err("missing service must fail");

        assert_eq!(
            error,
            BatchError::ExecutionServiceUnavailable {
                reason: MISSING_SERVICE_REASON,
            }
        );
    }

    #[test]
    fn injected_execution_service_is_called_once_per_planned_job() {
        let plan = valid_plan();
        let mut service = CountingExecutionService { calls: 0 };
        let outcomes = execute_batch_plan(&plan, Some(&mut service)).expect("service should run");

        assert_eq!(service.calls, plan.jobs().len());
        assert_eq!(outcomes.len(), plan.jobs().len());
    }

    #[test]
    fn matching_resume_manifest_skips_completed_jobs() {
        let plan = valid_plan();
        let first_evidence = plan.expected_output_evidence_hash(&plan.jobs()[0]);
        let second_evidence = plan.expected_output_evidence_hash(&plan.jobs()[1]);
        let first_entry = ResumeCompletedJobDraft {
            job_id: FIRST_JOB_ID,
            plan_hash: plan.identity().plan_hash(),
            output_format: plan.output_format().as_str(),
            redaction_policy: plan.redaction_policy().as_str(),
            output_evidence_hash: first_evidence.as_str(),
            status: RESUME_COMPLETED,
        };
        let second_entry = ResumeCompletedJobDraft {
            job_id: SECOND_JOB_ID,
            output_evidence_hash: second_evidence.as_str(),
            ..first_entry
        };
        let entries = [first_entry, second_entry];
        let resume = evaluate_resume_manifest(&plan, &manifest(&plan, entries.as_slice()))
            .expect("resume manifest should validate");

        assert!(
            resume
                .decisions()
                .iter()
                .all(|decision| decision.action() == BatchResumeAction::Skip)
        );
    }
    #[test]
    fn concurrent_outcomes_render_in_plan_order() {
        let plan = valid_plan();
        let second = outcome(SECOND_JOB_ID, SECOND_OUTPUT_LABEL, FIRST_COMPLETION);
        let first = outcome(FIRST_JOB_ID, FIRST_OUTPUT_LABEL, SECOND_COMPLETION);
        let records = render_plan_order_records(&plan, &[second, first]).expect("records render");

        assert_eq!(records[0].job_id(), FIRST_JOB_ID);
        assert_eq!(records[1].job_id(), SECOND_JOB_ID);
        assert_eq!(records[0].redaction_marker(), RECEIPT_REDACTION_MARKER);
    }
    #[test]
    fn receipts_include_safe_metadata_and_hashes_only() {
        let plan = valid_plan();
        let records = render_plan_order_records(
            &plan,
            &[
                outcome(FIRST_JOB_ID, FIRST_OUTPUT_LABEL, FIRST_COMPLETION),
                outcome(SECOND_JOB_ID, SECOND_OUTPUT_LABEL, SECOND_COMPLETION),
            ],
        )
        .expect("records should render");
        let receipt = BatchReceipt::from_plan_records(&plan, records.as_slice())
            .expect("receipt should validate");

        assert_eq!(receipt.run_id(), plan.identity().run_id());
        assert_eq!(receipt.jobs().len(), plan.jobs().len());
        assert_eq!(receipt.jobs()[0].prompt_bytes(), FIRST_PROMPT.len());
        assert!(!receipt.receipt_hash().contains(FIRST_PROMPT));
    }
