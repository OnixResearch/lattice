pub const RECORD_SCHEMA: &str = "lattice.operation-data-record.v1";
pub const QUERY_SCHEMA: &str = "lattice.operation-data-query.v1";
pub const PROJECTION_SCHEMA: &str = "lattice.operation-data-recorded-only.v1";
pub const MAX_RECORD_BYTES: usize = 16_777_216;
pub const MAX_METADATA_BYTES: usize = 65_536;
pub const MAX_QUERY_CANDIDATES: usize = 4_096;
pub const MAX_QUERY_RESULTS: usize = 100;
pub const MAX_TAGS: usize = 32;
pub const MAX_TEXT_BYTES: usize = 256;
pub const MAX_FIELD_PATH_PARTS: usize = 16;
pub const MAX_SELECTABLE_PATHS: usize = 64;
pub const MAX_NON_CLAIMS: usize = 16;
pub const MAXIMUM_VERSION_SEQUENCE: u64 = 9_007_199_254_740_991;
pub const REQUIRED_NON_CLAIMS: [&str; 3] = [
    "content identities prove exact bytes only",
    "operation data does not prove external truth",
    "retention outcomes do not prove secure erasure",
];
pub const PROJECTION_NON_CLAIM: &str = "recorded-only projection grants no execution authority";

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum RecordClass {
    Resource,
    File,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum SensitivityClass {
    Public,
    Internal,
    Sensitive,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum Lifetime {
    Ephemeral,
    Run { run_id: u64 },
    Duration { expires_epoch_seconds: i64 },
    Retained,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionPolicy {
    pub identity: String,
    pub maximum_versions: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub operation_type: String,
    pub type_revision: String,
    pub instance: String,
    pub method: String,
    pub invocation: String,
    pub run_id: u64,
    pub node: String,
    pub workflow_revision: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "class", deny_unknown_fields)]
pub enum PayloadDraft {
    Resource {
        schema_identity: String,
        selectable_paths: Vec<Vec<String>>,
        value: serde_json::Value,
    },
    File {
        media_class: String,
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDraft {
    pub logical_key: String,
    pub sequence: u64,
    pub producer: Producer,
    pub output_name: String,
    pub output_contract_identity: String,
    pub tags: std::collections::BTreeMap<String, String>,
    pub sensitivity: SensitivityClass,
    pub lifetime: Lifetime,
    pub retention: RetentionPolicy,
    pub outcome: String,
    pub payload: PayloadDraft,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadDescriptor {
    pub content_identity: String,
    pub byte_length_bytes: u64,
    pub storage_backend: String,
    pub safe_locator: String,
    pub safe_locator_identity: String,
    pub schema_identity: Option<String>,
    pub media_class: Option<String>,
    pub selectable_paths: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordMetadata {
    pub schema: String,
    pub record_identity: String,
    pub version_identity: String,
    pub record_class: RecordClass,
    pub logical_key: String,
    pub sequence: u64,
    pub producer: Producer,
    pub output_name: String,
    pub output_contract_identity: String,
    pub tags: std::collections::BTreeMap<String, String>,
    pub sensitivity: SensitivityClass,
    pub lifetime: Lifetime,
    pub retention: RetentionPolicy,
    pub outcome: String,
    pub payload: PayloadDescriptor,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOrder {
    VersionAscending,
    VersionDescending,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagSelector {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub schema: String,
    pub version_identity: Option<String>,
    pub record_identity: Option<String>,
    pub record_class: Option<RecordClass>,
    pub operation_type: Option<String>,
    pub method: Option<String>,
    pub instance: Option<String>,
    pub output_name: Option<String>,
    pub schema_identity: Option<String>,
    pub tag: Option<TagSelector>,
    pub producer_run_id: Option<u64>,
    pub workflow_revision: Option<String>,
    pub outcome: Option<String>,
    pub field_path: Option<Vec<String>>,
    pub order: QueryOrder,
    pub maximum_result_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryAuthority {
    pub allow_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryResult {
    pub metadata: RecordMetadata,
    pub selected_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedInput {
    pub version_identity: String,
    pub content_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundInvocationPlan {
    pub schema: String,
    pub identity: String,
    pub base_invocation_identity: String,
    pub invocation: crate::operation::InvocationPlan,
    pub resolved_data_inputs: Vec<ResolvedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionFacts {
    pub now_epoch_seconds: i64,
    pub terminal_run_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeletionPlan {
    pub identity: String,
    pub selected_versions: Vec<String>,
    pub reason: String,
    pub secure_erasure_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordedOnlyProjection {
    pub schema: String,
    pub identity: String,
    pub verification_role: String,
    pub record_identity: String,
    pub version_identity: String,
    pub producer_invocation: String,
    pub schema_identity: Option<String>,
    pub content_identity: String,
    pub sensitivity: SensitivityClass,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationDecision {
    Append,
    Idempotent,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationOutcome {
    Appended,
    Idempotent,
    Incomplete,
    Unknown,
}
