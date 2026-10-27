//! Authority-free application-host binding projection for one fake-backed pilot.
//!
//! The module validates already-loaded composition identities, commits a projection beside an
//! existing session authority binding, and reuses the runtime-authority seal before dependent
//! effects. It performs no I/O and grants no capability.

mod core;
mod data;

pub use core::{
    admit_pilot_identity, commit_projection, explain_application_host_binding, explain_error,
    prepare_pilot_dispatch, project_application_host_binding, render_kamacite_binding_json,
};
// These aliases preserve the established pilot API while concise names own the implementation.
#[cfg_attr(animus_octet, allow(tigerstyle::renamed_imports))]
pub use core::{
    commit_projection as commit_application_host_binding,
    explain_error as explain_application_host_binding_error,
};
pub use data::{
    KAMACITE_BINDING_RECEIPT_SCHEMA, KAMACITE_BINDING_SCHEMA, KAMACITE_CORE_BLAKE3,
    KAMACITE_PRODUCER_REVISION, KAMACITE_RECEIPT_SCHEMA_BLAKE3, KAMACITE_SCHEMA_BLAKE3,
    KAMACITE_VALIDATION_RECEIPT_BLAKE3, KamaciteContractCohort, NEXT_FOCUSED_CHECK,
    PILOT_BINDING_SCHEMA, PILOT_EXPLANATION_SCHEMA, PilotApplicationHostBinding, PilotBindingDraft,
    PilotCommit, PilotCommitDraft, PilotDispatchAdmission, PilotDispatchDraft,
    PilotEnforcementClass, PilotEnforcementObservation, PilotError, PilotIdentity,
    PilotIdentityDomain, PilotIssue, PilotIssueCode, PilotSensitiveDraft, REQUIRED_NON_CLAIMS,
};
// This alias keeps the existing DTO name without duplicating its definition.
#[cfg_attr(animus_octet, allow(tigerstyle::renamed_imports))]
pub use data::PilotCommit as PilotBindingCommit;

#[cfg(test)]
mod tests;
