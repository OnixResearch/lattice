//! Workspace shell pure reducers.

/// Reduces a command against a workspace shell model.
#[must_use]
pub fn reduce_command(
    model: &super::state::Model,
    command: &super::commands::Command,
) -> super::commands::CommandOutput {
    match reduce_inner(model, command) {
        Ok((accepted_model, effects)) => super::commands::CommandOutput {
            model: accepted_model,
            effects,
            diagnostics: Vec::new(),
        },
        Err(diagnostic) => super::commands::CommandOutput {
            model: model.clone(),
            effects: Vec::new(),
            diagnostics: vec![diagnostic],
        },
    }
}

fn reduce_inner(model: &super::state::Model, command: &super::commands::Command) -> super::support::CoreResult {
    match command {
        super::commands::Command::CreateTab {
            workspace,
            tab,
            focus_policy,
        } => create_tab(model, super::support::CreateTabRequest {
            space: workspace.clone(),
            tab: tab.clone(),
            focus_policy: *focus_policy,
        }),
        super::commands::Command::SplitPane {
            workspace,
            tab,
            target_pane,
            pane,
            axis,
            focus_policy,
        } => split_pane(model, super::support::SplitRequest {
            space: workspace.clone(),
            tab: tab.clone(),
            target_pane: target_pane.clone(),
            pane: pane.clone(),
            axis: *axis,
            focus_policy: *focus_policy,
        }),
        super::commands::Command::FocusPane { workspace, tab, pane } => focus_pane(model, super::support::PaneTarget {
            space: workspace.clone(),
            tab: tab.clone(),
            pane: pane.clone(),
        }),
        super::commands::Command::ClosePane { workspace, tab, pane } => close_pane(model, super::support::PaneTarget {
            space: workspace.clone(),
            tab: tab.clone(),
            pane: pane.clone(),
        }),
        super::commands::Command::RestoreLayout { model: candidate } => restore_layout(candidate.clone()),
        super::commands::Command::UpdateTaskStatus { pane, status } => update_task_status(model, pane, *status),
        super::commands::Command::DisabledAction { action_id, reason } => {
            Err(super::support::disabled_diagnostic(super::support::DisabledRef { action_id, reason }))
        }
    }
}

fn create_tab(model: &super::state::Model, request: super::support::CreateTabRequest) -> super::support::CoreResult {
    debug_assert!(!request.space.is_empty());
    debug_assert!(!request.tab.live_id.is_empty());
    super::validation::ensure_unique_ids(model, super::validation::CandidateIds {
        live: request.tab.live_id.as_str(),
        persisted: request.tab.persisted_id.as_str(),
    })?;
    let mut next = model.clone();
    let space_index = super::validation::space_index(&next, &request.space).ok_or_else(|| {
        super::validation::missing_live_diagnostic(&next, super::validation::LiveRef {
            id: request.space.as_str(),
            subject: super::types::SUBJECT_SPACE,
        })
    })?;
    let inserted_tab_id = request.tab.live_id.clone();
    let new_focus = request
        .tab
        .active_pane
        .as_ref()
        .map(|pane| super::state::Focus::new(request.space.clone(), inserted_tab_id.clone(), pane.clone()));
    next.workspaces[space_index].tabs.push(request.tab);
    if request.focus_policy == super::commands::FocusPolicy::Transfer {
        next.workspaces[space_index].active_tab = Some(inserted_tab_id.clone());
        next.focus = new_focus;
    }
    Ok((next, vec![super::commands::EffectRequest::CreateTab { tab: inserted_tab_id }]))
}

fn split_pane(model: &super::state::Model, request: super::support::SplitRequest) -> super::support::CoreResult {
    debug_assert!(!request.space.is_empty());
    debug_assert!(!request.target_pane.is_empty());
    super::validation::ensure_unique_ids(model, super::validation::CandidateIds {
        live: request.pane.live_id.as_str(),
        persisted: request.pane.persisted_id.as_str(),
    })?;
    let mut next = model.clone();
    let position = super::validation::require_tab(&next, super::validation::TabRef {
        space: &request.space,
        tab: &request.tab,
    })?;
    super::support::ensure_pane_present(&next, position, &request.target_pane)?;
    let inserted_pane_id = request.pane.live_id.clone();
    next.workspaces[position.workspace].tabs[position.tab].panes.push(request.pane);
    if request.focus_policy == super::commands::FocusPolicy::Transfer {
        next.workspaces[position.workspace].active_tab = Some(request.tab.clone());
        next.workspaces[position.workspace].tabs[position.tab].active_pane = Some(inserted_pane_id.clone());
        next.focus = Some(super::state::Focus::new(request.space, request.tab, inserted_pane_id.clone()));
    }
    Ok((next, vec![super::commands::EffectRequest::SplitPane {
        pane: inserted_pane_id,
        axis: request.axis,
    }]))
}

fn focus_pane(model: &super::state::Model, target: super::support::PaneTarget) -> super::support::CoreResult {
    debug_assert!(!target.space.is_empty());
    debug_assert!(!target.pane.is_empty());
    let mut next = model.clone();
    let position = super::validation::require_tab(&next, super::validation::TabRef {
        space: &target.space,
        tab: &target.tab,
    })?;
    super::support::ensure_pane_present(&next, position, &target.pane)?;
    next.workspaces[position.workspace].active_tab = Some(target.tab.clone());
    next.workspaces[position.workspace].tabs[position.tab].active_pane = Some(target.pane.clone());
    next.focus = Some(super::state::Focus::new(target.space, target.tab, target.pane.clone()));
    Ok((next, vec![super::commands::EffectRequest::FocusPane { pane: target.pane }]))
}

fn close_pane(model: &super::state::Model, target: super::support::PaneTarget) -> super::support::CoreResult {
    debug_assert!(!target.space.is_empty());
    debug_assert!(!target.pane.is_empty());
    let mut next = model.clone();
    let position = super::validation::require_tab(&next, super::validation::TabRef {
        space: &target.space,
        tab: &target.tab,
    })?;
    let pane_position =
        super::validation::pane_index(&next.workspaces[position.workspace].tabs[position.tab], &target.pane)
            .ok_or_else(|| super::support::pane_missing(&next, &target.pane))?;
    next.workspaces[position.workspace].tabs[position.tab].panes.remove(pane_position);
    next.retired_live_ids.insert(super::types::RetiredLiveId::Pane(target.pane.clone()));
    super::state::repair_focus_after_close(&mut next, position, &target.pane);
    Ok((next, vec![super::commands::EffectRequest::ClosePane { pane: target.pane }]))
}

fn restore_layout(candidate: super::state::Model) -> super::support::CoreResult {
    let diagnostics = super::validation::validate_model(&candidate);
    if let Some(diagnostic) = diagnostics.into_iter().next() {
        return Err(super::types::Diagnostic::new(
            super::types::DiagnosticCode::UnsupportedRestoreShape,
            super::types::SUBJECT_LAYOUT,
            diagnostic.message,
        ));
    }
    Ok((candidate, vec![super::commands::EffectRequest::RestoreLayout]))
}

fn update_task_status(
    model: &super::state::Model,
    pane_id: &super::types::PaneId,
    status: super::types::TaskStatus,
) -> super::support::CoreResult {
    debug_assert!(!pane_id.is_empty());
    debug_assert!(model.workspaces.len() <= model.workspaces.capacity());
    let mut next = model.clone();
    let Some((space_position, tab_position, pane_position)) = super::validation::pane_position_in_model(&next, pane_id)
    else {
        return Err(super::support::pane_missing(&next, pane_id));
    };
    next.workspaces[space_position].tabs[tab_position].panes[pane_position].status = status;
    Ok((next, vec![super::commands::EffectRequest::UpdateTaskStatus {
        pane: pane_id.clone(),
        status,
    }]))
}
