use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use animus_adapter_change_request::{ChangeRequestDraft, PublicationMode};
use blake3::Hasher;

use crate::{
    Ecosystem, Error, MaintenanceBinding, RiskLevel, RunBounds, UpdateCandidate, UpdateKind,
    UpdatePolicy, ValidationProfileRef, bounded_u32,
};

const GROUP_IDENTITY_DOMAIN: &str = "animus.maintenance-adapter.group.v1";
const RISK_RANK_LOW: u8 = 1;
const RISK_RANK_MEDIUM: u8 = 2;
const RISK_RANK_HIGH: u8 = 3;
const RISK_RANK_UNKNOWN: u8 = 4;

/// Input for one pure dependency update plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePlanInput {
    /// Exact plan binding.
    pub binding: MaintenanceBinding,
    /// Update candidates supplied by admitted host services.
    pub candidates: Vec<UpdateCandidate>,
    /// Update policy.
    pub policy: UpdatePolicy,
    /// Run bounds.
    pub bounds: RunBounds,
}

/// One deterministic compatible update group.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateGroup {
    /// One-based group number in the plan.
    pub group_number: u32,
    /// BLAKE3 group identity.
    pub group_identity: String,
    /// Prepared branch label.
    pub branch_label: String,
    /// Grouped updates.
    pub updates: Vec<UpdateCandidate>,
    /// Maximum risk in the group.
    pub risk: RiskLevel,
    /// Whether human review is required.
    pub requires_human_review: bool,
}

impl UpdateGroup {
    /// Validates the complete deterministic group projection.
    ///
    /// # Errors
    ///
    /// Returns a typed error when group metadata, updates, or identity are invalid.
    pub fn validate(&self) -> Result<(), Error> {
        if self.group_number == 0 {
            return Err(Error::InvalidBound("update_group.group_number"));
        }
        for update in &self.updates {
            update.validate()?;
        }
        validate_group_compatibility(&self.updates)?;
        let risk = group_risk(&self.updates);
        let requires_human_review = self.updates.iter().any(|candidate| {
            candidate.update_kind == UpdateKind::Security
                || matches!(risk, RiskLevel::High | RiskLevel::Unknown)
        });
        let expected_branch = format!("animus/maintenance/update-{}", self.group_number);
        let expected_identity = group_identity(&self.updates, self.group_number);
        if self.risk != risk
            || self.requires_human_review != requires_human_review
            || self.branch_label != expected_branch
            || self.group_identity != expected_identity
        {
            return Err(Error::BindingMismatch);
        }
        Ok(())
    }
}

/// Update-plan escalation reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpdateEscalationReason {
    /// No trusted validation profile is configured.
    MissingValidationProfile,
    /// Update has high or unknown risk.
    HighOrUnknownRisk,
}

/// Pure dependency-maintenance effect intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateEffect {
    /// Request bounded advisory risk analysis.
    RequestRiskAssessment {
        /// Exact group identity.
        group_identity: String,
    },
    /// Request an injected workspace update.
    PrepareWorkspaceUpdate {
        /// Exact update group.
        group: UpdateGroup,
    },
    /// Request trusted validation profiles.
    RequestValidation {
        /// Exact group identity.
        group_identity: String,
        /// Trusted validation profiles.
        profiles: Vec<ValidationProfileRef>,
    },
    /// Prepare a neutral change-request or patch-series draft.
    PrepareChangeRequest {
        /// Exact group identity.
        group_identity: String,
        /// Prepared draft.
        draft: ChangeRequestDraft,
    },
    /// Escalate a bounded condition.
    Escalate {
        /// Exact group identity.
        group_identity: String,
        /// Escalation reason.
        reason: UpdateEscalationReason,
    },
}

/// Deterministic dependency update plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePlan {
    /// Exact plan binding.
    pub binding: MaintenanceBinding,
    /// Ordered compatible groups.
    pub groups: Vec<UpdateGroup>,
    /// Ordered pure effect intents.
    pub effects: Vec<UpdateEffect>,
}

/// Plans one bounded dependency update run without effects.
///
/// # Errors
///
/// Returns a typed error when candidates, policy, bounds, or grouping are invalid.
pub fn plan_update_run(input: UpdatePlanInput) -> Result<UpdatePlan, Error> {
    input.bounds.validate()?;
    input.policy.validate()?;
    validate_collection_bound(
        "update_plan.candidates",
        input.candidates.len(),
        input.bounds.max_candidates,
    )?;
    validate_unique_candidates(&input.candidates)?;
    for candidate in &input.candidates {
        candidate.validate()?;
    }

    let mut groups = Vec::new();
    let mut current_batch = Vec::new();
    for candidate in input.candidates {
        if must_isolate(&candidate) {
            flush_batch(&mut current_batch, &mut groups, &input.policy, input.bounds)?;
            push_group(vec![candidate], &mut groups, &input.policy, input.bounds)?;
            continue;
        }

        if !current_batch.is_empty() && !candidate_matches_batch(&candidate, &current_batch) {
            flush_batch(&mut current_batch, &mut groups, &input.policy, input.bounds)?;
        }
        current_batch.push(candidate);
        if bounded_u32(current_batch.len()) == input.bounds.max_group_size {
            flush_batch(&mut current_batch, &mut groups, &input.policy, input.bounds)?;
        }
    }
    flush_batch(&mut current_batch, &mut groups, &input.policy, input.bounds)?;

    let mut effects = Vec::new();
    for group in &groups {
        if group.requires_human_review {
            effects.push(UpdateEffect::RequestRiskAssessment {
                group_identity: group.group_identity.clone(),
            });
        }
        effects.push(UpdateEffect::PrepareWorkspaceUpdate {
            group: group.clone(),
        });
        if input.policy.validation_profiles.is_empty() {
            effects.push(UpdateEffect::Escalate {
                group_identity: group.group_identity.clone(),
                reason: UpdateEscalationReason::MissingValidationProfile,
            });
        } else {
            effects.push(UpdateEffect::RequestValidation {
                group_identity: group.group_identity.clone(),
                profiles: input.policy.validation_profiles.clone(),
            });
        }
        if matches!(group.risk, RiskLevel::High | RiskLevel::Unknown) {
            effects.push(UpdateEffect::Escalate {
                group_identity: group.group_identity.clone(),
                reason: UpdateEscalationReason::HighOrUnknownRisk,
            });
        }
        effects.push(UpdateEffect::PrepareChangeRequest {
            group_identity: group.group_identity.clone(),
            draft: group_change_request_draft(group, input.policy.publication_mode),
        });
    }

    Ok(UpdatePlan {
        binding: input.binding,
        groups,
        effects,
    })
}

fn must_isolate(candidate: &UpdateCandidate) -> bool {
    matches!(
        candidate.update_kind,
        UpdateKind::Major | UpdateKind::Security
    )
}

fn candidate_matches_batch(candidate: &UpdateCandidate, batch: &[UpdateCandidate]) -> bool {
    let Some(first) = batch.first() else {
        return true;
    };
    candidate.ecosystem == first.ecosystem
        && candidate.manifest_identity == first.manifest_identity
        && !must_isolate(candidate)
}

fn flush_batch(
    current_batch: &mut Vec<UpdateCandidate>,
    groups: &mut Vec<UpdateGroup>,
    policy: &UpdatePolicy,
    bounds: RunBounds,
) -> Result<(), Error> {
    if current_batch.is_empty() {
        return Ok(());
    }
    let updates = core::mem::take(current_batch);
    push_group(updates, groups, policy, bounds)
}

fn push_group(
    updates: Vec<UpdateCandidate>,
    groups: &mut Vec<UpdateGroup>,
    policy: &UpdatePolicy,
    bounds: RunBounds,
) -> Result<(), Error> {
    if updates.is_empty() {
        return Err(Error::IncompatibleGroup);
    }
    validate_collection_bound("update_group.updates", updates.len(), bounds.max_group_size)?;
    validate_group_compatibility(&updates)?;
    let next_group_count = bounded_u32(groups.len()).saturating_add(1);
    if next_group_count > bounds.max_groups {
        return Err(Error::BoundExceeded {
            field: "update_plan.groups",
            actual: next_group_count,
            maximum: bounds.max_groups,
        });
    }
    let risk = group_risk(&updates);
    let requires_human_review = group_requires_review(&updates, policy, risk);
    let group_identity = group_identity(&updates, next_group_count);
    groups.push(UpdateGroup {
        group_number: next_group_count,
        branch_label: format!("animus/maintenance/update-{next_group_count}"),
        group_identity,
        updates,
        risk,
        requires_human_review,
    });
    Ok(())
}

fn validate_group_compatibility(updates: &[UpdateCandidate]) -> Result<(), Error> {
    let Some(first) = updates.first() else {
        return Err(Error::IncompatibleGroup);
    };
    if must_isolate(first) && updates.len() != 1 {
        return Err(Error::IncompatibleGroup);
    }
    for candidate in updates {
        if candidate.ecosystem != first.ecosystem
            || candidate.manifest_identity != first.manifest_identity
            || (must_isolate(candidate) && updates.len() != 1)
        {
            return Err(Error::IncompatibleGroup);
        }
    }
    Ok(())
}

fn group_risk(updates: &[UpdateCandidate]) -> RiskLevel {
    let mut risk = RiskLevel::Low;
    for update in updates {
        if risk_rank(update.risk) > risk_rank(risk) {
            risk = update.risk;
        }
        if update.update_kind == UpdateKind::Major {
            risk = RiskLevel::High;
        }
    }
    risk
}

const fn risk_rank(risk: RiskLevel) -> u8 {
    match risk {
        RiskLevel::Low => RISK_RANK_LOW,
        RiskLevel::Medium => RISK_RANK_MEDIUM,
        RiskLevel::High => RISK_RANK_HIGH,
        RiskLevel::Unknown => RISK_RANK_UNKNOWN,
    }
}

fn group_requires_review(
    updates: &[UpdateCandidate],
    policy: &UpdatePolicy,
    risk: RiskLevel,
) -> bool {
    updates.iter().any(|candidate| {
        candidate.update_kind == UpdateKind::Security
            || (policy.require_review_for_major && candidate.update_kind == UpdateKind::Major)
    }) || matches!(risk, RiskLevel::High | RiskLevel::Unknown)
}

fn group_identity(updates: &[UpdateCandidate], group_number: u32) -> String {
    let mut hasher = Hasher::new();
    frame(&mut hasher, GROUP_IDENTITY_DOMAIN);
    frame(&mut hasher, &group_number.to_string());
    for update in updates {
        frame(&mut hasher, &update.candidate_identity);
        frame(&mut hasher, &update.dependency_name);
        frame(&mut hasher, ecosystem_label(update.ecosystem));
        frame(&mut hasher, &update.manifest_identity);
        frame(&mut hasher, &update.current_version);
        frame(&mut hasher, &update.target_version);
        frame(&mut hasher, update_kind_label(update.update_kind));
        frame(&mut hasher, risk_label(update.risk));
        frame(
            &mut hasher,
            update.advisory_identity.as_deref().unwrap_or("none"),
        );
    }
    format!("b3:{}", hasher.finalize().to_hex())
}

fn group_change_request_draft(
    group: &UpdateGroup,
    publication_mode: PublicationMode,
) -> ChangeRequestDraft {
    ChangeRequestDraft {
        title: format!("Update dependency group {}", group.group_identity),
        body: String::from(
            "The host must add verified update, risk, validation, and publication evidence.",
        ),
        source_branch: group.branch_label.clone(),
        mode: publication_mode,
    }
}

fn frame(hasher: &mut Hasher, value: &str) {
    let length = u64::try_from(value.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(value.as_bytes());
}

const fn ecosystem_label(ecosystem: Ecosystem) -> &'static str {
    match ecosystem {
        Ecosystem::Cargo => "cargo",
        Ecosystem::Nix => "nix",
        Ecosystem::Npm => "npm",
        Ecosystem::Python => "python",
        Ecosystem::Go => "go",
        Ecosystem::Unknown => "unknown",
    }
}

const fn update_kind_label(kind: UpdateKind) -> &'static str {
    match kind {
        UpdateKind::Patch => "patch",
        UpdateKind::Minor => "minor",
        UpdateKind::Major => "major",
        UpdateKind::Security => "security",
        UpdateKind::Unknown => "unknown",
    }
}

const fn risk_label(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::Low => "low",
        RiskLevel::Medium => "medium",
        RiskLevel::High => "high",
        RiskLevel::Unknown => "unknown",
    }
}

fn validate_unique_candidates(candidates: &[UpdateCandidate]) -> Result<(), Error> {
    for (index, candidate) in candidates.iter().enumerate() {
        if candidates
            .iter()
            .skip(index.saturating_add(1))
            .any(|other| other.candidate_identity == candidate.candidate_identity)
        {
            return Err(Error::InvalidField("update_plan.duplicate_candidate"));
        }
    }
    Ok(())
}

fn validate_collection_bound(
    field: &'static str,
    actual: usize,
    maximum: u32,
) -> Result<(), Error> {
    let actual = bounded_u32(actual);
    if actual > maximum {
        return Err(Error::BoundExceeded {
            field,
            actual,
            maximum,
        });
    }
    Ok(())
}
