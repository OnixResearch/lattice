//! Actual redb durability and public recovery admission. These fixtures use the
//! native producer contract and real Ed25519 observation verification, not a
//! network service or a claim that any remote build occurred.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use basalt::ucan::Ed25519SigningKey;
use basalt::ucan::TokenSigner;

use super::*;

const NOW_MS: u64 = 1_000_000;
const SIGNING_SEED: [u8; 32] = [7; 32];

fn id(byte: u8) -> Identity {
    Identity([byte; 32])
}

fn policy() -> Policy {
    Policy {
        bounds: core::Bounds {
            matrix_axes: 8,
            matrix_values_per_axis: 8,
            matrix_jobs: 16,
            public_label_bytes: 128,
            event_history: 32,
            evidence: 8,
            annotations: 2,
            annotation_bytes: 128,
            title_bytes: 64,
            summary_bytes: 256,
            text_bytes: 512,
            url_bytes: 512,
            reconciliations: 4,
        },
        retry: core::RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 800,
            max_elapsed_ms: 2_000,
        },
        capability_lifetime_ms: 60_000,
        public_urls: core::PublicUrlPolicy {
            allowed: BTreeSet::new(),
        },
    }
}

fn derivation() -> String {
    serde_json::json!({
        "name":"request-payload-only-sentinel", "builder":"/bin/sh", "system":"x86_64-linux",
        "args":[], "outputs":["out"], "dynamic_plan_outputs":[], "env":{}, "inputs":[],
        "fixed_output":null, "addressing_mode":"input-addressed", "provenance":null,
    })
    .to_string()
}

fn fixture() -> (mantle::Endpoint, mantle::PlannedBuild, Policy, Record) {
    let endpoint = mantle::Endpoint {
        socket: "/run/mantle/private-build.sock".into(),
        peer_uid: 1000,
        service: "did:example:mantle-private".into(),
        project: "session_recovery".into(),
        owner: "did:example:lattice-holder".into(),
        account: "account-a".into(),
        endpoint: "builder-a".into(),
        generation: 3,
        endpoint_generation: 7,
        authority_policy: basalt::authority_core::BasaltPolicyRef::new(
            format!("blake3:{}", "a".repeat(64)),
            basalt::SUPPORTED_SCHEMA_VERSION,
            "mantle-build-submit",
        ),
        signing_key: basalt::ucan::derive_ed25519_public_key(&SIGNING_SEED).unwrap(),
        producer: id(8),
    };
    let derivation = derivation();
    let policy = policy();
    let job = core::plan_job(
        &core::WorkflowJobInput {
            workflow: id(1),
            node: id(2),
            generation: 0,
            coordinates: BTreeMap::from([("system".into(), "linux".into())]),
            subject: id(9), // Public verified-subject identity, not the holder DID.
            audience: endpoint.service.clone(),
            project: mantle::public_identity(endpoint.project.as_bytes()),
            request: core::ConcreteRequest {
                source: id(3),
                derivation: mantle::public_identity(derivation.as_bytes()),
                input_artifacts: BTreeSet::new(),
            },
        },
        &policy,
        NOW_MS,
    )
    .unwrap();
    let plan = mantle::PlannedBuild::new(
        &endpoint,
        job,
        "/nix/store/00000000000000000000000000000000-request-payload-only-sentinel.drv".into(),
        derivation,
    )
    .unwrap();
    let check = github::CheckRunBinding::new(github::CheckRunTarget {
        owner: "test-owner".into(),
        repository: "test-repository".into(),
        head_sha: "a".repeat(40),
        name: "Observed build".into(),
        external_id: plan.job.job,
    })
    .unwrap();
    let record = Record {
        revision: 0,
        workflow: plan.job.workflow,
        job: plan.job.job,
        subject: plan.job.subject,
        generation: plan.job.generation,
        idempotency: plan.job.idempotency,
        native_request_hash: plan.native_request_hash,
        native_submission: mantle::public_identity(plan.resource().as_bytes()),
        service: mantle::public_identity(endpoint.service.as_bytes()),
        project: mantle::public_identity(endpoint.project.as_bytes()),
        owner: mantle::public_identity(endpoint.owner.as_bytes()),
        producer: endpoint.producer,
        attempt: None,
        state: None,
        event_cursor: None,
        check,
        exchanges: BTreeSet::new(),
    };
    (endpoint, plan, policy, record)
}

fn admitted_record(
    endpoint: &mantle::Endpoint,
    plan: &mantle::PlannedBuild,
    policy: &Policy,
    pending: &Record,
) -> Record {
    let signer = Ed25519SigningKey::from_bytes(SIGNING_SEED);
    let status = wire::Status {
        attempt_id: "session_attempt_1".into(),
        request_digest: plan.native_request_hash.0,
        observation: wire::Observation {
            phase: wire::Phase::Running,
            fence: 1,
            result_evidence: None,
        },
        sequence: 1,
        cancel_requested: false,
    };
    let mut response = wire::SignedResponse {
        schema: wire::SIGNED_RESPONSE_SCHEMA.into(),
        service: endpoint.service.clone(),
        project: endpoint.project.clone(),
        owner: endpoint.owner.clone(),
        request_digest: wire::request_digest(plan.request(), &endpoint.project, endpoint.endpoint_generation).unwrap(),
        created_seconds: NOW_MS / 1000,
        producer: endpoint.signing_key,
        response: wire::Response::Submitted {
            attempt_id: status.attempt_id.clone(),
            recovered: false,
        },
        status: Some(status),
        signature: Vec::new(),
    };
    let digest = wire::response_digest(&response).unwrap();
    response.signature = signer.sign(&digest).unwrap().as_bytes().to_vec();
    let client = mantle::MantleClient::new(endpoint.clone()).unwrap();
    let response = client.verify_response(plan.request(), response, NOW_MS).unwrap();
    let (attempt, verified_facts) = client.bind_submission(plan, &response, NOW_MS).unwrap();
    let initial = core::DurableState::new(&plan.job, attempt.binding).unwrap();
    let state = core::reconcile(&initial, &core::VerifiedStatus::after_verification(verified_facts), policy)
        .unwrap()
        .state;
    let mut admitted = pending.clone();
    admitted.revision += 1;
    admitted.attempt = Some(attempt);
    admitted.state = Some(state);
    admitted
}

#[test]
fn pending_submit_and_verified_attempt_survive_database_close_and_reopen() {
    let (endpoint, plan, policy, pending) = fixture();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("recovery.redb");
    {
        let db = redb::Database::create(&path).unwrap();
        let store = RecoveryStore::new(&db, 2).unwrap();
        pending.validate_for(&plan, &endpoint, &policy).unwrap();
        store.commit(None, &pending).unwrap();
    }
    let admitted;
    {
        let db = redb::Database::open(&path).unwrap();
        let store = RecoveryStore::new(&db, 2).unwrap();
        let recovered = store.load(plan.job.job).unwrap().unwrap();
        assert_eq!(recovered, pending);
        assert!(recovered.attempt.is_none());
        assert_eq!(recovered.idempotency, plan.job.idempotency);
        admitted = admitted_record(&endpoint, &plan, &policy, &recovered);
        admitted.validate_for(&plan, &endpoint, &policy).unwrap();
        store.commit(Some(&recovered), &admitted).unwrap();
    }
    let db = redb::Database::open(&path).unwrap();
    let store = RecoveryStore::new(&db, 2).unwrap();
    let recovered = store.load(plan.job.job).unwrap().unwrap();
    assert_eq!(recovered, admitted);
    recovered.validate_for(&plan, &endpoint, &policy).unwrap();
    let state = recovered.state.as_ref().unwrap();
    assert_eq!(state.cursor, 1);
    assert_eq!(state.phase, core::Phase::Running);
    assert_eq!(core::restart(state, &policy).unwrap(), core::RestartDecision::ReacquireAndReconcile(state.binding));
}

#[test]
fn compare_and_swap_rejects_stale_writers_missing_previous_and_revision_skips() {
    let (endpoint, plan, policy, pending) = fixture();
    let directory = tempfile::tempdir().unwrap();
    let db = redb::Database::create(directory.path().join("cas.redb")).unwrap();
    let store = RecoveryStore::new(&db, 2).unwrap();
    store.commit(None, &pending).unwrap();
    let reader_a = store.load(pending.job).unwrap().unwrap();
    let reader_b = store.load(pending.job).unwrap().unwrap();
    let winner = admitted_record(&endpoint, &plan, &policy, &reader_a);
    store.commit(Some(&reader_a), &winner).unwrap();
    let mut stale = reader_b.clone();
    stale.revision += 1;
    assert!(matches!(store.commit(Some(&reader_b), &stale), Err(Error::Conflict)));
    assert!(matches!(store.commit(None, &pending), Err(Error::Conflict)));
    let mut skipped = winner.clone();
    skipped.revision += 2;
    assert!(matches!(store.commit(Some(&winner), &skipped), Err(Error::Conflict)));
    let mut overflow = winner.clone();
    overflow.revision = u64::MAX;
    assert!(matches!(store.commit(Some(&overflow), &overflow), Err(Error::Conflict)));
    assert_eq!(store.load(pending.job).unwrap(), Some(winner));
}

#[test]
fn record_quota_blocks_new_jobs_but_not_existing_job_advancement() {
    let (endpoint, plan, policy, pending) = fixture();
    let directory = tempfile::tempdir().unwrap();
    let db = redb::Database::create(directory.path().join("quota.redb")).unwrap();
    assert!(matches!(RecoveryStore::new(&db, 0), Err(Error::Policy)));
    let store = RecoveryStore::new(&db, 1).unwrap();
    store.commit(None, &pending).unwrap();
    let admitted = admitted_record(&endpoint, &plan, &policy, &pending);
    store.commit(Some(&pending), &admitted).unwrap();
    let mut foreign = pending.clone();
    foreign.job = id(99);
    foreign.check.target.external_id = foreign.job;
    assert!(matches!(store.commit(None, &foreign), Err(Error::Policy)));
    assert_eq!(store.load(pending.job).unwrap(), Some(admitted));
    assert_eq!(store.load(foreign.job).unwrap(), None);
}

#[test]
fn oversized_record_is_rejected_without_overwriting_last_durable_observation() {
    let (_, _, _, pending) = fixture();
    let directory = tempfile::tempdir().unwrap();
    let db = redb::Database::create(directory.path().join("size.redb")).unwrap();
    let store = RecoveryStore::new(&db, 1).unwrap();
    store.commit(None, &pending).unwrap();
    let mut oversized = pending.clone();
    oversized.revision += 1;
    oversized.event_cursor = Some("x".repeat(RECORD_BYTES_MAX + 1));
    assert!(matches!(store.commit(Some(&pending), &oversized), Err(Error::Policy)));
    assert_eq!(store.load(pending.job).unwrap(), Some(pending));
    assert!(matches!(decode(&vec![b' '; RECORD_BYTES_MAX + 1]), Err(Error::Policy)));
}

#[test]
fn recovery_rejects_foreign_job_native_request_scope_and_producer() {
    let (endpoint, plan, policy, pending) = fixture();
    let admitted = admitted_record(&endpoint, &plan, &policy, &pending);
    for field in 0..14 {
        let mut bad = admitted.clone();
        match field {
            0 => bad.workflow = id(99),
            1 => bad.job = id(99),
            2 => bad.generation += 1,
            3 => bad.idempotency = id(99),
            4 => bad.native_request_hash = id(99),
            5 => bad.service = id(99),
            6 => bad.project = id(99),
            7 => bad.owner = id(99),
            8 => bad.producer = id(99),
            9 => bad.attempt.as_mut().unwrap().native_request_hash = id(99),
            10 => bad.check.target.external_id = id(99),
            11 => bad.subject = id(99),
            12 => bad.native_submission = id(99),
            _ => bad.attempt.as_mut().unwrap().binding.producer = id(99),
        }
        assert!(bad.validate_for(&plan, &endpoint, &policy).is_err(), "mutation {field}");
    }
    let alternate =
        mantle::PlannedBuild::new(&endpoint, plan.job.clone(), "/nix/store/other.drv".into(), derivation()).unwrap();
    assert!(matches!(admitted.validate_for(&alternate, &endpoint, &policy), Err(Error::Conflict)));
}

#[test]
fn partial_attempt_check_id_and_cursor_records_fail_closed() {
    let (endpoint, plan, policy, pending) = fixture();
    let admitted = admitted_record(&endpoint, &plan, &policy, &pending);
    for field in 0..7 {
        let mut bad = admitted.clone();
        match field {
            0 => bad.attempt = None,
            1 => bad.state = None,
            2 => bad.check.check_id = Some(123),
            3 => bad.state.as_mut().unwrap().binding.fence += 1,
            4 => bad.attempt.as_mut().unwrap().fence += 1,
            5 => bad.attempt.as_mut().unwrap().attempt = "other_attempt".into(),
            _ => bad.state.as_mut().unwrap().schema_version += 1,
        }
        assert!(bad.validate_for(&plan, &endpoint, &policy).is_err(), "mutation {field}");
    }
    let mut premature_cursor = pending.clone();
    premature_cursor.event_cursor = Some("not-an-admitted-cursor".into());
    assert!(matches!(premature_cursor.validate_for(&plan, &endpoint, &policy), Err(Error::Conflict)));
}

#[test]
fn public_record_omits_request_payload_and_rejects_credential_extension_fields() {
    let (endpoint, plan, policy, pending) = fixture();
    let record = admitted_record(&endpoint, &plan, &policy, &pending);
    let public = serde_json::to_string(&record).unwrap();
    assert!(!public.contains("request-payload-only-sentinel"));
    assert!(!public.contains("derivation_json"));
    assert!(!public.contains("drv_path"));
    for key in [
        "oidc_token",
        "ucan",
        "github_token",
        "provider_value",
        "value_derived_hash",
    ] {
        let mut invalid = serde_json::to_value(&record).unwrap();
        invalid[key] = serde_json::json!("credential-extension-must-not-survive");
        assert!(matches!(decode(&serde_json::to_vec(&invalid).unwrap()), Err(Error::Conflict)), "{key}");
    }
    let mut nested = serde_json::to_value(&record).unwrap();
    nested["attempt"]["bearer"] = serde_json::json!("credential-extension-must-not-survive");
    assert!(matches!(decode(&serde_json::to_vec(&nested).unwrap()), Err(Error::Conflict)));
    let restored = decode(public.as_bytes()).unwrap();
    assert_eq!(restored, record);
    restored.validate_for(&plan, &endpoint, &policy).unwrap();
}

#[test]
fn consistently_malformed_attempt_and_deserialized_check_target_are_not_public_authority() {
    let (endpoint, plan, policy, pending) = fixture();
    let admitted = admitted_record(&endpoint, &plan, &policy, &pending);
    for field in 0..4 {
        let mut bad = admitted.clone();
        match field {
            0 => {
                let attempt = bad.attempt.as_mut().unwrap();
                attempt.attempt = "../foreign".into();
                attempt.binding.attempt = mantle::public_identity(attempt.attempt.as_bytes());
                bad.state.as_mut().unwrap().binding.attempt = attempt.binding.attempt;
                for event in bad.state.as_mut().unwrap().history.values_mut() {
                    event.binding.attempt = attempt.binding.attempt;
                }
            }
            1 => {
                let attempt = bad.attempt.as_mut().unwrap();
                attempt.fence = 0;
                attempt.binding.fence = 0;
                let state = bad.state.as_mut().unwrap();
                state.binding.fence = 0;
                for event in state.history.values_mut() {
                    event.binding.fence = 0;
                }
            }
            2 => bad.check.target.head_sha = "not-a-commit".into(),
            _ => bad.check.target.repository = "../foreign".into(),
        }
        assert!(bad.validate_for(&plan, &endpoint, &policy).is_err(), "mutation {field}");
    }
}

#[test]
fn unknown_persisted_extension_refuses_load_and_cannot_be_overwritten_as_new() {
    let (_, _, _, pending) = fixture();
    let directory = tempfile::tempdir().unwrap();
    let db = redb::Database::create(directory.path().join("corrupt.redb")).unwrap();
    let store = RecoveryStore::new(&db, 1).unwrap();
    store.commit(None, &pending).unwrap();
    let mut invalid = serde_json::to_value(&pending).unwrap();
    invalid["provider_value"] = serde_json::json!("unknown-extension-must-not-be-recovered");
    let bytes = serde_json::to_vec(&invalid).unwrap();
    let tx = db.begin_write().unwrap();
    {
        let mut table = tx.open_table(RECORDS).unwrap();
        table.insert(pending.job.hex().as_str(), bytes.as_slice()).unwrap();
    }
    tx.commit().unwrap();
    assert!(matches!(store.load(pending.job), Err(Error::Conflict)));
    assert!(matches!(store.commit(None, &pending), Err(Error::Conflict)));
    let read = db.begin_read().unwrap();
    let table = read.open_table(RECORDS).unwrap();
    assert_eq!(table.get(pending.job.hex().as_str()).unwrap().unwrap().value(), bytes.as_slice());
}

#[test]
fn session_rejects_forged_success_and_tighter_policy_violation_before_check_reservation() {
    let (endpoint, plan, policy, pending) = fixture();
    let admitted = admitted_record(&endpoint, &plan, &policy, &pending);
    let directory = tempfile::tempdir().unwrap();
    let db = redb::Database::create(directory.path().join("projection.redb")).unwrap();
    let store = RecoveryStore::new(&db, 1).unwrap();
    store.commit(None, &pending).unwrap();
    store.commit(Some(&pending), &admitted).unwrap();
    let required_producers = BTreeSet::from([endpoint.producer]);
    let evidence = core::RolloutEvidence {
        candidate: plan.job.workflow,
        producer_prerequisites: required_producers.clone(),
        offline: true,
        private: false,
        report_only: false,
        recovery: false,
        duplicate_events: false,
        cancellation: false,
        credential_redaction: false,
        cairn_gate: None,
    };
    let session = Session {
        store,
        client: mantle::MantleClient::new(endpoint).unwrap(),
        policy: &policy,
        rollout: Rollout {
            stage: core::Stage::Private,
            candidate: plan.job.workflow,
            required_producers: &required_producers,
            evidence: &evidence,
        },
    };
    // Actual native HTTP and registry objects: no fake responses or provider.
    // No credential is declared, so even a regression reaching the transport
    // would fail resolution instead of contacting a network endpoint.
    let http = crate::http::NativeHttpClient;
    let credentials = crate::credentials::CredentialProviderRegistry::new();
    let github =
        github::GitHubChecksClient::new(&http, &credentials, &BTreeSet::from([github::API_HOST.to_owned()])).unwrap();
    let context = crate::credentials::CredentialResolutionContext {
        subject: crate::credentials::CredentialSubject::Node {
            workflow_id: plan.job.workflow.hex(),
            node_id: "publish".into(),
        },
        binding: "Authorization".into(),
        reference: crate::credentials::SecretRef::new("undeclared", "github", None),
        destination: crate::credentials::CredentialDestination::HttpHeader,
        reason: "projection admission boundary only".into(),
    };
    let projection =
        core::project_check(admitted.state.as_ref().unwrap(), "Observed build", "Queued", "", &[], &[], &policy)
            .unwrap();
    let mut forged = projection.clone();
    forged.status = core::CheckStatus::Completed;
    forged.conclusion = Some(core::Conclusion::Success);
    assert!(matches!(session.update_check(&plan, &admitted, &forged, &github, &context), Err(Error::Conflict)));
    assert_eq!(session.store.load(admitted.job).unwrap(), Some(admitted.clone()));
    let mut oversized = projection;
    oversized.summary = "x".repeat(policy.bounds.summary_bytes + 1);
    assert!(oversized.summary.len() < github::MAX_SUMMARY_BYTES);
    assert!(matches!(
        session.update_check(&plan, &admitted, &oversized, &github, &context),
        Err(Error::Core(core::Error::Bounds))
    ));
    assert_eq!(session.store.load(admitted.job).unwrap(), Some(admitted));
}

#[test]
fn recorded_exchange_subject_cannot_substitute_same_holder_for_planned_verified_subject() {
    let (endpoint, plan, policy, pending) = fixture();
    let mut record = admitted_record(&endpoint, &plan, &policy, &pending);
    let holder = mantle::public_identity(endpoint.owner.as_bytes());
    assert_ne!(holder, plan.job.subject);
    // Public receipt metadata only: this test checks durable admission, not
    // issuance or signature authority for these synthetic receipt identities.
    let observation = ExchangeObservation {
        receipt: id(20),
        policy: id(21),
        subject: plan.job.subject,
        requested: id(22),
        granted: id(23),
    };
    record.exchanges.insert(observation.clone());
    record.validate_for(&plan, &endpoint, &policy).unwrap();
    let original_idempotency = record.idempotency;
    record.exchanges.clear();
    record.exchanges.insert(ExchangeObservation {
        subject: holder,
        ..observation
    });
    assert_eq!(record.owner, holder);
    assert_eq!(record.idempotency, original_idempotency);
    assert!(record.validate_for(&plan, &endpoint, &policy).is_err());
    record.exchanges.clear();
    record.subject = holder;
    assert!(record.validate_for(&plan, &endpoint, &policy).is_err());
}
