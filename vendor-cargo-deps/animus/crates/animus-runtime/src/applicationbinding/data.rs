//! Typed data for the non-default application-host binding pilot.

#![allow(missing_docs)]

pub const PILOT_BINDING_SCHEMA: &str = "animus.application-host-binding-pilot.v1";
pub const KAMACITE_BINDING_SCHEMA: &str = "kamacite.application-host-binding.v1";
pub const KAMACITE_BINDING_RECEIPT_SCHEMA: &str = "kamacite.application-host-binding-receipt.v1";
pub const KAMACITE_PRODUCER_REVISION: &str = "b21732a7458b660367f7281c717a5acc3f8c8190";
pub const KAMACITE_SCHEMA_BLAKE3: &str =
    "34fdebab3c8a274729f364af8da58691dba38809c67d2643c1f0d0354cb0c80b";
pub const KAMACITE_RECEIPT_SCHEMA_BLAKE3: &str =
    "3b48bf894cf40990a6d857542e9c3b26df2f6b50510999698323a7cb5cbca39e";
pub const KAMACITE_VALIDATION_RECEIPT_BLAKE3: &str =
    "75ac779da7b5fde841188ba326e81c253ba5b78dbf0cc632bc90f8b58a9fb303";
pub const KAMACITE_CORE_BLAKE3: &str =
    "4912fa0d0b81c5ac526d6fd762f4fe83d6d7dbba21cb661f3724bb499af5a1db";
pub const PILOT_BINDING_HASH_DOMAIN: &str = "animus.application-host-binding-pilot.v1";
pub const PILOT_COMMIT_HASH_DOMAIN: &str = "animus.application-host-binding-commit.v1";
pub const PILOT_EXPLANATION_SCHEMA: &str = "animus.application-host-binding-explanation.v1";
pub const HASH_HEX_CHARS: usize = 64;
pub const MAX_EFFECT_OPERATIONS: usize = 16;
pub const REQUIRED_NON_CLAIM_COUNT: usize = 9;
pub const REQUIRED_ENFORCEMENT_CLASS_COUNT: usize = 8;
pub const MAX_PILOT_ISSUES: usize = 64;
pub const FIRST_SEQUENCE: u64 = 1;
pub const HASH_SEPARATOR: &[u8] = b"\x1f";
pub const NEXT_FOCUSED_CHECK: &str = "scripts/cairn/check-application-host-binding --root .";
pub const REDACTION_MARKER: &str = "prompts-policy-bodies-tool-payloads-credentials-endpoints-environment-values-and-host-paths-omitted";

pub const REQUIRED_NON_CLAIMS: [&str; REQUIRED_NON_CLAIM_COUNT] = [
    "not effect completeness",
    "not application correctness",
    "not host capability or authorization",
    "not host behavior",
    "not sandbox admission",
    "not provider correctness",
    "not tool safety",
    "not runtime success",
    "not release eligibility",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PilotIdentityDomain {
    TargetArtifact,
    Adapter,
    AlgebraicEffect,
    EffectOperation,
    Evaluator,
    EvidenceChain,
    Projection,
    ApplicationHostBinding,
    Receipt,
}

impl PilotIdentityDomain {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TargetArtifact => "target-artifact",
            Self::Adapter => "adapter",
            Self::AlgebraicEffect => "algebraic-effect",
            Self::EffectOperation => "effect-operation",
            Self::Evaluator => "evaluator",
            Self::EvidenceChain => "evidence-chain",
            Self::Projection => "projection",
            Self::ApplicationHostBinding => "application-host-binding",
            Self::Receipt => "receipt",
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PilotIdentity {
    domain: PilotIdentityDomain,
    hex: String,
}

impl PilotIdentity {
    #[must_use]
    pub const fn domain(&self) -> PilotIdentityDomain {
        self.domain
    }

    #[must_use]
    pub fn hex(&self) -> &str {
        self.hex.as_str()
    }

    pub(super) fn new(domain: PilotIdentityDomain, hex: String) -> Self {
        Self { domain, hex }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KamaciteContractCohort<'a> {
    pub binding_schema: &'a str,
    pub receipt_schema: &'a str,
    pub producer_revision: &'a str,
    pub schema_blake3: &'a str,
    pub receipt_schema_blake3: &'a str,
    pub validation_receipt_blake3: &'a str,
    pub core_blake3: &'a str,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PilotSensitiveDraft<'a> {
    pub prompt: Option<&'a str>,
    pub policy_body: Option<&'a str>,
    pub tool_payload: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub endpoint: Option<&'a str>,
    pub environment_value: Option<&'a str>,
    pub host_path: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PilotBindingDraft<'a> {
    pub cohort: KamaciteContractCohort<'a>,
    pub application_identity: &'a PilotIdentity,
    pub entrypoint_identity: &'a PilotIdentity,
    pub input_schema_identity: &'a PilotIdentity,
    pub output_schema_identity: &'a PilotIdentity,
    pub effect_summary_identity: &'a PilotIdentity,
    pub effect_summary_closed: bool,
    pub effect_operation_identities: &'a [PilotIdentity],
    pub host_profile_identities: &'a [PilotIdentity],
    pub adapter_cohort_identity: &'a PilotIdentity,
    pub resource_profile_identity: &'a PilotIdentity,
    pub policy_identity: &'a PilotIdentity,
    pub compatibility_identity: &'a PilotIdentity,
    pub expected_adapter_cohort_identity: &'a PilotIdentity,
    pub expected_resource_profile_identity: &'a PilotIdentity,
    pub expected_host_profile_identity: &'a PilotIdentity,
    pub non_claims: &'a [&'a str],
    pub semantic_claims: &'a [&'a str],
    pub sensitive: PilotSensitiveDraft<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PilotApplicationHostBinding {
    schema: &'static str,
    kamacite_schema: &'static str,
    application_identity: PilotIdentity,
    entrypoint_identity: PilotIdentity,
    input_schema_identity: PilotIdentity,
    output_schema_identity: PilotIdentity,
    effect_summary_identity: PilotIdentity,
    effect_operation_identities: Vec<PilotIdentity>,
    host_profile_identity: PilotIdentity,
    adapter_cohort_identity: PilotIdentity,
    resource_profile_identity: PilotIdentity,
    policy_identity: PilotIdentity,
    compatibility_identity: PilotIdentity,
    binding_identity: PilotIdentity,
    non_claims: Vec<String>,
}

impl PilotApplicationHostBinding {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    #[must_use]
    pub const fn kamacite_schema(&self) -> &'static str {
        self.kamacite_schema
    }

    #[must_use]
    pub const fn application_identity(&self) -> &PilotIdentity {
        &self.application_identity
    }

    #[must_use]
    pub const fn entrypoint_identity(&self) -> &PilotIdentity {
        &self.entrypoint_identity
    }

    #[must_use]
    pub const fn input_schema_identity(&self) -> &PilotIdentity {
        &self.input_schema_identity
    }

    #[must_use]
    pub const fn output_schema_identity(&self) -> &PilotIdentity {
        &self.output_schema_identity
    }

    #[must_use]
    pub const fn effect_summary_identity(&self) -> &PilotIdentity {
        &self.effect_summary_identity
    }

    #[must_use]
    pub fn effect_operation_identities(&self) -> &[PilotIdentity] {
        self.effect_operation_identities.as_slice()
    }

    #[must_use]
    pub const fn host_profile_identity(&self) -> &PilotIdentity {
        &self.host_profile_identity
    }

    #[must_use]
    pub const fn adapter_cohort_identity(&self) -> &PilotIdentity {
        &self.adapter_cohort_identity
    }

    #[must_use]
    pub const fn resource_profile_identity(&self) -> &PilotIdentity {
        &self.resource_profile_identity
    }

    #[must_use]
    pub const fn policy_identity(&self) -> &PilotIdentity {
        &self.policy_identity
    }

    #[must_use]
    pub const fn compatibility_identity(&self) -> &PilotIdentity {
        &self.compatibility_identity
    }

    #[must_use]
    pub const fn binding_identity(&self) -> &PilotIdentity {
        &self.binding_identity
    }

    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.non_claims.as_slice()
    }

    pub(super) fn from_parts(parts: PilotBindingParts) -> Self {
        Self {
            schema: PILOT_BINDING_SCHEMA,
            kamacite_schema: KAMACITE_BINDING_SCHEMA,
            application_identity: parts.application_identity,
            entrypoint_identity: parts.entrypoint_identity,
            input_schema_identity: parts.input_schema_identity,
            output_schema_identity: parts.output_schema_identity,
            effect_summary_identity: parts.effect_summary_identity,
            effect_operation_identities: parts.effect_operation_identities,
            host_profile_identity: parts.host_profile_identity,
            adapter_cohort_identity: parts.adapter_cohort_identity,
            resource_profile_identity: parts.resource_profile_identity,
            policy_identity: parts.policy_identity,
            compatibility_identity: parts.compatibility_identity,
            binding_identity: parts.binding_identity,
            non_claims: parts.non_claims,
        }
    }
}

pub(super) struct PilotBindingParts {
    pub application_identity: PilotIdentity,
    pub entrypoint_identity: PilotIdentity,
    pub input_schema_identity: PilotIdentity,
    pub output_schema_identity: PilotIdentity,
    pub effect_summary_identity: PilotIdentity,
    pub effect_operation_identities: Vec<PilotIdentity>,
    pub host_profile_identity: PilotIdentity,
    pub adapter_cohort_identity: PilotIdentity,
    pub resource_profile_identity: PilotIdentity,
    pub policy_identity: PilotIdentity,
    pub compatibility_identity: PilotIdentity,
    pub binding_identity: PilotIdentity,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PilotEnforcementClass {
    Authorization,
    CapabilityProbe,
    AdapterReadiness,
    CredentialAdmission,
    SandboxPreflight,
    Cancellation,
    Timeout,
    ResourceEnforcement,
}

impl PilotEnforcementClass {
    pub const ALL: [Self; REQUIRED_ENFORCEMENT_CLASS_COUNT] = [
        Self::Authorization,
        Self::CapabilityProbe,
        Self::AdapterReadiness,
        Self::CredentialAdmission,
        Self::SandboxPreflight,
        Self::Cancellation,
        Self::Timeout,
        Self::ResourceEnforcement,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authorization => "authorization",
            Self::CapabilityProbe => "capability-probe",
            Self::AdapterReadiness => "adapter-readiness",
            Self::CredentialAdmission => "credential-admission",
            Self::SandboxPreflight => "sandbox-preflight",
            Self::Cancellation => "cancellation",
            Self::Timeout => "timeout",
            Self::ResourceEnforcement => "resource-enforcement",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PilotEnforcementObservation {
    pub class: PilotEnforcementClass,
    pub admitted: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PilotDispatchDraft<'a> {
    pub binding: &'a PilotApplicationHostBinding,
    pub commit: &'a PilotCommit,
    pub sealed_unit: &'a super::super::runtime_authority_seal::RuntimeAuthorityUnit,
    pub operation: &'a PilotIdentity,
    pub effect_sequence: u64,
    pub enforcement: &'a [PilotEnforcementObservation],
    pub recheck: super::super::runtime_authority_seal::RuntimeAuthorityRecheckDraft<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PilotCommitDraft<'a> {
    pub binding: &'a PilotApplicationHostBinding,
    pub session_binding: &'a super::super::session_authority_profile::SessionAuthorityBinding,
    pub application_binding_sequence: u64,
    pub session_binding_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PilotCommit {
    binding_identity: PilotIdentity,
    session_binding_identity: String,
    highest_binding_sequence: u64,
    commit_identity: PilotIdentity,
}

impl PilotCommit {
    #[must_use]
    pub const fn binding_identity(&self) -> &PilotIdentity {
        &self.binding_identity
    }

    #[must_use]
    pub fn session_binding_identity(&self) -> &str {
        self.session_binding_identity.as_str()
    }

    #[must_use]
    pub const fn highest_binding_sequence(&self) -> u64 {
        self.highest_binding_sequence
    }

    #[must_use]
    pub const fn commit_identity(&self) -> &PilotIdentity {
        &self.commit_identity
    }

    pub(super) fn new(
        binding_identity: PilotIdentity,
        session_binding_identity: String,
        highest_binding_sequence: u64,
        commit_identity: PilotIdentity,
    ) -> Self {
        Self {
            binding_identity,
            session_binding_identity,
            highest_binding_sequence,
            commit_identity,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PilotDispatchAdmission {
    pub binding_identity: PilotIdentity,
    pub commit_identity: PilotIdentity,
    pub operation_identity: PilotIdentity,
    pub running_unit: super::super::runtime_authority_seal::RuntimeAuthorityUnit,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PilotIssueCode {
    StaleKamaciteContract,
    InvalidIdentity,
    WrongIdentityDomain,
    MissingHostProfile,
    AmbiguousHostProfile,
    OpenEffectSummary,
    MissingEffectOperation,
    TooManyEffectOperations,
    DuplicateEffectOperation,
    CohortMismatch,
    MissingNonClaim,
    AuthorityOverclaim,
    UnsafeField,
    InvalidSequence,
    BindingMismatch,
    SessionBindingMismatch,
    EffectBeforeBinding,
    UndeclaredOperation,
    MissingSeal,
    WidenedSeal,
    CurrentEnforcementDenied,
}

impl PilotIssueCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StaleKamaciteContract => "APPLICATION_BINDING_STALE_KAMACITE_CONTRACT",
            Self::InvalidIdentity => "APPLICATION_BINDING_INVALID_IDENTITY",
            Self::WrongIdentityDomain => "APPLICATION_BINDING_WRONG_IDENTITY_DOMAIN",
            Self::MissingHostProfile => "APPLICATION_BINDING_MISSING_HOST_PROFILE",
            Self::AmbiguousHostProfile => "APPLICATION_BINDING_AMBIGUOUS_HOST_PROFILE",
            Self::OpenEffectSummary => "APPLICATION_BINDING_OPEN_EFFECT_SUMMARY",
            Self::MissingEffectOperation => "APPLICATION_BINDING_MISSING_EFFECT_OPERATION",
            Self::TooManyEffectOperations => "APPLICATION_BINDING_TOO_MANY_EFFECT_OPERATIONS",
            Self::DuplicateEffectOperation => "APPLICATION_BINDING_DUPLICATE_EFFECT_OPERATION",
            Self::CohortMismatch => "APPLICATION_BINDING_COHORT_MISMATCH",
            Self::MissingNonClaim => "APPLICATION_BINDING_MISSING_NON_CLAIM",
            Self::AuthorityOverclaim => "APPLICATION_BINDING_AUTHORITY_OVERCLAIM",
            Self::UnsafeField => "APPLICATION_BINDING_UNSAFE_FIELD",
            Self::InvalidSequence => "APPLICATION_BINDING_INVALID_SEQUENCE",
            Self::BindingMismatch => "APPLICATION_BINDING_BINDING_MISMATCH",
            Self::SessionBindingMismatch => "APPLICATION_BINDING_SESSION_BINDING_MISMATCH",
            Self::EffectBeforeBinding => "APPLICATION_BINDING_EFFECT_BEFORE_BINDING",
            Self::UndeclaredOperation => "APPLICATION_BINDING_UNDECLARED_OPERATION",
            Self::MissingSeal => "APPLICATION_BINDING_MISSING_SEAL",
            Self::WidenedSeal => "APPLICATION_BINDING_WIDENED_SEAL",
            Self::CurrentEnforcementDenied => "APPLICATION_BINDING_CURRENT_ENFORCEMENT_DENIED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PilotIssue {
    pub code: PilotIssueCode,
    pub field: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PilotError {
    issues: Vec<PilotIssue>,
}

impl PilotError {
    #[must_use]
    pub fn issues(&self) -> &[PilotIssue] {
        self.issues.as_slice()
    }

    pub(super) fn from_issues(mut issues: Vec<PilotIssue>) -> Self {
        issues.sort_by(|left, right| {
            left.code
                .cmp(&right.code)
                .then_with(|| left.field.cmp(right.field))
        });
        issues.dedup();
        issues.truncate(MAX_PILOT_ISSUES);
        Self { issues }
    }
}
