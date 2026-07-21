    #[test]
    fn retrieval_planning_marks_conflicts_redaction_and_budget() {
        // r[verify agent_harness_runtime.memory_adapters.retrieval_plan]
        let selected = generic_envelope(GenericEnvelopeFixture {
            conflict_group: Some("project-preference"),
            ..GenericEnvelopeFixture::valid()
        });
        let conflicting = generic_envelope(GenericEnvelopeFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            conflict_group: Some("project-preference"),
            ..GenericEnvelopeFixture::valid()
        });
        let redacted = generic_envelope(GenericEnvelopeFixture {
            id: THIRD_RECORD_ID,
            body_hash: THIRD_HASH,
            redaction_class: MemoryRedactionClass::Secret,
            ..GenericEnvelopeFixture::valid()
        });
        let over_budget = generic_envelope(GenericEnvelopeFixture {
            id: FOURTH_RECORD_ID,
            body_hash: SECOND_HASH,
            budget_units: MEMORY_UNITS,
            ..GenericEnvelopeFixture::valid()
        });
        let plan = plan_memory_retrieval(MemoryRetrievalInput {
            policy: generic_retrieval_policy(),
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &over_budget,
                    match_reason: MemoryMatchReason::Semantic,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "vector-backend",
                },
                MemoryRetrievalCandidate {
                    record: &redacted,
                    match_reason: MemoryMatchReason::Keyword,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "file-backend",
                },
                MemoryRetrievalCandidate {
                    record: &conflicting,
                    match_reason: MemoryMatchReason::ExactScope,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "sql-backend",
                },
                MemoryRetrievalCandidate {
                    record: &selected,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "mcp-backend",
                },
            ],
        })
        .expect("retrieval planning should succeed");
        assert_eq!(plan.outcomes().len(), EXPECTED_FOUR_ITEMS);
        assert_eq!(plan.selected_count(), EXPECTED_SINGLE_ITEM_U32);
        assert_retrieval_status(&plan, RECORD_ID, MemoryRetrievalOutcomeStatus::Selected);
        assert_retrieval_status(
            &plan,
            SECOND_RECORD_ID,
            MemoryRetrievalOutcomeStatus::Conflicting,
        );
        assert_retrieval_status(&plan, THIRD_RECORD_ID, MemoryRetrievalOutcomeStatus::Redacted);
        assert_retrieval_status(
            &plan,
            FOURTH_RECORD_ID,
            MemoryRetrievalOutcomeStatus::OverBudget,
        );
    }

    #[test]
    fn extraction_admission_and_operation_receipts_are_separate_and_redacted() {
        // r[verify agent_harness_runtime.memory_adapters.extraction_admission]
        // r[verify agent_harness_runtime.memory_adapters.extraction_admission.denied]
        // r[verify agent_harness_runtime.memory_adapters.receipts]
        // r[verify agent_harness_runtime.memory_adapters.receipts.rejects_leaks]
        let candidate = candidate();
        let denied = admission(MemoryAdmissionStatus::Denied);
        let effect = resolve_memory_admission(&candidate, &denied, admission_context())
            .expect("denied admission should resolve");
        assert!(effect.descriptor().is_none());
        assert_eq!(
            effect.persistence(),
            MemoryPersistenceIntent::DoNotPersist
        );

        let receipt = MemoryOperationReceipt::validate(&generic_receipt_draft())
            .expect("safe operation receipt should validate");
        assert_eq!(receipt.operation_kind(), MemoryOperationKind::Retrieval);
        assert_eq!(receipt.record_id(), Some(RECORD_ID));
        assert_eq!(receipt.item_count(), EXPECTED_SINGLE_ITEM_U32);
        assert!(matches!(
            MemoryOperationReceipt::validate(&MemoryOperationReceiptDraft {
                raw_memory_body: Some(RAW_BODY),
                ..generic_receipt_draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryOperationReceipt::validate(&MemoryOperationReceiptDraft {
                credential: Some(CREDENTIAL),
                ..generic_receipt_draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryOperationReceipt::validate(&MemoryOperationReceiptDraft {
                full_path: Some(FULL_PATH),
                ..generic_receipt_draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryOperationReceipt::validate(&MemoryOperationReceiptDraft {
                provider_payload: Some(PROVIDER_PAYLOAD),
                ..generic_receipt_draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryOperationReceipt::validate(&MemoryOperationReceiptDraft {
                unbounded_prompt_text: Some(UNBOUNDED_PROMPT),
                ..generic_receipt_draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }
