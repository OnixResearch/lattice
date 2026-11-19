//! Independent Lattice-side receipt validation, typed outcomes and fallback resolution.

const FAILURE_ABSTAINED: &str = "abstained";
const FAILURE_UNAVAILABLE: &str = "unavailable";
const FAILURE_OPERATION_BUDGET: &str = "operation-budget";
const REASON_BACKEND_UNAVAILABLE: &str = "backend-unavailable";
const REASON_ARTIFACT_UNAVAILABLE: &str = "artifact-unavailable";
const REASON_DEADLINE: &str = "deadline";
const REASON_OVERSIZED: &str = "oversized-receipt";
const REASON_UNVERIFIED: &str = "unverified-receipt";
const REASON_CROSSED_BINDING: &str = "crossed-binding";
const REASON_CROSSED_INPUT: &str = "crossed-input";
const REASON_CANDIDATE_SET: &str = "candidate-set";
const REASON_STALE_RUNTIME: &str = "stale-runtime";
const REASON_STALE_ARTIFACT: &str = "stale-artifact";
const REASON_UNDECLARED_LABEL: &str = "undeclared-label";
const REASON_UNEXPECTED_FALLBACK: &str = "unexpected-fallback";
const REASON_INCONSISTENT: &str = "inconsistent-receipt";
const REASON_CANCELLED: &str = "cancelled";
const REASON_AMBIGUOUS_PREDICATES: &str = "fallback-predicates-not-unique";

/// The typed outcome of one invocation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    /// A declared label passed every independent check.
    Accepted {
        /// The accepted label.
        label: String,
        /// The runtime score; evidence only, never permission.
        score: i64,
    },
    /// The policy's best score was below its abstention threshold.
    Abstained,
    /// The backend or artifact was unavailable.
    Unavailable {
        /// The bounded reason.
        reason: String,
    },
    /// The wall-clock deadline elapsed or the operation budget was exceeded.
    TimedOut {
        /// The bounded reason.
        reason: String,
    },
    /// The response failed an independent check.
    Invalid {
        /// The bounded reason.
        reason: String,
    },
    /// Cancellation was observed; terminal.
    Cancelled,
}

impl Outcome {
    /// The bounded code naming this outcome and its reason.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::Accepted { .. } => "accepted".to_owned(),
            Self::Abstained => FAILURE_ABSTAINED.to_owned(),
            Self::Unavailable { reason } => format!("unavailable:{reason}"),
            Self::TimedOut { reason } => format!("timed_out:{reason}"),
            Self::Invalid { reason } => format!("invalid:{reason}"),
            Self::Cancelled => REASON_CANCELLED.to_owned(),
        }
    }

    fn invalid(reason: &str) -> Self {
        Self::Invalid {
            reason: reason.to_owned(),
        }
    }
}

/// A validated response: its outcome and, when verified, the receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// The typed outcome.
    pub outcome: Outcome,
    /// The verified receipt, when the bytes verified.
    pub receipt: Option<animus_frozen_policy::DecisionReceipt>,
}

/// The outcome for an elapsed wall-clock deadline.
#[must_use]
pub fn timed_out() -> Checked {
    Checked {
        outcome: Outcome::TimedOut {
            reason: REASON_DEADLINE.to_owned(),
        },
        receipt: None,
    }
}

/// The outcome for observed cancellation.
#[must_use]
pub const fn cancelled() -> Checked {
    Checked {
        outcome: Outcome::Cancelled,
        receipt: None,
    }
}

/// The outcome for a projection refusal; no invocation was made.
#[must_use]
pub fn unprojected(fault: super::binding::ProjectionFault) -> Checked {
    Checked {
        outcome: Outcome::invalid(fault.code()),
        receipt: None,
    }
}

/// Validates one adapter response independently of the producer.
#[must_use]
// r[impl lattice.typed_policy.validation]
pub fn validate(
    profile: &super::admission::Profile,
    invocation: &super::binding::Invocation,
    response: &super::Response,
) -> Checked {
    let bytes = match response {
        super::Response::Receipt(bytes) => bytes,
        super::Response::Unavailable => {
            return Checked {
                outcome: Outcome::Unavailable {
                    reason: REASON_BACKEND_UNAVAILABLE.to_owned(),
                },
                receipt: None,
            };
        }
    };
    let is_bounded = u64::try_from(bytes.len()).is_ok_and(|size| size <= profile.result.max_receipt_bytes);
    if !is_bounded {
        return Checked {
            outcome: Outcome::invalid(REASON_OVERSIZED),
            receipt: None,
        };
    }
    let Ok(receipt) = animus_frozen_policy::verify_receipt(bytes) else {
        return Checked {
            outcome: Outcome::invalid(REASON_UNVERIFIED),
            receipt: None,
        };
    };
    let outcome = classify(profile, invocation, &receipt.body);
    debug_assert!(receipt.identity.starts_with(animus_frozen_policy::BLAKE3_PREFIX));
    Checked {
        outcome,
        receipt: Some(receipt),
    }
}

fn classify(
    profile: &super::admission::Profile,
    invocation: &super::binding::Invocation,
    body: &animus_frozen_policy::ReceiptBody,
) -> Outcome {
    if let Some(reason) = crossed(invocation, body).or_else(|| stale_runtime(&profile.cohort, body)) {
        return Outcome::invalid(reason);
    }
    match body.outcome {
        animus_frozen_policy::ReceiptOutcome::Selected => selected(profile, invocation, body),
        animus_frozen_policy::ReceiptOutcome::Fallback => Outcome::invalid(REASON_UNEXPECTED_FALLBACK),
        animus_frozen_policy::ReceiptOutcome::Refused => refused(body.failure.as_deref()),
    }
}

fn crossed(invocation: &super::binding::Invocation, body: &animus_frozen_policy::ReceiptBody) -> Option<&'static str> {
    if body.binding != invocation.binding {
        return Some(REASON_CROSSED_BINDING);
    }
    if body.features != invocation.features {
        return Some(REASON_CROSSED_INPUT);
    }
    if body.eligible != invocation.candidates {
        return Some(REASON_CANDIDATE_SET);
    }
    None
}

fn stale_runtime(cohort: &super::admission::Cohort, body: &animus_frozen_policy::ReceiptBody) -> Option<&'static str> {
    let is_same_runtime = body.runtime_crate == cohort.runtime_crate && body.runtime_version == cohort.runtime_version;
    if is_same_runtime && body.artifact_api == cohort.artifact_api {
        return None;
    }
    Some(REASON_STALE_RUNTIME)
}

fn selected(
    profile: &super::admission::Profile,
    invocation: &super::binding::Invocation,
    body: &animus_frozen_policy::ReceiptBody,
) -> Outcome {
    let cohort = &profile.cohort;
    let is_same_artifact = body.artifact.as_deref() == Some(cohort.artifact.as_str())
        && body.model.as_deref() == Some(cohort.model.as_str());
    if !is_same_artifact || body.policy_kind.as_deref() != Some(cohort.policy_kind.as_str()) {
        return Outcome::invalid(REASON_STALE_ARTIFACT);
    }
    let (Some(label), Some(score)) = (body.action.as_ref(), body.score) else {
        return Outcome::invalid(REASON_INCONSISTENT);
    };
    if invocation.candidates.contains(label) {
        return Outcome::Accepted {
            label: label.clone(),
            score,
        };
    }
    Outcome::invalid(REASON_UNDECLARED_LABEL)
}

fn refused(failure: Option<&str>) -> Outcome {
    match failure {
        Some(FAILURE_ABSTAINED) => Outcome::Abstained,
        Some(FAILURE_UNAVAILABLE) => Outcome::Unavailable {
            reason: REASON_ARTIFACT_UNAVAILABLE.to_owned(),
        },
        Some(FAILURE_OPERATION_BUDGET) => Outcome::TimedOut {
            reason: FAILURE_OPERATION_BUDGET.to_owned(),
        },
        Some(code) => Outcome::invalid(code),
        None => Outcome::invalid(REASON_INCONSISTENT),
    }
}

/// Who chose a routed label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// The accepted policy result.
    Policy,
    /// The profile's declared fallback.
    Fallback,
}

/// Where a resolved outcome goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    /// One declared branch label.
    Branch {
        /// The label.
        label: String,
        /// Who chose it.
        source: Source,
    },
    /// An explicit terminal failure.
    Terminal {
        /// The bounded reason.
        reason: String,
    },
}

/// Resolves `outcome` through the declared fallback; `predicate_labels` are the labels the
/// Condition node's own predicates matched.
#[must_use]
// r[impl lattice.typed_policy.fallback]
pub fn resolve(fallback: &super::admission::Fallback, outcome: &Outcome, predicate_labels: &[String]) -> Route {
    match outcome {
        Outcome::Accepted { label, .. } => Route::Branch {
            label: label.clone(),
            source: Source::Policy,
        },
        Outcome::Cancelled => Route::Terminal {
            reason: REASON_CANCELLED.to_owned(),
        },
        Outcome::Abstained | Outcome::Unavailable { .. } | Outcome::TimedOut { .. } | Outcome::Invalid { .. } => {
            fall_back(fallback, outcome, predicate_labels)
        }
    }
}

fn fall_back(fallback: &super::admission::Fallback, outcome: &Outcome, predicate_labels: &[String]) -> Route {
    match (fallback, predicate_labels) {
        (super::admission::Fallback::Label { label }, _)
        | (super::admission::Fallback::DeclaredPredicates, [label]) => Route::Branch {
            label: label.clone(),
            source: Source::Fallback,
        },
        (super::admission::Fallback::DeclaredPredicates, _) => Route::Terminal {
            reason: format!("{}; {REASON_AMBIGUOUS_PREDICATES}", outcome.code()),
        },
        (super::admission::Fallback::Terminal, _) => Route::Terminal { reason: outcome.code() },
    }
}
