use alloc::{string::String, vec::Vec};

use crate::{
    ChangeRequestBinding, ChangeRequestDraft, ChangeRequestRef, CheckFailureObservation,
    DiffObservation, Error, RepositoryRef, ReviewEffect, ReviewFinding, ServiceAvailability,
    ServiceFailureClass, ServiceKind, validate_binding,
};

// r[impl agent_harness_runtime.remora_adapters.ports]
// r[impl agent_harness_runtime.remora_adapters.binding]
/// Descriptor for one injected service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceDescriptor {
    /// Service family.
    pub kind: ServiceKind,
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

/// Explicit service catalog without discovery behavior.
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
    pub fn admit(&self, kind: ServiceKind) -> Result<&ServiceDescriptor, Error> {
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

/// Request to fetch one bounded change request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestFetchRequest {
    /// Exact request binding.
    pub binding: ChangeRequestBinding,
    /// Repository reference.
    pub repository: RepositoryRef,
    /// Provider-local change-request identity.
    pub change_request_identity: String,
    /// Bounds for returned diff and check metadata.
    pub bounds: crate::ReviewBounds,
}

/// Bounded change-request source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestFetchObservation {
    /// Exact echoed binding.
    pub binding: ChangeRequestBinding,
    /// Change-request reference.
    pub change_request: ChangeRequestRef,
    /// Bounded diff metadata.
    pub diff: DiffObservation,
    /// Bounded failed-check observations.
    pub ci_failures: Vec<CheckFailureObservation>,
}

/// Request for injected advisory review analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAnalysisRequest {
    /// Exact request binding.
    pub binding: ChangeRequestBinding,
    /// Reviewed change request.
    pub change_request: ChangeRequestRef,
    /// BLAKE3 identity of bounded review context.
    pub context_identity: String,
    /// Maximum returned findings.
    pub max_findings: u32,
}

/// Bounded advisory review observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewAnalysisObservation {
    /// Exact echoed binding.
    pub binding: ChangeRequestBinding,
    /// Advisory findings.
    pub findings: Vec<ReviewFinding>,
    /// BLAKE3 identity of bounded analysis output.
    pub output_identity: String,
}

/// Request to publish a prepared change request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestPublishRequest {
    /// Exact request binding.
    pub binding: ChangeRequestBinding,
    /// Repository reference.
    pub repository: RepositoryRef,
    /// Exact prepared commit identity.
    pub commit_identity: String,
    /// Validated draft.
    pub draft: ChangeRequestDraft,
}

/// Bounded publication observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeRequestPublishObservation {
    /// Exact echoed binding.
    pub binding: ChangeRequestBinding,
    /// Published change-request identity.
    pub change_request_identity: String,
    /// Exact published commit identity.
    pub commit_identity: String,
    /// BLAKE3 identity of provider evidence.
    pub evidence_identity: String,
}

/// Request to publish review effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPublishRequest {
    /// Exact request binding.
    pub binding: ChangeRequestBinding,
    /// Reviewed change request.
    pub change_request: ChangeRequestRef,
    /// Prepared review effects.
    pub effects: Vec<ReviewEffect>,
    /// Maximum publication effects in this request.
    pub max_effects: u32,
}

/// Bounded review publication observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewPublishObservation {
    /// Exact echoed binding.
    pub binding: ChangeRequestBinding,
    /// Number of published inline comments.
    pub inline_comment_count: u32,
    /// Whether a summary was published.
    pub summary_published: bool,
    /// BLAKE3 identity of provider evidence.
    pub evidence_identity: String,
}

/// Failure returned by an injected service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortFailure {
    /// Service family.
    pub service: ServiceKind,
    /// Stable failure class.
    pub class: ServiceFailureClass,
    /// Safe BLAKE3 evidence identity when available.
    pub evidence_identity: Option<String>,
}

/// Change-request source port.
pub trait ChangeRequestSourcePort {
    /// Fetches bounded metadata for one exact request.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn fetch(
        &mut self,
        request: &ChangeRequestFetchRequest,
    ) -> Result<ChangeRequestFetchObservation, PortFailure>;
}

/// Advisory review-analysis port.
pub trait ReviewAnalysisPort {
    /// Analyzes one bounded immutable review subject.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn analyze(
        &mut self,
        request: &ReviewAnalysisRequest,
    ) -> Result<ReviewAnalysisObservation, PortFailure>;
}

/// Change-request publication port.
pub trait ChangeRequestPublicationPort {
    /// Publishes one evidence-bound prepared draft.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn publish_change_request(
        &mut self,
        request: &ChangeRequestPublishRequest,
    ) -> Result<ChangeRequestPublishObservation, PortFailure>;
}

/// Review publication port.
pub trait ReviewPublicationPort {
    /// Publishes one bounded effect batch.
    ///
    /// # Errors
    ///
    /// Returns [`PortFailure`] when the injected service cannot complete the request.
    fn publish_review(
        &mut self,
        request: &ReviewPublishRequest,
    ) -> Result<ReviewPublishObservation, PortFailure>;
}

/// Validates one fetched observation against its request.
///
/// # Errors
///
/// Returns a typed error when identities, metadata, or bounds do not match.
pub fn validate_fetch_observation(
    request: &ChangeRequestFetchRequest,
    observation: &ChangeRequestFetchObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.repository.validate()?;
    request.bounds.validate()?;
    observation.change_request.validate()?;
    crate::validate_identity(
        "fetch_request.change_request_identity",
        &request.change_request_identity,
    )?;
    if request.binding.repository_identity() != request.repository.identity
        || request.repository.default_branch != observation.change_request.target_branch
        || request.change_request_identity != observation.change_request.identity
        || observation.diff.commit_identity != observation.change_request.commit_identity
    {
        return Err(Error::BindingMismatch);
    }
    validate_diff_observation(&observation.diff, request.bounds)?;
    validate_count(
        "fetch_observation.ci_failures",
        observation.ci_failures.len(),
        request.bounds.max_ci_failures,
    )?;
    for failure in &observation.ci_failures {
        failure.validate()?;
    }
    Ok(())
}

/// Validates one advisory analysis observation.
///
/// # Errors
///
/// Returns a typed error when identities, findings, or bounds are invalid.
pub fn validate_analysis_observation(
    request: &ReviewAnalysisRequest,
    observation: &ReviewAnalysisObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.change_request.validate()?;
    if request.max_findings == 0 {
        return Err(Error::InvalidBound("analysis.max_findings"));
    }
    crate::validate_blake3("analysis.context_identity", &request.context_identity)?;
    crate::validate_blake3("analysis.output_identity", &observation.output_identity)?;
    if crate::bounded_u32(observation.findings.len()) > request.max_findings {
        return Err(Error::BoundExceeded {
            field: "analysis.findings",
            actual: crate::bounded_u32(observation.findings.len()),
            maximum: request.max_findings,
        });
    }
    for finding in &observation.findings {
        finding.validate()?;
    }
    Ok(())
}

/// Validates one change-request publication observation.
///
/// # Errors
///
/// Returns a typed error when request and publication evidence do not match.
pub fn validate_change_request_publication(
    request: &ChangeRequestPublishRequest,
    observation: &ChangeRequestPublishObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.repository.validate()?;
    request.draft.validate()?;
    if request.binding.repository_identity() != request.repository.identity {
        return Err(Error::BindingMismatch);
    }
    crate::validate_identity("publication.commit_identity", &request.commit_identity)?;
    crate::validate_identity(
        "publication.change_request_identity",
        &observation.change_request_identity,
    )?;
    crate::validate_blake3(
        "publication.evidence_identity",
        &observation.evidence_identity,
    )?;
    if request.commit_identity != observation.commit_identity {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

/// Validates one review publication observation.
///
/// # Errors
///
/// Returns a typed error when request and publication evidence do not match.
pub fn validate_review_publication(
    request: &ReviewPublishRequest,
    observation: &ReviewPublishObservation,
) -> Result<(), Error> {
    validate_binding(&request.binding, &observation.binding)?;
    request.change_request.validate()?;
    if request.max_effects == 0 {
        return Err(Error::InvalidBound("review_publication.max_effects"));
    }
    validate_count(
        "review_publication.effects",
        request.effects.len(),
        request.max_effects,
    )?;
    crate::validate_blake3(
        "review_publication.evidence_identity",
        &observation.evidence_identity,
    )?;
    let mut inline_comment_count = 0_u32;
    let mut summary_published = false;
    for effect in &request.effects {
        validate_review_effect(effect, &request.change_request)?;
        match effect {
            ReviewEffect::PrepareInlineComment { .. } => {
                inline_comment_count = inline_comment_count.saturating_add(1);
            }
            ReviewEffect::PrepareSummary { .. } => summary_published = true,
            ReviewEffect::RequestAnalysis { .. }
            | ReviewEffect::RequestCiRepair { .. }
            | ReviewEffect::Escalate { .. } => {}
        }
    }
    if observation.inline_comment_count != inline_comment_count
        || observation.summary_published != summary_published
    {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

fn validate_diff_observation(
    diff: &DiffObservation,
    bounds: crate::ReviewBounds,
) -> Result<(), Error> {
    crate::validate_identity("diff.commit_identity", &diff.commit_identity)?;
    validate_count(
        "fetch_observation.files",
        diff.files.len(),
        bounds.max_files,
    )?;
    if diff.total_hunk_count > bounds.max_hunks {
        return Err(Error::BoundExceeded {
            field: "fetch_observation.hunks",
            actual: diff.total_hunk_count,
            maximum: bounds.max_hunks,
        });
    }
    let changed_lines = diff
        .total_added_line_count
        .saturating_add(diff.total_deleted_line_count);
    if changed_lines > bounds.max_changed_lines {
        return Err(Error::BoundExceeded {
            field: "fetch_observation.changed_lines",
            actual: changed_lines,
            maximum: bounds.max_changed_lines,
        });
    }
    let mut observed_hunks = 0_u32;
    let mut observed_added_lines = 0_u32;
    let mut observed_deleted_lines = 0_u32;
    for file in &diff.files {
        crate::validate_identity("diff_file.file_identity", &file.file_identity)?;
        observed_hunks = observed_hunks.saturating_add(file.hunk_count);
        observed_added_lines = observed_added_lines.saturating_add(file.added_line_count);
        observed_deleted_lines = observed_deleted_lines.saturating_add(file.deleted_line_count);
        for anchor in &file.inline_anchors {
            anchor.validate()?;
            if anchor.commit_identity != diff.commit_identity
                || anchor.file_identity != file.file_identity
            {
                return Err(Error::BindingMismatch);
            }
        }
    }
    if observed_hunks != diff.total_hunk_count
        || observed_added_lines != diff.total_added_line_count
        || observed_deleted_lines != diff.total_deleted_line_count
    {
        return Err(Error::BindingMismatch);
    }
    Ok(())
}

fn validate_review_effect(
    effect: &ReviewEffect,
    change_request: &ChangeRequestRef,
) -> Result<(), Error> {
    match effect {
        ReviewEffect::PrepareInlineComment { finding, anchor } => {
            finding.validate()?;
            anchor.validate()?;
            if anchor.commit_identity != change_request.commit_identity
                || finding.file_identity.as_ref() != Some(&anchor.file_identity)
                || finding.side != Some(anchor.side)
                || finding.line_range.is_none_or(|range| {
                    range.end != anchor.line
                        || range.start != anchor.start_line.unwrap_or(anchor.line)
                })
            {
                return Err(Error::BindingMismatch);
            }
        }
        ReviewEffect::PrepareSummary { finding_count } => {
            if *finding_count == 0 {
                return Err(Error::InvalidBound("review_effect.finding_count"));
            }
        }
        ReviewEffect::RequestAnalysis { .. }
        | ReviewEffect::RequestCiRepair { .. }
        | ReviewEffect::Escalate { .. } => {
            return Err(Error::InvalidField("review_publication.effect"));
        }
    }
    Ok(())
}

fn validate_count(field: &'static str, actual: usize, maximum: u32) -> Result<(), Error> {
    let actual = crate::bounded_u32(actual);
    if actual > maximum {
        return Err(Error::BoundExceeded {
            field,
            actual,
            maximum,
        });
    }
    Ok(())
}
