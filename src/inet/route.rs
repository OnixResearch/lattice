//! The closed operation contract of the opt-in worker on the existing
//! `bounded_process` route: one type, one `evaluate` method, one launch label,
//! one absolute artifact-root target and exact invocation arguments. No
//! runtime class, workflow node, scheduler or fallback is added; admission,
//! durable start, authority, process execution and publication stay with the
//! existing operation and operation-pack owners.

/// The only operation type of the profile.
pub const TYPE_ID: &str = "lattice.pure-inet-i64";
/// The existing process runtime class.
pub const RUNTIME_KIND: &str = "process";
/// The closed launch label of the pinned Noble executable.
pub const RUNTIME_KEY: &str = "noble-ngraph-net-i64-v1";
/// The only method.
pub const METHOD: &str = "evaluate";
/// The one instance target: the absolute working directory holding the
/// immutable artifacts, admitted by the existing repository-root rules.
pub const ARTIFACT_ROOT_TARGET: &str = crate::operation::runtime::REPOSITORY_ROOT_TARGET;
/// The published result and diagnostic document schema.
pub const RESULT_SCHEMA: &str = "lattice.pure-inet-result.v1";
/// The profile's producer ceilings; invocations may only lower them.
pub const CEILINGS: Ceilings = Ceilings {
    fuel_interactions: 2_097_152,
    max_input_bytes: 1_048_576,
    max_retained_bytes: 67_108_864,
    max_output_bytes: 65_536,
};
/// The process bounds; stdout exceeds the producer output ceiling so the
/// producer refuses before the host would truncate.
pub const BOUNDS: crate::operation::Bounds = crate::operation::Bounds {
    timeout_seconds: 60,
    input_bytes: 4_096,
    output_bytes: 131_072,
    artifact_bytes: 1_048_576,
};

pub mod arguments;
pub mod pack;

const PREFLIGHT: [crate::operation::model::PreflightCheck; 4] = [
    crate::operation::model::PreflightCheck::RuntimeAvailable,
    crate::operation::model::PreflightCheck::ExecutableAvailable,
    crate::operation::model::PreflightCheck::RepositoryRootAdmitted,
    crate::operation::model::PreflightCheck::CredentialReferencesValid,
];
const MAXIMUM_NAME_BYTES: usize = 128;
const IDENTITY_FIELD_BYTES: usize = 80;
const INTEGER_FIELD_BYTES: usize = 24;

pub fn is_runtime(runtime: &crate::operation::model::RuntimeRequirement) -> bool {
    runtime.kind == RUNTIME_KIND && runtime.key == RUNTIME_KEY
}

pub fn uses_runtime(plan: &crate::operation::InvocationPlan) -> bool {
    plan.runtime.kind == RUNTIME_KIND && plan.runtime.key == RUNTIME_KEY
}

/// Execution requirements of the sequential worker; not an OS sandbox claim.
pub fn requirements() -> crate::operation::ExecutionRequirements {
    crate::operation::backend::ExecutionRequirements {
        format: crate::operation::backend::ExecutionFormat::NativeBinary,
        filesystem: crate::operation::backend::FilesystemSemantics::ReadOnlyFilesystem,
        network: crate::operation::backend::NetworkPosture::NetworkDenied,
        process: crate::operation::backend::ProcessModel::SingleProcess,
        terminal: crate::operation::backend::TerminalNeed::NoTerminal,
        peripheral: crate::operation::backend::PeripheralNeed::NoPeripheral,
        persistence: crate::operation::backend::PersistenceNeed::Ephemeral,
        isolation: crate::operation::backend::IsolationClaim::GuardedProcessIsolation,
        resources: crate::operation::backend::ResourceRequirements {
            memory_bytes: 268_435_456,
            storage_bytes: 1_048_576,
            cpu_milliseconds: 60_000,
            wall_milliseconds: 60_000,
        },
    }
}

fn field(
    name: &str,
    value_type: crate::operation::model::ValueType,
    max_bytes: usize,
) -> crate::operation::model::FieldContract {
    crate::operation::model::FieldContract {
        name: name.to_string(),
        value_type,
        required: true,
        max_bytes,
    }
}

fn optional(field: crate::operation::model::FieldContract) -> crate::operation::model::FieldContract {
    crate::operation::model::FieldContract {
        required: false,
        ..field
    }
}

/// The exact argument contract: located artifacts, their identities and
/// lengths, lowered ceilings and the attempt lineage. No snapshot, heap or
/// resume field exists.
pub fn arguments_contract() -> crate::operation::model::RecordContract {
    let string = crate::operation::model::ValueType::String;
    let integer = crate::operation::model::ValueType::Integer;
    const { assert!(MAXIMUM_NAME_BYTES > IDENTITY_FIELD_BYTES) };
    const { assert!(IDENTITY_FIELD_BYTES > INTEGER_FIELD_BYTES) };
    crate::operation::model::RecordContract {
        fields: vec![
            field("attempt", string, MAXIMUM_NAME_BYTES),
            optional(field("consumer_binding", string, IDENTITY_FIELD_BYTES)),
            field("fuel_interactions", integer, INTEGER_FIELD_BYTES),
            field("inputs", string, MAXIMUM_NAME_BYTES),
            field("inputs_bytes", integer, INTEGER_FIELD_BYTES),
            field("inputs_identity", string, IDENTITY_FIELD_BYTES),
            field("max_input_bytes", integer, INTEGER_FIELD_BYTES),
            field("max_output_bytes", integer, INTEGER_FIELD_BYTES),
            field("max_retained_bytes", integer, INTEGER_FIELD_BYTES),
            optional(field("predecessor", string, MAXIMUM_NAME_BYTES)),
            field("program", string, MAXIMUM_NAME_BYTES),
            field("program_bytes", integer, INTEGER_FIELD_BYTES),
            field("program_identity", string, IDENTITY_FIELD_BYTES),
        ],
        deny_unknown_fields: true,
    }
}

/// The exact result/diagnostic document contract.
pub fn output_contract() -> crate::operation::model::RecordContract {
    let string = crate::operation::model::ValueType::String;
    let object = crate::operation::model::ValueType::Object;
    crate::operation::model::RecordContract {
        fields: vec![
            field("authoritative", crate::operation::model::ValueType::Boolean, 8),
            field("binding", object, 4_096),
            optional(field("counters", object, 1_024)),
            field("evaluation", string, 32),
            optional(field("exhaustion", object, 512)),
            field("limits", object, 512),
            field("reason", string, 64),
            optional(field("result", string, 32)),
            field("schema", string, 64),
            optional(field("semantic_interactions", object, 4_096)),
        ],
        deny_unknown_fields: true,
    }
}

fn method() -> crate::operation::model::Method {
    crate::operation::model::Method {
        name: METHOD.to_string(),
        description: "Evaluate one immutable two-input wrapping-i64 program through the pinned Noble net worker"
            .to_string(),
        arguments: arguments_contract(),
        output: output_contract(),
        preflight: PREFLIGHT.to_vec(),
        capabilities: vec![crate::operation::model::CapabilityClass::ProcessRead],
        side_effect: crate::operation::model::SideEffectClass::ReadOnly,
    }
}

/// True only for the exact authored type of this profile.
pub fn type_is_supported(definition: &crate::operation::OperationType) -> bool {
    let is_contract_exact = definition.methods == [method()]
        && definition.capabilities == [crate::operation::model::CapabilityClass::ProcessRead]
        && definition.credential_bindings.is_empty();
    let is_route_exact = definition.id == TYPE_ID
        && is_runtime(&definition.runtime)
        && definition.execution_requirements == requirements();
    is_contract_exact && is_route_exact && definition.bounds == BOUNDS
}

/// The catalog diagnostic for an operation type that names this runtime but
/// differs from the exact authored profile.
pub fn type_diagnostic(
    definition: &crate::operation::OperationType,
    path: &str,
) -> Option<crate::operation::Diagnostic> {
    (!type_is_supported(definition)).then(|| crate::operation::Diagnostic {
        code: crate::operation::DiagnosticCode::RuntimeUnavailable,
        path: format!("{path}.runtime"),
        message: "pure interaction-net operation differs from the exact pinned worker profile".to_string(),
    })
}

/// True only for one absolute artifact-root string.
pub fn target_is_supported(target: &std::collections::BTreeMap<String, serde_json::Value>) -> bool {
    target.len() == 1
        && target
            .get(ARTIFACT_ROOT_TARGET)
            .and_then(serde_json::Value::as_str)
            .is_some_and(|root| std::path::Path::new(root).is_absolute())
}

/// Author the profile's one-type catalog for an exact executable.
pub fn catalog(executable: &std::path::Path) -> Result<crate::operation::Catalog, super::Error> {
    let mut definition = crate::operation::OperationType {
        id: TYPE_ID.to_string(),
        schema_version: crate::operation::model::TYPE_SCHEMA_VERSION,
        revision: String::new(),
        description: "Opt-in pure interaction-net arithmetic through the pinned sequential Noble worker".to_string(),
        tags: vec!["interaction-net".to_string(), "pure".to_string()],
        methods: vec![method()],
        runtime: crate::operation::model::RuntimeRequirement {
            kind: RUNTIME_KIND.to_string(),
            key: RUNTIME_KEY.to_string(),
            executor_identity: crate::operation::identity::executor_file(executable)
                .map_err(|_| super::Error::ArtifactUnavailable)?,
        },
        execution_requirements: requirements(),
        capabilities: vec![crate::operation::model::CapabilityClass::ProcessRead],
        credential_bindings: vec![],
        bounds: BOUNDS,
        required_non_claims: crate::operation::model::REQUIRED_NON_CLAIMS.iter().map(|s| (*s).to_string()).collect(),
    };
    definition.revision =
        crate::operation::identity::type_revision(&definition).map_err(|_| super::Error::PackAuthoring)?;
    debug_assert!(type_is_supported(&definition));
    Ok(crate::operation::Catalog {
        schema: crate::operation::model::CATALOG_SCHEMA.to_string(),
        types: vec![definition],
    })
}

/// Author one data-only instance whose target is the artifact root.
pub fn instance(
    catalog: &crate::operation::Catalog,
    artifact_root: &std::path::Path,
) -> Result<crate::operation::InstanceFile, super::Error> {
    let definition = catalog.types.first().ok_or(super::Error::PackAuthoring)?;
    let root = artifact_root.to_str().filter(|_| artifact_root.is_absolute());
    let root = root.ok_or(super::Error::ArtifactUnavailable)?;
    Ok(crate::operation::InstanceFile {
        schema: crate::operation::model::INSTANCE_SCHEMA.to_string(),
        instances: vec![crate::operation::OperationInstance {
            id: "pure-inet-artifacts".to_string(),
            display_alias: "Immutable pure interaction-net artifacts".to_string(),
            type_id: TYPE_ID.to_string(),
            type_revision: definition.revision.clone(),
            target: std::collections::BTreeMap::from([(ARTIFACT_ROOT_TARGET.to_string(), serde_json::json!(root))]),
            tags: std::collections::BTreeMap::new(),
            method_defaults: std::collections::BTreeMap::new(),
            credentials: std::collections::BTreeMap::new(),
        }],
    })
}

/// Producer request ceilings; each flag may only lower its default.
pub const DEFAULT_CEILINGS: Ceilings = Ceilings {
    fuel_interactions: 2_097_152,
    max_input_bytes: 134_217_728,
    max_retained_bytes: 268_435_456,
    max_output_bytes: 67_108_864,
};

/// The producer's whole-request ceilings for one invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ceilings {
    pub fuel_interactions: u64,
    pub max_input_bytes: u64,
    pub max_retained_bytes: u64,
    pub max_output_bytes: u64,
}

impl Ceilings {
    /// True when no ceiling raises the producer's default.
    pub fn is_within_producer_defaults(&self) -> bool {
        self.fuel_interactions <= DEFAULT_CEILINGS.fuel_interactions
            && self.max_input_bytes <= DEFAULT_CEILINGS.max_input_bytes
            && self.max_retained_bytes <= DEFAULT_CEILINGS.max_retained_bytes
            && self.max_output_bytes <= DEFAULT_CEILINGS.max_output_bytes
    }

    /// The producer flags that select these ceilings.
    pub fn flags(&self) -> [String; 8] {
        [
            "--fuel".to_string(),
            self.fuel_interactions.to_string(),
            "--max-input-bytes".to_string(),
            self.max_input_bytes.to_string(),
            "--max-retained-bytes".to_string(),
            self.max_retained_bytes.to_string(),
            "--max-output-bytes".to_string(),
            self.max_output_bytes.to_string(),
        ]
    }
}

/// The exact `limits` header object a producer reports for these ceilings.
pub fn limits_object(ceilings: Ceilings) -> serde_json::Value {
    let mut limits = serde_json::Map::new();
    limits.insert("max_input_bytes".to_string(), ceilings.max_input_bytes.into());
    limits.insert("max_retained_bytes".to_string(), ceilings.max_retained_bytes.into());
    limits.insert("max_output_bytes".to_string(), ceilings.max_output_bytes.into());
    limits.insert("interaction_fuel".to_string(), ceilings.fuel_interactions.into());
    for (key, value) in super::report::FIXED_CAPACITIES {
        limits.insert(key.to_string(), value.into());
    }
    serde_json::Value::Object(limits)
}
