//! Pure prompt-session transition reducer.

/// Applies one deterministic prompt transition.
#[must_use]
pub fn reduce_prompt(
    session: &super::state::PromptSession,
    message: super::state::PromptMessage,
) -> super::state::PromptTransition {
    match message {
        super::state::PromptMessage::Reset { input, completion } => reset(session, input, completion),
        super::state::PromptMessage::ReplaceInput {
            input,
            expected_revision,
        } => replace_input(session, input, expected_revision),
        super::state::PromptMessage::ReplaceCandidates(snapshot) => replace_candidates(session, snapshot),
        super::state::PromptMessage::Move(direction) => move_candidate(session, direction),
        super::state::PromptMessage::SetActive(id) => set_active(session, id),
        super::state::PromptMessage::ReplaceHistory(snapshot) => replace_history(session, snapshot),
        super::state::PromptMessage::MoveHistory(direction) => move_history(session, direction),
        super::state::PromptMessage::ReplaceValidation(snapshot) => replace_validation(session, snapshot),
        super::state::PromptMessage::Submit {
            input_revision,
            candidate_generation,
        } => super::acceptance::submit(session, input_revision, candidate_generation),
        super::state::PromptMessage::Cancel => cancel(session),
        super::state::PromptMessage::AcknowledgeClose => acknowledge_close(session),
    }
}

pub(super) fn accepted(
    session: super::state::PromptSession,
    output: Option<super::state::PromptOutput>,
    invalidated: bool,
) -> super::state::PromptTransition {
    super::state::PromptTransition {
        session,
        accepted: true,
        output,
        diagnostics: Vec::new(),
        invalidated,
    }
}

pub(super) fn rejected(
    session: &super::state::PromptSession,
    diagnostic: super::state::PromptDiagnostic,
) -> super::state::PromptTransition {
    super::state::PromptTransition {
        session: session.clone(),
        accepted: false,
        output: None,
        diagnostics: vec![diagnostic],
        invalidated: false,
    }
}

fn reset(
    session: &super::state::PromptSession,
    input: String,
    completion: Option<super::model::PromptCompletionSnapshot>,
) -> super::state::PromptTransition {
    debug_assert!(session.config.max_candidates > 0);
    debug_assert!(session.id.is_valid());
    if !super::model::text_is_bounded(&input) {
        return rejected(session, invalid_input());
    }
    let mut next = session.clone();
    next.lifecycle = super::state::PromptLifecycle::Open;
    next.input = input;
    next.input_revision = super::model::PromptInputRevision(next.input_revision.0.saturating_add(1));
    next.completion = None;
    next.active = None;
    next.history_position = None;
    next.history_draft = None;
    next.validation = None;
    if let Some(snapshot) = completion {
        if let Some(diagnostic) = super::checks::validate_completion(&next, &snapshot) {
            return rejected(session, diagnostic);
        }
        next.completion = Some(snapshot);
        super::matching::repair_active(&mut next);
    }
    accepted(next, None, true)
}

fn replace_input(
    session: &super::state::PromptSession,
    input: String,
    expected: super::model::PromptInputRevision,
) -> super::state::PromptTransition {
    if let Some(diagnostic) = super::checks::validate_input(session, &input, expected) {
        return rejected(session, diagnostic);
    }
    let mut next = session.clone();
    next.input = input;
    next.input_revision = super::model::PromptInputRevision(expected.0.saturating_add(1));
    next.history_position = None;
    next.history_draft = None;
    next.validation = None;
    super::matching::repair_active(&mut next);
    accepted(next, None, true)
}

fn replace_candidates(
    session: &super::state::PromptSession,
    snapshot: super::model::PromptCompletionSnapshot,
) -> super::state::PromptTransition {
    if let Some(diagnostic) = super::checks::validate_completion(session, &snapshot) {
        return rejected(session, diagnostic);
    }
    let mut next = session.clone();
    next.completion = Some(snapshot);
    super::matching::repair_active(&mut next);
    accepted(next, None, true)
}

fn move_candidate(
    session: &super::state::PromptSession,
    direction: super::state::PromptDirection,
) -> super::state::PromptTransition {
    if session.lifecycle != super::state::PromptLifecycle::Open {
        return rejected(session, invalid_lifecycle());
    }
    let mut next = session.clone();
    match super::matching::move_active(&mut next, direction) {
        Ok(()) => accepted(next, None, true),
        Err(diagnostic) => rejected(session, diagnostic),
    }
}

fn set_active(
    session: &super::state::PromptSession,
    id: super::model::PromptCandidateId,
) -> super::state::PromptTransition {
    debug_assert!(session.config.max_candidates > 0);
    debug_assert!(session.id.is_valid());
    if session.lifecycle != super::state::PromptLifecycle::Open {
        return rejected(session, invalid_lifecycle());
    }
    let is_visible = super::matching::filtered_indices(session).into_iter().any(|index| {
        session
            .completion
            .as_ref()
            .and_then(|snapshot| snapshot.candidates.get(index))
            .is_some_and(|candidate| candidate.id == id)
    });
    if !is_visible {
        return rejected(
            session,
            super::state::PromptDiagnostic::new(
                super::state::PromptDiagnosticCode::UnknownCandidate,
                "candidate is missing or filtered from the current projection",
            ),
        );
    }
    let mut next = session.clone();
    next.active = Some(id);
    accepted(next, None, true)
}

fn replace_history(
    session: &super::state::PromptSession,
    snapshot: super::model::PromptHistorySnapshot,
) -> super::state::PromptTransition {
    if let Some(diagnostic) = super::checks::validate_history(session, &snapshot) {
        return rejected(session, diagnostic);
    }
    let mut next = session.clone();
    next.history = Some(snapshot);
    next.history_position = None;
    next.history_draft = None;
    accepted(next, None, false)
}

fn move_history(
    session: &super::state::PromptSession,
    direction: super::state::PromptHistoryDirection,
) -> super::state::PromptTransition {
    let mut next = session.clone();
    match super::navigation::navigate(&mut next, direction) {
        Ok(()) => accepted(next, None, true),
        Err(diagnostic) => rejected(session, diagnostic),
    }
}

fn replace_validation(
    session: &super::state::PromptSession,
    snapshot: super::model::PromptValidationSnapshot,
) -> super::state::PromptTransition {
    if let Some(diagnostic) = super::checks::validate_observation(session, &snapshot) {
        return rejected(session, diagnostic);
    }
    let mut next = session.clone();
    next.validation = Some(snapshot);
    accepted(next, None, true)
}

fn cancel(session: &super::state::PromptSession) -> super::state::PromptTransition {
    if session.lifecycle != super::state::PromptLifecycle::Open {
        return rejected(session, invalid_lifecycle());
    }
    close_with_output(session, super::state::PromptOutput::Cancelled {
        session_id: session.id.clone(),
    })
}

fn acknowledge_close(session: &super::state::PromptSession) -> super::state::PromptTransition {
    if session.lifecycle != super::state::PromptLifecycle::ClosePending {
        return rejected(session, invalid_lifecycle());
    }
    let mut next = session.clone();
    next.lifecycle = super::state::PromptLifecycle::Closed;
    accepted(next, None, true)
}

pub(super) fn close_with_output(
    session: &super::state::PromptSession,
    output: super::state::PromptOutput,
) -> super::state::PromptTransition {
    let mut next = session.clone();
    next.lifecycle = super::state::PromptLifecycle::ClosePending;
    accepted(next, Some(output), true)
}

fn invalid_input() -> super::state::PromptDiagnostic {
    super::state::PromptDiagnostic::new(
        super::state::PromptDiagnosticCode::InvalidInput,
        "input is excessive or malformed",
    )
}

pub(super) fn invalid_lifecycle() -> super::state::PromptDiagnostic {
    super::state::PromptDiagnostic::new(
        super::state::PromptDiagnosticCode::InvalidLifecycle,
        "message is unavailable in the current prompt lifecycle",
    )
}

pub(super) fn stale_submission() -> super::state::PromptDiagnostic {
    super::state::PromptDiagnostic::new(
        super::state::PromptDiagnosticCode::StaleSubmission,
        "submission observed stale input or completion generation",
    )
}

pub(super) fn no_candidate() -> super::state::PromptDiagnostic {
    super::state::PromptDiagnostic::new(
        super::state::PromptDiagnosticCode::NoEligibleCandidate,
        "no active completion candidate can be submitted",
    )
}
