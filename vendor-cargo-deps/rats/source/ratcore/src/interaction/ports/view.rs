//! Renderer-neutral prompt projection shared by maintained adapters.

/// One projected completion row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptCandidateProjection {
    /// Stable candidate identity.
    pub id: super::model::PromptCandidateId,
    /// Visible label.
    pub label: String,
    /// Optional visible detail.
    pub detail: Option<String>,
    /// Optional visible group.
    pub group: Option<String>,
    /// Whether submission is eligible.
    pub enabled: bool,
    /// Optional disabled reason.
    pub disabled_reason: Option<String>,
    /// Whether this identity is active.
    pub active: bool,
}

/// Pure projection consumed by terminal and DOM adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptProjection {
    /// Stable session identity.
    pub session_id: super::model::PromptSessionId,
    /// Visible prompt label.
    pub label: String,
    /// Optional visible help.
    pub help: Option<String>,
    /// Committed input text. Sensitive sessions must not enter replay evidence.
    pub input: String,
    /// Current input revision.
    pub input_revision: super::model::PromptInputRevision,
    /// Current lifecycle.
    pub lifecycle: super::state::PromptLifecycle,
    /// Current catalog generation.
    pub candidate_generation: Option<super::model::PromptGeneration>,
    /// Filtered deterministic candidate rows.
    pub candidates: Vec<PromptCandidateProjection>,
    /// Whether older history navigation is available.
    pub history_older_available: bool,
    /// Whether newer history or draft restoration is available.
    pub history_newer_available: bool,
    /// Current validation state.
    pub validation: Option<super::model::PromptValidationState>,
}

/// Projects accepted state without renderer objects or callbacks.
#[must_use]
pub fn project_prompt(session: &super::state::PromptSession) -> PromptProjection {
    debug_assert!(session.id.is_valid());
    debug_assert!(session.input.len() <= 4096);
    let candidates = super::matching::filtered_indices(session)
        .into_iter()
        .filter_map(|index| {
            session.completion.as_ref().and_then(|snapshot| snapshot.candidates.get(index)).map(|candidate| {
                PromptCandidateProjection {
                    id: candidate.id.clone(),
                    label: candidate.label.clone(),
                    detail: candidate.detail.clone(),
                    group: candidate.group.clone(),
                    enabled: candidate.enabled,
                    disabled_reason: candidate.disabled_reason.clone(),
                    active: session.active.as_ref() == Some(&candidate.id),
                }
            })
        })
        .collect();
    let history_len = session.history().map_or(0, |history| history.entries.len());
    PromptProjection {
        session_id: session.id.clone(),
        label: session.label.clone(),
        help: session.help.clone(),
        input: session.input.clone(),
        input_revision: session.input_revision,
        lifecycle: session.lifecycle,
        candidate_generation: session.completion.as_ref().map(|snapshot| snapshot.generation),
        candidates,
        history_older_available: history_len > 0,
        history_newer_available: session.history_position.is_some(),
        validation: session.validation.as_ref().map(|snapshot| snapshot.state.clone()),
    }
}

/// Produces privacy-bounded replay text.
#[must_use]
pub fn replay_input(session: &super::state::PromptSession) -> Option<String> {
    match session.config.history {
        super::model::PromptHistoryPolicy::Sensitive => Some("[redacted]".to_string()),
        super::model::PromptHistoryPolicy::Disabled => None,
        super::model::PromptHistoryPolicy::Enabled => Some(session.input.clone()),
    }
}
