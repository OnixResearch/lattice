use ratcore::headless_components::TabId;
use ratcore::headless_components::TabsActivationPolicy;
use ratcore::headless_components::TabsAdapterReceipt;
use ratcore::headless_components::TabsCatalog;
use ratcore::headless_components::TabsDiagnostic;
use ratcore::headless_components::TabsMessage;
use ratcore::headless_components::TabsOrientation;
use ratcore::headless_components::TabsProjection;
use ratcore::headless_components::TabsState;
use ratcore::headless_components::TabsTransition;
use ratcore::headless_components::canonical_tabs_state;
use ratcore::headless_components::project_tabs;
use ratcore::headless_components::reduce_tabs;
use ratcore::headless_components::run_tabs_adapter_script;

/// Subwayrat-owned physical key vocabulary normalized before Ratcore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTabKey {
    /// Terminal left-arrow key.
    Left,
    /// Terminal right-arrow key.
    Right,
    /// Terminal up-arrow key.
    Up,
    /// Terminal down-arrow key.
    Down,
    /// Terminal Home key.
    Home,
    /// Terminal End key.
    End,
    /// Terminal Enter key.
    Enter,
    /// Terminal Space key.
    Space,
    /// Unhandled terminal key.
    Other,
}

/// Terminal-lifetime-owned adapter around accepted Ratcore tabs behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubwayTabsAdapter {
    state: TabsState,
}

impl SubwayTabsAdapter {
    /// Validates and constructs an adapter without storing terminal events or geometry.
    ///
    /// # Errors
    ///
    /// Returns shared catalog, selection, and focus diagnostics.
    pub fn new(
        catalog: TabsCatalog,
        selected: TabId,
        focused: TabId,
        orientation: TabsOrientation,
        activation: TabsActivationPolicy,
    ) -> Result<Self, Vec<TabsDiagnostic>> {
        TabsState::new(catalog, selected, focused, orientation, activation).map(|state| Self { state })
    }

    /// Returns the shared immutable projection consumed by Ratatui rendering.
    #[must_use]
    pub fn projection(&self) -> TabsProjection {
        project_tabs(&self.state)
    }

    /// Normalizes and applies one terminal key, leaving caller selection controlled.
    #[must_use]
    pub fn handle_key(&mut self, key: TerminalTabKey) -> Option<TabsTransition> {
        let message = normalize_terminal_tab_key(self.state.orientation(), key)?;
        Some(reduce_tabs(&mut self.state, message))
    }

    /// Applies one primary pointer activation after renderer hit testing.
    #[must_use]
    pub fn handle_primary_pointer(&mut self, id: TabId) -> TabsTransition {
        reduce_tabs(&mut self.state, TabsMessage::PointerActivate(id))
    }

    /// Commits a selection after the caller accepts a typed proposal.
    #[must_use]
    pub fn commit_selection(&mut self, id: TabId) -> TabsTransition {
        reduce_tabs(&mut self.state, TabsMessage::CommitSelection(id))
    }

    /// Atomically replaces the shared catalog and applies deterministic repair.
    #[must_use]
    pub fn replace_catalog(&mut self, catalog: TabsCatalog) -> TabsTransition {
        reduce_tabs(&mut self.state, TabsMessage::ReplaceCatalog(catalog))
    }
}

/// Maps terminal keys according to accepted orientation.
#[must_use]
pub const fn normalize_terminal_tab_key(orientation: TabsOrientation, key: TerminalTabKey) -> Option<TabsMessage> {
    match (orientation, key) {
        (TabsOrientation::Horizontal, TerminalTabKey::Left) | (TabsOrientation::Vertical, TerminalTabKey::Up) => {
            Some(TabsMessage::MovePrevious)
        }
        (TabsOrientation::Horizontal, TerminalTabKey::Right) | (TabsOrientation::Vertical, TerminalTabKey::Down) => {
            Some(TabsMessage::MoveNext)
        }
        (_, TerminalTabKey::Home) => Some(TabsMessage::MoveFirst),
        (_, TerminalTabKey::End) => Some(TabsMessage::MoveLast),
        (_, TerminalTabKey::Enter | TerminalTabKey::Space) => Some(TabsMessage::ActivateFocused),
        (TabsOrientation::Horizontal, TerminalTabKey::Up | TerminalTabKey::Down)
        | (TabsOrientation::Vertical, TerminalTabKey::Left | TerminalTabKey::Right)
        | (_, TerminalTabKey::Other) => None,
    }
}

/// Returns deterministic Subwayrat adapter evidence for cross-renderer comparison.
///
/// # Errors
///
/// Returns shared fixture diagnostics if the canonical pilot drifts.
pub fn subway_tabs_adapter_receipt() -> Result<TabsAdapterReceipt, Vec<TabsDiagnostic>> {
    let mut state = canonical_tabs_state(TabsActivationPolicy::Automatic)?;
    let orientation = state.orientation();
    let mut messages = Vec::new();
    if let Some(message) = normalize_terminal_tab_key(orientation, TerminalTabKey::Right) {
        messages.push(message);
    }
    messages.push(TabsMessage::PointerActivate(TabId::new("tab.history")));
    messages.push(TabsMessage::PointerActivate(TabId::new("tab.missing")));
    Ok(run_tabs_adapter_script(&mut state, messages))
}

#[cfg(test)]
mod tests;
