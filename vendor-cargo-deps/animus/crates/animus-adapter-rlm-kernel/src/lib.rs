//! Pure no-std contracts for the optional Animus model-facing RLM kernel.
//!
//! This crate owns bounded policy, persistent environment metadata, typed host
//! requests, child-spawn admission, compaction preservation, and metadata-only
//! receipts. It performs no evaluation or host effect. `animus-steel` owns the
//! Steel evaluator, while host shells own files, processes, networks, clocks,
//! credentials, persistence, tools, and child lifecycle.

#![no_std]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::module_name_repetitions)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::ToString;

mod child;
mod environment;
mod policy;
mod receipt;
mod request;

pub use child::ChildResult;
pub use child::ChildSpawnRequest;
pub use child::Lineage;
pub use child::admit_child_spawn;
pub use environment::CompactionCause;
pub use environment::CompactionRequest;
pub use environment::ContextBinding;
pub use environment::DefinitionCommit;
pub use environment::EnvironmentState;
pub use environment::EvaluationMetrics;
pub use environment::EvaluationPlan;
pub use environment::EvaluationRequest;
pub use environment::KernelCompactionPlan;
pub use environment::bind_prompt_context;
pub use environment::commit_evaluation;
pub use environment::plan_compaction;
pub use environment::plan_evaluation;
pub use policy::REQUIRED_NON_CLAIMS;
pub use policy::RLM_KERNEL_POLICY_SCHEMA;
pub use policy::ResourceBounds;
pub use policy::RlmKernelPolicy;
pub use policy::RlmKernelPolicyDraft;
pub use receipt::Receipt;
pub use receipt::ReceiptCount;
pub use receipt::ReceiptDraft;
pub use receipt::ReceiptLeakClass;
pub use receipt::RlmKernelStatus;
pub use request::HostOperationClass;
pub use request::HostRequest;
pub use request::HostRequestAdmission;
pub use request::admit_host_request;

/// Stable schema for the model-facing Steel tool request.
pub const STEEL_REPL_TOOL_SCHEMA: &str = "animus.steel-repl.v1";
/// Stable model-facing Steel tool name.
pub const STEEL_REPL_TOOL_NAME: &str = "steel_repl";
/// Stable capability label required for model-facing Steel evaluation.
pub const STEEL_REPL_CAPABILITY: &str = "steel.evaluate.model-facing";
/// Stable schema for RLM kernel receipts.
pub const RLM_KERNEL_RECEIPT_SCHEMA: &str = "animus.rlm-kernel-receipt.v1";

/// Bounded typed contract error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A required text field was empty.
    EmptyField {
        /// Stable field name.
        field: &'static str,
    },
    /// A text field contains a NUL character.
    NulCharacter {
        /// Stable field name.
        field: &'static str,
    },
    /// A text field exceeded its named character limit.
    TextTooLong {
        /// Stable field name.
        field: &'static str,
        /// Maximum admitted character count.
        max_chars: u32,
        /// Observed character count.
        actual_chars: u32,
    },
    /// A required numeric bound was zero.
    ZeroBound {
        /// Stable bound name.
        field: &'static str,
    },
    /// A numeric value exceeded its named limit.
    BoundExceeded {
        /// Stable bound name.
        field: &'static str,
        /// Maximum admitted value.
        max: u64,
        /// Observed value.
        actual: u64,
    },
    /// A Git cohort revision was malformed.
    InvalidCohortRevision,
    /// A BLAKE3 identity was malformed.
    InvalidBlake3Identity {
        /// Stable identity field.
        field: &'static str,
    },
    /// A policy omitted an operation class.
    OperationNotAdmitted {
        /// Denied operation class.
        operation: HostOperationClass,
    },
    /// A required policy non-claim was absent.
    MissingNonClaim {
        /// Required non-claim label.
        non_claim: &'static str,
    },
    /// A collection contains duplicate identities.
    DuplicateIdentity {
        /// Stable collection field.
        field: &'static str,
    },
    /// A context binding name is not a safe Steel identifier.
    InvalidBindingName,
    /// A child profile is not in the admitted profile set.
    ChildProfileNotAdmitted,
    /// A child spawn expects ambient discovery or cross-session registration.
    AmbientChildSpawnDenied,
    /// An implicit kernel clear was requested.
    ImplicitKernelClearDenied,
    /// A receipt draft includes prohibited content.
    ReceiptLeak {
        /// Prohibited content class.
        class: ReceiptLeakClass,
    },
    /// A receipt draft includes a prohibited claim.
    ProhibitedClaim,
    /// A receipt contradicts its status or preservation facts.
    InconsistentReceipt,
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "{field} must not be empty"),
            Self::NulCharacter { field } => write!(formatter, "{field} contains a NUL character"),
            Self::TextTooLong {
                field,
                max_chars,
                actual_chars,
            } => write!(formatter, "{field} must contain at most {max_chars} characters; got {actual_chars}"),
            Self::ZeroBound { field } => write!(formatter, "{field} must be positive"),
            Self::BoundExceeded { field, max, actual } => {
                write!(formatter, "{field} must be at most {max}; got {actual}")
            }
            Self::InvalidCohortRevision => {
                formatter.write_str("cohort revision must be lowercase 40-character Git hex")
            }
            Self::InvalidBlake3Identity { field } => {
                write!(formatter, "{field} must be lowercase 64-character BLAKE3 hex")
            }
            Self::OperationNotAdmitted { operation } => {
                write!(formatter, "operation {} is not admitted", operation.as_str())
            }
            Self::MissingNonClaim { non_claim } => {
                write!(formatter, "required non-claim {non_claim} is missing")
            }
            Self::DuplicateIdentity { field } => write!(formatter, "{field} contains duplicates"),
            Self::InvalidBindingName => formatter.write_str("binding name is not a safe Steel identifier"),
            Self::ChildProfileNotAdmitted => formatter.write_str("child profile is not admitted"),
            Self::AmbientChildSpawnDenied => formatter.write_str("ambient child spawn is denied"),
            Self::ImplicitKernelClearDenied => formatter.write_str("implicit kernel clear is denied"),
            Self::ReceiptLeak { class } => {
                write!(formatter, "receipt contains prohibited {} content", class.as_str())
            }
            Self::ProhibitedClaim => formatter.write_str("receipt contains a prohibited claim"),
            Self::InconsistentReceipt => formatter.write_str("receipt facts are inconsistent"),
        }
    }
}

pub(crate) fn required_text(field: &'static str, value: &str, max_chars: u32) -> Result<alloc::string::String, Error> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(Error::EmptyField { field });
    }
    if normalized.contains('\0') {
        return Err(Error::NulCharacter { field });
    }
    let actual_chars = usize_to_u32(normalized.chars().count());
    if actual_chars > max_chars {
        return Err(Error::TextTooLong {
            field,
            max_chars,
            actual_chars,
        });
    }
    Ok(alloc::string::String::from(normalized))
}

pub(crate) fn ensure_positive(field: &'static str, value: u64) -> Result<(), Error> {
    if value == 0 {
        return Err(Error::ZeroBound { field });
    }
    Ok(())
}

pub(crate) fn ensure_at_most(field: &'static str, value: u64, max: u64) -> Result<(), Error> {
    if value > max {
        return Err(Error::BoundExceeded {
            field,
            max,
            actual: value,
        });
    }
    Ok(())
}

pub(crate) fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

pub(crate) fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

pub(crate) fn is_lower_hex(value: &str, expected_chars: usize) -> bool {
    value.len() == expected_chars && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn hash_parts(domain: &str, parts: &[&str]) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let length = usize_to_u64(part.len()).to_le_bytes();
        hasher.update(&length);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}
