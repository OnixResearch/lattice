const MAX_IDENTITIES: usize = 128;
const MAX_ALIASES_PER_IDENTITY: usize = 8;
const MAX_ID_BYTES: usize = 96;
const MAX_IDENTITY_ISSUES: usize = 128;

semantic_string_id! {
    /// Stable deterministic-simulation scenario identity.
    pub struct SimulationScenarioId;
}
semantic_string_id! {
    /// Stable observation-probe identity.
    pub struct SimulationProbeId;
}
semantic_string_id! {
    /// Stable simulation invariant identity.
    pub struct SimulationInvariantId;
}
semantic_string_id! {
    /// Stable finite choice-point identity.
    pub struct SimulationChoicePointId;
}
semantic_string_id! {
    /// Stable canonical state-projection identity.
    pub struct SimulationStateProjectionId;
}
semantic_string_id! {
    /// Stable declarative effect-class identity.
    pub struct SimulationEffectClassId;
}
semantic_string_id! {
    /// Stable simulation adapter-version identity.
    pub struct SimulationAdapterVersionId;
}

/// Namespace carried by one simulation identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SimulationIdentityKind {
    /// Scenario identity.
    Scenario,
    /// Probe identity.
    Probe,
    /// Invariant identity.
    Invariant,
    /// Finite choice-point identity.
    ChoicePoint,
    /// State-projection identity.
    StateProjection,
    /// Effect-class identity.
    EffectClass,
    /// Adapter-version identity.
    AdapterVersion,
}

/// One checked durable identity and its prior aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationIdentityEntry {
    /// Identity namespace.
    pub kind: SimulationIdentityKind,
    /// Current stable semantic identity.
    pub id: String,
    /// Prior semantic identities retained for migration.
    pub aliases: Vec<String>,
}

impl SimulationIdentityEntry {
    /// Creates an identity entry without aliases.
    #[must_use]
    pub fn new(kind: SimulationIdentityKind, id: impl Into<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            aliases: Vec::new(),
        }
    }

    /// Attaches reviewed prior aliases.
    #[must_use]
    pub fn with_aliases(mut self, aliases: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.aliases = aliases.into_iter().map(Into::into).collect();
        self
    }
}

/// Stable identity-registry validation code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationIdentityDiagnosticCode {
    /// The registry exceeds its reviewed identity bound.
    RegistryTooLarge,
    /// An identity is empty, oversized, or malformed.
    InvalidIdentity,
    /// A durable identity is derived only from source location.
    SourceLocationIdentity,
    /// An identity is repeated in one namespace.
    DuplicateIdentity,
    /// An entry declares too many aliases.
    TooManyAliases,
    /// An alias is malformed.
    InvalidAlias,
    /// An alias is claimed more than once or conflicts with a current ID.
    ConflictingAlias,
    /// Two identities project to the same SDK `u32`.
    SdkProjectionCollision,
    /// A requested identity is absent from the checked registry.
    UnknownIdentity,
}

/// Stable identity-registry diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationIdentityDiagnostic {
    /// Machine-comparable code.
    pub code: SimulationIdentityDiagnosticCode,
    /// Identity namespace involved in the failure, when known.
    pub kind: Option<SimulationIdentityKind>,
    /// Safe bounded identity detail.
    pub identity: Option<String>,
}
