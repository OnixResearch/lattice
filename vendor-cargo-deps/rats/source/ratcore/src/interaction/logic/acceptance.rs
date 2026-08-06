//! Generation-checked candidate and free-text acceptance.

pub(super) fn submit(
    session: &super::state::PromptSession,
    input_revision: super::model::PromptInputRevision,
    candidate_generation: Option<super::model::PromptGeneration>,
) -> super::state::PromptTransition {
    debug_assert!(session.config.max_candidates > 0);
    debug_assert!(session.id.is_valid());
    if session.lifecycle != super::state::PromptLifecycle::Open {
        return super::reducer::rejected(session, super::reducer::invalid_lifecycle());
    }
    if input_revision != session.input_revision {
        return super::reducer::rejected(session, super::reducer::stale_submission());
    }
    if let Some(validation) = &session.validation {
        match validation.state {
            super::model::PromptValidationState::Pending => {
                return super::reducer::rejected(
                    session,
                    super::state::PromptDiagnostic::new(
                        super::state::PromptDiagnosticCode::ValidationPending,
                        "input validation is pending",
                    ),
                );
            }
            super::model::PromptValidationState::Rejected(_) => {
                return super::reducer::rejected(
                    session,
                    super::state::PromptDiagnostic::new(
                        super::state::PromptDiagnosticCode::ValidationRejected,
                        "input validation rejected submission",
                    ),
                );
            }
            super::model::PromptValidationState::Accepted => {}
        }
    }
    match session.config.acceptance {
        super::model::PromptAcceptancePolicy::Candidate => submit_candidate(session, candidate_generation),
        super::model::PromptAcceptancePolicy::FreeText => submit_text(session),
    }
}

fn submit_candidate(
    session: &super::state::PromptSession,
    observed: Option<super::model::PromptGeneration>,
) -> super::state::PromptTransition {
    debug_assert!(session.config.acceptance == super::model::PromptAcceptancePolicy::Candidate);
    debug_assert!(session.completion.as_ref().is_none_or(|snapshot| snapshot.candidates.len() <= 256));
    let Some(snapshot) = &session.completion else {
        return super::reducer::rejected(session, super::reducer::no_candidate());
    };
    if observed != Some(snapshot.generation) {
        return super::reducer::rejected(session, super::reducer::stale_submission());
    }
    if snapshot.status == super::model::PromptCompletionStatus::Pending {
        return super::reducer::rejected(
            session,
            super::state::PromptDiagnostic::new(
                super::state::PromptDiagnosticCode::ValidationPending,
                "completion catalog is pending",
            ),
        );
    }
    let Some(id) = &session.active else {
        return super::reducer::rejected(session, super::reducer::no_candidate());
    };
    let Some(candidate) = super::matching::candidate(session, id) else {
        return super::reducer::rejected(session, super::reducer::no_candidate());
    };
    if !candidate.enabled {
        return super::reducer::rejected(
            session,
            super::state::PromptDiagnostic::new(
                super::state::PromptDiagnosticCode::DisabledCandidate,
                "active candidate is disabled",
            ),
        );
    }
    let output = super::state::PromptOutput::Candidate {
        id: candidate.id.clone(),
        value: candidate.label.clone(),
        candidate_generation: snapshot.generation,
        input_revision: session.input_revision,
    };
    super::reducer::close_with_output(session, output)
}

fn submit_text(session: &super::state::PromptSession) -> super::state::PromptTransition {
    debug_assert!(session.config.acceptance == super::model::PromptAcceptancePolicy::FreeText);
    debug_assert!(super::model::text_is_bounded(&session.input));
    let value = if session.config.trim_on_submit {
        session.input.trim()
    } else {
        session.input.as_str()
    };
    if value.is_empty() && !session.config.allow_empty {
        return super::reducer::rejected(
            session,
            super::state::PromptDiagnostic::new(
                super::state::PromptDiagnosticCode::EmptyInput,
                "empty input is not accepted",
            ),
        );
    }
    super::reducer::close_with_output(session, super::state::PromptOutput::FreeText {
        value: value.to_string(),
        input_revision: session.input_revision,
    })
}
