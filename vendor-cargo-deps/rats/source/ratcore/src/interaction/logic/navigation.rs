//! Supplied-history navigation and draft restoration.

pub(crate) fn navigate(
    session: &mut super::state::PromptSession,
    direction: super::state::PromptHistoryDirection,
) -> Result<(), super::state::PromptDiagnostic> {
    if session.config.history != super::model::PromptHistoryPolicy::Enabled {
        return Err(unavailable());
    }
    let Some(history) = &session.history else {
        return Err(unavailable());
    };
    if history.entries.is_empty() {
        return Err(unavailable());
    }
    debug_assert!(history.namespace.is_valid());
    debug_assert!(history.entries.len() <= history.entries.capacity());
    let next = match direction {
        super::state::PromptHistoryDirection::Older => {
            if session.history_position.is_none() {
                session.history_draft = Some(session.input.clone());
            }
            Some(
                session
                    .history_position
                    .map_or(history.entries.len().saturating_sub(1), |index| index.saturating_sub(1)),
            )
        }
        super::state::PromptHistoryDirection::Newer => match session.history_position {
            Some(index) if index.saturating_add(1) < history.entries.len() => Some(index.saturating_add(1)),
            Some(_) => None,
            None => return Err(unavailable()),
        },
    };
    session.history_position = next;
    session.input = next
        .and_then(|index| history.entries.get(index).cloned())
        .or_else(|| session.history_draft.take())
        .unwrap_or_default();
    session.input_revision = super::model::PromptInputRevision(session.input_revision.0.saturating_add(1));
    session.validation = None;
    super::matching::repair_active(session);
    Ok(())
}

fn unavailable() -> super::state::PromptDiagnostic {
    super::state::PromptDiagnostic::new(
        super::state::PromptDiagnosticCode::HistoryUnavailable,
        "history navigation is unavailable",
    )
}
