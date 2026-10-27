//! Pure skill-source lifecycle contracts.
//!
//! Hosts supply already-loaded source observations. This module never discovers,
//! fetches, traverses, extracts, stores, reviews, or activates source material.

#![allow(missing_docs)]

use alloc::string::ToString;

const BLAKE3_HEX_CHARS: usize = 64;
const MIN_REVISION_CHARS: usize = 32;
const MAX_REVISION_CHARS: usize = 128;
const MAX_LABEL_CHARS: usize = 96;
const MAX_OBSERVATION_AGE_EPOCHS: u32 = 2;
const RECEIPT_DOMAIN: &str = "animus.skill-source-receipt.v1";

/// Effects that remain outside the reusable source lifecycle core.
pub const PROHIBITED_SKILL_SOURCE_EFFECTS: &[&str] = &[
    "source discovery",
    "credential resolution",
    "VCS execution",
    "network transport",
    "archive handling",
    "filesystem traversal",
    "symlink resolution",
    "extraction",
    "storage mutation",
    "clock access",
    "review authority",
    "runtime activation",
];

/// Mandatory limits for one host-owned source observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSourceBounds {
    pub file_count_max: u32,
    pub total_bytes_max: u64,
    pub file_bytes_max: u64,
    pub path_depth_max: u16,
    pub process_timeout_ms: u64,
    pub stdout_bytes_max: u64,
    pub stderr_bytes_max: u64,
}

impl SkillSourceBounds {
    fn valid(self) -> bool {
        self.file_count_max > 0
            && self.total_bytes_max > 0
            && self.file_bytes_max > 0
            && self.file_bytes_max <= self.total_bytes_max
            && self.path_depth_max > 0
            && self.process_timeout_ms > 0
            && self.stdout_bytes_max > 0
            && self.stderr_bytes_max > 0
    }
}

/// Closed source-kind vocabulary. Values are labels, not transport authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillSourceKind {
    Git,
    Radicle,
    Archive,
    Filesystem,
}

/// Host-supplied observation facts from bounded mechanisms.
// Each boolean records one independent host attestation and preserves the public DTO.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSourceHostFacts {
    pub process_bounded: bool,
    pub path_admitted: bool,
    pub entry_types_admitted: bool,
    pub symlink_free: bool,
    pub submodule_free: bool,
    pub complete: bool,
}

/// Source and asset collision facts supplied after host inventory checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSourceCollisionFacts {
    pub duplicate_asset: bool,
    pub source_collision: bool,
}

/// Pure policy projection consumed by the reusable core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSourcePolicy {
    policy_identity: alloc::string::String,
    bounds_profile: alloc::string::String,
    canonicalization_profile: alloc::string::String,
    bounds: SkillSourceBounds,
}

/// Borrowed policy fields loaded by a host from the checked projection.
// Each boolean denies one independent authority or unsafe source behavior.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSourcePolicyDraft<'a> {
    pub policy_identity: &'a str,
    pub bounds_profile: &'a str,
    pub canonicalization_profile: &'a str,
    pub bounds: SkillSourceBounds,
    pub allow_symlinks: bool,
    pub allow_submodules: bool,
    pub automatic_activation: bool,
    pub ambient_discovery: bool,
}

/// Stable fail-closed issue classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillSourceIssue {
    UnsafeMetadata,
    InvalidIdentity,
    InvalidRevision,
    InvalidBounds,
    PolicyMismatch,
    UnsupportedMode,
    HostFactsDenied,
    StaleObservation,
    DuplicateAsset,
    SourceCollision,
    FrozenMismatch,
    CandidateConflict,
    CandidateInactive,
    PromotionDrift,
    ReviewMissing,
    AutomaticActivation,
    MutationDenied,
    UnsupportedClaim,
}

impl SkillSourcePolicy {
    /// Validate a checked policy projection without evaluating Nickel at runtime.
    ///
    /// # Errors
    ///
    /// Returns an issue when identities, bounds, or denied policy flags are invalid.
    pub fn validate(draft: &SkillSourcePolicyDraft<'_>) -> Result<Self, SkillSourceIssue> {
        validate_hash(draft.policy_identity)?;
        validate_label(draft.bounds_profile)?;
        validate_label(draft.canonicalization_profile)?;
        if !draft.bounds.valid() {
            return Err(SkillSourceIssue::InvalidBounds);
        }
        if draft.allow_symlinks {
            return Err(SkillSourceIssue::PolicyMismatch);
        }
        if draft.allow_submodules {
            return Err(SkillSourceIssue::PolicyMismatch);
        }
        if draft.automatic_activation {
            return Err(SkillSourceIssue::PolicyMismatch);
        }
        if draft.ambient_discovery {
            return Err(SkillSourceIssue::PolicyMismatch);
        }
        Ok(Self {
            policy_identity: draft.policy_identity.into(),
            bounds_profile: draft.bounds_profile.into(),
            canonicalization_profile: draft.canonicalization_profile.into(),
            bounds: draft.bounds,
        })
    }

    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }
    #[must_use]
    pub fn bounds_profile(&self) -> &str {
        &self.bounds_profile
    }
    #[must_use]
    pub fn canonicalization_profile(&self) -> &str {
        &self.canonicalization_profile
    }
    #[must_use]
    pub const fn bounds(&self) -> SkillSourceBounds {
        self.bounds
    }
}

/// Exact immutable source binding for an active skill.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenSkillSource {
    asset_id: alloc::string::String,
    source_kind: SkillSourceKind,
    origin_label: alloc::string::String,
    revision: alloc::string::String,
    tree_identity: alloc::string::String,
    manifest_identity: alloc::string::String,
    bounds_profile: alloc::string::String,
    canonicalization_profile: alloc::string::String,
    policy_identity: alloc::string::String,
    provenance_ref: alloc::string::String,
    trust_ref: alloc::string::String,
    review_ref: alloc::string::String,
}

/// Borrowed exact frozen source fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenSkillSourceDraft<'a> {
    pub asset_id: &'a str,
    pub source_kind: SkillSourceKind,
    pub origin_label: &'a str,
    pub revision: &'a str,
    pub tree_identity: &'a str,
    pub manifest_identity: &'a str,
    pub bounds_profile: &'a str,
    pub canonicalization_profile: &'a str,
    pub policy_identity: &'a str,
    pub provenance_ref: &'a str,
    pub trust_ref: &'a str,
    pub review_ref: &'a str,
}

impl FrozenSkillSource {
    /// Validate one exact immutable source binding.
    ///
    /// # Errors
    ///
    /// Returns an issue when metadata, identities, revision, or policy facts are invalid.
    pub fn validate(
        policy: &SkillSourcePolicy,
        draft: &FrozenSkillSourceDraft<'_>,
    ) -> Result<Self, SkillSourceIssue> {
        validate_label(draft.asset_id)?;
        validate_label(draft.origin_label)?;
        validate_revision(draft.revision)?;
        validate_hash(draft.tree_identity)?;
        validate_hash(draft.manifest_identity)?;
        validate_label(draft.provenance_ref)?;
        validate_label(draft.trust_ref)?;
        validate_label(draft.review_ref)?;
        if draft.bounds_profile != policy.bounds_profile
            || draft.canonicalization_profile != policy.canonicalization_profile
            || draft.policy_identity != policy.policy_identity
        {
            return Err(SkillSourceIssue::PolicyMismatch);
        }
        Ok(Self {
            asset_id: draft.asset_id.into(),
            source_kind: draft.source_kind,
            origin_label: draft.origin_label.into(),
            revision: draft.revision.into(),
            tree_identity: draft.tree_identity.into(),
            manifest_identity: draft.manifest_identity.into(),
            bounds_profile: draft.bounds_profile.into(),
            canonicalization_profile: draft.canonicalization_profile.into(),
            policy_identity: draft.policy_identity.into(),
            provenance_ref: draft.provenance_ref.into(),
            trust_ref: draft.trust_ref.into(),
            review_ref: draft.review_ref.into(),
        })
    }

    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
    #[must_use]
    pub fn tree_identity(&self) -> &str {
        &self.tree_identity
    }
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        &self.manifest_identity
    }
}

/// Borrowed tracked observation supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrackedSkillSourceObservationDraft<'a> {
    pub asset_id: &'a str,
    pub source_kind: SkillSourceKind,
    pub origin_label: &'a str,
    pub tracking_label: &'a str,
    pub observed_revision: &'a str,
    pub tree_identity: &'a str,
    pub manifest_identity: &'a str,
    pub observation_identity: &'a str,
    pub observed_epoch: u32,
    pub current_epoch: u32,
    pub bounds_profile: &'a str,
    pub canonicalization_profile: &'a str,
    pub policy_identity: &'a str,
    pub provenance_ref: &'a str,
    pub host_facts: SkillSourceHostFacts,
    pub collisions: SkillSourceCollisionFacts,
}

/// Pure tracked-source classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackedCandidateClass {
    Unchanged,
    ChangedInactive,
}

/// Validated inactive candidate or no-op observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackedSkillSourceCandidate {
    candidate_identity: alloc::string::String,
    asset_id: alloc::string::String,
    source_kind: SkillSourceKind,
    origin_label: alloc::string::String,
    tracking_label: alloc::string::String,
    revision: alloc::string::String,
    tree_identity: alloc::string::String,
    manifest_identity: alloc::string::String,
    observation_identity: alloc::string::String,
    observed_epoch: u32,
    class: TrackedCandidateClass,
    bounds_profile: alloc::string::String,
    canonicalization_profile: alloc::string::String,
    policy_identity: alloc::string::String,
    provenance_ref: alloc::string::String,
}

impl TrackedSkillSourceCandidate {
    #[must_use]
    pub fn candidate_identity(&self) -> &str {
        &self.candidate_identity
    }
    #[must_use]
    pub const fn class(&self) -> TrackedCandidateClass {
        self.class
    }
    #[must_use]
    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }
    #[must_use]
    pub fn tree_identity(&self) -> &str {
        &self.tree_identity
    }
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        &self.manifest_identity
    }
    #[must_use]
    pub fn tracking_label(&self) -> &str {
        &self.tracking_label
    }
    #[must_use]
    pub fn observation_identity(&self) -> &str {
        &self.observation_identity
    }
    #[must_use]
    pub const fn observed_epoch(&self) -> u32 {
        self.observed_epoch
    }
}

/// Classify a bounded host observation without activating it.
///
/// # Errors
///
/// Returns an issue when source facts are unsafe, stale, conflicting, or incomplete.
pub fn classify_tracked_skill_source(
    policy: &SkillSourcePolicy,
    draft: &TrackedSkillSourceObservationDraft<'_>,
    active: Option<&FrozenSkillSource>,
) -> Result<TrackedSkillSourceCandidate, SkillSourceIssue> {
    validate_tracked_observation(policy, draft)?;
    let class = classify_candidate_change(draft, active)?;
    debug_assert!(host_facts_are_admitted(draft.host_facts));
    debug_assert!(draft.current_epoch >= draft.observed_epoch);
    let candidate_identity = public_identity(&[
        "candidate",
        draft.asset_id,
        draft.origin_label,
        draft.tracking_label,
        draft.observed_revision,
        draft.tree_identity,
        draft.manifest_identity,
        draft.observation_identity,
        draft.policy_identity,
    ]);
    Ok(TrackedSkillSourceCandidate {
        candidate_identity,
        asset_id: draft.asset_id.into(),
        source_kind: draft.source_kind,
        origin_label: draft.origin_label.into(),
        tracking_label: draft.tracking_label.into(),
        revision: draft.observed_revision.into(),
        tree_identity: draft.tree_identity.into(),
        manifest_identity: draft.manifest_identity.into(),
        observation_identity: draft.observation_identity.into(),
        observed_epoch: draft.observed_epoch,
        class,
        bounds_profile: draft.bounds_profile.into(),
        canonicalization_profile: draft.canonicalization_profile.into(),
        policy_identity: draft.policy_identity.into(),
        provenance_ref: draft.provenance_ref.into(),
    })
}

fn validate_tracked_observation(
    policy: &SkillSourcePolicy,
    draft: &TrackedSkillSourceObservationDraft<'_>,
) -> Result<(), SkillSourceIssue> {
    validate_label(draft.asset_id)?;
    validate_label(draft.origin_label)?;
    validate_label(draft.tracking_label)?;
    validate_revision(draft.observed_revision)?;
    validate_hash(draft.tree_identity)?;
    validate_hash(draft.manifest_identity)?;
    validate_hash(draft.observation_identity)?;
    validate_label(draft.provenance_ref)?;
    if draft.bounds_profile != policy.bounds_profile {
        return Err(SkillSourceIssue::PolicyMismatch);
    }
    if draft.canonicalization_profile != policy.canonicalization_profile {
        return Err(SkillSourceIssue::PolicyMismatch);
    }
    if draft.policy_identity != policy.policy_identity {
        return Err(SkillSourceIssue::PolicyMismatch);
    }
    if draft.collisions.duplicate_asset {
        return Err(SkillSourceIssue::DuplicateAsset);
    }
    if draft.collisions.source_collision {
        return Err(SkillSourceIssue::SourceCollision);
    }
    if !host_facts_are_admitted(draft.host_facts) {
        return Err(SkillSourceIssue::HostFactsDenied);
    }
    let age_epochs = draft
        .current_epoch
        .checked_sub(draft.observed_epoch)
        .ok_or(SkillSourceIssue::StaleObservation)?;
    if age_epochs > MAX_OBSERVATION_AGE_EPOCHS {
        return Err(SkillSourceIssue::StaleObservation);
    }
    debug_assert!(host_facts_are_admitted(draft.host_facts));
    debug_assert!(draft.current_epoch >= draft.observed_epoch);
    Ok(())
}

fn host_facts_are_admitted(facts: SkillSourceHostFacts) -> bool {
    if !facts.process_bounded {
        return false;
    }
    if !facts.path_admitted {
        return false;
    }
    if !facts.entry_types_admitted {
        return false;
    }
    if !facts.symlink_free {
        return false;
    }
    if !facts.submodule_free {
        return false;
    }
    facts.complete
}

fn classify_candidate_change(
    draft: &TrackedSkillSourceObservationDraft<'_>,
    active: Option<&FrozenSkillSource>,
) -> Result<TrackedCandidateClass, SkillSourceIssue> {
    let Some(current) = active else {
        return Ok(TrackedCandidateClass::ChangedInactive);
    };
    if current.asset_id != draft.asset_id {
        return Err(SkillSourceIssue::SourceCollision);
    }
    if current.source_kind != draft.source_kind {
        return Err(SkillSourceIssue::SourceCollision);
    }
    if current.origin_label != draft.origin_label {
        return Err(SkillSourceIssue::SourceCollision);
    }
    debug_assert_eq!(current.asset_id, draft.asset_id);
    debug_assert_eq!(current.origin_label, draft.origin_label);
    let is_revision_same = current.revision == draft.observed_revision;
    let is_tree_same = current.tree_identity == draft.tree_identity;
    let is_manifest_same = current.manifest_identity == draft.manifest_identity;
    if is_revision_same && is_tree_same && is_manifest_same {
        return Ok(TrackedCandidateClass::Unchanged);
    }
    if !is_revision_same && !is_tree_same {
        return Ok(TrackedCandidateClass::ChangedInactive);
    }
    Err(SkillSourceIssue::CandidateConflict)
}

/// Compatibility result for host asset manifests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkillSourceManifestBinding {
    LegacyNoSourceClaim,
    ExactFrozen,
}

/// Bind an existing host asset manifest to an optional exact frozen source.
///
/// # Errors
///
/// Returns an issue when an exact source does not match the admitted manifest.
pub fn reconcile_skill_source_manifest(
    manifest: &crate::asset::HostAssetManifestEntry,
    source: Option<&FrozenSkillSource>,
) -> Result<SkillSourceManifestBinding, SkillSourceIssue> {
    let Some(source) = source else {
        return Ok(SkillSourceManifestBinding::LegacyNoSourceClaim);
    };
    if manifest.kind() != crate::asset::HostAssetKind::Skill
        || manifest.asset_id() != source.asset_id
        || manifest.content_identity() != source.manifest_identity
    {
        return Err(SkillSourceIssue::FrozenMismatch);
    }
    Ok(SkillSourceManifestBinding::ExactFrozen)
}

/// Exact promotion request. It cannot request direct activation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillSourcePromotionRequest<'a> {
    pub candidate_identity: &'a str,
    pub review_ref: &'a str,
    pub trust_ref: &'a str,
    pub requested_action: crate::asset::HostAssetActionKind,
    pub automatic_activation: bool,
    pub mutation: crate::asset::HostAssetMutationRequestDraft<'a>,
    pub service: crate::asset::HostAssetServiceStatus,
}

/// Passing promotion decision with the existing host mutation admission.
pub struct SkillSourcePromotionAdmission {
    target: FrozenSkillSource,
    mutation: crate::asset::HostAssetMutationAdmission,
}

impl SkillSourcePromotionAdmission {
    #[must_use]
    pub const fn target(&self) -> &FrozenSkillSource {
        &self.target
    }
    #[must_use]
    pub const fn mutation(&self) -> &crate::asset::HostAssetMutationAdmission {
        &self.mutation
    }
}

/// Admit an exact candidate-to-frozen transition and preserve the host mutation gate.
///
/// # Errors
///
/// Returns an issue when review, identity, target, action, or mutation facts do not match.
pub fn admit_skill_source_promotion(
    candidate: &TrackedSkillSourceCandidate,
    target: FrozenSkillSource,
    request: &SkillSourcePromotionRequest<'_>,
    plan: &crate::asset::HostAssetReconciliationPlan,
) -> Result<SkillSourcePromotionAdmission, SkillSourceIssue> {
    if candidate.class != TrackedCandidateClass::ChangedInactive {
        return Err(SkillSourceIssue::CandidateInactive);
    }
    validate_label(request.review_ref)?;
    validate_label(request.trust_ref)?;
    if request.automatic_activation {
        return Err(SkillSourceIssue::AutomaticActivation);
    }
    if !promotion_target_matches(candidate, &target, request) {
        return Err(SkillSourceIssue::PromotionDrift);
    }
    if !matches!(
        request.requested_action,
        crate::asset::HostAssetActionKind::Install | crate::asset::HostAssetActionKind::Update
    ) {
        return Err(SkillSourceIssue::AutomaticActivation);
    }
    if request.mutation.kind != request.requested_action {
        return Err(SkillSourceIssue::AutomaticActivation);
    }
    if request.mutation.asset_id != candidate.asset_id {
        return Err(SkillSourceIssue::AutomaticActivation);
    }
    debug_assert_eq!(target.revision, candidate.revision);
    debug_assert_eq!(target.manifest_identity, candidate.manifest_identity);
    let is_action_match = plan.actions().iter().any(|action| {
        action.action_id() == request.mutation.action_id
            && action.content_identity() == Some(candidate.manifest_identity.as_str())
    });
    if !is_action_match {
        return Err(SkillSourceIssue::PromotionDrift);
    }
    let mutation =
        crate::asset::admit_host_asset_mutation(plan, &request.mutation, request.service);
    if mutation.request().is_none() {
        return Err(SkillSourceIssue::MutationDenied);
    }
    Ok(SkillSourcePromotionAdmission { target, mutation })
}

fn promotion_target_matches(
    candidate: &TrackedSkillSourceCandidate,
    target: &FrozenSkillSource,
    request: &SkillSourcePromotionRequest<'_>,
) -> bool {
    if request.candidate_identity != candidate.candidate_identity {
        return false;
    }
    if target.asset_id != candidate.asset_id {
        return false;
    }
    if target.source_kind != candidate.source_kind {
        return false;
    }
    if target.origin_label != candidate.origin_label {
        return false;
    }
    if target.revision != candidate.revision {
        return false;
    }
    if target.tree_identity != candidate.tree_identity {
        return false;
    }
    if target.manifest_identity != candidate.manifest_identity {
        return false;
    }
    if target.bounds_profile != candidate.bounds_profile {
        return false;
    }
    if target.canonicalization_profile != candidate.canonicalization_profile {
        return false;
    }
    if target.policy_identity != candidate.policy_identity {
        return false;
    }
    if target.provenance_ref != candidate.provenance_ref {
        return false;
    }
    if target.review_ref != request.review_ref {
        return false;
    }
    debug_assert_eq!(target.revision, candidate.revision);
    debug_assert_eq!(target.manifest_identity, candidate.manifest_identity);
    target.trust_ref == request.trust_ref
}

/// Metadata-only promotion receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillSourceReceipt {
    receipt_identity: alloc::string::String,
    candidate_identity: alloc::string::String,
    asset_id: alloc::string::String,
    revision: alloc::string::String,
    tree_identity: alloc::string::String,
    manifest_identity: alloc::string::String,
    policy_identity: alloc::string::String,
    observation_identity: alloc::string::String,
    outcome: alloc::string::String,
}

impl SkillSourceReceipt {
    #[must_use]
    pub fn promotion(candidate: &TrackedSkillSourceCandidate) -> Self {
        let outcome = alloc::string::String::from("promotion-admitted");
        let receipt_identity = public_identity(&[
            RECEIPT_DOMAIN,
            &candidate.candidate_identity,
            &candidate.asset_id,
            &candidate.revision,
            &candidate.tree_identity,
            &candidate.manifest_identity,
            &candidate.policy_identity,
            &candidate.observation_identity,
            &outcome,
        ]);
        Self {
            receipt_identity,
            candidate_identity: candidate.candidate_identity.clone(),
            asset_id: candidate.asset_id.clone(),
            revision: candidate.revision.clone(),
            tree_identity: candidate.tree_identity.clone(),
            manifest_identity: candidate.manifest_identity.clone(),
            policy_identity: candidate.policy_identity.clone(),
            observation_identity: candidate.observation_identity.clone(),
            outcome,
        }
    }

    #[must_use]
    pub fn receipt_identity(&self) -> &str {
        &self.receipt_identity
    }

    /// Validate the metadata-only receipt fields.
    ///
    /// # Errors
    ///
    /// Returns an issue when a receipt field is malformed or unsafe.
    pub fn validate(&self) -> Result<(), SkillSourceIssue> {
        validate_hash(&self.receipt_identity)?;
        validate_hash(&self.candidate_identity)?;
        validate_label(&self.asset_id)?;
        validate_revision(&self.revision)?;
        validate_hash(&self.tree_identity)?;
        validate_hash(&self.manifest_identity)?;
        validate_hash(&self.policy_identity)?;
        validate_hash(&self.observation_identity)?;
        validate_label(&self.outcome)
    }
}

/// Reject an unsupported interpretation of a content identity.
///
/// # Errors
///
/// Returns an issue when the claim overstates content identity or contains unsafe metadata.
pub fn validate_skill_source_claim(claim: &str) -> Result<(), SkillSourceIssue> {
    const DENIED: &[&str] = &[
        "authentic",
        "author",
        "safe",
        "authority",
        "confined",
        "release-ready",
        "trusted",
    ];
    let lower = claim.to_ascii_lowercase();
    if DENIED.iter().any(|word| lower.contains(word)) {
        return Err(SkillSourceIssue::UnsupportedClaim);
    }
    validate_label(claim)
}

fn validate_hash(value: &str) -> Result<(), SkillSourceIssue> {
    if value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(SkillSourceIssue::InvalidIdentity)
    }
}

fn validate_revision(value: &str) -> Result<(), SkillSourceIssue> {
    if (MIN_REVISION_CHARS..=MAX_REVISION_CHARS).contains(&value.len())
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        Ok(())
    } else {
        Err(SkillSourceIssue::InvalidRevision)
    }
}

fn validate_label(value: &str) -> Result<(), SkillSourceIssue> {
    if label_is_safe(value) {
        Ok(())
    } else {
        Err(SkillSourceIssue::UnsafeMetadata)
    }
}

fn label_is_safe(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    if value.len() > MAX_LABEL_CHARS {
        return false;
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return false;
    }
    if value.contains("://") || value.contains("..") {
        return false;
    }
    let lowercase = value.to_ascii_lowercase();
    if lowercase.contains("token") || lowercase.contains("credential") {
        return false;
    }
    debug_assert!(!value.is_empty());
    debug_assert!(value.len() <= MAX_LABEL_CHARS);
    true
}

fn public_identity(fields: &[&str]) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    for field in fields {
        hasher.update(&(field.len() as u64).to_le_bytes());
        hasher.update(field.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const REV_A: &str = "1111111111111111111111111111111111111111";
    const REV_B: &str = "2222222222222222222222222222222222222222";
    const ACTIVE_SOURCE: SourceFixture<'_> = SourceFixture {
        revision: REV_A,
        tree_identity: HASH_A,
        manifest_identity: HASH_A,
    };
    const CHANGED_SOURCE: SourceFixture<'_> = SourceFixture {
        revision: REV_B,
        tree_identity: HASH_B,
        manifest_identity: HASH_B,
    };

    #[derive(Clone, Copy)]
    struct SourceFixture<'a> {
        revision: &'a str,
        tree_identity: &'a str,
        manifest_identity: &'a str,
    }

    fn bounds() -> SkillSourceBounds {
        SkillSourceBounds {
            file_count_max: 256,
            total_bytes_max: 1_048_576,
            file_bytes_max: 65_536,
            path_depth_max: 12,
            process_timeout_ms: 30_000,
            stdout_bytes_max: 4096,
            stderr_bytes_max: 4096,
        }
    }
    fn policy() -> SkillSourcePolicy {
        SkillSourcePolicy::validate(&SkillSourcePolicyDraft {
            policy_identity: HASH_A,
            bounds_profile: "skill-tree-standard-v1",
            canonicalization_profile: "skill-tree-blake3-v1",
            bounds: bounds(),
            allow_symlinks: false,
            allow_submodules: false,
            automatic_activation: false,
            ambient_discovery: false,
        })
        .unwrap()
    }
    fn frozen(source: SourceFixture<'_>) -> FrozenSkillSource {
        FrozenSkillSource::validate(
            &policy(),
            &FrozenSkillSourceDraft {
                asset_id: "proof-search",
                source_kind: SkillSourceKind::Git,
                origin_label: "qm-reference",
                revision: source.revision,
                tree_identity: source.tree_identity,
                manifest_identity: source.manifest_identity,
                bounds_profile: "skill-tree-standard-v1",
                canonicalization_profile: "skill-tree-blake3-v1",
                policy_identity: HASH_A,
                provenance_ref: "qm-7f2c916",
                trust_ref: "reviewed-source",
                review_ref: "review-42",
            },
        )
        .unwrap()
    }
    fn observation(source: SourceFixture<'_>) -> TrackedSkillSourceObservationDraft<'_> {
        TrackedSkillSourceObservationDraft {
            asset_id: "proof-search",
            source_kind: SkillSourceKind::Git,
            origin_label: "qm-reference",
            tracking_label: "main-line",
            observed_revision: source.revision,
            tree_identity: source.tree_identity,
            manifest_identity: source.manifest_identity,
            observation_identity: HASH_B,
            observed_epoch: 10,
            current_epoch: 11,
            bounds_profile: "skill-tree-standard-v1",
            canonicalization_profile: "skill-tree-blake3-v1",
            policy_identity: HASH_A,
            provenance_ref: "qm-7f2c916",
            host_facts: SkillSourceHostFacts {
                process_bounded: true,
                path_admitted: true,
                entry_types_admitted: true,
                symlink_free: true,
                submodule_free: true,
                complete: true,
            },
            collisions: SkillSourceCollisionFacts {
                duplicate_asset: false,
                source_collision: false,
            },
        }
    }

    #[test]
    fn frozen_and_tracked_sources_preserve_inactive_candidate_semantics() {
        let active = frozen(ACTIVE_SOURCE);
        let unchanged =
            classify_tracked_skill_source(&policy(), &observation(ACTIVE_SOURCE), Some(&active))
                .unwrap();
        assert_eq!(unchanged.class(), TrackedCandidateClass::Unchanged);
        let changed =
            classify_tracked_skill_source(&policy(), &observation(CHANGED_SOURCE), Some(&active))
                .unwrap();
        assert_eq!(changed.class(), TrackedCandidateClass::ChangedInactive);
        assert_ne!(changed.manifest_identity(), active.manifest_identity());
    }

    #[test]
    fn malformed_stale_collision_partial_and_unsafe_inputs_fail_closed() {
        let active = frozen(ACTIVE_SOURCE);
        let mut cases = alloc::vec::Vec::new();
        let mut stale = observation(CHANGED_SOURCE);
        stale.current_epoch = 20;
        cases.push(stale);
        let mut duplicate = observation(CHANGED_SOURCE);
        duplicate.collisions.duplicate_asset = true;
        cases.push(duplicate);
        let mut collision = observation(CHANGED_SOURCE);
        collision.collisions.source_collision = true;
        cases.push(collision);
        let mut partial = observation(CHANGED_SOURCE);
        partial.host_facts.complete = false;
        cases.push(partial);
        let mut symlink = observation(CHANGED_SOURCE);
        symlink.host_facts.symlink_free = false;
        cases.push(symlink);
        let mut submodule = observation(CHANGED_SOURCE);
        submodule.host_facts.submodule_free = false;
        cases.push(submodule);
        for case in cases {
            assert!(classify_tracked_skill_source(&policy(), &case, Some(&active)).is_err());
        }
        let moving_tag = FrozenSkillSourceDraft {
            revision: "latest",
            ..FrozenSkillSourceDraft {
                asset_id: "proof-search",
                source_kind: SkillSourceKind::Git,
                origin_label: "qm-reference",
                revision: REV_A,
                tree_identity: HASH_A,
                manifest_identity: HASH_A,
                bounds_profile: "skill-tree-standard-v1",
                canonicalization_profile: "skill-tree-blake3-v1",
                policy_identity: HASH_A,
                provenance_ref: "qm-7f2c916",
                trust_ref: "reviewed-source",
                review_ref: "review-42",
            }
        };
        assert_eq!(
            FrozenSkillSource::validate(&policy(), &moving_tag),
            Err(SkillSourceIssue::InvalidRevision)
        );
    }

    #[test]
    fn exact_promotion_reuses_dry_run_and_injected_mutation_gate() {
        let candidate =
            classify_tracked_skill_source(&policy(), &observation(CHANGED_SOURCE), None).unwrap();
        let manifests = [crate::asset::HostAssetManifestEntryDraft {
            asset_id: "proof-search",
            kind: crate::asset::HostAssetKind::Skill,
            store_label: "skill-store",
            content_identity: HASH_B,
            provenance_label: "qm-7f2c916",
            trust: crate::asset::HostAssetTrust::Trusted,
        }];
        let inventory =
            crate::asset::validate_host_asset_inventory(&crate::asset::HostAssetInventoryDraft {
                manifests: &manifests,
                observations: &[],
                current_epoch: 11,
            })
            .unwrap();
        let drift_assessments = crate::asset::classify_host_asset_drift(&inventory);
        let plan = crate::asset::plan_host_asset_reconciliation(&drift_assessments);
        let action = plan.actions().first().unwrap();
        assert_eq!(action.kind(), crate::asset::HostAssetActionKind::Install);
        let request = SkillSourcePromotionRequest {
            candidate_identity: candidate.candidate_identity(),
            review_ref: "review-42",
            trust_ref: "reviewed-source",
            requested_action: crate::asset::HostAssetActionKind::Install,
            automatic_activation: false,
            mutation: crate::asset::HostAssetMutationRequestDraft {
                action_id: action.action_id(),
                asset_id: "proof-search",
                kind: crate::asset::HostAssetActionKind::Install,
                capability_label: action.required_capability(),
                plan_identity: HASH_C,
            },
            service: crate::asset::HostAssetServiceStatus::Available,
        };
        let direct =
            crate::asset::admit_host_asset_mutation(&plan, &request.mutation, request.service);
        assert!(direct.request().is_some(), "{:?}", direct.diagnostic());
        let target = frozen(CHANGED_SOURCE);
        let admitted = admit_skill_source_promotion(&candidate, target, &request, &plan).unwrap();
        assert!(admitted.mutation().request().is_some());

        assert_promotion_rejections(&candidate, request, &plan);
    }

    fn assert_promotion_rejections(
        candidate: &TrackedSkillSourceCandidate,
        request: SkillSourcePromotionRequest<'_>,
        plan: &crate::asset::HostAssetReconciliationPlan,
    ) {
        let missing_service = SkillSourcePromotionRequest {
            service: crate::asset::HostAssetServiceStatus::Missing,
            ..request
        };
        assert_eq!(
            admit_skill_source_promotion(
                candidate,
                frozen(CHANGED_SOURCE),
                &missing_service,
                plan,
            )
            .err(),
            Some(SkillSourceIssue::MutationDenied)
        );
        let automatic = SkillSourcePromotionRequest {
            automatic_activation: true,
            ..request
        };
        assert_eq!(
            admit_skill_source_promotion(candidate, frozen(CHANGED_SOURCE), &automatic, plan).err(),
            Some(SkillSourceIssue::AutomaticActivation)
        );
        let drifted = SkillSourcePromotionRequest {
            candidate_identity: HASH_A,
            ..request
        };
        assert_eq!(
            admit_skill_source_promotion(candidate, frozen(CHANGED_SOURCE), &drifted, plan).err(),
            Some(SkillSourceIssue::PromotionDrift)
        );
    }

    #[test]
    fn automatic_drift_missing_service_leaks_and_overclaims_are_denied() {
        assert_eq!(
            validate_skill_source_claim("tree hash proves trusted source"),
            Err(SkillSourceIssue::UnsupportedClaim)
        );
        assert!(validate_label("https://example.invalid/private").is_err());
        assert!(validate_label("provider-token").is_err());
        let invalid_policy = SkillSourcePolicyDraft {
            automatic_activation: true,
            ..SkillSourcePolicyDraft {
                policy_identity: HASH_A,
                bounds_profile: "bounds",
                canonicalization_profile: "canonical",
                bounds: bounds(),
                allow_symlinks: false,
                allow_submodules: false,
                automatic_activation: false,
                ambient_discovery: false,
            }
        };
        assert_eq!(
            SkillSourcePolicy::validate(&invalid_policy),
            Err(SkillSourceIssue::PolicyMismatch)
        );
    }

    #[test]
    fn legacy_manifest_compatibility_and_safe_receipt_remain_explicit() {
        let manifests = [crate::asset::HostAssetManifestEntryDraft {
            asset_id: "proof-search",
            kind: crate::asset::HostAssetKind::Skill,
            store_label: "skill-store",
            content_identity: HASH_A,
            provenance_label: "legacy",
            trust: crate::asset::HostAssetTrust::Trusted,
        }];
        let inventory =
            crate::asset::validate_host_asset_inventory(&crate::asset::HostAssetInventoryDraft {
                manifests: &manifests,
                observations: &[],
                current_epoch: 1,
            })
            .unwrap();
        assert_eq!(
            reconcile_skill_source_manifest(&inventory.manifests()[0], None).unwrap(),
            SkillSourceManifestBinding::LegacyNoSourceClaim
        );
        let candidate = classify_tracked_skill_source(
            &policy(),
            &observation(CHANGED_SOURCE),
            Some(&frozen(ACTIVE_SOURCE)),
        )
        .unwrap();
        let receipt = SkillSourceReceipt::promotion(&candidate);
        receipt.validate().unwrap();
        assert_eq!(receipt.receipt_identity().len(), BLAKE3_HEX_CHARS);
    }
}
