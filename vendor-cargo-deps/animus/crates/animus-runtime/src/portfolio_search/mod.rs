//! Pure bounded portfolio-search state, registry, audit, and receipt contracts.
//!
//! Hosts own model calls, tool execution, clocks, files, networks, storage, and
//! validation execution. This module only validates supplied values and applies
//! deterministic state transitions.

mod core;
mod data;

pub use core::begin_adversarial_audit;
pub use core::begin_independent_exploration;
pub use core::begin_synthesis;
pub use core::begin_targeted_redirection;
pub use core::begin_validation;
pub use core::blake3_portfolio_fingerprint;
pub use core::block_approach;
pub use core::block_portfolio;
pub use core::exhaust_portfolio;
pub use core::initialize_portfolio;
pub use core::is_portfolio_budget_exhausted;
pub use core::portfolio_contract_fingerprint;
pub use core::portfolio_evidence_fingerprint;
pub use core::project_portfolio_receipt;
pub use core::record_adversarial_audit;
pub use core::record_portfolio_dispatch;
pub use core::register_approach;
pub use core::reopen_approach;
pub use core::underexplored_live_families;
pub use core::validate_approach_record;
pub use core::validate_portfolio_contract;
pub use core::validate_portfolio_dispatch;
pub use core::validate_portfolio_receipt;
pub use core::validate_portfolio_result;

pub use data::ApproachRecord;
pub use data::ApproachState;
pub use data::AuditOutcome;
pub use data::GapStrength;
pub use data::PortfolioAudit;
pub use data::PortfolioAuditFinding;
pub use data::PortfolioAuditRisk;
pub use data::PortfolioBlocker;
pub use data::PortfolioBlockerClass;
pub use data::PortfolioBudget;
pub use data::PortfolioContract;
pub use data::PortfolioDispatch;
pub use data::PortfolioError;
pub use data::PortfolioErrorKind;
pub use data::PortfolioFingerprint;
pub use data::PortfolioPhase;
pub use data::PortfolioReceipt;
pub use data::PortfolioResult;
pub use data::PortfolioState;
pub use data::PortfolioUsage;
pub use data::PortfolioValidationReceipt;
pub use data::PortfolioValidationStatus;
pub use data::ReopenNovelty;

pub(crate) const BLAKE3_ALGORITHM: &str = "BLAKE3";
pub(crate) const PORTFOLIO_HASH_DOMAIN: &str = "animus.portfolio-search.v1";
pub(crate) const REQUIRED_COUNT: u32 = 1;
pub(crate) const MAX_SAFE_LABEL_CHARS: usize = 256;
pub(crate) const MAX_REFERENCE_COUNT: usize = 64;
pub(crate) const BLAKE3_HEX_LENGTH: usize = 64;

#[cfg(test)]
#[path = "tests.rs"]
mod portfolio_search_tests;
