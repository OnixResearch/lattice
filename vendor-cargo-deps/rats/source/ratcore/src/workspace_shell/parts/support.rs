//! Private reducer request and diagnostic helpers.

pub(super) type CoreResult =
    Result<(super::state::Model, Vec<super::commands::EffectRequest>), super::types::Diagnostic>;

pub(super) struct CreateTabRequest {
    pub space: super::types::LiveId,
    pub tab: super::state::Tab,
    pub focus_policy: super::commands::FocusPolicy,
}

pub(super) struct SplitRequest {
    pub space: super::types::LiveId,
    pub tab: super::types::LiveId,
    pub target_pane: super::types::LiveId,
    pub pane: super::state::Pane,
    pub axis: super::commands::SplitAxis,
    pub focus_policy: super::commands::FocusPolicy,
}

pub(super) struct PaneTarget {
    pub space: super::types::LiveId,
    pub tab: super::types::LiveId,
    pub pane: super::types::LiveId,
}

pub(super) fn ensure_pane_present(
    model: &super::state::Model,
    position: super::state::TabPosition,
    pane_id: &str,
) -> Result<(), super::types::Diagnostic> {
    if super::validation::pane_index(
        &model.workspaces[position.workspace].tabs[position.tab],
        pane_id,
    )
    .is_some()
    {
        return Ok(());
    }
    Err(pane_missing(model, pane_id))
}

pub(super) fn pane_missing(model: &super::state::Model, pane_id: &str) -> super::types::Diagnostic {
    super::validation::missing_live_diagnostic(
        model,
        super::validation::LiveRef {
            id: pane_id,
            subject: super::types::SUBJECT_PANE,
        },
    )
}

pub(super) struct DisabledRef<'a> {
    pub action_id: &'a str,
    pub reason: &'a str,
}

pub(super) fn disabled_diagnostic(disabled: DisabledRef<'_>) -> super::types::Diagnostic {
    super::types::Diagnostic::new(
        super::types::DiagnosticCode::DisabledAction,
        disabled.action_id,
        format!(
            "workspace action `{}` is disabled: {}",
            disabled.action_id, disabled.reason
        ),
    )
}
