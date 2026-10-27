use super::{RecipeRetryCheck, RecipeRetryPolicy};

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Data required to classify retry requests against active host denials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecipeRetryPlanDraft<'a> {
    /// Validated retry policy.
    pub policy: &'a RecipeRetryPolicy,
    /// Capability labels currently denied by host policy.
    pub denied_capabilities: &'a [&'a str],
    /// Sandbox labels currently denied by host policy.
    pub denied_sandboxes: &'a [&'a str],
}

/// Retry plan status for one host-effect request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeRetryRequestStatus {
    /// Host may evaluate the request; reusable core has not executed it.
    PendingHostEffect,
    /// Host policy denied the request before execution.
    Denied {
        /// Denial reason class.
        reason: RecipeRetryDenialReason,
    },
}

/// Retry request denial reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipeRetryDenialReason {
    /// Active capability policy denied the request.
    Capability,
    /// Active sandbox policy denied the request.
    Sandbox,
}

/// Planned retry host-effect request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeRetryRequest {
    label: Text,
    status: RecipeRetryRequestStatus,
}

impl RecipeRetryRequest {
    /// Returns the retry check label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the classified host-effect status.
    #[must_use]
    pub const fn status(&self) -> RecipeRetryRequestStatus {
        self.status
    }
}

/// Pure retry plan; it records requests but never reports validation success.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeRetryPlan {
    requests: Collection<RecipeRetryRequest>,
}

impl RecipeRetryPlan {
    /// Returns planned host-effect requests.
    #[must_use]
    pub fn requests(&self) -> &[RecipeRetryRequest] {
        self.requests.as_slice()
    }
}

/// Builds a retry plan from validated metadata and active host denials.
#[must_use]
pub fn plan_recipe_retry(draft: RecipeRetryPlanDraft<'_>) -> RecipeRetryPlan {
    let mut requests: Collection<RecipeRetryRequest> =
        Collection::with_capacity(draft.policy.checks().len());
    for check in draft.policy.checks() {
        let status =
            classify_retry_request(check, draft.denied_capabilities, draft.denied_sandboxes);
        requests.push(RecipeRetryRequest {
            label: Text::from(check.label()),
            status,
        });
    }
    RecipeRetryPlan { requests }
}
fn classify_retry_request(
    check: &RecipeRetryCheck,
    denied_capabilities: &[&str],
    denied_sandboxes: &[&str],
) -> RecipeRetryRequestStatus {
    if denied_capabilities
        .iter()
        .any(|denied| *denied == check.capability())
    {
        return RecipeRetryRequestStatus::Denied {
            reason: RecipeRetryDenialReason::Capability,
        };
    }
    if denied_sandboxes
        .iter()
        .any(|denied| *denied == check.sandbox())
    {
        return RecipeRetryRequestStatus::Denied {
            reason: RecipeRetryDenialReason::Sandbox,
        };
    }
    RecipeRetryRequestStatus::PendingHostEffect
}
