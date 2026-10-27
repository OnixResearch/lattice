//! Metadata-only recovery evidence.

type CompletedDisposition = super::data::CompletedDisposition;
type EntryClass = super::data::EntryClass;
type Error = super::data::Error;
type Issue = super::data::Issue;
type Projection = super::data::Projection;
type Receipt = super::data::Receipt;
type ReceiptIdentity = super::data::ReceiptIdentity;
type SensitiveDraft<'a> = super::data::SensitiveDraft<'a>;

const RECEIPT_DOMAIN: &str = "animus.pending-entry-recovery.receipt.v1";
const HASH_SEPARATOR: &[u8] = b"\0";
const ALLOWED_CLAIMS: &[&str] = &[
    "deterministic-projection",
    "generation-fenced-disposition",
    "metadata-only-evidence",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "power-loss-durability",
    "storage-correctness",
    "operator-intent",
    "provider-delivery",
    "queue-delivery",
    "exactly-once-effects",
    "transcript-completeness",
    "encryption",
    "production-readiness",
    "release-eligibility",
];
const ENTRY_CLASSES: &[EntryClass] = &[
    EntryClass::ActiveAccepted,
    EntryClass::SteeringInFlight,
    EntryClass::SteeringQueued,
    EntryClass::FollowUp,
    EntryClass::Command,
    EntryClass::AttachmentReference,
];

/// Projects bounded evidence without entry bodies or host secrets.
///
/// # Errors
///
/// Returns receipt leak when any sensitive candidate field is present.
// r[impl agent_harness_runtime.pending_input_vault.evidence]
pub fn project_receipt(
    projection: &Projection,
    completed: Option<&CompletedDisposition>,
    sensitive: SensitiveDraft<'_>,
) -> Result<Receipt, Error> {
    reject_sensitive(sensitive)?;
    if let Some(value) = completed {
        super::core::validate_completed(value)?;
        if value.boundary != projection.boundary {
            return Err(Error::new(Issue::Stale));
        }
    }
    let class_counts = class_counts(projection)?;
    let entry_count =
        u32::try_from(projection.entries.len()).map_err(|_| Error::new(Issue::EntryLimit))?;
    let disposition = completed.map(|value| value.kind);
    let cleanup = completed.map_or(super::data::CleanupState::NotRequired, |value| {
        value.cleanup
    });
    let state = receipt_state(&projection.state, cleanup, completed.is_some());
    let claims = copy_labels(ALLOWED_CLAIMS);
    let non_claims = copy_labels(REQUIRED_NON_CLAIMS);
    let id = receipt_identity(projection, &state, completed, class_counts.as_slice());
    debug_assert!(entry_count <= super::data::MAX_ENTRY_COUNT);
    debug_assert_eq!(class_counts.len(), ENTRY_CLASSES.len());
    Ok(Receipt {
        id,
        projection: projection.identity.clone(),
        conversation: projection.boundary.conversation.clone(),
        branch: projection.boundary.branch.clone(),
        store: projection.boundary.store.clone(),
        authority: projection.boundary.authority.clone(),
        policy: projection.boundary.policy.clone(),
        generation: projection.boundary.generation,
        entry_count,
        class_counts,
        state,
        disposition,
        cleanup,
        claims,
        non_claims,
    })
}

/// Validates receipt claims and mandatory non-claims.
///
/// # Errors
///
/// Returns overclaim or missing-non-claim without echoing supplied text.
pub fn validate_receipt(receipt: &Receipt) -> Result<(), Error> {
    validate_receipt_header(receipt)?;
    for value in [
        receipt.conversation.as_str(),
        receipt.branch.as_str(),
        receipt.store.as_str(),
        receipt.authority.as_str(),
        receipt.policy.as_str(),
    ] {
        super::core::validate_metadata(value).map_err(|_| Error::new(Issue::InvalidReceipt))?;
    }
    validate_class_counts(receipt)?;
    validate_receipt_state(receipt)?;
    let is_claim_roster_exact = receipt.claims.len() == ALLOWED_CLAIMS.len()
        && receipt
            .claims
            .iter()
            .zip(ALLOWED_CLAIMS.iter())
            .all(|(actual, expected)| actual == expected);
    if !is_claim_roster_exact {
        return Err(Error::new(Issue::ReceiptOverclaim));
    }
    let is_non_claim_roster_exact = receipt.non_claims.len() == REQUIRED_NON_CLAIMS.len()
        && receipt
            .non_claims
            .iter()
            .zip(REQUIRED_NON_CLAIMS.iter())
            .all(|(actual, expected)| actual == expected);
    if !is_non_claim_roster_exact {
        return Err(Error::new(Issue::MissingNonClaim));
    }
    debug_assert!(super::core::is_blake3(receipt.id.as_str()));
    debug_assert!(super::core::is_blake3(receipt.projection.as_str()));
    Ok(())
}

fn validate_receipt_header(receipt: &Receipt) -> Result<(), Error> {
    if !super::core::is_blake3(receipt.id.as_str()) {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    if !super::core::is_blake3(receipt.projection.as_str()) {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    if receipt.generation.value() == 0 {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    if receipt.entry_count > super::data::MAX_ENTRY_COUNT {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    Ok(())
}

fn validate_receipt_state(receipt: &Receipt) -> Result<(), Error> {
    let is_consistent = match receipt.disposition {
        None => {
            receipt.cleanup == super::data::CleanupState::NotRequired
                && matches!(
                    receipt.state,
                    super::data::RecoveryState::NoVault
                        | super::data::RecoveryState::ReviewRequired
                        | super::data::RecoveryState::BlockedCorrupt(_)
                )
        }
        Some(_) => match receipt.cleanup {
            super::data::CleanupState::Required | super::data::CleanupState::Failed => {
                receipt.state == super::data::RecoveryState::CleanupRequired
            }
            super::data::CleanupState::Complete => {
                receipt.state == super::data::RecoveryState::DispositionComplete
            }
            super::data::CleanupState::NotRequired => false,
        },
    };
    if !is_consistent {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    debug_assert!(is_consistent);
    debug_assert_eq!(
        receipt.disposition.is_some(),
        receipt.cleanup != super::data::CleanupState::NotRequired
    );
    Ok(())
}

fn validate_class_counts(receipt: &Receipt) -> Result<(), Error> {
    if receipt.class_counts.len() != ENTRY_CLASSES.len() {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    let mut total = 0_u32;
    for (expected, (actual, count)) in ENTRY_CLASSES.iter().zip(receipt.class_counts.iter()) {
        if expected != actual {
            return Err(Error::new(Issue::InvalidReceipt));
        }
        total = total
            .checked_add(*count)
            .ok_or_else(|| Error::new(Issue::InvalidReceipt))?;
    }
    if total != receipt.entry_count {
        return Err(Error::new(Issue::InvalidReceipt));
    }
    Ok(())
}

fn reject_sensitive(sensitive: SensitiveDraft<'_>) -> Result<(), Error> {
    let values = [
        sensitive.body,
        sensitive.attachment,
        sensitive.prompt,
        sensitive.tool_data,
        sensitive.provider_data,
        sensitive.credential,
        sensitive.environment_value,
        sensitive.physical_path,
        sensitive.raw_store_error,
    ];
    if values.iter().any(Option::is_some) {
        return Err(Error::new(Issue::ReceiptLeak));
    }
    Ok(())
}

fn class_counts(projection: &Projection) -> Result<Vec<(EntryClass, u32)>, Error> {
    let mut counts = Vec::with_capacity(ENTRY_CLASSES.len());
    for class in ENTRY_CLASSES {
        let count = projection
            .entries
            .iter()
            .filter(|entry| entry.class == *class)
            .count();
        let bounded = u32::try_from(count).map_err(|_| Error::new(Issue::EntryLimit))?;
        counts.push((*class, bounded));
    }
    Ok(counts)
}

fn copy_labels(labels: &[&str]) -> Vec<String> {
    let mut values = Vec::with_capacity(labels.len());
    for label in labels {
        values.push((*label).to_string());
    }
    values
}

fn receipt_identity(
    projection: &Projection,
    state: &super::data::RecoveryState,
    completed: Option<&CompletedDisposition>,
    class_counts: &[(EntryClass, u32)],
) -> ReceiptIdentity {
    debug_assert!(!projection.identity.as_str().is_empty());
    debug_assert_eq!(class_counts.len(), ENTRY_CLASSES.len());
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, RECEIPT_DOMAIN);
    update_hash(&mut hasher, projection.identity.as_str());
    update_hash(&mut hasher, state.as_str());
    for (class, count) in class_counts {
        update_hash(&mut hasher, class.as_str());
        update_hash(&mut hasher, count.to_string().as_str());
    }
    if let Some(value) = completed {
        update_hash(&mut hasher, value.id.as_str());
        update_hash(&mut hasher, value.plan_identity.as_str());
        update_hash(&mut hasher, value.projection.as_str());
        update_hash(&mut hasher, value.kind.as_str());
        update_hash(&mut hasher, value.cleanup.as_str());
        update_hash(
            &mut hasher,
            if value.restore_material_retained {
                "restore-material-retained"
            } else {
                "restore-material-not-retained"
            },
        );
        for selected in &value.selected {
            update_hash(&mut hasher, selected.as_str());
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

fn receipt_state(
    projected: &super::data::RecoveryState,
    cleanup: super::data::CleanupState,
    has_completed: bool,
) -> super::data::RecoveryState {
    if !has_completed {
        return projected.clone();
    }
    match cleanup {
        super::data::CleanupState::Required | super::data::CleanupState::Failed => {
            super::data::RecoveryState::CleanupRequired
        }
        super::data::CleanupState::Complete => super::data::RecoveryState::DispositionComplete,
        super::data::CleanupState::NotRequired => projected.clone(),
    }
}

fn update_hash(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(HASH_SEPARATOR);
}
