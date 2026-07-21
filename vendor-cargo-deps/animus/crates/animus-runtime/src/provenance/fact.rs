/// Authorization decision status used by facts and effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceAuthorizationStatus {
    /// Effect was allowed by policy.
    Allowed,
    /// Effect was denied by policy.
    Denied,
    /// Effect requires external confirmation.
    RequiresConfirmation,
}

impl ProvenanceAuthorizationStatus {
    /// Returns the stable authorization label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::RequiresConfirmation => "requires-confirmation",
        }
    }
}

/// Append-only safe provenance fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProvenanceFact {
    /// Model activity safe metadata.
    ModelActivity(ProvenanceFactModelActivity),
    /// Tool activity safe metadata.
    ToolActivity(ProvenanceFactToolActivity),
    /// Runtime block safe metadata.
    RuntimeBlock(ProvenanceFactRuntimeBlock),
    /// Error safe metadata.
    Error(ProvenanceFactError),
    /// Artifact reference fact.
    Artifact(ProvenanceFactArtifact),
    /// Authorization decision fact.
    Authorization(ProvenanceFactAuthorization),
    /// Provenance observation fact.
    Observation(ProvenanceFactObservation),
    /// Opaque future fact with sanitized metadata only.
    Opaque(ProvenanceOpaqueFact),
}

/// Safe model-activity fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactModelActivity {
    /// Stable fact identity.
    pub fact_id: Text,
    /// BLAKE3 artifact hash.
    pub artifact_hash: Text,
    /// Optional requirement id.
    pub requirement_id: Option<Text>,
    /// Safe request-shape label.
    pub request_shape: Text,
    /// Safe actor label.
    pub actor_label: Text,
}

/// Safe tool-activity fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactToolActivity {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Safe tool kind.
    pub tool_kind: Text,
    /// BLAKE3 artifact hash.
    pub artifact_hash: Text,
    /// Effect ability label.
    pub effect_ability: Text,
    /// Effect resource label.
    pub effect_resource: Text,
}

/// Safe runtime-block fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactRuntimeBlock {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Safe block reason.
    pub block_reason: Text,
    /// Optional requirement id.
    pub requirement_id: Option<Text>,
}

/// Safe error fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactError {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Safe error class.
    pub error_class: Text,
    /// Optional requirement id.
    pub requirement_id: Option<Text>,
}

/// Safe artifact fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactArtifact {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Artifact kind.
    pub kind: ProvenanceArtifactKind,
    /// BLAKE3 artifact hash.
    pub artifact_hash: Text,
}

/// Safe authorization fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactAuthorization {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Authorization status.
    pub status: ProvenanceAuthorizationStatus,
    /// Effect ability label.
    pub effect_ability: Text,
    /// Effect resource label.
    pub effect_resource: Text,
}

/// Safe provenance-observation fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceFactObservation {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Optional requirement id.
    pub requirement_id: Option<Text>,
    /// Safe observation label.
    pub label: Text,
}

/// Sanitized key-value metadata field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceSafeField {
    /// Safe metadata key.
    pub key: Text,
    /// Safe metadata value.
    pub value: Text,
}

/// Opaque future fact containing sanitized metadata only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceOpaqueFact {
    /// Stable fact identity.
    pub fact_id: Text,
    /// Safe schema label.
    pub schema_label: Text,
    /// Sanitized safe metadata.
    pub metadata: Collection<ProvenanceSafeField>,
}

impl ProvenanceOpaqueFact {
    /// Creates an opaque fact from safe metadata only.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] if any key or value is unsafe.
    pub fn new(
        fact_id: impl Into<Text>,
        schema_label: impl Into<Text>,
        metadata: Collection<ProvenanceSafeField>,
    ) -> Result<Self, ProvenanceError> {
        let fact_id = validate_safe_text("fact_id", fact_id.into())?;
        let schema_label = validate_safe_text("schema_label", schema_label.into())?;
        for field in &metadata {
            validate_safe_text("metadata_key", field.key.clone())?;
            validate_safe_text("metadata_value", field.value.clone())?;
        }
        Ok(Self {
            fact_id,
            schema_label,
            metadata,
        })
    }
}

/// Query over safe provenance facts.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProvenanceFactQuery {
    /// Optional artifact-hash filter.
    pub artifact_hash: Option<Text>,
    /// Optional tool-kind filter.
    pub tool_kind: Option<Text>,
    /// Optional error-class filter.
    pub error_class: Option<Text>,
    /// Optional requirement-id filter.
    pub requirement_id: Option<Text>,
    /// Optional request-shape filter.
    pub request_shape: Option<Text>,
    /// Optional authorization-status filter.
    pub authorization_status: Option<ProvenanceAuthorizationStatus>,
    /// Optional effect-ability filter.
    pub effect_ability: Option<Text>,
    /// Optional effect-resource filter.
    pub effect_resource: Option<Text>,
}

/// Returns facts matching every supplied safe filter.
#[must_use]
pub fn query_provenance_facts(
    facts: &[ProvenanceFact],
    query: &ProvenanceFactQuery,
) -> Collection<ProvenanceFact> {
    facts
        .iter()
        .filter(|fact| fact_matches_query(fact, query))
        .cloned()
        .collect()
}
