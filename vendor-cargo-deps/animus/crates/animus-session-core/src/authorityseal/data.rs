//! Typed runtime-authority unit data.

#![allow(missing_docs)]

#[allow(unused_imports)]
use crate::prelude::*;

pub const RUNTIME_AUTHORITY_SEAL_SCHEMA: &str = "animus.runtime-authority-seal.v1";
pub const RUNTIME_AUTHORITY_SEAL_RECEIPT_SCHEMA: &str = "animus.runtime-authority-seal-receipt.v1";
pub const RUNTIME_AUTHORITY_SEAL_HASH_DOMAIN: &str = "animus.runtime-authority-seal.v1";
pub const RUNTIME_AUTHORITY_SEAL_RECEIPT_HASH_DOMAIN: &str =
    "animus.runtime-authority-seal-receipt.v1";
pub const MAX_RUNTIME_AUTHORITY_OPERATIONS: usize = 64;
pub const MAX_RUNTIME_AUTHORITY_RESOURCES: usize = 64;
pub const MAX_RUNTIME_AUTHORITY_VIEWS: usize = 32;

pub(super) const HASH_HEX_BYTES: usize = 64;
pub(super) const MAX_SAFE_LABEL_BYTES: usize = 128;
pub(super) const RUNTIME_AUTHORITY_VIEW_CLASS_COUNT: usize = 9;
pub(super) const FIXED_AUTHORITY_IDENTITY_COUNT: usize = 9;
pub(super) const MAX_RUNTIME_AUTHORITY_ISSUES: usize = 512;
pub(super) const HASH_SEPARATOR: &[u8] = b"\x1f";
pub(super) const REDACTION_MARKER: &str =
    "raw-grants-descriptors-paths-credentials-prompts-payloads-output-and-environment-omitted";
pub(super) const NON_CLAIMS: &[&str] = &[
    "runtime authority sealing is not authorization",
    "runtime authority sealing is not operating-system isolation",
    "runtime authority sealing is not hidden-authority absence",
    "runtime authority sealing is not sandbox or host integrity proof",
    "runtime authority sealing is not effect correctness or release eligibility",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeAuthorityUnitState {
    Preparing,
    Sealed,
    Running,
    Completed,
    Cancelled,
    Failed,
}

impl RuntimeAuthorityUnitState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Preparing => "preparing",
            Self::Sealed => "sealed",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeAuthorityViewClass {
    SessionLineage,
    DurableBinding,
    AuthorityProfile,
    EnforcementObservation,
    OperationSet,
    AdapterCohort,
    ResourceProfile,
    ResourceBudget,
    RuntimeServices,
}

impl RuntimeAuthorityViewClass {
    pub const ALL: [Self; RUNTIME_AUTHORITY_VIEW_CLASS_COUNT] = [
        Self::SessionLineage,
        Self::DurableBinding,
        Self::AuthorityProfile,
        Self::EnforcementObservation,
        Self::OperationSet,
        Self::AdapterCohort,
        Self::ResourceProfile,
        Self::ResourceBudget,
        Self::RuntimeServices,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionLineage => "session_lineage",
            Self::DurableBinding => "durable_binding",
            Self::AuthorityProfile => "authority_profile",
            Self::EnforcementObservation => "enforcement_observation",
            Self::OperationSet => "operation_set",
            Self::AdapterCohort => "adapter_cohort",
            Self::ResourceProfile => "resource_profile",
            Self::ResourceBudget => "resource_budget",
            Self::RuntimeServices => "runtime_services",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeAuthorityObservationStatus {
    Current,
    Stale,
    Unknown,
    Unsupported,
}

impl RuntimeAuthorityObservationStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Unknown => "unknown",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityExpectedViewDraft<'a> {
    pub class: RuntimeAuthorityViewClass,
    pub identity: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityObservedViewDraft<'a> {
    pub class: RuntimeAuthorityViewClass,
    pub identity: &'a str,
    pub status: RuntimeAuthorityObservationStatus,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuntimeAuthorityViewEntry {
    class: RuntimeAuthorityViewClass,
    identity: String,
}

impl RuntimeAuthorityViewEntry {
    #[must_use]
    pub const fn class(&self) -> RuntimeAuthorityViewClass {
        self.class
    }

    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    pub(super) fn new(class: RuntimeAuthorityViewClass, identity: String) -> Self {
        Self { class, identity }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityUnitDraft<'a> {
    pub session_id: &'a str,
    pub dispatch_id: &'a str,
    pub session_lineage_identity: &'a str,
    pub durable_binding_identity: &'a str,
    pub authority_profile_identity: &'a str,
    pub enforcement_observation_identity: &'a str,
    pub operation_set_identity: &'a str,
    pub adapter_cohort_identity: &'a str,
    pub resource_profile_identity: &'a str,
    pub resource_budget_identity: &'a str,
    pub runtime_services_identity: &'a str,
    pub operation_identities: &'a [&'a str],
    pub resource_identities: &'a [&'a str],
    pub expected_views: &'a [RuntimeAuthorityExpectedViewDraft<'a>],
    pub observed_views: &'a [RuntimeAuthorityObservedViewDraft<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityUnit {
    schema: &'static str,
    session_id: String,
    dispatch_id: String,
    session_lineage_identity: String,
    durable_binding_identity: String,
    authority_profile_identity: String,
    enforcement_observation_identity: String,
    operation_set_identity: String,
    adapter_cohort_identity: String,
    resource_profile_identity: String,
    resource_budget_identity: String,
    runtime_services_identity: String,
    operation_identities: Vec<String>,
    resource_identities: Vec<String>,
    runtime_views: Vec<RuntimeAuthorityViewEntry>,
    unit_identity: String,
    state: RuntimeAuthorityUnitState,
    seal_receipt_identity: Option<String>,
}

impl RuntimeAuthorityUnit {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    #[must_use]
    pub fn dispatch_id(&self) -> &str {
        self.dispatch_id.as_str()
    }

    #[must_use]
    pub fn session_lineage_identity(&self) -> &str {
        self.session_lineage_identity.as_str()
    }

    #[must_use]
    pub fn durable_binding_identity(&self) -> &str {
        self.durable_binding_identity.as_str()
    }

    #[must_use]
    pub fn authority_profile_identity(&self) -> &str {
        self.authority_profile_identity.as_str()
    }

    #[must_use]
    pub fn enforcement_observation_identity(&self) -> &str {
        self.enforcement_observation_identity.as_str()
    }

    #[must_use]
    pub fn operation_set_identity(&self) -> &str {
        self.operation_set_identity.as_str()
    }

    #[must_use]
    pub fn adapter_cohort_identity(&self) -> &str {
        self.adapter_cohort_identity.as_str()
    }

    #[must_use]
    pub fn resource_profile_identity(&self) -> &str {
        self.resource_profile_identity.as_str()
    }

    #[must_use]
    pub fn resource_budget_identity(&self) -> &str {
        self.resource_budget_identity.as_str()
    }

    #[must_use]
    pub fn runtime_services_identity(&self) -> &str {
        self.runtime_services_identity.as_str()
    }

    #[must_use]
    pub fn operation_identities(&self) -> &[String] {
        self.operation_identities.as_slice()
    }

    #[must_use]
    pub fn resource_identities(&self) -> &[String] {
        self.resource_identities.as_slice()
    }

    #[must_use]
    pub fn runtime_views(&self) -> &[RuntimeAuthorityViewEntry] {
        self.runtime_views.as_slice()
    }

    #[must_use]
    pub fn unit_identity(&self) -> &str {
        self.unit_identity.as_str()
    }

    #[must_use]
    pub const fn state(&self) -> RuntimeAuthorityUnitState {
        self.state
    }

    #[must_use]
    pub fn seal_receipt_identity(&self) -> Option<&str> {
        self.seal_receipt_identity.as_deref()
    }

    pub(super) fn from_parts(parts: RuntimeAuthorityUnitParts) -> Self {
        Self {
            schema: RUNTIME_AUTHORITY_SEAL_SCHEMA,
            session_id: parts.session_id,
            dispatch_id: parts.dispatch_id,
            session_lineage_identity: parts.session_lineage_identity,
            durable_binding_identity: parts.durable_binding_identity,
            authority_profile_identity: parts.authority_profile_identity,
            enforcement_observation_identity: parts.enforcement_observation_identity,
            operation_set_identity: parts.operation_set_identity,
            adapter_cohort_identity: parts.adapter_cohort_identity,
            resource_profile_identity: parts.resource_profile_identity,
            resource_budget_identity: parts.resource_budget_identity,
            runtime_services_identity: parts.runtime_services_identity,
            operation_identities: parts.operation_identities,
            resource_identities: parts.resource_identities,
            runtime_views: parts.runtime_views,
            unit_identity: parts.unit_identity,
            state: RuntimeAuthorityUnitState::Preparing,
            seal_receipt_identity: None,
        }
    }

    pub(super) fn with_state(
        &self,
        state: RuntimeAuthorityUnitState,
        seal_receipt_identity: Option<String>,
    ) -> Self {
        let mut next = self.clone();
        next.state = state;
        next.seal_receipt_identity = seal_receipt_identity;
        next
    }
}

pub(super) struct RuntimeAuthorityUnitParts {
    pub session_id: String,
    pub dispatch_id: String,
    pub session_lineage_identity: String,
    pub durable_binding_identity: String,
    pub authority_profile_identity: String,
    pub enforcement_observation_identity: String,
    pub operation_set_identity: String,
    pub adapter_cohort_identity: String,
    pub resource_profile_identity: String,
    pub resource_budget_identity: String,
    pub runtime_services_identity: String,
    pub operation_identities: Vec<String>,
    pub resource_identities: Vec<String>,
    pub runtime_views: Vec<RuntimeAuthorityViewEntry>,
    pub unit_identity: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RuntimeAuthoritySensitiveDraft<'a> {
    pub raw_grant: Option<&'a str>,
    pub descriptor: Option<&'a str>,
    pub host_path: Option<&'a str>,
    pub credential: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub provider_payload: Option<&'a str>,
    pub tool_body: Option<&'a str>,
    pub model_output: Option<&'a str>,
    pub environment_value: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityReceiptDraft<'a> {
    pub sensitive: RuntimeAuthoritySensitiveDraft<'a>,
    pub semantic_claims: &'a [&'a str],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAuthoritySealReceipt {
    pub schema: &'static str,
    pub unit_identity: String,
    pub session_id: String,
    pub dispatch_id: String,
    pub session_lineage_identity: String,
    pub authority_profile_identity: String,
    pub durable_binding_identity: String,
    pub enforcement_observation_identity: String,
    pub operation_set_identity: String,
    pub adapter_cohort_identity: String,
    pub resource_profile_identity: String,
    pub resource_budget_identity: String,
    pub runtime_services_identity: String,
    pub operation_identities: Vec<String>,
    pub resource_identities: Vec<String>,
    pub runtime_views: Vec<RuntimeAuthorityViewEntry>,
    pub state: RuntimeAuthorityUnitState,
    pub commit_required_before_effect: bool,
    pub redaction_marker: &'static str,
    pub non_claims: Vec<String>,
    pub receipt_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAuthoritySealPlan {
    unit: RuntimeAuthorityUnit,
    receipt: RuntimeAuthoritySealReceipt,
}

impl RuntimeAuthoritySealPlan {
    #[must_use]
    pub const fn unit(&self) -> &RuntimeAuthorityUnit {
        &self.unit
    }

    #[must_use]
    pub const fn receipt(&self) -> &RuntimeAuthoritySealReceipt {
        &self.receipt
    }

    pub(super) const fn from_parts(
        unit: RuntimeAuthorityUnit,
        receipt: RuntimeAuthoritySealReceipt,
    ) -> Self {
        Self { unit, receipt }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeAuthorityRecheckDraft<'a> {
    pub session_lineage_identity: &'a str,
    pub authority_profile_identity: &'a str,
    pub durable_binding_identity: &'a str,
    pub enforcement_observation_identity: &'a str,
    pub operation_set_identity: &'a str,
    pub adapter_cohort_identity: &'a str,
    pub resource_profile_identity: &'a str,
    pub resource_budget_identity: &'a str,
    pub runtime_services_identity: &'a str,
    pub operation_identities: &'a [&'a str],
    pub resource_identities: &'a [&'a str],
    pub admitted: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeAuthorityMutationKind {
    CapabilityAddition,
    ProfileRebind,
    AdapterReplacement,
    ResourceExpansion,
    AmbientDiscovery,
    Fallback,
    RepeatedSeal,
    StaleReuse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAuthorityTransition {
    StartEffect,
    Complete,
    Cancel,
    Fail,
    AttemptMutation(RuntimeAuthorityMutationKind),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeAuthorityIssueCode {
    UnsafeLabel,
    UnsafeIdentity,
    EmptyOperationSet,
    DuplicateOperation,
    DuplicateResource,
    ItemLimitExceeded,
    MissingView,
    UnexpectedView,
    DuplicateView,
    StaleView,
    UnknownView,
    UnsupportedView,
    ViewIdentityMismatch,
    InvalidStateTransition,
    PostSealWidening,
    ReceiptMismatch,
    CurrentEnforcementDenied,
    CurrentEnforcementMismatch,
    UnsafeReceipt,
    AuthorityOverclaim,
    StaleUnitReuse,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RuntimeAuthorityIssue {
    code: RuntimeAuthorityIssueCode,
    field: &'static str,
    view_class: Option<RuntimeAuthorityViewClass>,
}

impl RuntimeAuthorityIssue {
    #[must_use]
    pub const fn code(&self) -> RuntimeAuthorityIssueCode {
        self.code
    }

    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    #[must_use]
    pub const fn view_class(&self) -> Option<RuntimeAuthorityViewClass> {
        self.view_class
    }

    pub(super) const fn new(
        code: RuntimeAuthorityIssueCode,
        field: &'static str,
        view_class: Option<RuntimeAuthorityViewClass>,
    ) -> Self {
        Self {
            code,
            field,
            view_class,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAuthoritySealError {
    issues: Vec<RuntimeAuthorityIssue>,
}

impl RuntimeAuthoritySealError {
    #[must_use]
    pub fn issues(&self) -> &[RuntimeAuthorityIssue] {
        self.issues.as_slice()
    }

    pub(super) fn from_issues(mut issues: Vec<RuntimeAuthorityIssue>) -> Self {
        issues.sort();
        issues.dedup();
        Self { issues }
    }
}

impl core::fmt::Display for RuntimeAuthoritySealError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "runtime authority seal rejected {} issue(s)",
            self.issues.len()
        )
    }
}

impl core::error::Error for RuntimeAuthoritySealError {}
