//! Request-context retention denial and metadata-only receipts.

type Contribution = super::data::Contribution;
type ContributionIssue = super::data::ContributionIssue;
type ContributionPlan = super::data::ContributionPlan;
type ContributionReceipt = super::data::ContributionReceipt;
type ContributionReceiptEntry = super::data::ContributionReceiptEntry;
type ContributionReceiptSensitiveDraft<'a> = super::data::ContributionReceiptSensitiveDraft<'a>;
type ReceiptIdentity = super::data::ReceiptIdentity;
type RetentionSink = super::data::RetentionSink;

const RECEIPT_HASH_DOMAIN: &str = "animus.request-context-receipt.v1";
const HASH_SEPARATOR: &[u8] = b"\0";
const CLAIM_ORDERED_SELECTION: &str = "deterministic-selection";
const CLAIM_EXACT_BINDING: &str = "exact-invocation-binding";
const CLAIM_METADATA_ONLY: &str = "metadata-only-evidence";
const ALLOWED_CLAIMS: &[&str] = &[
    CLAIM_ORDERED_SELECTION,
    CLAIM_EXACT_BINDING,
    CLAIM_METADATA_ONLY,
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "prompt-correctness",
    "provider-delivery",
    "model-compliance",
    "skill-quality",
    "task-completion",
    "production-readiness",
];

/// Rejects durable storage of request-context payloads.
///
/// # Errors
///
/// Always returns retention denied. Callers may persist only metadata receipts.
// r[impl agent_harness_runtime.request_context_contributions.retention]
pub const fn admit_payload_retention(
    _contribution: &Contribution,
    _sink: RetentionSink,
) -> Result<(), ContributionIssue> {
    Err(ContributionIssue::RetentionDenied)
}

/// Projects a bounded metadata-only contribution receipt.
///
/// # Errors
///
/// Returns receipt leak when a caller supplies any sensitive candidate field.
// r[impl agent_harness_runtime.request_context_contributions.evidence]
pub fn project_contribution_receipt(
    plan: &ContributionPlan,
    sensitive: ContributionReceiptSensitiveDraft<'_>,
) -> Result<ContributionReceipt, ContributionIssue> {
    reject_sensitive_receipt_fields(sensitive)?;
    let selected = plan.selected.iter().map(receipt_entry).collect::<Vec<_>>();
    let claims = ALLOWED_CLAIMS.iter().map(ToString::to_string).collect();
    let non_claims = REQUIRED_NON_CLAIMS
        .iter()
        .map(ToString::to_string)
        .collect();
    let receipt_id = receipt_identity(plan, selected.as_slice());
    Ok(ContributionReceipt {
        receipt_id,
        plan_id: plan.plan_id.clone(),
        selected,
        omissions: plan.omissions.clone(),
        selected_budget: plan.selected_budget,
        claims,
        non_claims,
    })
}

/// Validates bounded receipt claims and non-claims.
///
/// # Errors
///
/// Returns overclaim or missing-non-claim without echoing caller values.
pub fn validate_contribution_receipt(
    receipt: &ContributionReceipt,
) -> Result<(), ContributionIssue> {
    if receipt
        .claims
        .iter()
        .any(|claim| !ALLOWED_CLAIMS.contains(&claim.as_str()))
    {
        return Err(ContributionIssue::ReceiptOverclaim);
    }
    if REQUIRED_NON_CLAIMS
        .iter()
        .any(|required| !receipt.non_claims.iter().any(|value| value == required))
    {
        return Err(ContributionIssue::MissingNonClaim);
    }
    Ok(())
}

fn reject_sensitive_receipt_fields(
    sensitive: ContributionReceiptSensitiveDraft<'_>,
) -> Result<(), ContributionIssue> {
    let values = [
        sensitive.prompt_body,
        sensitive.skill_body,
        sensitive.mcp_instructions,
        sensitive.memory_record,
        sensitive.path,
        sensitive.credential,
        sensitive.provider_data,
        sensitive.hidden_directive,
    ];
    if values.iter().any(Option::is_some) {
        return Err(ContributionIssue::ReceiptLeak);
    }
    Ok(())
}

fn receipt_entry(candidate: &Contribution) -> ContributionReceiptEntry {
    ContributionReceiptEntry {
        contribution_id: candidate.id.clone(),
        source: candidate.source,
        trust: candidate.trust,
        posture: candidate.posture,
        priority: candidate.priority,
        budget_cost: candidate.budget_cost,
        payload_identity: candidate.payload_identity.clone(),
        deduplication_key: candidate.deduplication_key.clone(),
        supersedes: candidate.supersedes.clone(),
        conflict_group: candidate.conflict_group.clone(),
        lifetime: candidate.lifetime,
        sensitivity: candidate.sensitivity,
        safe_label: candidate.safe_label.clone(),
    }
}

fn receipt_identity(
    plan: &ContributionPlan,
    selected: &[ContributionReceiptEntry],
) -> ReceiptIdentity {
    debug_assert!(!plan.plan_id.as_str().is_empty());
    debug_assert!(selected.iter().all(|entry| !entry.safe_label.is_empty()));
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, RECEIPT_HASH_DOMAIN);
    update_hash(&mut hasher, plan.plan_id.as_str());
    update_hash(&mut hasher, plan.selected_budget.to_string().as_str());
    for entry in selected {
        for value in [
            entry.contribution_id.as_str(),
            entry.source.as_str(),
            entry.trust.as_str(),
            entry.posture.as_str(),
            entry.payload_identity.as_str(),
            entry.deduplication_key.as_str(),
            entry.conflict_group.as_deref().unwrap_or("none"),
            entry.lifetime.as_str(),
            entry.sensitivity.as_str(),
            entry.safe_label.as_str(),
        ] {
            update_hash(&mut hasher, value);
        }
        update_hash(&mut hasher, entry.priority.to_string().as_str());
        update_hash(&mut hasher, entry.budget_cost.to_string().as_str());
        for superseded in &entry.supersedes {
            update_hash(&mut hasher, superseded.as_str());
        }
    }
    for claim in ALLOWED_CLAIMS {
        update_hash(&mut hasher, claim);
    }
    for non_claim in REQUIRED_NON_CLAIMS {
        update_hash(&mut hasher, non_claim);
    }
    ReceiptIdentity::new(hasher.finalize().to_hex().to_string())
}

fn update_hash(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(HASH_SEPARATOR);
}
