//! Action-specific compatibility adapter for the production command palette.

/// Runs the accepted command-palette transition through the shared prompt reducer.
///
/// The existing command model remains the source-compatible projection during the
/// staged migration. The shared reducer decides equivalent prompt behavior first;
/// focused parity tests reject any accepted/output divergence.
#[must_use]
pub fn reduce_command_palette_through_prompt(
    model: &crate::command_palette::CommandPaletteModel,
    message: crate::command_palette::CommandPaletteMessage,
) -> crate::command_palette::CommandPaletteTransition {
    let prompt = prompt_from_palette(model);
    let prompt_transition = prompt.and_then(|session| apply_palette_message(&session, &message));
    let legacy = crate::command_palette::reduce_command_palette(model, message);
    if let Some(prompt_transition) = prompt_transition {
        debug_assert_eq!(prompt_transition.accepted, legacy.accepted);
        debug_assert!(outputs_agree(prompt_transition.output.as_ref(), legacy.output.as_ref()));
    }
    legacy
}

fn prompt_from_palette(model: &crate::command_palette::CommandPaletteModel) -> Option<super::state::PromptSession> {
    debug_assert!(model.candidates().len() <= 256);
    debug_assert!(model.query().len() <= 4096);
    let configuration = super::model::PromptConfiguration {
        completion: super::model::PromptCompletionPolicy::Fuzzy,
        movement: match model.navigation_policy() {
            crate::command_palette::CommandPaletteNavigationPolicy::Wrap => super::model::PromptMovementPolicy::Wrap,
            crate::command_palette::CommandPaletteNavigationPolicy::Clamp => super::model::PromptMovementPolicy::Clamp,
            crate::command_palette::CommandPaletteNavigationPolicy::Reject => {
                super::model::PromptMovementPolicy::Reject
            }
        },
        history: super::model::PromptHistoryPolicy::Disabled,
        acceptance: super::model::PromptAcceptancePolicy::Candidate,
        allow_empty: false,
        trim_on_submit: false,
        max_candidates: 256,
    };
    let initial = super::checks::build_prompt_session(
        super::model::PromptSessionId::new("meteorite.command-palette"),
        "Command palette",
        None,
        configuration,
    )
    .ok()?;
    let reset = super::reducer::reduce_prompt(&initial, super::state::PromptMessage::Reset {
        input: model.query().to_string(),
        completion: Some(snapshot(model.candidates(), super::model::PromptGeneration(1))?),
    });
    if !reset.accepted {
        return None;
    }
    let mut prompt = reset.session;
    if let Some(active) = model.active_action_id() {
        let activated = super::reducer::reduce_prompt(
            &prompt,
            super::state::PromptMessage::SetActive(super::model::PromptCandidateId::new(active.as_str())),
        );
        if activated.accepted {
            prompt = activated.session;
        }
    }
    Some(prompt)
}

fn snapshot(
    candidates: &[crate::command_palette::CommandPaletteCandidate],
    generation: super::model::PromptGeneration,
) -> Option<super::model::PromptCompletionSnapshot> {
    debug_assert!(generation.0 > 0);
    debug_assert!(candidates.len() <= 256);
    if candidates.len() > 256 {
        return None;
    }
    let projected = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let source_order = u32::try_from(index).ok()?;
            Some(candidate.disabled_reason().map_or_else(
                || {
                    super::model::PromptCandidate::enabled(
                        candidate.action_id().as_str(),
                        candidate.label(),
                        source_order,
                    )
                },
                |reason| {
                    super::model::PromptCandidate::disabled(
                        candidate.action_id().as_str(),
                        candidate.label(),
                        reason,
                        source_order,
                    )
                },
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(super::model::PromptCompletionSnapshot {
        generation,
        status: super::model::PromptCompletionStatus::Complete,
        candidates: projected,
    })
}

fn apply_palette_message(
    session: &super::state::PromptSession,
    message: &crate::command_palette::CommandPaletteMessage,
) -> Option<super::state::PromptTransition> {
    match message {
        crate::command_palette::CommandPaletteMessage::Open => {
            Some(super::reducer::reduce_prompt(session, super::state::PromptMessage::Reset {
                input: String::new(),
                completion: session.completion().cloned(),
            }))
        }
        crate::command_palette::CommandPaletteMessage::ReplaceQuery(query) => {
            Some(super::reducer::reduce_prompt(session, super::state::PromptMessage::ReplaceInput {
                input: query.clone(),
                expected_revision: session.input_revision(),
            }))
        }
        crate::command_palette::CommandPaletteMessage::ReplaceCandidates(candidates) => {
            let generation = super::model::PromptGeneration(
                session.completion().map_or(1, |current| current.generation.0.saturating_add(1)),
            );
            Some(super::reducer::reduce_prompt(
                session,
                super::state::PromptMessage::ReplaceCandidates(snapshot(candidates, generation)?),
            ))
        }
        crate::command_palette::CommandPaletteMessage::Move(direction) => {
            let direction = match direction {
                crate::command_palette::CommandPaletteDirection::Previous => super::state::PromptDirection::Previous,
                crate::command_palette::CommandPaletteDirection::Next => super::state::PromptDirection::Next,
            };
            Some(super::reducer::reduce_prompt(session, super::state::PromptMessage::Move(direction)))
        }
        crate::command_palette::CommandPaletteMessage::Hover(action) => Some(super::reducer::reduce_prompt(
            session,
            super::state::PromptMessage::SetActive(super::model::PromptCandidateId::new(action.as_str())),
        )),
        crate::command_palette::CommandPaletteMessage::Activate => Some(submit(session)),
        crate::command_palette::CommandPaletteMessage::ActivateAction(action) => {
            let activated = super::reducer::reduce_prompt(
                session,
                super::state::PromptMessage::SetActive(super::model::PromptCandidateId::new(action.as_str())),
            );
            if activated.accepted {
                Some(submit(&activated.session))
            } else {
                Some(activated)
            }
        }
        crate::command_palette::CommandPaletteMessage::Cancel => {
            Some(super::reducer::reduce_prompt(session, super::state::PromptMessage::Cancel))
        }
    }
}

fn submit(session: &super::state::PromptSession) -> super::state::PromptTransition {
    super::reducer::reduce_prompt(session, super::state::PromptMessage::Submit {
        input_revision: session.input_revision(),
        candidate_generation: session.completion().map(|snapshot| snapshot.generation),
    })
}

fn outputs_agree(
    prompt: Option<&super::state::PromptOutput>,
    palette: Option<&crate::command_palette::CommandPaletteOutput>,
) -> bool {
    match (prompt, palette) {
        (None, None) => true,
        (
            Some(super::state::PromptOutput::Cancelled { .. }),
            Some(crate::command_palette::CommandPaletteOutput::Close),
        ) => true,
        (
            Some(super::state::PromptOutput::Candidate { id, .. }),
            Some(crate::command_palette::CommandPaletteOutput::Select(action)),
        ) => id.as_str() == action.as_str(),
        _ => false,
    }
}
