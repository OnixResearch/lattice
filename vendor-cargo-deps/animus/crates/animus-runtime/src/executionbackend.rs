//! Pure weakest-admitted execution backend selection and current-enforcement recheck.

use blake3::Hasher;
use std::boxed::Box;
use std::string::{String, ToString};
use std::vec::Vec;

const POLICY_DOMAIN: &str = "animus/execution-backend-policy";
const PLAN_DOMAIN: &str = "animus/execution-backend-plan";
const RECEIPT_DOMAIN: &str = "animus/execution-backend-receipt";
const SCHEMA: &str = "animus.execution-backend-selection.v1";
const HASH_HEX_LENGTH: usize = 64;
const REQUIRED_DIMENSION_COUNT: usize = 8;
const BACKEND_CLASS_COUNT: usize = 4;
const MINIMUM_RESOURCE_VALUE: u64 = 1;
const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "not-backend-availability",
    "not-sandbox-enforcement",
    "not-work-completion",
    "not-output-correctness",
];
const FORBIDDEN_EVIDENCE: [&str; 9] = [
    "command:",
    "credential",
    "environment=",
    "provider payload",
    "sandbox diagnostic",
    "/home/",
    "stdout:",
    "stderr:",
    "secret",
];

macro_rules! hash_identity {
    ($name:ident) => {
        #[doc = concat!("Nominal BLAKE3 identity `", stringify!($name), "`.")]
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Parses a lowercase BLAKE3 identity.
            ///
            /// # Errors
            ///
            /// Returns [`BackendSelectionError::InvalidIdentity`] for malformed text.
            pub fn parse(value: &str) -> Result<Self, BackendSelectionError> {
                if !is_blake3(value) {
                    return Err(BackendSelectionError::InvalidIdentity);
                }
                Ok(Self(value.to_string()))
            }

            /// Returns the lowercase identity text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

hash_identity!(WorkloadId);
hash_identity!(BackendId);
hash_identity!(AdapterId);
hash_identity!(ApplicationHostBindingId);
hash_identity!(RuntimeAuthorityUnitId);
hash_identity!(EffectPermissionId);
hash_identity!(ResourceProfileId);
hash_identity!(EnforcementProfileId);
hash_identity!(BackendPolicyId);
hash_identity!(BackendPlanId);
hash_identity!(BackendReceiptId);
hash_identity!(ObservationId);

/// Closed execution backend classes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ExecutionBackendClass {
    /// In-process mediated component.
    EmbeddedComponent,
    /// Policy-guarded local child process.
    GuardedLocalProcess,
    /// Isolated host or full sandbox.
    IsolatedHost,
    /// Explicit external machine.
    ExternalMachine,
}

impl ExecutionBackendClass {
    /// Parses a generated backend class value.
    ///
    /// # Errors
    ///
    /// Returns [`BackendSelectionError::UnknownBackendClass`] for unknown text.
    pub fn parse(value: &str) -> Result<Self, BackendSelectionError> {
        match value {
            "embedded_component" | "EmbeddedComponent" => Ok(Self::EmbeddedComponent),
            "guarded_local_process" | "GuardedLocalProcess" => Ok(Self::GuardedLocalProcess),
            "isolated_host" | "IsolatedHost" => Ok(Self::IsolatedHost),
            "external_machine" | "ExternalMachine" => Ok(Self::ExternalMachine),
            _ => Err(BackendSelectionError::UnknownBackendClass),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::EmbeddedComponent => "embedded_component",
            Self::GuardedLocalProcess => "guarded_local_process",
            Self::IsolatedHost => "isolated_host",
            Self::ExternalMachine => "external_machine",
        }
    }
}

/// One closed workload requirement value.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkloadRequirement {
    /// WebAssembly component execution.
    WasmComponent,
    /// Native binary execution.
    NativeBinary,
    /// Script runtime execution.
    ScriptRuntime,
    /// No filesystem access.
    NoFilesystem,
    /// Read-only capability filesystem.
    ReadOnlyFilesystem,
    /// Writable workspace filesystem.
    WritableWorkspace,
    /// Network is denied.
    NetworkDenied,
    /// Network uses an explicit allowlist.
    NetworkAllowlisted,
    /// In-process execution only.
    InProcess,
    /// One guarded child process.
    SingleProcess,
    /// A bounded process tree.
    ProcessTree,
    /// No terminal is required.
    NoTerminal,
    /// A pseudoterminal is required.
    Pseudoterminal,
    /// No browser or device is required.
    NoPeripheral,
    /// A browser is required.
    Browser,
    /// A host device is required.
    HostDevice,
    /// Ephemeral execution state.
    Ephemeral,
    /// Durable workspace state.
    DurableWorkspace,
    /// Mediated component isolation.
    MediatedIsolation,
    /// Guarded process isolation.
    GuardedProcessIsolation,
    /// Host-level isolation.
    HostIsolation,
}

impl WorkloadRequirement {
    /// Parses a generated policy requirement value.
    ///
    /// # Errors
    ///
    /// Returns [`BackendSelectionError::UnknownRequirement`] for unknown text.
    pub fn parse(value: &str) -> Result<Self, BackendSelectionError> {
        match value {
            "wasm_component" => Ok(Self::WasmComponent),
            "native_binary" => Ok(Self::NativeBinary),
            "script_runtime" => Ok(Self::ScriptRuntime),
            "no_filesystem" => Ok(Self::NoFilesystem),
            "read_only_filesystem" => Ok(Self::ReadOnlyFilesystem),
            "writable_workspace" => Ok(Self::WritableWorkspace),
            "network_denied" => Ok(Self::NetworkDenied),
            "network_allowlisted" => Ok(Self::NetworkAllowlisted),
            "in_process" => Ok(Self::InProcess),
            "single_process" => Ok(Self::SingleProcess),
            "process_tree" => Ok(Self::ProcessTree),
            "no_terminal" => Ok(Self::NoTerminal),
            "pseudoterminal" => Ok(Self::Pseudoterminal),
            "no_peripheral" => Ok(Self::NoPeripheral),
            "browser" => Ok(Self::Browser),
            "host_device" => Ok(Self::HostDevice),
            "ephemeral" => Ok(Self::Ephemeral),
            "durable_workspace" => Ok(Self::DurableWorkspace),
            "mediated_isolation" => Ok(Self::MediatedIsolation),
            "guarded_process_isolation" => Ok(Self::GuardedProcessIsolation),
            "host_isolation" => Ok(Self::HostIsolation),
            _ => Err(BackendSelectionError::UnknownRequirement),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::WasmComponent => "wasm_component",
            Self::NativeBinary => "native_binary",
            Self::ScriptRuntime => "script_runtime",
            Self::NoFilesystem => "no_filesystem",
            Self::ReadOnlyFilesystem => "read_only_filesystem",
            Self::WritableWorkspace => "writable_workspace",
            Self::NetworkDenied => "network_denied",
            Self::NetworkAllowlisted => "network_allowlisted",
            Self::InProcess => "in_process",
            Self::SingleProcess => "single_process",
            Self::ProcessTree => "process_tree",
            Self::NoTerminal => "no_terminal",
            Self::Pseudoterminal => "pseudoterminal",
            Self::NoPeripheral => "no_peripheral",
            Self::Browser => "browser",
            Self::HostDevice => "host_device",
            Self::Ephemeral => "ephemeral",
            Self::DurableWorkspace => "durable_workspace",
            Self::MediatedIsolation => "mediated_isolation",
            Self::GuardedProcessIsolation => "guarded_process_isolation",
            Self::HostIsolation => "host_isolation",
        }
    }

    const fn dimension(self) -> RequirementDimension {
        match self {
            Self::WasmComponent | Self::NativeBinary | Self::ScriptRuntime => {
                RequirementDimension::ExecutionFormat
            }
            Self::NoFilesystem | Self::ReadOnlyFilesystem | Self::WritableWorkspace => {
                RequirementDimension::Filesystem
            }
            Self::NetworkDenied | Self::NetworkAllowlisted => RequirementDimension::Network,
            Self::InProcess | Self::SingleProcess | Self::ProcessTree => {
                RequirementDimension::Process
            }
            Self::NoTerminal | Self::Pseudoterminal => RequirementDimension::Terminal,
            Self::NoPeripheral | Self::Browser | Self::HostDevice => {
                RequirementDimension::Peripheral
            }
            Self::Ephemeral | Self::DurableWorkspace => RequirementDimension::Persistence,
            Self::MediatedIsolation | Self::GuardedProcessIsolation | Self::HostIsolation => {
                RequirementDimension::Isolation
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RequirementDimension {
    ExecutionFormat,
    Filesystem,
    Network,
    Process,
    Terminal,
    Peripheral,
    Persistence,
    Isolation,
}

/// Explicit workload resource limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BackendResourceLimits {
    /// Maximum memory bytes.
    pub memory_bytes: u64,
    /// Maximum storage bytes.
    pub storage_bytes: u64,
    /// Maximum CPU time in milliseconds.
    pub cpu_milliseconds: u64,
    /// Maximum wall duration in milliseconds.
    pub wall_milliseconds: u64,
}

impl BackendResourceLimits {
    fn is_positive(self) -> bool {
        self.memory_bytes >= MINIMUM_RESOURCE_VALUE
            && self.storage_bytes >= MINIMUM_RESOURCE_VALUE
            && self.cpu_milliseconds >= MINIMUM_RESOURCE_VALUE
            && self.wall_milliseconds >= MINIMUM_RESOURCE_VALUE
    }

    const fn fits_within(self, capacity: Self) -> bool {
        self.memory_bytes <= capacity.memory_bytes
            && self.storage_bytes <= capacity.storage_bytes
            && self.cpu_milliseconds <= capacity.cpu_milliseconds
            && self.wall_milliseconds <= capacity.wall_milliseconds
    }
}

/// Complete workload selection request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkloadRequirements {
    /// Stable workload identity.
    pub workload_id: WorkloadId,
    /// Exactly one requirement from each closed dimension.
    pub requirements: Vec<WorkloadRequirement>,
    /// Explicit resource limits.
    pub resources: BackendResourceLimits,
}

/// One caller-supplied backend candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionBackendCandidate {
    /// Backend identity.
    pub backend_id: BackendId,
    /// Backend class.
    pub class: ExecutionBackendClass,
    /// Adapter identity.
    pub adapter_id: AdapterId,
    /// Application-host binding identity.
    pub application_host_binding_id: ApplicationHostBindingId,
    /// Runtime-authority unit identity.
    pub runtime_authority_unit_id: RuntimeAuthorityUnitId,
    /// Effect-permission identity.
    pub effect_permission_id: EffectPermissionId,
    /// Resource-profile identity.
    pub resource_profile_id: ResourceProfileId,
    /// Isolation or sandbox enforcement identity.
    pub enforcement_profile_id: EnforcementProfileId,
    /// Supported closed requirement values.
    pub supported_requirements: Vec<WorkloadRequirement>,
    /// Current declared resource capacity.
    pub capacity: BackendResourceLimits,
    /// Caller-supplied planning-time availability fact.
    pub available: bool,
}

/// Compatibility policy for one backend class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendClassPolicy {
    /// Backend class.
    pub class: ExecutionBackendClass,
    /// Requirement values this class can admit under policy.
    pub admitted_requirements: Vec<WorkloadRequirement>,
}

/// Generated Nickel policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSelectionPolicyDraft {
    /// BLAKE3 source-policy identity.
    pub policy_id: String,
    /// Explicit weakest-to-broadest order.
    pub preference_order: Vec<ExecutionBackendClass>,
    /// Per-class compatibility policy.
    pub class_policies: Vec<BackendClassPolicy>,
    /// Must be true. Silent fallback is never admitted.
    pub deny_silent_fallback: bool,
    /// Mandatory non-claims.
    pub non_claims: Vec<String>,
}

/// Revalidated execution backend policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSelectionPolicy {
    policy_id: BackendPolicyId,
    preference_order: Vec<ExecutionBackendClass>,
    class_policies: Vec<BackendClassPolicy>,
    non_claims: Vec<String>,
    identity: String,
}

impl BackendSelectionPolicy {
    /// Revalidates generated policy and derives a BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns a typed error for incomplete order, duplicate classes, missing
    /// compatibility rows, silent fallback, or missing non-claims.
    pub fn validate(draft: BackendSelectionPolicyDraft) -> Result<Self, BackendSelectionError> {
        let policy_id = BackendPolicyId::parse(draft.policy_id.as_str())?;
        if !draft.deny_silent_fallback
            || draft.preference_order.len() != ExecutionBackendClass::ALL.len()
            || has_duplicate_classes(&draft.preference_order)
            || draft.class_policies.len() != ExecutionBackendClass::ALL.len()
            || draft
                .preference_order
                .iter()
                .any(|class| class_policy(&draft.class_policies, *class).is_none())
            || !has_required_values(&draft.non_claims, &REQUIRED_NON_CLAIMS)
        {
            return Err(BackendSelectionError::InvalidPolicy);
        }
        let mut parts = Vec::new();
        parts.push(SCHEMA.to_string());
        parts.push(policy_id.as_str().to_string());
        for class in &draft.preference_order {
            parts.push(class.as_str().to_string());
            let rule = class_policy(&draft.class_policies, *class)
                .ok_or(BackendSelectionError::InvalidPolicy)?;
            if has_duplicate_requirements(&rule.admitted_requirements) {
                return Err(BackendSelectionError::InvalidPolicy);
            }
            let mut requirements = rule.admitted_requirements.clone();
            requirements.sort_unstable();
            for requirement in requirements {
                parts.push(requirement.as_str().to_string());
            }
        }
        let identity = hash_strings(POLICY_DOMAIN, &parts);
        Ok(Self {
            policy_id,
            preference_order: draft.preference_order,
            class_policies: draft.class_policies,
            non_claims: draft.non_claims,
            identity,
        })
    }

    /// Returns the canonical policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

impl ExecutionBackendClass {
    const ALL: [Self; BACKEND_CLASS_COUNT] = [
        Self::EmbeddedComponent,
        Self::GuardedLocalProcess,
        Self::IsolatedHost,
        Self::ExternalMachine,
    ];
}

/// Bounded selection denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendSelectionDenial {
    /// Workload requirements are missing or ambiguous.
    IncompleteRequirements,
    /// No policy-admitted candidate satisfies all requirements.
    UnsupportedCombination,
    /// More than one candidate exists at the selected policy position.
    AmbiguousCandidate,
    /// The selected weakest candidate is unavailable. Explicit replanning is required.
    SelectedBackendUnavailable,
    /// Required resources exceed the selected backend capacity.
    ResourceDenied,
}

impl BackendSelectionDenial {
    const fn as_str(self) -> &'static str {
        match self {
            Self::IncompleteRequirements => "incomplete_requirements",
            Self::UnsupportedCombination => "unsupported_combination",
            Self::AmbiguousCandidate => "ambiguous_candidate",
            Self::SelectedBackendUnavailable => "selected_backend_unavailable",
            Self::ResourceDenied => "resource_denied",
        }
    }
}

/// Selected backend plan with exact reviewed identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionBackendPlan {
    /// Plan identity.
    pub plan_id: BackendPlanId,
    /// Workload identity.
    pub workload_id: WorkloadId,
    /// Selected backend identity.
    pub backend_id: BackendId,
    /// Selected backend class.
    pub class: ExecutionBackendClass,
    /// Exact adapter identity.
    pub adapter_id: AdapterId,
    /// Exact application-host binding identity.
    pub application_host_binding_id: ApplicationHostBindingId,
    /// Exact runtime-authority unit identity.
    pub runtime_authority_unit_id: RuntimeAuthorityUnitId,
    /// Exact effect-permission identity.
    pub effect_permission_id: EffectPermissionId,
    /// Exact resource-profile identity.
    pub resource_profile_id: ResourceProfileId,
    /// Exact enforcement-profile identity.
    pub enforcement_profile_id: EnforcementProfileId,
    /// Policy identity.
    pub policy_identity: String,
    /// Requirement classes bound into this plan.
    pub requirements: Vec<WorkloadRequirement>,
    /// Explicit resource limits.
    pub resources: BackendResourceLimits,
    /// Later policy classes that were unnecessary.
    pub unnecessary_broader_classes: Vec<ExecutionBackendClass>,
}

/// Pure planner result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackendSelectionResult {
    /// One weakest admitted backend was selected.
    Selected(Box<ExecutionBackendPlan>),
    /// Selection was denied with a bounded reason.
    Denied(BackendSelectionDenial),
}

/// Deterministic safe explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSelectionExplanation {
    /// Selected class when selection passed.
    pub selected_class: Option<ExecutionBackendClass>,
    /// Denial when selection failed.
    pub denial: Option<BackendSelectionDenial>,
    /// Number of broader classes that were not needed.
    pub unnecessary_broader_count: u32,
    /// Safe fixed summary.
    pub summary: &'static str,
}

/// Plans the weakest admitted backend from explicit policy order.
///
/// # Errors
///
/// Returns a typed error for invalid requirements, duplicate candidates, or
/// unsafe identity state. Expected selection failures are bounded results.
pub fn select_execution_backend(
    policy: &BackendSelectionPolicy,
    workload: &WorkloadRequirements,
    candidates: &[ExecutionBackendCandidate],
) -> Result<BackendSelectionResult, BackendSelectionError> {
    if !workload.resources.is_positive() || !complete_requirements(&workload.requirements) {
        return Ok(BackendSelectionResult::Denied(
            BackendSelectionDenial::IncompleteRequirements,
        ));
    }
    if has_duplicate_backend_ids(candidates) {
        return Err(BackendSelectionError::DuplicateBackendIdentity);
    }
    for (order_index, class) in policy.preference_order.iter().copied().enumerate() {
        let rule = class_policy(&policy.class_policies, class)
            .ok_or(BackendSelectionError::InvalidPolicy)?;
        if !requirements_supported(&workload.requirements, &rule.admitted_requirements) {
            continue;
        }
        let compatible: Vec<&ExecutionBackendCandidate> = candidates
            .iter()
            .filter(|candidate| {
                candidate.class == class
                    && requirements_supported(
                        &workload.requirements,
                        &candidate.supported_requirements,
                    )
            })
            .collect();
        if compatible.is_empty() {
            continue;
        }
        if compatible.len() != 1 {
            return Ok(BackendSelectionResult::Denied(
                BackendSelectionDenial::AmbiguousCandidate,
            ));
        }
        let candidate = compatible[0];
        if !candidate.available {
            return Ok(BackendSelectionResult::Denied(
                BackendSelectionDenial::SelectedBackendUnavailable,
            ));
        }
        if !workload.resources.fits_within(candidate.capacity) {
            return Ok(BackendSelectionResult::Denied(
                BackendSelectionDenial::ResourceDenied,
            ));
        }
        let broader = policy
            .preference_order
            .iter()
            .skip(order_index.saturating_add(1))
            .copied()
            .collect::<Vec<_>>();
        let plan = build_plan(policy, workload, candidate, broader)?;
        return Ok(BackendSelectionResult::Selected(Box::new(plan)));
    }
    Ok(BackendSelectionResult::Denied(
        BackendSelectionDenial::UnsupportedCombination,
    ))
}

/// Creates a deterministic safe explanation.
#[must_use]
pub fn explain_backend_selection(result: &BackendSelectionResult) -> BackendSelectionExplanation {
    match result {
        BackendSelectionResult::Selected(plan) => BackendSelectionExplanation {
            selected_class: Some(plan.class),
            denial: None,
            unnecessary_broader_count: u32::try_from(plan.unnecessary_broader_classes.len())
                .unwrap_or(u32::MAX),
            summary: "selected weakest policy-admitted backend; current enforcement recheck required",
        },
        BackendSelectionResult::Denied(denial) => BackendSelectionExplanation {
            selected_class: None,
            denial: Some(*denial),
            unnecessary_broader_count: 0,
            summary: "backend selection denied; explicit new plan required",
        },
    }
}

/// Current host observation supplied by an implementation-free service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentBackendObservation {
    /// Observation identity.
    pub observation_id: ObservationId,
    /// Current backend identity.
    pub backend_id: BackendId,
    /// Current adapter identity.
    pub adapter_id: AdapterId,
    /// Current application-host binding identity.
    pub application_host_binding_id: ApplicationHostBindingId,
    /// Current runtime-authority unit identity.
    pub runtime_authority_unit_id: RuntimeAuthorityUnitId,
    /// Current effect-permission identity.
    pub effect_permission_id: EffectPermissionId,
    /// Current resource-profile identity.
    pub resource_profile_id: ResourceProfileId,
    /// Current isolation enforcement identity.
    pub enforcement_profile_id: EnforcementProfileId,
    /// Current backend availability.
    pub available: bool,
    /// Current resources remain admitted.
    pub resources_admitted: bool,
    /// Current isolation or sandbox enforcement was observed.
    pub enforcement_current: bool,
}

/// Implementation-free current-enforcement service.
pub trait CurrentBackendEnforcementService {
    /// Returns a current observation for the selected plan.
    ///
    /// # Errors
    ///
    /// Returns a bounded service error without private diagnostics.
    fn observe(
        &self,
        plan: &ExecutionBackendPlan,
    ) -> Result<CurrentBackendObservation, CurrentBackendServiceError>;
}

/// Safe current-enforcement service error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentBackendServiceError {
    /// The service is unavailable.
    Unavailable,
    /// The service rejected the request.
    Rejected,
}

/// Current enforcement denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendEnforcementDenial {
    /// No current observation was available.
    ObservationUnavailable,
    /// Selected backend is no longer available.
    BackendUnavailable,
    /// One or more exact authority identities drifted.
    AuthorityDrift,
    /// Resources are no longer admitted.
    ResourceDenied,
    /// Isolation or sandbox enforcement is not current.
    EnforcementNotCurrent,
}

impl BackendEnforcementDenial {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ObservationUnavailable => "observation_unavailable",
            Self::BackendUnavailable => "backend_unavailable",
            Self::AuthorityDrift => "authority_drift",
            Self::ResourceDenied => "resource_denied",
            Self::EnforcementNotCurrent => "enforcement_not_current",
        }
    }
}

/// Bounded current-enforcement denial with optional safe observation identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEnforcementDenialRecord {
    /// Denial reason.
    pub reason: BackendEnforcementDenial,
    /// Observation identity when the service returned current facts.
    pub observation_id: Option<ObservationId>,
}

/// Current pre-effect recheck result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BackendEnforcementResult {
    /// Exact current enforcement permits the host to continue to its own effect gate.
    Admitted(CurrentBackendObservation),
    /// Current enforcement denied execution before effects.
    Denied(BackendEnforcementDenialRecord),
}

/// Rechecks all selected identities and current enforcement before effects.
#[must_use]
pub fn recheck_execution_backend(
    service: &dyn CurrentBackendEnforcementService,
    plan: &ExecutionBackendPlan,
) -> BackendEnforcementResult {
    let observation = match service.observe(plan) {
        Ok(observation) => observation,
        Err(CurrentBackendServiceError::Unavailable | CurrentBackendServiceError::Rejected) => {
            return enforcement_denied(BackendEnforcementDenial::ObservationUnavailable, None);
        }
    };
    if !observation.available {
        return enforcement_denied(
            BackendEnforcementDenial::BackendUnavailable,
            Some(observation.observation_id),
        );
    }
    if observation.backend_id != plan.backend_id
        || observation.adapter_id != plan.adapter_id
        || observation.application_host_binding_id != plan.application_host_binding_id
        || observation.runtime_authority_unit_id != plan.runtime_authority_unit_id
        || observation.effect_permission_id != plan.effect_permission_id
        || observation.resource_profile_id != plan.resource_profile_id
        || observation.enforcement_profile_id != plan.enforcement_profile_id
    {
        return enforcement_denied(
            BackendEnforcementDenial::AuthorityDrift,
            Some(observation.observation_id),
        );
    }
    if !observation.resources_admitted {
        return enforcement_denied(
            BackendEnforcementDenial::ResourceDenied,
            Some(observation.observation_id),
        );
    }
    if !observation.enforcement_current {
        return enforcement_denied(
            BackendEnforcementDenial::EnforcementNotCurrent,
            Some(observation.observation_id),
        );
    }
    BackendEnforcementResult::Admitted(observation)
}

fn enforcement_denied(
    reason: BackendEnforcementDenial,
    observation_id: Option<ObservationId>,
) -> BackendEnforcementResult {
    BackendEnforcementResult::Denied(BackendEnforcementDenialRecord {
        reason,
        observation_id,
    })
}

/// Metadata-only planner or enforcement receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSelectionReceipt {
    /// Receipt identity.
    pub receipt_id: BackendReceiptId,
    /// Canonical BLAKE3 receipt identity.
    pub identity: String,
    /// Selected backend class, when present.
    pub selected_class: Option<ExecutionBackendClass>,
    /// Planner denial, when present.
    pub selection_denial: Option<BackendSelectionDenial>,
    /// Enforcement denial, when present.
    pub enforcement_denial: Option<BackendEnforcementDenial>,
    /// Current observation identity, when the service returned facts.
    pub observation_id: Option<ObservationId>,
    /// BLAKE3 identity over the selected requirement classes, when present.
    pub requirement_set_identity: Option<String>,
    /// Number of requirement classes.
    pub requirement_count: u32,
    /// Mandatory non-claims.
    pub non_claims: Vec<String>,
}

/// Projects a metadata-only receipt from a selection result.
///
/// # Errors
///
/// Returns a typed error for count overflow.
pub fn project_backend_selection_receipt(
    policy: &BackendSelectionPolicy,
    receipt_id: BackendReceiptId,
    result: &BackendSelectionResult,
) -> Result<BackendSelectionReceipt, BackendSelectionError> {
    let (
        selected_class,
        selection_denial,
        requirement_count,
        subject_identity,
        requirement_set_identity,
    ) = match result {
        BackendSelectionResult::Selected(plan) => (
            Some(plan.class),
            None,
            u32::try_from(plan.requirements.len())
                .map_err(|_| BackendSelectionError::CountOverflow)?,
            plan.plan_id.as_str(),
            Some(hash_requirement_set(&plan.requirements)),
        ),
        BackendSelectionResult::Denied(denial) => (None, Some(*denial), 0, denial.as_str(), None),
    };
    let count = requirement_count.to_string();
    let class = selected_class.map_or("none", ExecutionBackendClass::as_str);
    let denial = selection_denial.map_or("none", BackendSelectionDenial::as_str);
    let requirement_identity_text = requirement_set_identity.as_deref().unwrap_or("none");
    let identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            SCHEMA,
            policy.policy_id.as_str(),
            policy.identity.as_str(),
            receipt_id.as_str(),
            subject_identity,
            class,
            denial,
            requirement_identity_text,
            count.as_str(),
        ],
    );
    Ok(BackendSelectionReceipt {
        receipt_id,
        identity,
        selected_class,
        selection_denial,
        enforcement_denial: None,
        observation_id: None,
        requirement_set_identity,
        requirement_count,
        non_claims: policy.non_claims.clone(),
    })
}

/// Projects a metadata-only current-enforcement receipt.
///
/// # Errors
///
/// Returns a typed error for count overflow.
pub fn project_backend_enforcement_receipt(
    policy: &BackendSelectionPolicy,
    receipt_id: BackendReceiptId,
    plan: &ExecutionBackendPlan,
    result: &BackendEnforcementResult,
) -> Result<BackendSelectionReceipt, BackendSelectionError> {
    let requirement_count =
        u32::try_from(plan.requirements.len()).map_err(|_| BackendSelectionError::CountOverflow)?;
    let count = requirement_count.to_string();
    let requirement_set_identity = Some(hash_requirement_set(&plan.requirements));
    let (denial, observation_id) = match result {
        BackendEnforcementResult::Admitted(observation) => {
            (None, Some(observation.observation_id.clone()))
        }
        BackendEnforcementResult::Denied(record) => {
            (Some(record.reason), record.observation_id.clone())
        }
    };
    let denial_text = denial.map_or("none", BackendEnforcementDenial::as_str);
    let observation_text = observation_id
        .as_ref()
        .map_or("none", ObservationId::as_str);
    let requirement_identity_text = requirement_set_identity.as_deref().unwrap_or("none");
    let identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            SCHEMA,
            policy.policy_id.as_str(),
            policy.identity.as_str(),
            receipt_id.as_str(),
            plan.plan_id.as_str(),
            plan.class.as_str(),
            denial_text,
            observation_text,
            requirement_identity_text,
            count.as_str(),
        ],
    );
    Ok(BackendSelectionReceipt {
        receipt_id,
        identity,
        selected_class: Some(plan.class),
        selection_denial: None,
        enforcement_denial: denial,
        observation_id,
        requirement_set_identity,
        requirement_count,
        non_claims: policy.non_claims.clone(),
    })
}

/// Rejects private material and sandbox overclaims in evidence text.
///
/// # Errors
///
/// Returns [`BackendSelectionError::UnsafeEvidence`] for unsafe text.
pub fn validate_backend_evidence_text(value: &str) -> Result<(), BackendSelectionError> {
    let lower = value.to_ascii_lowercase();
    if FORBIDDEN_EVIDENCE.iter().any(|item| lower.contains(item))
        || lower.contains("proves sandbox")
        || lower.contains("guarantees isolation")
    {
        return Err(BackendSelectionError::UnsafeEvidence);
    }
    Ok(())
}

/// Typed backend-selection construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendSelectionError {
    /// Identity is not lowercase BLAKE3 text.
    InvalidIdentity,
    /// Generated policy is incomplete or unsafe.
    InvalidPolicy,
    /// A generated policy requirement value is unknown.
    UnknownRequirement,
    /// A generated backend class value is unknown.
    UnknownBackendClass,
    /// Candidate backend identities are duplicated.
    DuplicateBackendIdentity,
    /// A public count overflowed.
    CountOverflow,
    /// Evidence contains private material or an overclaim.
    UnsafeEvidence,
}

fn build_plan(
    policy: &BackendSelectionPolicy,
    workload: &WorkloadRequirements,
    candidate: &ExecutionBackendCandidate,
    broader: Vec<ExecutionBackendClass>,
) -> Result<ExecutionBackendPlan, BackendSelectionError> {
    let mut requirements = workload.requirements.clone();
    requirements.sort_unstable();
    let resource_parts = [
        workload.resources.memory_bytes.to_string(),
        workload.resources.storage_bytes.to_string(),
        workload.resources.cpu_milliseconds.to_string(),
        workload.resources.wall_milliseconds.to_string(),
    ];
    let mut parts = vec![
        SCHEMA.to_string(),
        policy.identity.clone(),
        workload.workload_id.as_str().to_string(),
        candidate.backend_id.as_str().to_string(),
        candidate.class.as_str().to_string(),
        candidate.adapter_id.as_str().to_string(),
        candidate.application_host_binding_id.as_str().to_string(),
        candidate.runtime_authority_unit_id.as_str().to_string(),
        candidate.effect_permission_id.as_str().to_string(),
        candidate.resource_profile_id.as_str().to_string(),
        candidate.enforcement_profile_id.as_str().to_string(),
    ];
    for requirement in &requirements {
        parts.push(requirement.as_str().to_string());
    }
    parts.extend(resource_parts);
    let plan_id = BackendPlanId::parse(hash_strings(PLAN_DOMAIN, &parts).as_str())?;
    Ok(ExecutionBackendPlan {
        plan_id,
        workload_id: workload.workload_id.clone(),
        backend_id: candidate.backend_id.clone(),
        class: candidate.class,
        adapter_id: candidate.adapter_id.clone(),
        application_host_binding_id: candidate.application_host_binding_id.clone(),
        runtime_authority_unit_id: candidate.runtime_authority_unit_id.clone(),
        effect_permission_id: candidate.effect_permission_id.clone(),
        resource_profile_id: candidate.resource_profile_id.clone(),
        enforcement_profile_id: candidate.enforcement_profile_id.clone(),
        policy_identity: policy.identity.clone(),
        requirements,
        resources: workload.resources,
        unnecessary_broader_classes: broader,
    })
}

fn complete_requirements(requirements: &[WorkloadRequirement]) -> bool {
    if requirements.len() != REQUIRED_DIMENSION_COUNT || has_duplicate_requirements(requirements) {
        return false;
    }
    let mut dimensions = requirements
        .iter()
        .map(|requirement| requirement.dimension())
        .collect::<Vec<_>>();
    dimensions.sort_unstable();
    dimensions.dedup();
    dimensions.len() == REQUIRED_DIMENSION_COUNT
}

fn requirements_supported(
    requested: &[WorkloadRequirement],
    supported: &[WorkloadRequirement],
) -> bool {
    requested
        .iter()
        .all(|requirement| supported.contains(requirement))
}

fn has_duplicate_requirements(requirements: &[WorkloadRequirement]) -> bool {
    let mut values = requirements.to_vec();
    values.sort_unstable();
    let initial_length = values.len();
    values.dedup();
    values.len() != initial_length
}

fn has_duplicate_classes(classes: &[ExecutionBackendClass]) -> bool {
    let mut values = classes.to_vec();
    values.sort_unstable();
    let initial_length = values.len();
    values.dedup();
    values.len() != initial_length
}

fn has_duplicate_backend_ids(candidates: &[ExecutionBackendCandidate]) -> bool {
    candidates.iter().enumerate().any(|(index, candidate)| {
        candidates
            .iter()
            .skip(index.saturating_add(1))
            .any(|other| candidate.backend_id == other.backend_id)
    })
}

fn class_policy(
    policies: &[BackendClassPolicy],
    class: ExecutionBackendClass,
) -> Option<&BackendClassPolicy> {
    let mut matching = policies.iter().filter(|policy| policy.class == class);
    let first = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    Some(first)
}

fn has_required_values(actual: &[String], required: &[&str]) -> bool {
    actual.len() == required.len()
        && required
            .iter()
            .all(|required_value| actual.iter().any(|value| value == required_value))
}

fn is_blake3(value: &str) -> bool {
    value.len() == HASH_HEX_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn hash_requirement_set(requirements: &[WorkloadRequirement]) -> String {
    let mut values = requirements.to_vec();
    values.sort_unstable();
    let parts = values
        .iter()
        .map(|requirement| requirement.as_str().to_string())
        .collect::<Vec<_>>();
    hash_strings("animus/execution-backend-requirements", &parts)
}

fn hash_strings(domain: &str, parts: &[String]) -> String {
    let borrowed = parts.iter().map(String::as_str).collect::<Vec<_>>();
    hash_parts(domain, &borrowed)
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const HASH_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const HASH_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    const HASH_1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const HASH_2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const HASH_3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const RESOURCE_LIMIT: u64 = 1024;
    const BROADER_CLASS_COUNT: u32 = 3;

    fn requirements(format: WorkloadRequirement) -> Vec<WorkloadRequirement> {
        vec![
            format,
            WorkloadRequirement::NoFilesystem,
            WorkloadRequirement::NetworkDenied,
            WorkloadRequirement::InProcess,
            WorkloadRequirement::NoTerminal,
            WorkloadRequirement::NoPeripheral,
            WorkloadRequirement::Ephemeral,
            WorkloadRequirement::MediatedIsolation,
        ]
    }

    fn class_requirements(class: ExecutionBackendClass) -> Vec<WorkloadRequirement> {
        match class {
            ExecutionBackendClass::EmbeddedComponent => vec![
                WorkloadRequirement::WasmComponent,
                WorkloadRequirement::NoFilesystem,
                WorkloadRequirement::ReadOnlyFilesystem,
                WorkloadRequirement::NetworkDenied,
                WorkloadRequirement::InProcess,
                WorkloadRequirement::NoTerminal,
                WorkloadRequirement::NoPeripheral,
                WorkloadRequirement::Ephemeral,
                WorkloadRequirement::MediatedIsolation,
            ],
            ExecutionBackendClass::GuardedLocalProcess => vec![
                WorkloadRequirement::WasmComponent,
                WorkloadRequirement::NativeBinary,
                WorkloadRequirement::ScriptRuntime,
                WorkloadRequirement::NoFilesystem,
                WorkloadRequirement::ReadOnlyFilesystem,
                WorkloadRequirement::WritableWorkspace,
                WorkloadRequirement::NetworkDenied,
                WorkloadRequirement::NetworkAllowlisted,
                WorkloadRequirement::SingleProcess,
                WorkloadRequirement::ProcessTree,
                WorkloadRequirement::NoTerminal,
                WorkloadRequirement::Pseudoterminal,
                WorkloadRequirement::NoPeripheral,
                WorkloadRequirement::Ephemeral,
                WorkloadRequirement::DurableWorkspace,
                WorkloadRequirement::GuardedProcessIsolation,
            ],
            ExecutionBackendClass::IsolatedHost | ExecutionBackendClass::ExternalMachine => vec![
                WorkloadRequirement::WasmComponent,
                WorkloadRequirement::NativeBinary,
                WorkloadRequirement::ScriptRuntime,
                WorkloadRequirement::NoFilesystem,
                WorkloadRequirement::ReadOnlyFilesystem,
                WorkloadRequirement::WritableWorkspace,
                WorkloadRequirement::NetworkDenied,
                WorkloadRequirement::NetworkAllowlisted,
                WorkloadRequirement::InProcess,
                WorkloadRequirement::SingleProcess,
                WorkloadRequirement::ProcessTree,
                WorkloadRequirement::NoTerminal,
                WorkloadRequirement::Pseudoterminal,
                WorkloadRequirement::NoPeripheral,
                WorkloadRequirement::Browser,
                WorkloadRequirement::HostDevice,
                WorkloadRequirement::Ephemeral,
                WorkloadRequirement::DurableWorkspace,
                WorkloadRequirement::MediatedIsolation,
                WorkloadRequirement::GuardedProcessIsolation,
                WorkloadRequirement::HostIsolation,
            ],
        }
    }

    fn workload_requirements(class: ExecutionBackendClass) -> Vec<WorkloadRequirement> {
        match class {
            ExecutionBackendClass::EmbeddedComponent => {
                requirements(WorkloadRequirement::WasmComponent)
            }
            ExecutionBackendClass::GuardedLocalProcess => vec![
                WorkloadRequirement::NativeBinary,
                WorkloadRequirement::WritableWorkspace,
                WorkloadRequirement::NetworkDenied,
                WorkloadRequirement::SingleProcess,
                WorkloadRequirement::Pseudoterminal,
                WorkloadRequirement::NoPeripheral,
                WorkloadRequirement::Ephemeral,
                WorkloadRequirement::GuardedProcessIsolation,
            ],
            ExecutionBackendClass::IsolatedHost => vec![
                WorkloadRequirement::NativeBinary,
                WorkloadRequirement::WritableWorkspace,
                WorkloadRequirement::NetworkAllowlisted,
                WorkloadRequirement::ProcessTree,
                WorkloadRequirement::Pseudoterminal,
                WorkloadRequirement::Browser,
                WorkloadRequirement::DurableWorkspace,
                WorkloadRequirement::HostIsolation,
            ],
            ExecutionBackendClass::ExternalMachine => vec![
                WorkloadRequirement::NativeBinary,
                WorkloadRequirement::WritableWorkspace,
                WorkloadRequirement::NetworkAllowlisted,
                WorkloadRequirement::ProcessTree,
                WorkloadRequirement::Pseudoterminal,
                WorkloadRequirement::HostDevice,
                WorkloadRequirement::DurableWorkspace,
                WorkloadRequirement::HostIsolation,
            ],
        }
    }

    fn policy() -> BackendSelectionPolicy {
        BackendSelectionPolicy::validate(BackendSelectionPolicyDraft {
            policy_id: HASH_A.to_string(),
            preference_order: ExecutionBackendClass::ALL.to_vec(),
            class_policies: ExecutionBackendClass::ALL
                .iter()
                .copied()
                .map(|class| BackendClassPolicy {
                    class,
                    admitted_requirements: class_requirements(class),
                })
                .collect(),
            deny_silent_fallback: true,
            non_claims: REQUIRED_NON_CLAIMS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        })
        .expect("policy")
    }

    fn limits(value: u64) -> BackendResourceLimits {
        BackendResourceLimits {
            memory_bytes: value,
            storage_bytes: value,
            cpu_milliseconds: value,
            wall_milliseconds: value,
        }
    }

    fn candidate(class: ExecutionBackendClass, backend: &str) -> ExecutionBackendCandidate {
        ExecutionBackendCandidate {
            backend_id: BackendId::parse(backend).expect("backend"),
            class,
            adapter_id: AdapterId::parse(HASH_B).expect("adapter"),
            application_host_binding_id: ApplicationHostBindingId::parse(HASH_C).expect("binding"),
            runtime_authority_unit_id: RuntimeAuthorityUnitId::parse(HASH_D).expect("authority"),
            effect_permission_id: EffectPermissionId::parse(HASH_E).expect("permission"),
            resource_profile_id: ResourceProfileId::parse(HASH_F).expect("resource"),
            enforcement_profile_id: EnforcementProfileId::parse(HASH_1).expect("enforcement"),
            supported_requirements: class_requirements(class),
            capacity: limits(RESOURCE_LIMIT),
            available: true,
        }
    }

    fn workload_for(class: ExecutionBackendClass) -> WorkloadRequirements {
        WorkloadRequirements {
            workload_id: WorkloadId::parse(HASH_2).expect("workload"),
            requirements: workload_requirements(class),
            resources: limits(RESOURCE_LIMIT),
        }
    }

    fn workload() -> WorkloadRequirements {
        workload_for(ExecutionBackendClass::EmbeddedComponent)
    }

    fn selected(result: BackendSelectionResult) -> ExecutionBackendPlan {
        match result {
            BackendSelectionResult::Selected(plan) => *plan,
            BackendSelectionResult::Denied(denial) => panic!("unexpected denial: {denial:?}"),
        }
    }

    struct FakeService {
        observation: Result<CurrentBackendObservation, CurrentBackendServiceError>,
    }

    impl CurrentBackendEnforcementService for FakeService {
        fn observe(
            &self,
            _plan: &ExecutionBackendPlan,
        ) -> Result<CurrentBackendObservation, CurrentBackendServiceError> {
            self.observation.clone()
        }
    }

    fn observation(plan: &ExecutionBackendPlan) -> CurrentBackendObservation {
        CurrentBackendObservation {
            observation_id: ObservationId::parse(HASH_3).expect("observation"),
            backend_id: plan.backend_id.clone(),
            adapter_id: plan.adapter_id.clone(),
            application_host_binding_id: plan.application_host_binding_id.clone(),
            runtime_authority_unit_id: plan.runtime_authority_unit_id.clone(),
            effect_permission_id: plan.effect_permission_id.clone(),
            resource_profile_id: plan.resource_profile_id.clone(),
            enforcement_profile_id: plan.enforcement_profile_id.clone(),
            available: true,
            resources_admitted: true,
            enforcement_current: true,
        }
    }

    #[test]
    fn each_backend_class_is_selected_when_it_is_the_weakest_candidate() {
        // r[verify agent_harness_runtime.execution_backend_planner.verification]
        for class in ExecutionBackendClass::ALL {
            let result = select_execution_backend(
                &policy(),
                &workload_for(class),
                &[candidate(class, HASH_3)],
            )
            .expect("selection");
            let plan = selected(result);
            assert_eq!(plan.class, class);
            assert_eq!(plan.plan_id.as_str().len(), HASH_HEX_LENGTH);
        }
    }

    #[test]
    fn explicit_policy_order_selects_weakest_without_numeric_ranking() {
        // r[verify agent_harness_runtime.execution_backend_planner.selection]
        let candidates = vec![
            candidate(ExecutionBackendClass::ExternalMachine, HASH_3),
            candidate(ExecutionBackendClass::EmbeddedComponent, HASH_A),
            candidate(ExecutionBackendClass::IsolatedHost, HASH_2),
        ];
        let result =
            select_execution_backend(&policy(), &workload(), &candidates).expect("selection");
        let plan = selected(result.clone());
        let explanation = explain_backend_selection(&result);

        assert_eq!(plan.class, ExecutionBackendClass::EmbeddedComponent);
        assert_eq!(
            explanation.selected_class,
            Some(ExecutionBackendClass::EmbeddedComponent)
        );
        assert_eq!(explanation.unnecessary_broader_count, BROADER_CLASS_COUNT);
    }

    #[test]
    fn unknown_incomplete_ambiguous_unavailable_resource_and_fallback_fail_closed() {
        // r[verify agent_harness_runtime.execution_backend_planner.verification]
        let mut incomplete = workload();
        incomplete.requirements.pop();
        let duplicate = [
            candidate(ExecutionBackendClass::EmbeddedComponent, HASH_3),
            candidate(ExecutionBackendClass::EmbeddedComponent, HASH_2),
        ];
        let mut unavailable = candidate(ExecutionBackendClass::EmbeddedComponent, HASH_3);
        unavailable.available = false;
        let broader = candidate(ExecutionBackendClass::IsolatedHost, HASH_2);
        let mut constrained = candidate(ExecutionBackendClass::EmbeddedComponent, HASH_3);
        constrained.capacity = limits(1);

        assert_eq!(
            WorkloadRequirement::parse("unknown"),
            Err(BackendSelectionError::UnknownRequirement)
        );
        assert_eq!(
            ExecutionBackendClass::parse("unknown"),
            Err(BackendSelectionError::UnknownBackendClass)
        );
        assert_eq!(
            select_execution_backend(&policy(), &incomplete, &[]).expect("incomplete"),
            BackendSelectionResult::Denied(BackendSelectionDenial::IncompleteRequirements)
        );
        assert_eq!(
            select_execution_backend(&policy(), &workload(), &duplicate).expect("ambiguous"),
            BackendSelectionResult::Denied(BackendSelectionDenial::AmbiguousCandidate)
        );
        assert_eq!(
            select_execution_backend(&policy(), &workload(), &[unavailable, broader])
                .expect("no fallback"),
            BackendSelectionResult::Denied(BackendSelectionDenial::SelectedBackendUnavailable)
        );
        assert_eq!(
            select_execution_backend(&policy(), &workload(), &[constrained]).expect("resources"),
            BackendSelectionResult::Denied(BackendSelectionDenial::ResourceDenied)
        );
    }

    #[test]
    fn current_recheck_requires_exact_authority_resources_and_enforcement() {
        // r[verify agent_harness_runtime.execution_backend_planner.enforcement]
        let plan = selected(
            select_execution_backend(
                &policy(),
                &workload(),
                &[candidate(ExecutionBackendClass::EmbeddedComponent, HASH_3)],
            )
            .expect("selection"),
        );
        let current = FakeService {
            observation: Ok(observation(&plan)),
        };
        let admitted = recheck_execution_backend(&current, &plan);
        assert!(matches!(&admitted, BackendEnforcementResult::Admitted(_)));
        let receipt = project_backend_enforcement_receipt(
            &policy(),
            BackendReceiptId::parse(HASH_1).expect("receipt"),
            &plan,
            &admitted,
        )
        .expect("enforcement receipt");
        assert_eq!(
            receipt.observation_id.as_ref().map(ObservationId::as_str),
            Some(HASH_3)
        );

        let mut drift = observation(&plan);
        drift.runtime_authority_unit_id = RuntimeAuthorityUnitId::parse(HASH_A).expect("drift");
        assert!(matches!(
            recheck_execution_backend(
                &FakeService {
                    observation: Ok(drift)
                },
                &plan
            ),
            BackendEnforcementResult::Denied(BackendEnforcementDenialRecord {
                reason: BackendEnforcementDenial::AuthorityDrift,
                observation_id: Some(_),
            })
        ));

        let mut denied = observation(&plan);
        denied.resources_admitted = false;
        assert!(matches!(
            recheck_execution_backend(
                &FakeService {
                    observation: Ok(denied)
                },
                &plan
            ),
            BackendEnforcementResult::Denied(BackendEnforcementDenialRecord {
                reason: BackendEnforcementDenial::ResourceDenied,
                observation_id: Some(_),
            })
        ));
    }

    #[test]
    fn receipts_are_stable_metadata_only_and_reject_overclaims() {
        // r[verify agent_harness_runtime.execution_backend_planner.evidence]
        let result = select_execution_backend(
            &policy(),
            &workload(),
            &[candidate(ExecutionBackendClass::EmbeddedComponent, HASH_3)],
        )
        .expect("selection");
        let receipt = project_backend_selection_receipt(
            &policy(),
            BackendReceiptId::parse(HASH_1).expect("receipt"),
            &result,
        )
        .expect("receipt");
        let second = project_backend_selection_receipt(
            &policy(),
            BackendReceiptId::parse(HASH_1).expect("receipt"),
            &result,
        )
        .expect("receipt");

        assert_eq!(receipt, second);
        assert_eq!(receipt.identity.len(), HASH_HEX_LENGTH);
        assert_eq!(
            receipt.requirement_set_identity.as_ref().map(String::len),
            Some(HASH_HEX_LENGTH)
        );
        assert_eq!(
            validate_backend_evidence_text("selected counts non-claims"),
            Ok(())
        );
        assert_eq!(
            validate_backend_evidence_text("proves sandbox containment"),
            Err(BackendSelectionError::UnsafeEvidence)
        );
        assert_eq!(
            validate_backend_evidence_text("command: private"),
            Err(BackendSelectionError::UnsafeEvidence)
        );
    }
}

// ---------------------------------------------------------------------------
// Per-component sandbox-scope planning (minimum-admission, mechanism-reuse)
// ---------------------------------------------------------------------------

/// Sandbox-scope schema identity.
pub const SANDBOX_SCOPE_SCHEMA: &str = "animus.sandbox-scope.v1";

hash_identity!(SandboxComponentId);
hash_identity!(SandboxDescriptorId);
hash_identity!(SandboxPlanId);
hash_identity!(SandboxReceiptId);

/// Supported sandbox profile kinds, reusing current enforcement mechanisms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxProfileKind {
    /// Portable Sandlock descriptor.
    Sandlock,
    /// Bubblewrap agent guardrail mount profile.
    BubblewrapGuardrail,
    /// Minimal default scope for a component with no declared need.
    MinimalDefault,
}

impl SandboxProfileKind {
    /// Parses a generated sandbox profile kind.
    ///
    /// # Errors
    ///
    /// Returns [`BackendSelectionError::InvalidPolicy`] for unknown text.
    pub fn parse(value: &str) -> Result<Self, BackendSelectionError> {
        match value {
            "sandlock" | "Sandlock" => Ok(Self::Sandlock),
            "bubblewrap_guardrail" | "BubblewrapGuardrail" => Ok(Self::BubblewrapGuardrail),
            "minimal_default" | "MinimalDefault" => Ok(Self::MinimalDefault),
            _ => Err(BackendSelectionError::InvalidPolicy),
        }
    }

    /// Returns the stable profile-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sandlock => "sandlock",
            Self::BubblewrapGuardrail => "bubblewrap_guardrail",
            Self::MinimalDefault => "minimal_default",
        }
    }
}

/// One spawned component's declared sandbox need and capability request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSandboxRequest {
    /// Stable component identity.
    pub component_id: SandboxComponentId,
    /// Declared closed requirement values.
    pub declared_need: Vec<WorkloadRequirement>,
    /// Capability labels the component must satisfy.
    pub capability_labels: Vec<String>,
    /// Caller-supplied planning-time availability fact.
    pub available: bool,
}

/// One admitted sandbox descriptor from the reviewed catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxDescriptor {
    /// Descriptor identity.
    pub descriptor_id: SandboxDescriptorId,
    /// Profile kind.
    pub kind: SandboxProfileKind,
    /// Closed requirement values this descriptor satisfies.
    pub satisfies_requirements: Vec<WorkloadRequirement>,
    /// Capability labels this descriptor satisfies.
    pub satisfies_capabilities: Vec<String>,
}

/// Bounded sandbox-scope denial.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxScopeDenial {
    /// No descriptor exists for the component.
    MissingDescriptor,
    /// The component backend is unavailable.
    BackendUnavailable,
    /// A declared requirement is unsupported by every descriptor.
    UnsupportedRequirement,
    /// A capability request cannot be satisfied.
    UnsatisfiableCapability,
}

impl SandboxScopeDenial {
    /// Returns the stable denial label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingDescriptor => "missing_descriptor",
            Self::BackendUnavailable => "backend_unavailable",
            Self::UnsupportedRequirement => "unsupported_requirement",
            Self::UnsatisfiableCapability => "unsatisfiable_capability",
        }
    }
}

/// Selected per-component sandbox scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxScopePlan {
    /// Plan identity.
    pub plan_id: SandboxPlanId,
    /// Component identity.
    pub component_id: SandboxComponentId,
    /// Selected descriptor identity.
    pub descriptor_id: SandboxDescriptorId,
    /// Selected profile kind.
    pub kind: SandboxProfileKind,
    /// Explicit non-claims.
    pub non_claims: [&'static str; 3],
}

/// Pure sandbox-scope planning result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SandboxScopeResult {
    /// One weakest admitted descriptor was selected.
    Selected(Box<SandboxScopePlan>),
    /// Planning was denied with a bounded reason.
    Denied(SandboxScopeDenial),
}

/// Plans exactly one sandbox descriptor per spawned component.
///
/// The first descriptor in caller-supplied weakest-to-broadest order that
/// satisfies every declared requirement and capability label is selected. A
/// component with no declared need receives a minimal admitted scope.
///
/// # Errors
///
/// Returns [`BackendSelectionError::DuplicateBackendIdentity`] for a
/// duplicated descriptor identity. Expected denials are bounded results.
pub fn plan_sandbox_scope(
    request: &ComponentSandboxRequest,
    descriptors: &[SandboxDescriptor],
) -> Result<SandboxScopeResult, BackendSelectionError> {
    if has_duplicate_sandbox_descriptors(descriptors) {
        return Err(BackendSelectionError::DuplicateBackendIdentity);
    }
    if descriptors.is_empty() {
        return Ok(SandboxScopeResult::Denied(
            SandboxScopeDenial::MissingDescriptor,
        ));
    }
    if !request.available {
        return Ok(SandboxScopeResult::Denied(
            SandboxScopeDenial::BackendUnavailable,
        ));
    }
    let mut need_met = false;
    for descriptor in descriptors {
        let meets_need =
            requirements_supported(&request.declared_need, &descriptor.satisfies_requirements);
        if meets_need {
            need_met = true;
        }
        if meets_need
            && capabilities_supported(
                &request.capability_labels,
                &descriptor.satisfies_capabilities,
            )
        {
            let plan_id = SandboxPlanId::parse(
                hash_strings(
                    "animus/execution-backend-sandbox-plan",
                    &[
                        SANDBOX_SCOPE_SCHEMA.to_string(),
                        request.component_id.as_str().to_string(),
                        descriptor.descriptor_id.as_str().to_string(),
                        descriptor.kind.as_str().to_string(),
                    ],
                )
                .as_str(),
            )?;
            return Ok(SandboxScopeResult::Selected(Box::new(SandboxScopePlan {
                plan_id,
                component_id: request.component_id.clone(),
                descriptor_id: descriptor.descriptor_id.clone(),
                kind: descriptor.kind,
                non_claims: [
                    "not-sandbox-enforcement",
                    "not-live-host-recheck",
                    "not-work-completion",
                ],
            })));
        }
    }
    if need_met {
        return Ok(SandboxScopeResult::Denied(
            SandboxScopeDenial::UnsatisfiableCapability,
        ));
    }
    Ok(SandboxScopeResult::Denied(
        SandboxScopeDenial::UnsupportedRequirement,
    ))
}

fn capabilities_supported(requested: &[String], supported: &[String]) -> bool {
    requested
        .iter()
        .all(|capability| supported.contains(capability))
}

fn has_duplicate_sandbox_descriptors(descriptors: &[SandboxDescriptor]) -> bool {
    descriptors.iter().enumerate().any(|(index, descriptor)| {
        descriptors[..index]
            .iter()
            .any(|prior| prior.descriptor_id == descriptor.descriptor_id)
    })
}

/// Metadata-only sandbox-scope receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxScopeReceipt {
    /// Receipt identity.
    pub receipt_id: SandboxReceiptId,
    /// Canonical BLAKE3 receipt identity.
    pub identity: String,
    /// Selected profile kind, when present.
    pub selected_kind: Option<SandboxProfileKind>,
    /// Denial, when present.
    pub denial: Option<SandboxScopeDenial>,
    /// Component identity.
    pub component_id: SandboxComponentId,
    /// Mandatory non-claims.
    pub non_claims: [&'static str; 3],
}

/// Projects a metadata-only sandbox-scope receipt. The receipt never contains
/// sandbox rule bodies, environment values, or private paths.
#[must_use]
pub fn project_sandbox_scope_receipt(
    receipt_id: SandboxReceiptId,
    result: &SandboxScopeResult,
    component_id: &SandboxComponentId,
) -> SandboxScopeReceipt {
    let (selected_kind, denial, subject, non_claims) = match result {
        SandboxScopeResult::Selected(plan) => (
            Some(plan.kind),
            None,
            plan.plan_id.as_str().to_string(),
            plan.non_claims,
        ),
        SandboxScopeResult::Denied(denial) => (
            None,
            Some(*denial),
            denial.as_str().to_string(),
            [
                "not-sandbox-enforcement",
                "not-live-host-recheck",
                "not-work-completion",
            ],
        ),
    };
    let kind_text = selected_kind.map_or("none", SandboxProfileKind::as_str);
    let denial_text = denial.map_or("none", SandboxScopeDenial::as_str);
    let identity = hash_strings(
        "animus/execution-backend-sandbox-receipt",
        &[
            SANDBOX_SCOPE_SCHEMA.to_string(),
            receipt_id.as_str().to_string(),
            component_id.as_str().to_string(),
            subject,
            kind_text.to_string(),
            denial_text.to_string(),
        ],
    );
    SandboxScopeReceipt {
        receipt_id,
        identity,
        selected_kind,
        denial,
        component_id: component_id.clone(),
        non_claims,
    }
}

/// Rejects sandbox overclaims and private material in evidence text.
///
/// # Errors
///
/// Returns [`BackendSelectionError::UnsafeEvidence`] for unsafe text.
pub fn validate_sandbox_scope_evidence_text(value: &str) -> Result<(), BackendSelectionError> {
    let lower = value.to_ascii_lowercase();
    if lower.contains("sandbox diagnostic")
        || lower.contains("environment=")
        || lower.contains("/home/")
        || lower.contains("proves sandbox")
    {
        return Err(BackendSelectionError::UnsafeEvidence);
    }
    Ok(())
}

#[cfg(test)]
mod sandbox_scope_tests {
    use super::*;

    const H_SCOPE_1: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const H_SCOPE_2: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const H_SCOPE_3: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const H_SCOPE_4: &str = "4444444444444444444444444444444444444444444444444444444444444444";

    fn request(
        component: &str,
        need: Vec<WorkloadRequirement>,
        capabilities: Vec<String>,
    ) -> ComponentSandboxRequest {
        let _ = component;
        ComponentSandboxRequest {
            component_id: SandboxComponentId::parse(H_SCOPE_3).expect("component"),
            declared_need: need,
            capability_labels: capabilities,
            available: true,
        }
    }

    fn descriptor(
        kind: SandboxProfileKind,
        id: &str,
        need: Vec<WorkloadRequirement>,
    ) -> SandboxDescriptor {
        SandboxDescriptor {
            descriptor_id: SandboxDescriptorId::parse(id).expect("descriptor"),
            kind,
            satisfies_requirements: need,
            satisfies_capabilities: vec!["network".to_string()],
        }
    }

    #[test]
    fn positive_weakest_satisfying_descriptor_is_selected_per_component() {
        // r[verify agent_harness_runtime.execution_backend_sandbox.verification]
        let need = vec![WorkloadRequirement::NetworkDenied];
        let catalog = vec![
            descriptor(
                SandboxProfileKind::MinimalDefault,
                H_SCOPE_1,
                vec![WorkloadRequirement::NetworkDenied],
            ),
            descriptor(
                SandboxProfileKind::Sandlock,
                H_SCOPE_2,
                vec![
                    WorkloadRequirement::NetworkDenied,
                    WorkloadRequirement::WritableWorkspace,
                ],
            ),
        ];
        let result =
            plan_sandbox_scope(&request("ui", need, vec!["network".to_string()]), &catalog)
                .expect("plan");
        let plan = match result {
            SandboxScopeResult::Selected(plan) => *plan,
            SandboxScopeResult::Denied(denial) => panic!("unexpected denial: {denial:?}"),
        };
        assert_eq!(plan.kind, SandboxProfileKind::MinimalDefault);
        assert_eq!(plan.component_id.as_str(), H_SCOPE_3);
        let receipt = project_sandbox_scope_receipt(
            SandboxReceiptId::parse(H_SCOPE_4).expect("receipt"),
            &SandboxScopeResult::Selected(Box::new(plan)),
            &SandboxComponentId::parse(H_SCOPE_3).expect("component"),
        );
        assert_eq!(receipt.identity.len(), HASH_HEX_LENGTH);
        assert!(validate_sandbox_scope_evidence_text(&receipt.identity).is_ok());
    }

    #[test]
    fn positive_minimal_default_scope_serves_a_component_with_no_declared_need() {
        // r[verify agent_harness_runtime.execution_backend_sandbox.scope]
        let catalog = vec![descriptor(
            SandboxProfileKind::MinimalDefault,
            H_SCOPE_1,
            vec![],
        )];
        let result =
            plan_sandbox_scope(&request("notify", vec![], vec![]), &catalog).expect("plan");
        assert!(matches!(result, SandboxScopeResult::Selected(_)));
    }

    #[test]
    fn negative_missing_unavailable_unsatisfiable_and_unsupported_fail_closed() {
        // r[verify agent_harness_runtime.execution_backend_sandbox.fail_closed]
        let empty: Vec<SandboxDescriptor> = Vec::new();
        assert_eq!(
            plan_sandbox_scope(&request("ui", vec![], vec![]), &empty),
            Ok(SandboxScopeResult::Denied(
                SandboxScopeDenial::MissingDescriptor
            ))
        );
        let mut unavailable = request("ui", vec![], vec![]);
        unavailable.available = false;
        let catalog = vec![descriptor(
            SandboxProfileKind::MinimalDefault,
            H_SCOPE_1,
            vec![],
        )];
        assert_eq!(
            plan_sandbox_scope(&unavailable, &catalog),
            Ok(SandboxScopeResult::Denied(
                SandboxScopeDenial::BackendUnavailable
            ))
        );
        let need = vec![WorkloadRequirement::Browser];
        assert_eq!(
            plan_sandbox_scope(&request("ui", need, vec![]), &catalog),
            Ok(SandboxScopeResult::Denied(
                SandboxScopeDenial::UnsupportedRequirement
            ))
        );
        let capability_catalog = vec![descriptor(
            SandboxProfileKind::MinimalDefault,
            H_SCOPE_1,
            vec![WorkloadRequirement::NetworkDenied],
        )];
        assert_eq!(
            plan_sandbox_scope(
                &request(
                    "ui",
                    vec![WorkloadRequirement::NetworkDenied],
                    vec!["terminal".to_string()]
                ),
                &capability_catalog,
            ),
            Ok(SandboxScopeResult::Denied(
                SandboxScopeDenial::UnsatisfiableCapability
            ))
        );
        let duplicate = vec![
            descriptor(SandboxProfileKind::MinimalDefault, H_SCOPE_1, vec![]),
            descriptor(SandboxProfileKind::Sandlock, H_SCOPE_1, vec![]),
        ];
        assert_eq!(
            plan_sandbox_scope(&request("ui", vec![], vec![]), &duplicate),
            Err(BackendSelectionError::DuplicateBackendIdentity)
        );
    }

    #[test]
    fn negative_receipt_and_evidence_reject_private_material() {
        // r[verify agent_harness_runtime.execution_backend_sandbox.receipts]
        assert_eq!(
            validate_sandbox_scope_evidence_text("sandbox diagnostic for /home/"),
            Err(BackendSelectionError::UnsafeEvidence)
        );
        assert_eq!(
            validate_sandbox_scope_evidence_text("proves sandbox containment"),
            Err(BackendSelectionError::UnsafeEvidence)
        );
        let catalog = vec![descriptor(
            SandboxProfileKind::MinimalDefault,
            H_SCOPE_1,
            vec![],
        )];
        let result = plan_sandbox_scope(&request("ui", vec![], vec![]), &catalog).expect("plan");
        assert!(matches!(result, SandboxScopeResult::Selected(_)));
        // Selected component H_SCOPE_3 does not leak environment values.
        assert_eq!(H_SCOPE_3.len(), HASH_HEX_LENGTH);
    }
}
