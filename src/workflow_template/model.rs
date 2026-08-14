pub const CURRENT_BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const CURRENT_WORKFLOW_SCHEMA_VERSION: u32 = 1;
pub const CURRENT_ORCHESTRATION_SCHEMA_VERSION: u32 = 1;
pub const MAX_CATALOG_BUNDLES: usize = 64;
pub const MAX_BUNDLE_FILES: usize = 64;
pub const MAX_BUNDLE_DIRECTORIES: usize = 64;
pub const MAX_FILE_BYTES: usize = 1_048_576;
pub const MAX_BUNDLE_BYTES: usize = 4_194_304;
pub const MAX_IDENTIFIER_BYTES: usize = 128;
pub const MAX_DISPLAY_TEXT_BYTES: usize = 512;
pub const MAX_COMPATIBILITY_ITEMS: usize = 64;
pub const MAX_NOTES: usize = 32;
const HEX_CHARS_PER_BYTE: usize = 2;
const DIGEST_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentIdentity(String);

impl ContentIdentity {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(blake3::hash(bytes).to_hex().to_string())
    }

    pub fn try_from_hex(value: &str) -> Result<Self, IdentityError> {
        if value.len() != DIGEST_HEX_LENGTH {
            return Err(IdentityError::InvalidLength);
        }
        if !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
            return Err(IdentityError::InvalidEncoding);
        }
        Ok(Self(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ContentIdentity {
    type Error = IdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from_hex(&value)
    }
}

impl From<ContentIdentity> for String {
    fn from(value: ContentIdentity) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    InvalidLength,
    InvalidEncoding,
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLength => formatter.write_str("content identity length is invalid"),
            Self::InvalidEncoding => formatter.write_str("content identity encoding is invalid"),
        }
    }
}

impl std::error::Error for IdentityError {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub category: String,
    pub workflow: FileBinding,
    pub orchestration: Option<FileBinding>,
    pub assets: Vec<AssetBinding>,
    pub compatibility: CompatibilityRequirements,
    pub operator_notes: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileBinding {
    pub path: String,
    pub blake3: ContentIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetBinding {
    pub path: String,
    pub media_kind: String,
    pub blake3: ContentIdentity,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompatibilityRequirements {
    pub workflow_schema: u32,
    pub orchestration_schema: Option<u32>,
    pub node_kinds: Vec<String>,
    pub runtime_kinds: Vec<String>,
    pub feature_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BundleValidationInput {
    pub manifest_bytes: Vec<u8>,
    pub manifest: BundleManifest,
    pub files: Vec<BundleFile>,
    pub workflow: Option<crate::engine::Workflow>,
    pub orchestration: Option<crate::orchestration::OrchestrationDefinition>,
}

#[derive(Debug, Clone)]
pub struct ValidatedBundle {
    pub identity: ContentIdentity,
    pub manifest_identity: ContentIdentity,
    pub manifest: BundleManifest,
    pub workflow_revision: crate::orchestration::WorkflowRevisionRef,
    pub workflow: crate::engine::Workflow,
    pub orchestration_revision: Option<crate::orchestration::DefinitionRevision>,
    pub orchestration: Option<crate::orchestration::OrchestrationDefinition>,
    pub file_identities: Vec<(String, ContentIdentity)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticCode {
    InvalidManifest,
    UnsupportedSchema,
    InvalidIdentifier,
    UnsafePath,
    DuplicatePath,
    MissingFile,
    UndeclaredFile,
    TooManyFiles,
    FileTooLarge,
    BundleTooLarge,
    IdentityMismatch,
    InvalidWorkflow,
    InvalidOrchestration,
    WorkflowRevisionMismatch,
    IncompatibleSchema,
    IncompatibleContent,
    LimitExceeded,
    FileAccess,
    SymlinkRejected,
    NonRegularFile,
    NickelEvaluation,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidManifest => "invalid_manifest",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::InvalidIdentifier => "invalid_identifier",
            Self::UnsafePath => "unsafe_path",
            Self::DuplicatePath => "duplicate_path",
            Self::MissingFile => "missing_file",
            Self::UndeclaredFile => "undeclared_file",
            Self::TooManyFiles => "too_many_files",
            Self::FileTooLarge => "file_too_large",
            Self::BundleTooLarge => "bundle_too_large",
            Self::IdentityMismatch => "identity_mismatch",
            Self::InvalidWorkflow => "invalid_workflow",
            Self::InvalidOrchestration => "invalid_orchestration",
            Self::WorkflowRevisionMismatch => "workflow_revision_mismatch",
            Self::IncompatibleSchema => "incompatible_schema",
            Self::IncompatibleContent => "incompatible_content",
            Self::LimitExceeded => "limit_exceeded",
            Self::FileAccess => "file_access",
            Self::SymlinkRejected => "symlink_rejected",
            Self::NonRegularFile => "non_regular_file",
            Self::NickelEvaluation => "nickel_evaluation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub bundle: String,
    pub code: DiagnosticCode,
    pub message: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub bundles: Vec<ValidatedBundle>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistingContent {
    Missing,
    Exact(ContentIdentity),
    Conflicting(ContentIdentity),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationFacts {
    pub workflow: ExistingContent,
    pub orchestration: Option<ExistingContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationPlan {
    Preview {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
    },
    CreateDraft {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
        orchestration: Option<crate::orchestration::DefinitionRevision>,
    },
    ExactAlreadyPresent {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
        orchestration: Option<crate::orchestration::DefinitionRevision>,
    },
    Conflict(DiagnosticCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationFacts {
    pub explicit_save_requested: bool,
    pub explicit_start_requested: bool,
    pub saved_workflow_revision: Option<crate::orchestration::WorkflowRevisionRef>,
    pub current_orchestration_revision: Option<crate::orchestration::DefinitionRevision>,
    pub runtime_admitted: bool,
    pub effects_admitted: bool,
    pub credentials_admitted: bool,
    pub triggers_admitted: bool,
    pub positive_budgets_admitted: bool,
    pub actor_authority_admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationPlan {
    PreviewOnly,
    SaveWorkflow {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
    },
    StartOrchestration {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
        orchestration: crate::orchestration::DefinitionRevision,
    },
    Blocked(ActivationBlock),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivationReceipt {
    pub identity: ContentIdentity,
    pub bundle: ContentIdentity,
    pub workflow: crate::orchestration::WorkflowRevisionRef,
    pub orchestration: crate::orchestration::DefinitionRevision,
    pub authority_policy: crate::orchestration::AuthorityPolicyIdentity,
    pub actor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationBlock {
    WorkflowNotSaved,
    WorkflowDrift,
    OrchestrationMissing,
    OrchestrationDrift,
    RuntimeNotAdmitted,
    EffectsNotAdmitted,
    CredentialsNotAdmitted,
    TriggersNotAdmitted,
    BudgetsNotAdmitted,
    AuthorityNotAdmitted,
}
