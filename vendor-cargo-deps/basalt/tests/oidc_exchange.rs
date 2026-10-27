// r[verify ucan_token_flow.oidc.versioned_policy]
// r[verify ucan_token_flow.oidc.pure_verification]
// r[verify ucan_token_flow.oidc.bounded_jwks]
// r[verify ucan_token_flow.oidc.attenuated_ucan]
// r[verify ucan_token_flow.oidc.replay_boundary]
// r[verify ucan_token_flow.oidc.redacted_evidence]

#![cfg(feature = "oidc-exchange")]

use std::collections::{BTreeSet, VecDeque};
use std::sync::OnceLock;

use basalt::oidc_exchange::{
    DocumentFailure, DocumentKind, DocumentPort, DocumentResponse, ExchangeFailureClass,
    ExchangeOutcome, ExchangeRequest, IssuerAdmission, JwksCache, KeyLoadFailure, MantleOperation,
    OidcPolicy, PublicJwk, RejectReason, ReplayCommit, ReplayFailure, ReplayKey, ReplayPort,
    exchange, inspect_token_key_id, load_verification_key, parse_policy,
    signer_verification_digest, validate_receipt_redaction, verify_and_plan,
};
use basalt::secretspec_issuer::{
    ISSUER_RESOLUTION_RESULT_SCHEMA, IssuerKeyDeclaration, IssuerResolutionReason,
    IssuerResolutionResult, REQUIRED_ISSUER_NON_CLAIMS, RotationWindow,
};
use base64::Engine as _;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use serde_json::Value;
use ucan::{
    Ed25519InMemorySigner, KeyResolutionContext, ProofCollection, TokenSigner, VerificationContext,
    VerificationTime, verify_compact_token,
};

const POLICY_JSON: &str = include_str!("../contracts/oidc-exchange/generated/policy.json");
const CLAIMS_JSON: &str = include_str!("fixtures/oidc/github-claims.json");
const NOW_SECONDS: u64 = 1_700_000_060;
const TOKEN_EXPIRATION_SECONDS: u64 = 1_700_000_300;
const REQUEST_LIFETIME_SECONDS: u64 = 120;
const CACHE_FRESH_SECONDS: u64 = 60;
const EXPIRED_TOKEN_SECONDS: u64 = 1_699_999_000;
const RSA_TEST_BITS: usize = 2048;
const RSA_TEST_SEED_BYTE: u8 = 7;
const UCAN_TEST_SEED_BYTE: u8 = 9;
const TAMPERED_SIGNATURE_BYTE: u8 = b'A';
const EXCESS_KEY: u32 = 1;

fn policy() -> OidcPolicy {
    parse_policy(POLICY_JSON).expect("generated policy must be admitted")
}

fn request() -> ExchangeRequest {
    ExchangeRequest {
        audience: "did:key:z6Mkmantle-service".to_owned(),
        project: "onix-stack".to_owned(),
        operation: MantleOperation::BuildSubmit,
        resource: "mantle://projects/onix-stack/builds/fixture".to_owned(),
        lifetime_seconds: REQUEST_LIFETIME_SECONDS,
    }
}

fn rsa_key() -> &'static rsa::RsaPrivateKey {
    static KEY: OnceLock<rsa::RsaPrivateKey> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut rng = ChaCha20Rng::from_seed([RSA_TEST_SEED_BYTE; 32]);
        rsa::RsaPrivateKey::new(&mut rng, RSA_TEST_BITS).expect("deterministic RSA fixture")
    })
}

fn public_jwk(key_id: &str) -> PublicJwk {
    let public = rsa_key().to_public_key();
    PublicJwk {
        key_id: key_id.to_owned(),
        modulus: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
        exponent: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
    }
}

fn claims() -> Value {
    serde_json::from_str(CLAIMS_JSON).expect("GitHub fixture claims")
}

fn token_with(claims: &Value, header: &Header) -> String {
    let der = rsa_key().to_pkcs1_der().expect("PKCS1 fixture key");
    encode(header, claims, &EncodingKey::from_rsa_der(der.as_bytes())).expect("fixture token")
}

fn valid_token_with_key(key_id: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(key_id.to_owned());
    token_with(&claims(), &header)
}

fn valid_token() -> String {
    valid_token_with_key("fixture-rsa")
}

fn issuer_admission(signer: &dyn TokenSigner) -> IssuerAdmission {
    let declaration = IssuerKeyDeclaration::new("production", "oidc-signing-key", "env://");
    let resolution = IssuerResolutionResult {
        schema: ISSUER_RESOLUTION_RESULT_SCHEMA.to_owned(),
        reason: IssuerResolutionReason::Loaded,
        allowed: true,
        issues: Vec::new(),
        non_claims: REQUIRED_ISSUER_NON_CLAIMS.map(str::to_owned).to_vec(),
    };
    let rotation = RotationWindow::new(
        signer_verification_digest(signer).expect("public signer digest"),
        Vec::new(),
        declaration.key_id.clone(),
    );
    IssuerAdmission {
        declaration,
        resolution,
        rotation,
    }
}

fn discovery_response(policy: &OidcPolicy) -> DocumentResponse {
    DocumentResponse {
        status: 200,
        content_type: "application/json".to_owned(),
        body: serde_json::to_vec(&serde_json::json!({
            "issuer": policy.issuer_url,
            "jwks_uri": policy.jwks_url,
            "authorization_endpoint": "https://example.invalid/not-used"
        }))
        .expect("discovery fixture"),
        fresh_seconds: CACHE_FRESH_SECONDS,
    }
}

fn jwks_response(key_id: &str, fresh_seconds: u64) -> DocumentResponse {
    let key = public_jwk(key_id);
    DocumentResponse {
        status: 200,
        content_type: "application/json".to_owned(),
        body: serde_json::to_vec(&serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "alg": "RS256",
                "kid": key.key_id,
                "key_ops": ["verify"],
                "n": key.modulus,
                "e": key.exponent
            }]
        }))
        .expect("JWKS fixture"),
        fresh_seconds,
    }
}

#[derive(Default)]
struct ScriptedDocuments {
    results: VecDeque<Result<DocumentResponse, DocumentFailure>>,
    calls: Vec<(DocumentKind, String, u64)>,
}

impl ScriptedDocuments {
    fn new(results: Vec<Result<DocumentResponse, DocumentFailure>>) -> Self {
        Self {
            results: results.into(),
            calls: Vec::new(),
        }
    }
}

impl DocumentPort for ScriptedDocuments {
    fn fetch(
        &mut self,
        kind: DocumentKind,
        url: &str,
        maximum_body_bytes: u64,
    ) -> Result<DocumentResponse, DocumentFailure> {
        self.calls.push((kind, url.to_owned(), maximum_body_bytes));
        self.results
            .pop_front()
            .unwrap_or(Err(DocumentFailure::Unavailable))
    }
}

#[derive(Default)]
struct MemoryReplay {
    committed: BTreeSet<String>,
    calls: usize,
    failure: Option<ReplayFailure>,
}

impl ReplayPort for MemoryReplay {
    fn commit(
        &mut self,
        key: &ReplayKey,
        _expires_at_seconds: u64,
    ) -> Result<ReplayCommit, ReplayFailure> {
        self.calls = self.calls.saturating_add(1);
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if self.committed.insert(key.as_str().to_owned()) {
            Ok(ReplayCommit::Fresh)
        } else {
            Ok(ReplayCommit::Duplicate)
        }
    }
}

#[test]
fn generated_policy_is_canonical_secret_free_and_admitted() {
    let admitted = policy();
    assert_eq!(admitted.issuer_id, "github-actions");
    assert_eq!(admitted.network.redirects, 0);
    assert_eq!(admitted.network.refreshes, 1);
    for marker in [
        "private_key",
        "client_secret",
        "oidc_token",
        "provider_credential",
    ] {
        assert!(!POLICY_JSON.contains(marker));
    }
}

#[test]
fn valid_github_token_produces_one_exact_attenuated_plan() {
    let policy = policy();
    let token = valid_token();
    let plan = verify_and_plan(
        token.as_str(),
        &public_jwk("fixture-rsa"),
        NOW_SECONDS,
        &policy,
        &request(),
    )
    .expect("valid token must plan");
    assert_eq!(plan.audience, policy.capability.audience);
    assert_eq!(plan.operation, MantleOperation::BuildSubmit);
    assert_eq!(
        plan.expires_at_seconds,
        NOW_SECONDS + REQUEST_LIFETIME_SECONDS
    );
    assert!(plan.expires_at_seconds <= TOKEN_EXPIRATION_SECONDS);
    assert!(plan.subject.identity.starts_with("blake3:"));
}

#[test]
fn malformed_algorithm_key_signature_type_and_claim_shapes_fail_closed() {
    let policy = policy();
    let key = public_jwk("fixture-rsa");
    let base_request = request();
    assert_eq!(
        verify_and_plan("not-a-token", &key, NOW_SECONDS, &policy, &base_request),
        Err(RejectReason::MalformedToken)
    );

    let mut hs_header = Header::new(Algorithm::HS256);
    hs_header.kid = Some("fixture-rsa".to_owned());
    let hs = encode(
        &hs_header,
        &claims(),
        &EncodingKey::from_secret(b"fixture-secret-not-authority"),
    )
    .expect("HS fixture");
    assert_eq!(
        verify_and_plan(hs.as_str(), &key, NOW_SECONDS, &policy, &base_request),
        Err(RejectReason::Algorithm)
    );

    let mut wrong_type = Header::new(Algorithm::RS256);
    wrong_type.kid = Some("fixture-rsa".to_owned());
    wrong_type.typ = Some("at+jwt".to_owned());
    assert_eq!(
        verify_and_plan(
            token_with(&claims(), &wrong_type).as_str(),
            &key,
            NOW_SECONDS,
            &policy,
            &base_request
        ),
        Err(RejectReason::TokenType)
    );

    let mut tampered = valid_token().into_bytes();
    let last = tampered.last_mut().expect("signature byte");
    *last = if *last == TAMPERED_SIGNATURE_BYTE {
        b'B'
    } else {
        TAMPERED_SIGNATURE_BYTE
    };
    let tampered = String::from_utf8(tampered).expect("ASCII compact token");
    assert_eq!(
        verify_and_plan(tampered.as_str(), &key, NOW_SECONDS, &policy, &base_request),
        Err(RejectReason::Signature)
    );

    let mut wrong_shape = claims();
    wrong_shape["aud"] = serde_json::json!(42);
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("fixture-rsa".to_owned());
    assert_eq!(
        verify_and_plan(
            token_with(&wrong_shape, &header).as_str(),
            &key,
            NOW_SECONDS,
            &policy,
            &base_request
        ),
        Err(RejectReason::MalformedToken)
    );
}

#[test]
fn issuer_audience_time_claim_and_capability_failures_are_distinct() {
    let policy = policy();
    let key = public_jwk("fixture-rsa");
    let base = claims();
    let cases = [
        (
            "iss",
            serde_json::json!("https://issuer.invalid"),
            RejectReason::Issuer,
        ),
        (
            "aud",
            serde_json::json!("other-audience"),
            RejectReason::Audience,
        ),
        (
            "exp",
            serde_json::json!(EXPIRED_TOKEN_SECONDS),
            RejectReason::Time,
        ),
        (
            "nbf",
            serde_json::json!(TOKEN_EXPIRATION_SECONDS),
            RejectReason::Time,
        ),
        (
            "iat",
            serde_json::json!(TOKEN_EXPIRATION_SECONDS),
            RejectReason::Time,
        ),
        (
            "repository",
            serde_json::json!("OnixResearch/other"),
            RejectReason::ClaimMismatch,
        ),
    ];
    for (field, value, expected) in cases {
        let mut mutated = base.clone();
        mutated[field] = value;
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("fixture-rsa".to_owned());
        let token = token_with(&mutated, &header);
        assert_eq!(
            verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &request()),
            Err(expected),
            "field {field}"
        );
    }

    let mut wrong_audience = request();
    wrong_audience.audience = "did:key:z6Mkother".to_owned();
    assert_eq!(
        verify_and_plan(
            valid_token().as_str(),
            &key,
            NOW_SECONDS,
            &policy,
            &wrong_audience
        ),
        Err(RejectReason::CapabilityAudience)
    );
    let mut wrong_resource = request();
    wrong_resource.resource = "mantle://projects/onix-stack-private/builds/fixture".to_owned();
    assert_eq!(
        verify_and_plan(
            valid_token().as_str(),
            &key,
            NOW_SECONDS,
            &policy,
            &wrong_resource
        ),
        Err(RejectReason::CapabilityResource)
    );
    let mut excessive_lifetime = request();
    excessive_lifetime.lifetime_seconds = policy.time.max_ucan_lifetime_seconds + 1;
    assert_eq!(
        verify_and_plan(
            valid_token().as_str(),
            &key,
            NOW_SECONDS,
            &policy,
            &excessive_lifetime
        ),
        Err(RejectReason::CapabilityLifetime)
    );
}

#[test]
fn decisions_are_deterministic_and_attenuation_is_monotonic() {
    let policy = policy();
    let token = valid_token();
    let key = public_jwk("fixture-rsa");
    let accepted = verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &request())
        .expect("accepted fixture");
    for _iteration in 0..16 {
        assert_eq!(
            verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &request()),
            Ok(accepted.clone())
        );
    }
    for operation in [
        MantleOperation::BuildSubmit,
        MantleOperation::InputUpload,
        MantleOperation::StoreRead,
        MantleOperation::StatusRead,
        MantleOperation::LogRead,
        MantleOperation::OwnedAttemptCancel,
    ] {
        let mut candidate = request();
        candidate.operation = operation;
        let decision = verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &candidate);
        assert_eq!(
            decision.is_ok(),
            policy.capability.operations.contains(&operation)
        );
    }
}

#[test]
fn bounded_discovery_jwks_cache_and_one_refresh_select_exact_key() {
    let policy = policy();
    let token = valid_token();
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let mut cache = JwksCache::default();
    let first = load_verification_key(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("first key load");
    assert_eq!(first.key_id, "fixture-rsa");
    assert_eq!(documents.calls.len(), 2);
    let second = load_verification_key(
        token.as_str(),
        NOW_SECONDS + 1,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("fresh cache");
    assert_eq!(second, first);
    assert_eq!(documents.calls.len(), 2);

    let rotated_token = valid_token_with_key("rotated-rsa");
    documents.results.extend([
        Ok(discovery_response(&policy)),
        Ok(jwks_response("rotated-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let rotated = load_verification_key(
        rotated_token.as_str(),
        NOW_SECONDS + 1,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("one bounded refresh");
    assert_eq!(rotated.key_id, "rotated-rsa");
    assert_eq!(documents.calls.len(), 4);
}

#[test]
fn network_tls_redirect_timeout_size_parse_key_and_stale_failures_are_typed() {
    let policy = policy();
    let token = valid_token();
    for failure in [
        DocumentFailure::Dns,
        DocumentFailure::Tls,
        DocumentFailure::Redirect,
        DocumentFailure::Timeout,
        DocumentFailure::Oversized,
        DocumentFailure::Unavailable,
    ] {
        let mut documents = ScriptedDocuments::new(vec![Err(failure)]);
        assert_eq!(
            load_verification_key(
                token.as_str(),
                NOW_SECONDS,
                &policy,
                &mut JwksCache::default(),
                &mut documents
            ),
            Err(KeyLoadFailure::Discovery(failure))
        );
    }

    let malformed = DocumentResponse {
        status: 200,
        content_type: "application/json".to_owned(),
        body: b"{".to_vec(),
        fresh_seconds: 0,
    };
    let mut documents = ScriptedDocuments::new(vec![Ok(malformed)]);
    assert_eq!(
        load_verification_key(
            token.as_str(),
            NOW_SECONDS,
            &policy,
            &mut JwksCache::default(),
            &mut documents
        ),
        Err(KeyLoadFailure::Discovery(DocumentFailure::Malformed))
    );

    let excessive_count = policy.network.jwks_keys + EXCESS_KEY;
    let excessive_keys: Vec<Value> = (0..excessive_count)
        .map(|index| {
            let key = public_jwk(format!("key-{index}").as_str());
            serde_json::json!({"kty":"RSA","use":"sig","alg":"RS256","kid":key.key_id,"n":key.modulus,"e":key.exponent})
        })
        .collect();
    let excessive_jwks = DocumentResponse {
        status: 200,
        content_type: "application/json".to_owned(),
        body: serde_json::to_vec(&serde_json::json!({"keys": excessive_keys}))
            .expect("excessive JWKS"),
        fresh_seconds: CACHE_FRESH_SECONDS,
    };
    let mut documents =
        ScriptedDocuments::new(vec![Ok(discovery_response(&policy)), Ok(excessive_jwks)]);
    assert_eq!(
        load_verification_key(
            token.as_str(),
            NOW_SECONDS,
            &policy,
            &mut JwksCache::default(),
            &mut documents
        ),
        Err(KeyLoadFailure::ExcessiveKeys)
    );

    let mut cache = JwksCache::default();
    let mut initial = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", 0)),
    ]);
    load_verification_key(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &mut cache,
        &mut initial,
    )
    .expect("initial zero-fresh key");
    let mut unavailable = ScriptedDocuments::new(vec![Err(DocumentFailure::Unavailable)]);
    assert_eq!(
        load_verification_key(
            token.as_str(),
            NOW_SECONDS + 1,
            &policy,
            &mut cache,
            &mut unavailable
        ),
        Err(KeyLoadFailure::Stale)
    );
}

#[test]
fn wrong_key_use_algorithm_and_unknown_key_fail_closed() {
    let policy = policy();
    let token = valid_token();
    for (public_use, algorithm) in [("enc", "RS256"), ("sig", "HS256")] {
        let key = public_jwk("fixture-rsa");
        let response = DocumentResponse {
            status: 200,
            content_type: "application/json".to_owned(),
            body: serde_json::to_vec(&serde_json::json!({
                "keys": [{
                    "kty": "RSA", "use": public_use, "alg": algorithm,
                    "kid": key.key_id, "n": key.modulus, "e": key.exponent
                }]
            }))
            .expect("JWK rejection fixture"),
            fresh_seconds: CACHE_FRESH_SECONDS,
        };
        let mut documents =
            ScriptedDocuments::new(vec![Ok(discovery_response(&policy)), Ok(response)]);
        assert_eq!(
            load_verification_key(
                token.as_str(),
                NOW_SECONDS,
                &policy,
                &mut JwksCache::default(),
                &mut documents
            ),
            Err(KeyLoadFailure::MalformedJwks)
        );
    }

    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("other-rsa", CACHE_FRESH_SECONDS)),
    ]);
    assert_eq!(
        load_verification_key(
            token.as_str(),
            NOW_SECONDS,
            &policy,
            &mut JwksCache::default(),
            &mut documents
        ),
        Err(KeyLoadFailure::UnknownKey)
    );
}

#[test]
#[allow(
    clippy::similar_names,
    reason = "issuer admission and issued UCAN are distinct security fixtures"
)]
fn exchange_issues_verifiable_ucan_once_and_fails_closed_on_replay_or_outage() {
    let policy = policy();
    let token = valid_token();
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let mut cache = JwksCache::default();
    let mut replay = MemoryReplay::default();
    let signer = Ed25519InMemorySigner::from_seed_bytes([UCAN_TEST_SEED_BYTE; 32]);
    let issuer = issuer_admission(&signer);
    let mut wrong_issuer = issuer.clone();
    wrong_issuer.rotation.active_verification_digest = "blake3:wrong-signer".to_owned();
    let mut unused_documents = ScriptedDocuments::default();
    let mut unused_replay = MemoryReplay::default();
    let binding_failure = exchange(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &request(),
        &mut JwksCache::default(),
        &mut unused_documents,
        &mut unused_replay,
        &wrong_issuer,
        &signer,
    )
    .expect_err("wrong signer binding must fail before effects");
    assert_eq!(
        binding_failure.class,
        ExchangeFailureClass::Issue(basalt::oidc_exchange::IssueFailure::IssuerBinding)
    );
    assert!(unused_documents.calls.is_empty());
    assert_eq!(unused_replay.calls, 0);

    let issued = exchange(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &request(),
        &mut cache,
        &mut documents,
        &mut replay,
        &issuer,
        &signer,
    )
    .expect("exchange must issue");
    assert_eq!(issued.receipt.outcome, ExchangeOutcome::Issued);
    assert_eq!(replay.calls, 1);
    let verification_key = signer.verification_key().expect("verification key");
    let context = VerificationContext::new(
        VerificationTime::from_unix_seconds(NOW_SECONDS + 1),
        KeyResolutionContext::new(vec![verification_key.clone()]),
        ProofCollection::empty(),
    );
    verify_compact_token(&issued.token, &context).expect("issued UCAN must verify");
    let expired_context = VerificationContext::new(
        VerificationTime::from_unix_seconds(TOKEN_EXPIRATION_SECONDS + 1),
        KeyResolutionContext::new(vec![verification_key]),
        ProofCollection::empty(),
    );
    assert!(verify_compact_token(&issued.token, &expired_context).is_err());

    let duplicate = exchange(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &request(),
        &mut cache,
        &mut documents,
        &mut replay,
        &issuer,
        &signer,
    )
    .expect_err("duplicate exchange must fail");
    assert_eq!(duplicate.class, ExchangeFailureClass::Duplicate);
    assert_eq!(replay.calls, 2);

    let mut unavailable_replay = MemoryReplay {
        failure: Some(ReplayFailure::Unavailable),
        ..MemoryReplay::default()
    };
    let failure = exchange(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &request(),
        &mut cache,
        &mut documents,
        &mut unavailable_replay,
        &issuer,
        &signer,
    )
    .expect_err("replay outage must fail");
    assert_eq!(
        failure.class,
        ExchangeFailureClass::Replay(ReplayFailure::Unavailable)
    );
}

// r[verify ucan_token_flow.oidc.holder_recipient]
#[test]
fn explicit_recipient_preserves_service_attenuation_and_one_time_exchange() {
    let policy = policy();
    let token = valid_token();
    let request = request();
    let signer = Ed25519InMemorySigner::from_seed_bytes([UCAN_TEST_SEED_BYTE; 32]);
    let holder = Ed25519InMemorySigner::from_seed_bytes([10; 32]);
    let recipient = ucan::AudienceDid::from(holder.issuer().expect("holder DID"));
    let issuer = issuer_admission(&signer);
    let mut cache = JwksCache::default();
    let mut replay = MemoryReplay::default();
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let issued = basalt::oidc_exchange::exchange_to_recipient(
        &token, NOW_SECONDS, &policy, &request, &mut cache, &mut documents,
        &mut replay, &issuer, &signer, &recipient,
    ).expect("exchange to holder");
    let context = VerificationContext::new(
        VerificationTime::from_unix_seconds(NOW_SECONDS),
        KeyResolutionContext::new(vec![signer.verification_key().expect("issuer key")]),
        ProofCollection::empty(),
    );
    let verified = verify_compact_token(&issued.token, &context).expect("signed holder grant");
    assert_eq!(verified.audience(), &recipient);
    let grants = verified.effective_delegation().capabilities().as_slice();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].resource, request.resource);
    assert_eq!(grants[0].ability, request.operation.ability());
    assert_eq!(verified.effective_delegation().time_bounds().expires_at_seconds(),
        NOW_SECONDS + REQUEST_LIFETIME_SECONDS);

    let other = ucan::AudienceDid::from(signer.issuer().expect("other holder DID"));
    let duplicate = basalt::oidc_exchange::exchange_to_recipient(
        &token, NOW_SECONDS, &policy, &request, &mut cache, &mut documents,
        &mut replay, &issuer, &signer, &other,
    ).expect_err("changing holder must not bypass replay");
    assert_eq!(duplicate.class, ExchangeFailureClass::Duplicate);

    let second = basalt::oidc_exchange::exchange_to_recipient(
        &token, NOW_SECONDS, &policy, &request, &mut cache, &mut documents,
        &mut MemoryReplay::default(), &issuer, &signer, &other,
    ).expect("independent replay fixture");
    assert_ne!(issued.receipt.granted_capability_identity, second.receipt.granted_capability_identity);

    let mut wrong_service = request.clone();
    wrong_service.audience = recipient.as_str().to_owned();
    let mut unused_replay = MemoryReplay::default();
    let failure = basalt::oidc_exchange::exchange_to_recipient(
        &token, NOW_SECONDS, &policy, &wrong_service, &mut cache, &mut documents,
        &mut unused_replay, &issuer, &signer, &recipient,
    ).expect_err("holder cannot replace service policy audience");
    assert_eq!(failure.class, ExchangeFailureClass::Verification(RejectReason::CapabilityAudience));
    assert_eq!(unused_replay.calls, 0);
    let receipt = serde_json::to_string(&issued.receipt).expect("redacted receipt");
    validate_receipt_redaction(&receipt, &[&token, issued.token.as_str()]).expect("no bearer evidence");
}

#[test]
fn events_and_results_require_explicit_policy_selection() {
    let token = valid_token();
    let key = public_jwk("fixture-rsa");
    for operation in [MantleOperation::EventRead, MantleOperation::ResultRead] {
        let mut request = request();
        request.operation = operation;
        assert_eq!(verify_and_plan(&token, &key, NOW_SECONDS, &policy(), &request),
            Err(RejectReason::CapabilityOperation));
        let mut document: Value = serde_json::from_str(POLICY_JSON).expect("policy fixture");
        document["capability"]["operations"] = serde_json::json!([operation]);
        let selected = parse_policy(&serde_json::to_string(&document).expect("selected policy"))
            .expect("explicit operation admitted");
        let plan = verify_and_plan(&token, &key, NOW_SECONDS, &selected, &request)
            .expect("selected read operation");
        assert_eq!(plan.operation, operation);
        request.operation = MantleOperation::BuildSubmit;
        assert_eq!(verify_and_plan(&token, &key, NOW_SECONDS, &selected, &request),
            Err(RejectReason::CapabilityOperation));
    }
}

#[test]
#[allow(
    clippy::similar_names,
    reason = "issuer admission and issued UCAN are distinct security fixtures"
)]
fn evidence_omits_oidc_ucan_private_and_provider_material() {
    let policy = policy();
    let token = valid_token();
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let mut replay = MemoryReplay::default();
    let signer = Ed25519InMemorySigner::from_seed_bytes([UCAN_TEST_SEED_BYTE; 32]);
    let issuer = issuer_admission(&signer);
    let issued = exchange(
        token.as_str(),
        NOW_SECONDS,
        &policy,
        &request(),
        &mut JwksCache::default(),
        &mut documents,
        &mut replay,
        &issuer,
        &signer,
    )
    .expect("exchange");
    let receipt = serde_json::to_string(&issued.receipt).expect("receipt JSON");
    let private_marker = "-----BEGIN PRIVATE KEY-----";
    let provider_marker = "provider-credential-fixture";
    validate_receipt_redaction(
        receipt.as_str(),
        &[
            token.as_str(),
            issued.token.as_str(),
            private_marker,
            provider_marker,
        ],
    )
    .expect("receipt must be redacted");
    assert!(!receipt.contains("token_digest"));
    assert!(!receipt.contains("oidc_token"));
}

#[test]
fn untrusted_header_only_selects_a_key_and_never_establishes_authority() {
    let token = valid_token();
    assert_eq!(
        inspect_token_key_id(token.as_str(), policy().network.token_bytes),
        Ok("fixture-rsa".to_owned())
    );
}
