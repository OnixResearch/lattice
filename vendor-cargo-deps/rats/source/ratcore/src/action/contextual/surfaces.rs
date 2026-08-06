/// Contextual route projection shared by every command surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRouteView {
    /// Stable action identity.
    pub action_id: crate::nexus_core::ActionId,
    /// Whether a generation-bound route is currently accepted.
    pub enabled: bool,
    /// Accepted pure target route.
    pub route: Option<super::model::ActionRoute>,
    /// Disabled, blocked, ambiguous, unsupported, or unhandled diagnostic.
    pub diagnostic: Option<super::model::RoutingDiagnostic>,
}

impl ActionRouteView {
    /// Returns the contextual disabled reason for display adapters.
    #[must_use]
    pub fn disabled_reason(&self) -> Option<&str> {
        self.diagnostic.as_ref().map(|diagnostic| diagnostic.message.as_str())
    }
}

/// Command surfaces whose enablement comes from production route probes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandViews {
    /// Routing generation represented by all projections.
    pub routing_generation: u64,
    /// Inspectable route projection for every action.
    pub routes: Vec<ActionRouteView>,
    /// Active keymap bindings.
    pub keymap: Vec<crate::nexus_core::KeymapBindingProjection>,
    /// Menu items, including disabled route diagnostics.
    pub menu: Vec<crate::nexus_core::MenuItemProjection>,
    /// Command palette items, including disabled route diagnostics.
    pub palette: Vec<crate::nexus_core::CommandPaletteItemProjection>,
    /// Toolbar items, including disabled route diagnostics.
    pub toolbar: Vec<crate::nexus_core::ToolbarItemProjection>,
    /// Shortcut help grouped by action category.
    pub shortcut_help: Vec<crate::nexus_core::ShortcutHelpSectionProjection>,
}

/// Projects action metadata through the production contextual resolver.
#[must_use]
pub fn build_command_views(snapshot: &super::model::ActionRoutingSnapshot) -> CommandViews {
    debug_assert!(snapshot.generation > 0);
    debug_assert!(snapshot.active_chain.len() <= snapshot.contexts.len());
    let routes = snapshot
        .registry
        .actions()
        .iter()
        .map(|action| {
            route_from_outcome(action.id.clone(), super::resolution::probe_action(snapshot, &action.id, None).outcome)
        })
        .collect::<Vec<_>>();
    let by_action = routes
        .iter()
        .map(|projection| (projection.action_id.clone(), projection))
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut keymap = snapshot.registry.keymap_bindings();
    keymap.retain(|item| by_action.get(&item.action_id).is_some_and(|projection| projection.enabled));

    let mut menu = snapshot.registry.menu_items();
    for item in &mut menu {
        apply_route_state(&by_action, &item.action_id, &mut item.enabled, &mut item.disabled_reason);
    }
    let mut palette = snapshot.registry.command_palette_items();
    for item in &mut palette {
        apply_route_state(&by_action, &item.action_id, &mut item.enabled, &mut item.disabled_reason);
    }
    let mut toolbar = snapshot.registry.toolbar_items();
    for item in &mut toolbar {
        apply_route_state(&by_action, &item.action_id, &mut item.enabled, &mut item.disabled_reason);
    }
    let mut shortcut_help = snapshot.registry.shortcut_help_sections();
    apply_shortcut_help_states(&by_action, &mut shortcut_help);

    CommandViews {
        routing_generation: snapshot.generation,
        routes,
        keymap,
        menu,
        palette,
        toolbar,
        shortcut_help,
    }
}

fn apply_shortcut_help_states(
    by_action: &std::collections::BTreeMap<crate::nexus_core::ActionId, &ActionRouteView>,
    sections: &mut [crate::nexus_core::ShortcutHelpSectionProjection],
) {
    for section in sections {
        for item in &mut section.entries {
            apply_route_state(by_action, &item.action_id, &mut item.enabled, &mut item.disabled_reason);
        }
    }
}

fn route_from_outcome(
    action_id: crate::nexus_core::ActionId,
    outcome: super::model::ActionRouteOutcome,
) -> ActionRouteView {
    match outcome {
        super::model::ActionRouteOutcome::Accepted(route) => ActionRouteView {
            action_id,
            enabled: true,
            route: Some(route),
            diagnostic: None,
        },
        super::model::ActionRouteOutcome::Rejected(diagnostic) => ActionRouteView {
            action_id,
            enabled: false,
            route: None,
            diagnostic: Some(diagnostic),
        },
    }
}

fn apply_route_state(
    by_action: &std::collections::BTreeMap<crate::nexus_core::ActionId, &ActionRouteView>,
    action_id: &crate::nexus_core::ActionId,
    enabled: &mut bool,
    disabled_reason: &mut Option<String>,
) {
    let Some(route) = by_action.get(action_id) else {
        *enabled = false;
        *disabled_reason = Some("contextual route projection is missing".to_string());
        return;
    };
    *enabled = route.enabled;
    *disabled_reason = route.diagnostic.as_ref().map(|diagnostic| diagnostic.message.clone());
}
