
    impl SourceFixture {
        const fn trusted() -> Self {
            Self {
                kind: PromptSourceKind::HostContext,
                label: HOST_LABEL,
                section: PromptSectionKind::Policy,
                hash: VALID_HASH,
                requested_units: HOST_UNITS,
                priority: LOW_PRIORITY,
                admission_status: PromptSourceAdmissionStatus::Admitted,
                redaction_class: PromptRedactionClass::Public,
            }
        }
    }

    fn source(fixture: SourceFixture) -> PromptSource {
        PromptSource::validate(&PromptSourceDraft {
            kind: fixture.kind,
            scope: PromptResourceScope::Host,
            trust_status: PromptSourceTrustStatus::Trusted,
            admission_status: fixture.admission_status,
            requirement: PromptSourceRequirement::Optional,
            priority: fixture.priority,
            budget_class: PromptBudgetClass::Normal,
            redaction_class: fixture.redaction_class,
            label: fixture.label,
            section: fixture.section,
            body_hash: fixture.hash,
            requested_units: fixture.requested_units,
        })
        .expect("prompt source should validate")
    }

    fn policy(mode: PromptAssemblyMode, section_budget_units: u32) -> PromptAssemblyPolicy {
        const SECTION_ORDER: &[PromptSectionKind] = &[
            PromptSectionKind::System,
            PromptSectionKind::Policy,
            PromptSectionKind::Context,
            PromptSectionKind::Tools,
            PromptSectionKind::Objective,
        ];
        PromptAssemblyPolicy::validate(&PromptAssemblyPolicyDraft {
            mode,
            section_order: SECTION_ORDER,
            section_budget_units,
            policy_hash: POLICY_HASH,
        })
        .expect("prompt assembly policy should validate")
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ReceiptLeakFixture {
        raw_body: Option<&'static str>,
        credential: Option<&'static str>,
        full_path: Option<&'static str>,
        provider_payload: Option<&'static str>,
        hidden_memory_body: Option<&'static str>,
    }

    impl ReceiptLeakFixture {
        const fn none() -> Self {
            Self {
                raw_body: None,
                credential: None,
                full_path: None,
                provider_payload: None,
                hidden_memory_body: None,
            }
        }
    }

    fn receipt(fixture: ReceiptLeakFixture) -> PromptAssemblyReceiptDraft<'static> {
        PromptAssemblyReceiptDraft {
            source_label: RECEIPT_LABEL,
            source_kind: PromptSourceKind::HostContext,
            trust_status: PromptSourceTrustStatus::Trusted,
            section: PromptSectionKind::Policy,
            consumed_units: HOST_UNITS,
            truncated: false,
            conflict_reason: Some(CONFLICT_REASON),
            policy_hash: POLICY_HASH,
            body_hash: VALID_HASH,
            raw_body: fixture.raw_body,
            credential: fixture.credential,
            full_path: fixture.full_path,
            provider_payload: fixture.provider_payload,
            hidden_memory_body: fixture.hidden_memory_body,
        }
    }

    fn assert_receipt_leak(fixture: ReceiptLeakFixture) {
        let error = PromptAssemblyReceipt::validate(&receipt(fixture))
            .expect_err("unsafe receipt should fail");
        assert!(matches!(error, PromptAssemblyError::ReceiptLeak { .. }));
    }
