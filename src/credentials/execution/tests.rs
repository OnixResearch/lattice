use parking_lot::Mutex;

use super::*;
use crate::credentials::SecretValue;
use crate::credentials::resolve_secret_env;
use crate::credentials::resolve_secret_env_in_context;

const FIRST: &str = "first-private-material";
const SECOND: &str = "second-private-material";

struct Provider {
    calls: Mutex<Vec<String>>,
    fail_second: bool,
    cancel_after_first: bool,
    cancellation: Arc<AtomicBool>,
}

impl CredentialProvider for Provider {
    fn resolve(&self, reference: &SecretRef) -> Result<SecretValue, CredentialError> {
        self.calls.lock().push(reference.key.clone());
        if self.cancellation.load(Ordering::Acquire) {
            return Err(cancelled_error());
        }
        if reference.key == "second" && self.fail_second {
            return Err(CredentialError::MissingSecret {
                identity: reference.identity(),
            });
        }
        if reference.key == "first" && self.cancel_after_first {
            self.cancellation.store(true, Ordering::Release);
        }
        Ok(SecretValue::new(if reference.key == "first" { FIRST } else { SECOND }))
    }
}

fn subject() -> CredentialSubject {
    CredentialSubject::Node {
        workflow_id: "workflow".to_owned(),
        node_id: "node".to_owned(),
    }
}

fn references() -> NodeCredentialRefs {
    // Deliberately inserted out of order: a late-slot error must follow material.
    NodeCredentialRefs {
        env: HashMap::from([
            ("B".to_owned(), SecretRef::new("test", "second", None)),
            ("A".to_owned(), SecretRef::new("test", "first", None)),
        ]),
    }
}

#[derive(Clone, Copy)]
enum Path {
    Legacy,
    Context,
    Bindings,
}

fn resolve(path: Path, provider: Arc<Provider>) -> Result<ResolvedSecretEnv, CredentialError> {
    let references = references();
    match path {
        Path::Legacy => resolve_secret_env(&references, provider.as_ref()),
        Path::Context => resolve_secret_env_in_context(
            &references,
            provider.as_ref(),
            &subject(),
            CredentialDestination::ShellEnvironment,
            Some(&provider.cancellation),
        ),
        Path::Bindings => Bindings::new(Some(&references), CredentialDestination::ShellEnvironment, provider.clone())?
            .resolve(&subject(), Some(&provider.cancellation)),
    }
}

fn provider(fail_second: bool, cancel_after_first: bool) -> Arc<Provider> {
    Arc::new(Provider {
        calls: Mutex::new(Vec::new()),
        fail_second,
        cancel_after_first,
        cancellation: Arc::new(AtomicBool::new(false)),
    })
}

#[test]
fn multi_slot_resolution_delivers_both_values_or_rejects_the_partial_result() {
    for path in [Path::Legacy, Path::Context, Path::Bindings] {
        let successful = provider(false, false);
        let resolved = resolve(path, successful.clone()).unwrap();
        assert_eq!(resolved.env.get("A").map(String::as_str), Some(FIRST));
        assert_eq!(resolved.env.get("B").map(String::as_str), Some(SECOND));
        assert_eq!(resolved.secret_values, [FIRST, SECOND]);
        let debug = format!("{resolved:?}");
        assert!(!debug.contains(FIRST));
        assert!(!debug.contains(SECOND));
        assert_eq!(*successful.calls.lock(), ["first", "second"]);

        let failing = provider(true, false);
        let error = resolve(path, failing.clone()).unwrap_err();
        assert!(matches!(error, CredentialError::MissingSecret { .. }));
        assert_eq!(*failing.calls.lock(), ["first", "second"]);
        assert!(!error.to_string().contains(FIRST));
        assert!(!error.to_string().contains(SECOND));
    }
}

#[test]
fn cancellation_after_a_successful_slot_rejects_contextual_and_sealed_resolution() {
    for path in [Path::Context, Path::Bindings] {
        let provider = provider(false, true);
        let error = resolve(path, provider.clone()).unwrap_err();
        assert_eq!(error.to_string(), cancelled_error().to_string());
        let expected = match path {
            Path::Context => vec!["first", "second"],
            Path::Bindings => vec!["first"],
            Path::Legacy => unreachable!(),
        };
        assert_eq!(*provider.calls.lock(), expected);
        assert!(!format!("{error:?}").contains(FIRST));
    }
}
