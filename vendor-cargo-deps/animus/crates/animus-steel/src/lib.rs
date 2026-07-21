//! Steel Scheme loop fabric for `animus` agent orchestration.
//!
//! This crate keeps the canonical scriptable agent loop separate from the
//! `no_std` core and adapter contracts. Steel programs decide the next loop
//! action while Rust validates every boundary decision before it reaches model
//! or tool adapters.

#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

mod action;
mod error;
mod fabric;
mod policy;
mod quoting;
mod turn;

/// Next action selected by the Steel loop fabric.
pub type LoopDecision = action::Decision;

/// Error returned by Steel loop fabric operations.
pub type LoopError = error::Failure;

/// Persistent Steel-backed loop fabric.
pub type SteelLoopFabric = fabric::Runtime;

/// Checked Steel runtime policy activation inputs.
pub type SteelPolicyActivation<'a> = policy::Activation<'a>;

/// Redacted Steel runtime policy activation receipt.
pub type SteelPolicyActivationReceipt = policy::ActivationReceipt;

/// Safe Steel script selection for policy activation.
pub type SteelPolicyScript<'a> = policy::Script<'a>;

/// Checked Steel runtime policy profile.
pub type SteelPolicyProfile = policy::Profile;

/// Borrowed Steel runtime policy profile data.
pub type SteelPolicyProfileDraft<'a> = policy::ProfileDraft<'a>;

/// Steel policy activation status.
pub type SteelPolicyActivationStatus = policy::ActivationStatus;

/// Steel policy denial reason class.
pub type SteelPolicyDenyReason = policy::DenyReason;

/// Steel policy error.
pub type SteelPolicyError = policy::Error;

/// Steel policy explicit fallback mode.
pub type SteelPolicyFallbackMode = policy::FallbackMode;

/// Steel policy host action.
pub type SteelHostAction = policy::HostAction;

/// Steel policy receipt redaction mode.
pub type SteelPolicyReceiptRedaction = policy::ReceiptRedaction;

pub use policy::DEFAULT_RUNTIME_HOST_ACTIONS;

/// Returns the canonical BLAKE3 hash for a Steel script body.
#[must_use]
pub fn steel_script_hash(source: &str) -> String {
    policy::script_hash(source)
}

/// Inputs made available to the Steel loop for one turn.
pub type LoopTurn = turn::Input;
