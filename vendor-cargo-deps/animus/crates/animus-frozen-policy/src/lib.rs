#![no_std]
#![forbid(unsafe_code)]
//! Frozen local selection policies: artifact admission, fixed-point evaluation, and receipts.
//!
//! A frozen policy answers one bounded question: given a finite vector of integer features, which
//! of a declared, finite set of actions should a consumer prefer? The crate is the shared
//! contract and runtime leaf of Animus's System One work. It has two consumers: the campaign
//! manager's evaluation lane and Lattice's optional typed policy operation.
//!
//! # One decision
//!
//! 1. [`admit_policy`] parses an [`ARTIFACT_SCHEMA`] artifact the host has already loaded, checks
//!    every contract and bound, and admits it only under the caller's selected `b3:` identity. A
//!    refused artifact is a typed [`AdmissionError`]; nothing is scored.
//! 2. [`decide`] checks the caller's [`PolicyBinding`] against the admitted artifact, validates the
//!    request's features and eligible actions, normalizes the features, evaluates the frozen
//!    decision tree or small MLP in checked integer arithmetic, and selects the best eligible
//!    action. Every refusal is a typed [`Failure`]; a declared fallback action answers instead,
//!    and the receipt says so.
//! 3. Every decision carries a bounded [`DecisionReceipt`] whose canonical encoding has its own
//!    `b3:` identity. [`verify_receipt`] re-admits a retained receipt and returns its decided
//!    outcome, so replay never re-runs inference.
//!
//! # Authority
//!
//! The crate reads no files, environment, clocks, randomness, or network, and it never trains or
//! updates weights. Hosts load artifact bytes, enforce wall-clock deadlines, and persist
//! receipts. A selected action is a preference among actions the caller already declared
//! eligible; it is neither task success nor authorization.
// r[impl agent_harness_runtime.frozen_policy.boundary]

extern crate alloc;

mod artifact;
mod digest;
mod inference;
mod record;
mod selection;

pub use artifact::{
    ARTIFACT_API, ARTIFACT_SCHEMA, AdmissionError, AdmittedPolicy, FeatureBound, MAX_ACTIONS,
    MAX_ARTIFACT_BYTES, MAX_FEATURE_MAGNITUDE, MAX_FEATURES, MAX_LABEL_BYTES, MAX_LAYER_WIDTH,
    MAX_LAYERS, MAX_OPERATION_BUDGET, MAX_PARAMETER_MAGNITUDE, MAX_TREE_DEPTH, MAX_TREE_NODES,
    MLP_POLICY_KIND, MODEL_DOMAIN, NORMALIZATION, RUNTIME_CRATE, TREE_POLICY_KIND, admit_policy,
    policy_identity,
};
pub use digest::{BLAKE3_PREFIX, domain_identity, is_b3_identity, is_safe_identity};
pub use inference::PER_MILLE;
pub use record::{
    ACTION_SET_DOMAIN, CostUnits, DecisionReceipt, INPUT_DOMAIN, MAX_RECEIPT_BYTES, RECEIPT_SCHEMA,
    ReceiptBody, ReceiptError, ReceiptOutcome, verify_receipt,
};
pub use selection::{
    Decision, DecisionRequest, Failure, InputFault, Outcome, PolicyBinding, StaleField, decide,
};
