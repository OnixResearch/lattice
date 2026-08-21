#![allow(
    unknown_lints,
    path_segment_repetition,
    reason = "public backend DTO names preserve exact orchestration-domain meaning"
)]

pub const PLANNER_COHORT: &str = "54b2fdaf57ac9bbc34e6c73f75d803d625f84ec0";
pub const PLANNER_SCHEMA: &str = "animus.execution-backend-selection.v1";
pub const ROUTE_SCHEMA: &str = "lattice.operation-backend-route.v1";
pub const ATTEMPT_SCHEMA: &str = "lattice.operation-backend-attempt.v1";

const CAIRN_MEMORY_BYTES: u64 = 67_108_864;
const CAIRN_STORAGE_BYTES: u64 = 4_194_304;
const CAIRN_CPU_MILLISECONDS: u64 = 60_000;
const CAIRN_WALL_MILLISECONDS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionFormat {
    WasmComponent,
    NativeBinary,
    ScriptRuntime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilesystemSemantics {
    NoFilesystem,
    ReadOnlyFilesystem,
    WritableWorkspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPosture {
    NetworkDenied,
    NetworkAllowlisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessModel {
    InProcess,
    SingleProcess,
    ProcessTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalNeed {
    NoTerminal,
    Pseudoterminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeripheralNeed {
    NoPeripheral,
    Browser,
    HostDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceNeed {
    Ephemeral,
    DurableWorkspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationClaim {
    MediatedIsolation,
    GuardedProcessIsolation,
    HostIsolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceRequirements {
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub cpu_milliseconds: u64,
    pub wall_milliseconds: u64,
}

impl ResourceRequirements {
    pub(crate) fn is_positive(self) -> bool {
        self.memory_bytes > 0 && self.storage_bytes > 0 && self.cpu_milliseconds > 0 && self.wall_milliseconds > 0
    }
}

// r[impl lattice.execution_backend_orchestration.contract]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRequirements {
    pub format: ExecutionFormat,
    pub filesystem: FilesystemSemantics,
    pub network: NetworkPosture,
    pub process: ProcessModel,
    pub terminal: TerminalNeed,
    pub peripheral: PeripheralNeed,
    pub persistence: PersistenceNeed,
    pub isolation: IsolationClaim,
    pub resources: ResourceRequirements,
}

impl ExecutionRequirements {
    pub fn cairn_read_only() -> Self {
        Self {
            format: ExecutionFormat::NativeBinary,
            filesystem: FilesystemSemantics::ReadOnlyFilesystem,
            network: NetworkPosture::NetworkDenied,
            process: ProcessModel::SingleProcess,
            terminal: TerminalNeed::NoTerminal,
            peripheral: PeripheralNeed::NoPeripheral,
            persistence: PersistenceNeed::Ephemeral,
            isolation: IsolationClaim::GuardedProcessIsolation,
            resources: ResourceRequirements {
                memory_bytes: CAIRN_MEMORY_BYTES,
                storage_bytes: CAIRN_STORAGE_BYTES,
                cpu_milliseconds: CAIRN_CPU_MILLISECONDS,
                wall_milliseconds: CAIRN_WALL_MILLISECONDS,
            },
        }
    }

    pub fn validate(&self) -> Result<(), crate::operation::Error> {
        if !self.resources.is_positive() {
            return Err(crate::operation::Error::Invalid("operation execution resources must be positive".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendClass {
    EmbeddedComponent,
    GuardedLocalProcess,
    IsolatedHost,
    ExternalMachine,
}

impl BackendClass {
    pub const fn label(self) -> &'static str {
        match self {
            Self::EmbeddedComponent => "embedded_component",
            Self::GuardedLocalProcess => "guarded_local_process",
            Self::IsolatedHost => "isolated_host",
            Self::ExternalMachine => "external_machine",
        }
    }
}

// r[impl lattice.execution_backend_orchestration.planner_mapping]
// r[impl lattice.execution_backend_orchestration.admission]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendRoutePlan {
    pub schema: String,
    pub planner_cohort: String,
    pub planner_schema: String,
    pub planner_plan_identity: String,
    pub requirement_profile_identity: String,
    pub backend_class: BackendClass,
    pub backend_identity: String,
    pub adapter_identity: String,
    pub application_host_binding_identity: String,
    pub runtime_authority_identity: String,
    pub credential_reference_identity: String,
    pub effect_permission_identity: String,
    pub budget_identity: String,
    pub resource_profile_identity: String,
    pub enforcement_profile_identity: String,
    pub persistence_policy_identity: String,
    pub recovery_policy_identity: String,
    pub policy_identity: String,
    pub resources: ResourceRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendAttemptState {
    Admitted,
    Observed,
    Unknown,
}

impl BackendAttemptState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Observed => "observed",
            Self::Unknown => "unknown",
        }
    }
}

// r[impl lattice.execution_backend_orchestration.persistence]
// r[impl lattice.execution_backend_orchestration.dispatch]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendAttemptRecord {
    pub schema: String,
    pub attempt_identity: String,
    pub run_id: crate::engine::executor::RunId,
    pub workflow_revision_identity: String,
    pub node_identity: String,
    pub operation_contract_identity: String,
    pub route: BackendRoutePlan,
    pub state: BackendAttemptState,
}

// r[impl lattice.execution_backend_orchestration.inspection]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SafeBackendView {
    pub attempt_identity: String,
    pub planner_cohort: String,
    pub planner_plan_identity: String,
    pub requirement_profile_identity: String,
    pub backend_class: BackendClass,
    pub backend_identity: String,
    pub adapter_identity: String,
    pub enforcement_profile_identity: String,
    pub state: BackendAttemptState,
}
