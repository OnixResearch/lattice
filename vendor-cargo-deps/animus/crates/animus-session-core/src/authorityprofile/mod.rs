//! Pure metadata-only runtime authority profile and durable binding contracts.

#[allow(unused_imports)]
use crate::prelude::*;

mod core;
mod data;
mod evidence;

pub use core::{
    build_session_authority_binding, build_session_authority_enforcement_observation,
    build_session_authority_profile, compare_session_authority_profiles,
    decode_session_authority_binding, encode_session_authority_binding,
    plan_session_authority_transition, validate_session_authority_enforcement,
    validate_session_authority_profile,
};
pub use data::{
    SESSION_AUTHORITY_BINDING_HASH_DOMAIN, SESSION_AUTHORITY_BINDING_SCHEMA,
    SESSION_AUTHORITY_ENFORCEMENT_HASH_DOMAIN, SESSION_AUTHORITY_PROFILE_HASH_DOMAIN,
    SESSION_AUTHORITY_PROFILE_SCHEMA, SESSION_AUTHORITY_RECEIPT_HASH_DOMAIN,
    SESSION_AUTHORITY_RECEIPT_SCHEMA, SessionAuthorityBinding, SessionAuthorityBindingDraft,
    SessionAuthorityComparison, SessionAuthorityComponent, SessionAuthorityComponentClass,
    SessionAuthorityComponentDraft, SessionAuthorityEnforcementDraft,
    SessionAuthorityEnforcementObservation, SessionAuthorityError, SessionAuthorityIssue,
    SessionAuthorityProfileDraft, SessionAuthorityReceipt, SessionAuthorityReceiptDraft,
    SessionAuthorityResumeStatus, SessionAuthoritySensitiveDraft, SessionAuthorityTransitionAction,
    SessionAuthorityTransitionDisposition, SessionAuthorityTransitionPlan,
    SessionAuthorityTransitionRequest, SessionRuntimeAuthorityProfile,
};
pub use evidence::project_session_authority_receipt;

#[cfg(test)]
mod tests;
