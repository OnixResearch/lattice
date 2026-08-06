const MAX_RECEIPT_COUNT: u32 = u32::MAX;

/// Borrowed inputs for one guarded-action authorization decision.
#[derive(Clone, Copy)]
pub struct AuthorizationContext<'a> {
    /// Normalized capability request.
    pub request: &'a super::CapabilityRequest,
    /// Nickel-reviewed policy contract fixture.
    pub contract: &'a super::CapabilityPolicyContract,
    /// Verified UCAN grant set available to this request.
    pub grants: &'a [super::UcanGrant],
    /// Local user/project/session/harness caveats.
    pub caveats: &'a [super::LocalCaveat],
    /// Optional sandbox execution constraint.
    pub sandbox: &'a super::SandboxConstraint,
}

struct DecisionEvaluation<'a> {
    context: &'a AuthorizationContext<'a>,
    matching_grants: u32,
    caveats: super::CaveatSummary,
    issue: super::AuthorizationIssue,
}

/// Evaluates a guarded action against contract, UCAN, local caveat, and sandbox checks.
#[must_use]
pub fn authorize_capability(context: &AuthorizationContext<'_>) -> super::AuthorizationReceipt {
    let matching_grants =
        receipt_count(context.grants.iter().filter(|grant| grant.authorizes(context.request)).count());
    let caveats = summarize_caveats(context.caveats);
    let issue = authorization_issue(context, matching_grants, caveats);
    build_receipt(&DecisionEvaluation {
        context,
        matching_grants,
        caveats,
        issue,
    })
}

fn authorization_issue(
    context: &AuthorizationContext<'_>,
    matching_grants: u32,
    caveats: super::CaveatSummary,
) -> super::AuthorizationIssue {
    debug_assert!(matching_grants <= receipt_count(context.grants.len()));
    debug_assert!(caveats.allows() <= receipt_count(context.caveats.len()));
    debug_assert!(caveats.denies() <= receipt_count(context.caveats.len()));
    debug_assert!(caveats.asks() <= receipt_count(context.caveats.len()));
    if !context.contract.permits(context.request) {
        return super::AuthorizationIssue::ContractMismatch;
    }
    if caveats.denies() > 0 {
        return super::AuthorizationIssue::LocalPolicyDenied;
    }
    if matching_grants == 0 {
        return super::AuthorizationIssue::MissingUcanGrant;
    }
    match context.sandbox {
        super::SandboxConstraint::NotRequired
        | super::SandboxConstraint::Required {
            state: super::SandboxState::Enforceable,
            ..
        } => super::AuthorizationIssue::None,
        super::SandboxConstraint::Required {
            state: super::SandboxState::Malformed,
            ..
        } => super::AuthorizationIssue::SandboxMalformed,
        super::SandboxConstraint::Required {
            state: super::SandboxState::Unavailable,
            ..
        } => super::AuthorizationIssue::SandboxUnavailable,
    }
}

fn build_receipt(evaluation: &DecisionEvaluation<'_>) -> super::AuthorizationReceipt {
    debug_assert!(evaluation.matching_grants <= receipt_count(evaluation.context.grants.len()));
    debug_assert!(evaluation.caveats.denies() <= receipt_count(evaluation.context.caveats.len()));
    let request = evaluation.context.request;
    let contract = evaluation.context.contract;
    let status = if matches!(evaluation.issue, super::AuthorizationIssue::None) {
        super::AuthorizationStatus::Allowed
    } else {
        super::AuthorizationStatus::Denied
    };
    let resource_hash = super::schema::hash_parts(&[request.resource()]);
    let sandbox_hash = sandbox_hash(evaluation.context.sandbox);
    let checked_grants = evaluation.matching_grants;
    debug_assert!(!resource_hash.is_empty());
    debug_assert!(!contract.policy_hash().is_empty());
    let receipt_hash = super::evidence::receipt_hash(&super::evidence::ReceiptHashFields {
        status,
        issue: evaluation.issue,
        request,
        contract,
        resource_hash: resource_hash.as_str(),
        checked_grants,
        caveats: evaluation.caveats,
        sandbox_hash: sandbox_hash.as_ref(),
    });

    super::AuthorizationReceipt::new(super::evidence::ReceiptFields {
        status,
        issue: evaluation.issue,
        action: request.action(),
        contract: alloc::string::String::from(request.contract()),
        ability: alloc::string::String::from(request.ability()),
        resource_hash,
        policy_hash: alloc::string::String::from(contract.policy_hash()),
        checked_grants,
        caveats: evaluation.caveats,
        sandbox_hash,
        receipt_hash,
    })
}

fn receipt_count(count: usize) -> u32 {
    u32::try_from(count).unwrap_or(MAX_RECEIPT_COUNT)
}

const fn increment_receipt_count(count: u32) -> u32 {
    count.saturating_add(1)
}

fn summarize_caveats(caveats: &[super::LocalCaveat]) -> super::CaveatSummary {
    let mut allows = 0;
    let mut denies = 0;
    let mut asks = 0;
    for caveat in caveats {
        match caveat.decision() {
            super::LocalCaveatDecision::Allow => allows = increment_receipt_count(allows),
            super::LocalCaveatDecision::Deny => denies = increment_receipt_count(denies),
            super::LocalCaveatDecision::Ask => asks = increment_receipt_count(asks),
        }
    }
    super::CaveatSummary { allows, denies, asks }
}

fn sandbox_hash(sandbox: &super::SandboxConstraint) -> Option<super::super::SafeHash> {
    match sandbox {
        super::SandboxConstraint::NotRequired => None,
        super::SandboxConstraint::Required { descriptor_hash, .. } => Some(descriptor_hash.clone()),
    }
}
