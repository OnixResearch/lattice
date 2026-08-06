pub(crate) mod validation;
pub use validation::validate_tabs_candidate;

pub(super) const MAX_TABS: usize = 32;
pub(super) const MAX_TAB_TEXT_BYTES: usize = 96;

semantic_string_id! {
    /// Stable identity of one tab trigger.
    pub struct TabId;
}

semantic_string_id! {
    /// Stable identity of one associated tab panel.
    pub struct TabPanelId;
}

semantic_string_id! {
    /// Caller-owned panel content slot identity.
    pub struct TabSlotId;
}

/// Tab-list orientation used by shared navigation policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsOrientation {
    /// Left and right arrows traverse triggers.
    Horizontal,
    /// Up and down arrows traverse triggers.
    Vertical,
}

impl TabsOrientation {
    /// Returns the stable projection spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Selection proposal policy after focus changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabsActivationPolicy {
    /// Focus changes propose selection immediately.
    Automatic,
    /// Focus changes require an explicit activation message.
    Manual,
}

impl TabsActivationPolicy {
    /// Returns the stable projection spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::Manual => "manual",
        }
    }
}

/// One stable tab trigger descriptor without panel content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabDescriptor {
    /// Stable trigger identity.
    pub id: TabId,
    /// Stable associated panel identity.
    pub panel_id: TabPanelId,
    /// Bounded semantic label.
    pub label: String,
    /// Whether activation and focus are unavailable.
    pub is_disabled: bool,
    /// Whether the trigger is absent from traversal and visual projection.
    pub is_hidden: bool,
}

/// One caller-owned panel slot descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabPanelDescriptor {
    /// Stable panel identity.
    pub id: TabPanelId,
    /// Caller-owned slot; Ratcore never stores slot content.
    pub slot_id: TabSlotId,
}

/// Atomically validated ordered tabs catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsCatalog {
    /// Ordered tab trigger descriptors.
    pub tabs: Vec<TabDescriptor>,
    /// Associated caller-owned panel descriptors.
    pub panels: Vec<TabPanelDescriptor>,
}

impl TabsCatalog {
    /// Returns a tab by stable identity.
    #[must_use]
    pub fn tab(&self, id: &TabId) -> Option<&TabDescriptor> {
        self.tabs.iter().find(|tab| &tab.id == id)
    }

    /// Returns a panel by stable identity.
    #[must_use]
    pub fn panel(&self, id: &TabPanelId) -> Option<&TabPanelDescriptor> {
        self.panels.iter().find(|panel| &panel.id == id)
    }

    pub(super) fn eligible_tabs(&self) -> impl Iterator<Item = &TabDescriptor> {
        self.tabs.iter().filter(|tab| !tab.is_disabled && !tab.is_hidden)
    }

    pub(super) fn is_eligible(&self, id: &TabId) -> bool {
        self.tab(id).is_some_and(|tab| !tab.is_disabled && !tab.is_hidden)
    }
}

/// Accepted split-ownership tabs state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabsState {
    pub(super) catalog: TabsCatalog,
    pub(super) selected: TabId,
    pub(super) focused: TabId,
    pub(super) orientation: TabsOrientation,
    pub(super) activation: TabsActivationPolicy,
    pub(super) generation: u64,
}

impl TabsState {
    /// Validates and constructs an accepted tabs state.
    ///
    /// # Errors
    ///
    /// Returns all bounded catalog, selection, and focus diagnostics.
    pub fn new(
        catalog: TabsCatalog,
        selected: TabId,
        focused: TabId,
        orientation: TabsOrientation,
        activation: TabsActivationPolicy,
    ) -> Result<Self, Vec<TabsDiagnostic>> {
        let issues = validation::validate_tabs_candidate(&catalog, &selected, &focused);
        if issues.is_empty() {
            Ok(Self {
                catalog,
                selected,
                focused,
                orientation,
                activation,
                generation: 1,
            })
        } else {
            Err(issues)
        }
    }

    /// Returns the accepted catalog.
    #[must_use]
    pub const fn catalog(&self) -> &TabsCatalog {
        &self.catalog
    }

    /// Returns the caller-accepted selected identity.
    #[must_use]
    pub const fn selected(&self) -> &TabId {
        &self.selected
    }

    /// Returns reducer-owned roving focus identity.
    #[must_use]
    pub const fn focused(&self) -> &TabId {
        &self.focused
    }

    /// Returns tab-list orientation.
    #[must_use]
    pub const fn orientation(&self) -> TabsOrientation {
        self.orientation
    }

    /// Returns activation policy.
    #[must_use]
    pub const fn activation(&self) -> TabsActivationPolicy {
        self.activation
    }

    /// Returns the accepted state generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

/// Stable tabs catalog, state, reducer, and projection diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TabsDiagnosticCode {
    /// Catalog has no trigger or panel pair.
    EmptyCatalog,
    /// Catalog exceeds its fixed bound.
    CatalogTooLarge,
    /// Trigger identity occurs more than once.
    DuplicateTab,
    /// Panel identity occurs more than once.
    DuplicatePanel,
    /// Identity, label, or slot shape is invalid.
    InvalidField,
    /// Trigger references a missing panel.
    MissingPanel,
    /// Panel has no associated trigger.
    OrphanPanel,
    /// Catalog contains no focusable and selectable trigger.
    NoEligibleTab,
    /// Selected identity is absent.
    UnknownSelection,
    /// Selected identity is hidden or disabled.
    IneligibleSelection,
    /// Focused identity is absent.
    UnknownFocus,
    /// Focused identity is hidden or disabled.
    IneligibleFocus,
    /// Message target identity is absent.
    UnknownTarget,
    /// Message target identity is hidden or disabled.
    IneligibleTarget,
    /// Caller attempted to commit an invalid selection.
    InvalidSelectionCommit,
    /// Adapter evidence script exceeded its fixed message bound.
    ScriptTooLarge,
}

/// One stable tabs diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TabsDiagnostic {
    /// Stable diagnostic code.
    pub code: TabsDiagnosticCode,
    /// Bounded stable identity or field subject.
    pub subject: String,
}
