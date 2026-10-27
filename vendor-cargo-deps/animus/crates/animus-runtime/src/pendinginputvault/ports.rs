//! Injected host service ports and thin orchestration.

// Method documentation defines the compact service DTO fields and outcomes.
#![allow(missing_docs)]

/// Request to load one explicit conversation boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadRequest {
    pub boundary: super::data::Boundary,
}

/// Body loaded only after current admission selects one restored reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedBody {
    pub entry: super::data::EntryIdentity,
    pub body_identity: super::data::BodyIdentity,
    pub body: String,
}

/// Cleanup observation for exact completed evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupObservation {
    Complete,
    Failed,
}

/// Observation for one recovery-only control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryControlObservation {
    Complete,
    Failed,
}

/// Stable service failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceIssue {
    Unavailable,
    LoadFailed,
    CommitFailed,
    CleanupFailed,
    BodyFailed,
    RecoveryControlFailed,
}

/// Explicit host-owned storage and transaction service.
pub trait Service {
    /// Loads supplied durable facts for one exact boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue without raw storage diagnostics.
    fn load(&mut self, request: &LoadRequest) -> Result<super::data::StartupFacts, ServiceIssue>;

    /// Commits disposition evidence before cleanup. A committed restore must
    /// retain each selected body under its admission reference.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue without claiming commit status.
    fn commit(
        &mut self,
        plan: &super::data::DispositionPlan,
    ) -> Result<super::data::CommitObservation, ServiceIssue>;

    /// Removes obsolete vault bodies or indexes after completed evidence
    /// exists. It must not remove material retained for restore admission.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue and leaves cleanup required.
    fn cleanup(
        &mut self,
        completed: &super::data::CompletedDisposition,
    ) -> Result<CleanupObservation, ServiceIssue>;

    /// Loads one admitted body reference. Startup recovery never calls this method.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue without exposing another entry.
    fn load_body(
        &mut self,
        request: &super::data::AdmissionRequest,
    ) -> Result<LoadedBody, ServiceIssue>;

    /// Performs one separately authorized recovery control.
    ///
    /// # Errors
    ///
    /// Returns a typed service issue without granting ordinary ingress.
    fn recover(
        &mut self,
        plan: &super::data::RecoveryControlPlan,
    ) -> Result<RecoveryControlObservation, ServiceIssue>;
}

/// Loads and projects one explicit startup boundary.
///
/// # Errors
///
/// Returns a bounded service or pure-validation error.
pub fn load_startup(
    service: &mut dyn Service,
    policy: &super::data::Policy,
    boundary: &super::data::Boundary,
) -> Result<super::data::Projection, super::data::Error> {
    super::core::validate_policy(policy)?;
    super::core::validate_boundary(boundary)?;
    let facts = service
        .load(&LoadRequest {
            boundary: boundary.clone(),
        })
        .map_err(map_service_issue)?;
    if facts.boundary != *boundary {
        return Err(super::data::Error::new(super::data::Issue::Stale));
    }
    super::core::project_startup(policy, &facts)
}

/// Commits one exact plan and reduces the returned observation.
///
/// # Errors
///
/// Returns a bounded service, stale-evidence, or reduction error.
pub fn commit_plan(
    service: &mut dyn Service,
    projection: &super::data::Projection,
    plan: &super::data::DispositionPlan,
) -> Result<super::data::CommitOutcome, super::data::Error> {
    super::core::validate_disposition_plan(projection, plan)?;
    let observation = service.commit(plan).map_err(map_service_issue)?;
    super::core::reduce_commit(projection, plan, &observation)
}

/// Attempts cleanup after disposition evidence is complete.
///
/// # Errors
///
/// Returns a bounded service error and does not change prior evidence.
pub fn cleanup_completed(
    service: &mut dyn Service,
    completed: &super::data::CompletedDisposition,
) -> Result<super::data::CompletedDisposition, super::data::Error> {
    super::core::validate_completed(completed)?;
    let observation = service.cleanup(completed).map_err(map_service_issue)?;
    let mut updated = completed.clone();
    updated.cleanup = match observation {
        CleanupObservation::Complete => super::data::CleanupState::Complete,
        CleanupObservation::Failed => super::data::CleanupState::Failed,
    };
    Ok(updated)
}

/// Loads the exact body only after a restored reference passes current admission.
///
/// # Errors
///
/// Returns stale or a bounded service failure without body substitution.
pub fn load_admitted_body(
    service: &mut dyn Service,
    request: &super::data::AdmissionRequest,
) -> Result<LoadedBody, super::data::Error> {
    let loaded = service.load_body(request).map_err(map_service_issue)?;
    if loaded.entry != request.entry || loaded.body_identity != request.body_identity {
        return Err(super::data::Error::new(super::data::Issue::Stale));
    }
    Ok(loaded)
}

/// Executes one recovery-only plan while ordinary ingress remains blocked.
///
/// # Errors
///
/// Returns a bounded service error.
pub fn execute_recovery_control(
    service: &mut dyn Service,
    boundary: &super::data::Boundary,
    plan: &super::data::RecoveryControlPlan,
) -> Result<RecoveryControlObservation, super::data::Error> {
    super::core::validate_boundary(boundary)?;
    super::core::validate_recovery_plan(plan)?;
    if plan.boundary != *boundary {
        return Err(super::data::Error::new(super::data::Issue::Stale));
    }
    service.recover(plan).map_err(map_service_issue)
}

fn map_service_issue(issue: ServiceIssue) -> super::data::Error {
    match issue {
        ServiceIssue::Unavailable => {
            super::data::Error::new(super::data::Issue::ServiceUnavailable)
        }
        ServiceIssue::LoadFailed
        | ServiceIssue::CommitFailed
        | ServiceIssue::CleanupFailed
        | ServiceIssue::BodyFailed
        | ServiceIssue::RecoveryControlFailed => {
            super::data::Error::new(super::data::Issue::ServiceFailure)
        }
    }
}
