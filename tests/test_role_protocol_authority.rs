mod common;

use std::collections::BTreeSet;

use lattice::engine::effect::authority;
use lattice::engine::effect::dispatch;
use lattice::engine::protocol::ActionRequest;
use lattice::engine::protocol::EffectAuthority;
use lattice::engine::protocol::Issue;
use lattice::engine::protocol::LabelId;
use lattice::engine::protocol::create_session;
use lattice::engine::protocol::plan_action;
use lattice::engine::protocol::plan_session;
use lattice::engine::protocol::prepare_attempt;
use lattice::engine::protocol::shell::ProtocolAuthority;

const PROTOCOL_CONTRACT: &str = "lattice-role-protocol";
const GRANT_REFERENCE: &str = "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const RECEIPT_REFERENCE: &str = "blake3:4444444444444444444444444444444444444444444444444444444444444444";

struct ProtocolFacts;

impl dispatch::AuthorityFactsProvider for ProtocolFacts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        let resource = binding.resource.as_str().to_owned();
        let ability = binding.ability.as_str().to_owned();
        let request_ref = binding.request.as_str().to_owned();
        Ok(authority::AuthorizationFacts {
            binding: binding.clone(),
            basalt_input: basalt::authority_core::AuthorityInput {
                schema: basalt::authority_core::AUTHORITY_INPUT_SCHEMA.to_owned(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: PROTOCOL_CONTRACT.to_owned(),
                    resource: resource.clone(),
                    ability: ability.clone(),
                    request_ref: request_ref.clone(),
                },
                verified_grants: vec![basalt::authority_core::VerifiedGrantRef {
                    grant_ref: GRANT_REFERENCE.to_owned(),
                    verification_receipt_ref: RECEIPT_REFERENCE.to_owned(),
                    resource,
                    ability,
                    kind: basalt::authority_core::VerifiedGrantKind::Direct,
                }],
                verification_receipts: vec![basalt::authority_core::UcanVerificationReceiptRef {
                    receipt_ref: RECEIPT_REFERENCE.to_owned(),
                    request_ref,
                    verified_grant_refs: vec![GRANT_REFERENCE.to_owned()],
                    disposition: basalt::authority_core::VerificationRefDisposition::Current,
                }],
                policy_refs: vec![dispatch::trusted_policy_ref(PROTOCOL_CONTRACT)],
                caveat: basalt::authority_core::AuthorityCaveatDisposition::Satisfied,
                replay: basalt::authority_core::AuthorityReplayDisposition::Admitted,
                claims: vec![basalt::authority_core::AuthorityClaim::ExactRequest],
                non_claims: basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
            },
            trusted_policy_ref: dispatch::trusted_policy_ref(PROTOCOL_CONTRACT),
            scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: authority::RevocationDisposition::Current,
        })
    }
}

struct NoScope;

impl dispatch::ScopeSnapshotProvider for NoScope {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
        Ok(None)
    }
}

#[derive(Default)]
struct MemoryAttempts {
    attempts: std::sync::Mutex<BTreeSet<String>>,
    outcomes: std::sync::Mutex<BTreeSet<String>>,
}

impl dispatch::EffectAttemptStore for MemoryAttempts {
    fn reserve(&self, binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
        let attempt =
            authority::reserve_attempt(binding, &BTreeSet::new()).map_err(|_| dispatch::Error::AttemptPersistence)?;
        let mut attempts = self.attempts.lock().map_err(|_| dispatch::Error::AttemptPersistence)?;
        if !attempts.insert(attempt.identity.as_str().to_owned()) {
            return Err(dispatch::Error::DuplicateAttempt);
        }
        Ok(attempt)
    }

    fn record_outcome(&self, event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
        self.outcomes
            .lock()
            .map_err(|_| dispatch::Error::OutcomePersistence)?
            .insert(event.identity.clone());
        Ok(())
    }
}

struct NoRoots;

impl dispatch::ConfiguredRootRegistry for NoRoots {
    fn contains_root(&self, _root_id: &str) -> bool {
        false
    }

    fn with_root(
        &self,
        _root_id: &str,
        _operation: &mut dyn FnMut(&cap_std::fs::Dir) -> Result<(), dispatch::Error>,
    ) -> Result<(), dispatch::Error> {
        Err(dispatch::Error::UnknownRoot)
    }
}

fn prepared() -> lattice::engine::protocol::SessionRecord {
    let cohort = common::protocol::cohort(common::protocol::native_source(common::protocol::transfer(
        common::protocol::ROLE_A,
        common::protocol::ROLE_B,
        common::protocol::LABEL_A,
        common::protocol::end(),
    )));
    let plan = match plan_session(
        cohort.clone(),
        common::protocol::native_placements(&cohort),
        common::protocol::identity("authority-session"),
    ) {
        Ok(plan) => plan,
        Err(error) => panic!("plan failed: {error:?}"),
    };
    let session = match create_session(plan) {
        Ok(session) => session,
        Err(error) => panic!("session failed: {error:?}"),
    };
    let action = match plan_action(&session, ActionRequest {
        source: common::protocol::lattice_role(common::protocol::ROLE_A),
        label: LabelId(common::protocol::LABEL_A.0),
        payload_reference: common::protocol::identity("authority-payload"),
    }) {
        Ok(action) => action,
        Err(error) => panic!("action failed: {error:?}"),
    };
    match prepare_attempt(&session, action) {
        Ok(prepared) => prepared,
        Err(error) => panic!("prepare failed: {error:?}"),
    }
}

// r[verify lattice.role_protocol.authorization.replayed]
#[test]
fn exact_protocol_envelope_uses_current_effect_authority() {
    let dispatch = dispatch::Shell::new(
        Box::new(ProtocolFacts),
        Box::new(NoScope),
        Box::new(MemoryAttempts::default()),
        Box::new(NoRoots),
    );
    let authority = EffectAuthority::new(
        &dispatch,
        authority::AuthorityIdentity::new(common::protocol::identity("authority").as_str())
            .unwrap_or_else(|error| panic!("authority identity failed: {error:?}")),
        authority::CollaborationScopeBinding::Unscoped,
    );
    let prepared = prepared();
    let envelope = &prepared.attempts[0].messages[0];
    let binding = match authority.binding(&prepared, envelope) {
        Ok(binding) => binding,
        Err(error) => panic!("binding failed: {error:?}"),
    };
    assert_eq!(binding.operation.as_str(), "lattice/protocol");
    assert_eq!(binding.ability.as_str(), "protocol/transport");
    assert_eq!(binding.payload.as_str(), envelope.identity.as_str());
    assert!(binding.resource.as_str().starts_with("lattice://role-protocol/"));
    assert!(authority.authorize(&prepared, envelope).is_ok());
}

#[test]
fn stale_envelope_binding_cannot_authorize_a_new_message() {
    let dispatch = dispatch::Shell::new(
        Box::new(ProtocolFacts),
        Box::new(NoScope),
        Box::new(MemoryAttempts::default()),
        Box::new(NoRoots),
    );
    let authority = EffectAuthority::new(
        &dispatch,
        authority::AuthorityIdentity::new(common::protocol::identity("authority").as_str())
            .unwrap_or_else(|error| panic!("authority identity failed: {error:?}")),
        authority::CollaborationScopeBinding::Unscoped,
    );
    let prepared = prepared();
    let mut stale = prepared.attempts[0].messages[0].clone();
    stale.identity = common::protocol::identity("stale-message");
    let stale_binding = authority.binding(&prepared, &stale);
    assert!(stale_binding.is_ok());
    assert_eq!(
        lattice::engine::protocol::admit_observation(
            &prepared,
            &stale,
            true,
            common::protocol::identity("stale-observation"),
            None,
        ),
        Err(lattice::engine::protocol::Error::Issue(Issue::IdentityMismatch))
    );
}
