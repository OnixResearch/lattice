#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: String,
    pub types: Vec<OperationType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationType {
    pub id: String,
    pub schema_version: u32,
    pub revision: String,
    pub description: String,
    #[serde(default = "empty_vec")]
    pub tags: Vec<String>,
    pub methods: Vec<Method>,
    pub runtime: RuntimeRequirement,
    pub execution_requirements: super::backend::ExecutionRequirements,
    pub capabilities: Vec<CapabilityClass>,
    #[serde(default = "empty_vec")]
    pub credential_bindings: Vec<CredentialBinding>,
    pub bounds: Bounds,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    pub name: String,
    pub description: String,
    pub arguments: RecordContract,
    pub output: RecordContract,
    pub preflight: Vec<PreflightCheck>,
    pub capabilities: Vec<CapabilityClass>,
    pub side_effect: SideEffectClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordContract {
    pub fields: Vec<FieldContract>,
    #[serde(default = "default_false")]
    pub deny_unknown_fields: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldContract {
    pub name: String,
    pub value_type: ValueType,
    pub required: bool,
    pub max_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueType {
    String,
    Integer,
    Boolean,
    Object,
    Array,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequirement {
    pub kind: String,
    pub key: String,
    pub executor_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityClass {
    ProcessRead,
    CredentialRead,
    FileRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideEffectClass {
    ReadOnly,
    Mutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreflightCheck {
    RuntimeAvailable,
    ExecutableAvailable,
    RepositoryRootAdmitted,
    CredentialReferencesValid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialBinding {
    pub name: String,
    pub required: bool,
    pub allow_field: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    #[allow(
        unknown_lints,
        numeric_units,
        reason = "the external operation schema uses the explicit timeout_seconds field name"
    )]
    pub timeout_seconds: u64,
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub artifact_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstanceFile {
    pub schema: String,
    pub instances: Vec<OperationInstance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationInstance {
    pub id: String,
    pub display_alias: String,
    pub type_id: String,
    pub type_revision: String,
    pub target: BTreeMap<String, serde_json::Value>,
    #[serde(default = "empty_map")]
    pub tags: BTreeMap<String, String>,
    #[serde(default = "empty_map")]
    pub method_defaults: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    #[serde(default = "empty_map")]
    pub credentials: BTreeMap<String, crate::credentials::SecretRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedCatalog {
    pub identity: String,
    pub types: BTreeMap<String, ValidatedType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedType {
    pub definition: OperationType,
    pub method_identities: BTreeMap<String, String>,
    pub argument_schema_identities: BTreeMap<String, String>,
    pub output_schema_identities: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedInstance {
    pub identity: String,
    pub definition: OperationInstance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFact {
    pub kind: String,
    pub key: String,
    pub executor_identity: String,
    pub executable_path: String,
    pub available: bool,
    pub maximum_bounds: Bounds,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityPolicy {
    pub allowed: Vec<CapabilityClass>,
    pub allow_mutation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreflightFacts {
    pub executable_available: bool,
    pub repository_root_admitted: bool,
    pub credential_references_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRequest<'a> {
    pub catalog: &'a ValidatedCatalog,
    pub instance: &'a ValidatedInstance,
    pub method: &'a str,
    pub arguments: BTreeMap<String, serde_json::Value>,
    pub expected_catalog_identity: &'a str,
    pub expected_instance_identity: &'a str,
    pub runtime: RuntimeFact,
    pub policy: CapabilityPolicy,
    pub preflight: PreflightFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationPlan {
    pub schema: String,
    pub identity: String,
    pub catalog_identity: String,
    pub type_id: String,
    pub type_revision: String,
    pub instance_id: String,
    pub instance_identity: String,
    pub method: String,
    pub method_identity: String,
    pub arguments: BTreeMap<String, serde_json::Value>,
    pub arguments_identity: String,
    pub output_contract: RecordContract,
    pub output_contract_identity: String,
    pub runtime: RuntimeFact,
    pub execution_requirements: super::backend::ExecutionRequirements,
    pub backend_route: super::backend::BackendRoutePlan,
    pub capabilities: Vec<CapabilityClass>,
    pub target: BTreeMap<String, serde_json::Value>,
    pub credentials: BTreeMap<String, crate::credentials::SecretRef>,
    pub bounds: Bounds,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionPlan {
    pub schema: String,
    pub identity: String,
    pub invocation_identity: String,
    pub destination: String,
    pub content_identity: String,
    pub nickel: String,
    pub safe_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionResult {
    pub plan: PromotionPlan,
    pub created: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Schema,
    Limit,
    Duplicate,
    InvalidIdentity,
    InvalidValue,
    MissingType,
    StaleType,
    MissingMethod,
    CapabilityDenied,
    RuntimeUnavailable,
    BackendDenied,
    CredentialRejected,
    PreflightRejected,
    OutputRejected,
    PromotionRejected,
}

fn empty_vec<T>() -> Vec<T> {
    Vec::new()
}

fn empty_map<K: Ord, V>() -> BTreeMap<K, V> {
    BTreeMap::new()
}

const fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub path: String,
    pub message: String,
}
