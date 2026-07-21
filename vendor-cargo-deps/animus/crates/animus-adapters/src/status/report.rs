const STATUS_SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("status_source_id");
const STATUS_TARGET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("status_target_id");
const AGENT_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("agent_label");
const STATUS_MESSAGE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("status_message");

/// Validated semantic status observation supplied by a host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    kind: super::lifecycle::ObservationKind,
    state: super::lifecycle::SemanticAgent,
    source_id: alloc::string::String,
    target_id: alloc::string::String,
    agent_label: alloc::string::String,
    session: Option<super::identity::Reference>,
    message: Option<alloc::string::String>,
    presentation: super::view::Presentation,
    sequence: super::lifecycle::Sequence,
    ttl: Option<super::lifecycle::Ttl>,
    evidence_hashes: alloc::vec::Vec<alloc::string::String>,
}

impl Observation {
    /// Builds a validated status observation from borrowed boundary fields.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when any field is malformed,
    /// unsupported, too long, or contains unsafe NUL characters.
    pub fn new(draft: ObservationDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let kind = super::lifecycle::ObservationKind::parse(draft.kind)?;
        let state = super::lifecycle::SemanticAgent::parse(draft.state)?;
        crate::validation::validate_required_adapter_field(STATUS_SOURCE_FIELD, draft.source_id)?;
        crate::validation::validate_required_adapter_field(STATUS_TARGET_FIELD, draft.target_id)?;
        crate::validation::validate_required_adapter_field(AGENT_LABEL_FIELD, draft.agent_label)?;
        let session = draft
            .session
            .map(super::identity::Reference::new)
            .transpose()?;
        let message = super::view::optional_field(STATUS_MESSAGE_FIELD, draft.message)?;
        let presentation = super::view::Presentation::new(draft.presentation)?;
        let evidence_hashes = super::evidence::collect_hashes(draft.evidence_hashes)?;

        Ok(Self {
            kind,
            state,
            source_id: alloc::string::String::from(draft.source_id),
            target_id: alloc::string::String::from(draft.target_id),
            agent_label: alloc::string::String::from(draft.agent_label),
            session,
            message,
            presentation,
            sequence: draft.sequence,
            ttl: draft.ttl,
            evidence_hashes,
        })
    }

    /// Returns the observation authority kind.
    #[must_use]
    pub const fn kind(&self) -> super::lifecycle::ObservationKind {
        self.kind
    }

    /// Returns the semantic state reported by the observation.
    #[must_use]
    pub const fn state(&self) -> super::lifecycle::SemanticAgent {
        self.state
    }

    /// Returns the source identity.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.source_id.as_str()
    }

    /// Returns the target identity.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.target_id.as_str()
    }

    /// Returns the display-independent agent label.
    #[must_use]
    pub fn agent_label(&self) -> &str {
        self.agent_label.as_str()
    }

    /// Returns the optional host-native session reference.
    #[must_use]
    pub const fn session(&self) -> Option<&super::identity::Reference> {
        self.session.as_ref()
    }

    /// Returns the optional bounded status message.
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Returns presentation-only metadata.
    #[must_use]
    pub const fn presentation(&self) -> &super::view::Presentation {
        &self.presentation
    }

    /// Returns the logical sequence.
    #[must_use]
    pub const fn sequence(&self) -> super::lifecycle::Sequence {
        self.sequence
    }

    /// Returns the optional TTL.
    #[must_use]
    pub const fn ttl(&self) -> Option<super::lifecycle::Ttl> {
        self.ttl
    }

    /// Returns safe evidence hashes associated with this observation.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[alloc::string::String] {
        self.evidence_hashes.as_slice()
    }

    /// Returns whether this observation is current at `current_sequence`.
    #[must_use]
    pub const fn is_current_at(&self, current_sequence: super::lifecycle::Sequence) -> bool {
        match self.ttl {
            Some(ttl) => ttl.is_current_at(current_sequence),
            None => true,
        }
    }
}

/// Borrowed status observation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObservationDraft<'a> {
    /// Observation authority kind string.
    pub kind: &'a str,
    /// Semantic state string.
    pub state: &'a str,
    /// Stable source identifier.
    pub source_id: &'a str,
    /// Stable target identifier.
    pub target_id: &'a str,
    /// Agent label used to identify the actor without presentation overrides.
    pub agent_label: &'a str,
    /// Optional native session reference.
    pub session: Option<super::identity::ReferenceDraft<'a>>,
    /// Optional bounded lifecycle message.
    pub message: Option<&'a str>,
    /// Presentation-only metadata.
    pub presentation: super::view::PresentationDraft<'a>,
    /// Logical observation sequence.
    pub sequence: super::lifecycle::Sequence,
    /// Optional logical TTL.
    pub ttl: Option<super::lifecycle::Ttl>,
    /// Safe BLAKE3 evidence hashes; raw evidence is intentionally absent.
    pub evidence_hashes: &'a [&'a str],
}

/// Request sent to a host status-observation adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostRequest {
    target_id: alloc::string::String,
}

impl HostRequest {
    /// Builds a host status request for one target.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the target is empty, too
    /// long, or contains a NUL character.
    pub fn new(target_id: &str) -> Result<Self, crate::AdapterContractError> {
        crate::validation::validate_required_adapter_field(STATUS_TARGET_FIELD, target_id)?;
        Ok(Self {
            target_id: alloc::string::String::from(target_id),
        })
    }

    /// Returns the target identity.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.target_id.as_str()
    }
}

/// Batch returned by a host status-observation adapter.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostObservationBatch {
    observations: alloc::vec::Vec<Observation>,
}

impl HostObservationBatch {
    /// Builds a batch from already validated observations.
    #[must_use]
    pub fn new(observations: alloc::vec::Vec<Observation>) -> Self {
        Self { observations }
    }

    /// Returns validated observations.
    #[must_use]
    pub fn observations(&self) -> &[Observation] {
        self.observations.as_slice()
    }

    /// Consumes the batch and returns validated observations.
    #[must_use]
    pub fn into_observations(self) -> alloc::vec::Vec<Observation> {
        self.observations
    }
}
