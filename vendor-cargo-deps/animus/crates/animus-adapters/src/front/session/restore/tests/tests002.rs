
    #[test]
    fn degraded_restore_cannot_claim_continuity_or_survival() {
        // r[verify agent_harness_runtime.session_restore.degraded_outcomes.no_false_claims]
        let missing_targets = [target(
            TARGET_NATIVE,
            RestoreMode::NativeSessionResume,
            None,
            None,
        )];
        let missing = plan_restore(input(
            &missing_targets,
            full_host(),
            history_policy(false, false),
        ))
        .expect("missing native reference should degrade");
        let denied_handoff_targets = [target(
            TARGET_HANDOFF,
            RestoreMode::HandoffTransfer,
            None,
            None,
        )];
        let denied_handoff = plan_restore(input(
            &denied_handoff_targets,
            no_handoff_host(),
            history_policy(false, false),
        ))
        .expect("unsupported handoff should degrade");
        let fallback_targets = [target(TARGET_FALLBACK, RestoreMode::Fallback, None, None)];
        let fallback = plan_restore(input(
            &fallback_targets,
            full_host(),
            history_policy(false, false),
        ))
        .expect("fallback should plan");

        assert_no_false_claims(
            &missing.targets()[0],
            RestoreDegradedReason::MissingNativeReference,
        );
        assert_no_false_claims(
            &denied_handoff.targets()[0],
            RestoreDegradedReason::FailedHandoffClaim,
        );
        assert_no_false_claims(&fallback.targets()[0], RestoreDegradedReason::SafeFallback);
    }

    #[test]
    fn restore_receipts_hash_safe_metadata_and_reject_leaks() {
        let native = native_ref(
            VALID_HASH,
            NativeSessionFreshness::Fresh,
            NativeResumeCapability::Supported,
        );
        let targets = [target(
            TARGET_NATIVE,
            RestoreMode::NativeSessionResume,
            Some(&native),
            None,
        )];
        let plan = plan_restore(input(&targets, full_host(), history_policy(false, false)))
            .expect("safe plan should succeed");
        let first = RestoreReceipt::validate(&receipt(&plan, ReceiptLeakFixture::none()))
            .expect("safe restore receipt should validate");
        let second = RestoreReceipt::validate(&receipt(&plan, ReceiptLeakFixture::none()))
            .expect("safe restore receipt should validate deterministically");
        assert_eq!(first.receipt_hash(), second.receipt_hash());
        assert_eq!(first.plan(), &plan);
        assert_receipt_leak(&plan, ReceiptLeakFixture::raw_history());
        assert_receipt_leak(&plan, ReceiptLeakFixture::credential());
        assert_receipt_leak(&plan, ReceiptLeakFixture::shell_command());
        assert_receipt_leak(&plan, ReceiptLeakFixture::full_path());
        assert_receipt_leak(&plan, ReceiptLeakFixture::transcript());
    }

    fn native_ref(
        hash: &'static str,
        freshness: NativeSessionFreshness,
        resume_capability: NativeResumeCapability,
    ) -> NativeSessionReference {
        NativeSessionReference::validate(&reference_draft(
            Some(SOURCE_LABEL),
            Some(AGENT_LABEL),
            Some(hash),
            freshness,
            resume_capability,
            None,
        ))
        .expect("native session reference should validate")
    }

    #[expect(clippy::too_many_arguments, reason = "fixture mirrors session draft fields")]
    const fn reference_draft(
        source_label: Option<&'static str>,
        agent_label: Option<&'static str>,
        value_hash: Option<&'static str>,
        freshness: NativeSessionFreshness,
        resume_capability: NativeResumeCapability,
        sensitive_value: Option<&'static str>,
    ) -> NativeSessionReferenceDraft<'static> {
        NativeSessionReferenceDraft {
            source_label,
            agent_label,
            kind: NativeSessionReferenceKind::AgentConversation,
            value_hash,
            freshness,
            uniqueness_scope: NativeSessionUniquenessScope::Host,
            resume_capability,
            sensitive_value,
        }
    }

    fn assert_native_error(draft: NativeSessionReferenceDraft<'_>, reason: RestoreDegradedReason) {
        assert_eq!(
            NativeSessionReference::validate(&draft),
            Err(SessionRestoreError::InvalidNativeReference { reason })
        );
    }

    fn single_native_plan(reference: &NativeSessionReference) -> RestorePlan {
        let targets = [target(
            TARGET_NATIVE,
            RestoreMode::NativeSessionResume,
            Some(reference),
            None,
        )];
        plan_restore(input(&targets, full_host(), history_policy(false, false)))
            .expect("single native restore should plan")
    }

    fn safe_history(enabled: bool, model_visible: bool) -> HistoryReplay {
        HistoryReplay::validate(&HistoryReplayDraft {
            enabled,
            bounded_line_count: LINE_COUNT,
            truncated: true,
            redaction_status: HistoryRedactionStatus::Redacted,
            storage_class: HistoryStorageClass::Ephemeral,
            model_visible,
            secret_history: None,
        })
        .expect("safe history replay should validate")
    }

    const fn unsafe_history(secret_history: Option<&'static str>) -> HistoryReplayDraft<'static> {
        HistoryReplayDraft {
            enabled: true,
            bounded_line_count: LINE_COUNT,
            truncated: false,
            redaction_status: HistoryRedactionStatus::Unredacted,
            storage_class: HistoryStorageClass::Persistent,
            model_visible: true,
            secret_history,
        }
    }

    const fn target<'a>(
        target_label: &'a str,
        requested_mode: RestoreMode,
        native_reference: Option<&'a NativeSessionReference>,
        history_replay: Option<&'a HistoryReplay>,
    ) -> RestoreTargetDraft<'a> {
        RestoreTargetDraft {
            target_label,
            requested_mode,
            native_reference,
            history_replay,
        }
    }

    const fn input<'a>(
        targets: &'a [RestoreTargetDraft<'a>],
        host: RestoreHostCapabilities,
        policy: RestorePolicy,
    ) -> RestorePlanningInput<'a> {
        RestorePlanningInput {
            host,
            policy,
            targets,
        }
    }

    const fn full_host() -> RestoreHostCapabilities {
        RestoreHostCapabilities::all()
    }

    const fn no_handoff_host() -> RestoreHostCapabilities {
        RestoreHostCapabilities::without_handoff()
    }

    const fn history_policy(
        history_replay_enabled: bool,
        admit_replay_as_context: bool,
    ) -> RestorePolicy {
        RestorePolicy {
            history_replay_enabled,
            admit_replay_as_context,
        }
    }

    fn assert_claim(
        plan: &RestorePlan,
        target_label: &str,
        predicate: fn(RestoreCapabilityClaims) -> bool,
    ) {
        let target = plan
            .targets()
            .iter()
            .find(|candidate| candidate.target_label() == target_label)
            .expect("target should exist");
        assert_eq!(target.status(), RestoreOutcomeStatus::Restored);
        assert!(predicate(target.claims()));
    }

    fn assert_no_false_claims(target: &RestoreTargetPlan, reason: RestoreDegradedReason) {
        assert_eq!(target.status(), RestoreOutcomeStatus::Degraded);
        assert_eq!(target.degraded_reason(), Some(reason));
        assert!(!target.claims().process_survived());
        assert!(!target.claims().handoff_requested());
        assert!(!target.claims().native_resume_requested());
    }
