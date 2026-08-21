use std::collections::BTreeMap;

use basalt::authority_core::AuthorityCaveatDisposition;
use basalt::authority_core::AuthorityClaim;
use basalt::authority_core::AuthorityInput;
use basalt::authority_core::AuthorityReplayDisposition;
use basalt::authority_core::AuthorityRequest;
use basalt::authority_core::UcanVerificationReceiptRef;
use basalt::authority_core::VerificationRefDisposition;
use basalt::authority_core::VerifiedGrantKind;
use basalt::authority_core::VerifiedGrantRef;
use lattice::engine::effect::authority;
use lattice::engine::effect::dispatch;
use lattice::engine::effect::dispatch::ConfiguredRootRegistry;
use lattice::engine::effect::dispatch::EffectAttemptStore;
use lattice::engine::effect::runtime::Adapters;
use lattice::engine::effect::runtime::FileAdapterOutput;
use lattice::engine::effect::runtime::FileAdapterRequest;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::store::EffectAttemptRepository;
use lattice::workflow_format::choregraph as choregraph_format;
use lattice::workflow_format::workflow_to_portable;

// r[verify lattice.choregraph_effect_authority.fixtures]
// r[verify lattice.choregraph_effect_authority.validation]
// r[verify lattice.choregraph_effect_authority.non_claims]

const REQUEST_REF: &str = "blake3:1111111111111111111111111111111111111111111111111111111111111111";
const PAYLOAD_REF: &str = "blake3:2222222222222222222222222222222222222222222222222222222222222222";
const GRANT_REF: &str = "blake3:3333333333333333333333333333333333333333333333333333333333333333";
const RECEIPT_REF: &str = "blake3:4444444444444444444444444444444444444444444444444444444444444444";
const AUTHORITY_REF: &str = "blake3:5555555555555555555555555555555555555555555555555555555555555555";
const MACHINE_REF: &str = "blake3:6666666666666666666666666666666666666666666666666666666666666666";
const REVISION_REF: &str = "blake3:7777777777777777777777777777777777777777777777777777777777777777";
const SNAPSHOT_REF: &str = "blake3:8888888888888888888888888888888888888888888888888888888888888888";
const CONSENT_REF: &str = "blake3:9999999999999999999999999999999999999999999999999999999999999999";
const RUN_ID: &str = "run-7";
const NODE_ID: &str = "7";
const TRANSITION_ID: &str = "Ready->Start";
const OPERATION_ID: &str = "File";
const FILE_RESOURCE: &str = "lattice://file/7";
const FILE_ABILITY: &str = "file/write";
const FILE_CONTRACT: &str = "lattice-file";
const ROOT_ID: &str = "workspace";
const NETWORK_RESOURCE: &str = "lattice://network/7";
const NETWORK_ABILITY: &str = "network/request";
const PROCESS_RESOURCE: &str = "lattice://process/7";
const PROCESS_ABILITY: &str = "process/execute";
const TEST_FILE_NAME: &str = "allowed.txt";
const TEST_FILE_CONTENT: &[u8] = b"authorized";
const HTTP_URL: &str = "https://example.test/resource";
const HTTP_TIMEOUT_SECS: u64 = 5;
const HTTP_STATUS_OK: u16 = 200;
const HTTP_BODY_OUTPUT_INDEX: usize = 2;
const PROCESS_TIMEOUT_SECS: u64 = 5;
const PROCESS_OUTPUT_LIMIT_BYTES: usize = 1_024;
const FILE_READ_LIMIT_BYTES: usize = 1_024;
const SCOPE_ID: &str = "scope-7";
const SCOPE_REVISION: &str = "scope-revision-7";
const MEMBERSHIP_EPOCH: u64 = 7;
const AUDIENCE_ID: &str = "audience-7";
const ATTEMPT_SEQUENCE: u64 = 1;
const END_TO_END_WORKFLOW_NAME: &str = "authorized-network-effect";
const END_TO_END_NODE_LABEL: &str = "network";
const END_TO_END_MACHINE_REF: &str = "machine-authorized-network";
const END_TO_END_REVISION_REF: &str = "revision-authorized-network";
const END_TO_END_RUN_REF: &str = "run-authorized-network";
const END_TO_END_AUTHORITY_REF: &str = "authority-authorized-network";
const END_TO_END_DECLARATION_PREFIX: &str = "lattice-effect-";
const END_TO_END_CHOREGRAPH_NODE_PREFIX: &str = "lattice-node/";
const END_TO_END_SUCCESS_BODY: &str = "authorized";

fn id<T>(constructor: fn(String) -> Result<T, authority::BindingError>, value: &str) -> T {
    constructor(value.to_owned()).expect("valid identity")
}

fn binding(resource: &str, ability: &str, scope: authority::CollaborationScopeBinding) -> authority::IntentBinding {
    binding_with_payload(resource, ability, scope, PAYLOAD_REF)
}

fn binding_with_payload(
    resource: &str,
    ability: &str,
    scope: authority::CollaborationScopeBinding,
    payload: &str,
) -> authority::IntentBinding {
    authority::IntentBinding {
        schema: authority::CHOREGRAPH_EFFECT_AUTHORITY_BINDING_SCHEMA.to_owned(),
        machine: id(authority::MachineIdentity::new, MACHINE_REF),
        workflow_revision: id(authority::WorkflowRevisionIdentity::new, REVISION_REF),
        run: id(authority::RunIdentity::new, RUN_ID),
        node: id(authority::NodeIdentity::new, NODE_ID),
        transition: id(authority::TransitionIdentity::new, TRANSITION_ID),
        operation: id(authority::OperationIdentity::new, OPERATION_ID),
        resource: id(authority::ResourceIdentity::new, resource),
        ability: id(authority::AbilityIdentity::new, ability),
        request: id(authority::RequestIdentity::new, REQUEST_REF),
        payload: id(authority::PayloadIdentity::new, payload),
        policy: id(authority::PolicyIdentity::new, authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF),
        authority: id(authority::AuthorityIdentity::new, AUTHORITY_REF),
        intent: id(authority::IntentIdentity::new, GRANT_REF),
        scope,
    }
}

fn protected_scope() -> authority::CollaborationScopeBinding {
    authority::CollaborationScopeBinding::Protected {
        scope: id(authority::ScopeIdentity::new, SCOPE_ID),
        revision: id(authority::ScopeRevisionIdentity::new, SCOPE_REVISION),
        membership_epoch: MEMBERSHIP_EPOCH,
        audience: id(authority::AudienceIdentity::new, AUDIENCE_ID),
        required_consent: id(authority::ConsentDecisionIdentity::new, CONSENT_REF),
        snapshot: id(authority::ScopeSnapshotIdentity::new, SNAPSHOT_REF),
    }
}

fn scope_snapshot(resource: &str, ability: &str) -> authority::ScopeSnapshotFacts {
    authority::ScopeSnapshotFacts {
        schema: authority::CHOREGRAPH_EFFECT_SCOPE_SCHEMA.to_owned(),
        scope: id(authority::ScopeIdentity::new, SCOPE_ID),
        revision: id(authority::ScopeRevisionIdentity::new, SCOPE_REVISION),
        membership_epoch: MEMBERSHIP_EPOCH,
        audience: id(authority::AudienceIdentity::new, AUDIENCE_ID),
        consent: id(authority::ConsentDecisionIdentity::new, CONSENT_REF),
        resource: id(authority::ResourceIdentity::new, resource),
        ability: id(authority::AbilityIdentity::new, ability),
        request: id(authority::RequestIdentity::new, REQUEST_REF),
        snapshot: id(authority::ScopeSnapshotIdentity::new, SNAPSHOT_REF),
        membership_current: true,
        audience_current: true,
        consent_current: true,
        revocation_current: true,
    }
}

fn basalt_input(resource: &str, ability: &str, contract: &str) -> AuthorityInput {
    AuthorityInput {
        schema: basalt::authority_core::AUTHORITY_INPUT_SCHEMA.to_owned(),
        request: AuthorityRequest {
            contract_id: contract.to_owned(),
            resource: resource.to_owned(),
            ability: ability.to_owned(),
            request_ref: REQUEST_REF.to_owned(),
        },
        verified_grants: vec![VerifiedGrantRef {
            grant_ref: GRANT_REF.to_owned(),
            verification_receipt_ref: RECEIPT_REF.to_owned(),
            resource: resource.to_owned(),
            ability: ability.to_owned(),
            kind: VerifiedGrantKind::Direct,
        }],
        verification_receipts: vec![UcanVerificationReceiptRef {
            receipt_ref: RECEIPT_REF.to_owned(),
            request_ref: REQUEST_REF.to_owned(),
            verified_grant_refs: vec![GRANT_REF.to_owned()],
            disposition: VerificationRefDisposition::Current,
        }],
        policy_refs: vec![dispatch::trusted_policy_ref(contract)],
        caveat: AuthorityCaveatDisposition::Satisfied,
        replay: AuthorityReplayDisposition::Admitted,
        claims: vec![AuthorityClaim::ExactRequest],
        non_claims: basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned).to_vec(),
    }
}

fn facts(scope: authority::CollaborationScopeBinding) -> authority::AuthorizationFacts {
    let binding = binding(FILE_RESOURCE, FILE_ABILITY, scope);
    authority::AuthorizationFacts {
        binding,
        basalt_input: basalt_input(FILE_RESOURCE, FILE_ABILITY, FILE_CONTRACT),
        trusted_policy_ref: dispatch::trusted_policy_ref(FILE_CONTRACT),
        scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
        scope_snapshot: None,
        revocation: authority::RevocationDisposition::Current,
    }
}

#[test]
fn allows_exact_unscoped_file_authority() {
    let decision = authority::evaluate(&facts(authority::CollaborationScopeBinding::Unscoped));

    assert!(decision.allowed);
    assert!(decision.binding.is_some());
    assert_eq!(decision.denial, None);
}

#[test]
fn allows_current_scope_protected_authority() {
    let mut facts = facts(protected_scope());
    facts.scope_snapshot = Some(scope_snapshot(FILE_RESOURCE, FILE_ABILITY));

    let decision = authority::evaluate(&facts);

    assert!(decision.allowed);
    assert_eq!(decision.denial, None);
}

#[test]
fn denies_missing_wrong_and_replayed_authority() {
    let mut missing = facts(authority::CollaborationScopeBinding::Unscoped);
    missing.basalt_input.verified_grants.clear();
    let mut wrong_resource = facts(authority::CollaborationScopeBinding::Unscoped);
    wrong_resource.basalt_input.request.resource = "lattice://file/other".to_owned();
    let mut wrong_ability = facts(authority::CollaborationScopeBinding::Unscoped);
    wrong_ability.basalt_input.verified_grants[0].ability = "file/read".to_owned();
    let mut wrong_request = facts(authority::CollaborationScopeBinding::Unscoped);
    wrong_request.basalt_input.request.request_ref = RECEIPT_REF.to_owned();
    let mut stale_policy = facts(authority::CollaborationScopeBinding::Unscoped);
    stale_policy.trusted_policy_ref.artifact_ref = RECEIPT_REF.to_owned();
    let mut replayed = facts(authority::CollaborationScopeBinding::Unscoped);
    replayed.basalt_input.replay = AuthorityReplayDisposition::Denied;
    let mut revoked = facts(authority::CollaborationScopeBinding::Unscoped);
    revoked.revocation = authority::RevocationDisposition::Revoked;

    assert_eq!(authority::evaluate(&missing).denial, Some(authority::AuthorizationDenial::BasaltDenied));
    assert_eq!(authority::evaluate(&wrong_resource).denial, Some(authority::AuthorizationDenial::BindingMismatch));
    assert_eq!(authority::evaluate(&wrong_ability).denial, Some(authority::AuthorizationDenial::BasaltDenied));
    assert_eq!(authority::evaluate(&wrong_request).denial, Some(authority::AuthorizationDenial::BindingMismatch));
    assert_eq!(authority::evaluate(&stale_policy).denial, Some(authority::AuthorizationDenial::PolicyMismatch));
    assert_eq!(authority::evaluate(&replayed).denial, Some(authority::AuthorizationDenial::ReplayDenied));
    assert_eq!(authority::evaluate(&revoked).denial, Some(authority::AuthorizationDenial::RevocationDenied));
}

#[test]
fn denies_missing_stale_removed_audience_consent_and_revocation_scope_facts() {
    let missing = facts(protected_scope());
    assert_eq!(authority::evaluate(&missing).denial, Some(authority::AuthorizationDenial::ScopeRequired));
    let mut fabricated_unscoped = facts(authority::CollaborationScopeBinding::Unscoped);
    fabricated_unscoped.scope_requirement = authority::ScopeRequirement::ProtectedRequired;
    assert_eq!(
        authority::evaluate(&fabricated_unscoped).denial,
        Some(authority::AuthorizationDenial::ScopeRequired)
    );

    for mutate in [
        |snapshot: &mut authority::ScopeSnapshotFacts| snapshot.membership_epoch += ATTEMPT_SEQUENCE,
        |snapshot: &mut authority::ScopeSnapshotFacts| snapshot.membership_current = false,
        |snapshot: &mut authority::ScopeSnapshotFacts| snapshot.audience_current = false,
        |snapshot: &mut authority::ScopeSnapshotFacts| snapshot.consent_current = false,
        |snapshot: &mut authority::ScopeSnapshotFacts| snapshot.revocation_current = false,
    ] {
        let mut changed = facts(protected_scope());
        let mut snapshot = scope_snapshot(FILE_RESOURCE, FILE_ABILITY);
        mutate(&mut snapshot);
        changed.scope_snapshot = Some(snapshot);
        assert!(!authority::evaluate(&changed).allowed);
    }
}

#[test]
fn replayed_allowance_never_becomes_fresh_dispatch_authority() {
    let binding = binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped);
    let prior = authority::AuthorizationDecision {
        allowed: true,
        binding: Some(binding.clone()),
        denial: None,
        basalt_diagnostic: None,
    };

    let replay = authority::replay_observation(&binding, Some(&prior));

    assert!(!replay.allowed);
    assert_eq!(replay.denial, Some(authority::AuthorizationDenial::ReplayDenied));
    assert!(replay.binding.is_none());
}

struct FixedFacts {
    facts: authority::AuthorizationFacts,
}
impl dispatch::AuthorityFactsProvider for FixedFacts {
    fn load(&self, _binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        Ok(self.facts.clone())
    }
}

struct DynamicNetworkFacts;

impl dispatch::AuthorityFactsProvider for DynamicNetworkFacts {
    fn load(&self, binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        let mut authorization_facts = end_to_end_authorization_facts(binding.clone());
        authorization_facts.binding = binding.clone();
        authorization_facts.basalt_input.request.request_ref = binding.request.as_str().to_owned();
        authorization_facts.verification_receipt_request(binding.request.as_str());
        Ok(authorization_facts)
    }
}

trait AuthorizationFactsTestExtension {
    fn verification_receipt_request(&mut self, request_ref: &str);
}

impl AuthorizationFactsTestExtension for authority::AuthorizationFacts {
    fn verification_receipt_request(&mut self, request_ref: &str) {
        self.basalt_input.request.request_ref = request_ref.to_owned();
        for receipt in &mut self.basalt_input.verification_receipts {
            receipt.request_ref = request_ref.to_owned();
        }
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

struct FailingAttempts;
impl EffectAttemptStore for FailingAttempts {
    fn reserve(&self, _binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
        Err(dispatch::Error::AttemptPersistence)
    }
    fn record_outcome(&self, _event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
        Err(dispatch::Error::OutcomePersistence)
    }
}

#[derive(Clone)]
struct RecordingAttempts {
    recorded: std::sync::Arc<std::sync::Mutex<Vec<authority::AuthorizationOutcomeEvent>>>,
}
impl EffectAttemptStore for RecordingAttempts {
    fn reserve(&self, binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
        Ok(authority::Attempt {
            identity: id(authority::AttemptIdentity::new, GRANT_REF),
            binding,
            sequence: ATTEMPT_SEQUENCE,
        })
    }
    fn record_outcome(&self, event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
        self.recorded.lock().expect("record lock").push(event.clone());
        Ok(())
    }
}

#[test]
fn unavailable_authority_records_deterministic_denial() {
    let recorded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shell = dispatch::Shell::new(
        Box::new(UnavailableFacts),
        Box::new(NoScope),
        Box::new(RecordingAttempts {
            recorded: std::sync::Arc::clone(&recorded),
        }),
        Box::new(NoRoots),
    );
    let binding = binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped);

    assert_eq!(shell.authorize(binding), Err(dispatch::Error::AuthorityUnavailable));
    let events = recorded.lock().expect("record lock");
    assert_eq!(events.len(), ATTEMPT_SEQUENCE as usize);
    assert_eq!(events[0].denial, Some(authority::AuthorizationDenial::AuthorityUnavailable));
    assert_eq!(events[0].state, authority::OutcomeState::Denied);
}

// r[verify lattice.workflow_execution.ready_frontier.dispatch.scenario.persistence_failure]
#[test]
fn authority_mismatch_and_persistence_failure_stop_before_adapters() {
    let requested = binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped);
    let mut mismatched = facts(authority::CollaborationScopeBinding::Unscoped);
    mismatched.binding.run = id(authority::RunIdentity::new, "another-run");
    let mismatch = authority::evaluate_for(&requested, &mismatched);
    assert_eq!(mismatch.denial, Some(authority::AuthorizationDenial::BindingMismatch));

    let persistence_shell = dispatch::Shell::new(
        Box::new(FixedFacts {
            facts: facts(authority::CollaborationScopeBinding::Unscoped),
        }),
        Box::new(NoScope),
        Box::new(FailingAttempts),
        Box::new(NoRoots),
    );
    assert_eq!(persistence_shell.authorize(requested), Err(dispatch::Error::AttemptPersistence));
}

#[test]
fn dispatch_records_unknown_after_post_effect_persistence_failure() {
    #[derive(Clone)]
    struct FailObservedOutcome {
        unknown: std::sync::Arc<std::sync::Mutex<Vec<authority::AuthorizationOutcomeEvent>>>,
    }
    impl EffectAttemptStore for FailObservedOutcome {
        fn reserve(&self, binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
            Ok(authority::Attempt {
                identity: id(authority::AttemptIdentity::new, GRANT_REF),
                binding,
                sequence: ATTEMPT_SEQUENCE,
            })
        }
        fn record_outcome(&self, event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
            if event.state == authority::OutcomeState::UnknownAfterDispatch {
                self.unknown.lock().expect("unknown lock").push(event.clone());
                Ok(())
            } else {
                Err(dispatch::Error::OutcomePersistence)
            }
        }
    }

    let request = lattice::http::HttpRequestPlan {
        method: "GET".to_owned(),
        url: HTTP_URL.to_owned(),
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_secs: HTTP_TIMEOUT_SECS,
    };
    let binding = binding_with_payload(
        NETWORK_RESOURCE,
        NETWORK_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::network_plan_identity(&request),
    );
    let mut authorization_facts = facts(authority::CollaborationScopeBinding::Unscoped);
    authorization_facts.binding = binding.clone();
    authorization_facts.basalt_input = basalt_input(NETWORK_RESOURCE, NETWORK_ABILITY, "lattice-network");
    authorization_facts.trusted_policy_ref = dispatch::trusted_policy_ref("lattice-network");
    let unknown = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shell = dispatch::Shell::new(
        Box::new(FixedFacts {
            facts: authorization_facts,
        }),
        Box::new(NoScope),
        Box::new(FailObservedOutcome {
            unknown: std::sync::Arc::clone(&unknown),
        }),
        Box::new(NoRoots),
    );
    let http = lattice::http::StaticHttpClient::new(std::collections::HashMap::from([(
        HTTP_URL.to_owned(),
        lattice::http::StaticHttpResult::Response(lattice::http::HttpResponse {
            status: HTTP_STATUS_OK,
            headers: std::collections::HashMap::new(),
            body: "ok".to_owned(),
        }),
    )]));
    let process_config = lattice::process::core::RuntimeConfig::default();
    let runtime = Adapters::new(&shell, &http, &process_config);
    assert_eq!(shell.dispatch(&runtime, binding, &request), Err(dispatch::Error::OutcomePersistence));
    let events = unknown.lock().expect("unknown lock");
    assert_eq!(events.len(), ATTEMPT_SEQUENCE as usize);
    assert_eq!(events[0].state, authority::OutcomeState::UnknownAfterDispatch);
}

#[test]
fn store_reserves_before_effect_and_rejects_duplicates() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = lattice::store::Store::new(temp.path().join("effects.redb")).expect("store");
    let binding = binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped);

    let attempt = store.reserve_effect_attempt(binding.clone()).expect("reserve");
    assert!(store.load_effect_attempt(&attempt.identity).expect("load").is_some());
    assert!(store.reserve_effect_attempt(binding).is_err());
    let unknown = authority::observed_outcome_event(&attempt, authority::OutcomeState::UnknownAfterDispatch, None);
    store.save_effect_outcome(&unknown).expect("record unknown crash boundary");
    assert_eq!(unknown.state, authority::OutcomeState::UnknownAfterDispatch);
}

// r[verify lattice.workflow_execution.ready_frontier.recovery]
// r[verify lattice.workflow_execution.ready_frontier.recovery.scenario.uncertain_attempts]
#[test]
fn crash_keeps_each_concurrent_attempt_independently_uncertain() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = lattice::store::Store::new(temp.path().join("concurrent-effects.redb")).expect("store");
    let bindings = [
        binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped),
        binding(NETWORK_RESOURCE, NETWORK_ABILITY, authority::CollaborationScopeBinding::Unscoped),
    ];
    let attempts = bindings
        .into_iter()
        .map(|candidate| store.reserve_effect_attempt(candidate).expect("reserve independent attempt"))
        .collect::<Vec<_>>();

    for attempt in &attempts {
        let unknown = authority::observed_outcome_event(attempt, authority::OutcomeState::UnknownAfterDispatch, None);
        store.save_effect_outcome(&unknown).expect("record independent unknown outcome");
        let retained = store.load_effect_outcome(&unknown.identity).expect("load outcome").expect("outcome exists");
        assert_eq!(retained.attempt, attempt.identity);
        assert_eq!(retained.state, authority::OutcomeState::UnknownAfterDispatch);
    }
}

struct NoRoots;
impl ConfiguredRootRegistry for NoRoots {
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

#[test]
fn root_and_path_admission_fail_closed() {
    let authorized = dispatch::AuthorizedAttempt {
        attempt: authority::Attempt {
            identity: id(authority::AttemptIdentity::new, GRANT_REF),
            binding: binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped),
            sequence: ATTEMPT_SEQUENCE,
        },
        authorization: authority::AuthorizationDecision {
            allowed: true,
            binding: Some(binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped)),
            denial: None,
            basalt_diagnostic: None,
        },
    };
    struct Unavailable;
    impl dispatch::AuthorityFactsProvider for Unavailable {
        fn load(&self, _binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
            Err(dispatch::Error::AuthorityUnavailable)
        }
    }
    impl dispatch::ScopeSnapshotProvider for Unavailable {
        fn load_current(
            &self,
            _binding: &authority::IntentBinding,
        ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
            Err(dispatch::Error::AuthorityUnavailable)
        }
    }
    impl EffectAttemptStore for Unavailable {
        fn reserve(&self, _binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
            Err(dispatch::Error::AttemptPersistence)
        }
        fn record_outcome(&self, _event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
            Err(dispatch::Error::OutcomePersistence)
        }
    }
    let shell =
        dispatch::Shell::new(Box::new(Unavailable), Box::new(Unavailable), Box::new(Unavailable), Box::new(NoRoots));

    assert_eq!(
        shell.admit_file_path(&authorized, dispatch::FileTarget {
            root_id: ROOT_ID,
            relative_path: "file.txt",
        },),
        Err(dispatch::Error::UnknownRoot)
    );
    assert!(
        cap_root_core::RootRelativePath::new("../outside", cap_root_core::AdmissionMode::NonEmptyRelative).is_err()
    );
}

#[test]
fn file_adapter_rejects_symlink_escape() {
    use std::os::unix::fs::symlink;

    let root_dir = tempfile::tempdir().expect("root dir");
    let outside_dir = tempfile::tempdir().expect("outside dir");
    std::fs::write(outside_dir.path().join(TEST_FILE_NAME), TEST_FILE_CONTENT).expect("outside file");
    symlink(outside_dir.path(), root_dir.path().join("escape")).expect("symlink");
    let request = FileAdapterRequest {
        root_id: ROOT_ID.to_owned(),
        relative_path: format!("escape/{TEST_FILE_NAME}"),
        content: None,
        overwrite: false,
        read_max_bytes: FILE_READ_LIMIT_BYTES,
    };
    let file_binding = binding_with_payload(
        FILE_RESOURCE,
        "file/read",
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::file_plan_identity(&request),
    );
    let root = cap_std::fs::Dir::open_ambient_dir(root_dir.path(), cap_std::ambient_authority()).expect("test root");
    let shell = dispatch::Shell::new(
        Box::new(UnavailableFacts),
        Box::new(UnavailableFacts),
        Box::new(UnavailableAttempts),
        Box::new(lattice::engine::effect::runtime::ConfiguredRoots::new(BTreeMap::from([(ROOT_ID.to_owned(), root)]))),
    );
    let http = lattice::http::StaticHttpClient::new(std::collections::HashMap::new());
    let process_config = lattice::process::core::RuntimeConfig::default();
    let runtime = Adapters::new(&shell, &http, &process_config);

    assert_eq!(runtime.file(&allowed_attempt(file_binding), &request), Err(dispatch::Error::AdapterRejected));
}

#[test]
fn allowed_file_process_and_network_use_selected_bounded_adapters() {
    let file_request = FileAdapterRequest {
        root_id: ROOT_ID.to_owned(),
        relative_path: TEST_FILE_NAME.to_owned(),
        content: Some(TEST_FILE_CONTENT.to_vec()),
        overwrite: false,
        read_max_bytes: FILE_READ_LIMIT_BYTES,
    };
    let process_request = lattice::agent_node::ProcessRequest {
        program: "sh".to_owned(),
        args: vec!["-c".to_owned(), "printf bounded".to_owned()],
        cwd: None,
        env: BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: PROCESS_TIMEOUT_SECS,
        output_limit_bytes: PROCESS_OUTPUT_LIMIT_BYTES,
    };
    let network_request = lattice::http::HttpRequestPlan {
        method: "GET".to_owned(),
        url: HTTP_URL.to_owned(),
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_secs: HTTP_TIMEOUT_SECS,
    };
    let file_binding = binding_with_payload(
        FILE_RESOURCE,
        FILE_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::file_plan_identity(&file_request),
    );
    let process_binding = binding_with_payload(
        PROCESS_RESOURCE,
        PROCESS_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::process_plan_identity(&process_request),
    );
    let network_binding = binding_with_payload(
        NETWORK_RESOURCE,
        NETWORK_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::network_plan_identity(&network_request),
    );
    let temp = tempfile::tempdir().expect("tempdir");
    let root = cap_std::fs::Dir::open_ambient_dir(temp.path(), cap_std::ambient_authority()).expect("test root");
    let mut roots = BTreeMap::new();
    roots.insert(ROOT_ID.to_owned(), root);
    let shell = dispatch::Shell::new(
        Box::new(UnavailableFacts),
        Box::new(UnavailableFacts),
        Box::new(UnavailableAttempts),
        Box::new(lattice::engine::effect::runtime::ConfiguredRoots::new(roots)),
    );
    let mut responses = std::collections::HashMap::new();
    responses.insert(
        HTTP_URL.to_owned(),
        lattice::http::StaticHttpResult::Response(lattice::http::HttpResponse {
            status: HTTP_STATUS_OK,
            headers: std::collections::HashMap::new(),
            body: "ok".to_owned(),
        }),
    );
    let http = lattice::http::StaticHttpClient::new(responses);
    let process_config = lattice::process::core::RuntimeConfig::default();
    let runtime = Adapters::new(&shell, &http, &process_config);

    let file_attempt = allowed_attempt(file_binding);
    let file_output = runtime.file(&file_attempt, &file_request).expect("file adapter");
    assert_eq!(file_output, FileAdapterOutput::Written {
        bytes: TEST_FILE_CONTENT.len()
    });
    assert_eq!(std::fs::read(temp.path().join(TEST_FILE_NAME)).expect("read file"), TEST_FILE_CONTENT);

    let network_output = runtime.network(&allowed_attempt(network_binding), &network_request).expect("network adapter");
    assert_eq!(network_output.status, HTTP_STATUS_OK);

    let process_output =
        runtime.process(&allowed_attempt(process_binding), &process_request, None).expect("process adapter");
    assert_eq!(process_output.stdout, b"bounded");
}

fn allowed_attempt(binding: authority::IntentBinding) -> dispatch::AuthorizedAttempt {
    dispatch::AuthorizedAttempt {
        attempt: authority::Attempt {
            identity: id(authority::AttemptIdentity::new, GRANT_REF),
            binding: binding.clone(),
            sequence: ATTEMPT_SEQUENCE,
        },
        authorization: authority::AuthorizationDecision {
            allowed: true,
            binding: Some(binding),
            denial: None,
            basalt_diagnostic: None,
        },
    }
}

struct UnavailableFacts;
impl dispatch::AuthorityFactsProvider for UnavailableFacts {
    fn load(&self, _binding: &authority::IntentBinding) -> Result<authority::AuthorizationFacts, dispatch::Error> {
        Err(dispatch::Error::AuthorityUnavailable)
    }
}
impl dispatch::ScopeSnapshotProvider for UnavailableFacts {
    fn load_current(
        &self,
        _binding: &authority::IntentBinding,
    ) -> Result<Option<authority::ScopeSnapshotFacts>, dispatch::Error> {
        Err(dispatch::Error::AuthorityUnavailable)
    }
}
struct UnavailableAttempts;
impl EffectAttemptStore for UnavailableAttempts {
    fn reserve(&self, _binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
        Err(dispatch::Error::AttemptPersistence)
    }
    fn record_outcome(&self, _event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
        Err(dispatch::Error::OutcomePersistence)
    }
}

#[test]
fn dispatch_shell_authorizes_runs_adapter_and_records_outcome() {
    let request = lattice::http::HttpRequestPlan {
        method: "GET".to_owned(),
        url: HTTP_URL.to_owned(),
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_secs: HTTP_TIMEOUT_SECS,
    };
    let binding = binding_with_payload(
        NETWORK_RESOURCE,
        NETWORK_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &lattice::engine::effect::runtime::network_plan_identity(&request),
    );
    let mut authorization_facts = facts(authority::CollaborationScopeBinding::Unscoped);
    authorization_facts.binding = binding.clone();
    authorization_facts.basalt_input = basalt_input(NETWORK_RESOURCE, NETWORK_ABILITY, "lattice-network");
    authorization_facts.trusted_policy_ref = dispatch::trusted_policy_ref("lattice-network");
    let recorded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shell = dispatch::Shell::new(
        Box::new(FixedFacts {
            facts: authorization_facts,
        }),
        Box::new(NoScope),
        Box::new(RecordingAttempts {
            recorded: std::sync::Arc::clone(&recorded),
        }),
        Box::new(NoRoots),
    );
    let http = lattice::http::StaticHttpClient::new(std::collections::HashMap::from([(
        HTTP_URL.to_owned(),
        lattice::http::StaticHttpResult::Response(lattice::http::HttpResponse {
            status: HTTP_STATUS_OK,
            headers: std::collections::HashMap::new(),
            body: "ok".to_owned(),
        }),
    )]));
    let process_config = lattice::process::core::RuntimeConfig::default();
    let runtime = Adapters::new(&shell, &http, &process_config);

    let response = shell.dispatch(&runtime, binding, &request).expect("authorized dispatch");
    assert_eq!(response.status, HTTP_STATUS_OK);
    let events = recorded.lock().expect("record lock");
    assert_eq!(events.len(), ATTEMPT_SEQUENCE as usize);
    assert_eq!(events[0].state, authority::OutcomeState::ObservedSuccess);
}

#[test]
fn wrong_payload_and_adapter_plan_are_denied() {
    let request = lattice::http::HttpRequestPlan {
        method: "GET".to_owned(),
        url: HTTP_URL.to_owned(),
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_secs: HTTP_TIMEOUT_SECS,
    };
    let wrong_binding = binding(NETWORK_RESOURCE, NETWORK_ABILITY, authority::CollaborationScopeBinding::Unscoped);
    let shell = dispatch::Shell::new(
        Box::new(UnavailableFacts),
        Box::new(UnavailableFacts),
        Box::new(UnavailableAttempts),
        Box::new(NoRoots),
    );
    let http = lattice::http::StaticHttpClient::new(std::collections::HashMap::new());
    let process_config = lattice::process::core::RuntimeConfig::default();
    let runtime = Adapters::new(&shell, &http, &process_config);

    assert_eq!(runtime.network(&allowed_attempt(wrong_binding), &request), Err(dispatch::Error::AdapterRejected));
}

struct DurableAttempts {
    store: std::sync::Arc<lattice::store::Store>,
    latest: std::sync::Arc<std::sync::Mutex<Option<authority::Attempt>>>,
}

impl EffectAttemptStore for DurableAttempts {
    fn reserve(&self, binding: authority::IntentBinding) -> Result<authority::Attempt, dispatch::Error> {
        let attempt = self.store.reserve_effect_attempt(binding).map_err(|error| match error {
            lattice::store::StoreError::Conflict(_) => dispatch::Error::DuplicateAttempt,
            _ => dispatch::Error::AttemptPersistence,
        })?;
        *self.latest.lock().expect("latest attempt lock") = Some(attempt.clone());
        Ok(attempt)
    }

    fn record_outcome(&self, event: &authority::AuthorizationOutcomeEvent) -> Result<(), dispatch::Error> {
        self.store.save_effect_outcome(event).map_err(|_| dispatch::Error::OutcomePersistence)
    }
}

fn end_to_end_authorization_facts(binding: authority::IntentBinding) -> authority::AuthorizationFacts {
    let resource = binding.resource.as_str().to_owned();
    let ability = binding.ability.as_str().to_owned();
    let request = binding.request.as_str().to_owned();
    let mut basalt_input = basalt_input(&resource, &ability, "lattice-network");
    basalt_input.request.request_ref.clone_from(&request);
    for receipt in &mut basalt_input.verification_receipts {
        receipt.request_ref.clone_from(&request);
    }
    authority::AuthorizationFacts {
        basalt_input,
        trusted_policy_ref: dispatch::trusted_policy_ref("lattice-network"),
        scope_requirement: authority::ScopeRequirement::UnscopedPermitted,
        scope_snapshot: None,
        revocation: authority::RevocationDisposition::Current,
        binding,
    }
}

// r[verify lattice.choregraph_effect_authority.binding]
// r[verify lattice.choregraph_effect_authority.adapters]
// r[verify lattice.choregraph_effect_authority.durability]
// r[verify lattice.choregraph_effect_authority.fixtures]
#[tokio::test]
async fn v2_handoff_authorizes_executor_network_effect_and_persists_outcome() {
    let mut workflow = lattice::engine::workflow::Workflow::new(
        END_TO_END_WORKFLOW_NAME,
        lattice::engine::workflow::WorkflowTimestamp::epoch(),
    );
    let node_id = workflow.add_http_node(END_TO_END_NODE_LABEL, "GET", HTTP_URL);
    let request = match workflow.node_config(node_id).expect("HTTP node config") {
        lattice::engine::workflow::NodeConfig::Http {
            method,
            url,
            headers,
            body,
            timeout_secs,
        } => lattice::http::HttpRequestPlan {
            method: method.clone(),
            url: url.clone(),
            headers: headers.clone(),
            body: body.clone(),
            timeout_secs: *timeout_secs,
        },
        _ => panic!("expected HTTP node config"),
    };
    let payload_ref = lattice::engine::effect::runtime::network_plan_identity(&request);
    let request_ref = lattice::engine::effect::runtime::dispatch_request_identity(&payload_ref);
    let lowered = choregraph_format::lower_portable(&workflow_to_portable(&workflow)).expect("workflow lowers");
    let choregraph_node = format!("{END_TO_END_CHOREGRAPH_NODE_PREFIX}{}", node_id.0);
    let context = lattice::engine::effect::choregraph::TransitionContext {
        machine: id(authority::MachineIdentity::new, END_TO_END_MACHINE_REF),
        workflow_revision: id(authority::WorkflowRevisionIdentity::new, END_TO_END_REVISION_REF),
        run: id(authority::RunIdentity::new, END_TO_END_RUN_REF),
        authority: id(authority::AuthorityIdentity::new, END_TO_END_AUTHORITY_REF),
        scope: authority::CollaborationScopeBinding::Unscoped,
    };
    let ready = choregraph_core::ConcreteEvent::new(choregraph_node.clone(), choregraph_core::Action::MarkReady);
    let lattice::engine::effect::choregraph::TransitionResult::Accepted {
        next: ready_state,
        bindings: ready_bindings,
    } = lattice::engine::effect::choregraph::transition(
        &lowered.machine,
        &lowered.machine.initial_state(),
        &ready,
        &context,
    )
    else {
        panic!("mark-ready transition must pass");
    };
    assert!(ready_bindings.is_empty());
    let start =
        lattice::engine::effect::choregraph::requested_event(lattice::engine::effect::choregraph::EventRequest {
            declaration_key: format!("{END_TO_END_DECLARATION_PREFIX}{}", node_id.0),
            node_id: choregraph_node,
            source_node_id: node_id.0.to_string(),
            action: choregraph_core::Action::Start,
            request_ref,
            payload_ref,
        });
    let lattice::engine::effect::choregraph::TransitionResult::Accepted { bindings, .. } =
        lattice::engine::effect::choregraph::transition(&lowered.machine, &ready_state, &start, &context)
    else {
        panic!("start transition must return a concrete binding");
    };
    let binding = bindings.into_iter().next().expect("one concrete binding");
    assert_eq!(binding.node.as_str(), format!("lattice-node/{}", node_id.0));
    assert_eq!(binding.transition.as_str(), "ready->start");
    assert_eq!(binding.operation.as_str(), "lattice/http");
    assert_eq!(binding.resource.as_str(), format!("lattice://network/{}", node_id.0));
    assert_eq!(binding.ability.as_str(), NETWORK_ABILITY);

    let temp = tempfile::tempdir().expect("tempdir");
    let store = std::sync::Arc::new(
        lattice::store::Store::new(temp.path().join("authorized-effect.redb")).expect("effect store"),
    );
    let latest = std::sync::Arc::new(std::sync::Mutex::new(None));
    let dispatch = dispatch::Shell::new(
        Box::new(FixedFacts {
            facts: end_to_end_authorization_facts(binding.clone()),
        }),
        Box::new(NoScope),
        Box::new(DurableAttempts {
            store: std::sync::Arc::clone(&store),
            latest: std::sync::Arc::clone(&latest),
        }),
        Box::new(NoRoots),
    );
    let response = lattice::http::HttpResponse {
        status: HTTP_STATUS_OK,
        headers: std::collections::HashMap::new(),
        body: END_TO_END_SUCCESS_BODY.to_owned(),
    };
    let client = lattice::http::StaticHttpClient::new(std::collections::HashMap::from([(
        HTTP_URL.to_owned(),
        lattice::http::StaticHttpResult::Response(response),
    )]));
    let executor = Executor::new_with_http_client(Box::new(client));

    let result = executor
        .execute_authorized_node(&workflow, node_id, &std::collections::HashMap::new(), &binding, &dispatch)
        .await;

    let NodeStatus::Completed { outputs, .. } = result.status else {
        panic!("authorized executor effect must complete: {:?}", result.status);
    };
    let node = workflow.graph.node(node_id).expect("HTTP node");
    assert_eq!(outputs[&node.output_ports[HTTP_BODY_OUTPUT_INDEX].id].as_display_string(), END_TO_END_SUCCESS_BODY);
    let attempt = latest.lock().expect("latest attempt lock").clone().expect("attempt was reserved");
    assert_eq!(store.load_effect_attempt(&attempt.identity).expect("attempt load"), Some(attempt.clone()));
    let expected_outcome = authority::observed_outcome_event(&attempt, authority::OutcomeState::ObservedSuccess, None);
    assert_eq!(store.load_effect_outcome(&expected_outcome.identity).expect("outcome load"), Some(expected_outcome));
}

#[tokio::test]
async fn authorized_executor_rejects_binding_for_another_node_before_http_dispatch() {
    #[derive(Clone)]
    struct CountingHttpClient {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }
    impl lattice::http::HttpClient for CountingHttpClient {
        fn execute(
            &self,
            _request: &lattice::http::HttpRequestPlan,
        ) -> Result<lattice::http::HttpResponse, lattice::http::HttpError> {
            self.calls.fetch_add(ATTEMPT_SEQUENCE as usize, std::sync::atomic::Ordering::SeqCst);
            Err(lattice::http::HttpError::Transport("unexpected dispatch".to_owned()))
        }
    }

    let mut workflow = lattice::engine::workflow::Workflow::new(
        END_TO_END_WORKFLOW_NAME,
        lattice::engine::workflow::WorkflowTimestamp::epoch(),
    );
    let node_id = workflow.add_http_node(END_TO_END_NODE_LABEL, "GET", HTTP_URL);
    let request = lattice::http::HttpRequestPlan {
        method: "GET".to_owned(),
        url: HTTP_URL.to_owned(),
        headers: std::collections::HashMap::new(),
        body: None,
        timeout_secs: HTTP_TIMEOUT_SECS,
    };
    let payload = lattice::engine::effect::runtime::network_plan_identity(&request);
    let mut wrong_binding = binding_with_payload(
        NETWORK_RESOURCE,
        NETWORK_ABILITY,
        authority::CollaborationScopeBinding::Unscoped,
        &payload,
    );
    wrong_binding.node = id(authority::NodeIdentity::new, "lattice-node/other");
    wrong_binding.transition = id(authority::TransitionIdentity::new, "ready->start");
    wrong_binding.operation = id(authority::OperationIdentity::new, "lattice/http");
    wrong_binding.request = id(
        authority::RequestIdentity::new,
        &lattice::engine::effect::runtime::dispatch_request_identity(&payload),
    );
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let client = CountingHttpClient {
        calls: std::sync::Arc::clone(&calls),
    };
    let shell = dispatch::Shell::new(
        Box::new(UnavailableFacts),
        Box::new(NoScope),
        Box::new(UnavailableAttempts),
        Box::new(NoRoots),
    );
    let executor = Executor::new_with_http_client(Box::new(client));

    let result = executor
        .execute_authorized_node(&workflow, node_id, &std::collections::HashMap::new(), &wrong_binding, &shell)
        .await;

    assert!(matches!(result.status, NodeStatus::Failed { ref error } if error.contains("binding does not match")));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}

// r[verify lattice.choregraph_effect_authority.binding]
// r[verify lattice.choregraph_effect_authority.denial]
// r[verify lattice.choregraph_effect_authority.fixtures]
#[tokio::test]
async fn product_run_uses_v2_binding_and_fails_closed_without_current_authority() {
    #[derive(Clone)]
    struct CountingHttpClient {
        calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    }

    impl lattice::http::HttpClient for CountingHttpClient {
        fn execute(
            &self,
            _request: &lattice::http::HttpRequestPlan,
        ) -> Result<lattice::http::HttpResponse, lattice::http::HttpError> {
            self.calls.fetch_add(ATTEMPT_SEQUENCE as usize, std::sync::atomic::Ordering::SeqCst);
            Ok(lattice::http::HttpResponse {
                status: HTTP_STATUS_OK,
                headers: std::collections::HashMap::new(),
                body: END_TO_END_SUCCESS_BODY.to_owned(),
            })
        }
    }

    let mut workflow = lattice::engine::workflow::Workflow::new(
        END_TO_END_WORKFLOW_NAME,
        lattice::engine::workflow::WorkflowTimestamp::epoch(),
    );
    workflow.add_http_node(END_TO_END_NODE_LABEL, "GET", HTTP_URL);
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut executor = Executor::new_with_http_client(Box::new(CountingHttpClient {
        calls: std::sync::Arc::clone(&calls),
    }));
    let unavailable = lattice::engine::effect::execution::RunContext::unavailable_for_store(
        lattice::engine::RunId(ATTEMPT_SEQUENCE),
        None,
    )
    .expect("unavailable context");

    let (_, denied) = executor.run_authorized(&workflow, &std::collections::HashMap::new(), &unavailable).await;

    assert!(matches!(denied[0].status, NodeStatus::Failed { ref error } if error.contains("effect dispatch failed")));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

    let recorded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let shell = dispatch::Shell::new(
        Box::new(DynamicNetworkFacts),
        Box::new(NoScope),
        Box::new(RecordingAttempts {
            recorded: std::sync::Arc::clone(&recorded),
        }),
        Box::new(NoRoots),
    );
    let context = lattice::engine::effect::execution::RunContext::new(
        id(authority::RunIdentity::new, END_TO_END_RUN_REF),
        id(authority::AuthorityIdentity::new, END_TO_END_AUTHORITY_REF),
        authority::CollaborationScopeBinding::Unscoped,
        std::sync::Arc::new(shell),
    );
    let mut executor = Executor::new_with_http_client(Box::new(CountingHttpClient {
        calls: std::sync::Arc::clone(&calls),
    }));

    let (_, allowed) = executor.run_authorized(&workflow, &std::collections::HashMap::new(), &context).await;

    assert!(matches!(allowed[0].status, NodeStatus::Completed { .. }));
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), ATTEMPT_SEQUENCE as usize);
    assert!(
        recorded
            .lock()
            .expect("recorded outcomes")
            .iter()
            .any(|event| event.state == authority::OutcomeState::ObservedSuccess)
    );
}

#[test]
fn stale_policy_binding_is_denied() {
    let requested = binding(FILE_RESOURCE, FILE_ABILITY, authority::CollaborationScopeBinding::Unscoped);
    let mut stale = facts(authority::CollaborationScopeBinding::Unscoped);
    stale.binding.policy = id(authority::PolicyIdentity::new, "blake3:stale-policy");

    let decision = authority::evaluate_for(&requested, &stale);

    assert_eq!(decision.denial, Some(authority::AuthorizationDenial::BindingMismatch));
}

#[test]
fn policy_is_separate_and_has_exact_blake3_identity() {
    let digest = blake3::hash(authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_JSON.as_bytes());
    assert_eq!(digest.to_hex().as_str(), authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_BLAKE3);
    assert!(!authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_JSON.contains("machine"));
    assert!(!authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_JSON.contains("workflow"));
}
