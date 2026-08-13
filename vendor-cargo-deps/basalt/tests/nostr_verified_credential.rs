use std::fmt::Write as _;

use basalt::nostr_identity::NostrAuthenticationDecision;
use basalt::nostr_identity::NostrDigestRole;
use basalt::nostr_identity::NostrIdentityBindingRequest;
use basalt::nostr_identity::NostrIdentityBoundary;
use basalt::nostr_identity::NostrTypedDigest;
use basalt::nostr_identity::parse_nostr_identity_policy;
use basalt::nostr_identity::resolve_verified_nostr_subject;
use basalt::nostr_identity::verify_nostr_authentication_receipt;
use k256::schnorr::Signature;
use k256::schnorr::SigningKey;
use ucan_nostr::AuthenticationExpectation;
use ucan_nostr::BoundaryClaims;
use ucan_nostr::FreshnessExpectation;
use ucan_nostr::NostrEvent;
use ucan_nostr::NostrEventId;
use ucan_nostr::ReplayObservation;
use ucan_nostr::VerificationRequest;
use ucan_nostr::compute_event_id;
use ucan_nostr::verify_credential;

const GENERATED_POLICY: &str = include_str!("../contracts/nostr-identity/generated/policy.json");
const SECRET_KEY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000003";
const SECRET_KEY_BYTES: usize = 32;
const HEX_RADIX: u32 = 16;
const HEX_CHARS_PER_BYTE: usize = 2;
const EVENT_TIME: u64 = 1_700_000_000;
const MAX_AGE_SECONDS: u64 = 600;
const MAX_FUTURE_SKEW_SECONDS: u64 = 60;
const APPLICATION_EVENT_KIND: u32 = 31_337;
const DOMAIN: &str = "onix.example.identity";
const AUDIENCE: &str = "service:operator";
const PURPOSE: &str = "authenticate-control-plane";
const CHALLENGE: &str = "challenge:application";
const REQUEST_ID: &str = "request:42";

fn decode_secret_key() -> [u8; SECRET_KEY_BYTES] {
    assert_eq!(SECRET_KEY_HEX.len(), SECRET_KEY_BYTES * HEX_CHARS_PER_BYTE);
    let mut output = [0_u8; SECRET_KEY_BYTES];
    for (index, byte) in output.iter_mut().enumerate() {
        let offset = index * HEX_CHARS_PER_BYTE;
        let end = offset + HEX_CHARS_PER_BYTE;
        *byte = u8::from_str_radix(&SECRET_KEY_HEX[offset..end], HEX_RADIX).expect("test key uses lowercase hex");
    }
    output
}

fn encode_lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * HEX_CHARS_PER_BYTE);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("string formatting is infallible");
    }
    output
}

fn tag(name: &str, value: &str) -> Vec<String> {
    vec![name.to_owned(), value.to_owned()]
}

fn signed_event() -> NostrEvent {
    let signing_key = SigningKey::from_bytes(&decode_secret_key()).expect("valid test key");
    let mut event = NostrEvent {
        id: String::new(),
        pubkey: encode_lower_hex(signing_key.verifying_key().to_bytes().as_slice()),
        created_at: EVENT_TIME,
        kind: APPLICATION_EVENT_KIND,
        tags: vec![
            tag("domain", DOMAIN),
            tag("audience", AUDIENCE),
            tag("purpose", PURPOSE),
            tag("challenge", CHALLENGE),
            tag("request", REQUEST_ID),
        ],
        content: String::new(),
        sig: String::new(),
    };
    let event_id = compute_event_id(&event).expect("bounded event");
    let auxiliary_random = [0_u8; SECRET_KEY_BYTES];
    let signature: Signature = signing_key.sign_raw(event_id.as_bytes(), &auxiliary_random).expect("test event signs");
    event.id = event_id.to_hex();
    event.sig = encode_lower_hex(signature.to_bytes().as_slice());
    event
}

fn verified_credential(event: &NostrEvent) -> ucan_nostr::VerifiedNostrCredential {
    let event_id = NostrEventId::parse(&event.id).expect("event ID");
    verify_credential(event, VerificationRequest {
        authentication: AuthenticationExpectation::ApplicationV1 {
            domain: DOMAIN,
            audience: AUDIENCE,
            purpose: PURPOSE,
            challenge: CHALLENGE,
            request_identity: REQUEST_ID,
            event_kind: APPLICATION_EVENT_KIND,
        },
        freshness: FreshnessExpectation {
            observed_now: EVENT_TIME,
            max_age_seconds: MAX_AGE_SECONDS,
            max_future_skew_seconds: MAX_FUTURE_SKEW_SECONDS,
        },
        replay: ReplayObservation::Fresh { event_id },
        boundary: BoundaryClaims::default(),
    })
    .expect("reviewed UCAN credential")
}

// r[verify nostr_identity_bindings.verified_input]
// r[verify nostr_identity_bindings.subject_resolution]
#[test]
fn reviewed_opaque_credential_resolves_through_the_public_adapter() {
    let event = signed_event();
    let credential = verified_credential(&event);
    let policy = parse_nostr_identity_policy(GENERATED_POLICY).expect("generated policy");
    let request = NostrIdentityBindingRequest {
        request_identity: NostrTypedDigest {
            role: NostrDigestRole::UcanAdapterBlake3,
            hex: credential.request_reference().to_hex(),
        },
        audience: AUDIENCE.to_owned(),
        purpose: PURPOSE.to_owned(),
        policy_identity: policy.policy_identity().clone(),
        boundary: NostrIdentityBoundary::PublicOnly,
    };

    let authenticated = resolve_verified_nostr_subject(&policy, &credential, &request);
    assert_eq!(authenticated.receipt.decision, NostrAuthenticationDecision::Authenticated);
    assert_eq!(
        authenticated.subject().map(basalt::nostr_identity::AuthenticatedNostrSubject::subject_id),
        Some("subject:operator")
    );
    assert!(verify_nostr_authentication_receipt(&authenticated.receipt).is_empty());

    let mut tampered_request = request;
    tampered_request.audience = "service:other".to_owned();
    let denied = resolve_verified_nostr_subject(&policy, &credential, &tampered_request);
    assert_eq!(denied.receipt.decision, NostrAuthenticationDecision::Denied);
    assert!(denied.subject().is_none());
}
