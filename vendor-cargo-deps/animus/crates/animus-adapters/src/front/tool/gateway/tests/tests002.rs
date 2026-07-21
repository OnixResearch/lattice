    fn request() -> Result<ArtifactDeliveryRequest, DeliveryError> {
        ArtifactDeliveryRequest::normalize(&request_draft())
    }

    fn request_draft() -> ArtifactDeliveryRequestDraft<'static> {
        ArtifactDeliveryRequestDraft {
            artifact_handle: ARTIFACT_HANDLE,
            artifact_hash: VALID_HASH,
            artifact_kind: DeliveryArtifactKind::Image,
            target_kind: DeliveryTargetKind::LocalSession,
            active_context_label: Some(CONTEXT_LABEL),
            safe_artifact_label: ARTIFACT_LABEL,
            toolset_label: TOOLSET_LABEL,
            capability: DeliveryCapabilityKind::SessionLocal,
            policy_status: DeliveryPolicyStatus::Pending,
            attempt_id: Some(ATTEMPT_ID),
            retryability: DeliveryRetryability::Retryable,
            adapter_outcome: None,
            raw_artifact_bytes: None,
        }
    }

    fn policy_input(request: &ArtifactDeliveryRequest) -> PolicyInput<'_> {
        PolicyInput {
            request,
            context_status: DeliveryContextStatus::Active,
            toolset_status: DeliveryToolsetStatus::Enabled,
            raw_destination: None,
            raw_credential: None,
            raw_header: None,
            cloud_object_url: None,
            webhook_url: None,
            platform_payload: None,
        }
    }

    fn assert_policy_denied(input: PolicyInput<'_>, failure_class: DeliveryFailureClass) {
        let decision = evaluate_policy(input);
        assert_eq!(decision.status(), DeliveryPolicyStatus::Denied);
        assert_eq!(decision.failure_class(), Some(failure_class));
        assert_eq!(decision.retryability(), DeliveryRetryability::NonRetryable);
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ReceiptLeakFixture {
        raw_destination_url: Option<&'static str>,
        token: Option<&'static str>,
        header: Option<&'static str>,
        full_path: Option<&'static str>,
        artifact_bytes: Option<&'static [u8]>,
        platform_payload: Option<&'static str>,
    }

    impl ReceiptLeakFixture {
        const fn none() -> Self {
            Self {
                raw_destination_url: None,
                token: None,
                header: None,
                full_path: None,
                artifact_bytes: None,
                platform_payload: None,
            }
        }

        const fn raw_destination() -> Self {
            Self {
                raw_destination_url: Some(RAW_WEBHOOK_URL),
                ..Self::none()
            }
        }

        const fn token() -> Self {
            Self {
                token: Some(RAW_TOKEN),
                ..Self::none()
            }
        }

        const fn header() -> Self {
            Self {
                header: Some(RAW_HEADER),
                ..Self::none()
            }
        }

        const fn full_path() -> Self {
            Self {
                full_path: Some(FULL_PATH),
                ..Self::none()
            }
        }

        const fn artifact_bytes() -> Self {
            Self {
                artifact_bytes: Some(RAW_ARTIFACT_BYTES),
                ..Self::none()
            }
        }

        const fn platform_payload() -> Self {
            Self {
                platform_payload: Some(RAW_PLATFORM_PAYLOAD),
                ..Self::none()
            }
        }
    }

    fn receipt(
        attempt: &DeliveryOutboxRecord,
        fixture: ReceiptLeakFixture,
    ) -> DeliveryReceiptDraft<'_> {
        DeliveryReceiptDraft {
            attempt,
            raw_destination_url: fixture.raw_destination_url,
            token: fixture.token,
            header: fixture.header,
            full_path: fixture.full_path,
            artifact_bytes: fixture.artifact_bytes,
            platform_payload: fixture.platform_payload,
        }
    }

    fn assert_receipt_leak(attempt: &DeliveryOutboxRecord, fixture: ReceiptLeakFixture) {
        let error = DeliveryReceipt::validate(&receipt(attempt, fixture))
            .expect_err("receipt leak should be rejected");
        assert!(matches!(error, DeliveryError::ReceiptLeak { .. }));
    }
