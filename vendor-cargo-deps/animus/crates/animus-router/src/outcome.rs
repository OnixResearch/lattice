#![allow(non_trait_imports, assertion_density, missing_docs)]

use alloc::string::String;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BudgetKind {
    Candidate,
    Parallelism,
    Synthesis,
    Repair,
}

impl BudgetKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Parallelism => "parallelism",
            Self::Synthesis => "synthesis",
            Self::Repair => "repair",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BudgetExhausted {
    kind: BudgetKind,
    remaining: u32,
}

impl BudgetExhausted {
    #[must_use]
    pub const fn new(kind: BudgetKind, remaining: u32) -> Self {
        Self { kind, remaining }
    }

    #[must_use]
    pub const fn kind(self) -> BudgetKind {
        self.kind
    }

    #[must_use]
    pub const fn remaining(self) -> u32 {
        self.remaining
    }
}

const RESULT_SUCCESS: &str = "success";
const RESULT_MALFORMED_OUTPUT: &str = "malformed_output";
const RESULT_BEST_VALID_EVIDENCE: &str = "best_valid_evidence";

/// Final route result status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteResultStatus {
    /// Final output validated successfully.
    Success,
    /// Final output was malformed and no acceptable fallback exists.
    MalformedOutput,
    /// Policy-declared best valid evidence fallback was used.
    BestValidEvidence,
}

impl RouteResultStatus {
    /// Returns stable result-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => RESULT_SUCCESS,
            Self::MalformedOutput => RESULT_MALFORMED_OUTPUT,
            Self::BestValidEvidence => RESULT_BEST_VALID_EVIDENCE,
        }
    }
}

/// Validated collaborative route result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteResult {
    status: RouteResultStatus,
    output_contract: crate::OutputContract,
    repair_attempts_used: u32,
}

impl RouteResult {
    /// Builds a validated route result from final-output validation facts.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when output-contract labels are invalid.
    pub fn new(draft: ResultDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let output_contract = crate::OutputContract::parse(draft.output_contract)?;
        let fallback_policy = crate::FallbackPolicy::parse(draft.fallback_policy)?;
        let status = result_status(draft.final_output_valid, fallback_policy, draft.best_valid_evidence_available);

        Ok(Self {
            status,
            output_contract,
            repair_attempts_used: draft.repair_attempts_used,
        })
    }

    /// Returns route result status.
    #[must_use]
    pub const fn status(&self) -> RouteResultStatus {
        self.status
    }

    /// Returns output contract.
    #[must_use]
    pub const fn output_contract(&self) -> crate::OutputContract {
        self.output_contract
    }

    /// Returns bounded repair attempts used.
    #[must_use]
    pub const fn repair_attempts_used(&self) -> u32 {
        self.repair_attempts_used
    }
}

/// Borrowed final-output validation facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultDraft<'a> {
    /// Output-contract label.
    pub output_contract: &'a str,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
    /// Whether final output passed validation.
    pub final_output_valid: bool,
    /// Bounded repair attempts used.
    pub repair_attempts_used: u32,
    /// Whether best valid evidence is available and policy-declared.
    pub best_valid_evidence_available: bool,
}

fn result_status(
    final_output_valid: bool,
    fallback_policy: crate::FallbackPolicy,
    best_valid_evidence_available: bool,
) -> RouteResultStatus {
    if final_output_valid {
        return RouteResultStatus::Success;
    }
    if fallback_policy == crate::FallbackPolicy::BestValidEvidence && best_valid_evidence_available {
        return RouteResultStatus::BestValidEvidence;
    }
    RouteResultStatus::MalformedOutput
}

/// Failure facts for a router step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepFailure {
    model_label: String,
    repairable: bool,
}

impl StepFailure {
    /// Builds a step failure fact.
    #[must_use]
    pub fn new(model_label: &str, repairable: bool) -> Self {
        Self {
            model_label: String::from(model_label),
            repairable,
        }
    }

    /// Returns the failed model label.
    #[must_use]
    pub fn model_label(&self) -> &str {
        self.model_label.as_str()
    }

    /// Returns whether repair is allowed for this failure.
    #[must_use]
    pub const fn repairable(&self) -> bool {
        self.repairable
    }
}

/// Fallback resolution outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FallbackDecision {
    /// Do not fallback.
    Deny,
    /// Repair the current step.
    Repair(u32),
    /// Fallback to the next eligible chain step.
    FallbackTo(crate::FallbackStep),
    /// Fail the route outright.
    FailRoute(FallbackRouteReason),
}

/// Route-failure reason for fallback resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FallbackRouteReason {
    /// Budget was exhausted.
    BudgetExhausted(crate::BudgetExhausted),
    /// No eligible fallback remained.
    NoEligibleFallback,
    /// The chain revisited a tried model.
    CyclicFallback,
    /// The remaining chain entries were not eligible under policy.
    IneligibleFallback,
}

/// Resolves fallback for a failed step.
#[must_use]
pub fn evaluate_fallback(
    failure: &StepFailure,
    ledger: &crate::BudgetLedger,
    chain: &crate::FallbackChain,
    policy: crate::FallbackPolicy,
) -> FallbackDecision {
    if policy == crate::FallbackPolicy::Deny {
        return FallbackDecision::Deny;
    }

    if failure.repairable() {
        return match ledger.remaining_repair_attempts() {
            0 => FallbackDecision::FailRoute(FallbackRouteReason::BudgetExhausted(crate::BudgetExhausted::new(
                crate::BudgetKind::Repair,
                0,
            ))),
            remaining => FallbackDecision::Repair(remaining),
        };
    }

    if ledger.remaining_candidates() == 0 {
        return FallbackDecision::FailRoute(FallbackRouteReason::BudgetExhausted(crate::BudgetExhausted::new(
            crate::BudgetKind::Candidate,
            0,
        )));
    }

    if ledger.remaining_parallelism() == 0 {
        return FallbackDecision::FailRoute(FallbackRouteReason::BudgetExhausted(crate::BudgetExhausted::new(
            crate::BudgetKind::Parallelism,
            0,
        )));
    }

    let mut has_repeated_model = false;
    let mut has_ineligible_step = false;
    for step in chain.steps() {
        if step.model_label() == failure.model_label()
            || ledger.tried_models().iter().any(|tried| tried.as_str() == step.model_label())
        {
            has_repeated_model = true;
            continue;
        }
        if !policy_allows_step(step, policy) {
            has_ineligible_step = true;
            continue;
        }
        return FallbackDecision::FallbackTo(step.clone());
    }

    if has_repeated_model {
        return FallbackDecision::FailRoute(FallbackRouteReason::CyclicFallback);
    }
    if has_ineligible_step {
        return FallbackDecision::FailRoute(FallbackRouteReason::IneligibleFallback);
    }
    FallbackDecision::FailRoute(FallbackRouteReason::NoEligibleFallback)
}

fn policy_allows_step(step: &crate::FallbackStep, policy: crate::FallbackPolicy) -> bool {
    match policy {
        crate::FallbackPolicy::BestValidEvidence => true,
        crate::FallbackPolicy::SafeSingle => step.is_local(),
        crate::FallbackPolicy::Deny => false,
    }
}
