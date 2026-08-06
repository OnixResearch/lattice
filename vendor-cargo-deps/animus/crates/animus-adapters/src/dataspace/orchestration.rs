const EVENT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_event_id");
const EVENT_KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("dataspace_event_kind");
const EVENT_TARGET_OWNER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("dataspace_event_target_owner");

const EVENT_KIND_PUBLISHED: &str = "published";
const EVENT_KIND_CANCELLED: &str = "cancelled";
const EVENT_KIND_EXPIRED: &str = "expired";
const EVENT_KIND_OWNERSHIP_TRANSFERRED: &str = "ownership_transferred";

/// Host orchestration event kinds accepted by a dataspace adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventKind {
    /// Host observed or acknowledged a publish operation.
    Published,
    /// Host cancelled the scoped actor or task.
    Cancelled,
    /// Host marked the scoped actor or task expired.
    Expired,
    /// Host transferred ownership to another actor or facet.
    OwnershipTransferred,
}

impl EventKind {
    /// Parses a stable orchestration-event-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] for empty, malformed, or unknown values.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        super::parse_value(EVENT_KIND_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(normalized: &str) -> Option<Self> {
        match normalized {
            EVENT_KIND_PUBLISHED => Some(Self::Published),
            EVENT_KIND_CANCELLED => Some(Self::Cancelled),
            EVENT_KIND_EXPIRED => Some(Self::Expired),
            EVENT_KIND_OWNERSHIP_TRANSFERRED => Some(Self::OwnershipTransferred),
            _ => None,
        }
    }

    /// Returns the stable orchestration-event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Published => EVENT_KIND_PUBLISHED,
            Self::Cancelled => EVENT_KIND_CANCELLED,
            Self::Expired => EVENT_KIND_EXPIRED,
            Self::OwnershipTransferred => EVENT_KIND_OWNERSHIP_TRANSFERRED,
        }
    }
}

/// Host orchestration event delivered to an optional dataspace adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    stable_id: alloc::string::String,
    kind: EventKind,
    scope: super::Scope,
    target_owner: Option<alloc::string::String>,
}

impl Event {
    /// Validates a host orchestration event for dataspace processing.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when labels are empty, malformed,
    /// too long, unsupported, or an ownership transfer omits its target owner.
    pub fn new(draft: Draft<'_>) -> Result<Self, crate::AdapterContractError> {
        let stable_id = crate::validation::normalized_adapter_field(EVENT_ID_FIELD, draft.stable_id)?;
        let kind = EventKind::parse(draft.kind)?;
        let target_owner = super::optional_field(EVENT_TARGET_OWNER_FIELD, draft.target_owner)?;
        if kind == EventKind::OwnershipTransferred && target_owner.is_none() {
            return Err(crate::AdapterContractError::EmptyField {
                field: EVENT_TARGET_OWNER_FIELD.as_str(),
            });
        }

        Ok(Self {
            stable_id,
            kind,
            scope: draft.scope,
            target_owner,
        })
    }

    /// Returns the stable event identity.
    #[must_use]
    pub fn stable_id(&self) -> &str {
        self.stable_id.as_str()
    }

    /// Returns the orchestration event kind.
    #[must_use]
    pub const fn kind(&self) -> EventKind {
        self.kind
    }

    /// Returns the scope affected by the event.
    #[must_use]
    pub const fn scope(&self) -> &super::Scope {
        &self.scope
    }

    /// Returns the target owner for ownership-transfer events.
    #[must_use]
    pub fn target_owner(&self) -> Option<&str> {
        self.target_owner.as_deref()
    }
}

/// Borrowed input used to validate a host orchestration event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Draft<'a> {
    /// Stable event identity.
    pub stable_id: &'a str,
    /// Event kind label.
    pub kind: &'a str,
    /// Affected scope.
    pub scope: super::Scope,
    /// Optional target owner required by ownership transfers.
    pub target_owner: Option<&'a str>,
}
