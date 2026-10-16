//! Pure Basalt and collaboration-scope evaluation.

// r[impl lattice.choregraph_effect_authority.basalt]
// r[impl lattice.choregraph_effect_authority.scope_currentness]
pub fn evaluate(facts: &super::AuthorizationFacts) -> super::AuthorizationDecision {
    evaluate_for(&facts.binding, facts)
}

pub fn evaluate_for(
    expected: &super::IntentBinding,
    facts: &super::AuthorizationFacts,
) -> super::AuthorizationDecision {
    debug_assert!(!expected.schema.is_empty());
    debug_assert!(!facts.binding.schema.is_empty());
    if &facts.binding != expected {
        return denied(super::AuthorizationDenial::BindingMismatch, None);
    }
    if let Some(denial) = binding_denial(facts) {
        return denied(denial, None);
    }
    if facts.scope_requirement == super::ScopeRequirement::ProtectedRequired
        && matches!(facts.binding.scope, super::CollaborationScopeBinding::Unscoped)
    {
        return denied(super::AuthorizationDenial::ScopeRequired, None);
    }
    if let Err(denial) = validate_scope(&facts.binding.scope, facts.scope_snapshot.as_ref(), &facts.binding) {
        return denied(denial, None);
    }
    if facts.revocation != super::RevocationDisposition::Current {
        return denied(super::AuthorizationDenial::RevocationDenied, None);
    }
    if let super::Evidence::VerifiedUcan(input) = &facts.evidence
        && input.replay != basalt::authority_core::AuthorityReplayDisposition::Admitted
    {
        return denied(super::AuthorizationDenial::ReplayDenied, None);
    }
    let policy = match basalt::parse_policy_json(super::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_JSON) {
        Ok(policy) => policy,
        Err(_) => return denied(super::AuthorizationDenial::AuthorityUnavailable, None),
    };
    if let Some(decision) = evidence_denial(&policy, facts) {
        return decision;
    }
    super::AuthorizationDecision {
        allowed: true,
        binding: Some(facts.binding.clone()),
        denial: None,
        basalt_diagnostic: None,
    }
}

fn binding_denial(facts: &super::AuthorizationFacts) -> Option<super::AuthorizationDenial> {
    if facts.binding.schema != super::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA
        || facts.binding.policy.as_str() != super::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF
    {
        return Some(super::AuthorizationDenial::MalformedBinding);
    }
    if !request_matches_binding(&facts.binding, &facts.evidence) {
        return Some(super::AuthorizationDenial::BindingMismatch);
    }
    if facts.trusted_policy_ref.artifact_ref != facts.binding.policy.as_str()
        || facts.trusted_policy_ref.schema_version != super::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_SCHEMA
        || facts.trusted_policy_ref.contract_id != evidence_request(&facts.evidence).contract_id
    {
        return Some(super::AuthorizationDenial::PolicyMismatch);
    }
    None
}

fn evidence_request(evidence: &super::Evidence) -> &basalt::authority_core::AuthorityRequest {
    match evidence {
        super::Evidence::VerifiedUcan(input) => &input.request,
        super::Evidence::LocalOperator(grant) => &grant.request,
    }
}

fn request_matches_binding(binding: &super::IntentBinding, evidence: &super::Evidence) -> bool {
    let request = evidence_request(evidence);
    let is_request_match = request.resource == binding.resource.as_str()
        && request.ability == binding.ability.as_str()
        && request.request_ref == binding.request.as_str();
    let is_evidence_match = match evidence {
        super::Evidence::VerifiedUcan(input) => {
            input.policy_refs.len() == 1 && input.policy_refs[0].artifact_ref == binding.policy.as_str()
        }
        super::Evidence::LocalOperator(grant) => {
            grant.authority_ref == binding.authority.as_str() && grant.operation == binding.operation.as_str()
        }
    };
    is_request_match && is_evidence_match
}

// r[impl lattice.standalone_effect_authority.provenance]
fn evidence_denial(policy: &basalt::Policy, facts: &super::AuthorizationFacts) -> Option<super::AuthorizationDecision> {
    match &facts.evidence {
        super::Evidence::VerifiedUcan(input) => {
            let decision = basalt::authority_core::evaluate_authority(policy, &facts.trusted_policy_ref, input);
            decision.is_denied().then(|| {
                denied(
                    super::AuthorizationDenial::BasaltDenied,
                    decision.diagnostic.map(|diagnostic| diagnostic.as_str().to_owned()),
                )
            })
        }
        super::Evidence::LocalOperator(grant) => {
            let request = basalt::EnforcementRequest::new(
                &grant.request.contract_id,
                facts.binding.resource.as_str(),
                facts.binding.ability.as_str(),
            )
            .with_capabilities(vec![basalt::CapabilityGrant::new(
                &grant.request.resource,
                &grant.request.ability,
            )]);
            match basalt::enforce(policy, &request) {
                Ok(receipt) if receipt.is_allowed() => None,
                _ => Some(denied(super::AuthorizationDenial::BasaltDenied, None)),
            }
        }
    }
}

fn validate_scope(
    expected: &super::CollaborationScopeBinding,
    observed: Option<&super::ScopeSnapshotFacts>,
    binding: &super::IntentBinding,
) -> Result<(), super::AuthorizationDenial> {
    match (expected, observed) {
        (super::CollaborationScopeBinding::Unscoped, None) => Ok(()),
        (super::CollaborationScopeBinding::Unscoped, Some(_)) => Err(super::AuthorizationDenial::ScopeMismatch),
        (super::CollaborationScopeBinding::Protected { .. }, None) => Err(super::AuthorizationDenial::ScopeRequired),
        (
            super::CollaborationScopeBinding::Protected {
                scope,
                revision,
                membership_epoch,
                audience,
                required_consent,
                snapshot,
            },
            Some(observed),
        ) => validate_protected_scope(
            ProtectedScopeExpectation {
                scope,
                revision,
                membership_epoch: *membership_epoch,
                audience,
                required_consent,
                snapshot,
            },
            observed,
            binding,
        ),
    }
}

struct ProtectedScopeExpectation<'a> {
    scope: &'a super::ScopeIdentity,
    revision: &'a super::ScopeRevisionIdentity,
    membership_epoch: u64,
    audience: &'a super::AudienceIdentity,
    required_consent: &'a super::ConsentDecisionIdentity,
    snapshot: &'a super::ScopeSnapshotIdentity,
}

fn validate_protected_scope(
    expected: ProtectedScopeExpectation<'_>,
    observed: &super::ScopeSnapshotFacts,
    binding: &super::IntentBinding,
) -> Result<(), super::AuthorizationDenial> {
    debug_assert!(!expected.scope.as_str().is_empty());
    debug_assert!(!expected.revision.as_str().is_empty());
    let is_scope_match = observed.schema == super::CHOREGRAPH_EFFECT_SCOPE_SCHEMA
        && observed.scope == *expected.scope
        && observed.revision == *expected.revision
        && observed.membership_epoch == expected.membership_epoch
        && observed.audience == *expected.audience
        && observed.consent == *expected.required_consent
        && observed.snapshot == *expected.snapshot;
    let is_request_match = observed.resource == binding.resource
        && observed.ability == binding.ability
        && observed.request == binding.request;
    if !is_scope_match || !is_request_match {
        return Err(super::AuthorizationDenial::ScopeMismatch);
    }
    let is_scope_current = observed.membership_current
        && observed.audience_current
        && observed.consent_current
        && observed.revocation_current;
    if !is_scope_current {
        return Err(super::AuthorizationDenial::StaleScope);
    }
    Ok(())
}

pub(super) fn denied(
    denial: super::AuthorizationDenial,
    basalt_diagnostic: Option<String>,
) -> super::AuthorizationDecision {
    super::AuthorizationDecision {
        allowed: false,
        binding: None,
        denial: Some(denial),
        basalt_diagnostic,
    }
}
