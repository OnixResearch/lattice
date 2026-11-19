//! One bounded selection's request, typed failures, and outcome.

pub(crate) mod flow;

/// Decides `request` under the artifact slot `policy`.
///
/// `policy` is the result of [`crate::admit_policy`]: pass `Err(&AdmissionError::Unavailable)`
/// when no artifact is loaded. Every refusal is a typed [`Failure`]. A request whose own shape
/// is valid and that declares a fallback answers with it; otherwise the outcome is
/// [`Outcome::Refused`]. Inference never runs for a refused binding or input.
///
/// [`AdmissionError`]: crate::AdmissionError
// r[impl agent_harness_runtime.frozen_policy.selection]
// r[impl agent_harness_runtime.frozen_policy.failure]
#[must_use]
pub fn decide(
    policy: Result<&crate::AdmittedPolicy, &crate::AdmissionError>,
    request: &DecisionRequest,
) -> Decision {
    let (shape, attempt) = flow::run(policy, request);
    let outcome = match (attempt.selected.clone(), &request.fallback) {
        (Ok((action, score)), _) => Outcome::Selected { action, score },
        (Err(failure), Some(fallback)) if shape.is_ok() => Outcome::Fallback {
            action: fallback.clone(),
            failure,
        },
        (Err(failure), _) => Outcome::Refused { failure },
    };
    debug_assert!(
        shape.is_ok() || matches!(outcome, Outcome::Refused { .. }),
        "a malformed request never falls back"
    );
    let receipt = crate::record::issue(request, attempt, &outcome, shape.is_ok());
    Decision { outcome, receipt }
}

/// The artifact and schemas a consumer's operation profile pins.
///
/// A binding names immutable identities, never a mutable model name or path.
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyBinding {
    /// The action schema identity the consumer's labels belong to.
    pub action_schema: alloc::string::String,
    /// The selected artifact identity.
    pub artifact: alloc::string::String,
    /// The feature schema identity the consumer's projection implements.
    pub feature_schema: alloc::string::String,
}

/// One request for a decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRequest {
    /// What the consumer's profile pins.
    pub binding: PolicyBinding,
    /// Raw feature values in the artifact's declared feature order.
    pub features: alloc::vec::Vec<i64>,
    /// The actions the consumer has already declared eligible for this decision; any order.
    pub eligible: alloc::vec::Vec<alloc::string::String>,
    /// The eligible action that answers when the policy cannot, if the consumer declares one.
    pub fallback: Option<alloc::string::String>,
    /// The most inference operations this decision may take: the deterministic deadline.
    pub max_operations: u64,
}

/// Which pinned identity differs from the admitted artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaleField {
    /// The artifact identity.
    Artifact,
    /// The feature schema identity.
    FeatureSchema,
    /// The action schema identity.
    ActionSchema,
}

/// Why a request is malformed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputFault {
    /// A binding identity is empty, oversized, or unsafe.
    Binding,
    /// The feature count differs from the declared features, or exceeds [`crate::MAX_FEATURES`].
    FeatureCount,
    /// No action is eligible.
    NoEligibleAction,
    /// More than [`crate::MAX_ACTIONS`] actions are eligible.
    TooManyActions,
    /// An eligible label repeats, or is unsafe or oversized.
    DuplicateAction,
    /// An eligible label is not in the artifact's action schema.
    UnknownAction,
    /// The fallback action is not eligible.
    FallbackNotEligible,
}

impl InputFault {
    /// The stable kebab-case code of the fault.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::FeatureCount => "feature-count",
            Self::NoEligibleAction => "no-eligible-action",
            Self::TooManyActions => "too-many-actions",
            Self::DuplicateAction => "duplicate-action",
            Self::UnknownAction => "unknown-action",
            Self::FallbackNotEligible => "fallback-not-eligible",
        }
    }
}

/// Why the policy did not select an action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Failure {
    /// No artifact was supplied.
    Unavailable,
    /// The artifact was refused at admission.
    IncompatibleArtifact(crate::AdmissionError),
    /// The consumer's binding pins another artifact or schema than the admitted one.
    Stale(StaleField),
    /// The request is malformed.
    MalformedInput(InputFault),
    /// A raw feature lies outside its declared range; its index is recorded.
    InputOutOfRange {
        /// The feature's index in declared order.
        feature: usize,
    },
    /// The artifact's worst-case operations exceed the request's budget.
    OperationBudget,
    /// A score lies outside the declared bounds, or checked arithmetic overflowed.
    ScoreOutOfRange,
    /// The best eligible score lies below the declared abstention threshold.
    Abstained,
}

impl Failure {
    /// Every failure code, in declaration order.
    pub const CODES: [&'static str; 8] = [
        "unavailable",
        "incompatible-artifact",
        "stale-binding",
        "malformed-input",
        "input-out-of-range",
        "operation-budget",
        "score-out-of-range",
        "abstained",
    ];

    /// The stable kebab-case code of the failure.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Unavailable => Self::CODES[0],
            Self::IncompatibleArtifact(_) => Self::CODES[1],
            Self::Stale(_) => Self::CODES[2],
            Self::MalformedInput(_) => Self::CODES[3],
            Self::InputOutOfRange { .. } => Self::CODES[4],
            Self::OperationBudget => Self::CODES[5],
            Self::ScoreOutOfRange => Self::CODES[6],
            Self::Abstained => Self::CODES[7],
        }
    }

    /// The failure's bounded detail: the admission code, stale field, input fault, or feature
    /// index.
    #[must_use]
    pub fn detail(self) -> Option<alloc::string::String> {
        let detail = match self {
            Self::IncompatibleArtifact(error) => error.code(),
            Self::Stale(StaleField::Artifact) => "artifact",
            Self::Stale(StaleField::FeatureSchema) => "feature-schema",
            Self::Stale(StaleField::ActionSchema) => "action-schema",
            Self::MalformedInput(fault) => fault.code(),
            Self::InputOutOfRange { feature } => return Some(alloc::format!("feature-{feature}")),
            Self::Unavailable | Self::OperationBudget | Self::ScoreOutOfRange | Self::Abstained => {
                return None;
            }
        };
        Some(alloc::string::String::from(detail))
    }
}

/// What one decision answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// The policy selected an eligible action.
    Selected {
        /// The selected action label.
        action: alloc::string::String,
        /// Its score.
        score: i64,
    },
    /// The policy failed and the consumer's declared fallback answered.
    Fallback {
        /// The fallback action label.
        action: alloc::string::String,
        /// Why the policy did not select.
        failure: Failure,
    },
    /// The policy failed and no valid fallback was declared; the consumer takes its terminal path.
    Refused {
        /// Why the policy did not select.
        failure: Failure,
    },
}

/// A decision and its receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    /// What the decision answered.
    pub outcome: Outcome,
    /// The bounded receipt of the decision.
    pub receipt: crate::DecisionReceipt,
}
