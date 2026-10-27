use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake3::Hasher;

use super::codec::{update_str, update_u32};
use super::data::{
    MODEL_REQUEST_SNAPSHOT_NON_CLAIMS, MODEL_REQUEST_SNAPSHOT_RECEIPT_IDENTITY_DOMAIN,
    MODEL_REQUEST_SNAPSHOT_RECEIPT_SCHEMA, MODEL_REQUEST_SNAPSHOT_REDACTION, ModelRequestSnapshot,
    ModelRequestSnapshotError, ModelRequestSnapshotIssue, ModelRequestSnapshotReceipt,
    ModelRequestSnapshotReceiptSensitiveDraft, ModelRequestSnapshotReceiptStatus,
};

// r[impl agent_harness_runtime.model_request_snapshot.evidence]
/// Projects a bounded metadata-only public snapshot receipt.
///
/// # Errors
///
/// Returns an unsafe-evidence issue when private request material is supplied or
/// a count cannot fit the bounded receipt representation.
pub fn project_model_request_snapshot_receipt(
    snapshot: &ModelRequestSnapshot,
    status: ModelRequestSnapshotReceiptStatus,
    sensitive: ModelRequestSnapshotReceiptSensitiveDraft<'_>,
) -> Result<ModelRequestSnapshotReceipt, ModelRequestSnapshotError> {
    reject_sensitive(sensitive)?;
    let message_count = u32::try_from(snapshot.message_count())
        .map_err(|_error| error(ModelRequestSnapshotIssue::BoundExceeded, "message_count"))?;
    let system_section_count =
        u32::try_from(snapshot.system_section_count()).map_err(|_error| {
            error(
                ModelRequestSnapshotIssue::BoundExceeded,
                "system_section_count",
            )
        })?;
    let contribution_count = u32::try_from(
        snapshot.selected_context_contribution_identities().len(),
    )
    .map_err(|_error| {
        error(
            ModelRequestSnapshotIssue::BoundExceeded,
            "contribution_count",
        )
    })?;
    let tool_count = u32::try_from(snapshot.tool_count())
        .map_err(|_error| error(ModelRequestSnapshotIssue::BoundExceeded, "tool_count"))?;
    let non_claims = MODEL_REQUEST_SNAPSHOT_NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
    let receipt_identity = receipt_identity(&ReceiptIdentityInput {
        snapshot,
        status,
        message_count,
        system_section_count,
        contribution_count,
        tool_count,
        non_claims: non_claims.as_slice(),
    });
    Ok(ModelRequestSnapshotReceipt {
        schema: MODEL_REQUEST_SNAPSHOT_RECEIPT_SCHEMA,
        snapshot_identity: String::from(snapshot.snapshot_identity()),
        canonical_request_identity: String::from(snapshot.canonical_request_identity()),
        route_identity: String::from(snapshot.route_identity()),
        model_invocation_identity: String::from(snapshot.model_invocation_identity()),
        request_context_plan_identity: String::from(snapshot.request_context_plan_identity()),
        tool_schema_set_identity: String::from(snapshot.tool_schema_set_identity()),
        output_control_identity: String::from(snapshot.output_control_identity()),
        budget_identity: String::from(snapshot.budget_identity()),
        authority_seal_identity: String::from(snapshot.authority_seal_identity()),
        adapter_cohort_identity: String::from(snapshot.adapter_cohort_identity()),
        message_count,
        system_section_count,
        contribution_count,
        tool_count,
        status,
        redaction_marker: MODEL_REQUEST_SNAPSHOT_REDACTION,
        non_claims,
        receipt_identity,
    })
}

/// Validates every public receipt field against the private snapshot.
///
/// # Errors
///
/// Returns an unsafe-evidence issue when any field or identity was changed.
pub fn validate_model_request_snapshot_receipt(
    snapshot: &ModelRequestSnapshot,
    receipt: &ModelRequestSnapshotReceipt,
) -> Result<(), ModelRequestSnapshotError> {
    let expected = project_model_request_snapshot_receipt(
        snapshot,
        receipt.status,
        ModelRequestSnapshotReceiptSensitiveDraft::default(),
    )?;
    if &expected == receipt {
        Ok(())
    } else {
        Err(error(
            ModelRequestSnapshotIssue::UnsafeEvidence,
            "snapshot_receipt",
        ))
    }
}

fn reject_sensitive(
    sensitive: ModelRequestSnapshotReceiptSensitiveDraft<'_>,
) -> Result<(), ModelRequestSnapshotError> {
    if [
        sensitive.raw_prompt,
        sensitive.tool_schema,
        sensitive.contribution_body,
        sensitive.credential,
        sensitive.provider_payload,
        sensitive.private_path,
        sensitive.raw_error,
    ]
    .iter()
    .any(Option::is_some)
    {
        return Err(error(
            ModelRequestSnapshotIssue::UnsafeEvidence,
            "snapshot_receipt",
        ));
    }
    Ok(())
}

struct ReceiptIdentityInput<'input> {
    snapshot: &'input ModelRequestSnapshot,
    status: ModelRequestSnapshotReceiptStatus,
    message_count: u32,
    system_section_count: u32,
    contribution_count: u32,
    tool_count: u32,
    non_claims: &'input [String],
}

fn receipt_identity(input: &ReceiptIdentityInput<'_>) -> String {
    let mut hasher = Hasher::new();
    update_str(&mut hasher, MODEL_REQUEST_SNAPSHOT_RECEIPT_IDENTITY_DOMAIN);
    update_str(&mut hasher, input.snapshot.snapshot_identity());
    update_str(&mut hasher, input.snapshot.canonical_request_identity());
    update_str(&mut hasher, input.snapshot.route_identity());
    update_str(&mut hasher, input.snapshot.model_invocation_identity());
    update_str(&mut hasher, input.snapshot.request_context_plan_identity());
    update_str(&mut hasher, input.snapshot.tool_schema_set_identity());
    update_str(&mut hasher, input.snapshot.output_control_identity());
    update_str(&mut hasher, input.snapshot.budget_identity());
    update_str(&mut hasher, input.snapshot.authority_seal_identity());
    update_str(&mut hasher, input.snapshot.adapter_cohort_identity());
    update_u32(&mut hasher, input.message_count);
    update_u32(&mut hasher, input.system_section_count);
    update_u32(&mut hasher, input.contribution_count);
    update_u32(&mut hasher, input.tool_count);
    update_str(&mut hasher, input.status.as_str());
    update_str(&mut hasher, MODEL_REQUEST_SNAPSHOT_REDACTION);
    for non_claim in input.non_claims {
        update_str(&mut hasher, non_claim);
    }
    hasher.finalize().to_hex().to_string()
}

const fn error(issue: ModelRequestSnapshotIssue, field: &'static str) -> ModelRequestSnapshotError {
    ModelRequestSnapshotError::new(issue, field)
}
