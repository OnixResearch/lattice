use super::data::{ContextReferenceError, ContextResolverKind};
use alloc::string::ToString;

const CONTEXT_ADAPTER_SCHEMA: &str = "animus.context-adapter.v1";
const CONTEXT_ROLE_MISMATCH_REASON: &str = "context adapter role does not match resolver kind";
const EMPTY_CONTEXT_FAMILY_REASON: &str = "context adapter family requires at least one role";
const DUPLICATE_CONTEXT_ROLE_REASON: &str = "context adapter family roles must be unique";
const MIN_ROLE_RECORD_COUNT: usize = 1;
const CONTEXT_ROLE_METADATA_LABEL_COUNT: usize = 7;
const CONTEXT_FAMILY_OPERATION_LABEL: &str = "context_adapter_family";

/// Explicit context adapter role kept separate from concrete host authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextAdapterRole {
    /// Local file context resolver role.
    File,
    /// Local directory listing resolver role.
    Directory,
    /// Version-control diff resolver role.
    Diff,
    /// Image context resolver role.
    Image,
    /// URL context resolver role.
    Url,
    /// Host-owned context-layer descriptor source role.
    ContextLayerSource,
    /// Host-owned external context handle resolver role.
    ExternalHandle,
}

impl ContextAdapterRole {
    /// Returns a stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Diff => "diff",
            Self::Image => "image",
            Self::Url => "url",
            Self::ContextLayerSource => "context_layer_source",
            Self::ExternalHandle => "external_handle",
        }
    }

    /// Returns the resolver kind when this role dispatches through a resolver trait.
    #[must_use]
    pub const fn resolver_kind(self) -> Option<ContextResolverKind> {
        match self {
            Self::File => Some(ContextResolverKind::File),
            Self::Directory => Some(ContextResolverKind::Directory),
            Self::Diff => Some(ContextResolverKind::Diff),
            Self::Image => Some(ContextResolverKind::Image),
            Self::Url => Some(ContextResolverKind::Url),
            Self::ContextLayerSource | Self::ExternalHandle => None,
        }
    }
}

/// Host-declared availability for a context role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextAdapterAvailability {
    /// The role is present and can be selected after policy admission.
    Available,
    /// No service was supplied for this role.
    Missing,
    /// The host intentionally does not support this role.
    Unsupported,
    /// The service exists but is unavailable for this request.
    Unavailable,
}

impl ContextAdapterAvailability {
    /// Returns a stable availability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Policy decision applied before context role dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextAdapterPolicyDecision {
    /// Policy admits the context role.
    Allowed,
    /// Policy denies the context role.
    Denied,
}

impl ContextAdapterPolicyDecision {
    /// Returns a stable policy decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
        }
    }
}

/// Safe outcome/failure class used by context adapter conformance evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextAdapterOutcomeClass {
    /// Dispatch completed with bounded output.
    Completed,
    /// Required service was missing.
    MissingService,
    /// Policy or capability denied dispatch.
    DeniedCapability,
    /// Service existed but was unavailable.
    UnavailableService,
    /// Host deadline expired.
    TimedOut,
    /// Host cancellation was observed.
    Cancelled,
    /// Resolver output was malformed.
    MalformedOutput,
    /// Resolver output exceeded policy bounds.
    OversizedOutput,
    /// Reference, path, URL, handle, or evidence was unsafe.
    UnsafeReference,
    /// Schema or role contract did not match.
    SchemaMismatch,
    /// Durable evidence leak was rejected.
    LeakRejected,
}

impl ContextAdapterOutcomeClass {
    /// Returns a stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::MissingService => "missing_service",
            Self::DeniedCapability => "denied_capability",
            Self::UnavailableService => "unavailable_service",
            Self::TimedOut => "timed_out",
            Self::Cancelled => "cancelled",
            Self::MalformedOutput => "malformed_output",
            Self::OversizedOutput => "oversized_output",
            Self::UnsafeReference => "unsafe_reference",
            Self::SchemaMismatch => "schema_mismatch",
            Self::LeakRejected => "leak_rejected",
        }
    }
}

/// Borrowed role metadata before safe evidence validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextAdapterRoleDraft<'a> {
    /// Independent context role.
    pub role: ContextAdapterRole,
    /// Host-declared role availability.
    pub availability: ContextAdapterAvailability,
    /// Policy decision before dispatch.
    pub policy_decision: ContextAdapterPolicyDecision,
    /// Capability or policy label for this role.
    pub capability_label: &'a str,
    /// Safe conformance evidence label.
    pub evidence_label: &'a str,
}

/// Validated independent context role record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextAdapterRoleRecord {
    role: ContextAdapterRole,
    availability: ContextAdapterAvailability,
    policy_decision: ContextAdapterPolicyDecision,
    capability_label: alloc::string::String,
    evidence_label: alloc::string::String,
}

impl ContextAdapterRoleRecord {
    /// Builds a role record and rejects unsafe labels before it becomes evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when any label contains unsafe metadata.
    pub fn new(draft: ContextAdapterRoleDraft<'_>) -> Result<Self, ContextReferenceError> {
        validate_role_metadata(draft)?;
        Ok(Self {
            role: draft.role,
            availability: draft.availability,
            policy_decision: draft.policy_decision,
            capability_label: draft.capability_label.to_string(),
            evidence_label: draft.evidence_label.to_string(),
        })
    }

    /// Returns the role.
    #[must_use]
    pub const fn role(&self) -> ContextAdapterRole {
        self.role
    }

    /// Returns availability.
    #[must_use]
    pub const fn availability(&self) -> ContextAdapterAvailability {
        self.availability
    }

    /// Returns policy decision.
    #[must_use]
    pub const fn policy_decision(&self) -> ContextAdapterPolicyDecision {
        self.policy_decision
    }

    /// Returns the safe capability label.
    #[must_use]
    pub fn capability_label(&self) -> &str {
        self.capability_label.as_str()
    }

    /// Returns the safe evidence label.
    #[must_use]
    pub fn evidence_label(&self) -> &str {
        self.evidence_label.as_str()
    }

    /// Returns true when dispatch is admitted for this role.
    #[must_use]
    pub const fn is_dispatch_admitted(&self) -> bool {
        matches!(self.availability, ContextAdapterAvailability::Available)
            && matches!(self.policy_decision, ContextAdapterPolicyDecision::Allowed)
    }
}

/// Validated context adapter family manifest over independent role records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    roles: alloc::vec::Vec<ContextAdapterRoleRecord>,
}

impl Manifest {
    /// Builds a family manifest and rejects empty or duplicate role records.
    ///
    /// # Errors
    ///
    /// Returns [`ContextReferenceError`] when records are empty, duplicated, or unsafe.
    pub fn new(records: &[ContextAdapterRoleRecord]) -> Result<Self, ContextReferenceError> {
        if records.len() < MIN_ROLE_RECORD_COUNT {
            return Err(ContextReferenceError::UnsupportedKind {
                reason: EMPTY_CONTEXT_FAMILY_REASON,
            });
        }
        reject_duplicate_roles(records)?;
        Ok(Self {
            roles: records.to_vec(),
        })
    }

    /// Returns all role records in deterministic insertion order.
    #[must_use]
    pub fn roles(&self) -> &[ContextAdapterRoleRecord] {
        self.roles.as_slice()
    }

    /// Returns the record for a role when present.
    #[must_use]
    pub fn role_record(&self, role: ContextAdapterRole) -> Option<&ContextAdapterRoleRecord> {
        self.roles.iter().find(|record| record.role == role)
    }

    /// Ensures a role is available and allowed before dispatch.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed context error for missing, denied, unsupported, or unavailable roles.
    pub fn require_admitted_role(
        &self,
        role: ContextAdapterRole,
    ) -> Result<&ContextAdapterRoleRecord, ContextReferenceError> {
        let record = self
            .role_record(role)
            .ok_or(ContextReferenceError::ResolverMissing)?;
        match (record.availability, record.policy_decision) {
            (ContextAdapterAvailability::Available, ContextAdapterPolicyDecision::Allowed) => {
                Ok(record)
            }
            (_, ContextAdapterPolicyDecision::Denied) => Err(ContextReferenceError::ResolverDenied),
            (ContextAdapterAvailability::Missing, _) => Err(ContextReferenceError::ResolverMissing),
            (ContextAdapterAvailability::Unsupported, _) => {
                Err(ContextReferenceError::UnsupportedKind {
                    reason: CONTEXT_ROLE_MISMATCH_REASON,
                })
            }
            (ContextAdapterAvailability::Unavailable, _) => {
                Err(ContextReferenceError::ResolverUnavailable)
            }
        }
    }
}

/// Validates that a resolver kind matches a role without granting ambient fallback authority.
///
/// # Errors
///
/// Returns [`ContextReferenceError`] when the role is not resolver-backed or the kind differs.
pub fn validate_context_resolver_role(
    role: ContextAdapterRole,
    resolver_kind: ContextResolverKind,
) -> Result<(), ContextReferenceError> {
    if role.resolver_kind() == Some(resolver_kind) {
        return Ok(());
    }
    Err(ContextReferenceError::UnsupportedKind {
        reason: CONTEXT_ROLE_MISMATCH_REASON,
    })
}

fn validate_role_metadata(draft: ContextAdapterRoleDraft<'_>) -> Result<(), ContextReferenceError> {
    let labels = [
        CONTEXT_ADAPTER_SCHEMA,
        CONTEXT_FAMILY_OPERATION_LABEL,
        draft.role.as_str(),
        draft.availability.as_str(),
        draft.policy_decision.as_str(),
        draft.capability_label,
        draft.evidence_label,
    ];
    debug_assert_eq!(labels.len(), CONTEXT_ROLE_METADATA_LABEL_COUNT);
    debug_assert!(labels.iter().all(|label| !label.is_empty()));
    crate::safe::receipt::Record::new(
        crate::safe::receipt::Draft::new(
            crate::safe::receipt::Status::Completed,
            crate::safe::receipt::IssueCode::ProvenanceObservation,
            crate::safe::receipt::Operation::ProvenanceObservation,
        )
        .with_labels(&labels)
        .with_redactions(&[crate::safe::receipt::Redaction::PayloadHashOnly]),
    )
    .map(|_record| ())
    .map_err(ContextReferenceError::SafeMetadata)
}

fn reject_duplicate_roles(
    records: &[ContextAdapterRoleRecord],
) -> Result<(), ContextReferenceError> {
    for (index, record) in records.iter().enumerate() {
        if records
            .iter()
            .skip(index.saturating_add(MIN_ROLE_RECORD_COUNT))
            .any(|candidate| candidate.role == record.role)
        {
            return Err(ContextReferenceError::UnsupportedKind {
                reason: DUPLICATE_CONTEXT_ROLE_REASON,
            });
        }
    }
    Ok(())
}
