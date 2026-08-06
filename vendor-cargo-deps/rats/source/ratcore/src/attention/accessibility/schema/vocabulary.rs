/// Bounded semantic role vocabulary used by accepted fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticRole {
    /// Application or top-level surface.
    Application,
    /// Modal or non-modal dialog.
    Dialog,
    /// Menu collection.
    Menu,
    /// Menu command or option.
    MenuItem,
    /// Hierarchical tree collection.
    Tree,
    /// Hierarchical tree item.
    TreeItem,
    /// Heading or visible label.
    Heading,
    /// Non-focusable descriptive content.
    Description,
    /// Button control.
    Button,
    /// Progress or adjustable range.
    Progress,
    /// Status or live message.
    Status,
    /// Structural group.
    Group,
    /// Tabs trigger collection.
    TabList,
    /// One selectable tab trigger.
    Tab,
    /// Content panel associated with a tab trigger.
    TabPanel,
}

impl SemanticRole {
    /// Returns the stable role name used by renderer summaries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Dialog => "dialog",
            Self::Menu => "menu",
            Self::MenuItem => "menu-item",
            Self::Tree => "tree",
            Self::TreeItem => "tree-item",
            Self::Heading => "heading",
            Self::Description => "description",
            Self::Button => "button",
            Self::Progress => "progress",
            Self::Status => "status",
            Self::Group => "group",
            Self::TabList => "tab-list",
            Self::Tab => "tab",
            Self::TabPanel => "tab-panel",
        }
    }
}

/// Tri-state checked value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticChecked {
    /// Not checked.
    False,
    /// Checked.
    True,
    /// Partially checked.
    Mixed,
}

/// Current-item classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticCurrent {
    /// Current item without a narrower category.
    Item,
    /// Current page.
    Page,
    /// Current step.
    Step,
}

/// State vocabulary used for renderer support declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticStateKind {
    /// Disabled state.
    Disabled,
    /// Hidden state.
    Hidden,
    /// Selection state.
    Selected,
    /// Checked state.
    Checked,
    /// Expansion state.
    Expanded,
    /// Current-item state.
    Current,
    /// Required state.
    Required,
    /// Invalid state.
    Invalid,
}

/// Property vocabulary used for renderer support declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticProperty {
    /// Accessible name.
    Name,
    /// Accessible description.
    Description,
    /// Value or range.
    Value,
    /// Collection level and position.
    Collection,
    /// Live announcement metadata.
    Announcement,
}

/// Relationship vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticRelationKind {
    /// Node owns another semantic node.
    Owns,
    /// Control affects another semantic node.
    Controls,
    /// Name is supplied by another node.
    LabelledBy,
    /// Description is supplied by another node.
    DescribedBy,
}

/// Semantic action vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SemanticAction {
    /// Activate the control.
    Activate,
    /// Increment a range.
    Increment,
    /// Decrement a range.
    Decrement,
    /// Expand a collection item.
    Expand,
    /// Collapse a collection item.
    Collapse,
    /// Dismiss a surface.
    Dismiss,
    /// Select an item.
    Select,
    /// Request application focus.
    Focus,
    /// Set a value explicitly.
    SetValue,
}

/// Live announcement urgency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticPoliteness {
    /// Announce when the adapter is idle.
    Polite,
    /// Interrupt with urgent information where supported.
    Assertive,
}
