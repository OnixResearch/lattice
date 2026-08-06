// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven
// validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(compound_condition, excessive_file_length, function_length, non_trait_imports)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
use std::collections::BTreeSet;

use super::BLAKE3_ALGORITHM;
use super::DEFAULT_FALLBACK_TASK_CLASS;
use super::FIELD_ACTIVE_REVISION;
use super::FIELD_BASE_REVISION;
use super::FIELD_CONTENT_HASH;
use super::FIELD_PROFILE_ENABLED;
use super::FIELD_PROFILE_ID;
use super::FIELD_PROFILE_PROVENANCE;
use super::FIELD_RECEIPT_HASH;
use super::FIELD_RECEIPT_PROFILE;
use super::FIELD_RECEIPT_ROUTE;
use super::FIELD_RECEIPT_SENSITIVE;
use super::FIELD_RECEIPT_TASK;
use super::FIELD_REVISION_HASH_MATCH;
use super::FIELD_REVISION_ID;
use super::FIELD_REVISION_STATUS;
use super::FIELD_ROUTE_CONSTRAINTS;
use super::FIELD_SELECTION_PROFILE;
use super::FIELD_STRATEGY_KNOBS;
use super::FIELD_TASK_CLASS;
use super::HASH_PART_SEPARATOR;
use super::SCAFFOLD_HASH_DOMAIN;
use super::data::ScaffoldAdmissionDecision;
use super::data::ScaffoldAdmissionEffect;
use super::data::ScaffoldAdmissionOutcome;
use super::data::ScaffoldAdmissionRequest;
use super::data::ScaffoldDiagnostic;
use super::data::ScaffoldDiagnosticKind;
use super::data::ScaffoldFingerprint;
use super::data::ScaffoldInfluenceReceipt;
use super::data::ScaffoldProfile;
use super::data::ScaffoldReceiptSensitiveEvidence;
use super::data::ScaffoldReviewStatus;
use super::data::ScaffoldRevision;
use super::data::ScaffoldSelectionOutcome;
use super::data::ScaffoldSelectionPlan;
use super::data::ScaffoldSelectionRequest;
use super::data::ScaffoldStrategyKnob;
use super::data::ScaffoldValidationResult;

const MIN_ID_SEGMENT_COUNT: usize = 2;
const ID_SEGMENT_SEPARATOR: char = '.';

/// Computes a domain-separated BLAKE3 fingerprint for scaffold descriptors.
#[must_use]
pub fn blake3_scaffold_fingerprint(label: &str, bytes: &[u8]) -> ScaffoldFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SCAFFOLD_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    hasher.update(bytes);
    ScaffoldFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates a scaffold profile before selection or receipt emission.
///
/// # Errors
///
/// Returns all deterministic profile diagnostics, including malformed IDs,
/// duplicate revisions, unsupported knobs, missing provenance, unreviewed active
/// revision, executable harness attempts, and content-hash failures.
pub fn validate_scaffold_profile(profile: &ScaffoldProfile) -> ScaffoldValidationResult {
    let mut diagnostics = Vec::new();
    require_id(&profile.profile_id, FIELD_PROFILE_ID, &mut diagnostics);
    require_collection(&profile.task_classes, FIELD_TASK_CLASS, &mut diagnostics);
    require_id(&profile.active_revision_id, FIELD_ACTIVE_REVISION, &mut diagnostics);
    require_collection(&profile.route_constraints, FIELD_ROUTE_CONSTRAINTS, &mut diagnostics);
    require_text(&profile.provenance.source, FIELD_PROFILE_PROVENANCE, &mut diagnostics);
    validate_revisions(profile, &mut diagnostics);
    if !profile.enabled {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::DisabledProfile,
            FIELD_PROFILE_ENABLED,
            "scaffold profile is disabled",
        ));
    }
    to_validation_result(diagnostics)
}

/// Selects a compatible scaffold profile before task execution.
#[must_use]
pub fn select_scaffold_profile(
    profiles: &[ScaffoldProfile],
    request: &ScaffoldSelectionRequest,
) -> ScaffoldSelectionPlan {
    let mut diagnostics = Vec::new();
    let fallback_task_class = request.fallback_task_class.as_deref().unwrap_or(DEFAULT_FALLBACK_TASK_CLASS);
    let exact = select_by_task_class(profiles, request, &request.task_class, &mut diagnostics);
    if let Some((profile_id, revision_id)) = exact {
        return ScaffoldSelectionPlan {
            outcome: ScaffoldSelectionOutcome::ExactMatch,
            profile_id: Some(profile_id),
            revision_id: Some(revision_id),
            diagnostics,
        };
    }
    let fallback = select_by_task_class(profiles, request, fallback_task_class, &mut diagnostics);
    if let Some((profile_id, revision_id)) = fallback {
        return ScaffoldSelectionPlan {
            outcome: ScaffoldSelectionOutcome::Fallback,
            profile_id: Some(profile_id),
            revision_id: Some(revision_id),
            diagnostics,
        };
    }
    diagnostics.push(ScaffoldDiagnostic::new(
        ScaffoldDiagnosticKind::NoCompatibleProfile,
        FIELD_SELECTION_PROFILE,
        "no compatible reviewed scaffold profile exists for task class and route labels",
    ));
    ScaffoldSelectionPlan {
        outcome: ScaffoldSelectionOutcome::NoCompatibleProfile,
        profile_id: None,
        revision_id: None,
        diagnostics,
    }
}

/// Applies a host admission outcome to a proposed scaffold revision.
#[must_use]
pub fn apply_scaffold_admission(
    profile: &ScaffoldProfile,
    request: &ScaffoldAdmissionRequest,
    outcome: ScaffoldAdmissionOutcome,
) -> ScaffoldAdmissionDecision {
    let mut diagnostics = Vec::new();
    let base_matches = request.proposal.profile_id == profile.profile_id
        && request.proposal.base_revision_id == profile.active_revision_id;
    let admitted = outcome == ScaffoldAdmissionOutcome::Admitted && base_matches;
    if outcome != ScaffoldAdmissionOutcome::Admitted {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::AdmissionRejected,
            FIELD_BASE_REVISION,
            "host admission did not admit scaffold revision proposal",
        ));
    }
    if outcome == ScaffoldAdmissionOutcome::Admitted && !base_matches {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ActiveRevisionInvalid,
            FIELD_BASE_REVISION,
            "admitted scaffold revision proposal has stale or mismatched base revision",
        ));
    }
    ScaffoldAdmissionDecision {
        outcome,
        persist: admitted,
        activate: admitted,
        future_influence: admitted,
        effect: if admitted {
            ScaffoldAdmissionEffect::PersistAndActivate
        } else {
            ScaffoldAdmissionEffect::RecordOnly
        },
        diagnostics,
    }
}

/// Validates safe scaffold influence receipt metadata.
///
/// # Errors
///
/// Returns diagnostics when required receipt metadata is missing, hashes are not
/// BLAKE3, or unsafe sensitive evidence classes appear in the receipt.
pub fn validate_scaffold_influence_receipt(receipt: &ScaffoldInfluenceReceipt) -> ScaffoldValidationResult {
    let mut diagnostics = Vec::new();
    require_id(&receipt.profile_id, FIELD_RECEIPT_PROFILE, &mut diagnostics);
    require_id(&receipt.revision_id, FIELD_REVISION_ID, &mut diagnostics);
    require_text(&receipt.task_class, FIELD_RECEIPT_TASK, &mut diagnostics);
    require_collection(&receipt.route_labels, FIELD_RECEIPT_ROUTE, &mut diagnostics);
    require_blake3(&receipt.profile_hash, FIELD_CONTENT_HASH, &mut diagnostics);
    require_blake3(&receipt.revision_hash, FIELD_RECEIPT_HASH, &mut diagnostics);
    for sensitive in &receipt.sensitive_evidence {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ReceiptLeak,
            FIELD_RECEIPT_SENSITIVE,
            sensitive_detail(sensitive),
        ));
    }
    to_validation_result(diagnostics)
}

fn active_revision(profile: &ScaffoldProfile) -> Option<&ScaffoldRevision> {
    profile.revisions.iter().find(|revision| revision.revision_id == profile.active_revision_id)
}

fn blank(text: &str) -> bool {
    text.trim().is_empty()
}

fn candidate_profile_ids(profiles: &[ScaffoldProfile]) -> Vec<&ScaffoldProfile> {
    let mut sorted = profiles.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
    sorted
}

fn collect_revision_ids<'a>(
    revisions: &'a [ScaffoldRevision],
    diagnostics: &mut Vec<ScaffoldDiagnostic>,
) -> BTreeSet<&'a str> {
    let mut ids = BTreeSet::new();
    for revision in revisions {
        if blank(&revision.revision_id) {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::MissingField,
                FIELD_REVISION_ID,
                "revision id is required",
            ));
        } else if !valid_id(&revision.revision_id) {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::MalformedId,
                FIELD_REVISION_ID,
                "revision id must contain stable dot-separated lowercase segments",
            ));
        } else if !ids.insert(revision.revision_id.as_str()) {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::DuplicateRevision,
                FIELD_REVISION_ID,
                "revision ids must be unique",
            ));
        }
    }
    ids
}

fn profile_disabled_by_request(profile: &ScaffoldProfile, request: &ScaffoldSelectionRequest) -> bool {
    request.disabled_profile_ids.iter().any(|disabled_id| disabled_id == &profile.profile_id)
}

fn profile_route_matches(profile: &ScaffoldProfile, request: &ScaffoldSelectionRequest) -> bool {
    let route_labels = request.route_capability_labels.iter().map(String::as_str).collect::<BTreeSet<_>>();
    profile.route_constraints.iter().all(|constraint| route_labels.contains(constraint.as_str()))
}

fn profile_task_matches(profile: &ScaffoldProfile, task_class: &str) -> bool {
    profile.task_classes.iter().any(|candidate_task_class| candidate_task_class == task_class)
}

fn require_blake3(fingerprint: &ScaffoldFingerprint, field: &str, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if fingerprint.algorithm != BLAKE3_ALGORITHM {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ContentHashInvalid,
            field,
            "scaffold fingerprints must use BLAKE3",
        ));
    }
    require_text(&fingerprint.value_hex, field, diagnostics);
}

fn require_collection(values: &[String], field: &str, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if values.is_empty() || values.iter().any(|value| blank(value)) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::EmptyCollection,
            field,
            "required collection is empty or contains a blank entry",
        ));
    }
}

fn require_id(id: &str, field: &str, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if blank(id) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::MissingField,
            field,
            "required identifier is missing or blank",
        ));
    } else if !valid_id(id) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::MalformedId,
            field,
            "identifier must contain stable dot-separated lowercase segments",
        ));
    }
}

fn require_text(text: &str, field: &str, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if blank(text) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::MissingField,
            field,
            "required field is missing or blank",
        ));
    }
}

fn revision_reviewed(revision: &ScaffoldRevision) -> bool {
    revision.review_status == ScaffoldReviewStatus::Reviewed
}

fn select_by_task_class(
    profiles: &[ScaffoldProfile],
    request: &ScaffoldSelectionRequest,
    task_class: &str,
    diagnostics: &mut Vec<ScaffoldDiagnostic>,
) -> Option<(String, String)> {
    for profile in candidate_profile_ids(profiles) {
        if !profile_task_matches(profile, task_class) {
            continue;
        }
        if let Err(mut profile_diagnostics) = validate_scaffold_profile(profile) {
            diagnostics.append(&mut profile_diagnostics);
            continue;
        }
        if profile_disabled_by_request(profile, request) {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::DisabledProfile,
                FIELD_PROFILE_ENABLED,
                "host policy disabled scaffold profile",
            ));
            continue;
        }
        if !profile_route_matches(profile, request) {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::RouteMismatch,
                FIELD_ROUTE_CONSTRAINTS,
                "route labels do not satisfy scaffold constraints",
            ));
            continue;
        }
        let Some(revision) = active_revision(profile) else {
            continue;
        };
        return Some((profile.profile_id.clone(), revision.revision_id.clone()));
    }
    None
}

fn sensitive_detail(sensitive: &ScaffoldReceiptSensitiveEvidence) -> &'static str {
    match sensitive {
        ScaffoldReceiptSensitiveEvidence::RawPrompt => "receipt contains raw prompt body",
        ScaffoldReceiptSensitiveEvidence::ScaffoldBody => "receipt contains scaffold body text",
        ScaffoldReceiptSensitiveEvidence::ProviderPayload => "receipt contains provider payload",
        ScaffoldReceiptSensitiveEvidence::Credential => "receipt contains credential-looking value",
        ScaffoldReceiptSensitiveEvidence::FullPath => "receipt contains full host path",
        ScaffoldReceiptSensitiveEvidence::UnboundedToolOutput => "receipt contains unbounded tool output",
    }
}

fn to_validation_result(diagnostics: Vec<ScaffoldDiagnostic>) -> ScaffoldValidationResult {
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

fn valid_id(id: &str) -> bool {
    let segments = id.split(ID_SEGMENT_SEPARATOR).collect::<Vec<_>>();
    segments.len() >= MIN_ID_SEGMENT_COUNT
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
}

fn validate_active_revision(
    profile: &ScaffoldProfile,
    revision_ids: &BTreeSet<&str>,
    diagnostics: &mut Vec<ScaffoldDiagnostic>,
) {
    if !revision_ids.contains(profile.active_revision_id.as_str()) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ActiveRevisionInvalid,
            FIELD_ACTIVE_REVISION,
            "active revision id does not refer to a known revision",
        ));
        return;
    }
    let Some(active) = active_revision(profile) else {
        return;
    };
    if !revision_reviewed(active) {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::UnreviewedActiveRevision,
            FIELD_REVISION_STATUS,
            "active scaffold revision must be reviewed before influence",
        ));
    }
}

fn validate_revision(revision: &ScaffoldRevision, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if revision.strategy_knobs.is_empty() {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::EmptyCollection,
            FIELD_STRATEGY_KNOBS,
            "scaffold revision must declare at least one strategy knob",
        ));
    }
    for knob in &revision.strategy_knobs {
        if let ScaffoldStrategyKnob::Unsupported(_) = knob {
            diagnostics.push(ScaffoldDiagnostic::new(
                ScaffoldDiagnosticKind::UnsupportedKnob,
                FIELD_STRATEGY_KNOBS,
                "scaffold revision declares an unsupported strategy knob",
            ));
        }
    }
    require_blake3(&revision.content_hash, FIELD_CONTENT_HASH, diagnostics);
    if !revision.hash_matches_content {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ContentHashInvalid,
            FIELD_REVISION_HASH_MATCH,
            "scaffold revision content hash does not match reviewed descriptor",
        ));
    }
    if revision.executable_harness {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::ExecutableHarness,
            FIELD_STRATEGY_KNOBS,
            "scaffold revisions must not contain executable model-authored harness behavior",
        ));
    }
}

fn validate_revisions(profile: &ScaffoldProfile, diagnostics: &mut Vec<ScaffoldDiagnostic>) {
    if profile.revisions.is_empty() {
        diagnostics.push(ScaffoldDiagnostic::new(
            ScaffoldDiagnosticKind::EmptyCollection,
            FIELD_REVISION_ID,
            "scaffold profile must include at least one revision",
        ));
        return;
    }
    let revision_ids = collect_revision_ids(&profile.revisions, diagnostics);
    for revision in &profile.revisions {
        validate_revision(revision, diagnostics);
    }
    validate_active_revision(profile, &revision_ids, diagnostics);
}
