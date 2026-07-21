
    #[test]
    fn routing_receipts_hash_safe_metadata_and_reject_leaks() {
        // r[verify agent_harness_runtime.cost_model_routing.receipts.rejects_leaks]
        let policy = validated_policy();
        let catalog = validated_catalog(&catalog_entries());
        let decision = route(&policy, &catalog, complex_signals(), within_usage());
        let first = ModelRoutingReceipt::validate(&receipt(&decision, ReceiptLeakFixture::none()))
            .expect("safe routing receipt should validate");
        let second = ModelRoutingReceipt::validate(&receipt(&decision, ReceiptLeakFixture::none()))
            .expect("safe routing receipt should validate deterministically");
        assert_eq!(first.receipt_hash(), second.receipt_hash());
        assert_eq!(first.decision(), &decision);

        assert_receipt_leak(&decision, ReceiptLeakFixture::raw_prompt());
        assert_receipt_leak(&decision, ReceiptLeakFixture::provider_payload());
        assert_receipt_leak(&decision, ReceiptLeakFixture::credential());
        assert_receipt_leak(&decision, ReceiptLeakFixture::pricing_store_dump());
        assert_receipt_leak(&decision, ReceiptLeakFixture::hidden_reasoning());
    }

    #[test]
    fn catalog_and_usage_bounds_fail_closed() {
        let too_many_tags = [POLICY_TAG; MAX_POLICY_TAGS + 1];
        let catalog_error = RoutingModelCatalog::validate(&[RoutingModelCatalogEntryDraft {
            policy_tags: &too_many_tags,
            ..catalog_entry(
                SIMPLE_ROLE,
                SIMPLE_MODEL,
                Some(LOW_COST),
                RoutingModelAvailability::Available,
                RoutingModelCapability::Text,
            )
        }])
        .expect_err("too many policy tags should fail");
        assert!(matches!(
            catalog_error,
            ModelRoutingError::TooManyItems { .. }
        ));

        let usage_error = RoutingUsage::validate(RoutingUsageDraft {
            current_cost: ZERO_COST_UNITS,
            soft_budget: HARD_BUDGET,
            hard_budget: SOFT_BUDGET,
        })
        .expect_err("unordered budgets should fail");
        assert_eq!(
            usage_error,
            ModelRoutingError::Policy(ModelRoutingPolicyIssue::UnorderedBudgets)
        );
    }

    fn validated_policy() -> ModelRoutingPolicy {
        ModelRoutingPolicy::validate(&policy_draft()).expect("routing policy should validate")
    }

    fn policy_draft() -> ModelRoutingPolicyDraft<'static> {
        policy_with_roles(&DEFAULT_ROLES)
    }

    fn policy_with_roles<'a>(roles: &'a [RoutingRoleDraft<'a>]) -> ModelRoutingPolicyDraft<'a> {
        ModelRoutingPolicyDraft {
            schema_version: SCHEMA_VERSION,
            roles,
            thresholds: RoutingThresholdsDraft {
                simple_max_score: SIMPLE_MAX_SCORE,
                medium_max_score: MEDIUM_MAX_SCORE,
            },
            weights: weights(),
            missing_price_behavior: Some(RoutingMissingPriceBehavior::Deny),
            budget_behavior: RoutingBudgetBehavior::PreferCheaperThenDeny,
            policy_hash: POLICY_HASH,
        }
    }

    const fn role(
        role_label: &'static str,
        minimum_score: u16,
        downgrade_role_label: Option<&'static str>,
    ) -> RoutingRoleDraft<'static> {
        RoutingRoleDraft {
            role_label,
            minimum_score,
            downgrade_role_label,
        }
    }

    const fn weights() -> RoutingComplexityWeightsDraft {
        RoutingComplexityWeightsDraft {
            token: TOKEN_WEIGHT,
            file: FILE_WEIGHT,
            tool: TOOL_WEIGHT,
            risk: RISK_WEIGHT,
        }
    }

    fn assert_policy_issue(draft: ModelRoutingPolicyDraft<'_>, issue: ModelRoutingPolicyIssue) {
        assert_eq!(
            ModelRoutingPolicy::validate(&draft),
            Err(ModelRoutingError::Policy(issue))
        );
    }

    fn catalog_entries() -> [RoutingModelCatalogEntryDraft<'static>; EXPECTED_CATALOG_COUNT] {
        [
            catalog_entry(
                SIMPLE_ROLE,
                SIMPLE_MODEL,
                Some(LOW_COST),
                RoutingModelAvailability::Available,
                RoutingModelCapability::Text,
            ),
            catalog_entry(
                MEDIUM_ROLE,
                MEDIUM_MODEL,
                Some(MEDIUM_COST),
                RoutingModelAvailability::Available,
                RoutingModelCapability::ToolUse,
            ),
            catalog_entry(
                COMPLEX_ROLE,
                COMPLEX_MODEL,
                Some(HIGH_COST),
                RoutingModelAvailability::Available,
                RoutingModelCapability::Reasoning,
            ),
        ]
    }

    #[expect(clippy::too_many_arguments, reason = "fixture mirrors catalog entry fields")]
    const fn catalog_entry(
        role_label: &'static str,
        model_label: &'static str,
        price_units: Option<u32>,
        availability: RoutingModelAvailability,
        capability: RoutingModelCapability,
    ) -> RoutingModelCatalogEntryDraft<'static> {
        RoutingModelCatalogEntryDraft {
            role_label,
            model_label,
            capability,
            price_units,
            availability,
            policy_tags: &[POLICY_TAG],
        }
    }

    fn validated_catalog(entries: &[RoutingModelCatalogEntryDraft<'_>]) -> RoutingModelCatalog {
        RoutingModelCatalog::validate(entries).expect("routing catalog should validate")
    }

    #[expect(clippy::too_many_arguments, reason = "fixture mirrors routing signal buckets")]
    fn signals(
        token_bucket: u16,
        file_bucket: u16,
        tool_bucket: u16,
        risk_bucket: u16,
        user_hint_role: Option<&'static str>,
    ) -> RoutingTaskSignals {
        RoutingTaskSignals::validate(&RoutingTaskSignalsDraft {
            token_bucket,
            file_bucket,
            tool_bucket,
            risk_bucket,
            user_hint_role,
        })
        .expect("routing signals should validate")
    }

    fn complex_signals() -> RoutingTaskSignals {
        signals(HIGH_BUCKET, HIGH_BUCKET, HIGH_BUCKET, HIGH_BUCKET, None)
    }

    fn within_usage() -> RoutingUsage {
        RoutingUsage::validate(RoutingUsageDraft {
            current_cost: CURRENT_LOW_COST,
            soft_budget: SOFT_BUDGET,
            hard_budget: HARD_BUDGET,
        })
        .expect("usage should validate")
    }

    fn soft_pressure_usage() -> RoutingUsage {
        RoutingUsage::validate(RoutingUsageDraft {
            current_cost: CURRENT_SOFT_PRESSURE,
            soft_budget: SOFT_BUDGET,
            hard_budget: HARD_BUDGET,
        })
        .expect("usage should validate")
    }

    fn hard_limit_usage() -> RoutingUsage {
        RoutingUsage::validate(RoutingUsageDraft {
            current_cost: CURRENT_HARD_LIMIT,
            soft_budget: SOFT_BUDGET,
            hard_budget: HARD_BUDGET,
        })
        .expect("usage should validate")
    }

    #[expect(clippy::needless_pass_by_value, reason = "fixtures are consumed as routing input")]
    fn route(
        policy: &ModelRoutingPolicy,
        catalog: &RoutingModelCatalog,
        signals: RoutingTaskSignals,
        usage: RoutingUsage,
    ) -> ModelRoutingDecision {
        route_model(ModelRoutingInput {
            policy,
            signals: &signals,
            catalog,
            usage,
        })
        .expect("routing should be pure and deterministic")
    }

    fn assert_selected(
        decision: &ModelRoutingDecision,
        role_label: &str,
        model_label: &str,
        score_bucket: RoutingScoreBucket,
    ) {
        assert_eq!(decision.status(), RoutingDecisionStatus::Selected);
        assert_eq!(decision.selected_role_label(), Some(role_label));
        assert_eq!(decision.selected_model_label(), Some(model_label));
        assert_eq!(decision.score_bucket(), score_bucket);
        assert_eq!(decision.candidate_count(), EXPECTED_CATALOG_COUNT);
    }

