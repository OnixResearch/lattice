use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeSet;

use super::*;

pub(crate) struct RecordingReplay<'a> {
    pub(crate) calls: Cell<u32>,
    seen: RefCell<BTreeSet<(String, [u8; basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES])>>,
    caveat_calls: Option<&'a Cell<u32>>,
    expected_caveat_calls: u32,
    pub(crate) committed: Cell<bool>,
}

impl RecordingReplay<'_> {
    pub(crate) const fn new() -> Self {
        Self {
            calls: Cell::new(0),
            seen: RefCell::new(BTreeSet::new()),
            caveat_calls: None,
            expected_caveat_calls: 0,
            committed: Cell::new(false),
        }
    }

    pub(crate) const fn after_caveats(caveat_calls: &Cell<u32>, expected_caveat_calls: u32) -> RecordingReplay<'_> {
        RecordingReplay {
            calls: Cell::new(0),
            seen: RefCell::new(BTreeSet::new()),
            caveat_calls: Some(caveat_calls),
            expected_caveat_calls,
            committed: Cell::new(false),
        }
    }
}

impl HolderInvocationReplayAdmission for RecordingReplay<'_> {
    fn admit_holder_invocation(
        &self,
        candidate: &HolderInvocationReplayCandidate<'_>,
    ) -> Result<(), HolderInvocationReplayError> {
        self.calls
            .set(self.calls.get().checked_add(FIRST_CALL).expect("test replay call count must not overflow"));
        if let Some(caveat_calls) = self.caveat_calls {
            assert_eq!(caveat_calls.get(), self.expected_caveat_calls);
        }
        let replay_key = (candidate.audience().as_str().to_owned(), *candidate.nonce().as_bytes());
        if !self.seen.borrow_mut().insert(replay_key) {
            return Err(HolderInvocationReplayError::Duplicate);
        }
        self.committed.set(true);
        Ok(())
    }
}

pub(crate) struct CountingCaveatPolicy {
    pub(crate) calls: Cell<u32>,
    is_allowed: bool,
}

impl CountingCaveatPolicy {
    const fn new(is_allowed: bool) -> Self {
        Self {
            calls: Cell::new(0),
            is_allowed,
        }
    }
}

impl CaveatPolicy for CountingCaveatPolicy {
    fn evaluate(&self, _caveat: &CaveatDocument, _request: &basalt::ucan::AuthorizationRequest) -> CaveatDecision {
        self.calls
            .set(self.calls.get().checked_add(FIRST_CALL).expect("test caveat call count must not overflow"));
        if self.is_allowed {
            CaveatDecision::Satisfied
        } else {
            CaveatDecision::Rejected {
                message: "test caveat denied".to_owned(),
            }
        }
    }
}

pub(crate) struct CountingCaveatSet {
    pub(crate) policy: CountingCaveatPolicy,
}

impl CountingCaveatSet {
    pub(crate) const fn new(is_allowed: bool) -> Self {
        Self {
            policy: CountingCaveatPolicy::new(is_allowed),
        }
    }
}

impl CaveatPolicySet for CountingCaveatSet {
    fn policy_for(&self, _caveat: &CaveatIdentifier) -> Option<&dyn CaveatPolicy> {
        Some(&self.policy)
    }
}

pub(crate) struct WrongIssuerKeyResolver {
    pub(crate) public_key: Ed25519PublicKey,
}

impl KeyResolver for WrongIssuerKeyResolver {
    fn resolve_key(&self, _issuer: &IssuerDid) -> Result<Ed25519PublicKey, KeyResolverError> {
        Ok(self.public_key.clone())
    }
}

pub(crate) struct FailsAfterResolver<'a> {
    pub(crate) inner: &'a KeyResolutionContext,
    pub(crate) allowed_calls: u32,
    pub(crate) calls: Cell<u32>,
}

impl KeyResolver for FailsAfterResolver<'_> {
    fn resolve_key(&self, issuer: &IssuerDid) -> Result<Ed25519PublicKey, KeyResolverError> {
        let next_call = self.calls.get().checked_add(FIRST_CALL).expect("test resolver call count must not overflow");
        self.calls.set(next_call);
        if next_call > self.allowed_calls {
            return Err(KeyResolverError::Backend {
                issuer: issuer.as_str().to_owned(),
                message: BACKEND_ERROR_MESSAGE.to_owned(),
            });
        }
        self.inner.resolve_key(issuer)
    }
}
