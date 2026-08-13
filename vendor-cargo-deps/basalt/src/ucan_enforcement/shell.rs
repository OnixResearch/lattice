use std::fmt;

use ucan::AuthorizationDecision;
use ucan::HolderInvocationAuthorized;
use ucan::HolderInvocationPolicyContext;
use ucan::HolderInvocationRequest;
use ucan::HolderTokenVerificationContext;
use ucan::OperationBinding;
use ucan::OperationProfileAdmission;
use ucan::verify_compact_token_str_with_resolvers_and_revocations;
use ucan::verify_holder_signed_invocation;

use super::core::ExactOperationProfile;
use super::core::bind_enforcement_action;
use super::core::build_authority_decision;
use super::core::decode_blake3_reference;
use super::core::operation_profile_digest;
use super::error::UcanEnforcementFailure;
use super::error::authorization_error_reason;
use super::error::failure;
use super::error::holder_error_reason;
use super::error::profile_error_reason;
use super::error::token_error_reason;
use super::receipt::ReceiptContext;
use super::receipt::UcanEnforcementReceipt;
use super::receipt::build_receipt;
use super::types::UcanEnforcementBoundaries;
use super::types::UcanEnforcementReason;
use super::types::UcanEnforcementRequest;
use crate::EnforcementRequest;
use crate::Policy;
use crate::authority_core::AuthorityDecision;
use crate::authority_core::BasaltPolicyRef;

/// Live exact-request authority after final UCAN replay admission.
///
/// This type has private construction and no clone or serde surface.
pub struct LiveUcanEnforcement<'request> {
    request: &'request EnforcementRequest,
    _authority_decision: AuthorityDecision,
    receipt: UcanEnforcementReceipt,
    holder_authorization: HolderInvocationAuthorized,
}

impl fmt::Debug for LiveUcanEnforcement<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LiveUcanEnforcement")
            .field("allowed", &true)
            .field("receipt_blake3", &self.receipt.receipt_blake3)
            .finish_non_exhaustive()
    }
}

impl<'request> LiveUcanEnforcement<'request> {
    /// Return the exact borrowed request admitted by this live capability.
    #[must_use]
    pub const fn request(&self) -> &'request EnforcementRequest {
        self.request
    }

    /// Return the redacted evidence receipt.
    #[must_use]
    pub const fn receipt(&self) -> &UcanEnforcementReceipt {
        &self.receipt
    }

    /// Return the final live UCAN holder authorization.
    #[must_use]
    pub const fn holder_authorization(&self) -> &HolderInvocationAuthorized {
        &self.holder_authorization
    }
}

struct PreparedRequest {
    operation: OperationBinding,
    profile: ExactOperationProfile,
    receipt_context: ReceiptContext,
}

struct ProspectiveAuthority {
    decision: AuthorityDecision,
    receipt: UcanEnforcementReceipt,
    operation: OperationBinding,
    receipt_context: ReceiptContext,
}

/// Verify one raw holder invocation and return live exact-request Basalt authority.
///
/// All fallible policy work finishes before the final UCAN replay call.
///
/// # Errors
///
/// Returns [`UcanEnforcementFailure`] with no authority when a boundary denies.
pub fn verify_ucan_enforcement_request<'request>(
    policy: &Policy,
    trusted_policy_ref: &BasaltPolicyRef,
    request: &UcanEnforcementRequest<'request, '_>,
    boundaries: &UcanEnforcementBoundaries<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    let prepared = prepare_bounded_request(trusted_policy_ref, request, boundaries)?;
    let prospective = prepare_prospective_authority(policy, trusted_policy_ref, request, boundaries, &prepared)?;
    let holder_request = HolderInvocationRequest::new(
        request.compact_token,
        request.enforcement.resource(),
        request.enforcement.ability(),
        prospective.operation,
        request.nonce,
        request.holder_signature,
    );
    let token_context = HolderTokenVerificationContext::new(
        boundaries.verification_time,
        boundaries.key_resolver,
        boundaries.proof_store,
        boundaries.revocations,
        boundaries.verification_limits,
    );
    let policy_context = HolderInvocationPolicyContext::new(
        boundaries.caveat_policies,
        &prepared.profile,
        boundaries.replay,
        boundaries.holder_limits,
    );

    match verify_holder_signed_invocation(&holder_request, &token_context, &policy_context) {
        Ok(holder_authorization) => Ok(LiveUcanEnforcement {
            request: request.enforcement,
            _authority_decision: prospective.decision,
            receipt: prospective.receipt,
            holder_authorization,
        }),
        Err(error) => Err(failure(holder_error_reason(&error), prospective.receipt_context)),
    }
}

fn prepare_bounded_request(
    trusted_policy_ref: &BasaltPolicyRef,
    request: &UcanEnforcementRequest<'_, '_>,
    boundaries: &UcanEnforcementBoundaries<'_>,
) -> Result<PreparedRequest, UcanEnforcementFailure> {
    let operation_profile = operation_profile_digest(request.action);
    let policy = decode_blake3_reference(trusted_policy_ref.artifact_ref.as_str());
    let credential_bytes = u64::try_from(request.compact_token.len()).ok();
    let initial_context = ReceiptContext {
        credential_bytes,
        credential: None,
        request_binding: None,
        operation_profile,
        policy,
    };
    if credential_bytes.is_none() {
        return Err(failure(UcanEnforcementReason::LimitExceeded, initial_context));
    }
    if request.compact_token.len() > boundaries.verification_limits.max_compact_token_bytes() {
        return Err(failure(UcanEnforcementReason::OversizedCredential, initial_context));
    }
    let credential = *blake3::hash(request.compact_token.as_bytes()).as_bytes();
    let bounded_context = ReceiptContext {
        credential: Some(credential),
        ..initial_context
    };
    let operation = bind_enforcement_action(request.action, request.enforcement)
        .map_err(|reason| failure(reason, bounded_context))?;
    let request_binding = *operation.request_binding().as_bytes();
    Ok(PreparedRequest {
        operation,
        profile: ExactOperationProfile {
            expected: operation.profile(),
        },
        receipt_context: ReceiptContext {
            request_binding: Some(request_binding),
            ..bounded_context
        },
    })
}

fn prepare_prospective_authority(
    policy: &Policy,
    trusted_policy_ref: &BasaltPolicyRef,
    request: &UcanEnforcementRequest<'_, '_>,
    boundaries: &UcanEnforcementBoundaries<'_>,
    prepared: &PreparedRequest,
) -> Result<ProspectiveAuthority, UcanEnforcementFailure> {
    let operation = prepared.operation;
    let receipt_context = prepared.receipt_context;
    let verified = verify_compact_token_str_with_resolvers_and_revocations(
        request.compact_token,
        boundaries.verification_time,
        boundaries.key_resolver,
        boundaries.proof_store,
        boundaries.revocations,
        boundaries.verification_limits,
    )
    .map_err(|error| failure(token_error_reason(&error), receipt_context))?;
    prepared
        .profile
        .admit_operation_profile(&operation.profile())
        .map_err(|error| failure(profile_error_reason(error), receipt_context))?;
    let authorization = verified.authorize_with_policies(
        request.enforcement.resource(),
        request.enforcement.ability(),
        boundaries.caveat_policies,
    );
    let AuthorizationDecision::Allowed { .. } = authorization else {
        let reason =
            authorization.denial_reason().map_or(UcanEnforcementReason::ScopeDenied, authorization_error_reason);
        return Err(failure(reason, receipt_context));
    };
    let decision = build_authority_decision(
        policy,
        trusted_policy_ref,
        request.enforcement,
        &verified,
        boundaries.verification_time,
        &receipt_context,
    )
    .map_err(|reason| failure(reason, receipt_context))?;
    if !decision.is_allowed() {
        return Err(failure(UcanEnforcementReason::PolicyDenied, receipt_context));
    }
    let receipt = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context);
    Ok(ProspectiveAuthority {
        decision,
        receipt,
        operation,
        receipt_context,
    })
}
