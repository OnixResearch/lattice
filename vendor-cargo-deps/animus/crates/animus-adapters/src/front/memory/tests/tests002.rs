    #[test]
    fn opt_in_policy_controls_use_and_generation_independently() {
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.policy_controls.consent]
        let enabled = opt_in_policy(MemoryScopeControls {
            user: MemoryOptInControl::Enabled,
            project: MemoryOptInControl::Enabled,
            thread: MemoryOptInControl::Enabled,
        });
        assert!(enabled.memory_use_enabled());
        assert!(enabled.memory_generation_enabled());

        let missing_thread = opt_in_policy_with_controls(
            MemoryScopeControls {
                user: MemoryOptInControl::Enabled,
                project: MemoryOptInControl::Enabled,
                thread: MemoryOptInControl::Enabled,
            },
            MemoryScopeControls {
                user: MemoryOptInControl::Enabled,
                project: MemoryOptInControl::Enabled,
                thread: MemoryOptInControl::Unset,
            },
        );
        let decision = evaluate_memory_generation(MemoryGenerationEligibility {
            policy: &missing_thread,
            external_context_persistable: true,
            contains_secret_bearing_content: false,
            redaction_succeeded: true,
            remaining_budget_units: MEMORY_UNITS,
        });
        assert_eq!(decision.status(), MemoryGenerationStatus::Denied);
        assert_eq!(
            decision.denial_reason(),
            Some(MemoryGenerationDenialReason::MissingConsent)
        );
    }

    #[test]
    fn opt_in_memory_records_validate_evidence_scope_freshness_and_confidence() {
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.record_contract.validation]
        let record = opt_in_record(RecordFixture::valid());
        assert_eq!(record.id(), RECORD_ID);
        assert_eq!(record.summary_label(), SUMMARY_LABEL);
        assert_eq!(record.scope(), MemoryTrustScope::Global);
        assert_eq!(record.confidence().percent(), MEMORY_CONFIDENCE_PERCENT);

        let stale = opt_in_record(RecordFixture {
            freshness: MemoryFreshness::Stale,
            ..RecordFixture::valid()
        });
        assert_eq!(stale.freshness(), MemoryFreshness::Stale);
        assert!(matches!(
            OptInMemoryRecord::validate(&OptInMemoryRecordDraft {
                evidence_handle: "   ",
                ..RecordFixture::valid().draft()
            }),
            Err(MemoryError::Contract(_))
        ));
        assert_eq!(
            OptInMemoryRecord::validate(&OptInMemoryRecordDraft {
                scope: None,
                ..RecordFixture::valid().draft()
            }),
            Err(MemoryError::MissingRecordScope {
                record_id: String::from(RECORD_ID),
            })
        );
        assert_eq!(
            OptInMemoryRecord::validate(&OptInMemoryRecordDraft {
                confidence_percent: INVALID_CONFIDENCE_PERCENT,
                ..RecordFixture::valid().draft()
            }),
            Err(MemoryError::InvalidConfidence {
                min_percent: MIN_MEMORY_CONFIDENCE_PERCENT,
                max_percent: MAX_MEMORY_CONFIDENCE_PERCENT,
                actual_percent: INVALID_CONFIDENCE_PERCENT,
            })
        );
    }

    #[test]
    fn generation_eligibility_rejects_external_context_secrets_redaction_and_budget() {
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.generation_eligibility.exclusions]
        let policy = opt_in_policy(MemoryScopeControls {
            user: MemoryOptInControl::Enabled,
            project: MemoryOptInControl::Enabled,
            thread: MemoryOptInControl::Enabled,
        });
        assert_generation_denial(GenerationDenialFixture {
            policy: &policy,
            external_context_persistable: false,
            contains_secret_bearing_content: false,
            redaction_succeeded: true,
            remaining_budget_units: MEMORY_UNITS,
            reason: MemoryGenerationDenialReason::ExternalContext,
        });
        assert_generation_denial(GenerationDenialFixture {
            policy: &policy,
            external_context_persistable: true,
            contains_secret_bearing_content: true,
            redaction_succeeded: true,
            remaining_budget_units: MEMORY_UNITS,
            reason: MemoryGenerationDenialReason::SecretBearingContent,
        });
        assert_generation_denial(GenerationDenialFixture {
            policy: &policy,
            external_context_persistable: true,
            contains_secret_bearing_content: false,
            redaction_succeeded: false,
            remaining_budget_units: MEMORY_UNITS,
            reason: MemoryGenerationDenialReason::RedactionFailed,
        });
        assert_generation_denial(GenerationDenialFixture {
            policy: &policy,
            external_context_persistable: true,
            contains_secret_bearing_content: false,
            redaction_succeeded: true,
            remaining_budget_units: u32::default(),
            reason: MemoryGenerationDenialReason::BudgetExhausted,
        });
        let allowed = evaluate_memory_generation(MemoryGenerationEligibility {
            policy: &policy,
            external_context_persistable: true,
            contains_secret_bearing_content: false,
            redaction_succeeded: true,
            remaining_budget_units: MEMORY_UNITS,
        });
        assert_eq!(allowed.status(), MemoryGenerationStatus::Allowed);
        assert_eq!(allowed.denial_reason(), None);
    }

    #[test]
    fn opt_in_memory_injection_is_bounded_source_attributed_and_swappable() {
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.adapter_boundary.swappable]
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.injection_ledger.bounded]
        let policy = opt_in_policy(MemoryScopeControls {
            user: MemoryOptInControl::Enabled,
            project: MemoryOptInControl::Enabled,
            thread: MemoryOptInControl::Enabled,
        });
        let included = opt_in_record(RecordFixture {
            budget_units: SMALL_MEMORY_UNITS,
            ..RecordFixture::valid()
        });
        let redacted = opt_in_record(RecordFixture {
            id: SECOND_RECORD_ID,
            body_hash: SECOND_HASH,
            redaction_status: MemoryRecordRedactionStatus::Redacted,
            ..RecordFixture::valid()
        });
        let stale = opt_in_record(RecordFixture {
            id: THIRD_RECORD_ID,
            body_hash: THIRD_HASH,
            freshness: MemoryFreshness::Stale,
            ..RecordFixture::valid()
        });
        let over_budget = opt_in_record(RecordFixture {
            id: FOURTH_RECORD_ID,
            body_hash: SECOND_HASH,
            budget_units: MEMORY_UNITS,
            ..RecordFixture::valid()
        });
        let first = plan_opt_in_memory_injection(
            &[
                over_budget.clone(),
                stale.clone(),
                redacted.clone(),
                included.clone(),
            ],
            &policy,
            MemoryInjectionBudget::new(MEMORY_UNITS).expect("budget should validate"),
        )
        .expect("first memory injection plan should succeed");
        let second = plan_opt_in_memory_injection(
            &[over_budget, stale, redacted, included],
            &policy,
            MemoryInjectionBudget::new(MEMORY_UNITS).expect("budget should validate"),
        )
        .expect("second memory injection plan should succeed");

        assert_eq!(first, second);
        assert_eq!(first.ledger().len(), EXPECTED_FOUR_ITEMS);
        assert!(first.ledger().iter().any(|entry| {
            entry.record_id() == RECORD_ID
                && entry.source_thread_id() == THREAD_ID
                && entry.evidence_handle() == EVIDENCE_HANDLE
                && entry.state() == MemoryInjectionState::Included
        }));
        assert!(first.ledger().iter().any(|entry| {
            entry.record_id() == SECOND_RECORD_ID && entry.state() == MemoryInjectionState::Redacted
        }));
        assert!(first.ledger().iter().any(|entry| {
            entry.record_id() == THIRD_RECORD_ID && entry.state() == MemoryInjectionState::Expired
        }));
        assert!(first.ledger().iter().any(|entry| {
            entry.record_id() == FOURTH_RECORD_ID
                && entry.state() == MemoryInjectionState::OverBudget
        }));
    }

    #[test]
    fn disabled_memories_do_not_disable_required_guidance_or_policy() {
        // r[verify agent_harness_runtime.openai_codex_opt_in_memories.non_authoritative.guidance]
        let disabled = opt_in_policy(MemoryScopeControls {
            user: MemoryOptInControl::Disabled,
            project: MemoryOptInControl::Disabled,
            thread: MemoryOptInControl::Disabled,
        });
        let active = resolve_required_rules(RequiredRuleInput {
            memory_policy: &disabled,
            guidance_present: true,
            policy_present: false,
        });
        assert!(active.required_rules_active());
        assert!(!active.memory_required());

        let absent = resolve_required_rules(RequiredRuleInput {
            memory_policy: &disabled,
            guidance_present: false,
            policy_present: false,
        });
        assert!(!absent.required_rules_active());
        assert!(!absent.memory_required());
    }

