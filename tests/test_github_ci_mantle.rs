//! Offline conformance for the real native signed gateway contract. These
//! cases exercise cryptographic and durable binding, not a live build claim.
use std::collections::BTreeSet;

use basalt::ucan::Ed25519SigningKey;
use basalt::ucan::TokenSigner;
use lattice::github_ci::core::ConcreteRequest;
use lattice::github_ci::core::Identity;
use lattice::github_ci::core::{self};
use lattice::github_ci::mantle::Endpoint;
use lattice::github_ci::mantle::Error;
use lattice::github_ci::mantle::MantleClient;
use lattice::github_ci::mantle::NativeAttempt;
use lattice::github_ci::mantle::PlannedBuild;
use lattice::github_ci::mantle::VerifiedResponse;
use lattice::github_ci::mantle::public_identity;
use lattice::github_ci::mantle::{self};
use mantle_build_contract::async_api as wire;
const NOW_MS: u64 = 1_000_000;
const SEED: [u8; 32] = [7; 32];

fn endpoint() -> Endpoint {
    Endpoint {
        socket: "/run/mantle/private-build.sock".into(),
        peer_uid: 1000,
        service: "did:example:mantle-private".into(),
        project: "lattice_ci".into(),
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
        signing_key: basalt::ucan::derive_ed25519_public_key(&SEED).unwrap(),
        producer: Identity([8; 32]),
    }
}
fn derivation() -> String {
    serde_json::json!({"name":"lattice-ci","builder":"/bin/sh","system":"x86_64-linux"}).to_string()
}
fn job(endpoint: &Endpoint, bytes: &str) -> core::JobPlan {
    let policy = core::Policy {
        bounds: core::Bounds {
            matrix_axes: 4,
            matrix_values_per_axis: 8,
            matrix_jobs: 64,
            public_label_bytes: 1024,
            event_history: 64,
            evidence: 16,
            annotations: 50,
            annotation_bytes: 4096,
            title_bytes: 255,
            summary_bytes: 32768,
            text_bytes: 32768,
            url_bytes: 8192,
            reconciliations: 4,
        },
        retry: core::RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 60_000,
            max_elapsed_ms: 120_000,
        },
        capability_lifetime_ms: 60_000,
        public_urls: core::PublicUrlPolicy {
            allowed: BTreeSet::new(),
        },
    };
    core::plan_job(
        &core::WorkflowJobInput {
            workflow: Identity([2; 32]),
            node: Identity([3; 32]),
            generation: 1,
            coordinates: core::Coordinates::new(),
            subject: Identity([5; 32]),
            audience: endpoint.service.clone(),
            project: public_identity(endpoint.project.as_bytes()),
            request: ConcreteRequest {
                source: Identity([1; 32]),
                derivation: public_identity(bytes.as_bytes()),
                input_artifacts: BTreeSet::new(),
            },
        },
        &policy,
        NOW_MS,
    )
    .unwrap()
}
fn plan(endpoint: &Endpoint) -> PlannedBuild {
    let bytes = derivation();
    PlannedBuild::new(
        endpoint,
        job(endpoint, &bytes),
        "/nix/store/00000000000000000000000000000000-lattice-ci.drv".into(),
        bytes,
    )
    .unwrap()
}
fn status(plan: &PlannedBuild) -> wire::Status {
    wire::Status {
        attempt_id: "attempt_7".into(),
        request_digest: plan.native_request_hash().0,
        observation: wire::Observation {
            phase: wire::Phase::Running,
            fence: 7,
            result_evidence: None,
        },
        cancel_requested: false,
        sequence: 1,
    }
}
fn sign(snapshot: &mut wire::SignedResponse) {
    let digest = wire::response_digest(snapshot).unwrap();
    snapshot.signature = Ed25519SigningKey::from_bytes(SEED).sign(&digest).unwrap().as_bytes().to_vec();
}
fn snapshot(
    endpoint: &Endpoint,
    request: &wire::Request,
    response: wire::Response,
    status: wire::Status,
) -> wire::SignedResponse {
    let mut snapshot = wire::SignedResponse {
        schema: wire::SIGNED_RESPONSE_SCHEMA.into(),
        service: endpoint.service.clone(),
        project: endpoint.project.clone(),
        owner: endpoint.owner.clone(),
        request_digest: wire::request_digest(request, &endpoint.project, endpoint.endpoint_generation).unwrap(),
        created_seconds: NOW_MS / 1000,
        producer: endpoint.signing_key,
        response,
        status: Some(status),
        signature: Vec::new(),
    };
    sign(&mut snapshot);
    snapshot
}
fn admitted() -> (Endpoint, PlannedBuild, MantleClient, NativeAttempt) {
    let endpoint = endpoint();
    let plan = plan(&endpoint);
    let client = MantleClient::new(endpoint.clone()).unwrap();
    let status = status(&plan);
    let submitted = snapshot(
        &endpoint,
        plan.request(),
        wire::Response::Submitted {
            attempt_id: status.attempt_id.clone(),
            recovered: false,
        },
        status,
    );
    let verified = client.verify_response(plan.request(), submitted, NOW_MS).unwrap();
    let (native, facts) = client.bind_submission(&plan, &verified, NOW_MS).unwrap();
    assert_eq!(facts.phase, core::Phase::Running);
    (endpoint, plan, client, native)
}
fn rejected<T>(result: Result<T, Error>) -> Error {
    match result {
        Err(error) => error,
        Ok(_) => panic!("accepted invalid observation"),
    }
}

#[test]
fn native_derivation_identity_and_semantic_source_identity_are_not_conflated() {
    let endpoint = endpoint();
    let bytes = derivation();
    let semantic = job(&endpoint, &bytes);
    let plan = plan(&endpoint);
    let pretty = serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&bytes).unwrap()).unwrap();
    assert_eq!(
        rejected(PlannedBuild::new(&endpoint, semantic, "/nix/store/boundary.drv".into(), pretty)),
        Error::Binding
    );
    let wire::Request::Submit { drv_path, .. } = plan.request() else {
        unreachable!()
    };
    assert_eq!(plan.native_request_hash(), public_identity(drv_path.as_bytes()));
    assert_ne!(plan.job().request_id, plan.native_request_hash());
    let mut changed = endpoint.clone();
    changed.endpoint_generation += 1;
    assert_ne!(
        mantle::canonical_resource(&endpoint, plan.request()).unwrap(),
        mantle::canonical_resource(&changed, plan.request()).unwrap()
    );
    changed = endpoint.clone();
    changed.generation += 1;
    assert_ne!(
        mantle::canonical_resource(&endpoint, plan.request()).unwrap(),
        mantle::canonical_resource(&changed, plan.request()).unwrap()
    );
}

#[test]
fn signed_snapshot_rejects_scope_request_time_and_signature_substitution() {
    let (endpoint, plan, client, _) = admitted();
    let status = status(&plan);
    let request = wire::Request::Status {
        attempt_id: status.attempt_id.clone(),
    };
    let original = snapshot(&endpoint, &request, wire::Response::Status(status.clone()), status);
    for field in 0..9 {
        let mut bad = original.clone();
        match field {
            0 => bad.service = "did:example:foreign".into(),
            1 => bad.project = "foreign".into(),
            2 => bad.owner = "did:example:foreign".into(),
            3 => bad.request_digest = [99; 32],
            4 => bad.producer = [99; 32],
            5 => bad.schema = "unsupported".into(),
            6 => bad.created_seconds = u64::MAX,
            7 => bad.created_seconds = (NOW_MS + mantle::CLOCK_SKEW_MS) / 1000 + 1,
            _ => bad.created_seconds = 0,
        }
        sign(&mut bad);
        assert!(
            matches!(rejected(client.verify_response(&request, bad, NOW_MS)), Error::Binding | Error::Bounds),
            "field {field}"
        );
    }
    let mut tampered = original.clone();
    tampered.status.as_mut().unwrap().sequence += 1;
    assert_eq!(rejected(client.verify_response(&request, tampered, NOW_MS)), Error::Signature);
    let mut malformed = original.clone();
    malformed.signature.truncate(63);
    assert_eq!(rejected(client.verify_response(&request, malformed, NOW_MS)), Error::Signature);
    assert_eq!(
        rejected(client.verify_response(&request, original, NOW_MS + mantle::MAX_EVENT_AGE_MS + 1)),
        Error::Binding
    );
}

#[test]
fn authenticated_status_cannot_relabel_attempt_fence_request_or_source_cohort() {
    let (endpoint, plan, client, native) = admitted();
    for field in 0..6 {
        let mut status = status(&plan);
        match field {
            0 => status.attempt_id = "foreign".into(),
            1 => status.request_digest = [99; 32],
            2 => status.observation.fence += 1,
            3 => status.observation.fence = 0,
            4 => status.sequence = 0,
            _ => status.observation.result_evidence = Some([99; 32]),
        }
        let request = wire::Request::Status {
            attempt_id: native.attempt.clone(),
        };
        let signed = snapshot(&endpoint, &request, wire::Response::Status(status.clone()), status);
        let verified = client.verify_response(&request, signed, NOW_MS).unwrap();
        assert_eq!(rejected(client.signed_status(&verified, &native, NOW_MS)), Error::Binding);
    }
    let request = wire::Request::Status {
        attempt_id: native.attempt.clone(),
    };
    let status = status(&plan);
    let verified = client
        .verify_response(
            &request,
            snapshot(&endpoint, &request, wire::Response::Status(status.clone()), status),
            NOW_MS,
        )
        .unwrap();
    let mut foreign = native;
    foreign.binding.producer = Identity([99; 32]);
    assert_eq!(rejected(client.signed_status(&verified, &foreign, NOW_MS)), Error::Binding);
}

fn completion(endpoint: &Endpoint, native: &NativeAttempt) -> wire::Completion {
    let mut completion = wire::Completion {
        version: wire::API_VERSION,
        event_id: [0; 32],
        attempt_id: native.attempt.clone(),
        request_digest: native.native_request_hash.0,
        terminal: wire::Phase::Succeeded,
        result_evidence: Some([9; 32]),
        policy_version: [4; 32],
        sequence: 2,
        producer: endpoint.signing_key,
        signature: Vec::new(),
    };
    completion.event_id = wire::completion_identity(&completion).unwrap();
    let mut message = wire::COMPLETION_DOMAIN.to_vec();
    message.extend_from_slice(&completion.event_id);
    completion.signature = Ed25519SigningKey::from_bytes(SEED).sign(&message).unwrap().as_bytes().to_vec();
    completion
}
fn terminal_status(plan: &PlannedBuild) -> wire::Status {
    let mut status = status(plan);
    status.sequence = 2;
    status.observation.phase = wire::Phase::Succeeded;
    status.observation.result_evidence = Some([9; 32]);
    status
}

#[test]
fn result_discovery_requires_independent_native_completion_signature_and_exact_evidence() {
    let (endpoint, plan, client, native) = admitted();
    let status = terminal_status(&plan);
    let request = wire::Request::Result {
        attempt_id: native.attempt.clone(),
    };
    let good = completion(&endpoint, &native);
    let verify = |completion: wire::Completion| -> VerifiedResponse {
        client
            .verify_response(
                &request,
                snapshot(&endpoint, &request, wire::Response::Result(Some(completion)), status.clone()),
                NOW_MS,
            )
            .unwrap()
    };
    assert_eq!(client.verify_result(&verify(good.clone()), &native, Identity([9; 32])).unwrap(), Identity([9; 32]));
    assert_eq!(rejected(client.verify_result(&verify(good.clone()), &native, Identity([8; 32]))), Error::Binding);
    for field in 0..5 {
        let mut bad = good.clone();
        match field {
            0 => bad.signature[0] ^= 1,
            1 => bad.attempt_id = "foreign".into(),
            2 => bad.request_digest = [99; 32],
            3 => bad.result_evidence = Some([99; 32]),
            _ => bad.sequence += 1,
        }
        assert!(matches!(
            rejected(client.verify_result(&verify(bad), &native, Identity([9; 32]))),
            Error::Binding | Error::Signature
        ));
    }
}

#[test]
fn signed_event_page_preserves_stable_events_and_rejects_missing_or_foreign_fence_facts() {
    let (endpoint, plan, client, native) = admitted();
    let status = terminal_status(&plan);
    let request = wire::Request::Events {
        attempt_id: native.attempt.clone(),
        cursor: None,
        items: 64,
    };
    let good = wire::Event {
        sequence: 2,
        phase: wire::Phase::Succeeded,
        completion: Some(completion(&endpoint, &native)),
        observation: Some(status.observation.clone()),
    };
    let verify = |event: wire::Event| {
        client
            .verify_response(
                &request,
                snapshot(
                    &endpoint,
                    &request,
                    wire::Response::Events(wire::Page {
                        values: vec![event],
                        next_cursor: "opaque".into(),
                    }),
                    status.clone(),
                ),
                NOW_MS,
            )
            .unwrap()
    };
    let first = client.verify_event(&verify(good.clone()), &good, &native).unwrap();
    let duplicate = client.verify_event(&verify(good.clone()), &good, &native).unwrap();
    assert_eq!(first, duplicate);
    assert_eq!(first.evidence, BTreeSet::from([Identity([9; 32])]));
    for field in 0..4 {
        let mut bad = good.clone();
        match field {
            0 => bad.observation = None,
            1 => bad.observation.as_mut().unwrap().fence += 1,
            2 => bad.sequence = 0,
            _ => bad.completion = None,
        }
        assert_eq!(rejected(client.verify_event(&verify(bad.clone()), &bad, &native)), Error::Binding);
    }
}
