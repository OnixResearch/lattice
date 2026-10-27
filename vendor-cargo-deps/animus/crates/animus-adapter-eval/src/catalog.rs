//! Service catalogs, capability ceilings, and pure admission.

use alloc::{format, string::String, vec::Vec};

use animus_eval_core::{hash_identity, validate_identity};

use crate::{MAX_SERVICE_CAPABILITIES, MAX_SERVICE_ENTRIES};

/// Maximum bounded items admitted for one service call.
pub const MAX_SERVICE_ITEMS: u32 = 65_536;
/// Maximum bounded bytes admitted for one service call.
pub const MAX_SERVICE_BYTES: u64 = 64 * 1_024 * 1_024;

/// One least-authority evaluation service.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ServiceKind {
    /// Bounded task materialization.
    TaskSource,
    /// Admitted agent execution.
    AgentExecution,
    /// Independently identified verifier execution.
    VerifierExecution,
    /// Artifact collection.
    ArtifactCollect,
    /// Artifact restore.
    ArtifactRestore,
    /// Bounded model-call metadata capture.
    ModelCallCapture,
    /// Append-only result recording.
    ResultAppend,
    /// Exact result replacement.
    ResultReplace,
    /// Existing result reopening.
    ResultReopen,
}

impl ServiceKind {
    /// Return the stable service label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TaskSource => "task_source",
            Self::AgentExecution => "agent_execution",
            Self::VerifierExecution => "verifier_execution",
            Self::ArtifactCollect => "artifact_collect",
            Self::ArtifactRestore => "artifact_restore",
            Self::ModelCallCapture => "model_call_capture",
            Self::ResultAppend => "result_append",
            Self::ResultReplace => "result_replace",
            Self::ResultReopen => "result_reopen",
        }
    }

    /// Return the only capability this port can claim.
    #[must_use]
    pub const fn capability(self) -> ServiceCapability {
        match self {
            Self::TaskSource => ServiceCapability::TaskSource,
            Self::AgentExecution => ServiceCapability::AgentExecution,
            Self::VerifierExecution => ServiceCapability::VerifierExecution,
            Self::ArtifactCollect => ServiceCapability::ArtifactCollect,
            Self::ArtifactRestore => ServiceCapability::ArtifactRestore,
            Self::ModelCallCapture => ServiceCapability::ModelCallCapture,
            Self::ResultAppend => ServiceCapability::ResultAppend,
            Self::ResultReplace => ServiceCapability::ResultReplace,
            Self::ResultReopen => ServiceCapability::ResultReopen,
        }
    }
}

/// Explicit service capability.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ServiceCapability {
    /// Task-source capability.
    TaskSource,
    /// Agent-execution capability.
    AgentExecution,
    /// Verifier-execution capability.
    VerifierExecution,
    /// Artifact-collection capability.
    ArtifactCollect,
    /// Artifact-restore capability.
    ArtifactRestore,
    /// Model-call-capture capability.
    ModelCallCapture,
    /// Result-append capability.
    ResultAppend,
    /// Result-replace capability.
    ResultReplace,
    /// Result-reopen capability.
    ResultReopen,
    /// Invalid ambient service discovery capability.
    AmbientDiscovery,
    /// Invalid provider transport capability.
    ProviderTransport,
    /// Invalid shared mutable state capability.
    SharedMutableState,
}

impl ServiceCapability {
    /// Return the stable capability label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TaskSource => "task_source",
            Self::AgentExecution => "agent_execution",
            Self::VerifierExecution => "verifier_execution",
            Self::ArtifactCollect => "artifact_collect",
            Self::ArtifactRestore => "artifact_restore",
            Self::ModelCallCapture => "model_call_capture",
            Self::ResultAppend => "result_append",
            Self::ResultReplace => "result_replace",
            Self::ResultReopen => "result_reopen",
            Self::AmbientDiscovery => "ambient_discovery",
            Self::ProviderTransport => "provider_transport",
            Self::SharedMutableState => "shared_mutable_state",
        }
    }
}

/// Service availability supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceAvailability {
    /// Service is currently available under its declared bounds.
    Available,
    /// Service is not configured.
    Absent,
    /// Policy denied the service.
    Denied,
    /// Implementation is temporarily unavailable.
    Unavailable,
    /// Implementation does not support the request cohort.
    Unsupported,
    /// Descriptor evidence is stale.
    Stale,
}

/// Hard resource ceiling for one service implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceResourceBounds {
    /// Maximum item count per call.
    pub max_items: u32,
    /// Maximum aggregate metadata or artifact bytes per call.
    pub max_bytes: u64,
}

/// Borrowed service descriptor supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceDescriptor<'a> {
    /// Service kind.
    pub kind: ServiceKind,
    /// Exact implementation identity.
    pub implementation_identity: &'a str,
    /// Explicit capabilities. Capabilities outside the port ceiling are invalid.
    pub capabilities: &'a [ServiceCapability],
    /// Current availability.
    pub availability: ServiceAvailability,
    /// Resource ceiling.
    pub bounds: ServiceResourceBounds,
    /// Exact service policy identity.
    pub policy_identity: &'a str,
}

/// Borrowed service catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceCatalog<'a> {
    /// Exact catalog identity supplied by the host.
    pub catalog_identity: &'a str,
    /// Catalog entries.
    pub services: &'a [ServiceDescriptor<'a>],
}

/// A validated service admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceAdmission {
    /// Canonical admission identity.
    pub(crate) identity: String,
    /// Admitted service kind.
    pub(crate) kind: ServiceKind,
    /// Exact implementation identity.
    pub(crate) implementation_identity: String,
    /// Admitted resource ceiling.
    pub(crate) bounds: ServiceResourceBounds,
    /// Exact policy identity.
    pub(crate) policy_identity: String,
}

impl ServiceAdmission {
    /// Return the canonical admission identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Return the admitted service kind.
    #[must_use]
    pub const fn kind(&self) -> ServiceKind {
        self.kind
    }

    /// Return the exact implementation identity.
    #[must_use]
    pub fn implementation_identity(&self) -> &str {
        &self.implementation_identity
    }

    /// Return the admitted resource ceiling.
    #[must_use]
    pub const fn bounds(&self) -> ServiceResourceBounds {
        self.bounds
    }

    /// Return the exact service policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }
}

/// Required service and resource facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceRequirement<'a> {
    /// Required service kind.
    pub kind: ServiceKind,
    /// Optional exact implementation identity selected by the plan.
    pub implementation_identity: Option<&'a str>,
    /// Required item capacity.
    pub required_items: u32,
    /// Required byte capacity.
    pub required_bytes: u64,
    /// Exact current policy identity.
    pub policy_identity: &'a str,
}

/// Fail-closed service admission error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionError {
    /// Catalog, implementation, or policy identity is malformed.
    Identity,
    /// Catalog is empty, duplicated, or exceeds its bound.
    Catalog,
    /// A service claims capability outside its port ceiling.
    CapabilityWidening,
    /// A required service is absent.
    Absent,
    /// Policy denied the service.
    Denied,
    /// Service is unavailable.
    Unavailable,
    /// Service does not support the request.
    Unsupported,
    /// Service evidence is stale.
    Stale,
    /// Resource bounds are zero, invalid, or insufficient.
    Bound,
    /// Selected implementation or policy does not match.
    Mismatch,
}

/// Validate a catalog without discovering any implementation.
///
/// # Errors
/// Returns a deterministic identity, catalog, capability, or bound class.
pub fn validate_service_catalog(catalog: &ServiceCatalog<'_>) -> Result<String, AdmissionError> {
    validate_identity(catalog.catalog_identity).map_err(|_| AdmissionError::Identity)?;
    if catalog.services.is_empty() || catalog.services.len() > MAX_SERVICE_ENTRIES {
        return Err(AdmissionError::Catalog);
    }
    let mut material = Vec::with_capacity(catalog.services.len());
    for (index, service) in catalog.services.iter().enumerate() {
        if catalog.services[..index]
            .iter()
            .any(|prior| prior.kind == service.kind)
        {
            return Err(AdmissionError::Catalog);
        }
        validate_descriptor(service)?;
        material.push(format!(
            "{}:{}:{}:{}",
            service.kind.label(),
            service.implementation_identity,
            service.bounds.max_items,
            service.bounds.max_bytes
        ));
    }
    let joined = material.join(",");
    Ok(hash_identity(
        "animus.native-evaluation.service-catalog.v1",
        &[catalog.catalog_identity, &joined],
    ))
}

/// Admit one exact service requirement from an explicit catalog.
///
/// # Errors
/// Returns a typed absence, availability, mismatch, capability, or bound class.
pub fn admit_service(
    catalog: &ServiceCatalog<'_>,
    requirement: &ServiceRequirement<'_>,
) -> Result<ServiceAdmission, AdmissionError> {
    let catalog_hash = validate_service_catalog(catalog)?;
    validate_identity(requirement.policy_identity).map_err(|_| AdmissionError::Identity)?;
    if requirement.required_items == 0
        || requirement.required_items > MAX_SERVICE_ITEMS
        || requirement.required_bytes == 0
        || requirement.required_bytes > MAX_SERVICE_BYTES
    {
        return Err(AdmissionError::Bound);
    }
    let service = catalog
        .services
        .iter()
        .find(|service| service.kind == requirement.kind)
        .ok_or(AdmissionError::Absent)?;
    match service.availability {
        ServiceAvailability::Available => {}
        ServiceAvailability::Absent => return Err(AdmissionError::Absent),
        ServiceAvailability::Denied => return Err(AdmissionError::Denied),
        ServiceAvailability::Unavailable => return Err(AdmissionError::Unavailable),
        ServiceAvailability::Unsupported => return Err(AdmissionError::Unsupported),
        ServiceAvailability::Stale => return Err(AdmissionError::Stale),
    }
    if requirement
        .implementation_identity
        .is_some_and(|identity| identity != service.implementation_identity)
        || requirement.policy_identity != service.policy_identity
    {
        return Err(AdmissionError::Mismatch);
    }
    if requirement.required_items > service.bounds.max_items
        || requirement.required_bytes > service.bounds.max_bytes
    {
        return Err(AdmissionError::Bound);
    }
    let items = format!("{}", requirement.required_items);
    let bytes = format!("{}", requirement.required_bytes);
    let identity = hash_identity(
        "animus.native-evaluation.service-admission.v1",
        &[
            &catalog_hash,
            service.kind.label(),
            service.implementation_identity,
            service.policy_identity,
            &items,
            &bytes,
        ],
    );
    Ok(ServiceAdmission {
        identity,
        kind: service.kind,
        implementation_identity: service.implementation_identity.into(),
        bounds: service.bounds,
        policy_identity: service.policy_identity.into(),
    })
}

fn validate_descriptor(service: &ServiceDescriptor<'_>) -> Result<(), AdmissionError> {
    validate_identity(service.implementation_identity).map_err(|_| AdmissionError::Identity)?;
    validate_identity(service.policy_identity).map_err(|_| AdmissionError::Identity)?;
    if service.capabilities.is_empty()
        || service.capabilities.len() > MAX_SERVICE_CAPABILITIES
        || service.bounds.max_items == 0
        || service.bounds.max_items > MAX_SERVICE_ITEMS
        || service.bounds.max_bytes == 0
        || service.bounds.max_bytes > MAX_SERVICE_BYTES
    {
        return Err(AdmissionError::Bound);
    }
    for (index, capability) in service.capabilities.iter().enumerate() {
        if *capability != service.kind.capability()
            || service.capabilities[..index].contains(capability)
        {
            return Err(AdmissionError::CapabilityWidening);
        }
    }
    Ok(())
}
