//! Deterministic filtering, ordering, and stable-identity repair.

pub(crate) fn filtered_indices(session: &super::state::PromptSession) -> Vec<usize> {
    debug_assert!(session.config.max_candidates > 0);
    debug_assert!(session.input.len() <= 4096);
    let Some(snapshot) = &session.completion else {
        return Vec::new();
    };
    let mut matches = Vec::with_capacity(snapshot.candidates.len());
    for (index, candidate) in snapshot.candidates.iter().enumerate() {
        let text = candidate.matching_text();
        let query = session.input.as_str();
        let score = match session.config.completion {
            super::model::PromptCompletionPolicy::None => None,
            super::model::PromptCompletionPolicy::CallerOrder => Some(0),
            super::model::PromptCompletionPolicy::Exact => text.eq_ignore_ascii_case(query).then_some(0),
            super::model::PromptCompletionPolicy::Prefix => {
                text.to_ascii_lowercase().starts_with(&query.to_ascii_lowercase()).then_some(0)
            }
            super::model::PromptCompletionPolicy::Fuzzy => {
                if query.is_empty() {
                    Some(0)
                } else {
                    crate::fuzzy::fuzzy_score(crate::fuzzy::FuzzyScoreInput { text, query })
                        .map(|matched| matched.score)
                }
            }
        };
        if let Some(score) = score {
            matches.push((index, score, candidate.source_order));
        }
    }
    if session.config.completion == super::model::PromptCompletionPolicy::Fuzzy {
        matches.sort_by(|left, right| right.1.cmp(&left.1).then(left.2.cmp(&right.2)));
    } else {
        matches.sort_by_key(|entry| entry.2);
    }
    matches.into_iter().map(|entry| entry.0).collect()
}

pub(crate) fn repair_active(session: &mut super::state::PromptSession) {
    let indices = filtered_indices(session);
    debug_assert!(indices.len() <= session.config.max_candidates.into());
    debug_assert!(session.completion.as_ref().is_none_or(|snapshot| snapshot.candidates.len() <= 256));
    let is_still_eligible = session.active.as_ref().is_some_and(|active| {
        indices.iter().any(|index| {
            session
                .completion
                .as_ref()
                .and_then(|snapshot| snapshot.candidates.get(*index))
                .is_some_and(|candidate| candidate.enabled && candidate.id == *active)
        })
    });
    if is_still_eligible {
        return;
    }
    session.active = indices.into_iter().find_map(|index| {
        session
            .completion
            .as_ref()
            .and_then(|snapshot| snapshot.candidates.get(index))
            .filter(|candidate| candidate.enabled)
            .map(|candidate| candidate.id.clone())
    });
}

pub(crate) fn candidate<'a>(
    session: &'a super::state::PromptSession,
    id: &super::model::PromptCandidateId,
) -> Option<&'a super::model::PromptCandidate> {
    session.completion.as_ref()?.candidates.iter().find(|candidate| candidate.id == *id)
}

pub(crate) fn move_active(
    session: &mut super::state::PromptSession,
    direction: super::state::PromptDirection,
) -> Result<(), super::state::PromptDiagnostic> {
    debug_assert!(session.config.max_candidates > 0);
    debug_assert!(session.lifecycle == super::state::PromptLifecycle::Open);
    let eligible = filtered_indices(session)
        .into_iter()
        .filter_map(|index| {
            session
                .completion
                .as_ref()
                .and_then(|snapshot| snapshot.candidates.get(index))
                .filter(|candidate| candidate.enabled)
                .map(|candidate| candidate.id.clone())
        })
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return Err(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::NoEligibleCandidate,
            "no eligible completion candidate is available",
        ));
    }
    let current = session.active.as_ref().and_then(|active| eligible.iter().position(|id| id == active));
    let final_index = eligible.len().saturating_sub(1);
    let current = current.unwrap_or(0);
    let is_at_boundary = match direction {
        super::state::PromptDirection::Previous => current == 0,
        super::state::PromptDirection::Next => current == final_index,
    };
    if is_at_boundary && session.config.movement == super::model::PromptMovementPolicy::Reject {
        return Err(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::MovementBoundary,
            "candidate movement reached a rejecting boundary",
        ));
    }
    let next = match (direction, session.config.movement, is_at_boundary) {
        (super::state::PromptDirection::Previous, super::model::PromptMovementPolicy::Wrap, true) => final_index,
        (super::state::PromptDirection::Next, super::model::PromptMovementPolicy::Wrap, true) => 0,
        (_, super::model::PromptMovementPolicy::Clamp, true) => current,
        (super::state::PromptDirection::Previous, _, false) => current.saturating_sub(1),
        (super::state::PromptDirection::Next, _, false) => current.saturating_add(1),
        (_, super::model::PromptMovementPolicy::Reject, true) => current,
    };
    session.active = eligible.get(next).cloned();
    Ok(())
}
