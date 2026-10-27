mod auth;
mod backend;
mod input;
mod policy;
mod record;
mod taxonomy;
#[cfg(test)]
mod tests;

pub(super) type CredentialEvidence = auth::CredentialEvidence;
pub(super) type CredentialEvidenceDraft<'a> = auth::CredentialEvidenceDraft<'a>;
/// Credential expiry status supplied by the host verifier.
pub type CredentialExpiry = auth::CredentialExpiry;
/// Credential verifier status supplied by a host-owned verifier backend.
pub type CredentialVerifierStatus = auth::CredentialVerifierStatus;
pub(super) type AdmissionRequest = input::AdmissionRequest;
pub(super) type AdmissionRequestDraft<'a> = input::AdmissionRequestDraft<'a>;
pub(super) type AdmissionDecision = policy::AdmissionDecision;
pub(super) type AdmissionError = policy::AdmissionError;
pub(super) type AdmissionReceipt = record::AdmissionReceipt;
pub(super) type Action = taxonomy::Action;
pub(super) type AdmissionCaveat = taxonomy::AdmissionCaveat;
pub(super) type AdmissionReason = taxonomy::AdmissionReason;
pub(super) type AdmissionVerdict = taxonomy::AdmissionVerdict;
pub(super) type Entrypoint = taxonomy::Entrypoint;

/// Remote or headless entrypoint requesting admission.
pub type RemoteEntrypoint = Entrypoint;
/// Entry action requiring admission before side effects.
pub type RemoteAction = Action;
/// Local caveat result for the admission request.
pub type RemoteAdmissionCaveat = AdmissionCaveat;
/// Admission verdict.
pub type RemoteAdmissionVerdict = AdmissionVerdict;
/// Admission denial reason.
pub type RemoteAdmissionReason = AdmissionReason;
/// Borrowed credential evidence before validation.
pub type RemoteCredentialEvidenceDraft<'a> = CredentialEvidenceDraft<'a>;
/// Validated credential evidence.
pub type RemoteCredentialEvidence = CredentialEvidence;
/// Borrowed remote admission request before validation.
pub type RemoteAdmissionRequestDraft<'a> = AdmissionRequestDraft<'a>;
/// Validated remote admission request.
pub type RemoteAdmissionRequest = AdmissionRequest;
/// Admission decision and safe receipt.
pub type RemoteAdmissionDecision = AdmissionDecision;
/// Safe admission receipt metadata.
pub type RemoteAdmissionReceipt = AdmissionReceipt;
/// Admission validation error.
pub type RemoteAdmissionError = AdmissionError;

pub use backend::{
    REMOTE_SERVER_RECEIPT_LEAK_MARKERS, RemoteServerAdmissionDecision, RemoteServerAdmissionError,
    RemoteServerAdmissionOutcome, RemoteServerAdmissionPolicy, RemoteServerAdmissionPolicyDraft,
    RemoteServerAdmissionReason, RemoteServerAdmissionReceipt, RemoteServerAdmissionRequest,
    RemoteServerAuthOutcome, RemoteServerCredentialSourceAvailability, RemoteServerDescriptor,
    RemoteServerDescriptorDraft, RemoteServerEndpointIdentityClass, RemoteServerFailureClass,
    RemoteServerFingerprintPolicy, RemoteServerFingerprintStatus, RemoteServerHandshakeSummary,
    RemoteServerHandshakeSummaryDraft, RemoteServerObservedTls, RemoteServerPlatformRole,
    RemoteServerProtocolRole, RemoteServerProtocolVersionStatus, RemoteServerSecretSourceClass,
    RemoteServerTlsRequirement, RemoteServerTransportKind, evaluate_remote_server_admission,
    validate_remote_server_admission_receipt_text,
};

pub(super) const AUDIENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_admission_audience");
pub(super) const ACTOR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_admission_actor");
pub(super) const SCOPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_admission_scope");
pub(super) const RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_admission_resource");
pub(super) const ABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("remote_admission_ability");

const LEAK_RAW_TOKEN: &str = "raw_token";
const LEAK_PRIVATE_KEY: &str = "private_key";
const LEAK_PROMPT_BODY: &str = "prompt_body";
const LEAK_PROVIDER_JSON: &str = "provider_json";
const LEAK_TRANSPORT_HEADER: &str = "authorization:";
const LEAK_MATRIX_ROOM: &str = "!matrix-room";
const LEAK_FULL_PATH: &str = "/home/";
const LEAK_CREDENTIAL_BODY: &str = "credential_body";

pub(super) const LEAK_MARKERS: &[&str] = &[
    LEAK_RAW_TOKEN,
    LEAK_PRIVATE_KEY,
    LEAK_PROMPT_BODY,
    LEAK_PROVIDER_JSON,
    LEAK_TRANSPORT_HEADER,
    LEAK_MATRIX_ROOM,
    LEAK_FULL_PATH,
    LEAK_CREDENTIAL_BODY,
];

#[cfg(test)]
pub(super) fn admit(request: &AdmissionRequest) -> Result<AdmissionDecision, AdmissionError> {
    policy::admit(request)
}

#[cfg(test)]
pub(super) fn validate_safe_text(text: &str) -> Result<(), AdmissionError> {
    policy::validate_safe_text(text)
}

/// Evaluates a remote admission request before side effects.
///
/// # Errors
///
/// Returns [`RemoteAdmissionError`] if safe receipt identity construction fails.
pub fn evaluate_remote_admission(
    request: &RemoteAdmissionRequest,
) -> Result<RemoteAdmissionDecision, RemoteAdmissionError> {
    policy::admit(request)
}

/// Rejects unsafe rendered admission receipt text.
///
/// # Errors
///
/// Returns [`RemoteAdmissionError`] when text contains a known unsafe marker.
pub fn validate_remote_admission_receipt_text(text: &str) -> Result<(), RemoteAdmissionError> {
    policy::validate_safe_text(text)
}

fn normalized(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<alloc::string::String, crate::AdapterContractError> {
    crate::validation::normalized_adapter_field(field, value)
}
