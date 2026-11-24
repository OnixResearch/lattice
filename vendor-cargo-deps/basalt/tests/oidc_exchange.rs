// r[verify ucan_token_flow.oidc.versioned_policy]
// r[verify ucan_token_flow.oidc.pure_verification]
// r[verify ucan_token_flow.oidc.bounded_jwks]
// r[verify ucan_token_flow.oidc.attenuated_ucan]
// r[verify ucan_token_flow.oidc.replay_boundary]
// r[verify ucan_token_flow.oidc.redacted_evidence]

#![cfg(feature = "oidc-exchange")]

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::OnceLock;

use basalt::authority_core::BasaltPolicyRef;
use basalt::mantle;
use basalt::oidc_exchange::{
    DocumentFailure, DocumentKind, DocumentPort, DocumentResponse, ExchangeFailure,
    ExchangeFailureClass, ExchangeOutcome, ExchangeRequest, ExchangeResult, HolderDelivery,
    IssueFailure, IssuerAdmission, JwksCache, KeyLoadFailure, MantleOperation, OidcPolicy,
    PublicJwk, RejectReason, ReplayCommit, ReplayFailure, ReplayKey, ReplayPort, exchange,
    exchange_to_recipient, inspect_token_key_id, load_verification_key, parse_policy,
    signer_verification_digest, validate_receipt_redaction, verify_and_plan,
};
use basalt::secretspec_issuer::{
    ISSUER_RESOLUTION_RESULT_SCHEMA, IssuerKeyDeclaration, IssuerResolutionReason,
    IssuerResolutionResult, REQUIRED_ISSUER_NON_CLAIMS, RotationWindow,
};
use basalt::ucan::core::holder_invocation::HOLDER_INVOCATION_NONCE_BYTES;
use basalt::ucan::token::DenyUnknownCaveats;
use basalt::ucan_enforcement::{UcanEnforcementBoundaries, bind_enforcement_action};
use base64::Engine as _;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::traits::PublicKeyParts;
use serde_json::Value;
use ucan::{
    AudienceDid, CompactToken, Ed25519InMemorySigner, HolderInvocationLimits,
    HolderInvocationNonce, HolderInvocationReplayAdmission, HolderInvocationReplayCandidate,
    HolderInvocationReplayError, HolderInvocationSigningRequest, KeyResolutionContext,
    NoRevocations, ProofCollection, TokenSigner, VerificationContext, VerificationLimits,
    VerificationTime, sign_holder_invocation, verify_compact_token,
};

const POLICY_JSON: &str = include_str!("../contracts/oidc-exchange/generated/policy.json");
const CLAIMS_JSON: &str = include_str!("fixtures/oidc/github-claims.json");
const NOW_SECONDS: u64 = 1_700_000_060;
const TOKEN_EXPIRATION_SECONDS: u64 = 1_700_000_300;
const TOKEN_ISSUED_AT_SECONDS: u64 = 1_700_000_000;
/// `max(exp, iat + 600 s token age) + 30 s skew` for the fixture claims.
const FIXTURE_REPLAY_RESERVED_UNTIL_SECONDS: u64 = 1_700_000_630;
/// A token that outlives the 300 s UCAN lifetime: `exp = iat + 900`.
const LONG_TOKEN_EXPIRATION_SECONDS: u64 = 1_700_000_900;
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

/// A distinct OIDC token: same identity claims, its own `jti` and expiry.
fn token_with_jti(jti: &str, expiration_seconds: u64) -> String {
    let mut fields = claims();
    fields["jti"] = serde_json::json!(jti);
    fields["exp"] = serde_json::json!(expiration_seconds);
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("fixture-rsa".to_owned());
    token_with(&fields, &header)
}

/// One exchange service: policy, key cache, documents, replay store and signer.
struct Harness {
    policy: OidcPolicy,
    cache: JwksCache,
    documents: ScriptedDocuments,
    replay: MemoryReplay,
    signer: Ed25519InMemorySigner,
    issuer: IssuerAdmission,
}

impl Harness {
    /// Key documents stay fresh for the policy maximum, so one load serves all.
    fn new(policy: OidcPolicy) -> Self {
        let mut discovery = discovery_response(&policy);
        discovery.fresh_seconds = policy.cache.max_fresh_seconds;
        let documents = ScriptedDocuments::new(vec![
            Ok(discovery),
            Ok(jwks_response("fixture-rsa", policy.cache.max_fresh_seconds)),
        ]);
        let signer = Ed25519InMemorySigner::from_seed_bytes([UCAN_TEST_SEED_BYTE; 32]);
        let issuer = issuer_admission(&signer);
        Self {
            policy,
            cache: JwksCache::default(),
            documents,
            replay: MemoryReplay::default(),
            signer,
            issuer,
        }
    }

    fn exchange(
        &mut self,
        token: &str,
        now_seconds: u64,
        request: &ExchangeRequest,
    ) -> Result<ExchangeResult, ExchangeFailure> {
        exchange(
            token,
            now_seconds,
            &self.policy,
            request,
            &mut self.cache,
            &mut self.documents,
            &mut self.replay,
            &self.issuer,
            &self.signer,
        )
    }
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

/// In-memory replay store that frees a key once its reservation has passed.
#[derive(Default)]
struct MemoryReplay {
    committed: BTreeMap<String, u64>,
    calls: usize,
    failure: Option<ReplayFailure>,
    now_seconds: u64,
}

impl ReplayPort for MemoryReplay {
    fn commit(
        &mut self,
        key: &ReplayKey,
        expires_at_seconds: u64,
    ) -> Result<ReplayCommit, ReplayFailure> {
        self.calls = self.calls.saturating_add(1);
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        match self.committed.get(key.as_str()) {
            Some(reserved_until) if *reserved_until > self.now_seconds => {
                Ok(ReplayCommit::Duplicate)
            }
            _ => {
                self.committed
                    .insert(key.as_str().to_owned(), expires_at_seconds);
                Ok(ReplayCommit::Fresh)
            }
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
    assert_eq!(
        plan.replay_expires_at_seconds,
        FIXTURE_REPLAY_RESERVED_UNTIL_SECONDS
    );
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

// r[verify ucan_token_flow.oidc.replay_boundary]
#[test]
fn one_oidc_token_admits_exactly_one_exchange_until_it_can_no_longer_be_accepted() {
    let mut harness = Harness::new(policy());
    harness.replay.now_seconds = NOW_SECONDS;
    let token = valid_token();
    harness
        .exchange(&token, NOW_SECONDS, &request())
        .expect("first exchange issues");
    let mut other_operation = request();
    other_operation.operation = MantleOperation::StatusRead;
    let mut other_resource = request();
    other_resource.resource = "mantle://projects/onix-stack/builds/other".to_owned();
    for (label, candidate) in [("operation", other_operation), ("resource", other_resource)] {
        let duplicate = harness
            .exchange(&token, NOW_SECONDS, &candidate)
            .expect_err("a second exchange of one OIDC token must be denied");
        assert_eq!(duplicate.class, ExchangeFailureClass::Duplicate, "{label}");
    }
    let fresh = token_with_jti("fixture-exchange-2", TOKEN_EXPIRATION_SECONDS);
    harness
        .exchange(&fresh, NOW_SECONDS, &request())
        .expect("a fresh token is accepted");
    assert_eq!(
        harness
            .replay
            .committed
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![FIXTURE_REPLAY_RESERVED_UNTIL_SECONDS; 2]
    );

    // The old reservation ended with the UCAN (at most 300 s); this token outlives it.
    let long = token_with_jti("fixture-exchange-long", LONG_TOKEN_EXPIRATION_SECONDS);
    let mut full_lifetime = request();
    full_lifetime.lifetime_seconds = harness.policy.time.max_ucan_lifetime_seconds;
    let issued = harness
        .exchange(&long, NOW_SECONDS, &full_lifetime)
        .expect("long-lived token exchanges once");
    assert_eq!(issued.receipt.outcome, ExchangeOutcome::Issued);
    let later = NOW_SECONDS + harness.policy.time.max_ucan_lifetime_seconds + 40;
    assert!(
        verify_and_plan(
            &long,
            &public_jwk("fixture-rsa"),
            later,
            &harness.policy,
            &full_lifetime
        )
        .is_ok(),
        "the token must still be acceptable after the UCAN window"
    );
    harness.replay.now_seconds = later;
    let late = harness
        .exchange(&long, later, &full_lifetime)
        .expect_err("re-exchange after the UCAN window must be denied");
    assert_eq!(late.class, ExchangeFailureClass::Duplicate);
    let expected_reservation = LONG_TOKEN_EXPIRATION_SECONDS
        .max(TOKEN_ISSUED_AT_SECONDS + harness.policy.time.max_token_age_seconds)
        + harness.policy.time.clock_skew_seconds;
    assert_eq!(
        harness.replay.committed.values().copied().max(),
        Some(expected_reservation)
    );
}

const HOLDER_SEED_BYTE: u8 = 10;
/// A third key, distinct from the exchange signer and the first holder.
const SECOND_HOLDER_SEED_BYTE: u8 = 12;
const MANTLE_SERVICE: &str = "did:key:z6Mkmantle-service";
const MANTLE_GENERATION: u64 = 1;
const MANTLE_NONCE_BYTE: u8 = 19;

fn holder_signer(seed_byte: u8) -> Ed25519InMemorySigner {
    Ed25519InMemorySigner::from_seed_bytes([seed_byte; 32])
}

fn did_of(signer: &Ed25519InMemorySigner) -> AudienceDid {
    AudienceDid::from(signer.issuer().expect("signer DID"))
}

/// The shipped policy, admitting Mantle gateway resources for holder delivery.
fn holder_policy() -> OidcPolicy {
    let mut document: Value = serde_json::from_str(POLICY_JSON).expect("policy fixture");
    document["capability"]["resource_prefix"] = serde_json::json!("mantle://gateway/v1/");
    parse_policy(&serde_json::to_string(&document).expect("holder policy JSON"))
        .expect("holder policy admitted")
}

fn mantle_policy_ref() -> BasaltPolicyRef {
    BasaltPolicyRef::new(
        format!(
            "blake3:{}",
            blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()
        ),
        basalt::SUPPORTED_SCHEMA_VERSION,
        "mantle-build-submit",
    )
}

fn mantle_request(subject: &AudienceDid) -> mantle::Request {
    mantle::Request {
        operation: mantle::Operation::BuildSubmit,
        subject: subject.as_str().to_owned(),
        account: "account-a".to_owned(),
        project: "onix-stack".to_owned(),
        endpoint: "private-gateway".to_owned(),
        audience: MANTLE_SERVICE.to_owned(),
        generation: MANTLE_GENERATION,
        request_digest: [41; 32],
        target: "/jobs/approved".to_owned(),
    }
}

/// The exchange request for exactly the canonical resource of `mantle`.
fn holder_exchange_request(
    mantle: &mantle::Request,
    policy_ref: &BasaltPolicyRef,
) -> ExchangeRequest {
    ExchangeRequest {
        audience: MANTLE_SERVICE.to_owned(),
        project: "onix-stack".to_owned(),
        operation: MantleOperation::BuildSubmit,
        resource: mantle::canonical_enforcement(mantle, policy_ref)
            .expect("canonical fixture")
            .resource,
        lifetime_seconds: REQUEST_LIFETIME_SECONDS,
    }
}

impl Harness {
    fn exchange_to(
        &mut self,
        token: &str,
        request: &ExchangeRequest,
        holder: &HolderDelivery<'_>,
    ) -> Result<ExchangeResult, ExchangeFailure> {
        exchange_to_recipient(
            token,
            NOW_SECONDS,
            &self.policy,
            request,
            &mut self.cache,
            &mut self.documents,
            &mut self.replay,
            &self.issuer,
            &self.signer,
            holder,
        )
    }

    fn verify(&self, token: &CompactToken) -> ucan::VerifiedToken {
        let context = VerificationContext::new(
            VerificationTime::from_unix_seconds(NOW_SECONDS),
            KeyResolutionContext::new(vec![self.signer.verification_key().expect("issuer key")]),
            ProofCollection::empty(),
        );
        verify_compact_token(token, &context).expect("signed holder grant")
    }
}

// r[verify ucan_token_flow.oidc.holder_recipient]
#[test]
fn holder_delivery_issues_exactly_the_canonical_grant_to_the_holder_once() {
    let token = valid_token();
    let policy_ref = mantle_policy_ref();
    let mut harness = Harness::new(holder_policy());
    let first = did_of(&holder_signer(HOLDER_SEED_BYTE));
    let first_mantle = mantle_request(&first);
    let first_request = holder_exchange_request(&first_mantle, &policy_ref);
    let first_holder = HolderDelivery {
        recipient: &first,
        request: &first_mantle,
        policy_ref: &policy_ref,
    };
    let issued = harness
        .exchange_to(&token, &first_request, &first_holder)
        .expect("exchange to holder");
    let verified = harness.verify(&issued.token);
    assert_eq!(verified.audience(), &first);
    let grants = verified.effective_delegation().capabilities().as_slice();
    assert_eq!(grants.len(), 1);
    assert_eq!(grants[0].resource, first_request.resource);
    assert_eq!(grants[0].ability, "mantle/build/submit");
    assert_eq!(
        verified
            .effective_delegation()
            .time_bounds()
            .expires_at_seconds(),
        NOW_SECONDS + REQUEST_LIFETIME_SECONDS
    );

    let second_signer = holder_signer(SECOND_HOLDER_SEED_BYTE);
    let second = did_of(&second_signer);
    assert_ne!(
        second.as_str(),
        harness.signer.issuer().expect("DID").as_str()
    );
    let second_mantle = mantle_request(&second);
    let second_request = holder_exchange_request(&second_mantle, &policy_ref);
    let second_holder = HolderDelivery {
        recipient: &second,
        request: &second_mantle,
        policy_ref: &policy_ref,
    };
    let duplicate = harness
        .exchange_to(&token, &second_request, &second_holder)
        .expect_err("changing holder must not bypass replay");
    assert_eq!(duplicate.class, ExchangeFailureClass::Duplicate);

    let mut independent = Harness::new(holder_policy());
    let delivered = independent
        .exchange_to(&token, &second_request, &second_holder)
        .expect("independent replay store");
    let verified = independent.verify(&delivered.token);
    assert_eq!(verified.audience(), &second);
    assert_eq!(
        verified.effective_delegation().capabilities().as_slice()[0].resource,
        second_request.resource
    );
    assert_ne!(
        issued.receipt.granted_capability_identity,
        delivered.receipt.granted_capability_identity
    );
    let receipt = serde_json::to_string(&issued.receipt).expect("redacted receipt");
    validate_receipt_redaction(&receipt, &[&token, issued.token.as_str()])
        .expect("no bearer evidence");
}

#[test]
fn holder_delivery_refuses_unbound_requests_before_key_load_and_replay() {
    let token = valid_token();
    let policy_ref = mantle_policy_ref();
    let mut harness = Harness::new(holder_policy());
    let holder = did_of(&holder_signer(HOLDER_SEED_BYTE));
    let other_holder = did_of(&holder_signer(SECOND_HOLDER_SEED_BYTE));
    let exact = mantle_request(&holder);
    let exact_request = holder_exchange_request(&exact, &policy_ref);

    let mut swapped_project = mantle_request(&holder);
    swapped_project.project = "other-project".to_owned();
    let mut swapped_audience = mantle_request(&holder);
    swapped_audience.audience = "did:key:z6Mkother-service".to_owned();
    let swapped_subject = mantle_request(&other_holder);
    let mut bare_prefix = exact_request.clone();
    bare_prefix.resource = "mantle://gateway/v1/".to_owned();
    let mut truncated = exact_request.clone();
    let generation_end = truncated.resource.rfind('/').expect("digest separator");
    truncated.resource.truncate(generation_end);
    assert!(
        truncated.resource.ends_with("/1"),
        "cut after the generation"
    );
    let mut other_operation = exact_request.clone();
    other_operation.operation = MantleOperation::StatusRead;
    let mut holder_as_service = exact_request.clone();
    holder_as_service.audience = holder.as_str().to_owned();

    let cases = [
        (
            "swapped project",
            holder_exchange_request(&swapped_project, &policy_ref),
            &swapped_project,
            RejectReason::CapabilityProject,
        ),
        (
            "swapped audience",
            holder_exchange_request(&swapped_audience, &policy_ref),
            &swapped_audience,
            RejectReason::CapabilityAudience,
        ),
        (
            "swapped subject",
            holder_exchange_request(&swapped_subject, &policy_ref),
            &swapped_subject,
            RejectReason::CapabilityHolder,
        ),
        (
            "bare prefix",
            bare_prefix,
            &exact,
            RejectReason::CapabilityResource,
        ),
        (
            "truncated generation",
            truncated,
            &exact,
            RejectReason::CapabilityResource,
        ),
        (
            "other operation",
            other_operation,
            &exact,
            RejectReason::CapabilityOperation,
        ),
        (
            "holder as service",
            holder_as_service,
            &exact,
            RejectReason::CapabilityAudience,
        ),
    ];
    for (label, request, mantle, expected) in cases {
        let delivery = HolderDelivery {
            recipient: &holder,
            request: mantle,
            policy_ref: &policy_ref,
        };
        let failure = harness
            .exchange_to(&token, &request, &delivery)
            .expect_err("unbound holder request must be refused");
        assert_eq!(
            failure.class,
            ExchangeFailureClass::Verification(expected),
            "{label}"
        );
        assert_eq!(failure.receipt.outcome, ExchangeOutcome::Denied, "{label}");
    }
    assert!(harness.documents.calls.is_empty(), "no key load");
    assert_eq!(harness.replay.calls, 0, "no replay commit");

    let delivery = HolderDelivery {
        recipient: &holder,
        request: &exact,
        policy_ref: &policy_ref,
    };
    harness
        .exchange_to(&token, &exact_request, &delivery)
        .expect("the exact request still exchanges once");
    assert_eq!(harness.replay.calls, 1);
}

#[test]
fn holder_recipient_equal_to_the_issuer_is_refused_before_key_load_and_replay() {
    let token = valid_token();
    let policy_ref = mantle_policy_ref();
    let mut harness = Harness::new(holder_policy());
    let issuer_did = did_of(&harness.signer);
    let self_issued = mantle_request(&issuer_did);
    let delivery = HolderDelivery {
        recipient: &issuer_did,
        request: &self_issued,
        policy_ref: &policy_ref,
    };
    let failure = harness
        .exchange_to(
            &token,
            &holder_exchange_request(&self_issued, &policy_ref),
            &delivery,
        )
        .expect_err("a recipient equal to the issuer is refused");
    assert_eq!(
        failure.class,
        ExchangeFailureClass::Issue(IssueFailure::Audience)
    );
    assert!(harness.documents.calls.is_empty(), "no key load");
    assert_eq!(harness.replay.calls, 0, "no replay commit");
}

#[test]
fn holder_failure_receipts_bind_the_recipient() {
    let token = valid_token();
    let policy_ref = mantle_policy_ref();
    let mut harness = Harness::new(holder_policy());
    let first = did_of(&holder_signer(HOLDER_SEED_BYTE));
    let second = did_of(&holder_signer(SECOND_HOLDER_SEED_BYTE));
    let mut receipts = Vec::new();
    for recipient in [&first, &second] {
        let mantle = mantle_request(recipient);
        let mut request = holder_exchange_request(&mantle, &policy_ref);
        request.audience = "did:key:z6Mkother-service".to_owned();
        let delivery = HolderDelivery {
            recipient,
            request: &mantle,
            policy_ref: &policy_ref,
        };
        let failure = harness
            .exchange_to(&token, &request, &delivery)
            .expect_err("wrong service audience");
        receipts.push(failure.receipt);
    }
    let mut legacy_request = holder_exchange_request(&mantle_request(&first), &policy_ref);
    legacy_request.audience = "did:key:z6Mkother-service".to_owned();
    let legacy = harness
        .exchange(&token, NOW_SECONDS, &legacy_request)
        .expect_err("legacy wrong service audience");
    assert_eq!(
        legacy.class,
        ExchangeFailureClass::Verification(RejectReason::CapabilityAudience)
    );
    receipts.push(legacy.receipt);
    let identities: BTreeSet<&str> = receipts
        .iter()
        .map(|receipt| receipt.requested_capability_identity.as_str())
        .collect();
    assert_eq!(
        identities.len(),
        3,
        "each recipient binds a distinct identity"
    );
    for receipt in &receipts {
        assert_eq!(receipt.outcome, ExchangeOutcome::Denied);
        assert!(receipt.granted_capability_identity.is_none());
    }
}

#[test]
fn legacy_exchange_still_issues_to_the_policy_service_audience() {
    let token = valid_token();
    let mut harness = Harness::new(policy());
    let issued = harness
        .exchange(&token, NOW_SECONDS, &request())
        .expect("legacy exchange");
    let verified = harness.verify(&issued.token);
    assert_eq!(verified.audience().as_str(), MANTLE_SERVICE);
    assert_eq!(
        verified.audience().as_str(),
        harness.policy.capability.audience
    );
}

/// Final holder replay admission that counts and admits every candidate.
struct CountingAdmission(Cell<u32>);

impl HolderInvocationReplayAdmission for CountingAdmission {
    fn admit_holder_invocation(
        &self,
        _candidate: &HolderInvocationReplayCandidate<'_>,
    ) -> Result<(), HolderInvocationReplayError> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
}

#[test]
fn holder_grant_from_the_exchange_is_admitted_by_the_native_mantle_verifier() {
    let token = valid_token();
    let policy_ref = mantle_policy_ref();
    let mut harness = Harness::new(holder_policy());
    let holder = holder_signer(HOLDER_SEED_BYTE);
    let recipient = did_of(&holder);
    let mantle = mantle_request(&recipient);
    let delivery = HolderDelivery {
        recipient: &recipient,
        request: &mantle,
        policy_ref: &policy_ref,
    };
    let issued = harness
        .exchange_to(
            &token,
            &holder_exchange_request(&mantle, &policy_ref),
            &delivery,
        )
        .expect("exchange to holder");

    let policy = basalt::default_policy().expect("default Basalt policy");
    let roots = [harness.signer.issuer().expect("DID").as_str().to_owned()];
    let verifier = mantle::Verifier {
        policy: &policy,
        policy_ref: &policy_ref,
        generation: MANTLE_GENERATION,
        endpoint: "private-gateway",
        audience: MANTLE_SERVICE,
        trusted_roots: &roots,
    };
    let exact = verifier.enforcement(&mantle).expect("receiver enforcement");
    let binding =
        bind_enforcement_action(mantle.operation.action(), &exact).expect("operation binding");
    let nonce =
        HolderInvocationNonce::from_bytes([MANTLE_NONCE_BYTE; HOLDER_INVOCATION_NONCE_BYTES]);
    let signature = sign_holder_invocation(
        &HolderInvocationSigningRequest::new(
            issued.token.as_str(),
            &recipient,
            exact.resource(),
            exact.ability(),
            binding,
            nonce,
        ),
        &holder,
        VerificationLimits::default(),
        HolderInvocationLimits::default(),
    )
    .expect("holder signs the canonical request");
    let keys = KeyResolutionContext::new(vec![
        harness.signer.verification_key().expect("issuer key"),
        holder.verification_key().expect("holder key"),
    ]);
    let proofs = ProofCollection::empty();
    let admission = CountingAdmission(Cell::new(0));
    let boundaries = UcanEnforcementBoundaries {
        verification_time: VerificationTime::from_unix_seconds(NOW_SECONDS + 1),
        key_resolver: &keys,
        trusted_roots: &roots,
        proof_store: &proofs,
        revocations: &NoRevocations,
        caveat_policies: &DenyUnknownCaveats,
        replay: &admission,
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };
    let invocation = mantle::Invocation {
        compact_token: issued.token.as_str(),
        nonce,
        holder_signature: &signature,
    };
    let session = verifier
        .verify(mantle.clone(), &invocation, &boundaries)
        .expect("native Mantle verifier admits the exchanged holder grant");
    verifier
        .revalidate(&session, &boundaries)
        .expect("revalidation");
    session
        .require(mantle.operation, &mantle.target, mantle.request_digest)
        .expect("exact operation");
    assert_eq!(admission.0.get(), 1);

    let mut other_project = mantle;
    other_project.project = "other-project".to_owned();
    assert!(
        verifier
            .verify(other_project, &invocation, &boundaries)
            .is_err(),
        "the grant covers only the admitted project"
    );
    assert_eq!(admission.0.get(), 1);
}

#[test]
fn events_and_results_require_explicit_policy_selection() {
    let token = valid_token();
    let key = public_jwk("fixture-rsa");
    for operation in [MantleOperation::EventRead, MantleOperation::ResultRead] {
        let mut request = request();
        request.operation = operation;
        assert_eq!(
            verify_and_plan(&token, &key, NOW_SECONDS, &policy(), &request),
            Err(RejectReason::CapabilityOperation)
        );
        let mut document: Value = serde_json::from_str(POLICY_JSON).expect("policy fixture");
        document["capability"]["operations"] = serde_json::json!([operation]);
        let selected = parse_policy(&serde_json::to_string(&document).expect("selected policy"))
            .expect("explicit operation admitted");
        let plan = verify_and_plan(&token, &key, NOW_SECONDS, &selected, &request)
            .expect("selected read operation");
        assert_eq!(plan.operation, operation);
        request.operation = MantleOperation::BuildSubmit;
        assert_eq!(
            verify_and_plan(&token, &key, NOW_SECONDS, &selected, &request),
            Err(RejectReason::CapabilityOperation)
        );
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

#[test]
fn arbitrary_branch_workflow_repo_id_and_environment_claims_fail_closed() {
    let policy = policy();
    let key = public_jwk("fixture-rsa");
    let base = claims();
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("fixture-rsa".to_owned());

    let cases = [
        (
            "feature branch ref",
            "ref",
            serde_json::json!("refs/heads/feature"),
            RejectReason::ClaimMismatch,
        ),
        (
            "different workflow on main",
            "job_workflow_ref",
            serde_json::json!("OnixResearch/mantle/.github/workflows/evil.yml@refs/heads/main"),
            RejectReason::ClaimMismatch,
        ),
        (
            "prefix confusion on workflow ref",
            "job_workflow_ref",
            serde_json::json!(
                "OnixResearch/mantle/.github/workflows/build.yml@refs/heads/main-evil"
            ),
            RejectReason::ClaimMismatch,
        ),
        (
            "re-registered repository_id",
            "repository_id",
            serde_json::json!("9999999999"),
            RejectReason::ClaimMismatch,
        ),
        (
            "re-registered repository_owner_id",
            "repository_owner_id",
            serde_json::json!("8888888888"),
            RejectReason::ClaimMismatch,
        ),
        (
            "non-numeric repository_id",
            "repository_id",
            serde_json::json!("abc123"),
            RejectReason::ClaimType,
        ),
        (
            "wrong environment",
            "environment",
            serde_json::json!("staging"),
            RejectReason::ClaimMismatch,
        ),
    ];

    for (label, field, value, expected) in cases {
        let mut mutated = base.clone();
        mutated[field] = value;
        let token = token_with(&mutated, &header);
        assert_eq!(
            verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &request()),
            Err(expected),
            "{label}"
        );
    }

    // Missing environment claim must fail closed as malformed token.
    let mut missing_env = base;
    missing_env
        .as_object_mut()
        .expect("claims object")
        .remove("environment");
    let token = token_with(&missing_env, &header);
    assert_eq!(
        verify_and_plan(token.as_str(), &key, NOW_SECONDS, &policy, &request()),
        Err(RejectReason::MalformedToken)
    );
}

#[test]
fn oidc_policy_parser_rejects_broad_prefixes_and_non_numeric_ids() {
    let base: Value = serde_json::from_str(POLICY_JSON).expect("generated policy JSON");

    let mut wildcard_ref = base.clone();
    wildcard_ref["claims"]["allowed_refs"] = serde_json::json!(["refs/heads/"]);
    assert!(parse_policy(&wildcard_ref.to_string()).is_err());

    let mut non_numeric_repo_id = base.clone();
    non_numeric_repo_id["claims"]["repository_id"] = serde_json::json!("not-numeric");
    assert!(parse_policy(&non_numeric_repo_id.to_string()).is_err());

    let mut unbound_workflow = base.clone();
    unbound_workflow["claims"]["job_workflow_refs"] =
        serde_json::json!(["OnixResearch/mantle/.github/workflows/build.yml@refs/heads/dev"]);
    assert!(parse_policy(&unbound_workflow.to_string()).is_err());

    let mut prefix_only_workflow = base.clone();
    prefix_only_workflow["claims"]["job_workflow_refs"] =
        serde_json::json!(["OnixResearch/mantle/.github/workflows/"]);
    assert!(parse_policy(&prefix_only_workflow.to_string()).is_err());

    let mut empty_env = base;
    empty_env["claims"]["environment"] = serde_json::json!("");
    assert!(parse_policy(&empty_env.to_string()).is_err());
}

#[test]
fn unknown_kid_storm_is_rate_limited_and_negatively_cached() {
    let policy = policy();
    let known = valid_token();
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
        Ok(discovery_response(&policy)),
        Ok(jwks_response("fixture-rsa", CACHE_FRESH_SECONDS)),
        Ok(discovery_response(&policy)),
        Ok(jwks_response("rotated-rsa", CACHE_FRESH_SECONDS)),
    ]);
    let mut cache = JwksCache::default();

    // Initial cache fill.
    load_verification_key(
        known.as_str(),
        NOW_SECONDS,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("initial key load");
    assert_eq!(documents.calls.len(), 2);

    // First unknown kid triggers one bounded refresh and fails closed.
    let unknown_one = valid_token_with_key("attacker-kid-1");
    assert_eq!(
        load_verification_key(
            unknown_one.as_str(),
            NOW_SECONDS + 1,
            &policy,
            &mut cache,
            &mut documents,
        ),
        Err(KeyLoadFailure::UnknownKey)
    );
    assert_eq!(documents.calls.len(), 4);

    // Repeated same unknown kid and distinct random unknown kids within cooldown
    // must fail closed with zero additional outbound fetches.
    for index in 1..=20_u64 {
        let probe = valid_token_with_key(&format!("attacker-kid-{index}"));
        assert_eq!(
            load_verification_key(
                probe.as_str(),
                NOW_SECONDS + 1 + index,
                &policy,
                &mut cache,
                &mut documents,
            ),
            Err(KeyLoadFailure::UnknownKey)
        );
    }
    assert_eq!(documents.calls.len(), 4);

    // Known cached key still verifies with zero fetches during cooldown.
    let cached = load_verification_key(
        known.as_str(),
        NOW_SECONDS + 10,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("cached key stays usable");
    assert_eq!(cached.key_id, "fixture-rsa");
    assert_eq!(documents.calls.len(), 4);

    // After cooldown expires, legitimate key rotation refreshes once and succeeds.
    let rotated = valid_token_with_key("rotated-rsa");
    let loaded = load_verification_key(
        rotated.as_str(),
        NOW_SECONDS + 31,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("legitimate rotation refreshes after cooldown");
    assert_eq!(loaded.key_id, "rotated-rsa");
    assert_eq!(documents.calls.len(), 6);
}

#[test]
fn unsupported_jwk_entries_are_skipped_without_failing_valid_rs256_keys() {
    let policy = policy();
    let key = public_jwk("fixture-rsa");
    let mixed_jwks = DocumentResponse {
        status: 200,
        content_type: "application/json".to_owned(),
        body: serde_json::to_vec(&serde_json::json!({
            "keys": [
                {
                    "kty": "EC",
                    "use": "sig",
                    "alg": "ES256",
                    "kid": "ec-p256-key",
                    "crv": "P-256",
                    "x": "f83OJ3D2xF1Bg8vub9tLe1gHMzV76e8Tus9uPHvRVEU",
                    "y": "x_FEzRu9m36HLN_tue659LNpXW6pCyStikYjKIWI5a0"
                },
                {
                    "kty": "OKP",
                    "use": "sig",
                    "alg": "EdDSA",
                    "kid": "okp-ed25519-key",
                    "crv": "Ed25519",
                    "x": "11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"
                },
                {
                    "kty": "RSA",
                    "use": "enc",
                    "alg": "RS256",
                    "kid": "rsa-enc-key",
                    "n": key.modulus,
                    "e": key.exponent
                },
                {
                    "kty": "RSA",
                    "use": "sig",
                    "alg": "RS256",
                    "kid": key.key_id,
                    "key_ops": ["verify"],
                    "n": key.modulus,
                    "e": key.exponent
                }
            ]
        }))
        .expect("mixed JWKS fixture"),
        fresh_seconds: CACHE_FRESH_SECONDS,
    };
    let mut documents = ScriptedDocuments::new(vec![
        Ok(discovery_response(&policy)),
        Ok(mixed_jwks.clone()),
        Ok(discovery_response(&policy)),
        Ok(mixed_jwks),
    ]);
    let mut cache = JwksCache::default();

    let admitted = load_verification_key(
        valid_token().as_str(),
        NOW_SECONDS,
        &policy,
        &mut cache,
        &mut documents,
    )
    .expect("valid RS256 key must be admitted alongside unsupported key types");
    assert_eq!(admitted.key_id, "fixture-rsa");
    assert_eq!(documents.calls.len(), 2);

    // Skipped unsupported keys must never be admitted, and at most one bounded
    // refresh runs across all three unknown-kid probes within cooldown.
    for skipped_kid in ["ec-p256-key", "okp-ed25519-key", "rsa-enc-key"] {
        let token = valid_token_with_key(skipped_kid);
        assert_eq!(
            load_verification_key(
                token.as_str(),
                NOW_SECONDS + 1,
                &policy,
                &mut cache,
                &mut documents,
            ),
            Err(KeyLoadFailure::UnknownKey)
        );
    }
    assert_eq!(documents.calls.len(), 4);
}
