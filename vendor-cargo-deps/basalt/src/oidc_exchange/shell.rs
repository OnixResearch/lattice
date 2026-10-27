use ucan::{AudienceDid, CompactToken, TokenSigner};

use super::core::verify_and_plan;
use super::issue::{IssueFailure, IssuerAdmission, issue_plan, issue_plan_to_recipient, validate_issuer_admission};
use super::model::{ExchangeRequest, OidcPolicy, RejectReason, ReplayKey};
use super::network::{DocumentPort, JwksCache, KeyLoadFailure, load_verification_key};
use super::receipt::{ExchangeReceipt, bind_recipient, receipt_for_failure, receipt_for_success};

/// One replay-store commit outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayCommit {
    /// The replay key was reserved for the first time.
    Fresh,
    /// The replay key was already reserved.
    Duplicate,
}

/// Stable replay-store failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayFailure {
    /// The store is unavailable.
    Unavailable,
    /// The store rejected malformed or contradictory input.
    Invalid,
}

/// Caller-owned consumptive replay capability.
pub trait ReplayPort {
    /// Atomically reserve one replay key until its expiration.
    ///
    /// # Errors
    ///
    /// Returns a stable failure when storage cannot make a durable decision.
    fn commit(
        &mut self,
        key: &ReplayKey,
        expires_at_seconds: u64,
    ) -> Result<ReplayCommit, ReplayFailure>;
}

/// Successful exchange result. The compact token remains live bearer material.
pub struct ExchangeResult {
    /// Issued compact UCAN. Callers must not log or persist it as evidence.
    pub token: CompactToken,
    /// Metadata-only exchange receipt.
    pub receipt: ExchangeReceipt,
}

impl std::fmt::Debug for ExchangeResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExchangeResult")
            .field("token", &"[REDACTED]")
            .field("receipt", &self.receipt)
            .finish()
    }
}

/// Terminal exchange failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeFailureClass {
    /// Discovery or JWK selection failed.
    KeyLoad(KeyLoadFailure),
    /// Pure verification or attenuation failed.
    Verification(RejectReason),
    /// Replay storage failed or reported a duplicate.
    Replay(ReplayFailure),
    /// The replay key was already consumed.
    Duplicate,
    /// UCAN issuance failed.
    Issue(IssueFailure),
}

/// Terminal failure with metadata-only evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeFailure {
    /// Stable failure class.
    pub class: ExchangeFailureClass,
    /// Redacted failure receipt.
    pub receipt: Box<ExchangeReceipt>,
}

/// Complete imperative exchange shell.
///
/// The shell retrieves public keys, runs the pure core, commits replay once,
/// and then asks the caller-owned signer to issue one attenuated UCAN.
///
/// # Errors
///
/// Returns a redacted typed failure. No failure contains OIDC or UCAN bearer text.
#[allow(
    clippy::too_many_arguments,
    reason = "explicit ports keep exchange authority visible"
)]
pub fn exchange(
    token: &str,
    now_seconds: u64,
    policy: &OidcPolicy,
    request: &ExchangeRequest,
    cache: &mut JwksCache,
    documents: &mut dyn DocumentPort,
    replay: &mut dyn ReplayPort,
    issuer: &IssuerAdmission,
    signer: &dyn TokenSigner,
) -> Result<ExchangeResult, ExchangeFailure> {
    exchange_inner(token, now_seconds, policy, request, cache, documents, replay, issuer, signer, None)
}

// r[impl ucan_token_flow.oidc.holder_recipient]
/// Exchange for an explicit holder without replacing the policy's service audience.
///
/// The resource and operation remain attenuated by the admitted service policy.
/// The recipient is a delivery binding, not a proof of holder possession. The
/// receiver must verify a holder-signed invocation before dispatch. Replay keeps
/// the original request key, so changing recipients cannot repeat an exchange.
///
/// # Errors
/// Returns the same fail-closed, redacted failures as [`exchange`].
#[allow(clippy::too_many_arguments, reason = "explicit ports keep exchange authority visible")]
pub fn exchange_to_recipient(
    token: &str,
    now_seconds: u64,
    policy: &OidcPolicy,
    request: &ExchangeRequest,
    cache: &mut JwksCache,
    documents: &mut dyn DocumentPort,
    replay: &mut dyn ReplayPort,
    issuer: &IssuerAdmission,
    signer: &dyn TokenSigner,
    recipient: &AudienceDid,
) -> Result<ExchangeResult, ExchangeFailure> {
    match exchange_inner(token, now_seconds, policy, request, cache, documents, replay, issuer, signer, Some(recipient)) {
        Ok(mut result) => {
            bind_recipient(&mut result.receipt, recipient);
            Ok(result)
        }
        Err(mut failure) => {
            bind_recipient(&mut failure.receipt, recipient);
            Err(failure)
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "explicit ports keep exchange authority visible")]
fn exchange_inner(
    token: &str,
    now_seconds: u64,
    policy: &OidcPolicy,
    request: &ExchangeRequest,
    cache: &mut JwksCache,
    documents: &mut dyn DocumentPort,
    replay: &mut dyn ReplayPort,
    issuer: &IssuerAdmission,
    signer: &dyn TokenSigner,
    recipient: Option<&AudienceDid>,
) -> Result<ExchangeResult, ExchangeFailure> {
    validate_issuer_admission(issuer, signer).map_err(|class| ExchangeFailure {
        receipt: Box::new(receipt_for_failure(
            policy,
            request,
            issue_failure_reason(class),
            None,
        )),
        class: ExchangeFailureClass::Issue(class),
    })?;
    let key =
        load_verification_key(token, now_seconds, policy, cache, documents).map_err(|class| {
            ExchangeFailure {
                receipt: Box::new(receipt_for_failure(
                    policy,
                    request,
                    key_load_reason(class),
                    None,
                )),
                class: ExchangeFailureClass::KeyLoad(class),
            }
        })?;
    let plan = verify_and_plan(token, &key, now_seconds, policy, request).map_err(|class| {
        ExchangeFailure {
            receipt: Box::new(receipt_for_failure(
                policy,
                request,
                class.as_str(),
                Some(key.key_id.as_str()),
            )),
            class: ExchangeFailureClass::Verification(class),
        }
    })?;
    let replay_outcome = replay
        .commit(&plan.replay_key, plan.expires_at_seconds)
        .map_err(|class| ExchangeFailure {
            receipt: Box::new(receipt_for_failure(
                policy,
                request,
                replay_failure_reason(class),
                Some(plan.key_id.as_str()),
            )),
            class: ExchangeFailureClass::Replay(class),
        })?;
    if replay_outcome == ReplayCommit::Duplicate {
        return Err(ExchangeFailure {
            class: ExchangeFailureClass::Duplicate,
            receipt: Box::new(receipt_for_failure(
                policy,
                request,
                "replay-duplicate",
                Some(plan.key_id.as_str()),
            )),
        });
    }
    let issued = match recipient {
        Some(recipient) => issue_plan_to_recipient(&plan, recipient, issuer, signer),
        None => issue_plan(&plan, issuer, signer),
    };
    let token = issued.map_err(|class| ExchangeFailure {
        receipt: Box::new(receipt_for_failure(
            policy,
            request,
            issue_failure_reason(class),
            Some(plan.key_id.as_str()),
        )),
        class: ExchangeFailureClass::Issue(class),
    })?;
    let receipt = receipt_for_success(policy, &plan);
    Ok(ExchangeResult { token, receipt })
}

const fn replay_failure_reason(failure: ReplayFailure) -> &'static str {
    match failure {
        ReplayFailure::Unavailable => "replay-unavailable",
        ReplayFailure::Invalid => "replay-invalid",
    }
}

const fn issue_failure_reason(failure: IssueFailure) -> &'static str {
    match failure {
        IssueFailure::Audience => "issuance-audience",
        IssueFailure::Capability => "issuance-capability",
        IssueFailure::Time => "issuance-time",
        IssueFailure::Token => "issuance-token",
        IssueFailure::IssuerBinding => "issuance-issuer-binding",
    }
}

const fn key_load_reason(failure: KeyLoadFailure) -> &'static str {
    match failure {
        KeyLoadFailure::Token(reason) => reason.as_str(),
        KeyLoadFailure::Discovery(_) => "discovery",
        KeyLoadFailure::DiscoveryIdentity => "discovery-identity",
        KeyLoadFailure::Jwks(_) => "jwks",
        KeyLoadFailure::MalformedJwks => "malformed-jwks",
        KeyLoadFailure::ExcessiveKeys => "excessive-keys",
        KeyLoadFailure::UnknownKey => "unknown-key",
        KeyLoadFailure::Stale => "stale-jwks",
    }
}
