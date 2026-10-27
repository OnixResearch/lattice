//! Deterministic boundary conformance, not live GitHub or Basalt evidence.
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::atomic::AtomicBool;

use lattice::credentials::CredentialDestination;
use lattice::credentials::CredentialProviderCapabilities;
use lattice::credentials::CredentialProviderMetadata;
use lattice::credentials::CredentialProviderRegistry;
use lattice::credentials::CredentialResolutionContext;
use lattice::credentials::CredentialSubject;
use lattice::credentials::MapCredentialProvider;
use lattice::credentials::SecretRef;
use lattice::github_ci::core::Annotation;
use lattice::github_ci::core::CheckProjection;
use lattice::github_ci::core::CheckStatus;
use lattice::github_ci::core::Conclusion;
use lattice::github_ci::core::Identity;
use lattice::github_ci::core::{self};
use lattice::github_ci::github::*;
use lattice::http::HttpBounds;
use lattice::http::HttpClient;
use lattice::http::HttpError;
use lattice::http::HttpRequestPlan;
use lattice::http::HttpResponse;
use parking_lot::Mutex;

struct Boundary {
    steps: Mutex<VecDeque<(&'static str, Result<HttpResponse, HttpError>)>>,
    methods: Mutex<Vec<String>>,
}
impl Boundary {
    fn new(steps: Vec<(&'static str, Result<HttpResponse, HttpError>)>) -> Self {
        Self {
            steps: Mutex::new(steps.into()),
            methods: Mutex::new(Vec::new()),
        }
    }
    fn methods(&self) -> Vec<String> {
        self.methods.lock().clone()
    }
}
impl HttpClient for Boundary {
    fn execute(&self, _: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        panic!("GitHub must never use the unbounded port")
    }
    fn execute_bounded(
        &self,
        request: &HttpRequestPlan,
        bounds: &HttpBounds,
        _: Option<&AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        let (method, response) = self.steps.lock().pop_front().expect("unexpected HTTP effect");
        assert_eq!(request.method, method);
        assert_eq!(request.timeout_secs, REQUEST_TIMEOUT_SECS);
        assert_eq!(bounds.connect_timeout_secs, DNS_CONNECT_TLS_TIMEOUT_SECS);
        assert_eq!(bounds.header_bytes, MAX_HEADER_BYTES);
        let url = url::Url::parse(&request.url).unwrap();
        assert_eq!(url.scheme(), "https");
        assert!(url.username().is_empty());
        if url.host_str() == Some(API_HOST) {
            assert_eq!(request.headers.get("X-GitHub-Api-Version").map(String::as_str), Some(API_VERSION));
            assert_eq!(request.headers.get("Accept").map(String::as_str), Some("application/vnd.github+json"));
            assert_eq!(bounds.response_body_bytes, MAX_API_BODY_BYTES);
            if let Some(body) = &request.body {
                let body: serde_json::Value = serde_json::from_str(body).unwrap();
                assert_eq!(body["external_id"], Identity([1; 32]).hex());
                if method == "POST" {
                    assert_eq!(body["head_sha"], "a".repeat(40));
                }
            } else {
                assert!(url.path().ends_with(&format!("/commits/{}/check-runs", "a".repeat(40))));
                assert!(url.query_pairs().any(|(k, v)| k == "filter" && v == "all"));
            }
        } else {
            assert_eq!(bounds.response_body_bytes, MAX_OIDC_BODY_BYTES);
            assert_eq!(url.query_pairs().filter(|(k, _)| k == "audience").count(), 1);
            assert!(url.query_pairs().any(|(k, v)| k == "audience" && v == "basalt://exchange/project"));
        }
        assert!(request.headers.get("Authorization").is_some_and(|v| v.starts_with("Bearer ")));
        self.methods.lock().push(method.into());
        response
    }
}
fn response(status: u16, body: impl Into<String>) -> Result<HttpResponse, HttpError> {
    Ok(HttpResponse {
        status,
        headers: HashMap::new(),
        body: body.into(),
    })
}
fn error<T>(result: Result<T, Error>) -> Error {
    match result {
        Ok(_) => panic!("expected fail-closed result"),
        Err(error) => error,
    }
}
fn hosts() -> BTreeSet<String> {
    [API_HOST.into()].into()
}
fn target() -> CheckRunTarget {
    CheckRunTarget {
        owner: "OnixResearch".into(),
        repository: "private-ci".into(),
        head_sha: "a".repeat(40),
        name: "bounded build".into(),
        external_id: Identity([1; 32]),
    }
}
fn projection(version: u64, terminal: bool) -> CheckProjection {
    CheckProjection {
        check_id: None,
        external_id: Identity([1; 32]),
        update_id: Identity([version as u8 + 2; 32]),
        state_version: version,
        status: if terminal {
            CheckStatus::Completed
        } else {
            CheckStatus::Queued
        },
        conclusion: terminal.then_some(Conclusion::Success),
        title: "Build observation".into(),
        summary: "Observed bounded result; not release eligibility".into(),
        text: String::new(),
        annotations: vec![],
        links: vec![],
    }
}
fn remote(id: u64, p: &CheckProjection) -> serde_json::Value {
    serde_json::json!({"id":id,"name":target().name,"head_sha":target().head_sha,"external_id":p.external_id.hex(),
        "status":if p.status == CheckStatus::Completed {"completed"} else {"queued"},
        "conclusion":if p.status == CheckStatus::Completed {Some("success")} else {None},
        "output":{"summary":format!("{}\n\n<!-- lattice-check-v1:{}:{} -->",p.summary,p.state_version,p.update_id.hex())}})
}
fn list(runs: Vec<serde_json::Value>) -> Result<HttpResponse, HttpError> {
    response(200, serde_json::json!({"total_count":runs.len(),"check_runs":runs}).to_string())
}
fn credentials() -> (CredentialProviderRegistry, CredentialResolutionContext) {
    let reference = SecretRef::new("fixture", "github", None);
    let mut registry = CredentialProviderRegistry::new();
    registry
        .insert_provider(
            CredentialProviderMetadata {
                id: "fixture".into(),
                display_name: "test-only".into(),
                capabilities: CredentialProviderCapabilities::read_only_without_fields(),
            },
            Box::new(MapCredentialProvider::new([(reference.identity(), "fixture-api-bearer".into())].into())),
        )
        .unwrap();
    let context = CredentialResolutionContext {
        subject: CredentialSubject::Node {
            workflow_id: "workflow".into(),
            node_id: "publish".into(),
        },
        binding: "Authorization".into(),
        reference,
        destination: CredentialDestination::HttpHeader,
        reason: "publish check".into(),
    };
    (registry, context)
}
fn policy() -> core::Policy {
    core::Policy {
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
    }
}

fn request_token() -> ActionsRequestToken {
    ActionsRequestToken::from_documented_value("fixture-request-token".into()).unwrap()
}

#[test]
fn oidc_acquires_only_explicit_allowed_endpoint_and_audience() {
    let http = Boundary::new(vec![("GET", response(200, r#"{"value":"fixture.oidc.signature"}"#))]);
    let client = ActionsOidcClient::new(&http, ["identity.actions.githubusercontent.com".into()].into()).unwrap();
    let token = client
        .acquire(
            "https://identity.actions.githubusercontent.com/id?api-version=2.0",
            &request_token(),
            "basalt://exchange/project",
        )
        .unwrap();
    assert_eq!(token.expose_for_exchange(), "fixture.oidc.signature");
}

#[test]
fn oidc_rejects_unapproved_host_tls_userinfo_query_credentials_and_duplicate_audience() {
    let http = Boundary::new(vec![]);
    let client = ActionsOidcClient::new(&http, ["identity.actions.githubusercontent.com".into()].into()).unwrap();
    for url in [
        "http://identity.actions.githubusercontent.com/id",
        "https://attacker.example/id",
        "https://bearer@identity.actions.githubusercontent.com/id",
        "https://identity.actions.githubusercontent.com/id?token=secret",
        "https://identity.actions.githubusercontent.com/id?audience=attacker",
        "https://identity.actions.githubusercontent.com:444/id",
        "https://identity.actions.githubusercontent.com/id#fragment",
    ] {
        assert_eq!(
            error(client.acquire(url, &request_token(), "basalt://exchange/project")).kind,
            ErrorKind::UnsafeUrl
        );
    }
    assert!(http.methods().is_empty());
}

#[test]
fn oidc_rejects_redirect_body_overflow_malformed_missing_permission_and_transport_secret_errors() {
    let cases = vec![
        response(302, "fixture-request-token"),
        response(200, "x".repeat(MAX_OIDC_BODY_BYTES + 1)),
        response(200, r#"{"value":3}"#),
        Err(HttpError::Transport("fixture-request-token".into())),
    ];
    for (case, expected) in cases.into_iter().zip([
        ErrorKind::Redirect,
        ErrorKind::Bounds,
        ErrorKind::Malformed,
        ErrorKind::Transient,
    ]) {
        let http = Boundary::new(vec![("GET", case)]);
        let client = ActionsOidcClient::new(&http, ["identity.actions.githubusercontent.com".into()].into()).unwrap();
        let error = error(client.acquire(
            "https://identity.actions.githubusercontent.com/id",
            &request_token(),
            "basalt://exchange/project",
        ));
        assert_eq!(error.kind, expected);
        assert!(!format!("{error:?} {error}").contains("fixture-request-token"));
    }
    assert_eq!(
        error(ActionsRequestToken::from_documented_value(String::new())).kind,
        ErrorKind::MissingIdentityPermission
    );
}

#[test]
fn create_reserves_then_acknowledges_public_binding_and_terminal_duplicate_is_noop() {
    let p = projection(1, true);
    let http = Boundary::new(vec![
        ("GET", list(vec![])),
        ("POST", response(201, remote(42, &p).to_string())),
    ]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let plan = prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap();
    assert_eq!(plan.reserved_binding().pending.as_ref().unwrap().version, 1);
    let binding = client.execute(plan, &p, &context).unwrap();
    assert_eq!(binding.check_id, Some(42));
    assert!(binding.pending.is_none());
    let durable = serde_json::to_string(&binding).unwrap();
    assert!(!durable.contains("fixture-api-bearer"));
    let duplicate = client.execute(prepare_check(&binding, &p).unwrap(), &p, &context).unwrap();
    assert_eq!(duplicate, binding);
    assert_eq!(http.methods(), ["GET", "POST"]);
}

#[test]
fn lost_create_response_recovers_exact_external_id_without_second_post() {
    let p = projection(1, true);
    let (registry, context) = credentials();
    let http = Boundary::new(vec![
        ("GET", list(vec![])),
        ("POST", Err(HttpError::Timeout("fixture-api-bearer".into()))),
        ("GET", list(vec![remote(42, &p)])),
    ]);
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let plan = prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap();
    let reservation = plan.reserved_binding().clone();
    let uncertain = error(client.execute(plan, &p, &context));
    assert!(!uncertain.remote_rejected);
    assert!(reservation.after_rejection(&uncertain).is_err());
    let recovered = client.execute(prepare_check(&reservation, &p).unwrap(), &p, &context).unwrap();
    assert_eq!(recovered.check_id, Some(42));
    assert_eq!(http.methods(), ["GET", "POST", "GET"]);
}

#[test]
fn missing_uncertain_create_never_authorizes_recreate() {
    let p = projection(1, false);
    let reservation = prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap().reserved_binding().clone();
    let http = Boundary::new(vec![("GET", list(vec![]))]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    assert_eq!(
        error(client.execute(prepare_check(&reservation, &p).unwrap(), &p, &context)).kind,
        ErrorKind::ReconciliationRequired
    );
    assert_eq!(http.methods(), ["GET"]);
}

#[test]
fn duplicate_external_ids_and_wrong_commit_fail_closed() {
    let p = projection(1, false);
    let mut wrong = remote(42, &p);
    wrong["head_sha"] = "b".repeat(40).into();
    for runs in [vec![remote(42, &p), remote(43, &p)], vec![wrong]] {
        let http = Boundary::new(vec![("GET", list(runs))]);
        let (registry, context) = credentials();
        let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
        assert_eq!(
            error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context))
                .kind,
            ErrorKind::Ambiguous
        );
        assert_eq!(http.methods(), ["GET"]);
    }
}

#[test]
fn update_and_uncertain_update_recovery_do_not_append_annotations_twice() {
    let old = projection(1, false);
    let mut p = projection(2, true);
    p.annotations.push(Annotation {
        path: "src/main.rs".into(),
        line: 1,
        message: "bounded observation".into(),
    });
    let mut binding = CheckRunBinding::new(target()).unwrap();
    binding.check_id = Some(42);
    binding.applied = Some(AppliedProjection {
        version: 1,
        update_id: old.update_id,
        terminal: false,
    });
    let http = Boundary::new(vec![
        ("GET", list(vec![remote(42, &old)])),
        ("PATCH", Err(HttpError::Transport("lost".into()))),
        ("GET", list(vec![remote(42, &p)])),
    ]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let plan = prepare_check(&binding, &p).unwrap();
    let reservation = plan.reserved_binding().clone();
    assert_eq!(error(client.execute(plan, &p, &context)).kind, ErrorKind::Transient);
    let recovered = client.execute(prepare_check(&reservation, &p).unwrap(), &p, &context).unwrap();
    assert_eq!(recovered.applied.unwrap().version, 2);
    assert_eq!(http.methods(), ["GET", "PATCH", "GET"]);
}

#[test]
fn stale_projection_is_noop_and_terminal_mutation_is_denied() {
    let mut binding = CheckRunBinding::new(target()).unwrap();
    binding.check_id = Some(42);
    let latest = projection(2, true);
    binding.applied = Some(AppliedProjection {
        version: 2,
        update_id: latest.update_id,
        terminal: true,
    });
    let http = Boundary::new(vec![]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let stale = projection(1, false);
    assert_eq!(client.execute(prepare_check(&binding, &stale).unwrap(), &stale, &context).unwrap(), binding);
    assert_eq!(error(prepare_check(&binding, &projection(3, true))).kind, ErrorKind::StateConflict);
    let mut conflicting = latest;
    conflicting.update_id = Identity([99; 32]);
    assert_eq!(error(prepare_check(&binding, &conflicting)).kind, ErrorKind::StateConflict);
    assert!(http.methods().is_empty());
}

#[test]
fn pagination_is_bounded_and_untrusted_link_is_never_followed() {
    let p = projection(1, false);
    let mut first = Vec::new();
    for id in 1..=100 {
        let mut run = remote(id, &p);
        run["external_id"] = "unrelated".into();
        run["output"] = serde_json::json!({"summary":null});
        first.push(run);
    }
    let mut first_response =
        response(200, serde_json::json!({"total_count":101,"check_runs":first}).to_string()).unwrap();
    first_response
        .headers
        .insert("Link".into(), "<https://attacker.example/?token=secret>; rel=next".into());
    let http = Boundary::new(vec![
        ("GET", Ok(first_response)),
        (
            "GET",
            response(200, serde_json::json!({"total_count":101,"check_runs":[remote(101,&p)]}).to_string()),
        ),
    ]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let binding = client
        .execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context)
        .unwrap();
    assert_eq!(binding.check_id, Some(101));
    assert_eq!(http.methods(), ["GET", "GET"]);
    let http = Boundary::new(vec![(
        "GET",
        response(200, serde_json::json!({"total_count":1001,"check_runs":[]}).to_string()),
    )]);
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    assert_eq!(
        error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context)).kind,
        ErrorKind::Bounds
    );
}

#[test]
fn annotation_title_summary_and_text_limits_reject_instead_of_truncating() {
    let binding = CheckRunBinding::new(target()).unwrap();
    let mut p = projection(1, false);
    p.title = "x".repeat(MAX_TITLE_BYTES + 1);
    assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
    let mut p = projection(1, false);
    p.summary = "x".repeat(MAX_SUMMARY_BYTES + 1);
    assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
    let mut p = projection(1, false);
    p.text = "x".repeat(MAX_TEXT_BYTES + 1);
    assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
    let mut p = projection(1, false);
    p.annotations = vec![
        Annotation {
            path: "src/main.rs".into(),
            line: 1,
            message: "bounded".into()
        };
        MAX_ANNOTATIONS + 1
    ];
    assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
}

#[test]
fn revoked_missing_permission_abuse_and_transient_are_closed_redacted_categories() {
    for (status, body, kind) in [
        (401, "fixture-api-bearer", ErrorKind::RevokedCredential),
        (404, "fixture-api-bearer", ErrorKind::MissingRepository),
        (403, "denied fixture-api-bearer", ErrorKind::PermissionDenied),
        (403, "secondary rate limit fixture-api-bearer", ErrorKind::AbuseLimited),
        (503, "fixture-api-bearer", ErrorKind::Transient),
    ] {
        let http = Boundary::new(vec![("GET", response(status, body))]);
        let (registry, context) = credentials();
        let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
        let p = projection(1, false);
        let e =
            error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context));
        assert_eq!(e.kind, kind);
        assert!(!format!("{e:?} {e}").contains("fixture-api-bearer"));
        if status == 503 {
            assert!(!e.remote_rejected);
        }
        if status == 401 {
            assert_eq!(e.retry_decision(0, 0, 0, &policy()).unwrap(), core::RetryDecision::Stop);
        }
    }
}

#[test]
fn rate_limits_use_injected_time_and_never_retry_before_server_delay() {
    let mut limited = response(403, "fixture-api-bearer").unwrap();
    limited.headers.insert("X-RateLimit-Remaining".into(), "0".into());
    limited.headers.insert("X-RateLimit-Reset".into(), "10".into());
    let http = Boundary::new(vec![("GET", Ok(limited))]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let p = projection(1, false);
    let e = error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context));
    assert_eq!(e.kind, ErrorKind::RateLimited);
    assert_eq!(e.retry_decision(0, 1000, 2000, &policy()).unwrap(), core::RetryDecision::RetryAt(10_000));
    let mut delayed = e;
    delayed.retry_after_ms = Some(60_001);
    assert_eq!(delayed.retry_decision(0, 1000, 2000, &policy()).unwrap(), core::RetryDecision::Stop);
}

#[test]
fn denied_context_and_missing_credential_disable_all_http_mutations() {
    let http = Boundary::new(vec![]);
    let (registry, mut context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let p = projection(1, false);
    context.destination = CredentialDestination::ShellEnvironment;
    assert_eq!(
        error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context)).kind,
        ErrorKind::CredentialUnavailable
    );
    context.destination = CredentialDestination::HttpHeader;
    context.reference.key = "missing".into();
    assert_eq!(
        error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context)).kind,
        ErrorKind::CredentialUnavailable
    );
    assert!(http.methods().is_empty());
}

#[test]
fn remote_newer_projection_is_never_overwritten_by_stale_local_state() {
    let newer = projection(3, true);
    let stale = projection(1, false);
    let http = Boundary::new(vec![("GET", list(vec![remote(42, &newer)]))]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let binding = client
        .execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &stale).unwrap(), &stale, &context)
        .unwrap();
    assert_eq!(binding.applied.unwrap().version, 3);
    assert_eq!(http.methods(), ["GET"]);
}

#[test]
fn definitive_rate_rejection_can_retry_but_server_error_keeps_reservation() {
    let p = projection(1, false);
    for (status, clear) in [(429, true), (503, false)] {
        let http = Boundary::new(vec![("GET", list(vec![])), ("POST", response(status, "fixture-api-bearer"))]);
        let (registry, context) = credentials();
        let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
        let plan = prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap();
        let reserved = plan.reserved_binding().clone();
        let e = error(client.execute(plan, &p, &context));
        assert_eq!(reserved.after_rejection(&e).is_ok(), clear);
        if clear {
            assert!(reserved.after_rejection(&e).unwrap().pending.is_none());
        }
    }
}

#[test]
fn annotation_maximum_is_admitted_but_unsafe_paths_and_zero_lines_are_rejected() {
    let binding = CheckRunBinding::new(target()).unwrap();
    let mut p = projection(1, false);
    p.annotations = vec![
        Annotation {
            path: "src/main.rs".into(),
            line: 1,
            message: "public observation".into()
        };
        MAX_ANNOTATIONS
    ];
    assert!(prepare_check(&binding, &p).is_ok());
    for path in ["../secret", "/absolute", "src/../secret", "src\\secret"] {
        p.annotations[0].path = path.into();
        assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
    }
    p.annotations[0].path = "src/main.rs".into();
    p.annotations[0].line = 0;
    assert_eq!(error(prepare_check(&binding, &p)).kind, ErrorKind::Bounds);
}

#[test]
fn recovery_read_denial_does_not_clear_prior_uncertain_mutation() {
    let p = projection(1, false);
    let reserved = prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap().reserved_binding().clone();
    let http = Boundary::new(vec![("GET", response(404, "fixture-api-bearer"))]);
    let (registry, context) = credentials();
    let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
    let e = error(client.execute(prepare_check(&reserved, &p).unwrap(), &p, &context));
    assert_eq!(e.kind, ErrorKind::MissingRepository);
    assert!(!e.remote_rejected);
    assert!(reserved.after_rejection(&e).is_err());
}

#[test]
fn api_redirect_malformed_body_and_header_body_overflow_are_rejected() {
    let mut headers = response(200, "{}").unwrap();
    headers.headers.insert("X-Untrusted".into(), "x".repeat(MAX_HEADER_BYTES));
    let cases = vec![
        (response(302, "fixture-api-bearer"), ErrorKind::Redirect),
        (response(200, "not JSON fixture-api-bearer"), ErrorKind::Malformed),
        (response(200, "x".repeat(MAX_API_BODY_BYTES + 1)), ErrorKind::Bounds),
        (Ok(headers), ErrorKind::Bounds),
    ];
    for (response, expected) in cases {
        let http = Boundary::new(vec![("GET", response)]);
        let (registry, context) = credentials();
        let client = GitHubChecksClient::new(&http, &registry, &hosts()).unwrap();
        let p = projection(1, false);
        let e =
            error(client.execute(prepare_check(&CheckRunBinding::new(target()).unwrap(), &p).unwrap(), &p, &context));
        assert_eq!(e.kind, expected);
        assert!(!format!("{e:?} {e}").contains("fixture-api-bearer"));
        assert_eq!(http.methods(), ["GET"]);
    }
}
