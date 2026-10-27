// r[verify basalt.ucan_token_flow.v1_profile.upstream]
// r[verify basalt.ucan_token_flow.v1_profile.tests]
// r[verify basalt.ucan_token_flow.v1_profile.authority]
// r[verify basalt.ucan_token_flow.v1_profile.receipt]

use basalt::authority_core::default_authority_policy_ref;
use basalt::ucan::core::profile::v1 as wire_core;
use basalt::ucan::profile::v1::{
    Cbor, DidKeyResolver, canonical_payload_bytes, cid_v1_of_payload, sign_payload,
};
use basalt::ucan::profile::v1_authorization::{
    V1DelegationStore, V1DelegationStoreError, V1InvocationReplay, V1ReplayError,
    V1RevocationObservation,
};
use basalt::ucan_profile::BasaltUcanProtocolProfile;
use basalt::ucan_v1_profile::{
    FINAL_PROFILE_MAPPING_JSON, FinalProfileEnforcementRequest, FinalProfileReason,
    ResourceOwnerError, ResourceOwnerVerifier, parse_final_profile_mapping,
    verify_final_profile_enforcement,
};
use basalt::{EnforcementRequest, default_policy};
use ed25519_dalek::SigningKey;
use std::cell::Cell;
use std::collections::BTreeMap;

const ROOT_KEY_BYTE: u8 = 1;
const MIDDLE_KEY_BYTE: u8 = 3;
const INVOKER_KEY_BYTE: u8 = 7;
const FIRST_ADMISSION_TIME: i64 = 1_000;
const LAST_ADMISSION_TIME: i64 = 2_000;
const VERIFICATION_TIME: i64 = 1_500;
const CLOCK_SKEW_SECONDS: i64 = 60;
const INITIAL_CALL_COUNT: usize = 0;
const ONE_CALL: usize = 1;
const EXPECTED_LICENSE_COUNT: usize = 3;
const SUBJECT: &str = "did:key:z6Mksubject";
const DOCUMENT_RESOURCE: &str = "did:key:z6Mkresource/docs/final-profile";
const OUTSIDE_POLICY_RESOURCE: &str = "https://outside.example/document";
const DOCUMENT_WRITE_COMMAND: &str = "/document/write";
const UNKNOWN_COMMAND: &str = "/document/unknown";
const ROOT_COMMAND: &str = "/document";
const SOURCE_EVIDENCE_JSON: &str =
    include_str!("../contracts/ucan-v1-profile/generated/source-evidence.json");
const EXPECTED_RUNTIME_REVISION: &str = "3a2aa043068a82663cf72ec05ed5d56a7d65ecd5";
const EXPECTED_SOURCE_MANIFEST: &str =
    "82b5e1c414fadcbdc90ef951624c9ccd4e8eb6bdce0d57daf2830fd79cd592fb";

struct KeyMap(BTreeMap<String, [u8; ed25519_dalek::PUBLIC_KEY_LENGTH]>);

impl DidKeyResolver for KeyMap {
    fn resolve_did_key(&self, did: &str) -> Result<[u8; ed25519_dalek::PUBLIC_KEY_LENGTH], String> {
        self.0
            .get(did)
            .copied()
            .ok_or_else(|| "key unavailable".to_owned())
    }
}

#[derive(Default)]
struct MemoryStore {
    envelopes: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl V1DelegationStore for MemoryStore {
    fn get(&self, cid: &[u8]) -> Result<Option<Vec<u8>>, V1DelegationStoreError> {
        Ok(self.envelopes.get(cid).cloned())
    }
}

struct NoRevocations;

impl V1RevocationObservation for NoRevocations {
    fn is_revoked(&self, _cid: &[u8]) -> bool {
        false
    }
}

struct CountingReplay {
    calls: Cell<usize>,
    result: Result<(), V1ReplayError>,
}

impl CountingReplay {
    const fn allow() -> Self {
        Self {
            calls: Cell::new(INITIAL_CALL_COUNT),
            result: Ok(()),
        }
    }

    const fn deny() -> Self {
        Self {
            calls: Cell::new(INITIAL_CALL_COUNT),
            result: Err(V1ReplayError::Duplicate),
        }
    }
}

impl V1InvocationReplay for CountingReplay {
    fn admit(
        &self,
        _invocation_cid: &[u8],
        _command: &str,
        _subject: &str,
    ) -> Result<(), V1ReplayError> {
        self.calls.set(self.calls.get() + ONE_CALL);
        self.result
    }
}

struct ExactOwner {
    resource: &'static str,
    result: Result<(), ResourceOwnerError>,
    calls: Cell<usize>,
}

impl ExactOwner {
    const fn allow(resource: &'static str) -> Self {
        Self {
            resource,
            result: Ok(()),
            calls: Cell::new(INITIAL_CALL_COUNT),
        }
    }

    const fn deny(resource: &'static str) -> Self {
        Self {
            resource,
            result: Err(ResourceOwnerError::Denied),
            calls: Cell::new(INITIAL_CALL_COUNT),
        }
    }
}

impl ResourceOwnerVerifier for ExactOwner {
    fn verify(
        &self,
        subject: &str,
        resource: &str,
    ) -> Result<[u8; blake3::OUT_LEN], ResourceOwnerError> {
        self.calls.set(self.calls.get() + ONE_CALL);
        self.result?;
        if subject != SUBJECT || resource != self.resource {
            return Err(ResourceOwnerError::Denied);
        }
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"basalt.test.resource-owner\0");
        hasher.update(subject.as_bytes());
        hasher.update(resource.as_bytes());
        Ok(*hasher.finalize().as_bytes())
    }
}

struct FinalFixture {
    keys: KeyMap,
    store: MemoryStore,
    invocation: Vec<u8>,
    executor: String,
}

fn signing_key(byte: u8) -> SigningKey {
    SigningKey::from_bytes(&[byte; ed25519_dalek::SECRET_KEY_LENGTH])
}

fn did_of(key: &SigningKey) -> String {
    String::from_utf8(
        basalt::ucan::encode_ed25519_did_key(&key.verifying_key().to_bytes())
            .expect("test public key must encode"),
    )
    .expect("encoded did:key must be UTF-8")
}

fn delegation_fields(
    issuer: &str,
    audience: &str,
    subject: Option<&str>,
    command: &str,
) -> Vec<(String, Cbor)> {
    let mut fields = vec![
        (
            wire_core::FIELD_TAG_ANNOTATION.to_owned(),
            Cbor::Text(wire_core::DELEGATION_TAG.to_owned()),
        ),
        (
            wire_core::FIELD_VERSION.to_owned(),
            Cbor::Text(wire_core::WIRE_VERSION.to_owned()),
        ),
        (
            wire_core::FIELD_ISSUER.to_owned(),
            Cbor::Text(issuer.to_owned()),
        ),
        (
            wire_core::FIELD_AUDIENCE.to_owned(),
            Cbor::Text(audience.to_owned()),
        ),
        (
            wire_core::FIELD_COMMAND.to_owned(),
            Cbor::Text(command.to_owned()),
        ),
        (
            wire_core::FIELD_NOT_BEFORE.to_owned(),
            Cbor::UInt(u64::try_from(FIRST_ADMISSION_TIME).expect("positive test time")),
        ),
        (
            wire_core::FIELD_EXPIRES.to_owned(),
            Cbor::UInt(u64::try_from(LAST_ADMISSION_TIME).expect("positive test time")),
        ),
    ];
    if let Some(subject) = subject {
        fields.push((
            wire_core::FIELD_SUBJECT.to_owned(),
            Cbor::Text(subject.to_owned()),
        ));
    }
    fields
}

fn invocation_fields(
    issuer: &str,
    audience: &str,
    command: &str,
    proofs: &[Vec<u8>],
) -> Vec<(String, Cbor)> {
    vec![
        (
            wire_core::FIELD_TAG_ANNOTATION.to_owned(),
            Cbor::Text(wire_core::INVOCATION_TAG.to_owned()),
        ),
        (
            wire_core::FIELD_VERSION.to_owned(),
            Cbor::Text(wire_core::WIRE_VERSION.to_owned()),
        ),
        (
            wire_core::FIELD_ISSUER.to_owned(),
            Cbor::Text(issuer.to_owned()),
        ),
        (
            wire_core::FIELD_AUDIENCE.to_owned(),
            Cbor::Text(audience.to_owned()),
        ),
        (
            wire_core::FIELD_SUBJECT.to_owned(),
            Cbor::Text(SUBJECT.to_owned()),
        ),
        (
            wire_core::FIELD_COMMAND.to_owned(),
            Cbor::Text(command.to_owned()),
        ),
        (
            wire_core::FIELD_PROOFS.to_owned(),
            Cbor::Array(proofs.iter().cloned().map(Cbor::Bytes).collect()),
        ),
    ]
}

fn install_delegation(
    store: &mut MemoryStore,
    key: &SigningKey,
    fields: &[(String, Cbor)],
) -> Vec<u8> {
    let cid = cid_v1_of_payload(&canonical_payload_bytes(fields));
    let envelope = sign_payload(fields, key);
    let previous = store.envelopes.insert(cid.clone(), envelope);
    assert!(previous.is_none(), "test proof CIDs must be unique");
    cid
}

fn final_fixture(delegated: bool, command: &str) -> FinalFixture {
    let root = signing_key(ROOT_KEY_BYTE);
    let middle = signing_key(MIDDLE_KEY_BYTE);
    let invoker = signing_key(INVOKER_KEY_BYTE);
    let root_did = did_of(&root);
    let middle_did = did_of(&middle);
    let invoker_did = did_of(&invoker);
    let mut store = MemoryStore::default();
    let mut proofs = Vec::new();

    if delegated {
        proofs.push(install_delegation(
            &mut store,
            &root,
            &delegation_fields(&root_did, &middle_did, Some(SUBJECT), ROOT_COMMAND),
        ));
        proofs.push(install_delegation(
            &mut store,
            &middle,
            &delegation_fields(&middle_did, &invoker_did, None, DOCUMENT_WRITE_COMMAND),
        ));
    } else {
        proofs.push(install_delegation(
            &mut store,
            &root,
            &delegation_fields(&root_did, &invoker_did, Some(SUBJECT), ROOT_COMMAND),
        ));
    }

    let fields = invocation_fields(&invoker_did, &invoker_did, command, &proofs);
    let invocation = sign_payload(&fields, &invoker);
    let keys = [
        (root_did, root.verifying_key().to_bytes()),
        (middle_did, middle.verifying_key().to_bytes()),
        (invoker_did.clone(), invoker.verifying_key().to_bytes()),
    ]
    .into_iter()
    .collect();
    FinalFixture {
        keys: KeyMap(keys),
        store,
        invocation,
        executor: invoker_did,
    }
}

fn document_request(resource: &str) -> EnforcementRequest {
    EnforcementRequest::new("document-write", resource, "document/write")
}

#[test]
fn source_evidence_binds_archives_manifest_and_licenses() {
    let evidence: serde_json::Value =
        serde_json::from_str(SOURCE_EVIDENCE_JSON).expect("source evidence JSON must parse");
    assert_eq!(evidence["runtime_revision"], EXPECTED_RUNTIME_REVISION);
    assert_eq!(evidence["source_manifest_blake3"], EXPECTED_SOURCE_MANIFEST);
    assert_eq!(evidence["wire_archive"]["verdict"], "PASS");
    assert_eq!(evidence["authorization_archive"]["verdict"], "PASS");
    assert_eq!(evidence["profile_supported"], true);
    assert_eq!(evidence["conformance_tested"], true);
    assert_eq!(
        evidence["licenses"]
            .as_array()
            .expect("license array")
            .len(),
        EXPECTED_LICENSE_COUNT
    );
}

#[test]
fn direct_and_delegated_final_invocations_return_live_exact_authority() {
    for delegated in [false, true] {
        let fixture = final_fixture(delegated, DOCUMENT_WRITE_COMMAND);
        let policy = default_policy().expect("default policy");
        let trusted = default_authority_policy_ref("document-write");
        let mapping =
            parse_final_profile_mapping(FINAL_PROFILE_MAPPING_JSON).expect("checked mapping");
        let enforcement = document_request(DOCUMENT_RESOURCE);
        let request = FinalProfileEnforcementRequest::new(
            &enforcement,
            &fixture.invocation,
            BasaltUcanProtocolProfile::UcanWg100,
            fixture.executor.as_str(),
        );
        let replay = CountingReplay::allow();
        let owner = ExactOwner::allow(DOCUMENT_RESOURCE);
        let live = verify_final_profile_enforcement(
            &policy,
            &trusted,
            &mapping,
            &request,
            &fixture.keys,
            &fixture.store,
            &NoRevocations,
            &replay,
            &owner,
            VERIFICATION_TIME,
            CLOCK_SKEW_SECONDS,
        )
        .expect("final profile must authorize");

        assert_eq!(live.request().resource(), DOCUMENT_RESOURCE);
        assert_eq!(
            live.authorization().command().as_str(),
            DOCUMENT_WRITE_COMMAND
        );
        assert_eq!(live.authorization().subject(), SUBJECT);
        assert!(live.receipt().is_allowed());
        assert!(live.ucan_receipt().allowed);
        assert_eq!(replay.calls.get(), ONE_CALL);
        assert_eq!(owner.calls.get(), ONE_CALL);
        let serialized = serde_json::to_string(live.receipt()).expect("receipt JSON");
        assert!(!serialized.contains(SUBJECT));
        assert!(!serialized.contains(DOCUMENT_RESOURCE));
        assert!(!serialized.contains(DOCUMENT_WRITE_COMMAND));
    }
}

#[test]
fn mapping_and_owner_denials_never_call_final_replay() {
    let policy = default_policy().expect("default policy");
    let trusted = default_authority_policy_ref("document-write");
    let mapping = parse_final_profile_mapping(FINAL_PROFILE_MAPPING_JSON).expect("checked mapping");

    let unknown = final_fixture(false, UNKNOWN_COMMAND);
    let enforcement = document_request(DOCUMENT_RESOURCE);
    let request = FinalProfileEnforcementRequest::new(
        &enforcement,
        &unknown.invocation,
        BasaltUcanProtocolProfile::UcanWg100,
        unknown.executor.as_str(),
    );
    let replay = CountingReplay::allow();
    let owner = ExactOwner::allow(DOCUMENT_RESOURCE);
    let failure = verify_final_profile_enforcement(
        &policy,
        &trusted,
        &mapping,
        &request,
        &unknown.keys,
        &unknown.store,
        &NoRevocations,
        &replay,
        &owner,
        VERIFICATION_TIME,
        CLOCK_SKEW_SECONDS,
    )
    .expect_err("unknown mapping must deny");
    assert_eq!(failure.reason(), FinalProfileReason::UnknownCommand);
    assert_eq!(replay.calls.get(), INITIAL_CALL_COUNT);
    assert_eq!(owner.calls.get(), INITIAL_CALL_COUNT);

    let valid = final_fixture(false, DOCUMENT_WRITE_COMMAND);
    let request = FinalProfileEnforcementRequest::new(
        &enforcement,
        &valid.invocation,
        BasaltUcanProtocolProfile::UcanWg100,
        valid.executor.as_str(),
    );
    let replay = CountingReplay::allow();
    let owner = ExactOwner::deny(DOCUMENT_RESOURCE);
    let failure = verify_final_profile_enforcement(
        &policy,
        &trusted,
        &mapping,
        &request,
        &valid.keys,
        &valid.store,
        &NoRevocations,
        &replay,
        &owner,
        VERIFICATION_TIME,
        CLOCK_SKEW_SECONDS,
    )
    .expect_err("owner denial must deny");
    assert_eq!(failure.reason(), FinalProfileReason::ResourceOwnerDenied);
    assert_eq!(replay.calls.get(), INITIAL_CALL_COUNT);
}

#[test]
fn policy_and_profile_denials_never_call_final_replay() {
    let policy = default_policy().expect("default policy");
    let trusted = default_authority_policy_ref("document-write");
    let mapping = parse_final_profile_mapping(FINAL_PROFILE_MAPPING_JSON).expect("checked mapping");
    let valid = final_fixture(false, DOCUMENT_WRITE_COMMAND);
    let enforcement = document_request(DOCUMENT_RESOURCE);
    let outside = document_request(OUTSIDE_POLICY_RESOURCE);
    let request = FinalProfileEnforcementRequest::new(
        &outside,
        &valid.invocation,
        BasaltUcanProtocolProfile::UcanWg100,
        valid.executor.as_str(),
    );
    let replay = CountingReplay::allow();
    let owner = ExactOwner::allow(OUTSIDE_POLICY_RESOURCE);
    let failure = verify_final_profile_enforcement(
        &policy,
        &trusted,
        &mapping,
        &request,
        &valid.keys,
        &valid.store,
        &NoRevocations,
        &replay,
        &owner,
        VERIFICATION_TIME,
        CLOCK_SKEW_SECONDS,
    )
    .expect_err("Basalt policy denial must deny");
    assert_eq!(failure.reason(), FinalProfileReason::PolicyDenied);
    assert_eq!(replay.calls.get(), INITIAL_CALL_COUNT);

    let request = FinalProfileEnforcementRequest::new(
        &enforcement,
        &valid.invocation,
        BasaltUcanProtocolProfile::OnixCompactEd25519V1,
        valid.executor.as_str(),
    );
    let replay = CountingReplay::allow();
    let owner = ExactOwner::allow(DOCUMENT_RESOURCE);
    let failure = verify_final_profile_enforcement(
        &policy,
        &trusted,
        &mapping,
        &request,
        &valid.keys,
        &valid.store,
        &NoRevocations,
        &replay,
        &owner,
        VERIFICATION_TIME,
        CLOCK_SKEW_SECONDS,
    )
    .expect_err("compact profile substitution must deny");
    assert_eq!(failure.reason(), FinalProfileReason::ProtocolProfileDenied);
    assert_eq!(replay.calls.get(), INITIAL_CALL_COUNT);
}

#[test]
fn final_replay_is_called_once_and_persisted_receipts_are_not_authority() {
    let fixture = final_fixture(false, DOCUMENT_WRITE_COMMAND);
    let policy = default_policy().expect("default policy");
    let trusted = default_authority_policy_ref("document-write");
    let mapping = parse_final_profile_mapping(FINAL_PROFILE_MAPPING_JSON).expect("checked mapping");
    let enforcement = document_request(DOCUMENT_RESOURCE);
    let request = FinalProfileEnforcementRequest::new(
        &enforcement,
        &fixture.invocation,
        BasaltUcanProtocolProfile::UcanWg100,
        fixture.executor.as_str(),
    );
    let replay = CountingReplay::deny();
    let owner = ExactOwner::allow(DOCUMENT_RESOURCE);
    let failure = verify_final_profile_enforcement(
        &policy,
        &trusted,
        &mapping,
        &request,
        &fixture.keys,
        &fixture.store,
        &NoRevocations,
        &replay,
        &owner,
        VERIFICATION_TIME,
        CLOCK_SKEW_SECONDS,
    )
    .expect_err("duplicate replay must deny");
    assert_eq!(failure.reason(), FinalProfileReason::ReplayDenied);
    assert_eq!(replay.calls.get(), ONE_CALL);
    assert!(!failure.receipt().is_allowed());

    let persisted = serde_json::to_string(failure.receipt()).expect("receipt JSON");
    let restored: basalt::ucan_v1_profile::FinalProfileReceipt =
        serde_json::from_str(&persisted).expect("receipt must deserialize as evidence");
    assert!(!restored.is_allowed());
    assert!(!persisted.contains(SUBJECT));
    assert!(!persisted.contains(DOCUMENT_RESOURCE));
}
