    // r[verify agent_harness_runtime.artifact_delivery_gateway.verification]
    // r[verify agent_harness_runtime.artifact_delivery_gateway.verification.closeout]
    use super::*;

    const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ATTEMPT_ID: &str = "attempt-001";
    const RETRY_ATTEMPT_ID: &str = "attempt-retryable";
    const ARTIFACT_HANDLE: &str = "artifact-handle-001";
    const ARTIFACT_LABEL: &str = "safe-artifact";
    const CONTEXT_LABEL: &str = "active-session";
    const TOOLSET_LABEL: &str = "local-delivery";
    const PLATFORM_HANDLE: &str = "platform-handle-safe";
    const RAW_DESTINATION: &str = "matrix:!room:example";
    const RAW_WEBHOOK_URL: &str = "https://hooks.example.invalid/secret";
    const RAW_HEADER: &str = "authorization: bearer secret";
    const RAW_CLOUD_URL: &str = "s3://bucket/private";
    const RAW_PLATFORM_PAYLOAD: &str = "{\"room\":\"secret\"}";
    const RAW_TOKEN: &str = "token=secret";
    const FULL_PATH: &str = "/home/example/private/artifact.png";
    const RAW_ARTIFACT_BYTES: &[u8] = b"raw artifact bytes";

    #[test]
    fn local_delivery_request_normalizes_without_raw_artifact_bytes() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.delivery_dto.local]
        let request = request().expect("safe local delivery request should normalize");

        assert_eq!(request.artifact_handle(), ARTIFACT_HANDLE);
        assert_eq!(request.artifact_hash().as_str(), VALID_HASH);
        assert_eq!(request.artifact_kind(), DeliveryArtifactKind::Image);
        assert_eq!(request.target_kind(), DeliveryTargetKind::LocalSession);
        assert_eq!(request.active_context_label(), Some(CONTEXT_LABEL));
        assert_eq!(request.safe_artifact_label(), ARTIFACT_LABEL);
        assert_eq!(request.toolset_label(), TOOLSET_LABEL);
        assert_eq!(request.capability(), DeliveryCapabilityKind::SessionLocal);

        assert_eq!(
            ArtifactDeliveryRequest::normalize(&ArtifactDeliveryRequestDraft {
                raw_artifact_bytes: Some(RAW_ARTIFACT_BYTES),
                ..request_draft()
            }),
            Err(DeliveryError::RawArtifactBytesEmbedded)
        );
    }

    #[test]
    fn policy_rejects_raw_destinations_before_adapter_execution() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.policy.raw_destination]
        let request = request().expect("safe request should normalize");
        assert_eq!(request.target_kind(), DeliveryTargetKind::LocalSession);
        assert_eq!(request.active_context_label(), Some(CONTEXT_LABEL));

        assert_policy_denied(
            PolicyInput {
                raw_destination: Some(RAW_DESTINATION),
                ..policy_input(&request)
            },
            DeliveryFailureClass::UnsafeDestination,
        );
        assert_policy_denied(
            PolicyInput {
                webhook_url: Some(RAW_WEBHOOK_URL),
                ..policy_input(&request)
            },
            DeliveryFailureClass::UnsafeDestination,
        );
        assert_policy_denied(
            PolicyInput {
                raw_header: Some(RAW_HEADER),
                ..policy_input(&request)
            },
            DeliveryFailureClass::UnsafeDestination,
        );
        assert_policy_denied(
            PolicyInput {
                cloud_object_url: Some(RAW_CLOUD_URL),
                ..policy_input(&request)
            },
            DeliveryFailureClass::UnsafeDestination,
        );
        assert_policy_denied(
            PolicyInput {
                platform_payload: Some(RAW_PLATFORM_PAYLOAD),
                ..policy_input(&request)
            },
            DeliveryFailureClass::UnsafeDestination,
        );
    }

    #[test]
    fn policy_rejects_unsupported_targets_and_missing_context() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.policy]
        let matrix_request = ArtifactDeliveryRequest::normalize(&ArtifactDeliveryRequestDraft {
            target_kind: DeliveryTargetKind::MatrixRoom,
            ..request_draft()
        })
        .expect("unsupported target request should still normalize");
        assert_eq!(matrix_request.target_kind(), DeliveryTargetKind::MatrixRoom);
        assert_eq!(matrix_request.active_context_label(), Some(CONTEXT_LABEL));
        assert_policy_denied(
            policy_input(&matrix_request),
            DeliveryFailureClass::UnsupportedTarget,
        );

        let missing_context = ArtifactDeliveryRequest::normalize(&ArtifactDeliveryRequestDraft {
            active_context_label: None,
            ..request_draft()
        })
        .expect("missing context label is policy-level denial");
        assert_policy_denied(
            PolicyInput {
                context_status: DeliveryContextStatus::Missing,
                ..policy_input(&missing_context)
            },
            DeliveryFailureClass::MissingContext,
        );
    }

    #[test]
    fn missing_adapter_fails_without_success_receipt() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.adapters.missing]
        let request = request().expect("safe request should normalize");
        let policy = evaluate_policy(policy_input(&request));
        let attempt = record_delivery_attempt(DeliveryExecutionInput {
            request: &request,
            policy: &policy,
            attempt_id: ATTEMPT_ID,
            adapter_availability: DeliveryAdapterAvailability::Missing,
            adapter_outcome: None,
        })
        .expect("missing adapter should record a safe failed attempt");

        assert_eq!(attempt.status(), DeliveryAttemptStatus::Failed);
        assert_eq!(
            attempt.error_class(),
            Some(DeliveryFailureClass::MissingAdapter)
        );
        assert_eq!(attempt.adapter_outcome(), None);
        assert_eq!(attempt.retryability(), DeliveryRetryability::NonRetryable);
    }
    #[test]
    fn adapter_success_and_retryable_failure_record_safe_attempts() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.adapters]
        // r[verify agent_harness_runtime.artifact_delivery_gateway.outbox]
        let request = request().expect("safe request should normalize");
        let policy = evaluate_policy(policy_input(&request));
        let success = record_delivery_attempt(DeliveryExecutionInput {
            request: &request,
            policy: &policy,
            attempt_id: ATTEMPT_ID,
            adapter_availability: DeliveryAdapterAvailability::Available,
            adapter_outcome: Some(DeliveryAdapterOutcomeDraft {
                status: DeliveryAdapterOutcomeStatus::Succeeded,
                platform_handle: Some(PLATFORM_HANDLE),
                failure_class: None,
            }),
        })
        .expect("available adapter success should record");

        assert_eq!(success.status(), DeliveryAttemptStatus::Succeeded);
        assert_eq!(success.error_class(), None);
        assert_eq!(success.platform_handle(), Some(PLATFORM_HANDLE));

        let retryable = record_delivery_attempt(DeliveryExecutionInput {
            request: &request,
            policy: &policy,
            attempt_id: RETRY_ATTEMPT_ID,
            adapter_availability: DeliveryAdapterAvailability::Available,
            adapter_outcome: Some(DeliveryAdapterOutcomeDraft {
                status: DeliveryAdapterOutcomeStatus::RetryableFailure,
                platform_handle: None,
                failure_class: None,
            }),
        })
        .expect("retryable adapter failure should record");

        assert_eq!(retryable.status(), DeliveryAttemptStatus::Failed);
        assert_eq!(retryable.retryability(), DeliveryRetryability::Retryable);
        assert_eq!(
            retryable.error_class(),
            Some(DeliveryFailureClass::RetryableAdapterFailure)
        );
        assert_eq!(
            plan_delivery_retry(DeliveryRetryInput {
                attempt: &retryable,
                current_policy: &policy,
                adapter_availability: DeliveryAdapterAvailability::Available,
            })
            .status(),
            DeliveryRetryPlanStatus::Execute
        );
    }

    #[test]
    fn non_retryable_policy_denial_preserves_original_failure_class() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.outbox.non_retryable]
        let request = request().expect("safe request should normalize");
        let denied_policy = evaluate_policy(PolicyInput {
            raw_credential: Some(RAW_TOKEN),
            ..policy_input(&request)
        });
        let attempt = record_delivery_attempt(DeliveryExecutionInput {
            request: &request,
            policy: &denied_policy,
            attempt_id: ATTEMPT_ID,
            adapter_availability: DeliveryAdapterAvailability::Available,
            adapter_outcome: Some(DeliveryAdapterOutcomeDraft {
                status: DeliveryAdapterOutcomeStatus::Succeeded,
                platform_handle: Some(PLATFORM_HANDLE),
                failure_class: None,
            }),
        })
        .expect("policy denial should record without adapter success");
        let retry_plan = plan_delivery_retry(DeliveryRetryInput {
            attempt: &attempt,
            current_policy: &evaluate_policy(policy_input(&request)),
            adapter_availability: DeliveryAdapterAvailability::Available,
        });

        assert_eq!(attempt.status(), DeliveryAttemptStatus::Failed);
        assert_eq!(
            attempt.error_class(),
            Some(DeliveryFailureClass::UnsafeDestination)
        );
        assert_eq!(retry_plan.status(), DeliveryRetryPlanStatus::Refused);
        assert_eq!(
            retry_plan.failure_class(),
            Some(DeliveryFailureClass::UnsafeDestination)
        );
    }

    #[test]
    fn receipt_hashes_safe_metadata_and_rejects_leaks() {
        // r[verify agent_harness_runtime.artifact_delivery_gateway.receipts.rejects_leaks]
        let request = request().expect("safe request should normalize");
        let policy = evaluate_policy(policy_input(&request));
        let attempt = record_delivery_attempt(DeliveryExecutionInput {
            request: &request,
            policy: &policy,
            attempt_id: ATTEMPT_ID,
            adapter_availability: DeliveryAdapterAvailability::Available,
            adapter_outcome: Some(DeliveryAdapterOutcomeDraft {
                status: DeliveryAdapterOutcomeStatus::Succeeded,
                platform_handle: Some(PLATFORM_HANDLE),
                failure_class: None,
            }),
        })
        .expect("success should record");
        let first = DeliveryReceipt::validate(&receipt(&attempt, ReceiptLeakFixture::none()))
            .expect("safe receipt should validate");
        let second = DeliveryReceipt::validate(&receipt(&attempt, ReceiptLeakFixture::none()))
            .expect("safe receipt should validate deterministically");

        assert_eq!(first.receipt_hash(), second.receipt_hash());
        assert_eq!(first.attempt(), &attempt);
        assert_receipt_leak(&attempt, ReceiptLeakFixture::raw_destination());
        assert_receipt_leak(&attempt, ReceiptLeakFixture::token());
        assert_receipt_leak(&attempt, ReceiptLeakFixture::header());
        assert_receipt_leak(&attempt, ReceiptLeakFixture::full_path());
        assert_receipt_leak(&attempt, ReceiptLeakFixture::artifact_bytes());
        assert_receipt_leak(&attempt, ReceiptLeakFixture::platform_payload());
    }

