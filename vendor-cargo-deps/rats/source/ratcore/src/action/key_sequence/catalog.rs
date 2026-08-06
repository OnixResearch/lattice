use super::ActionBindingId;
use super::BindingLayerId;
use super::KeySequence;

const DEFAULT_MAX_SEQUENCE_STROKES: u16 = 8;
const DEFAULT_MAX_BINDINGS: u32 = 4_096;
const DEFAULT_MAX_LAYERS: u16 = 32;
const DEFAULT_MAX_ACTIVE_LAYERS: u16 = 16;
const DEFAULT_MAX_CONTINUATIONS: u16 = 64;
const HARD_MAX_SEQUENCE_STROKES: u16 = 32;
const HARD_MAX_BINDINGS: u32 = 65_536;
const HARD_MAX_LAYERS: u16 = 256;
const HARD_MAX_ACTIVE_LAYERS: u16 = 128;
const HARD_MAX_CONTINUATIONS: u16 = 512;

/// Bounded resource policy for catalogs and sequence resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BindingCatalogLimits {
    /// Maximum strokes in one sequence.
    pub sequence_strokes: u16,
    /// Maximum bindings in one catalog.
    pub bindings: u32,
    /// Maximum declared layers.
    pub layers: u16,
    /// Maximum layers active in one context snapshot.
    pub active_layers: u16,
    /// Maximum immediate continuations for one prefix.
    pub continuations: u16,
}

impl Default for BindingCatalogLimits {
    fn default() -> Self {
        Self {
            sequence_strokes: DEFAULT_MAX_SEQUENCE_STROKES,
            bindings: DEFAULT_MAX_BINDINGS,
            layers: DEFAULT_MAX_LAYERS,
            active_layers: DEFAULT_MAX_ACTIVE_LAYERS,
            continuations: DEFAULT_MAX_CONTINUATIONS,
        }
    }
}

impl BindingCatalogLimits {
    pub(super) const fn valid(self) -> bool {
        self.sequence_strokes > 0
            && self.sequence_strokes <= HARD_MAX_SEQUENCE_STROKES
            && self.bindings > 0
            && self.bindings <= HARD_MAX_BINDINGS
            && self.layers > 0
            && self.layers <= HARD_MAX_LAYERS
            && self.active_layers > 0
            && self.active_layers <= HARD_MAX_ACTIVE_LAYERS
            && self.continuations > 0
            && self.continuations <= HARD_MAX_CONTINUATIONS
    }
}

/// Semantic role of one binding layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingLayerKind {
    /// Built-in application fallback.
    Application,
    /// Active workspace layer.
    Workspace,
    /// Active pane layer.
    Pane,
    /// Focused component or content layer.
    FocusedComponent,
    /// User-configured override layer.
    User,
    /// Explicit transient map.
    Transient,
    /// Modal surface that blocks lower layers for matching prefixes.
    Modal,
}

/// One declared binding layer and its lifecycle generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingLayer {
    /// Stable layer identity.
    pub id: BindingLayerId,
    /// Higher values resolve before lower values.
    pub precedence: i16,
    /// Semantic role.
    pub kind: BindingLayerKind,
    /// Layer lifecycle generation.
    pub generation: u64,
    /// Optional accepted contextual owner identity.
    pub context_id: Option<super::super::ActionContextId>,
    /// Whether the layer is eligible for activation.
    pub enabled: bool,
    /// Reason exposed when disabled.
    pub disabled_reason: Option<String>,
}

impl BindingLayer {
    /// Creates an enabled layer.
    #[must_use]
    pub fn new(id: impl Into<BindingLayerId>, precedence: i16, kind: BindingLayerKind, generation: u64) -> Self {
        Self {
            id: id.into(),
            precedence,
            kind,
            generation,
            context_id: None,
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Associates the layer with an accepted contextual owner.
    #[must_use]
    pub fn with_context(mut self, context_id: impl Into<super::super::ActionContextId>) -> Self {
        self.context_id = Some(context_id.into());
        self
    }

    /// Disables the layer with an inspectable reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// Provenance class for an action binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionBindingSourceKind {
    /// Built-in default.
    Default,
    /// Application-provided binding.
    Application,
    /// Component-provided binding.
    Component,
    /// Transient-surface binding.
    Transient,
    /// User-added or user-replaced binding.
    UserOverride,
}

/// Source and override provenance for one binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBindingSource {
    /// Stable source label.
    pub name: String,
    /// Source class.
    pub kind: ActionBindingSourceKind,
    /// Replaced binding identity for replacement overrides.
    pub replaces: Option<ActionBindingId>,
}

impl ActionBindingSource {
    /// Creates source provenance.
    #[must_use]
    pub fn new(name: impl Into<String>, kind: ActionBindingSourceKind) -> Self {
        Self {
            name: name.into(),
            kind,
            replaces: None,
        }
    }

    /// Marks this source as replacing one accepted binding.
    #[must_use]
    pub fn replacing(mut self, binding_id: impl Into<ActionBindingId>) -> Self {
        self.replaces = Some(binding_id.into());
        self
    }
}

/// One action-to-sequence binding record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionBinding {
    /// Stable binding identity.
    pub id: ActionBindingId,
    /// Bound action identity.
    pub action_id: super::super::ActionId,
    /// Complete logical sequence.
    pub sequence: KeySequence,
    /// Owning layer.
    pub layer_id: BindingLayerId,
    /// Source/default/override provenance.
    pub source: ActionBindingSource,
    /// Whether this binding is currently eligible.
    pub enabled: bool,
    /// Inspectable disabled reason.
    pub disabled_reason: Option<String>,
}

impl ActionBinding {
    /// Creates an enabled action binding.
    #[must_use]
    pub fn new(
        id: impl Into<ActionBindingId>,
        action_id: impl Into<super::super::ActionId>,
        sequence: KeySequence,
        layer_id: impl Into<BindingLayerId>,
        source: ActionBindingSource,
    ) -> Self {
        Self {
            id: id.into(),
            action_id: action_id.into(),
            sequence,
            layer_id: layer_id.into(),
            source,
            enabled: true,
            disabled_reason: None,
        }
    }

    /// Disables this binding with an inspectable reason.
    #[must_use]
    pub fn disabled(mut self, reason: impl Into<String>) -> Self {
        self.enabled = false;
        self.disabled_reason = Some(reason.into());
        self
    }
}

/// Policy-reserved complete sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedKeySequence {
    /// Reserved sequence.
    pub sequence: KeySequence,
    /// Reason presented by validation/editing.
    pub reason: String,
}

impl ReservedKeySequence {
    /// Creates a reserved-sequence policy entry.
    #[must_use]
    pub fn new(sequence: KeySequence, reason: impl Into<String>) -> Self {
        Self {
            sequence,
            reason: reason.into(),
        }
    }
}

#[path = "catalog/compatibility.rs"]
mod compatibility;
#[path = "catalog/state.rs"]
mod state;

pub use compatibility::binding_catalog_from_registry_defaults;
pub use state::ActiveBindingLayer;
pub use state::ActiveBindingLayers;
pub use state::BindingCatalogBuild;
pub use state::BindingCatalogCandidate;
pub use state::BindingCatalogDiagnostic;
pub use state::BindingCatalogDiagnosticCode;
pub use state::BindingCatalogSnapshot;
pub use state::accept_binding_catalog;
