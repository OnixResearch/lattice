// r[verify basalt.peer_auth.policy]
// r[verify basalt.peer_auth.enforcement]
// r[verify basalt.peer_auth.decision]
// r[verify basalt.peer_auth.reference]
// r[verify basalt.peer_auth.boundary]
// r[verify basalt.peer_auth.verification]

use basalt::peer_auth::{
    DEFAULT_PEER_AUTH_POLICY_JSON, PeerAuthAction, PeerAuthAuthority, PeerAuthBoundary,
    PeerAuthIssue, PeerAuthPolicyError, PeerAuthRequest, evaluate_peer_auth,
    parse_peer_auth_policy,
};
use serde_json::Value;
use ucan_core::peer_auth::{
    AUTHENTICATION_KEY_BYTES, AuthenticationKey, CELLD_REFERENCE_LICENSE, CELLD_REFERENCE_REVISION,
    CELLD_REFERENCE_SOURCE_BLOB, MESSAGE_IDENTITY_BYTES, MessageBoundary, MessageDigestRole,
    MessageDraft, MessageExpectation, PEER_AUTH_PROTOCOL_VERSION,
    PeerAuthIssue as UcanPeerAuthIssue, ProtocolVersion, REPLAY_NONCE_BYTES, ReplayNonce,
    ReplayObservation, VerifiedAuthenticatedMessage, WireMessageDigest, authenticate_message,
    validate_digest_role, verify_authenticated_message,
};

const AUTHENTICATION_KEY_BYTE: u8 = 7;
const REPLAY_NONCE_BYTE: u8 = 9;
const TEST_TIMESTAMP_MS: u64 = 1_000;
const CLOCK_SKEW_MS: u64 = 100;
const STALE_OBSERVED_TIME_MS: u64 = TEST_TIMESTAMP_MS + CLOCK_SKEW_MS + 1;
const OTHER_PROTOCOL_VERSION: u16 = PEER_AUTH_PROTOCOL_VERSION + 1;
const ROUTING_CONTEXT: &[u8] = b"peer/control";
const BODY: &[u8] = b"bounded peer control message";
const SECRET_SENTINEL: &str = "shared-secret-must-not-leak";
const EMPTY_BODY: &[u8] = b"";
const ZERO_DIGEST_BYTE: u8 = 0;
const EXPECTED_ALLOW_CASES: usize = 1;
const EXPECTED_DENY_CASES: usize = 5;
const STEEL_FIXTURES_JSON: &str =
    include_str!("../contracts/peer-auth/generated/steel-fixtures.json");

const fn key() -> AuthenticationKey {
    AuthenticationKey::new([AUTHENTICATION_KEY_BYTE; AUTHENTICATION_KEY_BYTES])
}

const fn nonce() -> ReplayNonce {
    ReplayNonce::new([REPLAY_NONCE_BYTE; REPLAY_NONCE_BYTES])
}

fn version(value: u16) -> ProtocolVersion {
    ProtocolVersion::new(value).expect("test protocol version must be non-zero")
}

fn verified_message(protocol_version: u16) -> VerifiedAuthenticatedMessage {
    let key = key();
    let protocol_version = version(protocol_version);
    let replay_nonce = nonce();
    let frame = authenticate_message(
        MessageDraft {
            protocol_version,
            timestamp_ms: TEST_TIMESTAMP_MS,
            nonce: replay_nonce,
            routing_context: ROUTING_CONTEXT,
            body: BODY,
            boundary: MessageBoundary::None,
        },
        &key,
    )
    .expect("test message must authenticate");
    verify_authenticated_message(
        &frame,
        &key,
        MessageExpectation {
            protocol_version,
            routing_context: ROUTING_CONTEXT,
            observed_now_ms: TEST_TIMESTAMP_MS,
            max_clock_skew_ms: CLOCK_SKEW_MS,
            replay: ReplayObservation::Fresh {
                nonce: replay_nonce,
            },
        },
    )
    .expect("test message must verify")
}

const fn admitted_request() -> PeerAuthRequest {
    PeerAuthRequest::new(
        PeerAuthAction::DispatchControlMessage,
        Some(PeerAuthAuthority::AuthenticatedPeerMessage),
        PeerAuthBoundary::None,
    )
}

#[test]
fn steel_fixture_matrix_has_positive_and_negative_decisions() {
    let fixtures: Value =
        serde_json::from_str(STEEL_FIXTURES_JSON).expect("Steel fixture JSON must parse");
    let cases = fixtures["cases"].as_array().expect("cases array");
    let allow_count = cases
        .iter()
        .filter(|case| case["expected"] == "allow")
        .count();
    let deny_count = cases
        .iter()
        .filter(|case| case["expected"] == "deny")
        .count();
    assert_eq!(allow_count, EXPECTED_ALLOW_CASES);
    assert_eq!(deny_count, EXPECTED_DENY_CASES);
}

#[test]
fn exact_policy_and_authenticated_verdict_admit_one_bound_action() {
    let policy = parse_peer_auth_policy(DEFAULT_PEER_AUTH_POLICY_JSON)
        .expect("checked peer policy must parse");
    let verdict = verified_message(PEER_AUTH_PROTOCOL_VERSION);
    let decision = evaluate_peer_auth(&policy, &verdict, admitted_request())
        .expect("exact peer message must be admitted");

    assert!(decision.receipt().is_allowed());
    assert_eq!(decision.receipt().issue(), None);
    let serialized = serde_json::to_string(decision.receipt()).expect("receipt JSON");
    assert!(!serialized.contains(std::str::from_utf8(BODY).expect("test body UTF-8")));
    assert!(
        !serialized.contains(std::str::from_utf8(ROUTING_CONTEXT).expect("test context UTF-8"))
    );
    assert!(!serialized.contains(SECRET_SENTINEL));
}

#[test]
fn version_action_authority_and_metadata_promotion_fail_closed() {
    let policy = parse_peer_auth_policy(DEFAULT_PEER_AUTH_POLICY_JSON)
        .expect("checked peer policy must parse");
    let wrong_version = verified_message(OTHER_PROTOCOL_VERSION);
    let failure = evaluate_peer_auth(&policy, &wrong_version, admitted_request())
        .expect_err("wrong protocol version must deny");
    assert_eq!(failure.issue(), PeerAuthIssue::ProtocolVersionMismatch);

    let verdict = verified_message(PEER_AUTH_PROTOCOL_VERSION);
    let wrong_action = PeerAuthRequest::new(
        PeerAuthAction::PublishStatus,
        Some(PeerAuthAuthority::AuthenticatedPeerMessage),
        PeerAuthBoundary::None,
    );
    assert_eq!(
        evaluate_peer_auth(&policy, &verdict, wrong_action)
            .expect_err("wrong action must deny")
            .issue(),
        PeerAuthIssue::ActionDenied
    );
    let missing_authority = PeerAuthRequest::new(
        PeerAuthAction::DispatchControlMessage,
        None,
        PeerAuthBoundary::None,
    );
    assert_eq!(
        evaluate_peer_auth(&policy, &verdict, missing_authority)
            .expect_err("missing authority must deny")
            .issue(),
        PeerAuthIssue::MissingAuthority
    );
    let promoted = PeerAuthRequest::new(
        PeerAuthAction::DispatchControlMessage,
        Some(PeerAuthAuthority::AuthenticatedPeerMessage),
        PeerAuthBoundary::PromotesEndUserAuthority,
    );
    assert_eq!(
        evaluate_peer_auth(&policy, &verdict, promoted)
            .expect_err("end-user promotion must deny")
            .issue(),
        PeerAuthIssue::BoundaryPromotion
    );
}

#[test]
fn ucan_rejects_stale_replayed_empty_digest_and_subject_promotion_inputs() {
    let key = key();
    let protocol_version = version(PEER_AUTH_PROTOCOL_VERSION);
    let replay_nonce = nonce();
    let frame = authenticate_message(
        MessageDraft {
            protocol_version,
            timestamp_ms: TEST_TIMESTAMP_MS,
            nonce: replay_nonce,
            routing_context: ROUTING_CONTEXT,
            body: BODY,
            boundary: MessageBoundary::None,
        },
        &key,
    )
    .expect("test frame");

    let stale = MessageExpectation {
        protocol_version,
        routing_context: ROUTING_CONTEXT,
        observed_now_ms: STALE_OBSERVED_TIME_MS,
        max_clock_skew_ms: CLOCK_SKEW_MS,
        replay: ReplayObservation::Fresh {
            nonce: replay_nonce,
        },
    };
    assert_eq!(
        verify_authenticated_message(&frame, &key, stale),
        Err(UcanPeerAuthIssue::MessageStale)
    );
    let replayed = MessageExpectation {
        observed_now_ms: TEST_TIMESTAMP_MS,
        replay: ReplayObservation::Replayed {
            nonce: replay_nonce,
        },
        ..stale
    };
    assert_eq!(
        verify_authenticated_message(&frame, &key, replayed),
        Err(UcanPeerAuthIssue::ReplayDenied)
    );

    let empty = authenticate_message(
        MessageDraft {
            protocol_version,
            timestamp_ms: TEST_TIMESTAMP_MS,
            nonce: replay_nonce,
            routing_context: ROUTING_CONTEXT,
            body: EMPTY_BODY,
            boundary: MessageBoundary::None,
        },
        &key,
    );
    assert_eq!(empty, Err(UcanPeerAuthIssue::EmptyMessageBody));

    let promoted = authenticate_message(
        MessageDraft {
            protocol_version,
            timestamp_ms: TEST_TIMESTAMP_MS,
            nonce: replay_nonce,
            routing_context: ROUTING_CONTEXT,
            body: BODY,
            boundary: MessageBoundary::PromotesSubject,
        },
        &key,
    );
    assert_eq!(promoted, Err(UcanPeerAuthIssue::BoundaryPromotion));

    let wrong_role = WireMessageDigest {
        role: MessageDigestRole::BodySha256,
        bytes: [ZERO_DIGEST_BYTE; MESSAGE_IDENTITY_BYTES],
    };
    assert_eq!(
        validate_digest_role(wrong_role, MessageDigestRole::MessageBlake3),
        Err(UcanPeerAuthIssue::DigestRoleMismatch)
    );
}

#[test]
fn policy_rejects_secret_fields_stale_sources_and_reference_overclaims() {
    let mut secret: Value =
        serde_json::from_str(DEFAULT_PEER_AUTH_POLICY_JSON).expect("checked policy JSON");
    secret["shared_secret"] = Value::String(SECRET_SENTINEL.to_owned());
    assert_eq!(
        parse_peer_auth_policy(&serde_json::to_string(&secret).expect("JSON")),
        Err(PeerAuthPolicyError::Malformed)
    );

    let mut stale: Value =
        serde_json::from_str(DEFAULT_PEER_AUTH_POLICY_JSON).expect("checked policy JSON");
    stale["ucan_revision"] = Value::String("0".repeat(CELLD_REFERENCE_REVISION.len()));
    assert_eq!(
        parse_peer_auth_policy(&serde_json::to_string(&stale).expect("JSON")),
        Err(PeerAuthPolicyError::SourceMismatch)
    );

    let mut parity: Value =
        serde_json::from_str(DEFAULT_PEER_AUTH_POLICY_JSON).expect("checked policy JSON");
    parity["reference"]["claim"] = Value::String("parity".to_owned());
    assert_eq!(
        parse_peer_auth_policy(&serde_json::to_string(&parity).expect("JSON")),
        Err(PeerAuthPolicyError::ReferenceMismatch)
    );

    assert_eq!(
        CELLD_REFERENCE_REVISION,
        "c978a2590e875b003ecd860a86cb7551287eba67"
    );
    assert_eq!(
        CELLD_REFERENCE_SOURCE_BLOB,
        "40d6ac4613d293a148e7222e192153eaeaeb29db"
    );
    assert_eq!(CELLD_REFERENCE_LICENSE, "Apache-2.0");
}
