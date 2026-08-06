use super::*;

fn configuration() -> PromptConfiguration {
    PromptConfiguration {
        completion: PromptCompletionPolicy::Fuzzy,
        movement: PromptMovementPolicy::Wrap,
        history: PromptHistoryPolicy::Enabled,
        acceptance: PromptAcceptancePolicy::Candidate,
        allow_empty: false,
        trim_on_submit: true,
        max_candidates: 16,
    }
}

fn candidates(generation: u64) -> PromptCompletionSnapshot {
    PromptCompletionSnapshot {
        generation: PromptGeneration(generation),
        status: PromptCompletionStatus::Complete,
        candidates: vec![
            PromptCandidate::enabled("open", "Open file", 0),
            PromptCandidate::disabled("build", "Build project", "busy", 1),
            PromptCandidate::enabled("settings", "Settings", 2),
        ],
    }
}

fn session() -> PromptSession {
    let session = build_prompt_session(PromptSessionId::new("test.prompt"), "Command", None, configuration())
        .expect("valid fixture");
    reduce_prompt(&session, PromptMessage::ReplaceCandidates(candidates(1))).session
}

#[test]
fn malformed_session_and_snapshot_fail_closed() {
    let malformed = build_prompt_session(PromptSessionId::new(""), "", None, PromptConfiguration {
        max_candidates: 0,
        ..configuration()
    });
    assert!(malformed.is_err());
    assert!(malformed.err().is_some_and(|diagnostics| diagnostics.len() == 2));

    let original = session();
    let invalid = PromptCompletionSnapshot {
        generation: PromptGeneration(2),
        status: PromptCompletionStatus::Complete,
        candidates: vec![
            PromptCandidate::enabled("duplicate", "One", 0),
            PromptCandidate::enabled("duplicate", "Two", 1),
        ],
    };
    let transition = reduce_prompt(&original, PromptMessage::ReplaceCandidates(invalid));
    assert!(!transition.accepted);
    assert_eq!(transition.session, original);
    assert_eq!(transition.diagnostics[0].code, PromptDiagnosticCode::InvalidCandidates);
}

#[test]
fn matching_movement_and_stable_repair_are_deterministic() {
    let original = session();
    let queried = reduce_prompt(&original, PromptMessage::ReplaceInput {
        input: "set".to_string(),
        expected_revision: PromptInputRevision(0),
    });
    assert!(queried.accepted);
    assert_eq!(queried.session.active_candidate(), Some(&PromptCandidateId::new("settings")));
    assert_eq!(project_prompt(&queried.session).candidates.len(), 1);

    let reset = reduce_prompt(&queried.session, PromptMessage::Reset {
        input: String::new(),
        completion: Some(candidates(2)),
    })
    .session;
    let moved = reduce_prompt(&reset, PromptMessage::Move(PromptDirection::Next));
    assert_eq!(moved.session.active_candidate(), Some(&PromptCandidateId::new("settings")));
    let replaced = reduce_prompt(
        &moved.session,
        PromptMessage::ReplaceCandidates(PromptCompletionSnapshot {
            generation: PromptGeneration(3),
            status: PromptCompletionStatus::Complete,
            candidates: vec![PromptCandidate::enabled("settings", "Settings renamed", 0)],
        }),
    );
    assert_eq!(replaced.session.active_candidate(), Some(&PromptCandidateId::new("settings")));
    assert_eq!(project_prompt(&replaced.session).candidates[0].label, "Settings renamed");
}

#[test]
fn stale_candidates_and_activation_preserve_state() {
    let original = session();
    let stale = reduce_prompt(&original, PromptMessage::ReplaceCandidates(candidates(1)));
    assert!(!stale.accepted);
    assert_eq!(stale.diagnostics[0].code, PromptDiagnosticCode::StaleCandidates);
    assert_eq!(stale.session, original);

    let missing = reduce_prompt(&original, PromptMessage::SetActive(PromptCandidateId::new("missing")));
    assert!(!missing.accepted);
    assert_eq!(missing.diagnostics[0].code, PromptDiagnosticCode::UnknownCandidate);
    assert_eq!(missing.session, original);
}

#[test]
fn history_restores_draft_without_persistence() {
    let original = session();
    let drafted = reduce_prompt(&original, PromptMessage::ReplaceInput {
        input: "draft".to_string(),
        expected_revision: PromptInputRevision(0),
    })
    .session;
    let with_history = reduce_prompt(
        &drafted,
        PromptMessage::ReplaceHistory(PromptHistorySnapshot {
            namespace: PromptHistoryNamespace::new("commands"),
            generation: PromptGeneration(1),
            entries: vec!["first".to_string(), "second".to_string()],
        }),
    )
    .session;
    let older = reduce_prompt(&with_history, PromptMessage::MoveHistory(PromptHistoryDirection::Older));
    assert_eq!(older.session.input(), "second");
    assert!(older.invalidated);
    let restored = reduce_prompt(&older.session, PromptMessage::MoveHistory(PromptHistoryDirection::Newer));
    assert_eq!(restored.session.input(), "draft");
    assert_eq!(restored.session.history().map(|history| history.entries.len()), Some(2));
}

#[test]
fn disabled_and_sensitive_history_never_retains_entries() {
    for policy in [PromptHistoryPolicy::Disabled, PromptHistoryPolicy::Sensitive] {
        let configured = PromptConfiguration {
            history: policy,
            ..configuration()
        };
        let original =
            build_prompt_session(PromptSessionId::new("private"), "Private", None, configured).expect("valid policy");
        let replacement = reduce_prompt(
            &original,
            PromptMessage::ReplaceHistory(PromptHistorySnapshot {
                namespace: PromptHistoryNamespace::new("private"),
                generation: PromptGeneration(1),
                entries: vec!["protected".to_string()],
            }),
        );
        assert!(!replacement.accepted);
        assert!(replacement.session.history().is_none());
        assert!(!format!("{:?}", replacement.session).contains("protected"));
    }
    let sensitive = build_prompt_session(PromptSessionId::new("secret"), "Secret", None, PromptConfiguration {
        history: PromptHistoryPolicy::Sensitive,
        ..configuration()
    })
    .expect("valid sensitive session");
    assert_eq!(replay_input(&sensitive), Some("[redacted]".to_string()));
}

#[test]
fn candidate_submit_is_generation_and_validation_checked() {
    let original = session();
    let stale = reduce_prompt(&original, PromptMessage::Submit {
        input_revision: PromptInputRevision(0),
        candidate_generation: Some(PromptGeneration(0)),
    });
    assert!(!stale.accepted);
    assert_eq!(stale.diagnostics[0].code, PromptDiagnosticCode::StaleSubmission);

    let pending = reduce_prompt(
        &original,
        PromptMessage::ReplaceValidation(PromptValidationSnapshot {
            input_revision: PromptInputRevision(0),
            state: PromptValidationState::Pending,
        }),
    )
    .session;
    let blocked = reduce_prompt(&pending, PromptMessage::Submit {
        input_revision: PromptInputRevision(0),
        candidate_generation: Some(PromptGeneration(1)),
    });
    assert_eq!(blocked.diagnostics[0].code, PromptDiagnosticCode::ValidationPending);

    let accepted_validation = reduce_prompt(
        &pending,
        PromptMessage::ReplaceValidation(PromptValidationSnapshot {
            input_revision: PromptInputRevision(0),
            state: PromptValidationState::Accepted,
        }),
    )
    .session;
    let submitted = reduce_prompt(&accepted_validation, PromptMessage::Submit {
        input_revision: PromptInputRevision(0),
        candidate_generation: Some(PromptGeneration(1)),
    });
    assert!(submitted.accepted);
    assert!(matches!(submitted.output, Some(PromptOutput::Candidate { .. })));
    assert_eq!(submitted.session.lifecycle(), PromptLifecycle::ClosePending);
}

#[path = "acceptance.rs"]
mod acceptance;
