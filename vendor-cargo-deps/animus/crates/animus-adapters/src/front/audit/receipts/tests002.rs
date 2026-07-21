
    fn empty_work() -> [AuditWorkDraft; EXPECTED_WORK_KINDS] {
        [
            empty(AuditWorkKind::ModelCall),
            empty(AuditWorkKind::ToolCall),
            empty(AuditWorkKind::Confirmation),
            empty(AuditWorkKind::BackgroundWork),
            empty(AuditWorkKind::DeliveryAttempt),
            empty(AuditWorkKind::Retry),
        ]
    }

    const fn empty(kind: AuditWorkKind) -> AuditWorkDraft {
        AuditWorkDraft {
            kind,
            pending_count: ZERO_COUNT,
            completed_count: ZERO_COUNT,
            failed_count: ZERO_COUNT,
            timing_bucket: AuditTimingBucket::None,
        }
    }

    const fn completed_work(kind: AuditWorkKind, completed_count: u32) -> AuditWorkDraft {
        AuditWorkDraft {
            kind,
            pending_count: ZERO_COUNT,
            completed_count,
            failed_count: ZERO_COUNT,
            timing_bucket: AuditTimingBucket::Background,
        }
    }

    const fn failure(failure_class: AuditFailureClass, count: u32) -> AuditFailureDraft {
        AuditFailureDraft {
            failure_class,
            count,
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ReceiptLeakFixture {
        raw_prompt: Option<&'static str>,
        tool_arguments: Option<&'static str>,
        tool_output: Option<&'static str>,
        provider_payload: Option<&'static str>,
        credential: Option<&'static str>,
        authorization_header: Option<&'static str>,
        environment_value: Option<&'static str>,
        full_path: Option<&'static str>,
        unbounded_log: Option<&'static str>,
    }

    impl ReceiptLeakFixture {
        const fn none() -> Self {
            Self {
                raw_prompt: None,
                tool_arguments: None,
                tool_output: None,
                provider_payload: None,
                credential: None,
                authorization_header: None,
                environment_value: None,
                full_path: None,
                unbounded_log: None,
            }
        }

        const fn raw_prompt() -> Self {
            Self {
                raw_prompt: Some(RAW_PROMPT),
                ..Self::none()
            }
        }

        const fn tool_arguments() -> Self {
            Self {
                tool_arguments: Some(TOOL_ARGUMENTS),
                ..Self::none()
            }
        }

        const fn tool_output() -> Self {
            Self {
                tool_output: Some(TOOL_OUTPUT),
                ..Self::none()
            }
        }

        const fn provider_payload() -> Self {
            Self {
                provider_payload: Some(PROVIDER_PAYLOAD),
                ..Self::none()
            }
        }

        const fn credential() -> Self {
            Self {
                credential: Some(CREDENTIAL),
                ..Self::none()
            }
        }

        const fn authorization_header() -> Self {
            Self {
                authorization_header: Some(AUTHORIZATION_HEADER),
                ..Self::none()
            }
        }

        const fn environment_value() -> Self {
            Self {
                environment_value: Some(ENVIRONMENT_VALUE),
                ..Self::none()
            }
        }

        const fn full_path() -> Self {
            Self {
                full_path: Some(FULL_PATH),
                ..Self::none()
            }
        }

        const fn unbounded_log() -> Self {
            Self {
                unbounded_log: Some(UNBOUNDED_LOG),
                ..Self::none()
            }
        }
    }

    fn receipt(
        snapshot: &AuditSnapshot,
        fixture: ReceiptLeakFixture,
    ) -> AuditReceiptDraft<'_> {
        AuditReceiptDraft {
            snapshot,
            raw_prompt: fixture.raw_prompt,
            tool_arguments: fixture.tool_arguments,
            tool_output: fixture.tool_output,
            provider_payload: fixture.provider_payload,
            credential: fixture.credential,
            authorization_header: fixture.authorization_header,
            environment_value: fixture.environment_value,
            full_path: fixture.full_path,
            unbounded_log: fixture.unbounded_log,
        }
    }

    fn assert_leak(snapshot: &AuditSnapshot, fixture: ReceiptLeakFixture) {
        let error = AuditReceipt::validate(&receipt(snapshot, fixture))
            .expect_err("unsafe audit receipt should fail");
        assert!(matches!(error, AuditReceiptError::ReceiptLeak { .. }));
    }
