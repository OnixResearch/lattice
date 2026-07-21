    // r[verify agent_harness_runtime.prompt_assembly.verification]
    // r[verify agent_harness_runtime.prompt_assembly.verification.closeout]
    use super::*;

    const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SECOND_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const THIRD_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const POLICY_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const HOST_LABEL: &str = "host-objective";
    const MEMORY_LABEL: &str = "memory-context";
    const ALPHA_LABEL: &str = "alpha-policy";
    const BETA_LABEL: &str = "beta-policy";
    const SECRET_LABEL: &str = "secret-policy";
    const RECEIPT_LABEL: &str = "receipt-source";
    const CONFLICT_REASON: &str = "same-section-priority";
    const RAW_BODY: &str = "raw prompt body";
    const CREDENTIAL: &str = "token-value";
    const FULL_PATH: &str = "/home/example/project/secret.md";
    const PROVIDER_PAYLOAD: &str = "{\"provider\":\"payload\"}";
    const HIDDEN_MEMORY: &str = "private memory body";
    const HOST_UNITS: u32 = 4;
    const MEMORY_UNITS: u32 = 3;
    const LARGE_UNITS: u32 = 12;
    const SECTION_BUDGET_UNITS: u32 = 8;
    const SMALL_SECTION_BUDGET_UNITS: u32 = 5;
    const LOW_PRIORITY: PromptSourcePriority = PromptSourcePriority::new(10);
    const HIGH_PRIORITY: PromptSourcePriority = PromptSourcePriority::new(20);
    const EXPECTED_SINGLE_ITEM: usize = 1;
    const EXPECTED_TWO_ITEMS: usize = 2;
    const EXPECTED_THREE_ITEMS: usize = 3;

    #[test]
    fn host_context_only_assembles_admitted_context_without_discovery() {
        // r[verify agent_harness_runtime.prompt_assembly.sources.host_context]
        // r[verify agent_harness_runtime.prompt_assembly.discovery.missing]
        let host = source(SourceFixture {
            kind: PromptSourceKind::HostContext,
            label: HOST_LABEL,
            section: PromptSectionKind::Objective,
            hash: VALID_HASH,
            requested_units: HOST_UNITS,
            priority: LOW_PRIORITY,
            ..SourceFixture::trusted()
        });
        let memory = source(SourceFixture {
            kind: PromptSourceKind::Memory,
            label: MEMORY_LABEL,
            section: PromptSectionKind::Context,
            hash: SECOND_HASH,
            requested_units: MEMORY_UNITS,
            priority: LOW_PRIORITY,
            ..SourceFixture::trusted()
        });
        let assembly = assemble_prompt_sources(
            &[memory, host],
            &policy(PromptAssemblyMode::HostContextOnly, SECTION_BUDGET_UNITS),
        )
        .expect("host-context-only assembly should succeed");

        assert_eq!(assembly.sections().len(), EXPECTED_SINGLE_ITEM);
        assert_eq!(assembly.sections()[0].source_label(), HOST_LABEL);
        assert_eq!(assembly.ledger().len(), EXPECTED_TWO_ITEMS);
        assert!(assembly.ledger().iter().any(|entry| {
            entry.source_label() == MEMORY_LABEL
                && entry.state() == PromptAssemblyLedgerState::Unsupported
                && entry.reason() == Some(PromptAssemblyLedgerReason::DiscoveryUnavailable)
        }));
    }

    #[test]
    fn conflicting_sources_resolve_in_deterministic_order() {
        // r[verify agent_harness_runtime.prompt_assembly.precedence.conflict]
        let beta = source(SourceFixture {
            kind: PromptSourceKind::PolicyProfile,
            label: BETA_LABEL,
            section: PromptSectionKind::Policy,
            hash: SECOND_HASH,
            requested_units: HOST_UNITS,
            priority: LOW_PRIORITY,
            ..SourceFixture::trusted()
        });
        let alpha = source(SourceFixture {
            kind: PromptSourceKind::PolicyProfile,
            label: ALPHA_LABEL,
            section: PromptSectionKind::Policy,
            hash: VALID_HASH,
            requested_units: HOST_UNITS,
            priority: LOW_PRIORITY,
            ..SourceFixture::trusted()
        });
        let assembly = assemble_prompt_sources(
            &[beta, alpha],
            &policy(PromptAssemblyMode::AdmittedSources, SECTION_BUDGET_UNITS),
        )
        .expect("conflict assembly should succeed");

        assert_eq!(assembly.sections().len(), EXPECTED_SINGLE_ITEM);
        assert_eq!(assembly.sections()[0].source_label(), ALPHA_LABEL);
        assert_eq!(assembly.ledger()[0].source_label(), ALPHA_LABEL);
        assert_eq!(
            assembly.ledger()[0].state(),
            PromptAssemblyLedgerState::Included
        );
        assert_eq!(assembly.ledger()[1].source_label(), BETA_LABEL);
        assert_eq!(
            assembly.ledger()[1].state(),
            PromptAssemblyLedgerState::Conflicting
        );
        assert_eq!(
            assembly.ledger()[1].reason(),
            Some(PromptAssemblyLedgerReason::SourceConflict)
        );
    }

    #[test]
    fn over_budget_source_truncates_with_bounded_identity() {
        // r[verify agent_harness_runtime.prompt_assembly.ledger.truncated]
        let large = source(SourceFixture {
            kind: PromptSourceKind::HostContext,
            label: HOST_LABEL,
            section: PromptSectionKind::Context,
            hash: THIRD_HASH,
            requested_units: LARGE_UNITS,
            priority: HIGH_PRIORITY,
            ..SourceFixture::trusted()
        });
        let assembly = assemble_prompt_sources(
            &[large],
            &policy(
                PromptAssemblyMode::AdmittedSources,
                SMALL_SECTION_BUDGET_UNITS,
            ),
        )
        .expect("truncated assembly should succeed");

        assert_eq!(assembly.sections().len(), EXPECTED_SINGLE_ITEM);
        assert!(assembly.sections()[0].truncated());
        assert_eq!(assembly.sections()[0].body_hash().as_str(), THIRD_HASH);
        assert_eq!(
            assembly.ledger()[0].state(),
            PromptAssemblyLedgerState::Truncated
        );
        assert_eq!(
            assembly.ledger()[0].reason(),
            Some(PromptAssemblyLedgerReason::OverBudget)
        );
        assert_eq!(
            assembly.ledger()[0].consumed_units(),
            SMALL_SECTION_BUDGET_UNITS
        );
    }

    #[test]
    fn denied_redacted_and_unsupported_sources_are_ledgered_safely() {
        let denied = source(SourceFixture {
            label: BETA_LABEL,
            admission_status: PromptSourceAdmissionStatus::Denied,
            hash: SECOND_HASH,
            ..SourceFixture::trusted()
        });
        let redacted = source(SourceFixture {
            label: SECRET_LABEL,
            redaction_class: PromptRedactionClass::Secret,
            hash: THIRD_HASH,
            priority: HIGH_PRIORITY,
            ..SourceFixture::trusted()
        });
        let unsupported = source(SourceFixture {
            kind: PromptSourceKind::Unsupported,
            label: MEMORY_LABEL,
            hash: VALID_HASH,
            priority: PromptSourcePriority::new(30),
            ..SourceFixture::trusted()
        });
        let assembly = assemble_prompt_sources(
            &[denied, redacted, unsupported],
            &policy(PromptAssemblyMode::AdmittedSources, SECTION_BUDGET_UNITS),
        )
        .expect("non-visible sources should ledger safely");

        assert!(assembly.sections().is_empty());
        assert_eq!(assembly.ledger().len(), EXPECTED_THREE_ITEMS);
        assert!(
            assembly
                .ledger()
                .iter()
                .any(|entry| entry.state() == PromptAssemblyLedgerState::Denied)
        );
        assert!(
            assembly
                .ledger()
                .iter()
                .any(|entry| entry.state() == PromptAssemblyLedgerState::Redacted)
        );
        assert!(
            assembly
                .ledger()
                .iter()
                .any(|entry| entry.state() == PromptAssemblyLedgerState::Unsupported)
        );
    }

    #[test]
    fn receipt_validation_rejects_sensitive_prompt_material() {
        // r[verify agent_harness_runtime.prompt_assembly.receipts.rejects_leaks]
        let valid = PromptAssemblyReceipt::validate(&receipt(ReceiptLeakFixture::none()))
            .expect("safe receipt should validate");
        assert_eq!(valid.source_label(), RECEIPT_LABEL);
        assert_eq!(valid.policy_hash().as_str(), POLICY_HASH);
        assert_eq!(valid.body_hash().as_str(), VALID_HASH);
        assert_eq!(valid.conflict_reason(), Some(CONFLICT_REASON));

        assert_receipt_leak(ReceiptLeakFixture {
            raw_body: Some(RAW_BODY),
            ..ReceiptLeakFixture::none()
        });
        assert_receipt_leak(ReceiptLeakFixture {
            credential: Some(CREDENTIAL),
            ..ReceiptLeakFixture::none()
        });
        assert_receipt_leak(ReceiptLeakFixture {
            full_path: Some(FULL_PATH),
            ..ReceiptLeakFixture::none()
        });
        assert_receipt_leak(ReceiptLeakFixture {
            provider_payload: Some(PROVIDER_PAYLOAD),
            ..ReceiptLeakFixture::none()
        });
        assert_receipt_leak(ReceiptLeakFixture {
            hidden_memory_body: Some(HIDDEN_MEMORY),
            ..ReceiptLeakFixture::none()
        });
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct SourceFixture {
        kind: PromptSourceKind,
        label: &'static str,
        section: PromptSectionKind,
        hash: &'static str,
        requested_units: u32,
        priority: PromptSourcePriority,
        admission_status: PromptSourceAdmissionStatus,
        redaction_class: PromptRedactionClass,
    }
