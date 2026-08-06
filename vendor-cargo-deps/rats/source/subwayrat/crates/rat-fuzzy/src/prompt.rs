//! Thin terminal adapter over the shared prompt reducer.

pub(crate) fn new_session() -> ratcore::prompt_completion::PromptSession {
    ratcore::prompt_completion::build_prompt_session(
        ratcore::prompt_completion::PromptSessionId::new("subwayrat.fuzzy"),
        "Find",
        None,
        ratcore::prompt_completion::PromptConfiguration {
            completion: ratcore::prompt_completion::PromptCompletionPolicy::Fuzzy,
            movement: ratcore::prompt_completion::PromptMovementPolicy::Clamp,
            history: ratcore::prompt_completion::PromptHistoryPolicy::Disabled,
            acceptance: ratcore::prompt_completion::PromptAcceptancePolicy::Candidate,
            allow_empty: false,
            trim_on_submit: false,
            max_candidates: 256,
        },
    )
    .unwrap_or_else(|_| unreachable!("static fuzzy prompt configuration is valid"))
}

pub(crate) fn snapshot(
    source: &dyn crate::types::FuzzySource,
    generation: u64,
) -> Option<ratcore::prompt_completion::PromptCompletionSnapshot> {
    let candidates = source.candidates();
    if candidates.len() > 256 {
        return None;
    }
    let projected = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let source_order = u32::try_from(index).ok()?;
            let mut projected = ratcore::prompt_completion::PromptCandidate::enabled(
                candidate.id.clone(),
                candidate.text.clone(),
                source_order,
            );
            projected.detail = candidate.context.clone();
            Some(projected)
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ratcore::prompt_completion::PromptCompletionSnapshot {
        generation: ratcore::prompt_completion::PromptGeneration(generation),
        status: ratcore::prompt_completion::PromptCompletionStatus::Complete,
        candidates: projected,
    })
}
