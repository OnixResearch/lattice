// Receipt validation keeps explicit metadata predicates and owner-qualified DTO imports.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    non_trait_imports
)]
#![allow(clippy::missing_errors_doc)]

#[allow(unused_imports)]
use crate::prelude::*;

#[allow(unused_imports)]
use super::core::hash_parts;
#[allow(clippy::wildcard_imports)]
use super::data::*;

const REDACTION_CLASS: &str = "metadata_only_redaction_first";

/// Project one metadata-only explorer receipt.
pub fn project_branch_explorer_receipt(
    input: &BranchExplorerReceiptInput<'_>,
) -> Result<BranchExplorerReceipt, BranchExplorerError> {
    reject_sensitive(input.sensitive)?;
    reject_claims(input.claims)?;
    if input.policy.policy_id != input.projection.policy_id {
        return Err(BranchExplorerError::new(
            BranchExplorerIssue::StaleRevision,
            "receipt.policy_id",
        ));
    }
    validate_optional_bindings(input)?;
    if let Some(action_id) = input.tui_action_id {
        validate_safe_metadata(action_id, "tui_action_id")?;
    }
    let node_count = u32::try_from(input.projection.nodes.len()).map_err(|_error| {
        BranchExplorerError::new(BranchExplorerIssue::BoundExceeded, "receipt.node_count")
    })?;
    let operation_class = input
        .operation
        .map(|operation| String::from(operation.operation.as_str()));
    let result_class = input
        .operation
        .map(|operation| String::from(operation.status.as_str()));
    let comparison_id = input
        .comparison
        .map(|comparison| comparison.comparison_id.clone());
    let equivalence_id = input
        .equivalence
        .map(|equivalence| equivalence.equivalence_id.clone());
    let tui_action_id = input.tui_action_id.map(String::from);
    let node_count_text = node_count.to_string();
    let omitted_count_text = input.projection.truncation.omitted_node_count.to_string();
    let receipt_id = hash_parts(
        BRANCH_EXPLORER_RECEIPT_DOMAIN,
        [
            input.policy.policy_id.as_str(),
            input.projection.projection_id.as_str(),
            input.projection.session_id.as_str(),
            input.projection.revision_id.as_str(),
            node_count_text.as_str(),
            omitted_count_text.as_str(),
            operation_class.as_deref().unwrap_or("none"),
            result_class.as_deref().unwrap_or("none"),
            comparison_id.as_deref().unwrap_or("none"),
            equivalence_id.as_deref().unwrap_or("none"),
            tui_action_id.as_deref().unwrap_or("none"),
            input.projection.external_state.as_str(),
            REDACTION_CLASS,
        ],
    );
    Ok(BranchExplorerReceipt {
        schema: String::from(BRANCH_EXPLORER_RECEIPT_SCHEMA),
        receipt_id,
        policy_id: input.policy.policy_id.clone(),
        projection_id: input.projection.projection_id.clone(),
        session_id: input.projection.session_id.clone(),
        revision_id: input.projection.revision_id.clone(),
        node_count,
        omitted_node_count: input.projection.truncation.omitted_node_count,
        operation_class,
        result_class,
        comparison_id,
        equivalence_id,
        tui_action_id,
        external_state: input.projection.external_state,
        redaction: String::from(REDACTION_CLASS),
        non_claims: input.policy.non_claims.clone(),
    })
}

fn validate_optional_bindings(
    input: &BranchExplorerReceiptInput<'_>,
) -> Result<(), BranchExplorerError> {
    if let Some(operation) = input.operation {
        let exact = operation.session_id == input.projection.session_id
            && operation.expected_revision_id == input.projection.revision_id
            && operation.policy_id == input.policy.policy_id;
        if !exact {
            return Err(BranchExplorerError::new(
                BranchExplorerIssue::StaleRevision,
                "receipt.operation",
            ));
        }
    }
    if let Some(comparison) = input.comparison {
        let exact = comparison.session_id == input.projection.session_id
            && comparison.revision_id == input.projection.revision_id;
        if !exact {
            return Err(BranchExplorerError::new(
                BranchExplorerIssue::CrossSession,
                "receipt.comparison",
            ));
        }
    }
    if let Some(equivalence) = input.equivalence {
        if !equivalence.branches_preserved {
            return Err(BranchExplorerError::new(
                BranchExplorerIssue::DestructiveAction,
                "receipt.equivalence",
            ));
        }
    }
    Ok(())
}

fn reject_sensitive(
    sensitive: BranchExplorerSensitiveDraft<'_>,
) -> Result<(), BranchExplorerError> {
    let leaks = sensitive.transcript_text.is_some()
        || sensitive.reasoning.is_some()
        || sensitive.tool_body.is_some()
        || sensitive.provider_payload.is_some()
        || sensitive.credential.is_some()
        || sensitive.environment_value.is_some()
        || sensitive.physical_path.is_some()
        || sensitive.raw_ui_state.is_some();
    if leaks {
        return Err(BranchExplorerError::new(
            BranchExplorerIssue::ReceiptLeak,
            "receipt.sensitive",
        ));
    }
    Ok(())
}

fn reject_claims(claims: BranchExplorerClaimDraft) -> Result<(), BranchExplorerError> {
    let overclaims = claims.rollback
        || claims.world_isolation
        || claims.semantic_quality
        || claims.completion
        || claims.production_readiness
        || claims.release_eligibility;
    if overclaims {
        return Err(BranchExplorerError::new(
            BranchExplorerIssue::Overclaim,
            "receipt.claims",
        ));
    }
    Ok(())
}

fn validate_safe_metadata(value: &str, field: &'static str) -> Result<(), BranchExplorerError> {
    let safe = !value.is_empty()
        && !value.chars().any(char::is_control)
        && !value.contains("/home/")
        && !value.contains("raw_prompt:");
    if !safe {
        return Err(BranchExplorerError::new(
            BranchExplorerIssue::ReceiptLeak,
            field,
        ));
    }
    Ok(())
}
