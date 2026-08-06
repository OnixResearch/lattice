//! Fail-closed prompt construction and supplied-snapshot validation.

const MAX_CANDIDATES: usize = 256;
const MAX_HISTORY_ENTRIES: usize = 64;
const MAX_PROMPT_TEXT_BYTES: usize = 4096;

/// Constructs one validated open prompt session.
pub fn build_prompt_session(
    id: super::model::PromptSessionId,
    label: impl Into<String>,
    help: Option<String>,
    configuration: super::model::PromptConfiguration,
) -> Result<super::state::PromptSession, Vec<super::state::PromptDiagnostic>> {
    debug_assert!(MAX_CANDIDATES <= usize::from(u16::MAX));
    debug_assert!(MAX_HISTORY_ENTRIES < MAX_CANDIDATES);
    let label = label.into();
    let mut diagnostics = Vec::with_capacity(2);
    let is_id_invalid = !id.is_valid();
    let is_label_invalid = label.trim().is_empty() || label.len() > MAX_PROMPT_TEXT_BYTES;
    let is_help_invalid = help.as_ref().is_some_and(|value| value.len() > MAX_PROMPT_TEXT_BYTES);
    if is_id_invalid || is_label_invalid || is_help_invalid {
        diagnostics.push(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::MalformedSession,
            "prompt session metadata is malformed",
        ));
    }
    let is_candidate_count_invalid =
        configuration.max_candidates == 0 || usize::from(configuration.max_candidates) > MAX_CANDIDATES;
    let is_policy_invalid = configuration.completion == super::model::PromptCompletionPolicy::None
        && configuration.acceptance == super::model::PromptAcceptancePolicy::Candidate;
    if is_candidate_count_invalid || is_policy_invalid {
        diagnostics.push(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::InvalidConfiguration,
            "prompt configuration is contradictory or excessive",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(super::state::PromptSession {
        id,
        label,
        help,
        config: configuration,
        lifecycle: super::state::PromptLifecycle::Open,
        input: String::new(),
        input_revision: super::model::PromptInputRevision(0),
        completion: None,
        active: None,
        history: None,
        history_position: None,
        history_draft: None,
        validation: None,
    })
}

pub(crate) fn validate_completion(
    session: &super::state::PromptSession,
    snapshot: &super::model::PromptCompletionSnapshot,
) -> Option<super::state::PromptDiagnostic> {
    debug_assert!(usize::from(session.config.max_candidates) <= MAX_CANDIDATES);
    debug_assert!(snapshot.candidates.len() <= snapshot.candidates.capacity());
    let is_policy_disabled = session.config.completion == super::model::PromptCompletionPolicy::None;
    let is_session_candidate_count_exceeded = snapshot.candidates.len() > usize::from(session.config.max_candidates);
    let is_global_candidate_count_exceeded = snapshot.candidates.len() > MAX_CANDIDATES;
    if is_policy_disabled || is_session_candidate_count_exceeded || is_global_candidate_count_exceeded {
        return Some(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::InvalidCandidates,
            "completion snapshot exceeds accepted policy",
        ));
    }
    if session.completion.as_ref().is_some_and(|current| snapshot.generation <= current.generation) {
        return Some(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::StaleCandidates,
            "completion snapshot generation is stale",
        ));
    }
    let mut identities = std::collections::BTreeSet::new();
    let mut orders = std::collections::BTreeSet::new();
    for candidate in &snapshot.candidates {
        let is_enablement_valid = candidate.enabled == candidate.disabled_reason.is_none();
        let is_identity_invalid = !candidate.id.is_valid();
        let is_metadata_invalid = !candidate.metadata_is_bounded();
        let is_identity_duplicate = !identities.insert(candidate.id.clone());
        let is_order_duplicate = !orders.insert(candidate.source_order);
        let is_candidate_invalid = is_identity_invalid || is_metadata_invalid || !is_enablement_valid;
        if is_candidate_invalid || is_identity_duplicate || is_order_duplicate {
            return Some(super::state::PromptDiagnostic::new(
                super::state::PromptDiagnosticCode::InvalidCandidates,
                "completion snapshot contains malformed or ambiguous candidates",
            ));
        }
    }
    None
}

pub(crate) fn validate_history(
    session: &super::state::PromptSession,
    snapshot: &super::model::PromptHistorySnapshot,
) -> Option<super::state::PromptDiagnostic> {
    debug_assert!(MAX_HISTORY_ENTRIES > 0);
    debug_assert!(snapshot.entries.len() <= snapshot.entries.capacity());
    let is_policy_disabled = session.config.history != super::model::PromptHistoryPolicy::Enabled;
    let is_namespace_invalid = !snapshot.namespace.is_valid();
    let is_history_entry_count_exceeded = snapshot.entries.len() > MAX_HISTORY_ENTRIES;
    let is_entry_invalid = snapshot.entries.iter().any(|entry| !super::model::text_is_bounded(entry));
    let is_generation_stale = session.history.as_ref().is_some_and(|current| snapshot.generation <= current.generation);
    let is_snapshot_invalid = is_policy_disabled || is_namespace_invalid || is_history_entry_count_exceeded;
    if is_snapshot_invalid || is_entry_invalid || is_generation_stale {
        return Some(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::HistoryUnavailable,
            "history is disabled, sensitive, stale, or malformed",
        ));
    }
    None
}

pub(crate) fn validate_input(
    session: &super::state::PromptSession,
    input: &str,
    expected: super::model::PromptInputRevision,
) -> Option<super::state::PromptDiagnostic> {
    if session.lifecycle != super::state::PromptLifecycle::Open
        || expected != session.input_revision
        || !super::model::text_is_bounded(input)
    {
        return Some(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::InvalidInput,
            "input replacement is stale, excessive, or unavailable",
        ));
    }
    None
}

pub(crate) fn validate_observation(
    session: &super::state::PromptSession,
    snapshot: &super::model::PromptValidationSnapshot,
) -> Option<super::state::PromptDiagnostic> {
    let is_reason_bounded = match &snapshot.state {
        super::model::PromptValidationState::Rejected(reason) => {
            !reason.trim().is_empty() && super::model::text_is_bounded(reason)
        }
        _ => true,
    };
    if snapshot.input_revision != session.input_revision || !is_reason_bounded {
        return Some(super::state::PromptDiagnostic::new(
            super::state::PromptDiagnosticCode::InvalidValidation,
            "validation observation is stale or malformed",
        ));
    }
    None
}
