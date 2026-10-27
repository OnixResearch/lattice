#![allow(missing_docs)]

use alloc::{string::String, vec::Vec};

use crate::{
    ATIF_SCHEMA, AtifCounts, AtifDiagnostic, AtifDispositionClass, AtifField, AtifIssueClass,
    AtifLocation, AtifLossClass, AtifProjectionStatus, CODEC_IDENTITY, HARBOR_RELEASE,
    HARBOR_REVISION, SOURCE_COHORT_IDENTITY, ValidatedAtifPolicy, ValidatedTrajectory,
    disposition_class_label, hash_identity, loss_label,
};

/// Canonical metadata fields permitted on ATIF receipts.
pub const REQUIRED_ATIF_RECEIPT_FIELDS: &[&str] = &[
    "cohort",
    "profile_identity",
    "codec_identity",
    "direction",
    "status",
    "semantic_identity",
    "projection_identity",
    "wire_identity",
    "bounded_counts",
    "loss_classes",
    "disposition_classes",
    "non_claims",
    "receipt_identity",
];

/// Canonical bounded non-claims required on ATIF receipts.
pub const REQUIRED_ATIF_NON_CLAIMS: &[&str] = &[
    "not_live_correctness",
    "not_confinement",
    "not_durability",
    "not_exactly_once_effects",
    "not_verifier_soundness",
    "not_artifact_authenticity",
    "not_benchmark_validity",
    "not_training_fitness",
    "not_authoritative_completion",
    "not_production_readiness",
    "not_release_eligibility",
];

/// Receipt operation class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifOperation {
    Validate,
    Import,
    Export,
    Decode,
    Encode,
}

/// Bounded receipt claim. No variant can claim effect success or completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifBoundedClaim {
    ExactCohortMatched,
    CallerBytesDecoded,
    TypedSchemaValidated,
    GraphChecked,
    ProjectionLossReported,
    OutputRevalidated,
}

/// Safe metadata-only ATIF receipt draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifReceiptDraft {
    pub schema: &'static str,
    pub operation: AtifOperation,
    pub receipt_identity: String,
    pub semantic_identity: String,
    pub policy_identity: String,
    pub projection_identity: Option<String>,
    pub wire_input_identity: Option<String>,
    pub wire_output_identity: Option<String>,
    pub codec_identity: &'static str,
    pub source_cohort_identity: &'static str,
    pub harbor_release: &'static str,
    pub harbor_revision: &'static str,
    pub atif_schema: &'static str,
    pub counts: AtifCounts,
    pub projection_status: Option<AtifProjectionStatus>,
    pub loss_classes: Vec<&'static str>,
    pub disposition_classes: Vec<&'static str>,
    pub claims: Vec<AtifBoundedClaim>,
    pub non_claims: Vec<String>,
}

/// Inputs that do not contain trajectory bodies.
#[derive(Clone, Copy)]
pub struct AtifReceiptInput<'a> {
    pub operation: AtifOperation,
    pub validated: &'a ValidatedTrajectory,
    pub projection_identity: Option<&'a str>,
    pub projection_status: Option<AtifProjectionStatus>,
    pub losses: &'a [AtifLossClass],
    pub dispositions: &'a [AtifDispositionClass],
    pub wire_input_identity: Option<&'a str>,
    pub wire_output_identity: Option<&'a str>,
    pub claims: &'a [AtifBoundedClaim],
}

/// Build a safe receipt from validated metadata.
///
/// # Errors
/// Rejects stale policy, excessive loss classes, duplicate claims, or missing non-claims.
pub fn build_atif_receipt(
    input: AtifReceiptInput<'_>,
    policy: &ValidatedAtifPolicy,
) -> Result<AtifReceiptDraft, AtifDiagnostic> {
    if input.validated.policy_identity() != policy.identity()
        || input.losses.len() > policy.value().bounds.max_loss_classes
        || input.dispositions.len() > policy.value().bounds.max_loss_classes
        || has_duplicate_dispositions(input.dispositions)
        || has_duplicate_claims(input.claims)
        || !required_non_claims_present(&policy.value().required_non_claims)
    {
        return Err(AtifDiagnostic::new(
            AtifIssueClass::Evidence,
            AtifLocation::Receipt,
            AtifField::Evidence,
        ));
    }
    let loss_classes = input
        .losses
        .iter()
        .map(|loss| loss_label(*loss))
        .collect::<Vec<_>>();
    let loss_text = loss_classes.join(":");
    let disposition_classes = input
        .dispositions
        .iter()
        .map(|value| disposition_class_label(*value))
        .collect::<Vec<_>>();
    let disposition_text = disposition_classes.join(":");
    let claims_text = input
        .claims
        .iter()
        .map(|claim| claim_label(*claim))
        .collect::<Vec<_>>()
        .join(":");
    let receipt_identity = hash_identity(
        "animus.harbor-atif-receipt.v1",
        &[
            operation_label(input.operation).as_bytes(),
            input.validated.semantic_identity().as_bytes(),
            policy.identity().as_bytes(),
            input.projection_identity.unwrap_or_default().as_bytes(),
            input.wire_input_identity.unwrap_or_default().as_bytes(),
            input.wire_output_identity.unwrap_or_default().as_bytes(),
            loss_text.as_bytes(),
            disposition_text.as_bytes(),
            claims_text.as_bytes(),
        ],
    );
    Ok(AtifReceiptDraft {
        schema: "animus.harbor-atif-receipt.v1",
        operation: input.operation,
        receipt_identity,
        semantic_identity: input.validated.semantic_identity().into(),
        policy_identity: policy.identity().into(),
        projection_identity: input.projection_identity.map(String::from),
        wire_input_identity: input.wire_input_identity.map(String::from),
        wire_output_identity: input.wire_output_identity.map(String::from),
        codec_identity: CODEC_IDENTITY,
        source_cohort_identity: SOURCE_COHORT_IDENTITY,
        harbor_release: HARBOR_RELEASE,
        harbor_revision: HARBOR_REVISION,
        atif_schema: ATIF_SCHEMA,
        counts: input.validated.counts(),
        projection_status: input.projection_status,
        loss_classes,
        disposition_classes,
        claims: input.claims.to_vec(),
        non_claims: policy.value().required_non_claims.clone(),
    })
}

pub(crate) fn required_receipt_fields_present(values: &[String]) -> bool {
    REQUIRED_ATIF_RECEIPT_FIELDS
        .iter()
        .all(|required| values.iter().any(|value| value == required))
}

pub(crate) fn required_non_claims_present(values: &[String]) -> bool {
    REQUIRED_ATIF_NON_CLAIMS
        .iter()
        .all(|required| values.iter().any(|value| value == required))
}

fn has_duplicate_dispositions(values: &[AtifDispositionClass]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

fn has_duplicate_claims(values: &[AtifBoundedClaim]) -> bool {
    values
        .iter()
        .enumerate()
        .any(|(index, value)| values[..index].contains(value))
}

const fn operation_label(value: AtifOperation) -> &'static str {
    match value {
        AtifOperation::Validate => "validate",
        AtifOperation::Import => "import",
        AtifOperation::Export => "export",
        AtifOperation::Decode => "decode",
        AtifOperation::Encode => "encode",
    }
}

const fn claim_label(value: AtifBoundedClaim) -> &'static str {
    match value {
        AtifBoundedClaim::ExactCohortMatched => "exact_cohort_matched",
        AtifBoundedClaim::CallerBytesDecoded => "caller_bytes_decoded",
        AtifBoundedClaim::TypedSchemaValidated => "typed_schema_validated",
        AtifBoundedClaim::GraphChecked => "graph_checked",
        AtifBoundedClaim::ProjectionLossReported => "projection_loss_reported",
        AtifBoundedClaim::OutputRevalidated => "output_revalidated",
    }
}
