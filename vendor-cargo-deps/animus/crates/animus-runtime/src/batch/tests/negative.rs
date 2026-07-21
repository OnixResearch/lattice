    #[test]
    fn remote_input_and_output_locations_fail_closed_before_execution() {
        let remote_input = BatchPlan::validate(&BatchRequestDraft {
            source_location: REMOTE_HTTP_LOCATION,
            ..valid_request()
        })
        .expect_err("remote input must fail");
        let remote_output = BatchPlan::validate(&BatchRequestDraft {
            output_location: REMOTE_S3_LOCATION,
            ..valid_request()
        })
        .expect_err("remote output must fail");

        assert_eq!(
            remote_input,
            BatchError::UnsupportedLocation {
                field: SOURCE_LOCATION_FIELD,
                value: Text::from(REMOTE_HTTP_LOCATION),
            }
        );
        assert_eq!(
            remote_output,
            BatchError::UnsupportedLocation {
                field: OUTPUT_LOCATION_FIELD,
                value: Text::from(REMOTE_S3_LOCATION),
            }
        );
    }
    #[test]
    fn stale_resume_manifest_does_not_skip_jobs() {
        let plan = valid_plan();
        let stale_entry = ResumeCompletedJobDraft {
            job_id: FIRST_JOB_ID,
            plan_hash: "stale-plan-hash",
            output_format: FORMAT_EVAL_JSONL,
            redaction_policy: plan.redaction_policy().as_str(),
            output_evidence_hash: "stale-evidence-hash",
            status: RESUME_COMPLETED,
        };
        let entries = [stale_entry];
        let resume = evaluate_resume_manifest(&plan, &manifest(&plan, entries.as_slice()))
            .expect("stale manifest should still report decisions");

        assert_eq!(resume.decisions()[0].action(), BatchResumeAction::Run);
        assert_eq!(
            resume.decisions()[0].stale_reason(),
            Some(ResumeStaleReason::PlanHashMismatch)
        );
        assert_eq!(resume.decisions()[1].action(), BatchResumeAction::Run);
        assert_eq!(
            resume.decisions()[1].stale_reason(),
            Some(ResumeStaleReason::MissingCompletedEvidence)
        );
    }
    #[test]
    fn output_rendering_rejects_unknown_and_missing_jobs() {
        let plan = valid_plan();
        let unknown = outcome(UNKNOWN_JOB_ID, FIRST_OUTPUT_LABEL, FIRST_COMPLETION);
        let missing = render_plan_order_records(
            &plan,
            &[outcome(FIRST_JOB_ID, FIRST_OUTPUT_LABEL, FIRST_COMPLETION)],
        )
        .expect_err("missing planned outcome should fail");
        let unknown_error =
            render_plan_order_records(&plan, &[unknown]).expect_err("unknown outcome should fail");

        assert_eq!(
            missing,
            BatchError::MissingOutcome {
                job_id: Text::from(SECOND_JOB_ID),
            }
        );
        assert_eq!(
            unknown_error,
            BatchError::UnknownJob {
                job_id: Text::from(UNKNOWN_JOB_ID),
            }
        );
    }
    #[test]
    fn receipt_metadata_rejects_leak_fixtures() {
        let fixtures = [
            (RAW_PROMPT_LEAK, "raw_prompt"),
            (PROVIDER_PAYLOAD_LEAK, "provider_payload"),
            (CREDENTIAL_LEAK, "credential"),
            (REMOTE_HTTP_LOCATION, "remote_destination"),
            (FULL_PATH_LEAK, "full_path"),
            (UNBOUNDED_OUTPUT_LEAK, "unbounded_output"),
        ];

        for (leak, marker) in fixtures {
            let error = BatchReceipt::validate(&BatchReceiptDraft {
                jobs: &[BatchJobReceiptDraft {
                    objective_summary: leak,
                    ..VALID_RECEIPT_JOB
                }],
                ..valid_receipt()
            })
            .expect_err("leak fixture should fail");
            assert_eq!(
                error,
                BatchError::UnsafeMetadata {
                    field: OBJECTIVE_SUMMARY_FIELD,
                    marker,
                }
            );
        }
    }
