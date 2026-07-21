    // r[verify agent_harness_runtime.claude_code_memory.descriptors.invalid]
    // r[verify agent_harness_runtime.claude_code_memory.prompt_assembly.conflict]
    // r[verify agent_harness_runtime.claude_code_memory.admission.denied]
    // r[verify agent_harness_runtime.claude_code_memory.adapter_boundary.swappable]
    // r[verify agent_harness_runtime.claude_code_memory.receipts.redacted]
    use super::*;

    const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const SECOND_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const THIRD_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const POLICY_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const DESCRIPTOR_ID: &str = "project-guidance";
    const SECOND_DESCRIPTOR_ID: &str = "learned-fact";
    const THIRD_DESCRIPTOR_ID: &str = "stale-preference";
    const SOURCE_LABEL: &str = "host-memory";
    const DECISION_LABEL: &str = "operator-reviewed";
    const RECEIPT_LABEL: &str = "memory-influence";
    const RECORD_ID: &str = "alpha-memory-record";
    const SECOND_RECORD_ID: &str = "beta-memory-record";
    const THIRD_RECORD_ID: &str = "gamma-memory-record";
    const FOURTH_RECORD_ID: &str = "omega-memory-record";
    const SUMMARY_LABEL: &str = "prefers concise answers";
    const THREAD_ID: &str = "thread-123";
    const EVIDENCE_HANDLE: &str = "evidence-handle";
    const RAW_BODY: &str = "private memory body";
    const CREDENTIAL: &str = "token-value";
    const PROVIDER_PAYLOAD: &str = "{\"provider\":\"payload\"}";
    const FULL_PATH: &str = "/home/example/project/memory.md";
    const UNBOUNDED_PROMPT: &str = "unbounded prompt text";
    const LOW_PRECEDENCE: MemoryPrecedence = MemoryPrecedence::new(10);
    const HIGH_PRECEDENCE: MemoryPrecedence = MemoryPrecedence::new(20);
    const MEMORY_UNITS: u32 = 4;
    const SMALL_MEMORY_UNITS: u32 = 2;
    const SECTION_BUDGET_UNITS: u32 = 8;
    const MEMORY_CONFIDENCE_PERCENT: u8 = 80;
    const INVALID_CONFIDENCE_PERCENT: u8 = 0;
    const EXPECTED_SINGLE_ITEM: usize = 1;
    const EXPECTED_TWO_ITEMS: usize = 2;
    const EXPECTED_THREE_ITEMS: usize = 3;
    const EXPECTED_FOUR_ITEMS: usize = 4;
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum IncludeStale {
        Yes,
        No,
    }

    impl IncludeStale {
        const fn enabled(self) -> bool {
            match self {
                Self::Yes => true,
                Self::No => false,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct DescriptorFixture {
        id: &'static str,
        body_hash: &'static str,
        precedence: MemoryPrecedence,
        freshness: MemoryFreshness,
        admission_status: MemoryAdmissionStatus,
        inclusion_policy: Option<MemoryInclusionPolicy>,
        budget_units: u32,
    }

    impl DescriptorFixture {
        const fn valid() -> Self {
            Self {
                id: DESCRIPTOR_ID,
                body_hash: VALID_HASH,
                precedence: LOW_PRECEDENCE,
                freshness: MemoryFreshness::Fresh,
                admission_status: MemoryAdmissionStatus::Admitted,
                inclusion_policy: Some(MemoryInclusionPolicy::Always),
                budget_units: MEMORY_UNITS,
            }
        }

        const fn draft(self) -> MemoryDescriptorDraft<'static> {
            MemoryDescriptorDraft {
                id: self.id,
                kind: MemoryDescriptorKind::Instruction,
                source_label: SOURCE_LABEL,
                trust_scope: Some(MemoryTrustScope::Project),
                precedence: self.precedence,
                freshness: self.freshness,
                inclusion_policy: self.inclusion_policy,
                admission_status: self.admission_status,
                redaction_class: MemoryRedactionClass::Public,
                body_hash: self.body_hash,
                budget_units: self.budget_units,
            }
        }
    }

    fn descriptor(fixture: DescriptorFixture) -> MemoryDescriptor {
        MemoryDescriptor::validate(&fixture.draft()).expect("memory descriptor should validate")
    }

    fn policy(include_stale: IncludeStale) -> MemoryPromptPolicy {
        MemoryPromptPolicy::validate(&MemoryPromptPolicyDraft {
            mode: PromptAssemblyMode::AdmittedSources,
            section_order: &[PromptSectionKind::Context],
            section_budget_units: SECTION_BUDGET_UNITS,
            policy_hash: POLICY_HASH,
            include_stale: include_stale.enabled(),
        })
        .expect("memory prompt policy should validate")
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct RecordFixture {
        id: &'static str,
        body_hash: &'static str,
        freshness: MemoryFreshness,
        redaction_status: MemoryRecordRedactionStatus,
        budget_units: u32,
    }

    impl RecordFixture {
        const fn valid() -> Self {
            Self {
                id: RECORD_ID,
                body_hash: VALID_HASH,
                freshness: MemoryFreshness::Fresh,
                redaction_status: MemoryRecordRedactionStatus::Clear,
                budget_units: MEMORY_UNITS,
            }
        }

        const fn draft(self) -> OptInMemoryRecordDraft<'static> {
            OptInMemoryRecordDraft {
                id: self.id,
                kind: OptInMemoryRecordKind::DurableFact,
                summary_label: SUMMARY_LABEL,
                scope: Some(MemoryTrustScope::Global),
                freshness: self.freshness,
                source_thread_id: THREAD_ID,
                evidence_handle: EVIDENCE_HANDLE,
                redaction_status: self.redaction_status,
                confidence_percent: MEMORY_CONFIDENCE_PERCENT,
                body_hash: self.body_hash,
                budget_units: self.budget_units,
            }
        }
    }

    fn opt_in_record(fixture: RecordFixture) -> OptInMemoryRecord {
        OptInMemoryRecord::validate(&fixture.draft()).expect("opt-in memory record should validate")
    }

    fn opt_in_policy(controls: MemoryScopeControls) -> OptInMemoryPolicy {
        opt_in_policy_with_controls(controls, controls)
    }

    fn opt_in_policy_with_controls(
        use_controls: MemoryScopeControls,
        generation_controls: MemoryScopeControls,
    ) -> OptInMemoryPolicy {
        OptInMemoryPolicy::validate(&OptInMemoryPolicyDraft {
            use_controls,
            generation_controls,
            policy_hash: POLICY_HASH,
        })
        .expect("opt-in memory policy should validate")
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct GenerationDenialFixture<'a> {
        policy: &'a OptInMemoryPolicy,
        external_context_persistable: bool,
        contains_secret_bearing_content: bool,
        redaction_succeeded: bool,
        remaining_budget_units: u32,
        reason: MemoryGenerationDenialReason,
    }

    fn assert_generation_denial(fixture: GenerationDenialFixture<'_>) {
        let decision = evaluate_memory_generation(MemoryGenerationEligibility {
            policy: fixture.policy,
            external_context_persistable: fixture.external_context_persistable,
            contains_secret_bearing_content: fixture.contains_secret_bearing_content,
            redaction_succeeded: fixture.redaction_succeeded,
            remaining_budget_units: fixture.remaining_budget_units,
        });
        assert_eq!(decision.status(), MemoryGenerationStatus::Denied);
        assert_eq!(decision.denial_reason(), Some(fixture.reason));
    }

    fn candidate() -> MemoryCandidate {
        MemoryCandidate::validate(&MemoryCandidateDraft {
            id: SECOND_DESCRIPTOR_ID,
            kind: MemoryDescriptorKind::LearnedFact,
            source_label: SOURCE_LABEL,
            redaction_class: MemoryRedactionClass::Public,
            body_hash: SECOND_HASH,
            budget_units: MEMORY_UNITS,
        })
        .expect("candidate should validate")
    }

    fn admission(status: MemoryAdmissionStatus) -> MemoryAdmissionDecision {
        MemoryAdmissionDecision::validate(&admission_draft(status))
            .expect("admission decision should validate")
    }

    fn admission_draft(status: MemoryAdmissionStatus) -> MemoryAdmissionDecisionDraft<'static> {
        MemoryAdmissionDecisionDraft {
            candidate_id: SECOND_DESCRIPTOR_ID,
            status,
            decision_label: DECISION_LABEL,
            body_hash: SECOND_HASH,
            raw_candidate_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
        }
    }

    const fn admission_context() -> MemoryAdmissionContext {
        MemoryAdmissionContext {
            trust_scope: MemoryTrustScope::Session,
            precedence: LOW_PRECEDENCE,
            freshness: MemoryFreshness::Fresh,
            inclusion_policy: MemoryInclusionPolicy::WhenRelevant,
        }
    }

    fn receipt_draft() -> MemoryInfluenceReceiptDraft<'static> {
        MemoryInfluenceReceiptDraft {
            receipt_label: RECEIPT_LABEL,
            descriptor_id: DESCRIPTOR_ID,
            source_label: SOURCE_LABEL,
            trust_scope: MemoryTrustScope::Project,
            freshness: MemoryFreshness::Fresh,
            inclusion_status: MemoryReceiptInclusionStatus::Included,
            redaction_class: MemoryRedactionClass::Public,
            body_hash: VALID_HASH,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
            unbounded_prompt_text: None,
        }
    }

    fn assert_receipt_leak(draft: MemoryInfluenceReceiptDraft<'_>) {
        assert!(matches!(
            MemoryInfluenceReceipt::validate(&draft),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }
