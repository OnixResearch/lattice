const RECEIPT_DOMAIN: &[u8] = b"animus/host-authority-receipt/v1";
const RECEIPT_SCHEMA: &str = "animus.host-authority-receipt.v1";
const BLAKE3_PREFIX: &str = "b3:";
const MAX_NON_CLAIMS: usize = 16;
const CREDENTIAL_MARKERS: &[&str] = &[
    "bearer ",
    "compact_token",
    "private_key",
    "signature",
    "proof_body",
    "replay_nonce",
    "credential",
];
const PRIVATE_MATERIAL_MARKERS: &[&str] = &[
    "raw_prompt",
    "tool_arguments",
    "provider_payload",
    "environment_value",
    "private_path",
    "effect_output",
];
const OVERCLAIM_MARKERS: &[&str] = &[
    "globally trusted",
    "sandbox is secure",
    "effect is correct",
    "agent is safe",
    "completion proved",
    "release ready",
];

/// Required bounded claim limits for all generic authority receipts.
pub const HOST_AUTHORITY_NON_CLAIMS: [&str; 8] = [
    "global_trust",
    "key_truth",
    "global_revocation_freshness",
    "replay_durability",
    "sandbox_confinement",
    "effect_correctness",
    "agent_safety",
    "completion_or_release_readiness",
];

/// Receipt input over one already-validated safe decision.
#[derive(Clone, Copy, Debug)]
pub struct HostAuthorityReceiptDraft<'a> {
    /// Exact safe decision.
    pub decision: &'a super::evaluate::HostAuthorityDecision,
    /// Safe dispatch class, such as `planned`, `committed`, or `blocked`.
    pub dispatch_class: &'a str,
    /// Required non-claims. The complete fixed set is mandatory.
    pub non_claims: &'a [&'a str],
}

/// Metadata-only authority receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostAuthorityReceipt {
    /// Receipt schema.
    pub schema: &'static str,
    /// Provider identity.
    pub provider_identity: alloc::string::String,
    /// Policy identity.
    pub policy_identity: alloc::string::String,
    /// Request identity.
    pub request_identity: alloc::string::String,
    /// Binding identity.
    pub binding_identity: alloc::string::String,
    /// Decision identity.
    pub decision_identity: alloc::string::String,
    /// Safe action label.
    pub action: &'static str,
    /// Safe ability label.
    pub ability: alloc::string::String,
    /// Decision status.
    pub status: super::evaluate::HostAuthorityStatus,
    /// Safe issue class.
    pub issue: super::evaluate::HostAuthorityIssue,
    /// Checked provider fact count.
    pub checked_fact_count: u32,
    /// Redacted principal hash.
    pub principal_hash: alloc::string::String,
    /// Redacted resource hash.
    pub resource_hash: alloc::string::String,
    /// Safe dispatch class.
    pub dispatch_class: alloc::string::String,
    /// Required non-claims.
    pub non_claims: alloc::vec::Vec<alloc::string::String>,
    /// Complete receipt identity.
    pub receipt_identity: alloc::string::String,
}

/// Receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostAuthorityReceiptLeak {
    /// Bearer, key, signature, proof, nonce, or credential material appeared.
    Credential,
    /// Prompt, tool argument, provider payload, environment, path, or output material appeared.
    PrivateMaterial,
    /// A completion, safety, trust, effect, or release overclaim appeared.
    Overclaim,
}

/// Build one metadata-only authority receipt.
///
/// # Errors
///
/// Returns [`HostAuthorityIssue::UnsafeReceipt`] for missing non-claims or unsafe dispatch metadata.
pub fn build_host_authority_receipt(
    draft: HostAuthorityReceiptDraft<'_>,
) -> Result<HostAuthorityReceipt, super::evaluate::HostAuthorityIssue> {
    if draft.non_claims.len() > MAX_NON_CLAIMS
        || draft.non_claims != HOST_AUTHORITY_NON_CLAIMS
        || !safe_dispatch_class(draft.dispatch_class)
    {
        return Err(super::evaluate::HostAuthorityIssue::UnsafeReceipt);
    }
    debug_assert_eq!(draft.non_claims, HOST_AUTHORITY_NON_CLAIMS);
    debug_assert!(safe_dispatch_class(draft.dispatch_class));
    let receipt_identity = receipt_identity(&draft)?;
    Ok(HostAuthorityReceipt {
        schema: RECEIPT_SCHEMA,
        provider_identity: alloc::string::String::from(draft.decision.provider_identity()),
        policy_identity: alloc::string::String::from(draft.decision.policy_identity()),
        request_identity: alloc::string::String::from(draft.decision.request_identity()),
        binding_identity: alloc::string::String::from(draft.decision.binding_identity()),
        decision_identity: alloc::string::String::from(draft.decision.decision_identity()),
        action: draft.decision.action().as_str(),
        ability: alloc::string::String::from(draft.decision.ability()),
        status: draft.decision.status(),
        issue: draft.decision.issue(),
        checked_fact_count: draft.decision.checked_fact_count(),
        principal_hash: alloc::string::String::from(draft.decision.principal_hash()),
        resource_hash: alloc::string::String::from(draft.decision.resource_hash()),
        dispatch_class: alloc::string::String::from(draft.dispatch_class),
        non_claims: draft
            .non_claims
            .iter()
            .map(|value| alloc::string::String::from(*value))
            .collect(),
        receipt_identity,
    })
}

/// Reject rendered receipt text that contains private material or overclaims.
///
/// # Errors
///
/// Returns the first stable leak class in credential, private-material, then overclaim order.
pub fn validate_host_authority_receipt_text(text: &str) -> Result<(), HostAuthorityReceiptLeak> {
    let lowered = text.to_ascii_lowercase();
    if CREDENTIAL_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return Err(HostAuthorityReceiptLeak::Credential);
    }
    if PRIVATE_MATERIAL_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return Err(HostAuthorityReceiptLeak::PrivateMaterial);
    }
    if OVERCLAIM_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
    {
        return Err(HostAuthorityReceiptLeak::Overclaim);
    }
    debug_assert!(
        !CREDENTIAL_MARKERS
            .iter()
            .any(|marker| lowered.contains(marker))
    );
    debug_assert!(
        !PRIVATE_MATERIAL_MARKERS
            .iter()
            .any(|marker| lowered.contains(marker))
    );
    Ok(())
}

fn safe_dispatch_class(value: &str) -> bool {
    matches!(value, "planned" | "committed" | "blocked")
}

fn receipt_identity(
    draft: &HostAuthorityReceiptDraft<'_>,
) -> Result<alloc::string::String, super::evaluate::HostAuthorityIssue> {
    debug_assert!(!draft.dispatch_class.is_empty());
    debug_assert!(!draft.non_claims.is_empty());
    let checked = draft.decision.checked_fact_count().to_be_bytes();
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_DOMAIN);
    for field in [
        RECEIPT_SCHEMA.as_bytes(),
        draft.decision.decision_identity().as_bytes(),
        draft.decision.request_identity().as_bytes(),
        draft.decision.binding_identity().as_bytes(),
        draft.decision.provider_identity().as_bytes(),
        draft.decision.policy_identity().as_bytes(),
        draft.decision.action().as_str().as_bytes(),
        draft.decision.ability().as_bytes(),
        draft.decision.principal_hash().as_bytes(),
        draft.decision.resource_hash().as_bytes(),
        checked.as_slice(),
        draft.dispatch_class.as_bytes(),
    ] {
        let field_bytes = u64::try_from(field.len())
            .map_err(|_| super::evaluate::HostAuthorityIssue::InvalidBound)?;
        hasher.update(&field_bytes.to_be_bytes());
        hasher.update(field);
    }
    for non_claim in draft.non_claims {
        let non_claim_bytes = u64::try_from(non_claim.len())
            .map_err(|_| super::evaluate::HostAuthorityIssue::InvalidBound)?;
        hasher.update(&non_claim_bytes.to_be_bytes());
        hasher.update(non_claim.as_bytes());
    }
    Ok(alloc::format!(
        "{BLAKE3_PREFIX}{}",
        hasher.finalize().to_hex()
    ))
}
