mod artifact;
pub use artifact::*;

const MAX_ADAPTER_MESSAGES: usize = 64;
const MAX_ADAPTER_EVENTS: usize = 192;
const MAX_ADAPTER_DIAGNOSTICS: usize = 65;

/// Renderer-neutral adapter convergence receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsAdapterReceipt {
    /// Accepted selected identity after caller commits proposals.
    pub selected: super::TabId,
    /// Accepted focused identity.
    pub focused: super::TabId,
    /// Ordered transition events.
    pub events: Vec<super::TabsTransitionEvent>,
    /// Ordered diagnostics.
    pub diagnostics: Vec<super::TabsDiagnostic>,
    /// Canonical semantic snapshot identity.
    pub semantic_identity: String,
}

impl TabsAdapterReceipt {
    /// Renders stable line-oriented adapter evidence.
    #[must_use]
    pub fn render(&self) -> String {
        format!(
            "selected={}\nfocused={}\nevents={:?}\ndiagnostics={:?}\nsemantic_identity={}\n",
            self.selected.as_str(),
            self.focused.as_str(),
            self.events,
            self.diagnostics,
            self.semantic_identity,
        )
    }
}

/// Returns the canonical three-tab pilot state.
///
/// # Errors
///
/// Returns catalog or accepted-state diagnostics if the built-in fixture drifts.
pub fn canonical_tabs_state(
    activation: super::TabsActivationPolicy,
) -> Result<super::TabsState, Vec<super::TabsDiagnostic>> {
    super::TabsState::new(
        fixture_catalog(FixtureAvailability::default()),
        super::TabId::new("tab.overview"),
        super::TabId::new("tab.overview"),
        super::TabsOrientation::Horizontal,
        activation,
    )
}

/// Runs semantic messages and commits every caller selection proposal in order.
#[must_use]
pub fn run_tabs_adapter_script(
    state: &mut super::TabsState,
    messages: impl IntoIterator<Item = super::TabsMessage>,
) -> TabsAdapterReceipt {
    let mut events = Vec::with_capacity(MAX_ADAPTER_EVENTS);
    let mut diagnostics = Vec::with_capacity(MAX_ADAPTER_DIAGNOSTICS);
    for (index, message) in messages.into_iter().enumerate() {
        if index >= MAX_ADAPTER_MESSAGES {
            diagnostics.push(super::model::validation::tabs_issue(
                super::TabsDiagnosticCode::ScriptTooLarge,
                "adapter-script",
            ));
            break;
        }
        let transition = super::reduce_tabs(state, message);
        events.extend(transition.events);
        diagnostics.extend(transition.diagnostics);
        for intent in transition.intents {
            let super::TabsIntent::SelectionProposed { tab_id, .. } = intent;
            let committed = super::reduce_tabs(state, super::TabsMessage::CommitSelection(tab_id));
            events.extend(committed.events);
            diagnostics.extend(committed.diagnostics);
        }
    }
    let projection = super::project_tabs(state);
    debug_assert!(events.len() <= MAX_ADAPTER_EVENTS);
    debug_assert!(diagnostics.len() <= MAX_ADAPTER_DIAGNOSTICS);
    TabsAdapterReceipt {
        selected: state.selected().clone(),
        focused: state.focused().clone(),
        events,
        diagnostics,
        semantic_identity: crate::attention_core::semantic_snapshot_hash(&projection.semantics),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct FixtureAvailability {
    is_details_disabled: bool,
    is_history_hidden: bool,
}

fn fixture_catalog(options: FixtureAvailability) -> super::TabsCatalog {
    let tab_rows = [
        ("tab.overview", "panel.overview", "Overview", false, false),
        ("tab.details", "panel.details", "Details", options.is_details_disabled, false),
        ("tab.history", "panel.history", "History", false, options.is_history_hidden),
    ];
    let catalog = super::TabsCatalog {
        tabs: tab_rows
            .iter()
            .map(|(id, panel_id, label, is_disabled, is_hidden)| super::TabDescriptor {
                id: super::TabId::new(*id),
                panel_id: super::TabPanelId::new(*panel_id),
                label: (*label).to_string(),
                is_disabled: *is_disabled,
                is_hidden: *is_hidden,
            })
            .collect(),
        panels: tab_rows
            .iter()
            .map(|(_, panel_id, _, _, _)| super::TabPanelDescriptor {
                id: super::TabPanelId::new(*panel_id),
                slot_id: super::TabSlotId::new(panel_id.replacen("panel.", "slot.", 1)),
            })
            .collect(),
    };
    debug_assert_eq!(catalog.tabs.len(), tab_rows.len());
    debug_assert_eq!(catalog.panels.len(), tab_rows.len());
    catalog
}
