const TABS_SURFACE_ID: &str = "surface.headless-tabs";
const TABS_LIST_ID: &str = "semantic.headless-tabs.list";

pub(super) fn snapshot(
    state: &super::super::model::TabsState,
    tabs: &[super::TabTriggerPart],
    panels: &[super::TabPanelPart],
) -> crate::attention_core::SemanticSnapshot {
    let node_slots = tabs.len().saturating_add(panels.len()).saturating_add(1);
    let mut nodes = Vec::with_capacity(node_slots);
    nodes.push(crate::attention_core::SemanticNode::new(
        TABS_LIST_ID,
        TABS_SURFACE_ID,
        crate::attention_core::SemanticRole::TabList,
        "Tabs",
    ));
    nodes.extend(tabs.iter().map(trigger));
    nodes.extend(panels.iter().filter_map(|panel| panel_node(panel, tabs)));
    let mut snapshot = crate::attention_core::SemanticSnapshot::new(
        crate::attention_core::SemanticGeneration::new(state.generation),
        vec![crate::attention_core::SemanticSurface::new(
            TABS_SURFACE_ID,
            TABS_LIST_ID,
        )],
        nodes,
    );
    snapshot.focus_links = tabs
        .iter()
        .filter(|tab| !tab.is_disabled() && !tab.is_hidden())
        .map(|tab| crate::attention_core::SemanticFocusLink {
            focus: crate::attention_core::FocusNodeId::new(focus_id(&tab.id)),
            semantic: trigger_id(&tab.id).into(),
        })
        .collect();
    debug_assert!(snapshot.nodes.len() <= node_slots);
    debug_assert!(snapshot.focus_links.len() <= tabs.len());
    snapshot
}

fn trigger(tab: &super::TabTriggerPart) -> crate::attention_core::SemanticNode {
    let mut node = crate::attention_core::SemanticNode::new(
        trigger_id(&tab.id),
        TABS_SURFACE_ID,
        crate::attention_core::SemanticRole::Tab,
        &tab.label,
    )
    .child_of(TABS_LIST_ID, tab.order)
    .with_relation(crate::attention_core::SemanticRelationKind::Controls, panel_id(&tab.panel_id));
    node.state.selected = Some(tab.is_selected());
    node = apply_availability(node, tab);
    if !tab.is_disabled() && !tab.is_hidden() {
        node = node
            .with_action(
                crate::attention_core::SemanticAction::Focus,
                crate::nexus_core::ActionId::new(focus_id(&tab.id)),
            )
            .with_action(
                crate::attention_core::SemanticAction::Select,
                crate::nexus_core::ActionId::new(select_id(&tab.id)),
            )
            .with_action(
                crate::attention_core::SemanticAction::Activate,
                crate::nexus_core::ActionId::new(activate_id(&tab.id)),
            );
    }
    debug_assert_eq!(node.state.selected, Some(tab.is_selected()));
    debug_assert_eq!(node.relations.len(), 1);
    node
}

fn apply_availability(
    mut node: crate::attention_core::SemanticNode,
    tab: &super::TabTriggerPart,
) -> crate::attention_core::SemanticNode {
    if tab.is_disabled() {
        node.state = node.state.with_flag(crate::attention_core::SemanticStateKind::Disabled);
    }
    if tab.is_hidden() {
        node.state = node.state.with_flag(crate::attention_core::SemanticStateKind::Hidden);
    } else {
        node.collection = Some(crate::attention_core::SemanticCollection {
            level: 1,
            position: tab.position.unwrap_or(1),
            set_size: tab.set_size,
        });
    }
    node
}

fn panel_node(
    panel: &super::TabPanelPart,
    tabs: &[super::TabTriggerPart],
) -> Option<crate::attention_core::SemanticNode> {
    let tab = tabs.iter().find(|tab| tab.id == panel.tab_id)?;
    let trigger_count = tabs.iter().fold(0_u64, |count, _| count.saturating_add(1));
    let mut node = crate::attention_core::SemanticNode::new(
        panel_id(&panel.id),
        TABS_SURFACE_ID,
        crate::attention_core::SemanticRole::TabPanel,
        &tab.label,
    )
    .child_of(TABS_LIST_ID, trigger_count.saturating_add(tab.order));
    if panel.is_hidden {
        node.state = node.state.with_flag(crate::attention_core::SemanticStateKind::Hidden);
    }
    Some(node)
}

fn trigger_id(id: &super::super::model::TabId) -> String {
    format!("semantic.headless-tabs.trigger.{}", id.as_str())
}

fn panel_id(id: &super::super::model::TabPanelId) -> String {
    format!("semantic.headless-tabs.panel.{}", id.as_str())
}

fn focus_id(id: &super::super::model::TabId) -> String {
    format!("headless-tabs.focus.{}", id.as_str())
}

fn select_id(id: &super::super::model::TabId) -> String {
    format!("headless-tabs.select.{}", id.as_str())
}

fn activate_id(id: &super::super::model::TabId) -> String {
    format!("headless-tabs.activate.{}", id.as_str())
}
