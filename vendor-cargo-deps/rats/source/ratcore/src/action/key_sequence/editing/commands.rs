use super::super::ActionBindingId;
use super::super::catalog::ActionBinding;
use super::super::catalog::ActionBindingSourceKind;
use super::super::catalog::BindingCatalogDiagnostic;
use super::super::catalog::BindingCatalogDiagnosticCode;
use super::super::catalog::BindingCatalogSnapshot;
use super::BindingEditCommand;
use super::BindingEditOutcome;

pub(super) fn apply_candidate_edit(
    bindings: &mut Vec<ActionBinding>,
    command: &BindingEditCommand,
) -> Option<BindingCatalogDiagnostic> {
    match command {
        BindingEditCommand::Add { binding, .. } => bindings.push(binding.clone()),
        BindingEditCommand::Replace {
            binding_id,
            replacement,
            ..
        } => {
            let Some(existing) = bindings.iter_mut().find(|item| &item.id == binding_id) else {
                return Some(unknown_binding(binding_id));
            };
            existing.enabled = false;
            existing.disabled_reason = Some("replaced by user override".to_string());
            let mut accepted_replacement = replacement.clone();
            accepted_replacement.source.kind = ActionBindingSourceKind::UserOverride;
            accepted_replacement.source.replaces = Some(binding_id.clone());
            bindings.push(accepted_replacement);
        }
        BindingEditCommand::Clear { binding_id, .. } => {
            let Some(binding) = bindings.iter_mut().find(|item| &item.id == binding_id) else {
                return Some(unknown_binding(binding_id));
            };
            binding.enabled = false;
            binding.disabled_reason = Some("cleared by user".to_string());
        }
        BindingEditCommand::ClearAction {
            action_id, layer_id, ..
        } => {
            let mut matched = false;
            for binding in bindings.iter_mut().filter(|binding| {
                binding.action_id == *action_id && layer_id.as_ref().is_none_or(|layer| &binding.layer_id == layer)
            }) {
                binding.enabled = false;
                binding.disabled_reason = Some("cleared by user".to_string());
                matched = true;
            }
            if !matched {
                return Some(
                    BindingCatalogDiagnostic::new(
                        BindingCatalogDiagnosticCode::UnknownAction,
                        "edit.action_id",
                        "no accepted binding matches the requested action/layer",
                    )
                    .with_action(action_id.clone()),
                );
            }
        }
        BindingEditCommand::ResetAction { action_id, .. } => {
            bindings.retain(|binding| {
                binding.action_id != *action_id || binding.source.kind != ActionBindingSourceKind::UserOverride
            });
            for binding in bindings.iter_mut().filter(|binding| binding.action_id == *action_id) {
                binding.enabled = true;
                binding.disabled_reason = None;
            }
        }
        BindingEditCommand::UnsupportedCapture { .. } => {}
    }
    None
}

pub(super) fn rejected(snapshot: &BindingCatalogSnapshot, diagnostic: BindingCatalogDiagnostic) -> BindingEditOutcome {
    BindingEditOutcome {
        snapshot: snapshot.clone(),
        diagnostics: vec![diagnostic],
        accepted: false,
    }
}

fn unknown_binding(binding_id: &ActionBindingId) -> BindingCatalogDiagnostic {
    BindingCatalogDiagnostic::new(
        BindingCatalogDiagnosticCode::UnknownBinding,
        "edit.binding_id",
        "binding edit references an unknown binding",
    )
    .with_binding(binding_id.clone())
}

pub(super) fn format_sequence_issues(issues: &[super::super::KeySequenceDiagnostic]) -> String {
    issues.iter().map(|issue| issue.message.clone()).collect::<Vec<_>>().join("; ")
}
