//! Pure per-dispatch runtime-authority sealing and bounded receipt contracts.

#[allow(unused_imports)]
use crate::prelude::*;

mod core;
mod data;
mod evidence;

pub use core::{
    commit_runtime_authority_seal, prepare_runtime_authority_seal, start_runtime_authority_effect,
    transition_runtime_authority_unit, validate_runtime_authority_narrowing,
    validate_runtime_authority_recheck,
};
pub use data::{
    MAX_RUNTIME_AUTHORITY_OPERATIONS, MAX_RUNTIME_AUTHORITY_RESOURCES, MAX_RUNTIME_AUTHORITY_VIEWS,
    RUNTIME_AUTHORITY_SEAL_HASH_DOMAIN, RUNTIME_AUTHORITY_SEAL_RECEIPT_HASH_DOMAIN,
    RUNTIME_AUTHORITY_SEAL_RECEIPT_SCHEMA, RUNTIME_AUTHORITY_SEAL_SCHEMA,
    RuntimeAuthorityExpectedViewDraft, RuntimeAuthorityIssue, RuntimeAuthorityIssueCode,
    RuntimeAuthorityMutationKind, RuntimeAuthorityObservationStatus,
    RuntimeAuthorityObservedViewDraft, RuntimeAuthorityReceiptDraft, RuntimeAuthorityRecheckDraft,
    RuntimeAuthoritySealError, RuntimeAuthoritySealPlan, RuntimeAuthoritySealReceipt,
    RuntimeAuthoritySensitiveDraft, RuntimeAuthorityTransition, RuntimeAuthorityUnit,
    RuntimeAuthorityUnitDraft, RuntimeAuthorityUnitState, RuntimeAuthorityViewClass,
    RuntimeAuthorityViewEntry,
};
pub use evidence::{
    project_runtime_authority_seal_receipt, validate_runtime_authority_seal_receipt,
};

#[cfg(test)]
mod tests;
