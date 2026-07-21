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
    output_contract: super::OutputContract,
    repair_attempts_used: u32,
}

impl RouteResult {
    /// Builds a validated route result from final-output validation facts.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when output-contract labels are invalid.
    pub fn new(draft: ResultDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let output_contract = super::OutputContract::parse(draft.output_contract)?;
        let fallback_policy = super::FallbackPolicy::parse(draft.fallback_policy)?;
        let status = result_status(
            draft.final_output_valid,
            fallback_policy,
            draft.best_valid_evidence_available,
        );

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
    pub const fn output_contract(&self) -> super::OutputContract {
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
    fallback_policy: super::FallbackPolicy,
    best_valid_evidence_available: bool,
) -> RouteResultStatus {
    if final_output_valid {
        return RouteResultStatus::Success;
    }
    if fallback_policy == super::FallbackPolicy::BestValidEvidence && best_valid_evidence_available
    {
        return RouteResultStatus::BestValidEvidence;
    }
    RouteResultStatus::MalformedOutput
}
