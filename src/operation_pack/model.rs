#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the schema module uses serialization derives and ordered identity maps throughout"
)]
#![allow(
    numeric_units,
    reason = "wire fields retain explicit byte and second terms required by the stable Nickel and JSON contracts"
)]
#![allow(
    path_segment_repetition,
    reason = "public domain types retain operation-pack terminology for stable decoded contracts"
)]

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const MANIFEST_SCHEMA: &str = "lattice.operation-pack-manifest.v1";
pub const LOCK_SCHEMA: &str = "lattice.operation-pack-lock.v1";
pub const REGISTRY_SCHEMA: &str = "lattice.operation-pack-registry.v1";
pub const ANALYZER_INPUT_SCHEMA: &str = "lattice.operation-pack-analyzer-input.v1";
pub const ANALYZER_OUTPUT_SCHEMA: &str = "lattice.operation-pack-analyzer-output.v1";
pub const INSTALL_RECEIPT_SCHEMA: &str = "lattice.operation-pack-install-receipt.v1";
pub const SKILL_INSTALL_RECEIPT_SCHEMA: &str = "lattice.operation-pack-skill-install.v1";
pub const MAXIMUM_CONTENT_ITEMS: usize = 128;
pub const MAXIMUM_OPERATIONS: usize = 64;
pub const MAXIMUM_RUNTIMES: usize = 32;
pub const MAXIMUM_ANALYZERS: usize = 32;
pub const MAXIMUM_SKILLS: usize = 32;
pub const MAXIMUM_DEPENDENCIES: usize = 32;
pub const MAXIMUM_EVIDENCE_FACTS: usize = 16;
pub const MAXIMUM_LICENSES: usize = 16;
pub const MAXIMUM_PLATFORMS: usize = 16;
pub const MAXIMUM_NON_CLAIMS: usize = 16;
pub const MAXIMUM_TEXT_BYTES: usize = 512;
pub const MAXIMUM_PATH_BYTES: usize = 240;
pub const MAXIMUM_PACK_BYTES: u64 = 67_108_864;
pub const MAXIMUM_MANIFEST_BYTES: u64 = 262_144;
pub const MAXIMUM_ANALYZER_MARKDOWN_BYTES: usize = 262_144;
pub const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "pack identity proves exact admitted content only",
    "pack admission does not prove maintainer or source trust",
    "bounded process execution does not prove sandbox completeness",
    "pack evidence does not prove production or release readiness",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRole {
    OperationCatalog,
    RuntimeArtifact,
    SkillBody,
    SkillReference,
    AnalyzerAsset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentItem {
    pub id: String,
    pub role: ContentRole,
    pub path: String,
    pub content_identity: String,
    pub byte_length: u64,
    pub media_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationBinding {
    pub type_id: String,
    pub type_revision: String,
    pub catalog_content_id: String,
    pub runtime_id: String,
    pub launch_label: String,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeArtifact {
    pub id: String,
    pub content_id: String,
    pub kind: String,
    pub launch_labels: Vec<String>,
    pub platforms: Vec<String>,
    pub capabilities: Vec<crate::operation::CapabilityClass>,
    pub timeout_seconds: u64,
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub clear_environment: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerScope {
    Method,
    Operation,
    Workflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerDataView {
    RunMetadata,
    ExactDataReferences,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerManifest {
    pub id: String,
    pub scope: AnalyzerScope,
    pub data_view: Vec<AnalyzerDataView>,
    pub required: bool,
    pub runtime_id: String,
    pub launch_label: String,
    pub input_bytes: usize,
    pub timeout_seconds: u64,
    pub output_bytes: usize,
    pub output_schema_identity: String,
    pub markdown_allowed: bool,
    pub maximum_markdown_bytes: usize,
    pub retention_identity: String,
    pub sensitivity: crate::operation_data::SensitivityClass,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillManifest {
    pub name: String,
    pub description: String,
    pub body_content_id: String,
    pub reference_content_ids: Vec<String>,
    pub hosts: Vec<String>,
    pub trust: String,
    pub positive_routes: Vec<String>,
    pub negative_routes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dependency {
    pub pack_id: String,
    pub pack_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    MantleBuild,
    ArtifactAuthentication,
    ValenceRecordedOnly,
    CairnPromotion,
    BasaltAuthority,
    UcanAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionProfile {
    pub identity: String,
    pub required_evidence: Vec<EvidenceKind>,
    pub allowed_licenses: Vec<String>,
    pub allowed_capabilities: Vec<crate::operation::CapabilityClass>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseChannel {
    Beta,
    Rc,
    Stable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseMetadata {
    pub display_version: String,
    pub channel: ReleaseChannel,
    pub deprecated: bool,
    pub superseded_by: Option<String>,
    pub publication_epoch_seconds: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeManifest {
    pub runtime_id: String,
    pub launch_label: String,
    pub effectful: bool,
    pub uses_production_credentials: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub pack_id: String,
    pub description: String,
    pub contents: Vec<ContentItem>,
    pub operations: Vec<OperationBinding>,
    pub runtimes: Vec<RuntimeArtifact>,
    pub analyzers: Vec<AnalyzerManifest>,
    pub skills: Vec<SkillManifest>,
    pub dependencies: Vec<Dependency>,
    pub platforms: Vec<String>,
    pub admission: AdmissionProfile,
    pub release: ReleaseMetadata,
    pub licenses: Vec<String>,
    pub probe: Option<ProbeManifest>,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasuredContent {
    pub id: String,
    pub path: String,
    pub content_identity: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub manifest: Manifest,
    pub manifest_identity: String,
    pub pack_identity: String,
    pub measured_contents: Vec<MeasuredContent>,
    pub source_locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFact {
    pub identity: String,
    pub current: bool,
    pub valid: bool,
    pub overclaimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionFacts {
    pub trusted_pack_identity: String,
    pub platform: String,
    pub dependency_identities: BTreeMap<String, String>,
    pub allowed_licenses: Vec<String>,
    pub allowed_capabilities: Vec<crate::operation::CapabilityClass>,
    pub evidence: BTreeMap<EvidenceKind, EvidenceFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub schema: String,
    pub identity: String,
    pub pack_id: String,
    pub pack_identity: String,
    pub manifest_identity: String,
    pub dependency_identities: BTreeMap<String, String>,
    pub platform: String,
    pub runtime_identities: BTreeMap<String, String>,
    pub analyzer_identities: BTreeMap<String, String>,
    pub skill_identities: BTreeMap<String, String>,
    pub trust_profile_identity: String,
    pub source_locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedPack {
    pub candidate: Candidate,
    pub lock: Lock,
}
