//! `Ratatui` feature policy validation.
//!
//! The policy core is deterministic and renderer-neutral. It validates checked-in
//! policy rows and separately validates reported feature uses against those rows,
//! without reading manifests or invoking Cargo.

mod check;
mod seeds;
mod types;

pub use check::{validate_ratatui_feature_policy, validate_ratatui_feature_uses};
pub use seeds::baseline_ratatui_feature_policy_rows;
pub use types::{
    FeaturePolicyIssue, FeaturePolicyIssueCode, FeaturePolicyReport, FeaturePolicyRow,
    FeaturePolicyRowParts, FeatureUse, FeatureUseTarget,
};

#[cfg(test)]
#[path = "../feature_policy_tests.rs"]
mod tests;
