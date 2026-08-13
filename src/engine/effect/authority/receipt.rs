//! Deterministic attempt, outcome, and replay receipts.

const ATTEMPT_IDENTITY_DOMAIN: &[u8] = b"lattice.choregraph-effect-attempt.v1";
const OUTCOME_IDENTITY_DOMAIN: &[u8] = b"lattice.choregraph-effect-outcome.v1";
const FIELD_SEPARATOR: &[u8] = &[0];
const INITIAL_ATTEMPT_SEQUENCE: u64 = 1;

pub fn reserve_attempt(
    binding: super::IntentBinding,
    existing: &std::collections::BTreeSet<super::AttemptIdentity>,
) -> Result<super::Attempt, super::AttemptError> {
    let identity = super::AttemptIdentity::new(blake3_reference(ATTEMPT_IDENTITY_DOMAIN, &binding_fields(&binding)))
        .map_err(|_| super::AttemptError::MalformedBinding)?;
    if existing.contains(&identity) {
        return Err(super::AttemptError::Duplicate);
    }
    Ok(super::Attempt {
        identity,
        binding,
        sequence: INITIAL_ATTEMPT_SEQUENCE,
    })
}

#[must_use]
pub fn outcome_event(
    attempt: &super::Attempt,
    decision: &super::AuthorizationDecision,
) -> super::AuthorizationOutcomeEvent {
    let state = if decision.allowed {
        super::OutcomeState::Authorized
    } else {
        super::OutcomeState::Denied
    };
    outcome_event_for_state(attempt, state, decision.denial)
}

#[must_use]
pub fn observed_outcome_event(
    attempt: &super::Attempt,
    state: super::OutcomeState,
    denial: Option<super::AuthorizationDenial>,
) -> super::AuthorizationOutcomeEvent {
    debug_assert!(matches!(
        state,
        super::OutcomeState::ObservedSuccess
            | super::OutcomeState::ObservedFailure
            | super::OutcomeState::UnknownAfterDispatch
    ));
    outcome_event_for_state(attempt, state, denial)
}

fn outcome_event_for_state(
    attempt: &super::Attempt,
    state: super::OutcomeState,
    denial: Option<super::AuthorizationDenial>,
) -> super::AuthorizationOutcomeEvent {
    let fields = [
        attempt.identity.as_str(),
        outcome_state_label(state),
        denial.map_or("none", super::AuthorizationDenial::as_str),
    ];
    super::AuthorizationOutcomeEvent {
        identity: blake3_reference(OUTCOME_IDENTITY_DOMAIN, &fields),
        attempt: attempt.identity.clone(),
        state,
        denial,
    }
}

fn outcome_state_label(state: super::OutcomeState) -> &'static str {
    match state {
        super::OutcomeState::Authorized => "authorized",
        super::OutcomeState::Denied => "denied",
        super::OutcomeState::ObservedSuccess => "observed-success",
        super::OutcomeState::ObservedFailure => "observed-failure",
        super::OutcomeState::UnknownAfterDispatch => "unknown-after-dispatch",
    }
}

fn binding_fields(binding: &super::IntentBinding) -> [&str; 14] {
    [
        binding.machine.as_str(),
        binding.workflow_revision.as_str(),
        binding.run.as_str(),
        binding.node.as_str(),
        binding.transition.as_str(),
        binding.operation.as_str(),
        binding.resource.as_str(),
        binding.ability.as_str(),
        binding.request.as_str(),
        binding.payload.as_str(),
        binding.policy.as_str(),
        binding.authority.as_str(),
        binding.intent.as_str(),
        scope_identity(&binding.scope),
    ]
}

fn scope_identity(scope: &super::CollaborationScopeBinding) -> &str {
    match scope {
        super::CollaborationScopeBinding::Unscoped => super::UNSCOPED_SCOPE_IDENTITY,
        super::CollaborationScopeBinding::Protected { snapshot, .. } => snapshot.as_str(),
    }
}

#[must_use]
pub fn blake3_reference(domain: &[u8], fields: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for field in fields {
        hasher.update(FIELD_SEPARATOR);
        hasher.update(field.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

// r[impl lattice.choregraph_effect_authority.replay]
#[must_use]
pub fn replay_observation(
    binding: &super::IntentBinding,
    prior: Option<&super::AuthorizationDecision>,
) -> super::AuthorizationDecision {
    let prior_diagnostic = prior
        .and_then(|decision| decision.denial)
        .map_or_else(|| "historical-allowance-is-not-authority".to_owned(), |denial| denial.as_str().to_owned());
    let _ = binding;
    super::evaluation::denied(super::AuthorizationDenial::ReplayDenied, Some(prior_diagnostic))
}

#[must_use]
pub fn basalt_diagnostic(decision: &basalt::authority_core::AuthorityDecision) -> Option<String> {
    decision.diagnostic.map(|diagnostic| diagnostic.as_str().to_owned())
}
