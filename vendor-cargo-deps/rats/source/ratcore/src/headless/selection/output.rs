mod semantics;

/// Eligible semantic action exposed by a named tab part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TabPartAction {
    /// Move roving focus to the trigger.
    Focus,
    /// Propose caller-controlled selection.
    Select,
    /// Activate the currently focused trigger.
    Activate,
}

/// Named tab-list projection part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabListPart {
    /// Stable list-part identity.
    pub id: String,
    /// Accepted orientation.
    pub orientation: super::model::TabsOrientation,
    /// Accepted activation policy.
    pub activation: super::model::TabsActivationPolicy,
}

/// One independently projected trigger state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TabTriggerState {
    /// Caller state selects the trigger.
    Selected,
    /// Roving focus targets the trigger.
    Focused,
    /// Activation and focus are unavailable.
    Disabled,
    /// Trigger is absent from visual traversal.
    Hidden,
}

/// Named tab-trigger projection part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabTriggerPart {
    /// Stable trigger identity.
    pub id: super::model::TabId,
    /// Associated stable panel identity.
    pub panel_id: super::model::TabPanelId,
    /// Bounded semantic label.
    pub label: String,
    /// Independently queryable selected, focused, disabled, and hidden states.
    pub states: std::collections::BTreeSet<TabTriggerState>,
    /// Zero-based stable catalog order, including hidden triggers.
    pub order: u64,
    /// One-based position among non-hidden triggers.
    pub position: Option<u64>,
    /// Number of non-hidden triggers.
    pub set_size: u64,
    /// Eligible semantic actions.
    pub actions: Vec<TabPartAction>,
}

impl TabTriggerPart {
    /// Returns whether caller state selects this trigger.
    #[must_use]
    pub fn is_selected(&self) -> bool {
        self.states.contains(&TabTriggerState::Selected)
    }

    /// Returns whether roving focus targets this trigger.
    #[must_use]
    pub fn is_focused(&self) -> bool {
        self.states.contains(&TabTriggerState::Focused)
    }

    /// Returns whether activation and focus are unavailable.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.states.contains(&TabTriggerState::Disabled)
    }

    /// Returns whether the trigger is absent from visual traversal.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.states.contains(&TabTriggerState::Hidden)
    }
}

/// Named tab-panel association part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabPanelPart {
    /// Stable panel identity.
    pub id: super::model::TabPanelId,
    /// Associated trigger identity.
    pub tab_id: super::model::TabId,
    /// Caller-owned content slot identity.
    pub slot_id: super::model::TabSlotId,
    /// Whether caller state selects this panel.
    pub is_active: bool,
    /// Whether renderer should hide this panel wrapper.
    pub is_hidden: bool,
}

/// Complete immutable tabs projection shared by renderer adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsProjection {
    /// Accepted state generation.
    pub generation: u64,
    /// Named tab-list part.
    pub list: TabListPart,
    /// Ordered named trigger parts.
    pub tabs: Vec<TabTriggerPart>,
    /// Ordered panel association parts.
    pub panels: Vec<TabPanelPart>,
    /// Shared semantic-tree projection.
    pub semantics: crate::attention_core::SemanticSnapshot,
    /// Semantic validation diagnostics; accepted projections normally have none.
    pub semantic_issues: Vec<crate::attention_core::SemanticIssue>,
}

/// Projects accepted tabs state into named parts and one semantic tree.
#[must_use]
pub fn project_tabs(state: &super::model::TabsState) -> TabsProjection {
    let visible_count = state
        .catalog
        .tabs
        .iter()
        .filter(|tab| !tab.is_hidden)
        .fold(0_u64, |count, _| count.saturating_add(1));
    let tabs = trigger_parts(state, visible_count);
    let panels = panel_parts(state);
    let semantics = semantics::snapshot(state, &tabs, &panels);
    let semantic_issues = crate::attention_core::validate_semantic_snapshot(&semantics, None).issues;
    debug_assert!(tabs.len() <= state.catalog.tabs.len());
    debug_assert!(panels.len() <= state.catalog.panels.len());
    TabsProjection {
        generation: state.generation,
        list: TabListPart {
            id: "headless-tabs.list.v1".to_string(),
            orientation: state.orientation,
            activation: state.activation,
        },
        tabs,
        panels,
        semantics,
        semantic_issues,
    }
}

fn trigger_parts(state: &super::model::TabsState, visible_count: u64) -> Vec<TabTriggerPart> {
    let mut visible_position = 0_u64;
    let mut catalog_order = 0_u64;
    let parts = state
        .catalog
        .tabs
        .iter()
        .map(|tab| {
            let order = catalog_order;
            catalog_order = catalog_order.saturating_add(1);
            let position = if tab.is_hidden {
                None
            } else {
                visible_position = visible_position.saturating_add(1);
                Some(visible_position)
            };
            TabTriggerPart {
                id: tab.id.clone(),
                panel_id: tab.panel_id.clone(),
                label: tab.label.clone(),
                states: trigger_states(state, tab),
                order,
                position,
                set_size: visible_count,
                actions: trigger_actions(tab),
            }
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(parts.len(), state.catalog.tabs.len());
    debug_assert!(parts.len() <= super::model::MAX_TABS);
    parts
}

fn trigger_states(
    state: &super::model::TabsState,
    tab: &super::model::TabDescriptor,
) -> std::collections::BTreeSet<TabTriggerState> {
    let mut states = std::collections::BTreeSet::new();
    if tab.id == state.selected {
        states.insert(TabTriggerState::Selected);
    }
    if tab.id == state.focused {
        states.insert(TabTriggerState::Focused);
    }
    if tab.is_disabled {
        states.insert(TabTriggerState::Disabled);
    }
    if tab.is_hidden {
        states.insert(TabTriggerState::Hidden);
    }
    states
}

fn trigger_actions(tab: &super::model::TabDescriptor) -> Vec<TabPartAction> {
    if tab.is_disabled || tab.is_hidden {
        Vec::new()
    } else {
        vec![TabPartAction::Focus, TabPartAction::Select, TabPartAction::Activate]
    }
}

fn panel_parts(state: &super::model::TabsState) -> Vec<TabPanelPart> {
    state
        .catalog
        .tabs
        .iter()
        .filter_map(|tab| {
            state.catalog.panel(&tab.panel_id).map(|panel| TabPanelPart {
                id: panel.id.clone(),
                tab_id: tab.id.clone(),
                slot_id: panel.slot_id.clone(),
                is_active: tab.id == state.selected,
                is_hidden: tab.id != state.selected || tab.is_hidden,
            })
        })
        .collect()
}
