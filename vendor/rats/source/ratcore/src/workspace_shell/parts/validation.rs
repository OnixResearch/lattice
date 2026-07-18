//! Workspace shell validation and lookup helpers.

/// Candidate live and persisted IDs to insert.
pub(crate) struct CandidateIds<'a> {
    pub live: &'a str,
    pub persisted: &'a str,
}

/// Tab lookup request scoped by workspace and tab IDs.
pub(crate) struct TabRef<'a> {
    pub space: &'a str,
    pub tab: &'a str,
}

/// Live-ID diagnostic request.
pub(crate) struct LiveRef<'a> {
    pub id: &'a str,
    pub subject: &'a str,
}

/// Validates live IDs, durable IDs, and focus targets.
#[must_use]
pub fn validate_model(model: &super::state::Model) -> Vec<super::types::Diagnostic> {
    let focus_diagnostics = focus_diagnostic(model).into_iter();
    duplicate_id_diagnostics(model)
        .into_iter()
        .chain(focus_diagnostics)
        .collect()
}

pub(crate) fn ensure_unique_ids(
    model: &super::state::Model,
    candidate: CandidateIds<'_>,
) -> Result<(), super::types::Diagnostic> {
    debug_assert!(!candidate.live.is_empty());
    debug_assert!(!candidate.persisted.is_empty());
    if all_live_ids(model)
        .iter()
        .any(|existing| existing == candidate.live)
    {
        return Err(super::types::Diagnostic::new(
            super::types::DiagnosticCode::DuplicateLiveId,
            candidate.live,
            format!("live ID `{}` already exists", candidate.live),
        ));
    }
    if all_persisted_ids(model)
        .iter()
        .any(|existing| existing == candidate.persisted)
    {
        return Err(super::types::Diagnostic::new(
            super::types::DiagnosticCode::DuplicatePersistedId,
            candidate.persisted,
            format!("persisted ID `{}` already exists", candidate.persisted),
        ));
    }
    Ok(())
}

pub(crate) fn require_tab(
    model: &super::state::Model,
    target: TabRef<'_>,
) -> Result<super::state::TabPosition, super::types::Diagnostic> {
    debug_assert!(!target.space.is_empty());
    debug_assert!(!target.tab.is_empty());
    let space = space_index(model, target.space).ok_or_else(|| {
        missing_live_diagnostic(
            model,
            LiveRef {
                id: target.space,
                subject: super::types::SUBJECT_SPACE,
            },
        )
    })?;
    let tab = tab_index(&model.workspaces[space], target.tab).ok_or_else(|| {
        missing_live_diagnostic(
            model,
            LiveRef {
                id: target.tab,
                subject: super::types::SUBJECT_TAB,
            },
        )
    })?;
    Ok(super::state::TabPosition {
        workspace: space,
        tab,
    })
}

pub(crate) fn space_index(model: &super::state::Model, live_id: &str) -> Option<usize> {
    model
        .workspaces
        .iter()
        .position(|workspace| workspace.live_id == live_id)
}

pub(crate) fn tab_index(workspace: &super::state::Space, live_id: &str) -> Option<usize> {
    workspace.tabs.iter().position(|tab| tab.live_id == live_id)
}

pub(crate) fn pane_index(tab: &super::state::Tab, live_id: &str) -> Option<usize> {
    tab.panes.iter().position(|pane| pane.live_id == live_id)
}

pub(crate) fn pane_position_in_model(
    model: &super::state::Model,
    live_id: &str,
) -> Option<(usize, usize, usize)> {
    for (space_position, workspace) in model.workspaces.iter().enumerate() {
        for (tab_position, tab) in workspace.tabs.iter().enumerate() {
            if let Some(pane_position) = pane_index(tab, live_id) {
                return Some((space_position, tab_position, pane_position));
            }
        }
    }
    None
}

pub(crate) fn missing_live_diagnostic(
    model: &super::state::Model,
    live: LiveRef<'_>,
) -> super::types::Diagnostic {
    if model.retired_live_ids.contains(live.id) {
        return super::types::Diagnostic::new(
            super::types::DiagnosticCode::StaleLiveId,
            live.id,
            format!("{} live ID `{}` is stale", live.subject, live.id),
        );
    }
    super::types::Diagnostic::new(
        super::types::DiagnosticCode::UnknownLiveId,
        live.id,
        format!("{} live ID `{}` is unknown", live.subject, live.id),
    )
}

fn duplicate_id_diagnostics(model: &super::state::Model) -> Vec<super::types::Diagnostic> {
    duplicate_live_id_diagnostics(all_live_ids(model))
        .into_iter()
        .chain(duplicate_persisted_id_diagnostics(all_persisted_ids(model)))
        .collect()
}

fn duplicate_live_id_diagnostics(ids: Vec<String>) -> Vec<super::types::Diagnostic> {
    let mut seen = std::collections::BTreeSet::new();
    let mut diagnostics = Vec::with_capacity(ids.len());
    for id in ids {
        if !seen.insert(id.clone()) {
            diagnostics.push(super::types::Diagnostic::new(
                super::types::DiagnosticCode::DuplicateLiveId,
                id.clone(),
                format!("duplicate live ID `{id}`"),
            ));
        }
    }
    diagnostics
}

fn duplicate_persisted_id_diagnostics(ids: Vec<String>) -> Vec<super::types::Diagnostic> {
    let mut seen = std::collections::BTreeSet::new();
    let mut diagnostics = Vec::with_capacity(ids.len());
    for id in ids {
        if !seen.insert(id.clone()) {
            diagnostics.push(super::types::Diagnostic::new(
                super::types::DiagnosticCode::DuplicatePersistedId,
                id.clone(),
                format!("duplicate persisted ID `{id}`"),
            ));
        }
    }
    diagnostics
}

fn all_live_ids(model: &super::state::Model) -> Vec<String> {
    let mut ids = Vec::with_capacity(id_capacity(model));
    for workspace in &model.workspaces {
        ids.push(workspace.live_id.clone());
        for tab in &workspace.tabs {
            ids.push(tab.live_id.clone());
            ids.extend(tab.panes.iter().map(|pane| pane.live_id.clone()));
        }
    }
    ids
}

fn all_persisted_ids(model: &super::state::Model) -> Vec<String> {
    let mut ids = Vec::with_capacity(id_capacity(model));
    for workspace in &model.workspaces {
        ids.push(workspace.persisted_id.clone());
        for tab in &workspace.tabs {
            ids.push(tab.persisted_id.clone());
            ids.extend(tab.panes.iter().map(|pane| pane.persisted_id.clone()));
        }
    }
    ids
}

fn id_capacity(model: &super::state::Model) -> usize {
    model.workspaces.iter().fold(0, |total, workspace| {
        total
            .saturating_add(1)
            .saturating_add(workspace.tabs.len())
            .saturating_add(
                workspace
                    .tabs
                    .iter()
                    .map(|tab| tab.panes.len())
                    .sum::<usize>(),
            )
    })
}

fn focus_diagnostic(model: &super::state::Model) -> Option<super::types::Diagnostic> {
    let focus = model.focus.as_ref()?;
    let Some(space_position) = space_index(model, &focus.workspace) else {
        return Some(missing_live_diagnostic(
            model,
            LiveRef {
                id: &focus.workspace,
                subject: super::types::SUBJECT_SPACE,
            },
        ));
    };
    tab_focus_diagnostic(model, space_position, focus)
}

fn tab_focus_diagnostic(
    model: &super::state::Model,
    space_position: usize,
    focus: &super::state::Focus,
) -> Option<super::types::Diagnostic> {
    debug_assert!(space_position < model.workspaces.len());
    debug_assert!(!focus.tab.is_empty());
    let workspace = &model.workspaces[space_position];
    let Some(tab_position) = tab_index(workspace, &focus.tab) else {
        return Some(missing_live_diagnostic(
            model,
            LiveRef {
                id: &focus.tab,
                subject: super::types::SUBJECT_TAB,
            },
        ));
    };
    pane_focus_diagnostic(
        model,
        super::state::TabPosition {
            workspace: space_position,
            tab: tab_position,
        },
        focus,
    )
}

fn pane_focus_diagnostic(
    model: &super::state::Model,
    position: super::state::TabPosition,
    focus: &super::state::Focus,
) -> Option<super::types::Diagnostic> {
    let tab = &model.workspaces[position.workspace].tabs[position.tab];
    if pane_index(tab, &focus.pane).is_some() {
        return None;
    }
    Some(missing_live_diagnostic(
        model,
        LiveRef {
            id: &focus.pane,
            subject: super::types::SUBJECT_PANE,
        },
    ))
}
