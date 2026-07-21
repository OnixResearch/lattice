    fn control_request_draft(kind: MemoryControlCommandKind) -> MemoryControlRequestDraft<'static> {
        MemoryControlRequestDraft {
            command_label: "memory-control",
            kind,
            scope: Some(MemoryTrustScope::Project),
            target_id: Some(RECORD_ID),
            evidence_handle: Some(EVIDENCE_HANDLE),
            capability_granted: true,
            remote_admission: MemoryRemoteAdmissionStatus::NotRemote,
            freshness: MemoryFreshness::Fresh,
            safe_output_budget_units: MEMORY_UNITS,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
        }
    }

    fn mutation_receipt_draft(status: MemoryOperationStatus) -> MemoryMutationReceiptDraft<'static> {
        MemoryMutationReceiptDraft {
            operation_kind: MemoryMutationOperationKind::Forget,
            target_id: RECORD_ID,
            adapter_family_label: "store-adapter",
            status,
            policy_hash: POLICY_HASH,
            body_hash: VALID_HASH,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
        }
    }

    #[test]
    fn memory_control_commands_validate_safe_listing_and_bounded_output() {
        // r[verify agent_harness_runtime.memory_controls.commands]
        // r[verify agent_harness_runtime.memory_controls.commands.safe_listing]
        let request = MemoryControlRequest::validate(&MemoryControlRequestDraft {
            target_id: None,
            evidence_handle: None,
            ..control_request_draft(MemoryControlCommandKind::List)
        })
        .expect("list command should validate");
        assert_eq!(request.kind(), MemoryControlCommandKind::List);
        assert_eq!(request.scope(), MemoryTrustScope::Project);

        let first = generic_envelope(GenericEnvelopeFixture::valid());
        let second = generic_envelope(GenericEnvelopeFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            ..GenericEnvelopeFixture::valid()
        });
        let descriptors = project_memory_control_listing(
            &[first, second],
            MemoryInjectionBudget::new(MEMORY_UNITS).expect("budget should validate"),
        );
        assert_eq!(descriptors.len(), EXPECTED_TWO_ITEMS);
        assert!(descriptors.iter().all(|descriptor| {
            descriptor.source_label() == SOURCE_LABEL
                && descriptor.evidence_handle() == EVIDENCE_HANDLE
                && descriptor.body_hash().as_str() != RAW_BODY
                && descriptor.record_id() != FULL_PATH
        }));
    }

    #[test]
    fn memory_control_validation_denies_bad_requests_before_adapter_dispatch() {
        // r[verify agent_harness_runtime.memory_controls.validation]
        // r[verify agent_harness_runtime.memory_controls.validation.denied]
        // r[verify agent_harness_runtime.memory_controls.remote_admission]
        // r[verify agent_harness_runtime.memory_controls.remote_admission.denied]
        assert!(matches!(
            MemoryControlRequest::validate(&MemoryControlRequestDraft {
                capability_granted: false,
                ..control_request_draft(MemoryControlCommandKind::Forget)
            }),
            Err(MemoryControlError::CapabilityDenied { .. })
        ));
        assert!(matches!(
            MemoryControlRequest::validate(&MemoryControlRequestDraft {
                freshness: MemoryFreshness::Stale,
                ..control_request_draft(MemoryControlCommandKind::Update)
            }),
            Err(MemoryControlError::StaleCommand { .. })
        ));
        assert!(matches!(
            MemoryControlRequest::validate(&MemoryControlRequestDraft {
                remote_admission: MemoryRemoteAdmissionStatus::Denied,
                ..control_request_draft(MemoryControlCommandKind::Forget)
            }),
            Err(MemoryControlError::RemoteAdmission { .. })
        ));
        assert!(matches!(
            MemoryControlRequest::validate(&MemoryControlRequestDraft {
                evidence_handle: None,
                ..control_request_draft(MemoryControlCommandKind::Approve)
            }),
            Err(MemoryControlError::Memory {
                source: MemoryError::MissingEvidence { .. }
            })
        ));
        assert!(matches!(
            MemoryControlRequest::validate(&MemoryControlRequestDraft {
                raw_memory_body: Some(RAW_BODY),
                ..control_request_draft(MemoryControlCommandKind::Why)
            }),
            Err(MemoryControlError::Memory {
                source: MemoryError::ReceiptLeak { .. }
            })
        ));
    }

    #[test]
    fn candidate_review_requires_scope_and_evidence_before_persistence() {
        // r[verify agent_harness_runtime.memory_controls.review]
        // r[verify agent_harness_runtime.memory_controls.review.approval]
        let approved = transition_memory_candidate_review(MemoryCandidateReviewInput {
            candidate_id: SECOND_DESCRIPTOR_ID,
            requested_state: MemoryCandidateReviewState::Approved,
            scope: Some(MemoryTrustScope::Session),
            evidence_handle: Some(EVIDENCE_HANDLE),
            confidence_percent: MEMORY_CONFIDENCE_PERCENT,
            admission_status: MemoryAdmissionStatus::Admitted,
            reason_label: "operator-approved",
        })
        .expect("approval should validate");
        assert_eq!(approved.state(), MemoryCandidateReviewState::Approved);
        assert!(approved.persistence_allowed());
        assert_eq!(approved.scope(), Some(MemoryTrustScope::Session));
        assert_eq!(approved.evidence_handle(), Some(EVIDENCE_HANDLE));

        assert!(matches!(
            transition_memory_candidate_review(MemoryCandidateReviewInput {
                evidence_handle: None,
                ..MemoryCandidateReviewInput {
                    candidate_id: SECOND_DESCRIPTOR_ID,
                    requested_state: MemoryCandidateReviewState::Approved,
                    scope: Some(MemoryTrustScope::Session),
                    evidence_handle: Some(EVIDENCE_HANDLE),
                    confidence_percent: MEMORY_CONFIDENCE_PERCENT,
                    admission_status: MemoryAdmissionStatus::Admitted,
                    reason_label: "operator-approved",
                }
            }),
            Err(MemoryControlError::Memory {
                source: MemoryError::MissingEvidence { .. }
            })
        ));
        let rejected = transition_memory_candidate_review(MemoryCandidateReviewInput {
            candidate_id: SECOND_DESCRIPTOR_ID,
            requested_state: MemoryCandidateReviewState::Rejected,
            scope: None,
            evidence_handle: None,
            confidence_percent: MEMORY_CONFIDENCE_PERCENT,
            admission_status: MemoryAdmissionStatus::Denied,
            reason_label: "operator-rejected",
        })
        .expect("rejection should not require persistence metadata");
        assert_eq!(rejected.state(), MemoryCandidateReviewState::Rejected);
        assert!(!rejected.persistence_allowed());
    }

    #[test]
    fn forget_update_receipts_do_not_claim_success_for_unavailable_or_partial_backends() {
        // r[verify agent_harness_runtime.memory_controls.forget_update]
        // r[verify agent_harness_runtime.memory_controls.forget_update.unavailable]
        let unavailable = MemoryMutationReceipt::validate(&mutation_receipt_draft(
            MemoryOperationStatus::Unavailable,
        ))
        .expect("unavailable receipt should validate");
        assert_eq!(unavailable.status(), MemoryOperationStatus::Unavailable);
        assert!(!unavailable.deletion_claimed());

        let partial = MemoryMutationReceipt::validate(&mutation_receipt_draft(
            MemoryOperationStatus::Partial,
        ))
        .expect("partial receipt should validate");
        assert_eq!(partial.status(), MemoryOperationStatus::Partial);
        assert!(!partial.deletion_claimed());

        let complete = MemoryMutationReceipt::validate(&mutation_receipt_draft(
            MemoryOperationStatus::Complete,
        ))
        .expect("complete delete receipt should validate");
        assert!(complete.deletion_claimed());
        assert!(matches!(
            MemoryMutationReceipt::validate(&MemoryMutationReceiptDraft {
                full_path: Some(FULL_PATH),
                ..mutation_receipt_draft(MemoryOperationStatus::Complete)
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    #[test]
    fn why_explanations_are_safe_and_derived_from_influence_ledgers() {
        // r[verify agent_harness_runtime.memory_controls.influence_explain]
        // r[verify agent_harness_runtime.memory_controls.influence_explain.safe]
        let included = generic_envelope(GenericEnvelopeFixture::valid());
        let skipped = generic_envelope(GenericEnvelopeFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            scope: Some(MemoryTrustScope::Host),
            ..GenericEnvelopeFixture::valid()
        });
        let plan = plan_memory_retrieval(MemoryRetrievalInput {
            policy: MemoryRetrievalPolicy {
                query_label: "why-query",
                allowed_scopes: &[MemoryTrustScope::Project],
                include_stale: false,
                budget: MemoryInjectionBudget::new(MEMORY_UNITS).expect("budget should validate"),
            },
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &skipped,
                    match_reason: MemoryMatchReason::Recent,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "history-backend",
                },
                MemoryRetrievalCandidate {
                    record: &included,
                    match_reason: MemoryMatchReason::ExactScope,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "history-backend",
                },
            ],
        })
        .expect("retrieval plan should validate");
        let explanation = explain_memory_influence(&plan);
        assert_eq!(explanation.entries().len(), EXPECTED_TWO_ITEMS);
        assert!(explanation.entries().iter().any(|entry| {
            entry.record_id() == RECORD_ID
                && entry.status() == MemoryRetrievalOutcomeStatus::Selected
                && entry.body_hash().as_str() == VALID_HASH
        }));
        assert!(explanation.entries().iter().any(|entry| {
            entry.record_id() == SECOND_RECORD_ID
                && entry.status() == MemoryRetrievalOutcomeStatus::SkippedScope
                && entry.source_label() != RAW_BODY
        }));
    }
