    fn conformance_row_draft() -> MemoryConformanceMatrixRowDraft<'static> {
        MemoryConformanceMatrixRowDraft {
            backend_family: MemoryBackendFamily::VectorHybrid,
            adapter_role: GenericMemoryAdapterRole::RetrievalSearch,
            capabilities: &[GenericMemoryAdapterCapability::RetrievalSearch],
            owner_label: "runtime-adapters",
            applicability: MemoryConformanceApplicability::Applicable,
            positive_case: Some("normalized-records-plan-equivalently"),
            negative_case: Some("unavailable-vector-has-no-fallback"),
            safe_evidence_label: Some("memory-conformance-vector"),
        }
    }

    fn remote_failure_draft(status: MemoryOperationStatus) -> MemoryRemoteBackendFailureDraft<'static> {
        MemoryRemoteBackendFailureDraft {
            service_label: "remote-memory-service",
            remote_id_label: "remote-record-safe-id",
            status,
            credential: None,
            provider_payload: None,
            transport_envelope: None,
            unbounded_remote_output: None,
        }
    }

    #[test]
    fn conformance_matrix_rows_require_positive_and_negative_safe_evidence() {
        // r[verify agent_harness_runtime.memory_conformance.matrix]
        // r[verify agent_harness_runtime.memory_conformance.matrix.coverage]
        let row = MemoryConformanceMatrixRow::validate(&conformance_row_draft())
            .expect("complete row should validate");
        assert_eq!(row.backend_family(), MemoryBackendFamily::VectorHybrid);
        assert_eq!(row.adapter_role(), GenericMemoryAdapterRole::RetrievalSearch);
        assert_eq!(row.owner_label(), "runtime-adapters");
        assert_eq!(
            row.negative_case(),
            Some("unavailable-vector-has-no-fallback")
        );

        assert!(matches!(
            MemoryConformanceMatrixRow::validate(&MemoryConformanceMatrixRowDraft {
                capabilities: &[],
                ..conformance_row_draft()
            }),
            Err(MemoryConformanceError::MissingCapability {
                family: MemoryBackendFamily::VectorHybrid,
            })
        ));
        assert!(matches!(
            MemoryConformanceMatrixRow::validate(&MemoryConformanceMatrixRowDraft {
                negative_case: None,
                ..conformance_row_draft()
            }),
            Err(MemoryConformanceError::MissingNegativeCase {
                family: MemoryBackendFamily::VectorHybrid,
            })
        ));
        let not_applicable = MemoryConformanceMatrixRow::validate(&MemoryConformanceMatrixRowDraft {
            backend_family: MemoryBackendFamily::SqlDocument,
            adapter_role: GenericMemoryAdapterRole::EmbeddingSearchSupport,
            capabilities: &[GenericMemoryAdapterCapability::EmbeddingSearch],
            owner_label: "runtime-adapters",
            applicability: MemoryConformanceApplicability::NotApplicable {
                reason: "sql-row-covered-by-document-store",
            },
            positive_case: None,
            negative_case: None,
            safe_evidence_label: None,
        })
        .expect("not-applicable row with reason should validate");
        assert_eq!(
            not_applicable.not_applicable_reason(),
            Some("sql-row-covered-by-document-store")
        );
    }

    #[test]
    fn null_memory_backend_performs_no_side_effects_and_keeps_rules_active() {
        // r[verify agent_harness_runtime.memory_conformance.null_backend]
        // r[verify agent_harness_runtime.memory_conformance.null_backend.disabled]
        let plan = evaluate_null_memory_backend(MemoryNullBackendInput {
            memory_use: MemoryNullBackendToggle::Disabled,
            memory_generation: MemoryNullBackendToggle::Disabled,
            guidance: MemoryRequiredRulePresence::Present,
            policy: MemoryRequiredRulePresence::Absent,
        });
        assert_eq!(plan.status(), MemoryOperationStatus::Unsupported);
        assert!(!plan.storage_planned());
        assert!(!plan.retrieval_planned());
        assert!(!plan.prompt_injection_planned());
        assert!(!plan.extraction_planned());
        assert!(!plan.mutation_planned());
        assert!(plan.required_rules_active());

        let unavailable = evaluate_null_memory_backend(MemoryNullBackendInput {
            memory_use: MemoryNullBackendToggle::Enabled,
            memory_generation: MemoryNullBackendToggle::Enabled,
            guidance: MemoryRequiredRulePresence::Absent,
            policy: MemoryRequiredRulePresence::Absent,
        });
        assert_eq!(unavailable.status(), MemoryOperationStatus::Unavailable);
        assert!(!unavailable.prompt_injection_planned());
    }

    #[test]
    fn file_backed_memory_evidence_redacts_paths_and_raw_bodies() {
        // r[verify agent_harness_runtime.memory_conformance.filesystem_backend]
        // r[verify agent_harness_runtime.memory_conformance.filesystem_backend.path_redaction]
        let evidence = MemoryFilesystemEvidence::validate(&MemoryFilesystemEvidenceDraft {
            source_label: SOURCE_LABEL,
            body_hash: VALID_HASH,
            full_path: None,
            raw_memory_body: None,
        })
        .expect("safe filesystem evidence should validate");
        assert_eq!(evidence.source_label(), SOURCE_LABEL);
        assert_eq!(evidence.body_hash().as_str(), VALID_HASH);
        assert!(matches!(
            MemoryFilesystemEvidence::validate(&MemoryFilesystemEvidenceDraft {
                source_label: SOURCE_LABEL,
                body_hash: VALID_HASH,
                full_path: Some(FULL_PATH),
                raw_memory_body: None,
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryFilesystemEvidence::validate(&MemoryFilesystemEvidenceDraft {
                source_label: SOURCE_LABEL,
                body_hash: VALID_HASH,
                full_path: None,
                raw_memory_body: Some(RAW_BODY),
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    #[test]
    fn semantic_backend_unavailable_has_no_ambient_fallback() {
        // r[verify agent_harness_runtime.memory_conformance.semantic_backend]
        // r[verify agent_harness_runtime.memory_conformance.semantic_backend.unavailable]
        let unavailable = plan_semantic_backend_fallback(MemorySemanticBackendFallbackInput {
            status: MemoryOperationStatus::Unavailable,
            explicit_alternate_adapter: false,
        });
        assert_eq!(unavailable.status(), MemoryOperationStatus::Unavailable);
        assert!(!unavailable.ambient_fallback_attempted());
        assert!(!unavailable.explicit_alternate_attempted());

        let explicit = plan_semantic_backend_fallback(MemorySemanticBackendFallbackInput {
            status: MemoryOperationStatus::Unavailable,
            explicit_alternate_adapter: true,
        });
        assert!(!explicit.ambient_fallback_attempted());
        assert!(explicit.explicit_alternate_attempted());
    }

    #[test]
    fn remote_memory_failures_are_safe_and_do_not_claim_success() {
        // r[verify agent_harness_runtime.memory_conformance.remote_backend]
        // r[verify agent_harness_runtime.memory_conformance.remote_backend.failures]
        let denied = MemoryRemoteBackendFailure::validate(&remote_failure_draft(
            MemoryOperationStatus::Denied,
        ))
        .expect("denied remote failure should validate");
        assert_eq!(denied.status(), MemoryOperationStatus::Denied);
        assert_eq!(denied.service_label(), "remote-memory-service");
        assert_eq!(denied.remote_id_label(), "remote-record-safe-id");

        assert!(matches!(
            MemoryRemoteBackendFailure::validate(&remote_failure_draft(
                MemoryOperationStatus::Complete,
            )),
            Err(MemoryConformanceError::UnsafeSuccessClaim {
                family: MemoryBackendFamily::McpRemote,
            })
        ));
        assert!(matches!(
            MemoryRemoteBackendFailure::validate(&MemoryRemoteBackendFailureDraft {
                credential: Some(CREDENTIAL),
                ..remote_failure_draft(MemoryOperationStatus::TimedOut)
            }),
            Err(MemoryConformanceError::Memory {
                source: MemoryError::ReceiptLeak { .. }
            })
        ));
        assert!(matches!(
            MemoryRemoteBackendFailure::validate(&MemoryRemoteBackendFailureDraft {
                provider_payload: Some(PROVIDER_PAYLOAD),
                ..remote_failure_draft(MemoryOperationStatus::Unavailable)
            }),
            Err(MemoryConformanceError::Memory {
                source: MemoryError::ReceiptLeak { .. }
            })
        ));
    }

    #[test]
    fn memory_failure_taxonomy_is_stable_across_backends() {
        // r[verify agent_harness_runtime.memory_conformance.failure_taxonomy]
        // r[verify agent_harness_runtime.memory_conformance.failure_taxonomy.stable]
        assert_eq!(
            memory_failure_for_retrieval_status(MemoryRetrievalOutcomeStatus::OverBudget),
            Some(MemoryFailureClass::OverBudget)
        );
        assert_eq!(
            memory_failure_for_retrieval_status(MemoryRetrievalOutcomeStatus::Conflicting),
            Some(MemoryFailureClass::Conflicting)
        );
        assert_eq!(
            memory_failure_for_retrieval_status(MemoryRetrievalOutcomeStatus::Redacted),
            Some(MemoryFailureClass::Redacted)
        );
        assert_eq!(
            memory_failure_for_operation_status(MemoryOperationStatus::Unavailable),
            Some(MemoryFailureClass::Unavailable)
        );
        assert_eq!(
            memory_failure_for_operation_status(MemoryOperationStatus::TimedOut),
            Some(MemoryFailureClass::TimedOut)
        );
        assert_eq!(
            memory_failure_for_operation_status(MemoryOperationStatus::Complete),
            None
        );
    }
