    use serde::Deserialize;

    const GENERATED_MODEL_ROUTING_POLICY_JSON: &str = include_str!(
        "../../../../../../../../policy/model-routing/generated/model-routing-policy.json"
    );

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
    struct GeneratedModelRoutingPolicy {
        schema_version: u16,
        roles: Vec<GeneratedRoutingRole>,
        thresholds: GeneratedRoutingThresholds,
        weights: GeneratedRoutingWeights,
        missing_price_behavior: String,
        budget_behavior: String,
        policy_hash: String,
    }

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
    struct GeneratedRoutingRole {
        role_label: String,
        minimum_score: u16,
        downgrade_role_label: String,
    }

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
    struct GeneratedRoutingThresholds {
        simple_max_score: u16,
        medium_max_score: u16,
    }

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
    struct GeneratedRoutingWeights {
        #[serde(rename = "token_bucket_weight")]
        token: i16,
        #[serde(rename = "file_bucket_weight")]
        file: i16,
        #[serde(rename = "tool_bucket_weight")]
        tool: i16,
        #[serde(rename = "risk_bucket_weight")]
        risk: i16,
    }

    #[test]
    fn generated_model_routing_policy_matches_validated_dto_meaning() {
        // r[verify agent_harness_runtime.nickel_policy_contracts.projection_freshness]
        let generated = generated_model_routing_policy();
        let role_drafts = generated_role_drafts(&generated);
        let draft = generated_policy_draft(&generated, role_drafts.as_slice());
        let validated = ModelRoutingPolicy::validate(&draft)
            .expect("generated model-routing policy should validate");

        assert_eq!(validated.schema_version().get(), generated.schema_version);
        assert_eq!(validated.roles().len(), generated.roles.len());
        for (validated_role, generated_role) in
            validated.roles().iter().zip(generated.roles.iter())
        {
            assert_eq!(validated_role.role_label(), generated_role.role_label);
            assert_eq!(validated_role.minimum_score(), generated_role.minimum_score);
            assert_eq!(
                validated_role.downgrade_role_label().unwrap_or(""),
                generated_role.downgrade_role_label
            );
        }
        assert_eq!(
            validated.thresholds().simple_max_score(),
            generated.thresholds.simple_max_score
        );
        assert_eq!(
            validated.thresholds().medium_max_score(),
            generated.thresholds.medium_max_score
        );
        assert_eq!(
            validated.weights().token_bucket_weight(),
            generated_weight(generated.weights.token)
        );
        assert_eq!(
            validated.weights().file_bucket_weight(),
            generated_weight(generated.weights.file)
        );
        assert_eq!(
            validated.weights().tool_bucket_weight(),
            generated_weight(generated.weights.tool)
        );
        assert_eq!(
            validated.weights().risk_bucket_weight(),
            generated_weight(generated.weights.risk)
        );
        assert_eq!(
            validated.missing_price_behavior().as_str(),
            generated.missing_price_behavior
        );
        assert_eq!(
            validated.budget_behavior().as_str(),
            generated.budget_behavior
        );
        assert_eq!(validated.policy_hash().as_str(), generated.policy_hash);
    }

    #[test]
    fn malformed_generated_model_routing_meaning_fails_closed() {
        // r[verify agent_harness_runtime.nickel_policy_contracts.fixtures]
        let mut generated = generated_model_routing_policy();
        generated.roles[1].role_label = generated.roles[0].role_label.clone();
        let role_drafts = generated_role_drafts(&generated);
        let draft = generated_policy_draft(&generated, role_drafts.as_slice());
        let error = ModelRoutingPolicy::validate(&draft)
            .expect_err("duplicate generated role should fail");

        assert_eq!(
            error,
            ModelRoutingError::Policy(ModelRoutingPolicyIssue::DuplicateRole)
        );
    }

    fn generated_model_routing_policy() -> GeneratedModelRoutingPolicy {
        serde_json::from_str(GENERATED_MODEL_ROUTING_POLICY_JSON)
            .expect("generated model-routing JSON should deserialize")
    }

    fn generated_role_drafts(
        generated: &GeneratedModelRoutingPolicy,
    ) -> Vec<RoutingRoleDraft<'_>> {
        generated
            .roles
            .iter()
            .map(|role| RoutingRoleDraft {
                role_label: role.role_label.as_str(),
                minimum_score: role.minimum_score,
                downgrade_role_label: if role.downgrade_role_label.is_empty() {
                    None
                } else {
                    Some(role.downgrade_role_label.as_str())
                },
            })
            .collect()
    }

    fn generated_weight(value: i16) -> u16 {
        u16::try_from(value).expect("generated routing weight should be non-negative")
    }

    fn generated_policy_draft<'a>(
        generated: &'a GeneratedModelRoutingPolicy,
        roles: &'a [RoutingRoleDraft<'a>],
    ) -> ModelRoutingPolicyDraft<'a> {
        assert_eq!(
            generated.missing_price_behavior,
            RoutingMissingPriceBehavior::Deny.as_str()
        );
        assert_eq!(
            generated.budget_behavior,
            RoutingBudgetBehavior::PreferCheaperThenDeny.as_str()
        );
        ModelRoutingPolicyDraft {
            schema_version: generated.schema_version,
            roles,
            thresholds: RoutingThresholdsDraft {
                simple_max_score: generated.thresholds.simple_max_score,
                medium_max_score: generated.thresholds.medium_max_score,
            },
            weights: RoutingComplexityWeightsDraft {
                token: generated.weights.token,
                file: generated.weights.file,
                tool: generated.weights.tool,
                risk: generated.weights.risk,
            },
            missing_price_behavior: Some(RoutingMissingPriceBehavior::Deny),
            budget_behavior: RoutingBudgetBehavior::PreferCheaperThenDeny,
            policy_hash: generated.policy_hash.as_str(),
        }
    }
