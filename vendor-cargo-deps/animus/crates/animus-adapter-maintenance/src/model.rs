use alloc::{string::String, vec::Vec};

use animus_adapter_change_request::{
    MAX_IDENTITY_BYTES, PublicationMode, validate_binding as validate_change_request_binding,
};

use crate::{
    DEFAULT_MAX_CANDIDATES, DEFAULT_MAX_GROUP_SIZE, DEFAULT_MAX_GROUPS, DEFAULT_MAX_MANIFESTS,
    MAX_DEPENDENCY_TEXT_BYTES,
};

/// Dependency-maintenance adapter error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required field is empty, malformed, or over its bound.
    InvalidField(&'static str),
    /// A numeric bound is zero.
    InvalidBound(&'static str),
    /// A collection or count exceeds its bound.
    BoundExceeded {
        /// Bounded field name.
        field: &'static str,
        /// Observed count.
        actual: u32,
        /// Allowed count.
        maximum: u32,
    },
    /// Request and observation identities do not match.
    BindingMismatch,
    /// Required service is absent.
    MissingService(MaintenanceServiceKind),
    /// A declared service is not currently available.
    ServiceBlocked {
        /// Blocked service family.
        service: MaintenanceServiceKind,
        /// Exact declared availability.
        availability: ServiceAvailability,
    },
    /// Candidate grouping violated ecosystem or manifest compatibility.
    IncompatibleGroup,
    /// A receipt requested raw content disclosure.
    UnsafeReceipt,
    /// A receipt requested an unsupported claim.
    UnsupportedClaim,
}

/// Supported dependency ecosystems.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    /// Rust Cargo.
    Cargo,
    /// Nix flakes.
    Nix,
    /// Node package managers.
    Npm,
    /// Python package managers.
    Python,
    /// Go modules.
    Go,
    /// Unsupported or unknown ecosystem.
    Unknown,
}

/// Update classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateKind {
    /// Patch-level version update.
    Patch,
    /// Minor-level version update.
    Minor,
    /// Major-level version update.
    Major,
    /// Security remediation update.
    Security,
    /// Unknown update class.
    Unknown,
}

/// Update risk class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskLevel {
    /// Low risk.
    Low,
    /// Medium risk.
    Medium,
    /// High risk.
    High,
    /// Unknown risk.
    Unknown,
}

/// Dependency-maintenance service families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaintenanceServiceKind {
    /// Dependency inventory service.
    Inventory,
    /// Update-candidate discovery service.
    CandidateDiscovery,
    /// Security-advisory observation service.
    AdvisoryObservation,
    /// Workspace update service.
    WorkspaceUpdate,
    /// Trusted validation-profile service.
    Validation,
}

/// Service availability class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceAvailability {
    /// Service is available.
    Available,
    /// Policy denied the service.
    Denied,
    /// Service does not support the request.
    Unsupported,
    /// Service is absent.
    Unavailable,
    /// Service evidence is stale.
    Stale,
}

/// Service failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceFailureClass {
    /// Policy denied the request.
    Denied,
    /// Service is unavailable.
    Unavailable,
    /// Service does not support the request.
    Unsupported,
    /// Request was cancelled.
    Cancelled,
    /// Request timed out.
    TimedOut,
    /// Response was malformed.
    Malformed,
    /// Response was stale.
    Stale,
    /// Response binding did not match.
    Mismatch,
    /// Response exceeded a declared bound.
    Overflow,
    /// Host infrastructure failed.
    Infrastructure,
}

/// Exact binding for one maintenance adapter call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaintenanceBinding {
    repository_identity: String,
    policy_identity: String,
    plan_identity: String,
    generation: u64,
    implementation_identity: String,
    correlation_identity: String,
}

impl MaintenanceBinding {
    /// Creates and validates one exact binding.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when an identity is empty or invalid.
    pub fn new(
        repository_identity: String,
        policy_identity: String,
        plan_identity: String,
        generation: u64,
        implementation_identity: String,
        correlation_identity: String,
    ) -> Result<Self, Error> {
        validate_identity("binding.repository_identity", &repository_identity)?;
        validate_identity("binding.policy_identity", &policy_identity)?;
        validate_identity("binding.plan_identity", &plan_identity)?;
        validate_identity("binding.implementation_identity", &implementation_identity)?;
        validate_identity("binding.correlation_identity", &correlation_identity)?;
        Ok(Self {
            repository_identity,
            policy_identity,
            plan_identity,
            generation,
            implementation_identity,
            correlation_identity,
        })
    }

    /// Returns the repository identity.
    #[must_use]
    pub fn repository_identity(&self) -> &str {
        &self.repository_identity
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }

    /// Returns the plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        &self.plan_identity
    }

    /// Returns the caller-supplied generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the implementation identity.
    #[must_use]
    pub fn implementation_identity(&self) -> &str {
        &self.implementation_identity
    }

    /// Returns the correlation identity.
    #[must_use]
    pub fn correlation_identity(&self) -> &str {
        &self.correlation_identity
    }
}

/// One declared dependency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyDeclaration {
    /// Dependency name.
    pub name: String,
    /// Current declared or locked version.
    pub current_version: String,
}

impl DependencyDeclaration {
    /// Validates bounded dependency values.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when a dependency value is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_dependency_text("dependency.name", &self.name)?;
        validate_dependency_text("dependency.current_version", &self.current_version)
    }
}

/// Bounded manifest observation supplied by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestObservation {
    /// Stable manifest identity, not a physical path.
    pub manifest_identity: String,
    /// Package ecosystem.
    pub ecosystem: Ecosystem,
    /// Optional lockfile identity, not a physical path.
    pub lockfile_identity: Option<String>,
    /// Declared dependencies.
    pub dependencies: Vec<DependencyDeclaration>,
    /// BLAKE3 identity of source evidence.
    pub evidence_identity: String,
}

impl ManifestObservation {
    /// Validates bounded manifest metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when manifest metadata is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("manifest.manifest_identity", &self.manifest_identity)?;
        if self.ecosystem == Ecosystem::Unknown {
            return Err(Error::InvalidField("manifest.ecosystem"));
        }
        if let Some(lockfile_identity) = &self.lockfile_identity {
            validate_identity("manifest.lockfile_identity", lockfile_identity)?;
        }
        validate_blake3("manifest.evidence_identity", &self.evidence_identity)?;
        for dependency in &self.dependencies {
            dependency.validate()?;
        }
        Ok(())
    }
}

/// Security-advisory observation supplied by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryObservation {
    /// Stable advisory identity.
    pub advisory_identity: String,
    /// Candidate identity that the advisory applies to.
    pub candidate_identity: String,
    /// Fixed version selected by the host policy.
    pub fixed_version: String,
    /// BLAKE3 identity of advisory evidence.
    pub evidence_identity: String,
}

impl AdvisoryObservation {
    /// Validates advisory metadata without claiming database freshness.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when advisory metadata is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("advisory.advisory_identity", &self.advisory_identity)?;
        validate_identity("advisory.candidate_identity", &self.candidate_identity)?;
        validate_dependency_text("advisory.fixed_version", &self.fixed_version)?;
        validate_blake3("advisory.evidence_identity", &self.evidence_identity)
    }
}

/// One dependency update candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateCandidate {
    /// Stable candidate identity.
    pub candidate_identity: String,
    /// Dependency name.
    pub dependency_name: String,
    /// Ecosystem.
    pub ecosystem: Ecosystem,
    /// Stable manifest identity.
    pub manifest_identity: String,
    /// Current version.
    pub current_version: String,
    /// Target version.
    pub target_version: String,
    /// Update class.
    pub update_kind: UpdateKind,
    /// Risk class.
    pub risk: RiskLevel,
    /// Optional advisory evidence identity for security updates.
    pub advisory_identity: Option<String>,
}

impl UpdateCandidate {
    /// Validates bounded candidate metadata.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when candidate metadata is invalid.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity("candidate.candidate_identity", &self.candidate_identity)?;
        validate_dependency_text("candidate.dependency_name", &self.dependency_name)?;
        validate_identity("candidate.manifest_identity", &self.manifest_identity)?;
        validate_dependency_text("candidate.current_version", &self.current_version)?;
        validate_dependency_text("candidate.target_version", &self.target_version)?;
        if self.ecosystem == Ecosystem::Unknown || self.update_kind == UpdateKind::Unknown {
            return Err(Error::InvalidField("candidate.classification"));
        }
        match (&self.advisory_identity, self.update_kind) {
            (Some(identity), UpdateKind::Security) => {
                validate_identity("candidate.advisory_identity", identity)
            }
            (None, UpdateKind::Security) | (Some(_), _) => {
                Err(Error::InvalidField("candidate.advisory_identity"))
            }
            (None, _) => Ok(()),
        }
    }
}

/// Trusted validation profile reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationProfileRef {
    /// Safe profile identity.
    pub profile_identity: String,
}

impl ValidationProfileRef {
    /// Validates that the profile is a safe identity, not a shell command.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when the profile identity is unsafe.
    pub fn validate(&self) -> Result<(), Error> {
        validate_identity(
            "validation_profile.profile_identity",
            &self.profile_identity,
        )?;
        if !self.profile_identity.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        }) {
            return Err(Error::InvalidField("validation_profile.profile_identity"));
        }
        Ok(())
    }
}

/// Dependency update policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePolicy {
    /// Change-request publication mode.
    pub publication_mode: PublicationMode,
    /// Require advisory review for major updates.
    pub require_review_for_major: bool,
    /// Require security updates to remain isolated.
    pub isolate_security_updates: bool,
    /// Trusted validation profiles requested for each group.
    pub validation_profiles: Vec<ValidationProfileRef>,
}

impl UpdatePolicy {
    /// Validates policy shape and safety posture.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidField`] when policy values are unsafe.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.isolate_security_updates {
            return Err(Error::InvalidField(
                "update_policy.isolate_security_updates",
            ));
        }
        for profile in &self.validation_profiles {
            profile.validate()?;
        }
        Ok(())
    }
}

/// Bounds for one update plan.
#[allow(
    clippy::struct_field_names,
    reason = "the max_ prefix makes each public bound explicit at call sites"
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunBounds {
    /// Maximum manifests observed.
    pub max_manifests: u32,
    /// Maximum candidates considered.
    pub max_candidates: u32,
    /// Maximum routine updates in one group.
    pub max_group_size: u32,
    /// Maximum groups in one plan.
    pub max_groups: u32,
}

impl RunBounds {
    /// Returns conservative bounds.
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            max_manifests: DEFAULT_MAX_MANIFESTS,
            max_candidates: DEFAULT_MAX_CANDIDATES,
            max_group_size: DEFAULT_MAX_GROUP_SIZE,
            max_groups: DEFAULT_MAX_GROUPS,
        }
    }

    /// Validates all bounds.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidBound`] when any bound is zero.
    pub const fn validate(self) -> Result<(), Error> {
        if self.max_manifests == 0 {
            return Err(Error::InvalidBound("run_bounds.max_manifests"));
        }
        if self.max_candidates == 0 {
            return Err(Error::InvalidBound("run_bounds.max_candidates"));
        }
        if self.max_group_size == 0 {
            return Err(Error::InvalidBound("run_bounds.max_group_size"));
        }
        if self.max_groups == 0 {
            return Err(Error::InvalidBound("run_bounds.max_groups"));
        }
        Ok(())
    }
}

/// Validates one exact maintenance binding.
///
/// # Errors
///
/// Returns [`Error::BindingMismatch`] when any bound identity differs.
pub fn validate_binding(
    expected: &MaintenanceBinding,
    actual: &MaintenanceBinding,
) -> Result<(), Error> {
    if expected != actual {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

#[allow(dead_code)]
fn _change_request_binding_owner(
    expected: &animus_adapter_change_request::ChangeRequestBinding,
    actual: &animus_adapter_change_request::ChangeRequestBinding,
) -> Result<(), Error> {
    validate_change_request_binding(expected, actual).map_err(|_| Error::BindingMismatch)
}

pub(crate) fn validate_identity(field: &'static str, value: &str) -> Result<(), Error> {
    validate_text(field, value, MAX_IDENTITY_BYTES)
}

pub(crate) fn validate_dependency_text(field: &'static str, value: &str) -> Result<(), Error> {
    validate_text(field, value, MAX_DEPENDENCY_TEXT_BYTES)
}

fn validate_text(field: &'static str, value: &str, maximum_bytes: usize) -> Result<(), Error> {
    if value.is_empty() || value.trim() != value || value.len() > maximum_bytes {
        return Err(Error::InvalidField(field));
    }
    Ok(())
}

pub(crate) fn validate_blake3(field: &'static str, value: &str) -> Result<(), Error> {
    const BLAKE3_HEX_BYTES: usize = 64;
    let digest = value.strip_prefix("b3:").unwrap_or(value);
    if digest.len() != BLAKE3_HEX_BYTES
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(Error::InvalidField(field));
    }
    Ok(())
}

pub(crate) fn bounded_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
