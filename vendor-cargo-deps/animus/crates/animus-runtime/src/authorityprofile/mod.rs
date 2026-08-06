//! Pure metadata-only runtime authority profile and durable binding contracts.

mod core;
mod data;
mod evidence;

pub use core::build_session_authority_binding;
pub use core::build_session_authority_enforcement_observation;
pub use core::build_session_authority_profile;
pub use core::compare_session_authority_profiles;
pub use core::decode_session_authority_binding;
pub use core::encode_session_authority_binding;
pub use core::plan_session_authority_transition;
pub use core::validate_session_authority_enforcement;
pub use core::validate_session_authority_profile;

pub use data::SESSION_AUTHORITY_BINDING_HASH_DOMAIN;
pub use data::SESSION_AUTHORITY_BINDING_SCHEMA;
pub use data::SESSION_AUTHORITY_ENFORCEMENT_HASH_DOMAIN;
pub use data::SESSION_AUTHORITY_PROFILE_HASH_DOMAIN;
pub use data::SESSION_AUTHORITY_PROFILE_SCHEMA;
pub use data::SESSION_AUTHORITY_RECEIPT_HASH_DOMAIN;
pub use data::SESSION_AUTHORITY_RECEIPT_SCHEMA;
pub use data::SessionAuthorityBinding;
pub use data::SessionAuthorityBindingDraft;
pub use data::SessionAuthorityComparison;
pub use data::SessionAuthorityComponent;
pub use data::SessionAuthorityComponentClass;
pub use data::SessionAuthorityComponentDraft;
pub use data::SessionAuthorityEnforcementDraft;
pub use data::SessionAuthorityEnforcementObservation;
pub use data::SessionAuthorityError;
pub use data::SessionAuthorityIssue;
pub use data::SessionAuthorityProfileDraft;
pub use data::SessionAuthorityReceipt;
pub use data::SessionAuthorityReceiptDraft;
pub use data::SessionAuthorityResumeStatus;
pub use data::SessionAuthoritySensitiveDraft;
pub use data::SessionAuthorityTransitionAction;
pub use data::SessionAuthorityTransitionDisposition;
pub use data::SessionAuthorityTransitionPlan;
pub use data::SessionAuthorityTransitionRequest;
pub use data::SessionRuntimeAuthorityProfile;
pub use evidence::project_session_authority_receipt;

#[cfg(test)]
mod tests;
