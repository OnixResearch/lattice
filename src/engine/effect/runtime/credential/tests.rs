use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use super::*;
use crate::credentials::CredentialError;
use crate::credentials::SecretRef;
use crate::credentials::SecretValue;
use crate::http::HttpClient;
use crate::http::HttpError;
use crate::http::HttpRequestPlan;
use crate::http::HttpResponse;

const SECRET: &str = "credential-private-sentinel";

struct Provider {
    calls: AtomicUsize,
    fail_second: bool,
}

impl CredentialProvider for Provider {
    fn resolve(&self, reference: &SecretRef) -> Result<SecretValue, CredentialError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if self.fail_second && reference.key == "second" {
            return Err(CredentialError::MissingSecret {
                identity: reference.identity(),
            });
        }
        Ok(SecretValue::new(SECRET))
    }
}

fn provider(fail_second: bool) -> Arc<Provider> {
    Arc::new(Provider {
        calls: AtomicUsize::new(0),
        fail_second,
    })
}

fn subject() -> CredentialSubject {
    CredentialSubject::Node {
        workflow_id: "workflow".to_owned(),
        node_id: "node".to_owned(),
    }
}

fn references(first: &str, second: &str) -> NodeCredentialRefs {
    NodeCredentialRefs {
        env: HashMap::from([
            (first.to_owned(), SecretRef::new("test", "first", None)),
            (second.to_owned(), SecretRef::new("test", "second", None)),
        ]),
    }
}

fn process_request(command: &str) -> crate::agent_node::ProcessRequest {
    crate::agent_node::ProcessRequest {
        program: "/bin/sh".to_owned(),
        args: vec!["-c".to_owned(), command.to_owned()],
        cwd: None,
        env: BTreeMap::from([("A_TOKEN".to_owned(), "literal-is-overridden".to_owned())]),
        stdin: Vec::new(),
        timeout_secs: 5,
        output_limit_bytes: 4096,
    }
}

#[test]
fn process_delivery_redacts_both_streams_and_rejects_transport_failures() {
    let references = references("A_TOKEN", "B_TOKEN");
    let cancellation = AtomicBool::new(false);
    let runtime = crate::process::core::RuntimeConfig::default();
    let recipe = ProcessRecipe::new(
        process_request("printf '%s' \"$A_TOKEN\"; printf '%s' \"$B_TOKEN\" >&2"),
        Some(&references),
        provider(false),
    )
    .unwrap();
    let output = recipe.consume(&runtime, Some(&cancellation), &subject()).unwrap();
    assert_eq!(output.status_code, Some(0));
    assert_eq!(output.stdout, b"<redacted>");
    assert_eq!(output.stderr, b"<redacted>");
    assert!(matches!(recipe.consume(&runtime, Some(&cancellation), &subject()), Err(Error::AdapterRejected)));

    for missing_executable in [false, true] {
        let mut request = process_request("printf '%s' \"$A_TOKEN\" >&2");
        if missing_executable {
            request.program = "/lattice-test-no-such-executable".to_owned();
        } else {
            request.output_limit_bytes = 0;
        }
        let provider = provider(false);
        let recipe = ProcessRecipe::new(request, Some(&references), provider.clone()).unwrap();
        let error = recipe.consume(&runtime, Some(&cancellation), &subject()).unwrap_err();
        assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
        assert!(matches!(error, Error::AdapterRejected));
    }
}

#[test]
fn partial_resolution_blocks_process_before_any_child_runs() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("should-not-exist");
    let mut request = process_request("printf started > \"$1\"");
    request.args.extend(["test".to_owned(), marker.to_str().unwrap().to_owned()]);
    let provider = provider(true);
    let recipe = ProcessRecipe::new(request, Some(&references("A_TOKEN", "B_TOKEN")), provider.clone()).unwrap();
    let result =
        recipe.consume(&crate::process::core::RuntimeConfig::default(), Some(&AtomicBool::new(false)), &subject());
    assert!(matches!(result, Err(Error::AdapterRejected)));
    assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
    assert!(!marker.exists());
}

#[derive(Clone, Copy)]
enum Outcome {
    Success,
    Failure,
    Cancellation,
}

struct Client {
    outcome: Outcome,
    calls: AtomicUsize,
}

impl HttpClient for Client {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        let authorization = request
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("authorization"))
            .collect::<Vec<_>>();
        assert_eq!(authorization.len(), 1);
        assert_eq!(authorization[0].1, SECRET);
        assert_eq!(request.headers.get("x-token").map(String::as_str), Some(SECRET));
        match self.outcome {
            Outcome::Success => Ok(HttpResponse {
                status: 200,
                headers: HashMap::from([("x-echo".to_owned(), format!("prefix {SECRET} suffix"))]),
                body: format!("result {SECRET}"),
            }),
            Outcome::Failure => Err(HttpError::Transport(format!("hostile transport detail: {SECRET}"))),
            Outcome::Cancellation => Err(HttpError::Cancelled),
        }
    }

    fn execute_with_cancellation(
        &self,
        request: &HttpRequestPlan,
        cancellation: Option<&AtomicBool>,
    ) -> Result<HttpResponse, HttpError> {
        if matches!(self.outcome, Outcome::Cancellation) {
            cancellation.unwrap().store(true, Ordering::Release);
        }
        self.execute(request)
    }
}

fn network_request() -> HttpRequestPlan {
    HttpRequestPlan {
        method: "GET".to_owned(),
        url: "https://example.invalid/credentials".to_owned(),
        headers: HashMap::from([("AUTHORIZATION".to_owned(), "literal-is-overridden".to_owned())]),
        body: None,
        timeout_secs: 5,
    }
}

#[test]
fn network_delivery_redacts_output_and_rejects_errors_or_cancellation_after_resolution() {
    for outcome in [Outcome::Success, Outcome::Failure, Outcome::Cancellation] {
        let provider = provider(false);
        let recipe =
            NetworkRecipe::new(network_request(), Some(&references("authorization", "x-token")), provider.clone())
                .unwrap();
        let client = Client {
            outcome,
            calls: AtomicUsize::new(0),
        };
        let cancellation = AtomicBool::new(false);
        let result = recipe.consume_for_embedding(&client, &cancellation, &subject());
        assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
        assert_eq!(client.calls.load(Ordering::Relaxed), 1);
        match outcome {
            Outcome::Success => {
                let response = result.unwrap();
                assert_eq!(response.status, 200);
                assert_eq!(response.body, "result <redacted>");
                assert_eq!(response.headers.get("x-echo").map(String::as_str), Some("prefix <redacted> suffix"));
            }
            Outcome::Failure | Outcome::Cancellation => {
                assert_eq!(result.unwrap_err(), "credential_network_rejected");
            }
        }
        assert_eq!(cancellation.load(Ordering::Acquire), matches!(outcome, Outcome::Cancellation));
        assert_eq!(
            recipe.consume_for_embedding(&client, &cancellation, &subject()).unwrap_err(),
            "credential_network_rejected"
        );
        assert_eq!(client.calls.load(Ordering::Relaxed), 1);
    }

    let provider = provider(true);
    let recipe =
        NetworkRecipe::new(network_request(), Some(&references("authorization", "x-token")), provider.clone()).unwrap();
    let client = Client {
        outcome: Outcome::Success,
        calls: AtomicUsize::new(0),
    };
    assert_eq!(
        recipe.consume_for_embedding(&client, &AtomicBool::new(false), &subject()).unwrap_err(),
        "credential_network_rejected"
    );
    assert_eq!(provider.calls.load(Ordering::Relaxed), 2);
    assert_eq!(client.calls.load(Ordering::Relaxed), 0);
}
