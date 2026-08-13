use super::*;

pub(crate) struct InvocationFixture {
    pub(crate) token: CompactToken,
    pub(crate) holder: Ed25519InMemorySigner,
    pub(crate) audience: AudienceDid,
    pub(crate) keys: KeyResolutionContext,
    pub(crate) proofs: ProofCollection,
}

impl InvocationFixture {
    pub(crate) fn direct(capability: CapabilityDocument) -> Self {
        Self::direct_with_expiration(capability, TOKEN_EXPIRES_SECONDS)
    }

    pub(crate) fn direct_with_expiration(capability: CapabilityDocument, expires_at_seconds: u64) -> Self {
        let issuer = signer(ISSUER_SEED_BYTE);
        let holder = signer(HOLDER_SEED_BYTE);
        let audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
        let token = issue_token_with_signer(
            &issuer,
            &audience,
            &CapabilitySet::new(vec![capability]).expect("test capability must validate"),
            &ProofReferences::empty(),
            TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, expires_at_seconds).expect("test time bounds must validate"),
        )
        .expect("direct test token must issue");
        let keys = KeyResolutionContext::new(vec![
            issuer.verification_key().expect("issuer verification key must derive"),
            holder.verification_key().expect("holder verification key must derive"),
        ]);
        Self {
            token,
            holder,
            audience,
            keys,
            proofs: ProofCollection::empty(),
        }
    }

    pub(crate) fn delegated() -> Self {
        let issuer = signer(ISSUER_SEED_BYTE);
        let delegate = signer(DELEGATE_SEED_BYTE);
        let holder = signer(HOLDER_SEED_BYTE);
        let delegate_audience = AudienceDid::from(delegate.issuer().expect("delegate DID must derive"));
        let audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
        let parent = issue_token_with_signer(
            &issuer,
            &delegate_audience,
            &CapabilitySet::new(vec![document_capability(PARENT_RESOURCE)]).expect("parent capability must validate"),
            &ProofReferences::empty(),
            test_time_bounds(),
        )
        .expect("parent token must issue");
        let parent_reference = proof_reference(&parent);
        let child = issue_token_with_signer(
            &delegate,
            &audience,
            &CapabilitySet::new(vec![document_capability(DOCUMENT_RESOURCE)]).expect("child capability must validate"),
            &ProofReferences::from_references(vec![parent_reference]),
            test_time_bounds(),
        )
        .expect("child token must issue");
        let keys = KeyResolutionContext::new(vec![
            issuer.verification_key().expect("issuer verification key must derive"),
            delegate.verification_key().expect("delegate verification key must derive"),
            holder.verification_key().expect("holder verification key must derive"),
        ]);
        Self {
            token: child,
            holder,
            audience,
            keys,
            proofs: ProofCollection::from_tokens(vec![parent]),
        }
    }
}

pub(crate) fn signer(seed_byte: u8) -> Ed25519InMemorySigner {
    Ed25519InMemorySigner::from_seed_bytes([seed_byte; ED25519_SECRET_KEY_BYTES])
}

fn test_time_bounds() -> TokenTimeBounds {
    TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).expect("test time bounds must validate")
}

pub(crate) fn document_capability(resource: &str) -> CapabilityDocument {
    CapabilityDocument::new(resource.to_owned(), DOCUMENT_WRITE_ABILITY.to_owned())
        .expect("test capability must validate")
}

pub(crate) fn caveated_document_capability() -> CapabilityDocument {
    let caveat = CaveatDocument::new(
        CAVEAT_DOMAIN.to_owned(),
        CAVEAT_TYPE.to_owned(),
        CAVEAT_KEY.to_owned(),
        CAVEAT_PAYLOAD.to_vec(),
    )
    .expect("test caveat must validate");
    CapabilityDocument::with_caveats(PARENT_RESOURCE.to_owned(), DOCUMENT_WRITE_ABILITY.to_owned(), vec![caveat])
        .expect("caveated test capability must validate")
}

pub(crate) fn enforcement_request(resource: &str) -> EnforcementRequest {
    EnforcementRequest::new("document-write", resource.to_owned(), DOCUMENT_WRITE_ABILITY)
}

pub(crate) const fn nonce() -> HolderInvocationNonce {
    HolderInvocationNonce::from_bytes(
        [NONCE_BYTE; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES],
    )
}

pub(crate) const fn other_nonce() -> HolderInvocationNonce {
    HolderInvocationNonce::from_bytes(
        [OTHER_NONCE_BYTE; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES],
    )
}
