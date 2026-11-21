//! Pure consumer checks: the port validator, set qualification, replay,
//! applicability, measures, and review authority.

use alloc::string::String;
use alloc::vec::Vec;

use super::codec::decode_attachment;
use super::data::{
    Applicability, ApplicabilityPolicy, Attachment, AttachmentDocument, ComparisonRefusal,
    CompatibilityDecision, ComponentBasis, ComponentKind, CostPerAccepted, EqualBudgetComparison,
    EvidenceClass, ExpectedSubject, Qualification, QualificationAnswer, QualificationSubject,
    Rejection, RejectionReason, ReplayReport,
};
use super::rules::{BLAKE3_PREFIX, is_digest_hex, is_identity, is_token};
use super::{COMPONENT_IDENTITY_DOMAIN, FIELD_QUALIFICATION};
use crate::eval_feedback::data::{
    EvalFeedbackDiagnostic, EvalFeedbackDiagnosticKind, EvalFeedbackReviewDecision,
};

/// Controls that must be identical for an equal-budget comparison.
const COMPARISON_CONTROLS: [ComponentKind; 11] = [
    ComponentKind::Budget,
    ComponentKind::Corpus,
    ComponentKind::Environment,
    ComponentKind::EvaluationConfig,
    ComponentKind::Isolation,
    ComponentKind::Network,
    ComponentKind::Permissions,
    ComponentKind::SelectionPolicy,
    ComponentKind::TaskProtocol,
    ComponentKind::Topology,
    ComponentKind::Verifier,
];

/// Validates one attachment for one exact expected subject.
///
/// Returns the attachment only when its subject keys equal the expected keys,
/// its task family is the expected family, its class is the required class,
/// no required or current component is unavailable, it is fresh at `now_ms`,
/// and every supplied current component identity is unchanged.
///
/// # Errors
///
/// Returns a closed [`RejectionReason`] equal to Lattice's `NotQualified` reason.
// r[impl agent_harness_runtime.qualification_attachment.validator]
pub fn validate_attachment(
    bytes: &[u8],
    expected: &ExpectedSubject<'_>,
) -> Result<Attachment, Rejection> {
    let attachment = claimed_attachment(bytes, expected)?;
    check_current(&attachment.document, expected)?;
    Ok(attachment)
}

/// Answers the qualification port over every supplied attachment.
///
/// Exact duplicates collapse to one identity. Distinct attachments that pass
/// the claim checks (subject keys, task family, class) and claim the same
/// source are `conflicting`, even when one of them is unusable or stale.
/// Otherwise the most recent attachment that also passes the currency checks
/// qualifies; with none, the furthest-stage rejection is returned. An empty
/// set is `Unavailable`.
#[must_use]
pub fn qualify(attachments: &[&[u8]], expected: &ExpectedSubject<'_>) -> QualificationAnswer {
    let mut claimed: Vec<Attachment> = Vec::new();
    let mut furthest: Option<RejectionReason> = None;
    for bytes in attachments {
        match claimed_attachment(bytes, expected) {
            Ok(attachment) => {
                if claimed
                    .iter()
                    .all(|seen| seen.identity != attachment.identity)
                {
                    claimed.push(attachment);
                }
            }
            Err(rejection) => furthest = furthest.max(Some(rejection.reason)),
        }
    }
    if !conflicting_sources(&claimed).is_empty() {
        return QualificationAnswer::NotQualified(RejectionReason::Conflicting);
    }
    let mut chosen: Option<&Attachment> = None;
    for attachment in &claimed {
        match check_current(&attachment.document, expected) {
            Ok(()) if chosen.is_none_or(|best| recency(attachment) > recency(best)) => {
                chosen = Some(attachment);
            }
            Ok(()) => {}
            Err(rejection) => furthest = furthest.max(Some(rejection.reason)),
        }
    }
    match (chosen, furthest) {
        (Some(chosen), _) => QualificationAnswer::Qualified(qualification(chosen)),
        (None, Some(reason)) => QualificationAnswer::NotQualified(reason),
        (None, None) => QualificationAnswer::Unavailable,
    }
}

/// Replays stored attachment bytes: decodes and checks them only. No model,
/// verifier, or evaluation runs, and missing facts stay missing.
#[must_use]
// r[impl agent_harness_runtime.qualification_attachment.replay]
pub fn replay_attachments(attachments: &[&[u8]]) -> ReplayReport {
    let mut report = ReplayReport::default();
    let mut valid: Vec<Attachment> = Vec::new();
    for (index, bytes) in (0_u64..).zip(attachments) {
        match decode_attachment(bytes) {
            Ok(attachment)
                if valid
                    .iter()
                    .any(|seen| seen.identity == attachment.identity) =>
            {
                report.duplicates = report.duplicates.saturating_add(1);
            }
            Ok(attachment) => {
                report.accepted.push(attachment.identity.clone());
                valid.push(attachment);
            }
            Err(rejection) => report.rejected.push((index, rejection.reason)),
        }
    }
    report.conflicting_sources = conflicting_sources(&valid);
    report
}

/// Classifies one attachment against the complete current subject.
///
/// Any changed component, profile revision, or expired freshness makes the
/// evidence historical. Reuse under a different revision or component needs an
/// explicit decision scoped to this exact attachment, revision pair, task
/// family, and class that waives every changed component and no evaluator
/// component; such reuse never strengthens the claims and is never current.
#[must_use]
// r[impl agent_harness_runtime.qualification_attachment.applicability]
pub fn classify_applicability(
    attachment: &Attachment,
    current: &QualificationSubject,
    policy: &ApplicabilityPolicy<'_>,
    decision: Option<&CompatibilityDecision>,
) -> Applicability {
    let evidence = &attachment.document.subject;
    if evidence.lattice_definition_identity != current.lattice_definition_identity {
        return Applicability::Inapplicable(RejectionReason::SubjectMismatch);
    }
    if evidence.task_family != current.task_family {
        return Applicability::Inapplicable(RejectionReason::ScopeMismatch);
    }
    if let Some(component) = policy
        .required
        .iter()
        .copied()
        .find(|kind| unavailable(evidence, *kind) || unavailable(current, *kind))
    {
        return Applicability::Unusable { component };
    }
    let changed = changed_components(evidence, current);
    let revision_changed = evidence.lattice_profile_revision != current.lattice_profile_revision;
    let provenance = &attachment.document.provenance;
    let expired =
        policy.now_ms >= provenance.valid_until_ms || policy.now_ms < provenance.observed_at_ms;
    if changed.is_empty() && !revision_changed && !expired {
        return Applicability::Current;
    }
    match decision {
        Some(decision) if !expired && decision_covers(decision, attachment, current, &changed) => {
            Applicability::CompatibleReuse {
                decision_identity: decision.decision_identity.clone(),
                changed,
            }
        }
        _ => Applicability::Historical {
            changed,
            revision_changed,
            expired,
        },
    }
}

/// Cost per independently accepted completion; undefined without an accepted
/// completion or with any unknown cost.
#[must_use]
pub fn cost_per_accepted(attachment: &Attachment) -> CostPerAccepted {
    let document = &attachment.document;
    let (Some(measures), Some(outcomes)) = (&document.measures, &document.outcomes) else {
        return CostPerAccepted::NotApplicable;
    };
    if outcomes.accepted == 0 {
        return CostPerAccepted::NoAcceptedCompletion;
    }
    let costs = &measures.costs;
    let known_total = [costs.model, costs.tool, costs.manager, costs.verifier]
        .into_iter()
        .try_fold(0_u64, |total, cost| total.checked_add(cost?));
    match known_total {
        Some(known_total) if !costs.unknown_remainder => CostPerAccepted::Defined {
            known_total,
            accepted: outcomes.accepted,
        },
        _ => CostPerAccepted::UnknownCost,
    }
}

/// Compares two task-family evaluations over identical controls by
/// independently accepted completions only.
///
/// # Errors
///
/// Refuses non-evaluations and any difference in task family, denominator, or
/// control component (including unavailable controls).
// r[impl agent_harness_runtime.qualification_attachment.measures]
pub fn compare_equal_budget(
    baseline: &Attachment,
    candidate: &Attachment,
) -> Result<EqualBudgetComparison, ComparisonRefusal> {
    let (base, cand) = (&baseline.document, &candidate.document);
    let evaluation = |document: &AttachmentDocument| {
        document.evidence.class == EvidenceClass::TaskFamilyEvaluation
    };
    let (Some(base_outcomes), Some(cand_outcomes)) = (base.outcomes, cand.outcomes) else {
        return Err(ComparisonRefusal::NotEvaluation);
    };
    if !evaluation(base) || !evaluation(cand) {
        return Err(ComparisonRefusal::NotEvaluation);
    }
    if base.subject.task_family != cand.subject.task_family
        || base_outcomes.planned != cand_outcomes.planned
    {
        return Err(ComparisonRefusal::UnmatchedControls(None));
    }
    if let Some(kind) = COMPARISON_CONTROLS.into_iter().find(|kind| {
        let identity = |subject: &QualificationSubject| {
            subject
                .component(*kind)
                .and_then(|component| component.identity.clone())
        };
        let base_identity = identity(&base.subject);
        base_identity.is_none() || base_identity != identity(&cand.subject)
    }) {
        return Err(ComparisonRefusal::UnmatchedControls(Some(kind)));
    }
    let (baseline, candidate) = (base_outcomes.accepted, cand_outcomes.accepted);
    Ok(if candidate > baseline {
        EqualBudgetComparison::CandidateMoreAccepted {
            baseline,
            candidate,
        }
    } else {
        EqualBudgetComparison::NoAcceptedGain {
            baseline,
            candidate,
        }
    })
}

/// Review authority of a qualification. Current evidence may request separate
/// review or admission; no attachment activates a revision or changes runtime
/// selection policy.
#[must_use]
// r[impl agent_harness_runtime.qualification_attachment.authority]
pub fn review_qualification(applicability: &Applicability) -> EvalFeedbackReviewDecision {
    let current = matches!(applicability, Applicability::Current);
    let diagnostics = if current {
        Vec::new()
    } else {
        alloc::vec![EvalFeedbackDiagnostic::new(
            EvalFeedbackDiagnosticKind::FreshnessMismatch,
            FIELD_QUALIFICATION,
            "qualification evidence is not current for the complete subject",
        )]
    };
    EvalFeedbackReviewDecision {
        can_request_scaffold_admission: current,
        activates_scaffold: false,
        changes_runtime_selection_policy: false,
        diagnostics,
    }
}

/// `blake3:` identity of one subject component's canonical bytes.
#[must_use]
pub fn component_identity(kind: ComponentKind, bytes: &[u8]) -> String {
    framed_identity(&[
        COMPONENT_IDENTITY_DOMAIN.as_bytes(),
        kind.as_str().as_bytes(),
        bytes,
    ])
}

/// Bytes of one length frame: the part length as a little-endian `u64`.
const LENGTH_FRAME_BYTES: usize = core::mem::size_of::<u64>();

// Every `usize` fits the `u64` length frame, so each length is framed exactly
// and no conversion can fail.
const _: () = assert!(usize::BITS <= u64::BITS);

/// `blake3:` identity over length-framed parts.
#[must_use]
pub fn framed_identity(parts: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(&length_frame(part.len()));
        hasher.update(part);
    }
    alloc::format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

/// `length` as a little-endian `u64`, zero-extended from `usize`.
fn length_frame(length: usize) -> [u8; LENGTH_FRAME_BYTES] {
    let mut frame = [0_u8; LENGTH_FRAME_BYTES];
    for (slot, byte) in frame.iter_mut().zip(length.to_le_bytes()) {
        *slot = byte;
    }
    frame
}

fn qualification(attachment: &Attachment) -> Qualification {
    let evidence = &attachment.document.evidence;
    Qualification {
        qualification_ref: attachment.identity.clone(),
        evidence_class: evidence.class,
        protocol_identity: evidence.protocol_identity.clone(),
        verifier_identity: evidence.verifier_identity.clone(),
        observed_at_ms: attachment.document.provenance.observed_at_ms,
        provenance_label: attachment.document.provenance.label,
    }
}

/// Ordering key of current attachments: the greatest is the most recent,
/// with ties broken by identity.
fn recency(attachment: &Attachment) -> (u64, &str) {
    (
        attachment.document.provenance.observed_at_ms,
        &attachment.identity,
    )
}

/// Decodes `bytes` and applies the claim checks only.
fn claimed_attachment(
    bytes: &[u8],
    expected: &ExpectedSubject<'_>,
) -> Result<Attachment, Rejection> {
    check_expected(expected)?;
    let attachment = decode_attachment(bytes)?;
    check_claim(&attachment.document, expected)?;
    Ok(attachment)
}

fn check_expected(expected: &ExpectedSubject<'_>) -> Result<(), Rejection> {
    let spelled = is_identity(expected.lattice_definition_identity)
        && is_digest_hex(expected.lattice_profile_revision)
        && is_token(expected.task_family)
        && expected
            .current_components
            .iter()
            .all(|component| component.identity.is_none_or(is_identity));
    if spelled {
        Ok(())
    } else {
        Err(Rejection::new(RejectionReason::Malformed, "expected"))
    }
}

/// Claim checks: the attachment claims the exact subject keys, task family,
/// and class.
fn check_claim(
    document: &AttachmentDocument,
    expected: &ExpectedSubject<'_>,
) -> Result<(), Rejection> {
    let subject = &document.subject;
    let reject = |reason, field| Err(Rejection::new(reason, field));
    if subject.lattice_definition_identity != expected.lattice_definition_identity
        || subject.lattice_profile_revision != expected.lattice_profile_revision
    {
        return reject(RejectionReason::SubjectMismatch, "subject");
    }
    if subject.task_family != expected.task_family {
        return reject(RejectionReason::ScopeMismatch, "subject.task_family");
    }
    if document.evidence.class != expected.required_class {
        return reject(RejectionReason::InsufficientClass, "evidence.class");
    }
    Ok(())
}

/// Currency checks: no required or current component is unavailable, the
/// evidence is fresh at `now_ms`, and no supplied current component changed.
fn check_current(
    document: &AttachmentDocument,
    expected: &ExpectedSubject<'_>,
) -> Result<(), Rejection> {
    let subject = &document.subject;
    let reject = |reason, field| Err(Rejection::new(reason, field));
    let required_missing = expected
        .required_components
        .iter()
        .any(|kind| unavailable(subject, *kind));
    let currently_missing = expected
        .current_components
        .iter()
        .any(|component| component.identity.is_none());
    if required_missing || currently_missing || expected.now_ms < document.provenance.observed_at_ms
    {
        return reject(RejectionReason::Unusable, "subject.components");
    }
    let changed = expected.current_components.iter().any(|component| {
        subject
            .component(component.kind)
            .and_then(|evidence| evidence.identity.as_deref())
            != component.identity
    });
    if changed || expected.now_ms >= document.provenance.valid_until_ms {
        return reject(RejectionReason::Stale, "provenance");
    }
    Ok(())
}

fn unavailable(subject: &QualificationSubject, kind: ComponentKind) -> bool {
    subject
        .component(kind)
        .is_none_or(|component| component.basis == ComponentBasis::Unavailable)
}

fn changed_components(
    evidence: &QualificationSubject,
    current: &QualificationSubject,
) -> Vec<ComponentKind> {
    let mut changed: Vec<ComponentKind> = ComponentKind::ALL
        .into_iter()
        .filter(|kind| {
            let identity = |subject: &QualificationSubject| {
                subject
                    .component(*kind)
                    .and_then(|component| component.identity.clone())
            };
            identity(evidence) != identity(current)
        })
        .collect();
    if evidence.model != current.model && !changed.contains(&ComponentKind::ModelRoute) {
        changed.push(ComponentKind::ModelRoute);
    }
    changed
}

fn decision_covers(
    decision: &CompatibilityDecision,
    attachment: &Attachment,
    current: &QualificationSubject,
    changed: &[ComponentKind],
) -> bool {
    let evidence = &attachment.document.subject;
    is_identity(&decision.decision_identity)
        && decision.attachment_identity == attachment.identity
        && decision.from_profile_revision == evidence.lattice_profile_revision
        && decision.to_profile_revision == current.lattice_profile_revision
        && decision.task_family == evidence.task_family
        && decision.evidence_class == attachment.document.evidence.class
        && changed.iter().all(|kind| decision.waived.contains(kind))
        && !decision
            .waived
            .iter()
            .any(|kind| ComponentKind::EVALUATOR.contains(kind))
}

/// Sources claimed differently for the same subject keys, task family and class.
fn conflicting_sources(attachments: &[Attachment]) -> Vec<String> {
    let same_claim = |left: &Attachment, right: &Attachment| {
        let (left, right) = (&left.document, &right.document);
        left.evidence.source_identity == right.evidence.source_identity
            && left.evidence.class == right.evidence.class
            && left.subject.lattice_definition_identity == right.subject.lattice_definition_identity
            && left.subject.lattice_profile_revision == right.subject.lattice_profile_revision
            && left.subject.task_family == right.subject.task_family
    };
    let mut sources: Vec<String> = Vec::new();
    for (index, attachment) in attachments.iter().enumerate() {
        let source = &attachment.document.evidence.source_identity;
        let repeated = attachments
            .iter()
            .skip(index.saturating_add(1))
            .any(|other| same_claim(attachment, other));
        if repeated && !sources.contains(source) {
            sources.push(source.clone());
        }
    }
    sources
}
