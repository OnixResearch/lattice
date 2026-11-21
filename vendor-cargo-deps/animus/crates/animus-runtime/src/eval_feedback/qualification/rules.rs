//! Structural rules every attachment satisfies before any consumer reads it.

use alloc::string::String;

use super::data::{
    AttachmentDocument, ComponentBasis, ComponentKind, EvidenceClass, Measures, ModelDeployment,
    ModelFacts, Outcomes, QualificationSubject, Rejection, RejectionReason, Scope, VersionBasis,
};

/// `blake3:` spelling prefix shared with Lattice.
pub(crate) const BLAKE3_PREFIX: &str = "blake3:";
/// Hex digits of a BLAKE3 digest.
const DIGEST_HEX_LEN: usize = 64;
/// Longest token, equal to Lattice's capability name bound.
pub(crate) const MAX_TOKEN_BYTES: usize = 64;
/// Most entries one scope list or the reference list may hold.
pub(crate) const MAX_LIST_ITEMS: usize = 32;
/// Token segments that indicate credentials rather than metadata. A marker
/// counts only at the start of a token or right after a `.`, `:` or `-`, so
/// ordinary words such as `task-family` stay valid.
const CREDENTIAL_MARKERS: [&str; 9] = [
    "sk-",
    "xoxb-",
    "xoxp-",
    "akia",
    "bearer",
    "password",
    "passwd",
    "private-key",
    "api-key",
];

/// Whether `value` is a bare 64-hex BLAKE3 digest.
pub(crate) fn is_digest_hex(value: &str) -> bool {
    value.len() == DIGEST_HEX_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Whether `value` is `blake3:` followed by 64 lowercase hex.
pub(crate) fn is_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_digest_hex)
}

/// Whether `value` follows Lattice's capability name rule and carries no
/// credential-looking fragment.
// r[impl agent_harness_runtime.qualification_attachment.redaction]
pub(crate) fn is_token(value: &str) -> bool {
    let mut bytes = value.bytes();
    let lead = bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let rest = bytes
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._:-".contains(&byte));
    lead && rest && value.len() <= MAX_TOKEN_BYTES && !looks_like_credential(value)
}

fn looks_like_credential(value: &str) -> bool {
    CREDENTIAL_MARKERS.iter().any(|marker| {
        value.match_indices(marker).any(|(index, _)| {
            index == 0
                || value
                    .as_bytes()
                    .get(index.saturating_sub(1))
                    .is_some_and(|byte| b".:-".contains(byte))
        })
    })
}

/// Whether `values` are tokens, strictly ascending (sorted and unique), and bounded.
pub(crate) fn is_token_set(values: &[String]) -> bool {
    values.len() <= MAX_LIST_ITEMS
        && values.iter().all(|value| is_token(value))
        && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn malformed(field: &'static str) -> Rejection {
    Rejection::new(RejectionReason::Malformed, field)
}

fn require(condition: bool, field: &'static str) -> Result<(), Rejection> {
    if condition {
        Ok(())
    } else {
        Err(malformed(field))
    }
}

/// Checks every structural rule of `document`.
pub(crate) fn check_document(document: &AttachmentDocument) -> Result<(), Rejection> {
    check_subject(&document.subject)?;
    check_evidence(document)?;
    check_scope(&document.scope)?;
    let provenance = &document.provenance;
    require(
        is_token(&provenance.producer) && is_token(&provenance.producer_revision),
        "provenance.producer",
    )?;
    require(
        provenance.observed_at_ms < provenance.valid_until_ms,
        "provenance.valid_until_ms",
    )?;
    require(
        document.references.len() <= MAX_LIST_ITEMS
            && document
                .references
                .iter()
                .all(|reference| is_token(&reference.kind) && is_identity(&reference.identity))
            && document.references.windows(2).all(|pair| pair[0] < pair[1]),
        "references",
    )
}

// r[impl agent_harness_runtime.qualification_attachment.subject]
fn check_subject(subject: &QualificationSubject) -> Result<(), Rejection> {
    require(
        is_identity(&subject.lattice_definition_identity),
        "subject.lattice_definition_identity",
    )?;
    require(
        is_digest_hex(&subject.lattice_profile_revision),
        "subject.lattice_profile_revision",
    )?;
    require(is_token(&subject.task_family), "subject.task_family")?;
    require(
        subject.components.len() == ComponentKind::ALL.len()
            && subject
                .components
                .iter()
                .zip(ComponentKind::ALL)
                .all(|(component, kind)| component.kind == kind),
        "subject.components",
    )?;
    for component in &subject.components {
        let present = match component.basis {
            ComponentBasis::Unavailable => component.identity.is_none(),
            ComponentBasis::Declared | ComponentBasis::Observed => {
                component.identity.as_deref().is_some_and(is_identity)
            }
        };
        require(present, "subject.components.identity")?;
    }
    check_model(&subject.model)
}

fn check_model(model: &ModelFacts) -> Result<(), Rejection> {
    let version_ok = model.version.as_deref().is_none_or(is_token);
    let weights_ok = model.weights_identity.as_deref().is_none_or(is_identity);
    require(version_ok && weights_ok, "subject.model")?;
    let has_version = model.version.is_some();
    let has_weights = model.weights_identity.is_some();
    let consistent = match model.version_basis {
        VersionBasis::ProviderDeclared => {
            model.deployment == ModelDeployment::Hosted && has_version && !has_weights
        }
        VersionBasis::VerifiedArtifact => {
            model.deployment != ModelDeployment::Hosted && has_weights
        }
        VersionBasis::Declared => {
            model.deployment != ModelDeployment::Hosted && has_version && !has_weights
        }
        VersionBasis::Unavailable => !has_version && !has_weights,
    };
    require(consistent, "subject.model.version_basis")
}

fn check_evidence(document: &AttachmentDocument) -> Result<(), Rejection> {
    let evidence = &document.evidence;
    require(
        is_identity(&evidence.protocol_identity)
            && is_identity(&evidence.verifier_identity)
            && is_identity(&evidence.source_identity)
            && evidence.corpus_identity.as_deref().is_none_or(is_identity),
        "evidence",
    )?;
    if !evidence.class.has_outcomes() {
        return require(
            document.outcomes.is_none()
                && document.measures.is_none()
                && evidence.corpus_identity.is_none(),
            "outcomes",
        );
    }
    let outcomes = document.outcomes.ok_or_else(|| malformed("outcomes"))?;
    check_outcomes(&outcomes)?;
    require(
        evidence.corpus_identity.is_some(),
        "evidence.corpus_identity",
    )?;
    check_bound(
        document,
        ComponentKind::TaskProtocol,
        &evidence.protocol_identity,
    )?;
    check_bound(
        document,
        ComponentKind::Verifier,
        &evidence.verifier_identity,
    )?;
    if let Some(corpus) = &evidence.corpus_identity {
        check_bound(document, ComponentKind::Corpus, corpus)?;
    }
    match (evidence.class, &document.measures) {
        (EvidenceClass::TaskFamilyEvaluation, Some(measures)) => {
            check_measures(measures, &outcomes)
        }
        (EvidenceClass::ContractConformance, None) => Ok(()),
        _ => Err(malformed("measures")),
    }
}

/// Evaluator identities must be the observed subject components.
fn check_bound(
    document: &AttachmentDocument,
    kind: ComponentKind,
    identity: &str,
) -> Result<(), Rejection> {
    let component = document.subject.component(kind);
    require(
        component.is_some_and(|component| {
            component.basis == ComponentBasis::Observed
                && component.identity.as_deref() == Some(identity)
        }),
        "evidence.component_binding",
    )
}

fn check_outcomes(outcomes: &Outcomes) -> Result<(), Rejection> {
    let sum = [
        outcomes.accepted,
        outcomes.rejected,
        outcomes.failed,
        outcomes.cancelled,
        outcomes.incomplete,
    ]
    .into_iter()
    .try_fold(0_u64, u64::checked_add);
    require(
        outcomes.planned > 0 && sum == Some(outcomes.planned),
        "outcomes.planned",
    )
}

fn check_measures(measures: &Measures, outcomes: &Outcomes) -> Result<(), Rejection> {
    let completed = outcomes.accepted.saturating_add(outcomes.rejected);
    let planned_trials = measures.tasks_declared.checked_mul(measures.repetitions);
    require(
        measures.repetitions > 0
            && measures.tasks_covered <= measures.tasks_declared
            && planned_trials == Some(outcomes.planned)
            && (completed == 0) == (measures.tasks_covered == 0),
        "measures.task_coverage",
    )?;
    let costs = &measures.costs;
    let all_known = [costs.model, costs.tool, costs.manager, costs.verifier]
        .iter()
        .all(Option::is_some);
    require(is_token(&costs.unit), "measures.costs.unit")?;
    require(
        all_known || costs.unknown_remainder,
        "measures.costs.unknown_remainder",
    )
}

fn check_scope(scope: &Scope) -> Result<(), Rejection> {
    require(is_token_set(&scope.partitions), "scope.partitions")?;
    require(
        !scope.limitations.is_empty() && is_token_set(&scope.limitations),
        "scope.limitations",
    )?;
    require(is_token_set(&scope.missing_modes), "scope.missing_modes")
}
