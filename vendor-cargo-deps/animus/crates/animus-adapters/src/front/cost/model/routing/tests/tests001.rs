    // r[verify agent_harness_runtime.cost_model_routing.verification]
    // r[verify agent_harness_runtime.cost_model_routing.verification.closeout]
    use super::*;

    const SCHEMA_VERSION: u16 = 1;
    const SIMPLE_ROLE: &str = "economy";
    const MEDIUM_ROLE: &str = "balanced";
    const COMPLEX_ROLE: &str = "deep-reasoning";
    const SIMPLE_MODEL: &str = "safe-economy-model";
    const MEDIUM_MODEL: &str = "safe-balanced-model";
    const COMPLEX_MODEL: &str = "safe-reasoning-model";
    const POLICY_TAG: &str = "default";
    const POLICY_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const SIMPLE_MAX_SCORE: u16 = 4;
    const MEDIUM_MAX_SCORE: u16 = 10;
    const SIMPLE_MIN_SCORE: u16 = 0;
    const MEDIUM_MIN_SCORE: u16 = 5;
    const COMPLEX_MIN_SCORE: u16 = 11;
    const TOKEN_WEIGHT: i16 = 2;
    const FILE_WEIGHT: i16 = 1;
    const TOOL_WEIGHT: i16 = 2;
    const RISK_WEIGHT: i16 = 3;
    const NEGATIVE_WEIGHT: i16 = -1;
    const LOW_BUCKET: u16 = 1;
    const MEDIUM_BUCKET: u16 = 3;
    const HIGH_BUCKET: u16 = 4;
    const ZERO_BUCKET: u16 = 0;
    const ZERO_COST_UNITS: u32 = 0;
    const LOW_COST: u32 = 1;
    const MEDIUM_COST: u32 = 5;
    const HIGH_COST: u32 = 15;
    const CURRENT_LOW_COST: u32 = 1;
    const SOFT_BUDGET: u32 = 10;
    const HARD_BUDGET: u32 = 20;
    const CURRENT_SOFT_PRESSURE: u32 = 12;
    const CURRENT_HARD_LIMIT: u32 = 20;
    const EXPECTED_CATALOG_COUNT: usize = 3;
    const DEFAULT_ROLES: [RoutingRoleDraft<'static>; EXPECTED_CATALOG_COUNT] = [
        role(SIMPLE_ROLE, SIMPLE_MIN_SCORE, None),
        role(MEDIUM_ROLE, MEDIUM_MIN_SCORE, Some(SIMPLE_ROLE)),
        role(COMPLEX_ROLE, COMPLEX_MIN_SCORE, Some(MEDIUM_ROLE)),
    ];
    const RAW_PROMPT: &str = "raw prompt text";
    const PROVIDER_PAYLOAD: &str = "{\"provider\":\"payload\"}";
    const CREDENTIAL: &str = "api_key=secret";
    const PRICING_STORE_DUMP: &str = "full pricing store dump";
    const HIDDEN_REASONING: &str = "hidden chain of thought";

    #[test]
    fn invalid_policies_fail_closed() {
        // r[verify agent_harness_runtime.cost_model_routing.policy.invalid]
        assert_policy_issue(
            policy_with_roles(&[
                role(SIMPLE_ROLE, SIMPLE_MIN_SCORE, None),
                role(SIMPLE_ROLE, MEDIUM_MIN_SCORE, Some(SIMPLE_ROLE)),
            ]),
            ModelRoutingPolicyIssue::DuplicateRole,
        );
        assert_policy_issue(
            ModelRoutingPolicyDraft {
                weights: RoutingComplexityWeightsDraft {
                    token: NEGATIVE_WEIGHT,
                    ..weights()
                },
                ..policy_draft()
            },
            ModelRoutingPolicyIssue::NegativeWeight { field: "token" },
        );
        assert_policy_issue(
            ModelRoutingPolicyDraft {
                thresholds: RoutingThresholdsDraft {
                    simple_max_score: MEDIUM_MAX_SCORE,
                    medium_max_score: SIMPLE_MAX_SCORE,
                },
                ..policy_draft()
            },
            ModelRoutingPolicyIssue::UnorderedThresholds,
        );
        assert_policy_issue(
            policy_with_roles(&[role(COMPLEX_ROLE, COMPLEX_MIN_SCORE, Some(MEDIUM_ROLE))]),
            ModelRoutingPolicyIssue::UnknownDowngradeTarget,
        );
        assert_policy_issue(
            ModelRoutingPolicyDraft {
                missing_price_behavior: None,
                ..policy_draft()
            },
            ModelRoutingPolicyIssue::MissingPriceBehaviorUnspecified,
        );
    }

    #[test]
    fn simple_medium_complex_and_explicit_hint_decisions_are_pure() {
        // r[verify agent_harness_runtime.cost_model_routing.decision.complex]
        let policy = validated_policy();
        let catalog = validated_catalog(&catalog_entries());
        let simple = route(
            &policy,
            &catalog,
            signals(LOW_BUCKET, LOW_BUCKET, ZERO_BUCKET, ZERO_BUCKET, None),
            within_usage(),
        );
        let medium = route(
            &policy,
            &catalog,
            signals(MEDIUM_BUCKET, ZERO_BUCKET, ZERO_BUCKET, ZERO_BUCKET, None),
            within_usage(),
        );
        let complex = route(
            &policy,
            &catalog,
            signals(HIGH_BUCKET, HIGH_BUCKET, HIGH_BUCKET, HIGH_BUCKET, None),
            within_usage(),
        );
        let hinted = route(
            &policy,
            &catalog,
            signals(
                LOW_BUCKET,
                LOW_BUCKET,
                ZERO_BUCKET,
                ZERO_BUCKET,
                Some(COMPLEX_ROLE),
            ),
            within_usage(),
        );

        assert_selected(
            &simple,
            SIMPLE_ROLE,
            SIMPLE_MODEL,
            RoutingScoreBucket::Simple,
        );
        assert_selected(
            &medium,
            MEDIUM_ROLE,
            MEDIUM_MODEL,
            RoutingScoreBucket::Medium,
        );
        assert_selected(
            &complex,
            COMPLEX_ROLE,
            COMPLEX_MODEL,
            RoutingScoreBucket::Complex,
        );
        assert_selected(
            &hinted,
            COMPLEX_ROLE,
            COMPLEX_MODEL,
            RoutingScoreBucket::Simple,
        );
        assert_eq!(
            hinted.fallback_class(),
            Some(RoutingFallbackClass::ExplicitUserHint)
        );
        assert!(!simple.provider_request_sent());
        assert!(!medium.provider_request_sent());
        assert!(!complex.provider_request_sent());
        assert!(!hinted.provider_request_sent());
    }

    #[test]
    fn budget_pressure_downgrades_or_denies_with_reason_metadata() {
        // r[verify agent_harness_runtime.cost_model_routing.budget.hard_limit]
        let policy = validated_policy();
        let catalog = validated_catalog(&catalog_entries());
        let soft = route(&policy, &catalog, complex_signals(), soft_pressure_usage());
        let hard = route(&policy, &catalog, complex_signals(), hard_limit_usage());
        assert_selected(
            &soft,
            MEDIUM_ROLE,
            MEDIUM_MODEL,
            RoutingScoreBucket::Complex,
        );
        assert_eq!(soft.budget_status(), RoutingBudgetStatus::SoftPressure);
        assert_eq!(
            soft.fallback_class(),
            Some(RoutingFallbackClass::SoftBudgetPressure)
        );
        assert_selected(
            &hard,
            MEDIUM_ROLE,
            MEDIUM_MODEL,
            RoutingScoreBucket::Complex,
        );
        assert_eq!(hard.budget_status(), RoutingBudgetStatus::HardLimit);
        assert_eq!(
            hard.fallback_class(),
            Some(RoutingFallbackClass::HardBudgetDowngrade)
        );

        let no_downgrade_policy = ModelRoutingPolicy::validate(&policy_with_roles(&[role(
            COMPLEX_ROLE,
            COMPLEX_MIN_SCORE,
            None,
        )]))
        .expect("single-role policy should validate");
        let denied = route(
            &no_downgrade_policy,
            &catalog,
            complex_signals(),
            hard_limit_usage(),
        );
        assert_eq!(denied.status(), RoutingDecisionStatus::Denied);
        assert_eq!(
            denied.fallback_class(),
            Some(RoutingFallbackClass::HardBudgetDenied)
        );
        assert!(!denied.provider_request_sent());
    }

    #[test]
    fn missing_price_and_unavailable_models_fail_without_provider_fallback() {
        // r[verify agent_harness_runtime.cost_model_routing.catalog.missing_price]
        let policy = validated_policy();
        let missing_price_catalog = validated_catalog(&[catalog_entry(
            COMPLEX_ROLE,
            COMPLEX_MODEL,
            None,
            RoutingModelAvailability::Available,
            RoutingModelCapability::Reasoning,
        )]);
        let missing_price = route(
            &policy,
            &missing_price_catalog,
            complex_signals(),
            within_usage(),
        );
        assert_eq!(missing_price.status(), RoutingDecisionStatus::Denied);
        assert_eq!(
            missing_price.fallback_class(),
            Some(RoutingFallbackClass::MissingPrice)
        );
        assert!(!missing_price.provider_request_sent());

        let unavailable_catalog = validated_catalog(&[catalog_entry(
            COMPLEX_ROLE,
            COMPLEX_MODEL,
            Some(HIGH_COST),
            RoutingModelAvailability::Unavailable,
            RoutingModelCapability::Reasoning,
        )]);
        let unavailable = route(
            &policy,
            &unavailable_catalog,
            complex_signals(),
            within_usage(),
        );
        assert_eq!(unavailable.status(), RoutingDecisionStatus::Denied);
        assert_eq!(
            unavailable.fallback_class(),
            Some(RoutingFallbackClass::UnavailableModel)
        );
        assert!(!unavailable.provider_request_sent());
    }
