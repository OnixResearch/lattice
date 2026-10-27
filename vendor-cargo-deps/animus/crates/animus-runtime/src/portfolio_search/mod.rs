//! Pure bounded portfolio-search state, registry, audit, and receipt contracts.
//!
//! Hosts own model calls, tool execution, clocks, files, networks, storage, and
//! validation execution. This module only validates supplied values and applies
//! deterministic state transitions.

mod core;
mod data;

pub use core::{
    begin_adversarial_audit, begin_independent_exploration, begin_synthesis,
    begin_targeted_redirection, begin_validation, blake3_portfolio_fingerprint, block_approach,
    block_portfolio, exhaust_portfolio, initialize_portfolio, is_portfolio_budget_exhausted,
    portfolio_contract_fingerprint, portfolio_evidence_fingerprint, project_portfolio_receipt,
    record_adversarial_audit, record_portfolio_dispatch, register_approach, reopen_approach,
    underexplored_live_families, validate_approach_record, validate_portfolio_contract,
    validate_portfolio_dispatch, validate_portfolio_receipt, validate_portfolio_result,
};
pub use data::{
    ApproachRecord, ApproachState, AuditOutcome, GapStrength, PortfolioAudit,
    PortfolioAuditFinding, PortfolioAuditRisk, PortfolioBlocker, PortfolioBlockerClass,
    PortfolioBudget, PortfolioContract, PortfolioDispatch, PortfolioError, PortfolioErrorKind,
    PortfolioFingerprint, PortfolioPhase, PortfolioReceipt, PortfolioResult, PortfolioState,
    PortfolioUsage, PortfolioValidationReceipt, PortfolioValidationStatus, ReopenNovelty,
};

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const PORTFOLIO_HASH_DOMAIN: &str = "animus.portfolio-search.v1";
pub(crate) const REQUIRED_COUNT: u32 = 1;
pub(crate) const MAX_SAFE_LABEL_CHARS: usize = 256;
pub(crate) const MAX_REFERENCE_COUNT: usize = 64;
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;

#[cfg(test)]
#[path = "tests.rs"]
mod portfolio_search_tests;
