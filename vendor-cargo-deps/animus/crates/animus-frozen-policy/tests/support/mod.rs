//! Shared support of the frozen-policy leaf tests.

#![allow(
    dead_code,
    reason = "each integration-test binary uses a subset of the shared support"
)]

/// The checked tree example, exported from `config/frozen-policy-examples.ncl`.
pub const TREE: &str = include_str!("../../../../config/generated/frozen-policy-example-tree.json");
/// The checked MLP example, exported from `config/frozen-policy-examples.ncl`.
pub const MLP: &str = include_str!("../../../../config/generated/frozen-policy-example-mlp.json");
/// The examples' feature schema.
pub const FEATURE_SCHEMA: &str = "animus.frozen-policy-example-features.v1";
/// The examples' action schema.
pub const ACTION_SCHEMA: &str = "animus.frozen-policy-example-actions.v1";
/// Every example action, in declared order.
pub const ACTIONS: [&str; 3] = ["proceed", "retry", "escalate"];
/// An operation budget no example exceeds.
pub const AMPLE_OPERATIONS: u64 = 64;

/// One edit of a JSON document under test.
pub type JsonEdit = fn(&mut serde_json::Value);
/// One artifact edit and the admission error it must cause.
pub type Refusal = (JsonEdit, animus_frozen_policy::AdmissionError);
/// One request edit under test.
pub type RequestEdit = fn(&mut animus_frozen_policy::DecisionRequest);

/// Why a test step failed.
#[derive(Debug)]
pub enum Failure {
    /// JSON could not be read or written.
    Json,
    /// An artifact was refused.
    Admission(animus_frozen_policy::AdmissionError),
    /// A receipt was refused.
    Receipt(animus_frozen_policy::ReceiptError),
}

impl From<serde_json::Error> for Failure {
    fn from(_: serde_json::Error) -> Self {
        Self::Json
    }
}

impl From<animus_frozen_policy::AdmissionError> for Failure {
    fn from(error: animus_frozen_policy::AdmissionError) -> Self {
        Self::Admission(error)
    }
}

impl From<animus_frozen_policy::ReceiptError> for Failure {
    fn from(error: animus_frozen_policy::ReceiptError) -> Self {
        Self::Receipt(error)
    }
}

/// `text` admitted under its own canonical identity.
pub fn admitted(text: &str) -> Result<animus_frozen_policy::AdmittedPolicy, Failure> {
    let identity = animus_frozen_policy::policy_identity(text.as_bytes())?;
    Ok(animus_frozen_policy::admit_policy(
        text.as_bytes(),
        &identity,
    )?)
}

/// `text` with `edit` applied, as bytes and their own canonical identity, or `text`'s identity
/// when the edit leaves no document to identify.
pub fn edited(
    text: &str,
    edit: impl FnOnce(&mut serde_json::Value),
) -> Result<(Vec<u8>, String), Failure> {
    let selected = animus_frozen_policy::policy_identity(text.as_bytes())?;
    let mut value: serde_json::Value = serde_json::from_str(text)?;
    edit(&mut value);
    let bytes = serde_json::to_vec(&value)?;
    let identity = animus_frozen_policy::policy_identity(&bytes).unwrap_or(selected);
    Ok((bytes, identity))
}

/// `text` with `edit` applied, admitted under its new identity.
pub fn admitted_edit(
    text: &str,
    edit: impl FnOnce(&mut serde_json::Value),
) -> Result<animus_frozen_policy::AdmittedPolicy, Failure> {
    let (bytes, identity) = edited(text, edit)?;
    Ok(animus_frozen_policy::admit_policy(&bytes, &identity)?)
}

/// Asserts that each edit of `text`, admitted under its own identity, is refused with its error,
/// and returns how many were.
pub fn refusals(text: &str, cases: &[Refusal]) -> Result<u32, Failure> {
    let mut refused = 0_u32;
    for (edit, expected) in cases {
        let (bytes, identity) = edited(text, edit)?;
        assert_eq!(
            animus_frozen_policy::admit_policy(&bytes, &identity),
            Err(*expected),
            "{}",
            String::from_utf8_lossy(&bytes)
        );
        refused = refused.saturating_add(1);
    }
    Ok(refused)
}

/// A request bound to `policy` for `features`, with every action eligible and no fallback.
pub fn request(
    policy: &animus_frozen_policy::AdmittedPolicy,
    features: &[i64],
) -> animus_frozen_policy::DecisionRequest {
    animus_frozen_policy::DecisionRequest {
        binding: animus_frozen_policy::PolicyBinding {
            action_schema: String::from(ACTION_SCHEMA),
            artifact: String::from(policy.identity()),
            feature_schema: String::from(FEATURE_SCHEMA),
        },
        features: features.to_vec(),
        eligible: labels(&ACTIONS),
        fallback: None,
        max_operations: AMPLE_OPERATIONS,
    }
}

/// [`request`] with `fallback` declared.
pub fn request_with_fallback(
    policy: &animus_frozen_policy::AdmittedPolicy,
    features: &[i64],
    fallback: &str,
) -> animus_frozen_policy::DecisionRequest {
    let mut request = request(policy, features);
    request.fallback = Some(String::from(fallback));
    request
}

/// `labels` as owned strings.
pub fn labels(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|label| String::from(*label)).collect()
}

/// The fallback outcome answering `action` for `failure`.
pub fn fallback(
    action: &str,
    failure: animus_frozen_policy::Failure,
) -> animus_frozen_policy::Outcome {
    animus_frozen_policy::Outcome::Fallback {
        action: String::from(action),
        failure,
    }
}

/// The selected outcome of `action` at `score`.
pub fn selected(action: &str, score: i64) -> animus_frozen_policy::Outcome {
    animus_frozen_policy::Outcome::Selected {
        action: String::from(action),
        score,
    }
}
