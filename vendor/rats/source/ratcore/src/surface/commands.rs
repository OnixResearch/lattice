use super::*;

/// Projected command data for IDE command surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeCommandSurfaces {
    /// Keymap bindings.
    pub keymap: Vec<crate::action_registry::KeymapBindingProjection>,
    /// Leader-key entries.
    pub leader: Vec<crate::action_registry::LeaderEntryProjection>,
    /// Command palette entries.
    pub palette: Vec<crate::action_registry::CommandPaletteItemProjection>,
    /// Menu entries.
    pub menu: Vec<crate::action_registry::MenuItemProjection>,
    /// Shortcut help labels by action ID.
    pub help: std::collections::BTreeMap<String, String>,
}

pub(super) fn registry_issues(
    registry: &crate::action_registry::ActionRegistryBuild,
) -> Vec<IdeIssue> {
    registry
        .issues
        .iter()
        .map(|issue| {
            IdeIssue::new(
                IdeIssueCode::ActionRegistryIssue,
                issue.field.clone().unwrap_or_else(|| "action".to_string()),
                issue.message.clone(),
            )
        })
        .collect()
}

pub(super) fn command_surfaces(
    registry: &crate::action_registry::ActionRegistryBuild,
) -> IdeCommandSurfaces {
    let keymap = registry.snapshot.keymap_bindings();
    let help = keymap
        .iter()
        .map(|binding| {
            let shortcut = shortcut_label(&binding.modifiers, &binding.key);
            (binding.action_id.clone(), shortcut)
        })
        .collect();
    IdeCommandSurfaces {
        keymap,
        leader: registry.snapshot.leader_entries(),
        palette: registry.snapshot.command_palette_items(),
        menu: registry.snapshot.menu_items(),
        help,
    }
}

fn shortcut_label(modifiers: &[String], key: &str) -> String {
    let mut parts = modifiers.to_vec();
    parts.push(key.to_string());
    parts.join("+")
}
