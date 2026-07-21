    const EXPECTED_SINGLE_ITEM_U32: u32 = 1;
    const EXPECTED_TWO_ITEMS_U32: u32 = 2;
    const EXPECTED_FOUR_ITEMS_U32: u32 = 4;
    const EXPECTED_FULL_PERCENT: u32 = 100;
    const HIGH_RERANK_PERCENT: u8 = 90;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct GenericEnvelopeFixture {
        id: &'static str,
        body_hash: &'static str,
        scope: Option<MemoryTrustScope>,
        freshness: MemoryFreshness,
        admission_status: MemoryAdmissionStatus,
        redaction_class: MemoryRedactionClass,
        confidence_percent: u8,
        budget_units: u32,
        conflict_group: Option<&'static str>,
        superseded_by_id: Option<&'static str>,
    }

    impl GenericEnvelopeFixture {
        const fn valid() -> Self {
            Self {
                id: RECORD_ID,
                body_hash: VALID_HASH,
                scope: Some(MemoryTrustScope::Project),
                freshness: MemoryFreshness::Fresh,
                admission_status: MemoryAdmissionStatus::Admitted,
                redaction_class: MemoryRedactionClass::Public,
                confidence_percent: MEMORY_CONFIDENCE_PERCENT,
                budget_units: SMALL_MEMORY_UNITS,
                conflict_group: None,
                superseded_by_id: None,
            }
        }

        const fn draft(self) -> MemoryRecordEnvelopeDraft<'static> {
            MemoryRecordEnvelopeDraft {
                id: self.id,
                kind: GenericMemoryRecordKind::Fact,
                category_label: None,
                require_explicit_category: false,
                scope: self.scope,
                source_label: SOURCE_LABEL,
                evidence_handle: Some(EVIDENCE_HANDLE),
                admission_status: self.admission_status,
                freshness: self.freshness,
                confidence_percent: self.confidence_percent,
                redaction_class: self.redaction_class,
                body_hash: self.body_hash,
                budget_units: self.budget_units,
                expiration_label: None,
                supersedes_id: None,
                superseded_by_id: self.superseded_by_id,
                conflict_group: self.conflict_group,
                raw_body_marker: None,
                hidden_body_marker: None,
            }
        }
    }

    fn generic_envelope(fixture: GenericEnvelopeFixture) -> MemoryRecordEnvelope {
        MemoryRecordEnvelope::validate(&fixture.draft()).expect("record envelope should validate")
    }

    fn adapter_descriptor(
        role: GenericMemoryAdapterRole,
        capability: GenericMemoryAdapterCapability,
    ) -> MemoryAdapterFamilyDescriptor {
        MemoryAdapterFamilyDescriptor::validate(&MemoryAdapterFamilyDescriptorDraft {
            adapter_label: role.as_str(),
            role,
            capabilities: &[capability, GenericMemoryAdapterCapability::SafeReceipts],
            availability: MemoryAvailabilityStatus::Available,
            owner_label: "host-owned-memory",
        })
        .expect("adapter descriptor should validate")
    }

    fn generic_receipt_draft() -> MemoryOperationReceiptDraft<'static> {
        MemoryOperationReceiptDraft {
            operation_label: "memory-operation",
            adapter_family_label: "retrieval-adapter",
            operation_kind: MemoryOperationKind::Retrieval,
            record_id: Some(RECORD_ID),
            candidate_id: None,
            item_count: EXPECTED_SINGLE_ITEM_U32,
            status: MemoryOperationStatus::Complete,
            redaction_class: MemoryRedactionClass::Public,
            policy_hash: POLICY_HASH,
            body_hash: VALID_HASH,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
            hidden_memory_body: None,
            unbounded_prompt_text: None,
        }
    }

    #[test]
    fn generic_memory_adapter_family_fails_closed_without_ambient_fallback() {
        // r[verify agent_harness_runtime.memory_adapters.family]
        // r[verify agent_harness_runtime.memory_adapters.family.missing]
        let source = adapter_descriptor(
            GenericMemoryAdapterRole::SourceImport,
            GenericMemoryAdapterCapability::SourceImport,
        );
        let retrieval = adapter_descriptor(
            GenericMemoryAdapterRole::RetrievalSearch,
            GenericMemoryAdapterCapability::RetrievalSearch,
        );
        let admission = adapter_descriptor(
            GenericMemoryAdapterRole::Admission,
            GenericMemoryAdapterCapability::AdmissionReview,
        );
        let required = &[
            GenericMemoryAdapterRole::SourceImport,
            GenericMemoryAdapterRole::RetrievalSearch,
            GenericMemoryAdapterRole::Admission,
        ];
        assert_eq!(
            evaluate_memory_adapter_operation(MemoryAdapterOperationInput {
                enabled: true,
                required_roles: required,
                adapters: &[source.clone(), retrieval.clone(), admission],
            }),
            MemoryAdapterOperationAvailability::Ready
        );
        assert_eq!(
            evaluate_memory_adapter_operation(MemoryAdapterOperationInput {
                enabled: true,
                required_roles: required,
                adapters: &[source.clone(), retrieval],
            }),
            MemoryAdapterOperationAvailability::Unavailable {
                role: GenericMemoryAdapterRole::Admission,
            }
        );
        let adapter_without_store_capability = MemoryAdapterFamilyDescriptor::validate(
            &MemoryAdapterFamilyDescriptorDraft {
                adapter_label: "store-adapter",
                role: GenericMemoryAdapterRole::DurableStore,
                capabilities: &[GenericMemoryAdapterCapability::SafeReceipts],
                availability: MemoryAvailabilityStatus::Available,
                owner_label: "host-owned-memory",
            },
        )
        .expect("descriptor with wrong capability should validate metadata");
        assert_eq!(
            evaluate_memory_adapter_operation(MemoryAdapterOperationInput {
                enabled: true,
                required_roles: &[GenericMemoryAdapterRole::DurableStore],
                adapters: &[adapter_without_store_capability],
            }),
            MemoryAdapterOperationAvailability::Unsupported {
                role: GenericMemoryAdapterRole::DurableStore,
            }
        );
        assert!(matches!(
            MemoryAdapterFamilyDescriptor::validate(&MemoryAdapterFamilyDescriptorDraft {
                adapter_label: "empty-adapter",
                role: GenericMemoryAdapterRole::Extraction,
                capabilities: &[],
                availability: MemoryAvailabilityStatus::Available,
                owner_label: "host-owned-memory",
            }),
            Err(MemoryError::MissingAdapterCapabilities { .. })
        ));
    }

    #[test]
    fn memory_record_envelopes_validate_required_metadata_and_reject_malformed_records() {
        // r[verify agent_harness_runtime.memory_adapters.record_envelope]
        // r[verify agent_harness_runtime.memory_adapters.record_envelope.invalid]
        let record = generic_envelope(GenericEnvelopeFixture::valid());
        assert_eq!(record.id(), RECORD_ID);
        assert_eq!(record.evidence_handle(), EVIDENCE_HANDLE);
        assert_eq!(record.kind(), GenericMemoryRecordKind::Fact);
        assert_eq!(record.category(), MemoryRecordCategory::Fact);
        assert_eq!(record.scope(), MemoryTrustScope::Project);
        assert_eq!(record.confidence().percent(), MEMORY_CONFIDENCE_PERCENT);
        assert_eq!(record.body_hash().as_str(), VALID_HASH);

        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                evidence_handle: None,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::MissingEvidence { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                scope: None,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::MissingRecordScope { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                confidence_percent: INVALID_CONFIDENCE_PERCENT,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::InvalidConfidence { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                budget_units: u32::default(),
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::ZeroBudgetUnits { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                freshness: MemoryFreshness::Expired,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::ExpiredDescriptor { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                body_hash: "not-a-hash",
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::InvalidHash(_))
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                raw_body_marker: Some(RAW_BODY),
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    #[test]
    fn taxonomy_categories_map_and_reject_unknown_labels() {
        // r[verify agent_harness_runtime.memory_taxonomy.record_categories]
        // r[verify agent_harness_runtime.memory_taxonomy.record_categories.distinct]
        // r[verify agent_harness_runtime.memory_taxonomy.compatibility]
        // r[verify agent_harness_runtime.memory_taxonomy.compatibility.unknown]
        let episodic = MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
            category_label: Some("episodic"),
            require_explicit_category: true,
            ..GenericEnvelopeFixture::valid().draft()
        })
        .expect("explicit episodic category should validate");
        let procedural = MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
            category_label: Some("procedural"),
            require_explicit_category: true,
            ..GenericEnvelopeFixture::valid().draft()
        })
        .expect("explicit procedural category should validate");
        let compatible = generic_envelope(GenericEnvelopeFixture::valid());

        assert_eq!(episodic.category(), MemoryRecordCategory::Episodic);
        assert_eq!(episodic.category().as_str(), "episodic");
        assert_eq!(procedural.category(), MemoryRecordCategory::Procedural);
        assert_eq!(compatible.category(), MemoryRecordCategory::Fact);
        assert_eq!(
            memory_category_for_generic_kind(GenericMemoryRecordKind::Pitfall),
            MemoryRecordCategory::Pitfall
        );
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                category_label: Some("mystery"),
                require_explicit_category: true,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::UnsupportedCategory { .. })
        ));
        assert!(matches!(
            MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
                require_explicit_category: true,
                ..GenericEnvelopeFixture::valid().draft()
            }),
            Err(MemoryError::MissingCategory { .. })
        ));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct RouteFixture {
        route_id: &'static str,
        adapter_label: &'static str,
        intent: MemoryRouteIntent,
        category: MemoryRecordCategory,
        scope: MemoryTrustScope,
        required_role: GenericMemoryAdapterRole,
        availability: MemoryAvailabilityStatus,
        capability_supported: bool,
        policy_admitted: bool,
        freshness: MemoryFreshness,
        confidence_percent: u8,
        budget_units: u32,
    }

    impl RouteFixture {
        const fn read(route_id: &'static str) -> Self {
            Self {
                route_id,
                adapter_label: "memory-route-adapter",
                intent: MemoryRouteIntent::ReadSearch,
                category: MemoryRecordCategory::Semantic,
                scope: MemoryTrustScope::Project,
                required_role: GenericMemoryAdapterRole::RetrievalSearch,
                availability: MemoryAvailabilityStatus::Available,
                capability_supported: true,
                policy_admitted: true,
                freshness: MemoryFreshness::Fresh,
                confidence_percent: MEMORY_CONFIDENCE_PERCENT,
                budget_units: SMALL_MEMORY_UNITS,
            }
        }

        const fn draft(self) -> MemoryRouteCandidateDraft<'static> {
            MemoryRouteCandidateDraft {
                route_id: self.route_id,
                adapter_label: self.adapter_label,
                intent: self.intent,
                category: self.category,
                scope: self.scope,
                required_role: self.required_role,
                availability: self.availability,
                capability_supported: self.capability_supported,
                policy_admitted: self.policy_admitted,
                freshness: self.freshness,
                confidence_percent: self.confidence_percent,
                budget_units: self.budget_units,
                raw_classifier_output: None,
                raw_memory_body: None,
                credential: None,
                provider_payload: None,
                full_path: None,
            }
        }
    }

    fn route_candidate(fixture: RouteFixture) -> MemoryRouteCandidate {
        MemoryRouteCandidate::validate(&fixture.draft()).expect("route candidate should validate")
    }

    fn route_policy() -> MemoryRoutePolicy<'static> {
        MemoryRoutePolicy {
            route_label: "memory-route-plan",
            policy_hash: POLICY_HASH,
            allowed_scopes: &[MemoryTrustScope::Project],
            include_stale: false,
            budget: MemoryInjectionBudget::new(MEMORY_UNITS).expect("route budget should validate"),
        }
    }

    fn assert_route_status(
        plan: &MemoryRoutePlan,
        route_id: &str,
        status: MemoryRouteOutcomeStatus,
    ) {
        assert!(plan.outcomes().iter().any(|outcome| {
            outcome.route_id() == route_id && outcome.status() == status
        }));
    }

    #[test]
    fn memory_route_planning_selects_safe_read_and_write_routes() {
        // r[verify agent_harness_runtime.memory_routing.route_plan]
        // r[verify agent_harness_runtime.memory_routing.route_plan.read]
        // r[verify agent_harness_runtime.memory_routing.pure_planner]
        // r[verify agent_harness_runtime.memory_routing.pure_planner.deterministic]
        let read = route_candidate(RouteFixture::read(RECORD_ID));
        let store = route_candidate(RouteFixture {
            route_id: SECOND_RECORD_ID,
            intent: MemoryRouteIntent::DurableStore,
            category: MemoryRecordCategory::Candidate,
            required_role: GenericMemoryAdapterRole::DurableStore,
            confidence_percent: MEMORY_CONFIDENCE_PERCENT,
            ..RouteFixture::read(SECOND_RECORD_ID)
        });
        let first = plan_memory_routes(MemoryRoutePlanningInput {
            policy: route_policy(),
            candidates: &[store.clone(), read.clone()],
        })
        .expect("route planning should validate");
        let second = plan_memory_routes(MemoryRoutePlanningInput {
            policy: route_policy(),
            candidates: &[read, store],
        })
        .expect("route planning should be deterministic");
        let receipt = MemoryRouteReceipt::validate(&MemoryRouteReceiptDraft {
            route_label: "memory-route-plan",
            policy_hash: POLICY_HASH,
            selected_count: EXPECTED_TWO_ITEMS_U32,
            skipped_count: u32::default(),
            raw_classifier_output: None,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
        })
        .expect("route receipt should validate");

        assert_eq!(first, second);
        assert_eq!(first.selected_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(first.consumed_units(), MEMORY_UNITS);
        assert_eq!(first.policy_hash().as_str(), POLICY_HASH);
        assert_eq!(receipt.selected_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(receipt.skipped_count(), u32::default());
        assert_route_status(&first, RECORD_ID, MemoryRouteOutcomeStatus::Selected);
        assert_route_status(&first, SECOND_RECORD_ID, MemoryRouteOutcomeStatus::Selected);
    }

    #[test]
    fn memory_route_planning_fails_closed_for_bad_routes_and_leaks() {
        // r[verify agent_harness_runtime.memory_routing.fail_closed]
        // r[verify agent_harness_runtime.memory_routing.fail_closed.malformed]
        // r[verify agent_harness_runtime.memory_routing.verification]
        let denied = route_candidate(RouteFixture {
            route_id: RECORD_ID,
            policy_admitted: false,
            ..RouteFixture::read(RECORD_ID)
        });
        let unavailable = route_candidate(RouteFixture {
            route_id: SECOND_RECORD_ID,
            availability: MemoryAvailabilityStatus::Unavailable,
            ..RouteFixture::read(SECOND_RECORD_ID)
        });
        let stale = route_candidate(RouteFixture {
            route_id: FOURTH_RECORD_ID,
            freshness: MemoryFreshness::Stale,
            ..RouteFixture::read(FOURTH_RECORD_ID)
        });
        let plan = plan_memory_routes(MemoryRoutePlanningInput {
            policy: route_policy(),
            candidates: &[
                denied,
                unavailable,
                route_candidate(RouteFixture {
                    route_id: THIRD_RECORD_ID,
                    capability_supported: false,
                    ..RouteFixture::read(THIRD_RECORD_ID)
                }),
                stale,
            ],
        })
        .expect("fail-closed route planning should validate");

        assert_eq!(plan.selected_count(), u32::default());
        assert_route_status(&plan, RECORD_ID, MemoryRouteOutcomeStatus::Denied);
        assert_route_status(
            &plan,
            SECOND_RECORD_ID,
            MemoryRouteOutcomeStatus::Unavailable,
        );
        assert_route_status(
            &plan,
            THIRD_RECORD_ID,
            MemoryRouteOutcomeStatus::Unsupported,
        );
        assert_route_status(&plan, FOURTH_RECORD_ID, MemoryRouteOutcomeStatus::Stale);
        assert!(matches!(
            MemoryRouteCandidate::validate(&MemoryRouteCandidateDraft {
                raw_classifier_output: Some(RAW_BODY),
                ..RouteFixture::read(RECORD_ID).draft()
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryRouteReceipt::validate(&MemoryRouteReceiptDraft {
                route_label: "memory-route-plan",
                policy_hash: POLICY_HASH,
                selected_count: u32::default(),
                skipped_count: EXPECTED_FOUR_ITEMS_U32,
                raw_classifier_output: None,
                raw_memory_body: None,
                credential: None,
                provider_payload: Some(PROVIDER_PAYLOAD),
                full_path: None,
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    fn evaluation_fixture() -> MemoryEvaluationFixture {
        MemoryEvaluationFixture::validate(&MemoryEvaluationFixtureDraft {
            fixture_id: "memory-evaluation-fixture",
            query_label: "memory-dependent-query",
            expected_record_ids: &[RECORD_ID, SECOND_RECORD_ID],
            scope: MemoryTrustScope::Project,
            category: MemoryRecordCategory::Semantic,
            abstention_expected: false,
            answer_use_label: "answer-used-memory",
            evidence_hash: VALID_HASH,
            raw_prompt_text: None,
            raw_memory_body: None,
            credential: None,
            provider_payload: None,
            full_path: None,
        })
        .expect("evaluation fixture should validate")
    }

    fn baseline() -> MemoryEvaluationBaseline {
        MemoryEvaluationBaseline::validate(&MemoryEvaluationBaselineDraft {
            baseline_label: "no-memory-baseline",
            policy_hash: POLICY_HASH,
            evidence_hash: SECOND_HASH,
            no_memory: true,
        })
        .expect("baseline should validate")
    }

    #[test]
    fn memory_evaluation_metrics_and_baselines_are_safe() {
        // r[verify agent_harness_runtime.memory_evaluation.fixtures]
        // r[verify agent_harness_runtime.memory_evaluation.fixtures.safe]
        // r[verify agent_harness_runtime.memory_evaluation.metrics]
        // r[verify agent_harness_runtime.memory_evaluation.metrics.reducers]
        // r[verify agent_harness_runtime.memory_evaluation.baselines]
        // r[verify agent_harness_runtime.memory_evaluation.external_import]
        // r[verify agent_harness_runtime.memory_evaluation.external_import.safe]
        let fixture = evaluation_fixture();
        let metrics = reduce_memory_evaluation_metrics(MemoryEvaluationInput {
            fixture: &fixture,
            observed: &[
                MemoryEvaluationObservedRecord {
                    record_id: RECORD_ID,
                    category: MemoryRecordCategory::Semantic,
                    freshness: MemoryFreshness::Fresh,
                    answer_used: true,
                    contradictory: false,
                },
                MemoryEvaluationObservedRecord {
                    record_id: SECOND_RECORD_ID,
                    category: MemoryRecordCategory::Episodic,
                    freshness: MemoryFreshness::Stale,
                    answer_used: false,
                    contradictory: true,
                },
            ],
        })
        .expect("metrics should reduce");
        let baseline = baseline();
        let claim = MemoryQualityClaim::validate(&MemoryQualityClaimDraft {
            claim_label: "retrieval-improves-safe-answer-use",
            baseline: Some(&baseline),
            current_metrics: &metrics,
        })
        .expect("baseline-backed claim should validate");
        let benchmark_cases_items =
            MemoryExternalBenchmarkImport::validate(&MemoryExternalBenchmarkImportDraft {
                benchmark_family: "locomo-like",
                run_label: "host-owned-run",
                metric_label: "precision-at-k",
                case_count: EXPECTED_TWO_ITEMS_U32,
                policy_hash: POLICY_HASH,
                evidence_hash: THIRD_HASH,
                raw_conversation: None,
                raw_judge_prompt: None,
                provider_payload: None,
                credential: None,
            })
            .expect("external import descriptor should validate")
            .case_count();

        assert_eq!(fixture.expected_record_ids().len(), EXPECTED_TWO_ITEMS);
        assert_eq!(metrics.observed_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(metrics.expected_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(metrics.relevant_hit_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(metrics.precision_percent(), EXPECTED_FULL_PERCENT);
        assert_eq!(metrics.recall_percent(), EXPECTED_FULL_PERCENT);
        assert_eq!(metrics.stale_hit_count(), EXPECTED_SINGLE_ITEM_U32);
        assert_eq!(metrics.contradiction_hit_count(), EXPECTED_SINGLE_ITEM_U32);
        assert_eq!(metrics.answer_used_count(), EXPECTED_SINGLE_ITEM_U32);
        assert!(metrics.abstention_correct());
        assert_eq!(claim.baseline_label(), baseline.baseline_label());
        assert_eq!(benchmark_cases_items, EXPECTED_TWO_ITEMS_U32);
    }

    #[test]
    fn memory_evaluation_rejects_leaks_and_missing_baselines() {
        // r[verify agent_harness_runtime.memory_evaluation.baselines.required]
        // r[verify agent_harness_runtime.memory_evaluation.verification]
        let fixture = evaluation_fixture();
        let metrics = reduce_memory_evaluation_metrics(MemoryEvaluationInput {
            fixture: &fixture,
            observed: &[],
        })
        .expect("empty observations should reduce");

        assert!(matches!(
            MemoryEvaluationFixture::validate(&MemoryEvaluationFixtureDraft {
                raw_prompt_text: Some(UNBOUNDED_PROMPT),
                ..MemoryEvaluationFixtureDraft {
                    fixture_id: "memory-evaluation-fixture",
                    query_label: "memory-dependent-query",
                    expected_record_ids: &[RECORD_ID],
                    scope: MemoryTrustScope::Project,
                    category: MemoryRecordCategory::Semantic,
                    abstention_expected: false,
                    answer_use_label: "answer-used-memory",
                    evidence_hash: VALID_HASH,
                    raw_prompt_text: None,
                    raw_memory_body: None,
                    credential: None,
                    provider_payload: None,
                    full_path: None,
                }
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryEvaluationFixture::validate(&MemoryEvaluationFixtureDraft {
                fixture_id: "memory-evaluation-fixture",
                query_label: "memory-dependent-query",
                expected_record_ids: &[],
                scope: MemoryTrustScope::Project,
                category: MemoryRecordCategory::Semantic,
                abstention_expected: false,
                answer_use_label: "answer-used-memory",
                evidence_hash: VALID_HASH,
                raw_prompt_text: None,
                raw_memory_body: None,
                credential: None,
                provider_payload: None,
                full_path: None,
            }),
            Err(MemoryError::MissingEvidence { .. })
        ));
        assert!(matches!(
            MemoryQualityClaim::validate(&MemoryQualityClaimDraft {
                claim_label: "unbacked-memory-quality-claim",
                baseline: None,
                current_metrics: &metrics,
            }),
            Err(MemoryError::MissingBaseline { .. })
        ));
        assert!(matches!(
            MemoryExternalBenchmarkImport::validate(&MemoryExternalBenchmarkImportDraft {
                benchmark_family: "locomo-like",
                run_label: "host-owned-run",
                metric_label: "precision-at-k",
                case_count: EXPECTED_TWO_ITEMS_U32,
                policy_hash: POLICY_HASH,
                evidence_hash: THIRD_HASH,
                raw_conversation: Some(RAW_BODY),
                raw_judge_prompt: None,
                provider_payload: None,
                credential: None,
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    fn generic_retrieval_policy() -> MemoryRetrievalPolicy<'static> {
        MemoryRetrievalPolicy {
            query_label: "session-query",
            allowed_scopes: &[MemoryTrustScope::Project],
            include_stale: false,
            budget: MemoryInjectionBudget::new(MEMORY_UNITS).expect("budget should validate"),
        }
    }

    fn assert_retrieval_status(
        plan: &MemoryRetrievalPlan,
        record_id: &str,
        status: MemoryRetrievalOutcomeStatus,
    ) {
        assert!(plan.outcomes().iter().any(|outcome| {
            outcome.record_id() == record_id && outcome.status() == status
        }));
    }

    fn quality_policy() -> MemoryRetrievalQualityPolicy {
        MemoryRetrievalQualityPolicy::validate(&MemoryRetrievalQualityPolicyDraft {
            policy_label: "quality-policy",
            policy_hash: POLICY_HASH,
            prioritize_manual_pins: true,
            category_priority: &[
                MemoryRecordCategory::Procedural,
                MemoryRecordCategory::Semantic,
                MemoryRecordCategory::Episodic,
            ],
            diversity_group_limit: EXPECTED_SINGLE_ITEM_U32,
            raw_policy_text: None,
        })
        .expect("quality policy should validate")
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct QualityRetrievalRecords {
        semantic: MemoryRecordEnvelope,
        duplicate: MemoryRecordEnvelope,
        procedural: MemoryRecordEnvelope,
        rerank: MemoryRerankEvidence,
    }

    fn quality_retrieval_records() -> QualityRetrievalRecords {
        let semantic = generic_envelope(GenericEnvelopeFixture {
            id: RECORD_ID,
            conflict_group: Some("duplicate-group"),
            ..GenericEnvelopeFixture::valid()
        });
        let duplicate = generic_envelope(GenericEnvelopeFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            conflict_group: Some("duplicate-group"),
            ..GenericEnvelopeFixture::valid()
        });
        let procedural = MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
            id: THIRD_RECORD_ID,
            body_hash: THIRD_HASH,
            category_label: Some("procedural"),
            ..GenericEnvelopeFixture::valid().draft()
        })
        .expect("procedural record should validate");
        let rerank = MemoryRerankEvidence::validate(&MemoryRerankEvidenceDraft {
            record_id: THIRD_RECORD_ID,
            policy_hash: POLICY_HASH,
            reason_label: "rerank-safe-reason",
            score_percent: HIGH_RERANK_PERCENT,
            raw_memory_body: None,
            provider_payload: None,
            credential: None,
        })
        .expect("rerank evidence should validate");
        assert_eq!(semantic.id(), RECORD_ID);
        assert_eq!(procedural.category(), MemoryRecordCategory::Procedural);
        QualityRetrievalRecords {
            semantic,
            duplicate,
            procedural,
            rerank,
        }
    }

    fn plan_primary_quality_retrieval(
        records: &QualityRetrievalRecords,
        quality: &MemoryRetrievalQualityPolicy,
    ) -> MemoryRetrievalPlan {
        plan_memory_retrieval_with_quality(MemoryQualityRetrievalInput {
            retrieval_policy: generic_retrieval_policy(),
            quality_policy: quality,
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &records.semantic,
                    match_reason: MemoryMatchReason::Semantic,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "vector-backend",
                },
                MemoryRetrievalCandidate {
                    record: &records.duplicate,
                    match_reason: MemoryMatchReason::Keyword,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "file-backend",
                },
                MemoryRetrievalCandidate {
                    record: &records.procedural,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "remote-backend",
                },
            ],
            rerank_evidence: core::slice::from_ref(&records.rerank),
        })
        .expect("quality retrieval should plan")
    }

    fn plan_permuted_quality_retrieval(
        records: &QualityRetrievalRecords,
        quality: &MemoryRetrievalQualityPolicy,
    ) -> MemoryRetrievalPlan {
        plan_memory_retrieval_with_quality(MemoryQualityRetrievalInput {
            retrieval_policy: generic_retrieval_policy(),
            quality_policy: quality,
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &records.procedural,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "other-remote-backend",
                },
                MemoryRetrievalCandidate {
                    record: &records.duplicate,
                    match_reason: MemoryMatchReason::Keyword,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "other-file-backend",
                },
                MemoryRetrievalCandidate {
                    record: &records.semantic,
                    match_reason: MemoryMatchReason::Semantic,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "other-vector-backend",
                },
            ],
            rerank_evidence: core::slice::from_ref(&records.rerank),
        })
        .expect("quality retrieval should be backend neutral")
    }

    fn assert_quality_retrieval_plan(plan: &MemoryRetrievalPlan) {
        assert_eq!(plan.selected_count(), EXPECTED_TWO_ITEMS_U32);
        assert_eq!(plan.consumed_units(), MEMORY_UNITS);
        assert_eq!(plan.outcomes()[0].record_id(), THIRD_RECORD_ID);
        assert_retrieval_status(plan, RECORD_ID, MemoryRetrievalOutcomeStatus::Selected);
        assert_retrieval_status(
            plan,
            SECOND_RECORD_ID,
            MemoryRetrievalOutcomeStatus::DiversityDuplicate,
        );
        assert_retrieval_status(plan, THIRD_RECORD_ID, MemoryRetrievalOutcomeStatus::Selected);
    }

    #[test]
    fn retrieval_quality_policy_orders_reranks_and_limits_duplicates() {
        // r[verify agent_harness_runtime.memory_retrieval_quality.policy]
        // r[verify agent_harness_runtime.memory_retrieval_quality.policy.ordering]
        // r[verify agent_harness_runtime.memory_retrieval_quality.planning]
        // r[verify agent_harness_runtime.memory_retrieval_quality.planning.backend_neutral]
        // r[verify agent_harness_runtime.memory_retrieval_quality.rerank_evidence]
        // r[verify agent_harness_runtime.memory_retrieval_quality.diversity]
        // r[verify agent_harness_runtime.memory_retrieval_quality.diversity.duplicates]
        let quality = quality_policy();
        let records = quality_retrieval_records();
        let first = plan_primary_quality_retrieval(&records, &quality);
        let second = plan_permuted_quality_retrieval(&records, &quality);

        assert_eq!(first, second);
        assert_quality_retrieval_plan(&first);
    }

    #[test]
    fn retrieval_quality_rejects_unsafe_policy_and_rerank_evidence() {
        // r[verify agent_harness_runtime.memory_retrieval_quality.rerank_evidence.unsafe]
        // r[verify agent_harness_runtime.memory_retrieval_quality.verification]
        assert!(matches!(
            MemoryRetrievalQualityPolicy::validate(&MemoryRetrievalQualityPolicyDraft {
                policy_label: "quality-policy",
                policy_hash: POLICY_HASH,
                prioritize_manual_pins: true,
                category_priority: &[MemoryRecordCategory::Semantic],
                diversity_group_limit: EXPECTED_SINGLE_ITEM_U32,
                raw_policy_text: Some(UNBOUNDED_PROMPT),
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
        assert!(matches!(
            MemoryRerankEvidence::validate(&MemoryRerankEvidenceDraft {
                record_id: RECORD_ID,
                policy_hash: POLICY_HASH,
                reason_label: "rerank-safe-reason",
                score_percent: INVALID_CONFIDENCE_PERCENT,
                raw_memory_body: None,
                provider_payload: None,
                credential: None,
            }),
            Err(MemoryError::InvalidConfidence { .. })
        ));
        assert!(matches!(
            MemoryRerankEvidence::validate(&MemoryRerankEvidenceDraft {
                record_id: RECORD_ID,
                policy_hash: POLICY_HASH,
                reason_label: "rerank-safe-reason",
                score_percent: MEMORY_CONFIDENCE_PERCENT,
                raw_memory_body: None,
                provider_payload: Some(PROVIDER_PAYLOAD),
                credential: None,
            }),
            Err(MemoryError::ReceiptLeak { .. })
        ));
    }

    #[test]
    fn retrieval_planning_ignores_backend_labels_for_equivalent_records() {
        // r[verify agent_harness_runtime.memory_adapters.retrieval_plan]
        // r[verify agent_harness_runtime.memory_adapters.retrieval_plan.backend_equivalence]
        let selected = generic_envelope(GenericEnvelopeFixture::valid());
        let skipped = generic_envelope(GenericEnvelopeFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            scope: Some(MemoryTrustScope::Host),
            ..GenericEnvelopeFixture::valid()
        });
        let first = plan_memory_retrieval(MemoryRetrievalInput {
            policy: generic_retrieval_policy(),
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &selected,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "mcp-backend",
                },
                MemoryRetrievalCandidate {
                    record: &skipped,
                    match_reason: MemoryMatchReason::ExactScope,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "sql-backend",
                },
            ],
        })
        .expect("retrieval planning should succeed");
        let second = plan_memory_retrieval(MemoryRetrievalInput {
            policy: generic_retrieval_policy(),
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &selected,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "another-mcp-backend",
                },
                MemoryRetrievalCandidate {
                    record: &skipped,
                    match_reason: MemoryMatchReason::ExactScope,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "another-sql-backend",
                },
            ],
        })
        .expect("second retrieval planning should succeed");
        assert_eq!(first, second);
        assert_eq!(first.selected_count(), EXPECTED_SINGLE_ITEM_U32);
    }

    #[test]
    fn taxonomy_category_does_not_bypass_prompt_admission() {
        // r[verify agent_harness_runtime.memory_taxonomy.non_authority]
        // r[verify agent_harness_runtime.memory_taxonomy.non_authority.policy]
        // r[verify agent_harness_runtime.memory_taxonomy.verification]
        let allowed = generic_envelope(GenericEnvelopeFixture::valid());
        let redacted_procedure = MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
            id: SECOND_RECORD_ID,
            category_label: Some("procedural"),
            redaction_class: MemoryRedactionClass::Secret,
            ..GenericEnvelopeFixture::valid().draft()
        })
        .expect("redacted procedural metadata should validate before retrieval");
        let stale_episode = MemoryRecordEnvelope::validate(&MemoryRecordEnvelopeDraft {
            id: THIRD_RECORD_ID,
            category_label: Some("episodic"),
            freshness: MemoryFreshness::Stale,
            ..GenericEnvelopeFixture::valid().draft()
        })
        .expect("stale episodic metadata should validate before retrieval");
        let plan = plan_memory_retrieval(MemoryRetrievalInput {
            policy: generic_retrieval_policy(),
            candidates: &[
                MemoryRetrievalCandidate {
                    record: &redacted_procedure,
                    match_reason: MemoryMatchReason::ManualPin,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "semantic-backend",
                },
                MemoryRetrievalCandidate {
                    record: &stale_episode,
                    match_reason: MemoryMatchReason::Recent,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "episode-backend",
                },
                MemoryRetrievalCandidate {
                    record: &allowed,
                    match_reason: MemoryMatchReason::Keyword,
                    relevance_percent: MEMORY_CONFIDENCE_PERCENT,
                    backend_label: "file-backend",
                },
            ],
        })
        .expect("taxonomy-aware retrieval should validate");

        assert_retrieval_status(&plan, RECORD_ID, MemoryRetrievalOutcomeStatus::Selected);
        assert_retrieval_status(
            &plan,
            SECOND_RECORD_ID,
            MemoryRetrievalOutcomeStatus::Redacted,
        );
        assert_retrieval_status(&plan, THIRD_RECORD_ID, MemoryRetrievalOutcomeStatus::Stale);
        assert!(plan.outcomes().iter().any(|outcome| {
            outcome.record_id() == SECOND_RECORD_ID
                && outcome.category() == MemoryRecordCategory::Procedural
                && outcome.consumed_units() == u32::default()
        }));
        assert_eq!(plan.selected_count(), EXPECTED_SINGLE_ITEM_U32);
    }
