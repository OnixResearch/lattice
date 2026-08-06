mod vocabulary;
pub use vocabulary::SemanticAction;
pub use vocabulary::SemanticChecked;
pub use vocabulary::SemanticCurrent;
pub use vocabulary::SemanticPoliteness;
pub use vocabulary::SemanticProperty;
pub use vocabulary::SemanticRelationKind;
pub use vocabulary::SemanticRole;
pub use vocabulary::SemanticStateKind;

/// Monotonic semantic snapshot generation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SemanticGeneration(u64);

impl SemanticGeneration {
    /// Creates a generation from its portable representation.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the portable representation.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Renderer-neutral node state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SemanticState {
    /// Presence-only disabled, hidden, required, and invalid flags.
    pub flags: std::collections::BTreeSet<SemanticStateKind>,
    /// Optional selection state.
    pub selected: Option<bool>,
    /// Optional checked state.
    pub checked: Option<SemanticChecked>,
    /// Optional expanded state.
    pub expanded: Option<bool>,
    /// Optional current-item state.
    pub current: Option<SemanticCurrent>,
}

impl SemanticState {
    /// Adds a presence-only state flag.
    #[must_use]
    pub fn with_flag(mut self, flag: SemanticStateKind) -> Self {
        self.flags.insert(flag);
        self
    }

    /// Returns whether the node is disabled.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        self.flags.contains(&SemanticStateKind::Disabled)
    }

    /// Returns whether the node is hidden.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.flags.contains(&SemanticStateKind::Hidden)
    }

    /// Returns whether the node is required.
    #[must_use]
    pub fn is_required(&self) -> bool {
        self.flags.contains(&SemanticStateKind::Required)
    }

    /// Returns whether accepted input is invalid.
    #[must_use]
    pub fn is_invalid(&self) -> bool {
        self.flags.contains(&SemanticStateKind::Invalid)
    }
}

/// Checked integer range metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticRange {
    /// Inclusive minimum.
    pub minimum: i64,
    /// Inclusive maximum.
    pub maximum: i64,
    /// Current value.
    pub current: i64,
    /// Positive adjustment step.
    pub step: u64,
}

/// Semantic value metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticValue {
    /// Human-readable value text.
    Text(String),
    /// Checked numeric range.
    Range(SemanticRange),
}

/// Collection position metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticCollection {
    /// One-based hierarchy level.
    pub level: u64,
    /// One-based item position.
    pub position: u64,
    /// Number of items in the set.
    pub set_size: u64,
}

/// Directed semantic relationship.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticRelation {
    /// Relationship kind.
    pub kind: SemanticRelationKind,
    /// Target node.
    pub target: super::SemanticNodeId,
}

/// Renderer-neutral live announcement metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticAnnouncement {
    /// Announcement urgency.
    pub politeness: SemanticPoliteness,
    /// Bounded application-supplied message.
    pub message: String,
}

/// Focus graph to semantic tree mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticFocusLink {
    /// Existing focus node identity.
    pub focus: super::super::FocusNodeId,
    /// Semantic node identity.
    pub semantic: super::SemanticNodeId,
}

/// One accepted semantic node projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticNode {
    /// Stable node identity.
    pub id: super::SemanticNodeId,
    /// Owning surface.
    pub surface: super::SemanticSurfaceId,
    /// Optional semantic parent.
    pub parent: Option<super::SemanticNodeId>,
    /// Stable order among siblings.
    pub order: u64,
    /// Bounded semantic role.
    pub role: SemanticRole,
    /// Accessible name, or empty when `LabelledBy` supplies it.
    pub name: String,
    /// Optional accessible description.
    pub description: Option<String>,
    /// Semantic state projected from accepted application state.
    pub state: SemanticState,
    /// Optional value metadata.
    pub value: Option<SemanticValue>,
    /// Optional collection metadata.
    pub collection: Option<SemanticCollection>,
    /// Directed relationships.
    pub relations: Vec<SemanticRelation>,
    /// Exposed actions and their typed application targets.
    pub actions: std::collections::BTreeMap<SemanticAction, crate::nexus_core::ActionId>,
    /// Optional live announcement metadata.
    pub announcement: Option<SemanticAnnouncement>,
}

impl SemanticNode {
    /// Creates a root candidate with no parent and default metadata.
    #[must_use]
    pub fn new(
        id: impl Into<super::SemanticNodeId>,
        surface: impl Into<super::SemanticSurfaceId>,
        role: SemanticRole,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            surface: surface.into(),
            parent: None,
            order: 0,
            role,
            name: name.into(),
            description: None,
            state: SemanticState::default(),
            value: None,
            collection: None,
            relations: Vec::new(),
            actions: std::collections::BTreeMap::new(),
            announcement: None,
        }
    }

    /// Assigns a parent and sibling order.
    #[must_use]
    pub fn child_of(mut self, parent: impl Into<super::SemanticNodeId>, order: u64) -> Self {
        self.parent = Some(parent.into());
        self.order = order;
        self
    }

    /// Exposes an action through a typed application action identity.
    #[must_use]
    pub fn with_action(mut self, action: SemanticAction, target: impl Into<crate::nexus_core::ActionId>) -> Self {
        self.actions.insert(action, target.into());
        self
    }

    /// Adds a directed relationship.
    #[must_use]
    pub fn with_relation(mut self, kind: SemanticRelationKind, target: impl Into<super::SemanticNodeId>) -> Self {
        self.relations.push(SemanticRelation {
            kind,
            target: target.into(),
        });
        self
    }
}

/// One semantic surface and its required root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSurface {
    /// Stable surface identity.
    pub id: super::SemanticSurfaceId,
    /// Root node identity.
    pub root: super::SemanticNodeId,
}

impl SemanticSurface {
    /// Creates a surface descriptor.
    #[must_use]
    pub fn new(id: impl Into<super::SemanticSurfaceId>, root: impl Into<super::SemanticNodeId>) -> Self {
        Self {
            id: id.into(),
            root: root.into(),
        }
    }
}

/// Complete immutable semantic projection candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSnapshot {
    /// Monotonic generation.
    pub generation: SemanticGeneration,
    /// Declared surfaces.
    pub surfaces: Vec<SemanticSurface>,
    /// Ordered node records.
    pub nodes: Vec<SemanticNode>,
    /// Explicit focus-to-semantic links.
    pub focus_links: Vec<SemanticFocusLink>,
}

impl SemanticSnapshot {
    /// Creates a snapshot without focus links.
    #[must_use]
    pub fn new(generation: SemanticGeneration, surfaces: Vec<SemanticSurface>, nodes: Vec<SemanticNode>) -> Self {
        Self {
            generation,
            surfaces,
            nodes,
            focus_links: Vec::new(),
        }
    }

    /// Returns a node by semantic identity.
    #[must_use]
    pub fn node(&self, id: &super::SemanticNodeId) -> Option<&SemanticNode> {
        self.nodes.iter().find(|node| &node.id == id)
    }

    /// Returns all action kinds exposed by this snapshot.
    #[must_use]
    pub fn action_kinds(&self) -> std::collections::BTreeSet<SemanticAction> {
        self.nodes.iter().flat_map(|node| node.actions.keys().copied()).collect()
    }
}
