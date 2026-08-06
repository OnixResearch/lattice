use super::super::ActionBindingId;
use super::super::BindingLayerId;
use super::ActionBinding;
use super::BindingCatalogLimits;
use super::BindingLayer;
use super::ReservedKeySequence;

/// Candidate catalog validated and accepted atomically.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCatalogCandidate {
    /// Monotonic catalog generation.
    pub generation: u64,
    /// Declared binding layers.
    pub layers: Vec<BindingLayer>,
    /// Candidate bindings.
    pub bindings: Vec<ActionBinding>,
    /// Reserved sequence policy.
    pub reserved: Vec<ReservedKeySequence>,
    /// Resource policy.
    pub limits: BindingCatalogLimits,
}

impl BindingCatalogCandidate {
    /// Creates a candidate with default limits and no reserved sequences.
    #[must_use]
    pub fn new(generation: u64, layers: Vec<BindingLayer>, bindings: Vec<ActionBinding>) -> Self {
        Self {
            generation,
            layers,
            bindings,
            reserved: Vec::new(),
            limits: BindingCatalogLimits::default(),
        }
    }

    /// Sets reserved sequences.
    #[must_use]
    pub fn with_reserved(mut self, reserved: Vec<ReservedKeySequence>) -> Self {
        self.reserved = reserved;
        self
    }

    /// Sets resource limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: BindingCatalogLimits) -> Self {
        self.limits = limits;
        self
    }
}

/// Stable binding-catalog validation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingCatalogDiagnosticCode {
    /// Generation or lifecycle generation is zero/non-monotonic.
    NonMonotonicGeneration,
    /// Resource limits are zero or exceed hard safety maxima.
    InvalidLimits,
    /// A catalog identity/source/reason is empty or not normalized.
    MalformedIdentity,
    /// A binding identity is duplicated.
    DuplicateBindingId,
    /// A layer identity is duplicated.
    DuplicateLayerId,
    /// A binding references an unknown action.
    UnknownAction,
    /// A binding references an unknown layer.
    UnknownLayer,
    /// An override has inconsistent replacement provenance.
    MalformedOverride,
    /// A sequence is reserved by policy.
    ReservedSequence,
    /// Equal-precedence bindings contain an exact duplicate.
    DuplicateSequence,
    /// Equal-precedence exact and prefix bindings conflict.
    PrefixConflict,
    /// A configured collection exceeds its accepted bound.
    LimitExceeded,
    /// One prefix has too many immediate continuations.
    ContinuationFanoutExceeded,
    /// Active layer stack repeats a layer.
    DuplicateActiveLayer,
    /// Active layer observes a stale lifecycle generation.
    StaleLayerGeneration,
    /// Active context generation is zero.
    StaleContextGeneration,
    /// A disabled layer was requested as active.
    DisabledLayer,
    /// An edit observes an old catalog generation.
    StaleEditGeneration,
    /// An edit references an unknown binding.
    UnknownBinding,
    /// A renderer rejected a captured physical key.
    UnsupportedCapture,
}

/// Deterministic catalog or active-layer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCatalogDiagnostic {
    /// Stable code.
    pub code: BindingCatalogDiagnosticCode,
    /// Related binding IDs in stable order.
    pub binding_ids: Vec<ActionBindingId>,
    /// Related layer IDs in stable order.
    pub layer_ids: Vec<BindingLayerId>,
    /// Related action when available.
    pub action_id: Option<super::super::super::ActionId>,
    /// Affected field.
    pub field: String,
    /// Human-readable explanation.
    pub message: String,
}

impl BindingCatalogDiagnostic {
    pub(in crate::nexus_core::binding_core) fn new(
        code: BindingCatalogDiagnosticCode,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            binding_ids: Vec::new(),
            layer_ids: Vec::new(),
            action_id: None,
            field: field.into(),
            message: message.into(),
        }
    }

    pub(in crate::nexus_core::binding_core) fn with_binding(mut self, id: ActionBindingId) -> Self {
        self.binding_ids.push(id);
        self.binding_ids.sort();
        self.binding_ids.dedup();
        self
    }

    pub(in crate::nexus_core::binding_core) fn with_layer(mut self, id: BindingLayerId) -> Self {
        self.layer_ids.push(id);
        self.layer_ids.sort();
        self.layer_ids.dedup();
        self
    }

    pub(in crate::nexus_core::binding_core) fn with_action(mut self, id: super::super::super::ActionId) -> Self {
        self.action_id = Some(id);
        self
    }
}

/// One generation-bound active layer reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveBindingLayer {
    /// Active layer identity.
    pub layer_id: BindingLayerId,
    /// Layer lifecycle generation observed by the shell.
    pub observed_generation: u64,
}

impl ActiveBindingLayer {
    /// Creates an active layer reference.
    #[must_use]
    pub fn new(layer_id: impl Into<BindingLayerId>, observed_generation: u64) -> Self {
        Self {
            layer_id: layer_id.into(),
            observed_generation,
        }
    }
}

/// Validated active layer stack for one accepted context generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveBindingLayers {
    pub(in crate::nexus_core::binding_core) context_generation: u64,
    pub(in crate::nexus_core::binding_core) layers: Vec<ActiveBindingLayer>,
}

impl ActiveBindingLayers {
    /// Returns the accepted context/focus generation.
    #[must_use]
    pub const fn context_generation(&self) -> u64 {
        self.context_generation
    }

    /// Returns layers in deterministic descending-precedence order.
    #[must_use]
    pub fn layers(&self) -> &[ActiveBindingLayer] {
        &self.layers
    }
}

/// Accepted immutable binding catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCatalogSnapshot {
    pub(in crate::nexus_core::binding_core) generation: u64,
    pub(in crate::nexus_core::binding_core) layers: Vec<BindingLayer>,
    pub(in crate::nexus_core::binding_core) bindings: Vec<ActionBinding>,
    pub(in crate::nexus_core::binding_core) reserved: Vec<ReservedKeySequence>,
    pub(in crate::nexus_core::binding_core) limits: BindingCatalogLimits,
}

impl BindingCatalogSnapshot {
    /// Returns the catalog generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns accepted layers in stable identity order.
    #[must_use]
    pub fn layers(&self) -> &[BindingLayer] {
        &self.layers
    }

    /// Returns accepted bindings in stable identity order.
    #[must_use]
    pub fn bindings(&self) -> &[ActionBinding] {
        &self.bindings
    }

    /// Returns configured resource limits.
    #[must_use]
    pub const fn limits(&self) -> BindingCatalogLimits {
        self.limits
    }

    /// Finds one accepted binding.
    #[must_use]
    pub fn binding(&self, id: &ActionBindingId) -> Option<&ActionBinding> {
        self.bindings.iter().find(|binding| &binding.id == id)
    }

    /// Finds one accepted layer.
    #[must_use]
    pub fn layer(&self, id: &BindingLayerId) -> Option<&BindingLayer> {
        self.layers.iter().find(|layer| &layer.id == id)
    }

    /// Validates and orders an explicit active-layer stack.
    ///
    /// # Errors
    ///
    /// Returns deterministic diagnostics for unknown, duplicate, disabled, stale,
    /// or excessive active layers and for a zero context generation.
    pub fn activate(
        &self,
        context_generation: u64,
        layers: Vec<ActiveBindingLayer>,
    ) -> Result<ActiveBindingLayers, Vec<BindingCatalogDiagnostic>> {
        super::super::validation::validate_active_layers(self, context_generation, layers)
    }

    pub(in crate::nexus_core::binding_core) fn candidate_with_generation(
        &self,
        generation: u64,
    ) -> BindingCatalogCandidate {
        BindingCatalogCandidate {
            generation,
            layers: self.layers.clone(),
            bindings: self.bindings.clone(),
            reserved: self.reserved.clone(),
            limits: self.limits,
        }
    }
}

/// Result of atomic binding-catalog validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingCatalogBuild {
    /// Accepted candidate or preserved previous snapshot.
    pub snapshot: Option<BindingCatalogSnapshot>,
    /// Deterministic rejection diagnostics.
    pub diagnostics: Vec<BindingCatalogDiagnostic>,
    /// Whether the candidate replaced the previous snapshot.
    pub candidate_accepted: bool,
}

/// Validates and atomically accepts one catalog candidate.
#[must_use]
pub fn accept_binding_catalog(
    candidate: BindingCatalogCandidate,
    registry: &super::super::super::ActionRegistrySnapshot,
    previous: Option<&BindingCatalogSnapshot>,
) -> BindingCatalogBuild {
    super::super::validation::accept(candidate, registry, previous)
}
