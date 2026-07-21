    // r[verify agent_harness_runtime.audit_receipts.verification]
    // r[verify agent_harness_runtime.audit_receipts.verification.closeout]
    use super::*;

    const SCHEMA_VERSION: u16 = 1;
    const ZERO_SCHEMA_VERSION: u16 = 0;
    const PENDING_LIMIT: u32 = 2;
    const ZERO_COUNT: u32 = 0;
    const ONE_COUNT: u32 = 1;
    const TWO_COUNT: u32 = 2;
    const THREE_COUNT: u32 = 3;
    const FOUR_COUNT: u32 = 4;
    const FIVE_COUNT: u32 = 5;
    const SIX_COUNT: u32 = 6;
    const WINDOW_ID: &str = "audit-window-main";
    const POLICY_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const WINDOW_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const RAW_PROMPT: &str = "raw prompt text";
    const TOOL_ARGUMENTS: &str = "tool arguments";
    const TOOL_OUTPUT: &str = "tool output";
    const PROVIDER_PAYLOAD: &str = "{\"provider\":\"payload\"}";
    const CREDENTIAL: &str = "api_key=secret";
    const AUTHORIZATION_HEADER: &str = "authorization: bearer token";
    const ENVIRONMENT_VALUE: &str = "HOME=/private";
    const FULL_PATH: &str = "/home/example/private.log";
    const UNBOUNDED_LOG: &str = "unbounded log follows";
    const EXPECTED_WORK_KINDS: usize = 6;
    const EXPECTED_FAILURE_CLASSES: usize = 6;

    #[test]
    fn empty_window_snapshot_has_zero_counts_and_safe_metadata() {
        // r[verify agent_harness_runtime.audit_receipts.snapshots.empty]
        let snapshot = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &empty_work(),
            failures: &[],
        })
        .expect("empty audit snapshot should project");

        assert_eq!(snapshot.schema_version().get(), SCHEMA_VERSION);
        assert_eq!(snapshot.window_id(), WINDOW_ID);
        assert_eq!(snapshot.policy_hash().as_str(), POLICY_HASH);
        assert_eq!(snapshot.event_window_hash().as_str(), WINDOW_HASH);
        assert_eq!(snapshot.work().len(), EXPECTED_WORK_KINDS);
        assert!(snapshot.failures().is_empty());
        assert!(snapshot.work().iter().all(|summary| {
            summary.status() == AuditStatusClass::Idle
                && summary.pending_public_count() == ZERO_COUNT
                && summary.pending_total_count() == ZERO_COUNT
                && summary.completed_count() == ZERO_COUNT
                && summary.failed_count() == ZERO_COUNT
                && !summary.pending_over_limit()
        }));
    }

    #[test]
    fn pending_tool_counts_clamp_without_private_state() {
        // r[verify agent_harness_runtime.audit_receipts.clamping.pending_tools]
        let work = [AuditWorkDraft {
            kind: AuditWorkKind::ToolCall,
            pending_count: FIVE_COUNT,
            completed_count: ONE_COUNT,
            failed_count: ZERO_COUNT,
            timing_bucket: AuditTimingBucket::Interactive,
        }];
        let snapshot = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &work,
            failures: &[],
        })
        .expect("clamped audit snapshot should project");

        let tool = snapshot
            .work()
            .iter()
            .find(|summary| summary.kind() == AuditWorkKind::ToolCall)
            .expect("tool summary should exist");
        assert_eq!(tool.status(), AuditStatusClass::Pending);
        assert_eq!(tool.pending_public_count(), PENDING_LIMIT);
        assert_eq!(tool.pending_total_count(), FIVE_COUNT);
        assert!(tool.pending_over_limit());
    }

    #[test]
    fn sink_unavailable_is_host_owned_and_projection_is_unchanged() {
        // r[verify agent_harness_runtime.audit_receipts.host_owned.sink_unavailable]
        let work = [AuditWorkDraft {
            kind: AuditWorkKind::ModelCall,
            pending_count: ONE_COUNT,
            completed_count: TWO_COUNT,
            failed_count: ZERO_COUNT,
            timing_bucket: AuditTimingBucket::Immediate,
        }];
        let before = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &work,
            failures: &[],
        })
        .expect("audit snapshot should project before sink failure");
        let host_outcome = AuditHostBoundaryOutcome::new(
            AuditHostBoundary::TelemetryExporter,
            AuditHostBoundaryStatus::Unavailable,
        );
        let after = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &work,
            failures: &[],
        })
        .expect("audit snapshot should project after sink failure");

        assert_eq!(
            host_outcome.boundary(),
            AuditHostBoundary::TelemetryExporter
        );
        assert_eq!(host_outcome.status(), AuditHostBoundaryStatus::Unavailable);
        assert_eq!(before, after);
    }

    #[test]
    fn completed_failure_classes_summarize_without_payloads() {
        // r[verify agent_harness_runtime.audit_receipts.composition.failures]
        let work = [
            completed_work(AuditWorkKind::ModelCall, ONE_COUNT),
            completed_work(AuditWorkKind::ToolCall, TWO_COUNT),
            completed_work(AuditWorkKind::Confirmation, THREE_COUNT),
            completed_work(AuditWorkKind::BackgroundWork, FOUR_COUNT),
            completed_work(AuditWorkKind::DeliveryAttempt, FIVE_COUNT),
            completed_work(AuditWorkKind::Retry, SIX_COUNT),
        ];
        let failures = [
            failure(AuditFailureClass::InvalidInput, ONE_COUNT),
            failure(AuditFailureClass::DeniedCapability, ONE_COUNT),
            failure(AuditFailureClass::Timeout, ONE_COUNT),
            failure(AuditFailureClass::Cancellation, ONE_COUNT),
            failure(AuditFailureClass::MalformedOutput, ONE_COUNT),
            failure(AuditFailureClass::UnavailableService, ONE_COUNT),
        ];
        let snapshot = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &work,
            failures: &failures,
        })
        .expect("completed audit snapshot should project");

        assert_eq!(snapshot.work().len(), EXPECTED_WORK_KINDS);
        assert!(
            snapshot
                .work()
                .iter()
                .all(|summary| summary.status() == AuditStatusClass::Completed)
        );
        assert_eq!(snapshot.failures().len(), EXPECTED_FAILURE_CLASSES);
        assert!(snapshot.failures().iter().any(|summary| {
            summary.failure_class() == AuditFailureClass::InvalidInput
                && summary.count() == ONE_COUNT
        }));
        assert!(snapshot.failures().iter().any(|summary| {
            summary.failure_class() == AuditFailureClass::UnavailableService
                && summary.count() == ONE_COUNT
        }));
    }

    #[test]
    fn receipt_validation_hashes_safe_metadata_and_rejects_leaks() {
        // r[verify agent_harness_runtime.audit_receipts.receipts.rejects_leaks]
        let snapshot = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &empty_work(),
            failures: &[],
        })
        .expect("safe audit snapshot should project");
        let first = AuditReceipt::validate(&receipt(&snapshot, ReceiptLeakFixture::none()))
            .expect("safe receipt should validate");
        let second = AuditReceipt::validate(&receipt(&snapshot, ReceiptLeakFixture::none()))
            .expect("safe receipt should validate deterministically");
        assert_eq!(first.receipt_hash(), second.receipt_hash());
        assert_eq!(first.snapshot(), &snapshot);

        assert_leak(&snapshot, ReceiptLeakFixture::raw_prompt());
        assert_leak(&snapshot, ReceiptLeakFixture::tool_arguments());
        assert_leak(&snapshot, ReceiptLeakFixture::tool_output());
        assert_leak(&snapshot, ReceiptLeakFixture::provider_payload());
        assert_leak(&snapshot, ReceiptLeakFixture::credential());
        assert_leak(&snapshot, ReceiptLeakFixture::authorization_header());
        assert_leak(&snapshot, ReceiptLeakFixture::environment_value());
        assert_leak(&snapshot, ReceiptLeakFixture::full_path());
        assert_leak(&snapshot, ReceiptLeakFixture::unbounded_log());
    }

    #[test]
    fn invalid_schema_and_oversized_projection_fail_closed() {
        assert_eq!(
            AuditProjectionPolicy::validate(&AuditProjectionPolicyDraft {
                schema_version: ZERO_SCHEMA_VERSION,
                public_pending_limit: PENDING_LIMIT,
                policy_hash: POLICY_HASH,
            }),
            Err(AuditReceiptError::ZeroSchemaVersion)
        );

        let too_many_work_items = [
            completed_work(AuditWorkKind::ModelCall, ONE_COUNT),
            completed_work(AuditWorkKind::ToolCall, ONE_COUNT),
            completed_work(AuditWorkKind::Confirmation, ONE_COUNT),
            completed_work(AuditWorkKind::BackgroundWork, ONE_COUNT),
            completed_work(AuditWorkKind::DeliveryAttempt, ONE_COUNT),
            completed_work(AuditWorkKind::Retry, ONE_COUNT),
            completed_work(AuditWorkKind::Retry, ONE_COUNT),
        ];
        let error = project_audit_snapshot(AuditProjectionInput {
            policy: &policy(PENDING_LIMIT),
            window: &window(),
            work: &too_many_work_items,
            failures: &[],
        })
        .expect_err("oversized audit work list should fail");
        assert!(matches!(error, AuditReceiptError::TooManyItems { .. }));
    }

    fn policy(public_pending_limit: u32) -> AuditProjectionPolicy {
        AuditProjectionPolicy::validate(&AuditProjectionPolicyDraft {
            schema_version: SCHEMA_VERSION,
            public_pending_limit,
            policy_hash: POLICY_HASH,
        })
        .expect("audit policy should validate")
    }

    fn window() -> AuditWindow {
        AuditWindow::validate(&AuditWindowDraft {
            window_id: WINDOW_ID,
            event_window_hash: WINDOW_HASH,
        })
        .expect("audit window should validate")
    }
