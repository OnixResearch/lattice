    // r[verify agent_harness_runtime.session_restore.verification]
    // r[verify agent_harness_runtime.session_restore.verification.closeout]
    use super::*;

    const VALID_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const SECOND_HASH: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const BAD_HASH: &str = "not-a-hash";
    const TARGET_LIVE: &str = "live-pane";
    const TARGET_LAYOUT: &str = "layout-pane";
    const TARGET_HISTORY: &str = "history-pane";
    const TARGET_NATIVE: &str = "native-agent";
    const TARGET_HANDOFF: &str = "handoff-pane";
    const TARGET_FALLBACK: &str = "fallback-pane";
    const SOURCE_LABEL: &str = "host-session-store";
    const AGENT_LABEL: &str = "codex-agent";
    const LINE_COUNT: u32 = 8;
    const RAW_HISTORY: &str = "raw history";
    const CREDENTIAL: &str = "api_key=secret";
    const SHELL_COMMAND: &str = "ssh private-host";
    const FULL_PATH: &str = "/home/example/.agent/session.json";
    const TRANSCRIPT: &str = "full transcript body";
    const EXPECTED_MATRIX_TARGETS: usize = 5;
    const EXPECTED_DUPLICATE_TARGETS: usize = 2;

    #[test]
    fn restore_matrix_claims_explicit_survival_boundaries() {
        // r[verify agent_harness_runtime.session_restore.restore_matrix.claims]
        let native = native_ref(
            VALID_HASH,
            NativeSessionFreshness::Fresh,
            NativeResumeCapability::Supported,
        );
        let history = safe_history(true, false);
        let targets = [
            target(TARGET_LIVE, RestoreMode::LiveProcess, None, None),
            target(TARGET_LAYOUT, RestoreMode::SnapshotLayout, None, None),
            target(
                TARGET_HISTORY,
                RestoreMode::HistoryReplay,
                None,
                Some(&history),
            ),
            target(
                TARGET_NATIVE,
                RestoreMode::NativeSessionResume,
                Some(&native),
                None,
            ),
            target(TARGET_HANDOFF, RestoreMode::HandoffTransfer, None, None),
        ];
        let plan = plan_restore(input(&targets, full_host(), history_policy(true, false)))
            .expect("restore matrix should plan");

        assert_eq!(plan.targets().len(), EXPECTED_MATRIX_TARGETS);
        assert_claim(
            &plan,
            TARGET_LIVE,
            RestoreCapabilityClaims::process_survived,
        );
        assert_claim(
            &plan,
            TARGET_LAYOUT,
            RestoreCapabilityClaims::layout_reconstructed,
        );
        assert_claim(
            &plan,
            TARGET_HISTORY,
            RestoreCapabilityClaims::display_replayed,
        );
        assert_claim(&plan, TARGET_NATIVE, |claims| {
            claims.native_resume_requested()
        });
        assert_claim(
            &plan,
            TARGET_HANDOFF,
            RestoreCapabilityClaims::handoff_requested,
        );
        assert!(plan.targets().iter().all(|target| {
            !target.claims().process_survived()
                || target.requested_mode() == RestoreMode::LiveProcess
        }));
    }

    #[test]
    fn invalid_native_references_degrade_or_reject_safely() {
        // r[verify agent_harness_runtime.session_restore.native_session_reference.invalid]
        assert_native_error(
            reference_draft(
                None,
                Some(AGENT_LABEL),
                Some(VALID_HASH),
                NativeSessionFreshness::Fresh,
                NativeResumeCapability::Supported,
                None,
            ),
            RestoreDegradedReason::MissingNativeReference,
        );
        assert_native_error(
            reference_draft(
                Some(SOURCE_LABEL),
                Some(AGENT_LABEL),
                Some(BAD_HASH),
                NativeSessionFreshness::Fresh,
                NativeResumeCapability::Supported,
                None,
            ),
            RestoreDegradedReason::MalformedNativeReference,
        );
        assert_native_error(
            reference_draft(
                Some(SOURCE_LABEL),
                Some(AGENT_LABEL),
                Some(VALID_HASH),
                NativeSessionFreshness::Fresh,
                NativeResumeCapability::Supported,
                Some("secret-session-id"),
            ),
            RestoreDegradedReason::SensitiveNativeReference,
        );

        let stale = native_ref(
            VALID_HASH,
            NativeSessionFreshness::Stale,
            NativeResumeCapability::Supported,
        );
        let unsupported = native_ref(
            SECOND_HASH,
            NativeSessionFreshness::Fresh,
            NativeResumeCapability::Unsupported,
        );
        let first_duplicate = native_ref(
            VALID_HASH,
            NativeSessionFreshness::Fresh,
            NativeResumeCapability::Supported,
        );
        let second_duplicate = native_ref(
            VALID_HASH,
            NativeSessionFreshness::Fresh,
            NativeResumeCapability::Supported,
        );
        let stale_plan = single_native_plan(&stale);
        let unsupported_plan = single_native_plan(&unsupported);
        assert_eq!(
            stale_plan.targets()[0].degraded_reason(),
            Some(RestoreDegradedReason::StaleNativeReference)
        );
        assert_eq!(
            unsupported_plan.targets()[0].degraded_reason(),
            Some(RestoreDegradedReason::UnsupportedNativeReference)
        );

        let duplicate_targets = [
            target(
                "native-one",
                RestoreMode::NativeSessionResume,
                Some(&first_duplicate),
                None,
            ),
            target(
                "native-two",
                RestoreMode::NativeSessionResume,
                Some(&second_duplicate),
                None,
            ),
        ];
        let duplicate_plan = plan_restore(input(
            &duplicate_targets,
            full_host(),
            history_policy(false, false),
        ))
        .expect("duplicate native references should plan with degradation");
        assert_eq!(duplicate_plan.targets().len(), EXPECTED_DUPLICATE_TARGETS);
        assert_eq!(
            duplicate_plan.targets()[1].degraded_reason(),
            Some(RestoreDegradedReason::DuplicatedNativeReference)
        );
    }

    #[test]
    fn history_replay_is_opt_in_redacted_and_display_only() {
        // r[verify agent_harness_runtime.session_restore.history_replay.safe]
        let safe = safe_history(true, false);
        let model_visible = safe_history(true, true);
        let disabled_targets = [target(
            TARGET_HISTORY,
            RestoreMode::HistoryReplay,
            None,
            Some(&safe),
        )];
        let disabled = plan_restore(input(
            &disabled_targets,
            full_host(),
            history_policy(false, false),
        ))
        .expect("disabled history should degrade");
        assert_eq!(
            disabled.targets()[0].degraded_reason(),
            Some(RestoreDegradedReason::HistoryReplayDisabled)
        );

        let display_only = plan_restore(input(
            &disabled_targets,
            full_host(),
            history_policy(true, false),
        ))
        .expect("safe display replay should plan");
        assert!(display_only.targets()[0].claims().display_replayed());
        assert!(!display_only.targets()[0].claims().process_survived());
        assert!(!display_only.targets()[0].claims().native_resume_requested());
        assert!(!display_only.targets()[0].claims().model_context_admitted());

        let context_targets = [target(
            TARGET_HISTORY,
            RestoreMode::HistoryReplay,
            None,
            Some(&model_visible),
        )];
        let admitted = plan_restore(input(
            &context_targets,
            full_host(),
            history_policy(true, true),
        ))
        .expect("policy-admitted replay should plan");
        assert!(admitted.targets()[0].claims().model_context_admitted());

        assert_eq!(
            HistoryReplay::validate(&unsafe_history(None)),
            Err(SessionRestoreError::UnsafeHistoryReplay)
        );
        assert_eq!(
            HistoryReplay::validate(&unsafe_history(Some("secret"))),
            Err(SessionRestoreError::UnsafeHistoryReplay)
        );
    }

    #[test]
    fn planning_has_no_ambient_io_and_is_deterministic() {
        // r[verify agent_harness_runtime.session_restore.host_owned_execution.no_ambient_io]
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
        let first = plan_restore(input(&targets, full_host(), history_policy(false, false)))
            .expect("first restore plan should succeed");
        let second = plan_restore(input(&targets, full_host(), history_policy(false, false)))
            .expect("second restore plan should succeed");
        assert_eq!(first, second);
        assert!(!first.effects().spawned_process());
        assert!(!first.effects().read_files());
        assert!(!first.effects().opened_pty());
        assert!(!first.effects().constructed_shell_command());
        assert!(!first.effects().inspected_environment());
    }
