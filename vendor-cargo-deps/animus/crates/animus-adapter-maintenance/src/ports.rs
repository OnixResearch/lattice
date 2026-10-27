use alloc::{string::String, vec::Vec};

use crate::{
    AdvisoryObservation, Error, MaintenanceBinding, MaintenanceServiceKind, ManifestObservation,
    ServiceAvailability, ServiceFailureClass, UpdateCandidate, UpdateGroup, ValidationProfileRef,
    bounded_u32, validate_binding,
};

// r[impl agent_harness_runtime.remora_adapters.ports]
// r[impl agent_harness_runtime.remora_adapters.binding]
/// Descriptor for one injected maintenance service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceDescriptor {
    /// Service family.
    pub kind: MaintenanceServiceKind,
    /// Stable implementation identity.
    pub implementation_identity: String,
    /// Current availability.
    pub availability: ServiceAvailability,
}

impl ServiceDescriptor {
    /// Validates the descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when the implementation identity is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        crate::validate_identity(
            "service_descriptor.implementation_identity",
            &self.implementation_identity,
        )
    }
}

/// Explicit maintenance service catalog without discovery behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceCatalog {
    /// Caller-supplied service descriptors.
    pub services: Vec<ServiceDescriptor>,
}

impl ServiceCatalog {
    /// Admits one exact available service.
    ///
    /// # Errors
    ///
    /// Returns a typed error when the service is missing, unavailable, or invalid.
    pub fn admit(&self, kind: MaintenanceServiceKind) -> Result<&ServiceDescriptor, Error> {
        let descriptor = self
            .services
            .iter()
            .find(|descriptor| descriptor.kind == kind)
            .ok_or(Error::MissingService(kind))?;
        descriptor.validate()?;
        if descriptor.availability != ServiceAvailability::Available {
            return Err(Error::ServiceBlocked {
                service: kind,
                availability: descriptor.availability,
            });
        }
        Ok(descriptor)
    }
}

/// Request for bounded dependency inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryRequest {
    /// Exact request binding.
    pub binding: MaintenanceBinding,
    /// Maximum returned manifests.
    pub max_manifests: u32,
    /// Maximum dependencies in each manifest.
    pub max_dependencies_per_manifest: u32,
}

/// Bounded dependency inventory observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InventoryObservation {
    /// Exact echoed binding.
    pub binding: MaintenanceBinding,
    /// Manifest observations.
    pub manifests: Vec<ManifestObservation>,
    /// BLAKE3 identity of inventory evidence.
    pub evidence_identity: String,
}

/// Request for bounded update-candidate discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDiscoveryRequest {
    /// Exact request binding.
    pub binding: MaintenanceBinding,
    /// BLAKE3 inventory evidence identity.
    pub inventory_identity: String,
    /// Maximum returned candidates.
    pub max_candidates: u32,
}

/// Bounded update-candidate observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDiscoveryObservation {
    /// Exact echoed binding.
    pub binding: MaintenanceBinding,
    /// Update candidates.
    pub candidates: Vec<UpdateCandidate>,
    /// BLAKE3 identity of candidate evidence.
    pub evidence_identity: String,
}

/// Request for one security-advisory observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryRequest {
    /// Exact request binding.
    pub binding: MaintenanceBinding,
    /// Candidate identity.
    pub candidate_identity: String,
}

/// Bounded security-advisory result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryResult {
    /// Exact echoed binding.
    pub binding: MaintenanceBinding,
    /// Advisory observation.
    pub advisory: AdvisoryObservation,
}

/// Request for an injected workspace update.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceUpdateRequest {
    /// Exact request binding.
    pub binding: MaintenanceBinding,
    /// Exact update group.
    pub group: UpdateGroup,
    /// Expected workspace subject identity.
    pub workspace_subject_identity: String,
}

/// Bounded workspace update observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceUpdateObservation {
    /// Exact echoed binding.
    pub binding: MaintenanceBinding,
    /// Exact group identity.
    pub group_identity: String,
    /// Resulting workspace subject identity.
    pub workspace_subject_identity: String,
    /// BLAKE3 identity of bounded mutation evidence.
    pub evidence_identity: String,
}

/// Request to execute one trusted validation profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationRequest {
    /// Exact request binding.
    pub binding: MaintenanceBinding,
    /// Exact group identity.
    pub group_identity: String,
    /// Trusted validation profile.
    pub profile: ValidationProfileRef,
    /// Exact workspace subject identity.
    pub workspace_subject_identity: String,
}

/// Validation status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationStatus {
    /// Validation passed.
    Passed,
    /// Validation failed.
    Failed,
    /// Validation was blocked.
    Blocked,
}

/// Bounded validation observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationObservation {
    /// Exact echoed binding.
    pub binding: MaintenanceBinding,
    /// Exact group identity.
    pub group_identity: String,
    /// Trusted validation profile identity.
    pub profile_identity: String,
    /// Exact workspace subject identity.
    pub workspace_subject_identity: String,
    /// Validation status.
    pub status: ValidationStatus,
    /// BLAKE3 identity of bounded validation evidence.
    pub evidence_identity: String,
}

/// Failure returned by an injected maintenance service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortFailure {
    /// Service family.
    pub service: MaintenanceServiceKind,
    /// Stable failure class.
    pub class: ServiceFailureClass,
    /// Safe BLAKE3 evidence identity when available.
    pub evidence_identity: Option<String>,
}

/// Dependency inventory port.
pub trait DependencyInventoryPort {
    /// Observes bounded dependency manifests.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn inventory(
        &mut self,
        request: &InventoryRequest,
    ) -> Result<InventoryObservation, PortFailure>;
}

/// Update-candidate discovery port.
pub trait UpdateCandidatePort {
    /// Discovers bounded update candidates from admitted inventory evidence.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn discover_candidates(
        &mut self,
        request: &CandidateDiscoveryRequest,
    ) -> Result<CandidateDiscoveryObservation, PortFailure>;
}

/// Security-advisory observation port.
pub trait SecurityAdvisoryPort {
    /// Observes one advisory result without claiming database completeness.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn observe_advisory(
        &mut self,
        request: &AdvisoryRequest,
    ) -> Result<AdvisoryResult, PortFailure>;
}

/// Workspace update port.
pub trait WorkspaceUpdatePort {
    /// Applies one already-admitted update group.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn update_workspace(
        &mut self,
        request: &WorkspaceUpdateRequest,
    ) -> Result<WorkspaceUpdateObservation, PortFailure>;
}

/// Trusted validation-profile port.
pub trait ValidationPort {
    /// Executes one already-admitted validation profile.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn validate(
        &mut self,
        request: &ValidationRequest,
    ) -> Result<ValidationObservation, PortFailure>;
}

/// Validates one inventory observation.
///
/// # Errors
///
/// Returns a typed error when bindings, evidence, manifests, or bounds are invalid.
pub fn validate_inventory_observation(
    request: &InventoryRequest,
    observation: &InventoryObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    if request.max_manifests == 0 {
        return Err(Error::InvalidBound("inventory.max_manifests"));
    }
    if request.max_dependencies_per_manifest == 0 {
        return Err(Error::InvalidBound(
            "inventory.max_dependencies_per_manifest",
        ));
    }
    crate::validate_blake3(
        "inventory.evidence_identity",
        &observation.evidence_identity,
    )?;
    if bounded_u32(observation.manifests.len()) > request.max_manifests {
        return Err(Error::BoundExceeded {
            field: "inventory.manifests",
            actual: bounded_u32(observation.manifests.len()),
            maximum: request.max_manifests,
        });
    }
    for manifest in &observation.manifests {
        manifest.validate()?;
        if bounded_u32(manifest.dependencies.len()) > request.max_dependencies_per_manifest {
            return Err(Error::BoundExceeded {
                field: "inventory.dependencies",
                actual: bounded_u32(manifest.dependencies.len()),
                maximum: request.max_dependencies_per_manifest,
            });
        }
    }
    Ok(())
}

/// Validates one candidate-discovery observation.
///
/// # Errors
///
/// Returns a typed error when bindings, evidence, candidates, or bounds are invalid.
pub fn validate_candidate_observation(
    request: &CandidateDiscoveryRequest,
    observation: &CandidateDiscoveryObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    if request.max_candidates == 0 {
        return Err(Error::InvalidBound("candidate.max_candidates"));
    }
    crate::validate_blake3("candidate.inventory_identity", &request.inventory_identity)?;
    crate::validate_blake3(
        "candidate.evidence_identity",
        &observation.evidence_identity,
    )?;
    if bounded_u32(observation.candidates.len()) > request.max_candidates {
        return Err(Error::BoundExceeded {
            field: "candidate.candidates",
            actual: bounded_u32(observation.candidates.len()),
            maximum: request.max_candidates,
        });
    }
    for candidate in &observation.candidates {
        candidate.validate()?;
    }
    Ok(())
}

/// Validates one advisory observation.
///
/// # Errors
///
/// Returns a typed error when request and advisory evidence do not match.
pub fn validate_advisory_result(
    request: &AdvisoryRequest,
    result: &AdvisoryResult,
) -> Result<(), Error> {
    validate_binding(&request.binding, &result.binding)?;
    crate::validate_identity("advisory.candidate_identity", &request.candidate_identity)?;
    result.advisory.validate()?;
    if request.candidate_identity != result.advisory.candidate_identity {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

/// Validates one workspace update observation.
///
/// # Errors
///
/// Returns a typed error when group, subject, binding, or evidence does not match.
pub fn validate_workspace_observation(
    request: &WorkspaceUpdateRequest,
    observation: &WorkspaceUpdateObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.group.validate()?;
    crate::validate_identity(
        "workspace.workspace_subject_identity",
        &request.workspace_subject_identity,
    )?;
    crate::validate_identity(
        "workspace.result_subject_identity",
        &observation.workspace_subject_identity,
    )?;
    crate::validate_blake3(
        "workspace.evidence_identity",
        &observation.evidence_identity,
    )?;
    if request.group.group_identity != observation.group_identity
        || request.workspace_subject_identity == observation.workspace_subject_identity
    {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

/// Validates one trusted validation observation.
///
/// # Errors
///
/// Returns a typed error when group, profile, subject, binding, or evidence does not match.
pub fn validate_validation_observation(
    request: &ValidationRequest,
    observation: &ValidationObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.profile.validate()?;
    crate::validate_blake3("validation.group_identity", &request.group_identity)?;
    crate::validate_identity(
        "validation.workspace_subject_identity",
        &request.workspace_subject_identity,
    )?;
    crate::validate_blake3(
        "validation.evidence_identity",
        &observation.evidence_identity,
    )?;
    if request.group_identity != observation.group_identity
        || request.profile.profile_identity != observation.profile_identity
        || request.workspace_subject_identity != observation.workspace_subject_identity
    {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}
