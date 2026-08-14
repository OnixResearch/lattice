use serde::Deserialize;
use serde::Serialize;

pub const FEATURE_ID: &str = "lattice.agent-evaluation-catalog.v1";
pub const CATALOG_SCHEMA: &str = "lattice.agent-evaluation-catalog.v1";
pub const RECEIPT_SCHEMA: &str = "lattice.agent-evaluation-receipt.v1";
pub const MAX_CASES: usize = 64;
pub const MAX_ASSETS: usize = 128;
pub const MAX_ID_BYTES: usize = 128;
pub const MAX_TRIALS_PER_CASE: u32 = 16;
pub const MAX_TRIALS_PER_CASE_USIZE: usize = 16;
pub const MAX_PLANNED_TRIALS: usize = MAX_CASES.saturating_mul(MAX_TRIALS_PER_CASE_USIZE);
pub const MAX_CONCURRENCY: u32 = 16;
pub const MAX_TOKENS: u64 = 1_000_000;
pub const MAX_DURATION_MS: u64 = 86_400_000;
pub const MAX_DEPLOYMENTS: u32 = 4;
pub const MAX_ARTIFACTS: u32 = 64;
pub const MAX_ARTIFACT_BYTES: u64 = 67_108_864;
pub const MAX_NON_CLAIMS: usize = 32;
pub const MAX_PATH_BYTES: usize = 240;
pub const MAX_RUBRIC_SCORE: u16 = 10_000;
pub const SERVICE_KIND_COUNT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseClass {
    Unit,
    Skill,
    Hard,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPosture {
    None,
    Isolated,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HiddenMaterialPosture {
    None,
    Frozen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Passed,
    Failed,
    Missing,
    Stale,
    TimedOut,
    Malformed,
    Contradictory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionClass {
    Ephemeral,
    ReviewedPrivate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownOutcomePolicy {
    RetainUnknown,
    CleanupRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrialStatus {
    Planned,
    Started,
    Passed,
    Failed,
    Cancelled,
    Partial,
    CleanupRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RouteKind {
    Agent,
    Workflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ServiceKind {
    Workspace,
    Agent,
    Workflow,
    Verifier,
    Judge,
    Deployment,
    Artifact,
    Cleanup,
    Clock,
    Store,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub repository_identity: String,
    pub workload_revision: String,
    pub fixture_identity: String,
    pub scaffold_identity: String,
    pub initial_state_identity: String,
    pub hidden_material: HiddenMaterialPosture,
    pub mutation_scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentProfile {
    pub identity: String,
    pub network: NetworkPosture,
    pub isolated: bool,
    pub target_class: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialBudget {
    pub trials: u32,
    pub concurrency: u32,
    pub max_tokens: u64,
    pub max_duration_ms: u64,
    pub max_deployments: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDefinition {
    pub id: String,
    pub implementation_identity: String,
    pub input_identity: String,
    pub evidence_schema: String,
    pub objective: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RubricDefinition {
    pub identity: String,
    pub judge_profile_identity: String,
    pub advisory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceDefinition {
    pub identity: String,
    pub source_identity: String,
    pub capture_class: String,
    pub freshness_required: bool,
    pub fresh: bool,
    pub agent_controlled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    SourceSnapshot,
    Result,
    BoundedLog,
    Trajectory,
    Screenshot,
    ApiSample,
    DeploymentObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaClass {
    Text,
    Json,
    Image,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactPolicy {
    pub identity: String,
    pub allowed_roles: Vec<ArtifactRole>,
    pub allowed_media: Vec<MediaClass>,
    pub max_count: u32,
    pub max_bytes: u64,
    pub retention: RetentionClass,
    pub private_destination: bool,
    pub redaction_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LivePosture {
    pub enabled: bool,
    pub operator_actor: Option<String>,
    pub target_namespace: Option<String>,
    pub credential_handles: Vec<String>,
    pub retention: RetentionClass,
    pub cleanup_required: bool,
    pub unknown_outcome: UnknownOutcomePolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseDefinition {
    pub id: String,
    pub class: CaseClass,
    pub asset_path: String,
    pub asset_identity: String,
    pub subject: Subject,
    pub route: RouteKind,
    pub agent_profile_identity: String,
    pub environment: EnvironmentProfile,
    pub policy_identity: String,
    pub budget: TrialBudget,
    pub gates: Vec<GateDefinition>,
    pub rubric: Option<RubricDefinition>,
    pub references: Vec<ReferenceDefinition>,
    pub artifacts: ArtifactPolicy,
    pub live: LivePosture,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: String,
    pub cases: Vec<CaseDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedCase {
    pub definition: CaseDefinition,
    pub revision_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedCatalog {
    pub cases: Vec<AdmittedCase>,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFact {
    pub relative_path: String,
    pub content_identity: String,
    pub byte_count: u64,
    pub regular_file: bool,
    pub symlink: bool,
}
