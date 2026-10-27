//! Metadata-only handoff receipt projection and validation.

type Error = super::data::Error;
type HandoffReceipt = super::data::HandoffReceipt;
type HandoffTransaction = super::data::HandoffTransaction;
type Issue = super::data::Issue;
type SensitiveEvidence = super::data::SensitiveEvidence;

/// Required receipt redaction marker.
pub const HANDOFF_REDACTION: &str =
    "draft-input-prompt-transcript-credential-path-process-provider-and-terminal-data-omitted";
/// Required PTY migration non-claim.
pub const NOT_PTY_MIGRATION: &str = "not-pty-migration";
/// Required process-memory continuity non-claim.
pub const NOT_PROCESS_MEMORY_CONTINUITY: &str = "not-process-memory-continuity";
/// Required sandbox and remote-security non-claim.
pub const NOT_SANDBOX_OR_REMOTE_SECURITY: &str = "not-sandbox-or-remote-security";
/// Required production-readiness non-claim.
pub const NOT_PRODUCTION_READINESS: &str = "not-production-readiness";

const RECEIPT_DOMAIN: &str = "animus.daemon-handoff.receipt.v1";
const FRAME_LENGTH_BYTES: usize = core::mem::size_of::<u64>();
const FRAME_LENGTH_OVERFLOW_TAG: &[u8] = b"animus.daemon-handoff.receipt-frame-overflow.v1";
const BLAKE3_HEX_BYTES: usize = 64;
const REQUIRED_NON_CLAIMS: [&str; 4] = [
    NOT_PTY_MIGRATION,
    NOT_PROCESS_MEMORY_CONTINUITY,
    NOT_SANDBOX_OR_REMOTE_SECURITY,
    NOT_PRODUCTION_READINESS,
];

/// Projects one validated transition into metadata-only evidence.
///
/// # Errors
///
/// Returns a receipt leak when any sensitive input class is present.
// r[impl agent_harness_runtime.daemon_handoff.evidence]
pub fn project_handoff_receipt(
    transaction: &HandoffTransaction,
    sensitive: SensitiveEvidence,
) -> Result<HandoffReceipt, Error> {
    super::core::validate_handoff_transaction(transaction)?;
    if contains_sensitive(sensitive) {
        return Err(Error::new(Issue::ReceiptLeak));
    }
    let non_claims = REQUIRED_NON_CLAIMS
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    let mut receipt = HandoffReceipt {
        handoff: transaction.handoff.clone(),
        worker: transaction.binding.old_worker.clone(),
        session: transaction.binding.session.clone(),
        lease_generation: transaction.binding.lease_generation,
        authority_profile: transaction.binding.authority_profile.clone(),
        transition: transaction.transition.clone(),
        state: transaction.state,
        endpoint: transaction.endpoint.clone(),
        process_attempt: transaction.process_attempt.clone(),
        readiness: transaction.readiness.clone(),
        cancellation: transaction.cancellation.clone(),
        vault_generation: transaction
            .durable_state
            .as_ref()
            .map(|state| state.vault_generation),
        redaction: HANDOFF_REDACTION.to_string(),
        non_claims,
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = receipt_digest(&receipt);
    debug_assert_eq!(receipt.redaction, HANDOFF_REDACTION);
    Ok(receipt)
}

/// Validates exact receipt binding, mandatory non-claims, and identity.
///
/// # Errors
///
/// Returns a bounded overclaim issue for drift, missing non-claims, or tampering.
pub fn validate_handoff_receipt(
    transaction: &HandoffTransaction,
    receipt: &HandoffReceipt,
) -> Result<(), Error> {
    super::core::validate_handoff_transaction(transaction)?;
    let is_exact_non_claims = receipt.non_claims.len() == REQUIRED_NON_CLAIMS.len()
        && receipt
            .non_claims
            .iter()
            .zip(REQUIRED_NON_CLAIMS)
            .all(|(actual, expected)| actual == expected);
    let is_exact = receipt.handoff == transaction.handoff
        && receipt.worker == transaction.binding.old_worker
        && receipt.session == transaction.binding.session
        && receipt.lease_generation == transaction.binding.lease_generation
        && receipt.authority_profile == transaction.binding.authority_profile
        && receipt.transition == transaction.transition
        && receipt.state == transaction.state
        && receipt.endpoint == transaction.endpoint
        && receipt.process_attempt == transaction.process_attempt
        && receipt.readiness == transaction.readiness
        && receipt.cancellation == transaction.cancellation
        && receipt.vault_generation
            == transaction
                .durable_state
                .as_ref()
                .map(|state| state.vault_generation)
        && receipt.redaction == HANDOFF_REDACTION
        && is_exact_non_claims;
    if !is_exact {
        return Err(Error::new(Issue::ReceiptOverclaim));
    }
    if receipt.receipt_identity.len() != BLAKE3_HEX_BYTES
        || !receipt
            .receipt_identity
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || receipt.receipt_identity != receipt_digest(receipt)
    {
        return Err(Error::new(Issue::ReceiptOverclaim));
    }
    debug_assert!(is_exact);
    Ok(())
}

fn receipt_digest(receipt: &HandoffReceipt) -> String {
    digest(
        RECEIPT_DOMAIN,
        &[
            receipt.handoff.as_str(),
            receipt.worker.as_str(),
            receipt.session.as_str(),
            &receipt.lease_generation.value().to_string(),
            receipt.authority_profile.as_str(),
            receipt.transition.as_str(),
            receipt.state.as_str(),
            receipt
                .endpoint
                .as_ref()
                .map_or("none", super::data::EndpointIdentity::as_str),
            receipt
                .process_attempt
                .as_ref()
                .map_or("none", super::data::ProcessAttemptIdentity::as_str),
            receipt
                .readiness
                .as_ref()
                .map_or("none", super::data::ReadinessIdentity::as_str),
            receipt
                .cancellation
                .as_ref()
                .map_or("none", super::data::CancellationIdentity::as_str),
            &receipt
                .vault_generation
                .map_or_else(|| "none".to_string(), |value| value.value().to_string()),
            &receipt.redaction,
            &receipt.non_claims.join("|"),
        ],
    )
}

fn digest(domain: &str, fields: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain.as_bytes());
    for field in fields {
        update_frame(&mut hasher, field.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn update_frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    let Ok(length_bytes) = u64::try_from(value.len()) else {
        hasher.update(FRAME_LENGTH_OVERFLOW_TAG);
        return;
    };
    let encoded = length_bytes.to_le_bytes();
    assert_eq!(encoded.len(), FRAME_LENGTH_BYTES);
    hasher.update(&encoded);
    hasher.update(value);
}

const fn contains_sensitive(sensitive: SensitiveEvidence) -> bool {
    !matches!(sensitive, SensitiveEvidence::None)
}
