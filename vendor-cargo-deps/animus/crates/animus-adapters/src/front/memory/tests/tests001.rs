    #[test]
    fn descriptors_validate_required_scope_policy_and_body_metadata() {
        let descriptor = descriptor(DescriptorFixture::valid());
        assert_eq!(descriptor.id(), DESCRIPTOR_ID);
        assert_eq!(descriptor.source_label(), SOURCE_LABEL);
        assert_eq!(descriptor.trust_scope(), MemoryTrustScope::Project);
        assert_eq!(descriptor.body_hash().as_str(), VALID_HASH);

        assert!(matches!(
            MemoryDescriptor::validate(&MemoryDescriptorDraft {
                source_label: "   ",
                ..DescriptorFixture::valid().draft()
            }),
            Err(MemoryError::Contract(_))
        ));
        assert_eq!(
            MemoryDescriptor::validate(&MemoryDescriptorDraft {
                trust_scope: None,
                ..DescriptorFixture::valid().draft()
            }),
            Err(MemoryError::MissingTrustScope {
                descriptor_id: String::from(DESCRIPTOR_ID),
            })
        );
        assert_eq!(
            MemoryDescriptor::validate(&MemoryDescriptorDraft {
                inclusion_policy: None,
                ..DescriptorFixture::valid().draft()
            }),
            Err(MemoryError::MissingInclusionPolicy {
                descriptor_id: String::from(DESCRIPTOR_ID),
            })
        );
        assert_eq!(
            MemoryDescriptor::validate(&MemoryDescriptorDraft {
                freshness: MemoryFreshness::Expired,
                ..DescriptorFixture::valid().draft()
            }),
            Err(MemoryError::ExpiredDescriptor {
                descriptor_id: String::from(DESCRIPTOR_ID),
            })
        );
        assert!(matches!(
            MemoryDescriptor::validate(&MemoryDescriptorDraft {
                body_hash: "not-a-hash",
                ..DescriptorFixture::valid().draft()
            }),
            Err(MemoryError::InvalidHash(_))
        ));
    }

    #[test]
    fn memory_prompt_assembly_is_deterministic_and_conflict_safe() {
        let low = descriptor(DescriptorFixture::valid());
        let conflicting = descriptor(DescriptorFixture {
            id: SECOND_DESCRIPTOR_ID,
            body_hash: SECOND_HASH,
            ..DescriptorFixture::valid()
        });
        let stale = descriptor(DescriptorFixture {
            id: THIRD_DESCRIPTOR_ID,
            body_hash: THIRD_HASH,
            precedence: HIGH_PRECEDENCE,
            freshness: MemoryFreshness::Stale,
            budget_units: SMALL_MEMORY_UNITS,
            ..DescriptorFixture::valid()
        });
        let assembly =
            assemble_memory_prompt(&[conflicting, stale, low], &policy(IncludeStale::No))
                .expect("memory assembly should succeed");

        assert_eq!(assembly.sections().len(), EXPECTED_SINGLE_ITEM);
        assert_eq!(assembly.sections()[0].source_label(), SECOND_DESCRIPTOR_ID);
        assert_eq!(assembly.ledger().len(), EXPECTED_THREE_ITEMS);
        assert!(assembly.ledger().iter().any(|entry| {
            entry.source_label() == DESCRIPTOR_ID
                && entry.state()
                    == super::super::prompt_assembly::PromptAssemblyLedgerState::Conflicting
        }));
        assert!(assembly.ledger().iter().any(|entry| {
            entry.source_label() == THIRD_DESCRIPTOR_ID
                && entry.state() == super::super::prompt_assembly::PromptAssemblyLedgerState::Denied
        }));
    }

    #[test]
    fn admitted_sources_mode_preserves_backend_swap_and_missing_adapter_outcomes() {
        let memory = descriptor(DescriptorFixture::valid());
        let unavailable = descriptor(DescriptorFixture {
            id: SECOND_DESCRIPTOR_ID,
            body_hash: SECOND_HASH,
            admission_status: MemoryAdmissionStatus::Unavailable,
            precedence: HIGH_PRECEDENCE,
            ..DescriptorFixture::valid()
        });
        let first = assemble_memory_prompt(
            &[memory.clone(), unavailable.clone()],
            &policy(IncludeStale::Yes),
        )
        .expect("first backend DTO assembly should succeed");
        let second = assemble_memory_prompt(&[memory, unavailable], &policy(IncludeStale::Yes))
            .expect("second backend DTO assembly should succeed");

        assert_eq!(first, second);
        assert_eq!(first.sections().len(), EXPECTED_SINGLE_ITEM);
        assert_eq!(first.ledger().len(), EXPECTED_TWO_ITEMS);
        assert!(first.ledger().iter().any(|entry| {
            entry.source_label() == SECOND_DESCRIPTOR_ID
                && entry.state() == super::super::prompt_assembly::PromptAssemblyLedgerState::Unsupported
                && entry.reason()
                    == Some(
                        super::super::prompt_assembly::PromptAssemblyLedgerReason::RequiredSourceUnavailable,
                    )
        }));
    }

    #[test]
    fn denied_or_unavailable_admission_prevents_persistence_and_prompt_influence() {
        let candidate = candidate();
        let denied = admission(MemoryAdmissionStatus::Denied);
        let denied_effect = resolve_memory_admission(&candidate, &denied, admission_context())
            .expect("denied admission should resolve");
        assert!(denied_effect.descriptor().is_none());
        assert_eq!(
            denied_effect.persistence(),
            MemoryPersistenceIntent::DoNotPersist
        );

        let unavailable = admission(MemoryAdmissionStatus::Unavailable);
        let unavailable_effect =
            resolve_memory_admission(&candidate, &unavailable, admission_context())
                .expect("unavailable admission should resolve");
        assert!(unavailable_effect.descriptor().is_none());
        assert_eq!(
            unavailable_effect.persistence(),
            MemoryPersistenceIntent::DoNotPersist
        );

        let admitted = admission(MemoryAdmissionStatus::Admitted);
        let admitted_effect = resolve_memory_admission(&candidate, &admitted, admission_context())
            .expect("admitted memory should resolve");
        assert!(admitted_effect.descriptor().is_some());
        assert_eq!(
            admitted_effect.persistence(),
            MemoryPersistenceIntent::Persist
        );
    }

    #[test]
    fn admission_and_influence_receipts_reject_sensitive_payloads() {
        assert!(
            MemoryAdmissionDecision::validate(&MemoryAdmissionDecisionDraft {
                raw_candidate_body: Some(RAW_BODY),
                ..admission_draft(MemoryAdmissionStatus::Admitted)
            })
            .is_err()
        );
        let receipt = MemoryInfluenceReceipt::validate(&receipt_draft())
            .expect("safe memory influence receipt should validate");
        assert_eq!(receipt.receipt_label(), RECEIPT_LABEL);
        assert_eq!(receipt.descriptor_id(), DESCRIPTOR_ID);
        assert_eq!(receipt.body_hash().as_str(), VALID_HASH);

        assert_receipt_leak(MemoryInfluenceReceiptDraft {
            raw_memory_body: Some(RAW_BODY),
            ..receipt_draft()
        });
        assert_receipt_leak(MemoryInfluenceReceiptDraft {
            credential: Some(CREDENTIAL),
            ..receipt_draft()
        });
        assert_receipt_leak(MemoryInfluenceReceiptDraft {
            provider_payload: Some(PROVIDER_PAYLOAD),
            ..receipt_draft()
        });
        assert_receipt_leak(MemoryInfluenceReceiptDraft {
            full_path: Some(FULL_PATH),
            ..receipt_draft()
        });
        assert_receipt_leak(MemoryInfluenceReceiptDraft {
            unbounded_prompt_text: Some(UNBOUNDED_PROMPT),
            ..receipt_draft()
        });
    }

