//! Pure workflow command contracts for named Animus workflows.
//!
//! Hosts own command discovery, file loading, prompt rendering, adapters,
//! permissions, VCS operations, network access, shell execution, and persistence.
//! This module validates already-loaded command manifests, binds invocation
//! arguments, builds deterministic registries, and checks output contracts before
//! any host-owned side effect can start.

#![allow(unknown_lints)]
#![allow(assertion_density, excessive_file_length, no_panic, unbounded_collection_growth)]

#[cfg(test)]
const WORKFLOW_COMMAND_SCHEMA: &str = "animus.workflow-command.v1";
const MAX_COMMAND_NAME_CHARS: usize = 64;
const MAX_SAFE_LABEL_CHARS: usize = 96;
const MAX_SAFE_SUMMARY_CHARS: usize = 512;
const MAX_ARGUMENT_TEXT_CHARS: usize = 2048;
const MAX_INPUT_FIELDS: usize = 16;
const MAX_ARGUMENTS: usize = 16;
const MAX_CAPABILITIES: usize = 8;
const MAX_VALIDATION_GATES: usize = 8;
const BLAKE3_HEX_CHARS: usize = 64;
const SOURCE_HOST_RANK: u8 = 3;
const SOURCE_USER_RANK: u8 = 2;
const SOURCE_PROJECT_RANK: u8 = 1;
const EMPTY_VALUE: &str = "";
const MISSING_INPUT_SCHEMA_VALUE: &str = "missing_input_schema";
const TOO_MANY_INPUTS_VALUE: &str = "too_many_inputs";
const MISSING_POSTURE_VALUE: &str = "missing_side_effect_posture";
const MISSING_CAPABILITIES_VALUE: &str = "missing_capabilities";
const TOO_MANY_CAPABILITIES_VALUE: &str = "too_many_capabilities";
const MISSING_GATES_VALUE: &str = "missing_validation_gates";
const TOO_MANY_GATES_VALUE: &str = "too_many_validation_gates";
const MISSING_OUTPUT_CONTRACT_VALUE: &str = "missing_output_contract";
const MISSING_SCHEMA_LABEL_VALUE: &str = "missing_schema_label";
const UNEXPECTED_SCHEMA_LABEL_VALUE: &str = "unexpected_schema_label";
const POLICY_POSTURE_DENIED_VALUE: &str = "side_effect_posture_denied";
const POLICY_CAPABILITY_DENIED_VALUE: &str = "capability_denied";
const VALIDATION_STATUS_REQUIRED_VALUE: &str = "validation_status_required";
#[cfg(test)]
const SOURCE_IDS_FIELD: &str = "source_ids";
const COMMAND_NAME_FIELD: &str = "command_name";
const INPUT_SCHEMA_FIELD: &str = "input_schema";
const INPUT_FIELD_NAME_FIELD: &str = "input_schema.name";
const CAPABILITIES_FIELD: &str = "capabilities";
const SIDE_EFFECT_POSTURE_FIELD: &str = "side_effect_posture";
const VALIDATION_GATES_FIELD: &str = "validation_gates";
const OUTPUT_CONTRACT_FIELD: &str = "output_contract";
const OUTPUT_KIND_FIELD: &str = "output_kind";
const OUTPUT_BODY_FIELD: &str = "output_body";
const STRUCTURED_SCHEMA_FIELD: &str = "structured_schema";
const SOURCE_ID_FIELD: &str = "source_id";
const CONTENT_HASH_FIELD: &str = "content_hash";
const TRUST_STATUS_FIELD: &str = "trust_status";
const ARGUMENT_NAME_FIELD: &str = "argument.name";
const ARGUMENT_VALUE_FIELD: &str = "argument.value";
const LEAK_MARKERS: &[&str] = &[
    "raw_prompt:",
    "provider_payload:",
    "credential:",
    "secret=",
    "token=",
    "api_key",
    "full_path:",
    "/home/",
    "unbounded_output:",
];

// r[impl agent_harness_runtime.claude_code_commands.manifest]
/// Scalar type accepted by one workflow command input field.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkflowCommandArgumentType {
    /// Bounded text value.
    Text,
    /// Boolean switch value.
    Boolean,
    /// Signed integer value.
    Integer,
    /// Ordered list of bounded text values.
    TextList,
}

impl WorkflowCommandArgumentType {
    /// Returns the stable argument type label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Boolean => "boolean",
            Self::Integer => "integer",
            Self::TextList => "text_list",
        }
    }

    const fn accepts(self, value: &WorkflowCommandArgumentValue) -> bool {
        matches!(
            (self, value),
            (Self::Text, WorkflowCommandArgumentValue::Text(_))
                | (Self::Boolean, WorkflowCommandArgumentValue::Boolean(_))
                | (Self::Integer, WorkflowCommandArgumentValue::Integer(_))
                | (Self::TextList, WorkflowCommandArgumentValue::TextList(_))
        )
    }
}

/// Owned workflow invocation argument value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowCommandArgumentValue {
    /// Bounded text value.
    Text(String),
    /// Boolean switch value.
    Boolean(bool),
    /// Signed integer value.
    Integer(i64),
    /// Ordered bounded text list.
    TextList(Vec<String>),
}

impl WorkflowCommandArgumentValue {
    /// Returns the value's stable type label.
    #[must_use]
    pub const fn value_type(&self) -> WorkflowCommandArgumentType {
        match self {
            Self::Text(_) => WorkflowCommandArgumentType::Text,
            Self::Boolean(_) => WorkflowCommandArgumentType::Boolean,
            Self::Integer(_) => WorkflowCommandArgumentType::Integer,
            Self::TextList(_) => WorkflowCommandArgumentType::TextList,
        }
    }
}

/// Borrowed workflow invocation argument value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandArgumentValueDraft<'a> {
    /// Bounded text value.
    Text(&'a str),
    /// Boolean switch value.
    Boolean(bool),
    /// Signed integer value.
    Integer(i64),
    /// Ordered bounded text list.
    TextList(&'a [&'a str]),
}

impl WorkflowCommandArgumentValueDraft<'_> {
    fn to_owned(self) -> Result<WorkflowCommandArgumentValue, WorkflowCommandError> {
        match self {
            Self::Text(value) => {
                Ok(WorkflowCommandArgumentValue::Text(String::from(validate_argument_text(FieldText {
                    field: ARGUMENT_VALUE_FIELD,
                    value,
                })?)))
            }
            Self::Boolean(value) => Ok(WorkflowCommandArgumentValue::Boolean(value)),
            Self::Integer(value) => Ok(WorkflowCommandArgumentValue::Integer(value)),
            Self::TextList(values) => {
                let mut owned = Vec::with_capacity(values.len());
                for value in values {
                    owned.push(String::from(validate_argument_text(FieldText {
                        field: ARGUMENT_VALUE_FIELD,
                        value,
                    })?));
                }
                Ok(WorkflowCommandArgumentValue::TextList(owned))
            }
        }
    }
}

/// Draft input-schema field supplied by a host loader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCommandInputFieldDraft<'a> {
    /// Stable argument name.
    pub name: &'a str,
    /// Expected argument value type.
    pub value_type: WorkflowCommandArgumentType,
    /// Whether the argument must be supplied by every invocation.
    pub required: bool,
}

/// Validated input-schema field for a workflow command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandInputField {
    name: String,
    value_type: WorkflowCommandArgumentType,
    required: bool,
}

impl WorkflowCommandInputField {
    /// Returns the stable argument name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the expected argument type.
    #[must_use]
    pub const fn value_type(&self) -> WorkflowCommandArgumentType {
        self.value_type
    }

    /// Returns true when the argument is required.
    #[must_use]
    pub const fn required(&self) -> bool {
        self.required
    }
}

/// Capability declared by a workflow command manifest before policy binding.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkflowCommandCapability {
    /// Read already-admitted context.
    ReadContext,
    /// Request model output through an injected adapter.
    CallModel,
    /// Request tool output through an injected adapter.
    CallTool,
    /// Propose or perform workspace mutation through a guarded host adapter.
    MutateWorkspace,
    /// Propose or perform VCS mutation through a guarded host adapter.
    MutateVcs,
    /// Use host-approved network-capable services.
    UseNetwork,
    /// Use host-specific authority outside reusable runtime contracts.
    HostSpecific,
    /// Run declared validation gates and report their outcomes.
    ValidateOutput,
}

impl WorkflowCommandCapability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadContext => "read_context",
            Self::CallModel => "call_model",
            Self::CallTool => "call_tool",
            Self::MutateWorkspace => "mutate_workspace",
            Self::MutateVcs => "mutate_vcs",
            Self::UseNetwork => "use_network",
            Self::HostSpecific => "host_specific",
            Self::ValidateOutput => "validate_output",
        }
    }
}

/// Declared side-effect posture for a workflow command.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorkflowCommandSideEffectPosture {
    /// The command can only read admitted context and produce responses.
    ReadOnly,
    /// The command can request guarded workspace edits.
    WorkspaceMutation,
    /// The command can request guarded VCS mutations.
    VcsMutation,
    /// The command can request network-capable host services.
    NetworkCapable,
    /// The command requires explicit host-specific authority.
    HostSpecific,
}

impl WorkflowCommandSideEffectPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::WorkspaceMutation => "workspace_mutation",
            Self::VcsMutation => "vcs_mutation",
            Self::NetworkCapable => "network_capable",
            Self::HostSpecific => "host_specific",
        }
    }

    const fn permits(self, capability: WorkflowCommandCapability) -> bool {
        match capability {
            WorkflowCommandCapability::ReadContext
            | WorkflowCommandCapability::CallModel
            | WorkflowCommandCapability::CallTool
            | WorkflowCommandCapability::ValidateOutput => true,
            WorkflowCommandCapability::MutateWorkspace => {
                matches!(self, Self::WorkspaceMutation | Self::VcsMutation | Self::HostSpecific)
            }
            WorkflowCommandCapability::MutateVcs => {
                matches!(self, Self::VcsMutation | Self::HostSpecific)
            }
            WorkflowCommandCapability::UseNetwork => {
                matches!(self, Self::NetworkCapable | Self::HostSpecific)
            }
            WorkflowCommandCapability::HostSpecific => matches!(self, Self::HostSpecific),
        }
    }
}

/// Source class for a workflow command manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandSourceScope {
    /// Host-managed command source.
    Host,
    /// User-local command source.
    User,
    /// Project-local command source.
    Project,
}

impl WorkflowCommandSourceScope {
    /// Returns the stable source-scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::User => "user",
            Self::Project => "project",
        }
    }

    const fn precedence_rank(self) -> u8 {
        match self {
            Self::Host => SOURCE_HOST_RANK,
            Self::User => SOURCE_USER_RANK,
            Self::Project => SOURCE_PROJECT_RANK,
        }
    }
}

/// Trust status assigned by the host to a workflow command source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandTrustStatus {
    /// The source and content hash are trusted for advertisement and invocation.
    Trusted,
    /// The source has not been reviewed or admitted.
    Untrusted,
    /// The source or content hash has been explicitly denied.
    Denied,
}

impl WorkflowCommandTrustStatus {
    /// Returns the stable trust-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Untrusted => "untrusted",
            Self::Denied => "denied",
        }
    }
}

/// Draft provenance metadata supplied with a workflow command manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCommandProvenanceDraft<'a> {
    /// Safe source identifier, never a raw path.
    pub source_id: &'a str,
    /// Source class for precedence and trust diagnostics.
    pub source_scope: WorkflowCommandSourceScope,
    /// Host trust decision for the source/content pair.
    pub trust_status: WorkflowCommandTrustStatus,
    /// BLAKE3 hash of the reviewed command content.
    pub content_hash: &'a str,
}

/// Validated safe provenance metadata for a workflow command manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandProvenance {
    source_id: String,
    source_scope: WorkflowCommandSourceScope,
    trust_status: WorkflowCommandTrustStatus,
    content_hash: String,
}

impl WorkflowCommandProvenance {
    /// Returns the safe source identifier.
    #[must_use]
    pub fn source_id(&self) -> &str {
        self.source_id.as_str()
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn source_scope(&self) -> WorkflowCommandSourceScope {
        self.source_scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust_status(&self) -> WorkflowCommandTrustStatus {
        self.trust_status
    }

    /// Returns the BLAKE3 content hash.
    #[must_use]
    pub fn content_hash(&self) -> &str {
        self.content_hash.as_str()
    }
}

/// Output kind declared by a workflow command contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandOutputKind {
    /// Explanatory prose for a human.
    Explanation,
    /// Review report output.
    Review,
    /// Patch summary output.
    PatchSummary,
    /// Validation report output with an explicit status.
    ValidationReport,
    /// Commit-message proposal output; hosts still own VCS mutation.
    CommitMessageProposal,
    /// Structured DTO output bound to a schema label.
    StructuredDto,
}

impl WorkflowCommandOutputKind {
    /// Returns the stable output-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Explanation => "explanation",
            Self::Review => "review",
            Self::PatchSummary => "patch_summary",
            Self::ValidationReport => "validation_report",
            Self::CommitMessageProposal => "commit_message_proposal",
            Self::StructuredDto => "structured_dto",
        }
    }
}

/// Draft output contract for a workflow command manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCommandOutputContractDraft<'a> {
    /// Expected output kind.
    pub kind: WorkflowCommandOutputKind,
    /// Required schema label for structured DTO output.
    pub structured_schema: Option<&'a str>,
}

/// Validated workflow command output contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandOutputContract {
    kind: WorkflowCommandOutputKind,
    structured_schema: Option<String>,
}

impl WorkflowCommandOutputContract {
    /// Validates an output contract draft.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowCommandError`] when the contract omits a structured
    /// schema label or supplies one for an unstructured output kind.
    pub fn validate(draft: &WorkflowCommandOutputContractDraft<'_>) -> Result<Self, WorkflowCommandError> {
        let structured_schema = match (draft.kind, draft.structured_schema) {
            (WorkflowCommandOutputKind::StructuredDto, Some(schema)) => {
                Some(String::from(validate_safe_label(FieldText {
                    field: STRUCTURED_SCHEMA_FIELD,
                    value: schema,
                })?))
            }
            (WorkflowCommandOutputKind::StructuredDto, None) => {
                return Err(WorkflowCommandError::new(
                    WorkflowCommandErrorKind::MissingOutputContract,
                    STRUCTURED_SCHEMA_FIELD,
                    MISSING_SCHEMA_LABEL_VALUE,
                ));
            }
            (_, Some(_)) => {
                return Err(WorkflowCommandError::new(
                    WorkflowCommandErrorKind::OutputContractMismatch,
                    STRUCTURED_SCHEMA_FIELD,
                    UNEXPECTED_SCHEMA_LABEL_VALUE,
                ));
            }
            (_, None) => None,
        };
        Ok(Self {
            kind: draft.kind,
            structured_schema,
        })
    }

    /// Returns the expected output kind.
    #[must_use]
    pub const fn kind(&self) -> WorkflowCommandOutputKind {
        self.kind
    }

    /// Returns the structured schema label when one is required.
    #[must_use]
    pub fn structured_schema(&self) -> Option<&str> {
        self.structured_schema.as_deref()
    }
}

/// Draft workflow command manifest supplied by a host loader.
pub struct WorkflowCommandManifestDraft<'a> {
    /// Stable command invocation name.
    pub name: &'a str,
    /// Human-readable bounded summary.
    pub summary: &'a str,
    /// Input schema. `None` means the schema was missing from the manifest.
    pub input_schema: Option<&'a [WorkflowCommandInputFieldDraft<'a>]>,
    /// Declared capabilities.
    pub capabilities: &'a [WorkflowCommandCapability],
    /// Declared side-effect posture.
    pub side_effect_posture: Option<WorkflowCommandSideEffectPosture>,
    /// Retrieval budget reference label.
    pub retrieval_budget: &'a str,
    /// Validation gate labels required before completion.
    pub validation_gates: &'a [&'a str],
    /// Expected output contract.
    pub output_contract: Option<WorkflowCommandOutputContractDraft<'a>>,
    /// Source/provenance metadata.
    pub provenance: WorkflowCommandProvenanceDraft<'a>,
}

/// Validated typed workflow command manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandManifest {
    name: String,
    summary: String,
    input_schema: Vec<WorkflowCommandInputField>,
    capabilities: Vec<WorkflowCommandCapability>,
    side_effect_posture: WorkflowCommandSideEffectPosture,
    retrieval_budget: String,
    validation_gates: Vec<String>,
    output_contract: WorkflowCommandOutputContract,
    provenance: WorkflowCommandProvenance,
}

impl WorkflowCommandManifest {
    /// Validates a workflow command manifest draft.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowCommandError`] when the manifest is unsafe, incomplete,
    /// untrusted, or internally inconsistent.
    pub fn validate(draft: &WorkflowCommandManifestDraft<'_>) -> Result<Self, WorkflowCommandError> {
        let name = String::from(validate_command_name(draft.name)?);
        let summary = String::from(validate_safe_summary(FieldText {
            field: "summary",
            value: draft.summary,
        })?);
        let input_schema = validate_input_schema(draft.input_schema)?;
        let side_effect_posture = draft.side_effect_posture.ok_or_else(|| {
            WorkflowCommandError::new(
                WorkflowCommandErrorKind::MissingSideEffectPosture,
                SIDE_EFFECT_POSTURE_FIELD,
                MISSING_POSTURE_VALUE,
            )
        })?;
        let capabilities = validate_capabilities(side_effect_posture, draft.capabilities)?;
        let retrieval_budget = String::from(validate_safe_label(FieldText {
            field: "retrieval_budget",
            value: draft.retrieval_budget,
        })?);
        let validation_gates = validate_validation_gates(draft.validation_gates)?;
        let output_contract = draft
            .output_contract
            .as_ref()
            .ok_or_else(|| {
                WorkflowCommandError::new(
                    WorkflowCommandErrorKind::MissingOutputContract,
                    OUTPUT_CONTRACT_FIELD,
                    MISSING_OUTPUT_CONTRACT_VALUE,
                )
            })
            .and_then(WorkflowCommandOutputContract::validate)?;
        let provenance = validate_provenance(&draft.provenance)?;
        Ok(Self {
            name,
            summary,
            input_schema,
            capabilities,
            side_effect_posture,
            retrieval_budget,
            validation_gates,
            output_contract,
            provenance,
        })
    }

    /// Returns the stable command invocation name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the bounded summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the validated input schema.
    #[must_use]
    pub fn input_schema(&self) -> &[WorkflowCommandInputField] {
        self.input_schema.as_slice()
    }

    /// Returns the declared capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[WorkflowCommandCapability] {
        self.capabilities.as_slice()
    }

    /// Returns the declared side-effect posture.
    #[must_use]
    pub const fn side_effect_posture(&self) -> WorkflowCommandSideEffectPosture {
        self.side_effect_posture
    }

    /// Returns the retrieval budget reference label.
    #[must_use]
    pub fn retrieval_budget(&self) -> &str {
        self.retrieval_budget.as_str()
    }

    /// Returns the validation gate labels required by the command.
    #[must_use]
    pub fn validation_gates(&self) -> &[String] {
        self.validation_gates.as_slice()
    }

    /// Returns the expected output contract.
    #[must_use]
    pub const fn output_contract(&self) -> &WorkflowCommandOutputContract {
        &self.output_contract
    }

    /// Returns safe provenance metadata.
    #[must_use]
    pub const fn provenance(&self) -> &WorkflowCommandProvenance {
        &self.provenance
    }
}

// r[impl agent_harness_runtime.claude_code_commands.registry]
/// Duplicate-name handling policy for workflow command registry construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandDuplicatePolicy {
    /// Disable a duplicated command name until the host resolves the collision.
    DisableName,
    /// Prefer host, then user, then project source precedence.
    PreferSourcePrecedence,
}

/// Registry construction policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCommandRegistryPolicy {
    /// Duplicate-name handling policy.
    pub duplicate_policy: WorkflowCommandDuplicatePolicy,
}

impl WorkflowCommandRegistryPolicy {
    /// Returns a fail-closed policy that disables duplicate names.
    #[must_use]
    pub const fn fail_closed() -> Self {
        Self {
            duplicate_policy: WorkflowCommandDuplicatePolicy::DisableName,
        }
    }

    /// Returns a deterministic precedence policy.
    #[must_use]
    pub const fn prefer_source_precedence() -> Self {
        Self {
            duplicate_policy: WorkflowCommandDuplicatePolicy::PreferSourcePrecedence,
        }
    }
}

/// Registry diagnostic kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandRegistryDiagnosticKind {
    /// More than one manifest declared the same command name.
    DuplicateName,
    /// The duplicate command name was disabled by policy.
    DisabledDuplicate,
    /// A deterministic source-precedence winner was selected.
    SelectedByPrecedence,
}

impl WorkflowCommandRegistryDiagnosticKind {
    /// Returns the stable diagnostic label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateName => "duplicate_name",
            Self::DisabledDuplicate => "disabled_duplicate",
            Self::SelectedByPrecedence => "selected_by_precedence",
        }
    }
}

/// Safe workflow command registry diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandRegistryDiagnostic {
    command_name: String,
    kind: WorkflowCommandRegistryDiagnosticKind,
    source_ids: Vec<String>,
    selected_source_id: Option<String>,
}

impl WorkflowCommandRegistryDiagnostic {
    /// Returns the command name covered by this diagnostic.
    #[must_use]
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }

    /// Returns the diagnostic kind.
    #[must_use]
    pub const fn kind(&self) -> WorkflowCommandRegistryDiagnosticKind {
        self.kind
    }

    /// Returns safe source identifiers involved in the diagnostic.
    #[must_use]
    pub fn source_ids(&self) -> &[String] {
        self.source_ids.as_slice()
    }

    /// Returns the selected source identifier when policy chose a winner.
    #[must_use]
    pub fn selected_source_id(&self) -> Option<&str> {
        self.selected_source_id.as_deref()
    }
}

/// Deterministic in-memory workflow command registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandRegistry {
    commands: Vec<WorkflowCommandManifest>,
    diagnostics: Vec<WorkflowCommandRegistryDiagnostic>,
}

impl WorkflowCommandRegistry {
    /// Looks up a command by invocation name.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&WorkflowCommandManifest> {
        self.commands.iter().find(|command| command.name() == name)
    }

    /// Returns commands advertised by this registry.
    #[must_use]
    pub fn commands(&self) -> &[WorkflowCommandManifest] {
        self.commands.as_slice()
    }

    /// Returns safe registry diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[WorkflowCommandRegistryDiagnostic] {
        self.diagnostics.as_slice()
    }
}

/// Builds a deterministic workflow command registry from validated manifests.
#[must_use]
pub fn build_workflow_command_registry(
    manifests: &[WorkflowCommandManifest],
    policy: WorkflowCommandRegistryPolicy,
) -> WorkflowCommandRegistry {
    let mut groups: std::collections::BTreeMap<&str, Vec<&WorkflowCommandManifest>> = std::collections::BTreeMap::new();
    for manifest in manifests {
        groups.entry(manifest.name()).or_default().push(manifest);
    }

    let mut commands = Vec::new();
    let mut diagnostics = Vec::new();
    for (name, group) in groups {
        if group.len() == 1 {
            commands.push(group[0].clone());
            continue;
        }
        let source_ids = sorted_source_ids(group.as_slice());
        diagnostics.push(registry_diagnostic(
            name,
            WorkflowCommandRegistryDiagnosticKind::DuplicateName,
            source_ids.clone(),
            None,
        ));
        match policy.duplicate_policy {
            WorkflowCommandDuplicatePolicy::DisableName => {
                diagnostics.push(registry_diagnostic(
                    name,
                    WorkflowCommandRegistryDiagnosticKind::DisabledDuplicate,
                    source_ids,
                    None,
                ));
            }
            WorkflowCommandDuplicatePolicy::PreferSourcePrecedence => {
                if let Some(selected) = select_precedence_winner(group.as_slice()) {
                    diagnostics.push(registry_diagnostic(
                        name,
                        WorkflowCommandRegistryDiagnosticKind::SelectedByPrecedence,
                        source_ids,
                        Some(selected.provenance().source_id()),
                    ));
                    commands.push(selected.clone());
                }
            }
        }
    }

    WorkflowCommandRegistry { commands, diagnostics }
}

// r[impl agent_harness_runtime.claude_code_commands.binding]
/// Borrowed invocation argument supplied by a user or host parser.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowInvocationArgumentDraft<'a> {
    /// Argument name.
    pub name: &'a str,
    /// Argument value.
    pub value: WorkflowCommandArgumentValueDraft<'a>,
}

/// Bound workflow invocation argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowInvocationArgument {
    name: String,
    value: WorkflowCommandArgumentValue,
}

impl WorkflowInvocationArgument {
    /// Returns the argument name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the bound argument value.
    #[must_use]
    pub const fn value(&self) -> &WorkflowCommandArgumentValue {
        &self.value
    }
}

/// Bound workflow invocation arguments in manifest schema order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowInvocationBinding {
    command_name: String,
    arguments: Vec<WorkflowInvocationArgument>,
}

impl WorkflowInvocationBinding {
    /// Returns the command name bound by this invocation.
    #[must_use]
    pub fn command_name(&self) -> &str {
        self.command_name.as_str()
    }

    /// Returns bound arguments in schema order.
    #[must_use]
    pub fn arguments(&self) -> &[WorkflowInvocationArgument] {
        self.arguments.as_slice()
    }
}

/// Binds invocation arguments before prompt expansion, tool dispatch, or mutation.
///
/// # Errors
///
/// Returns [`WorkflowCommandError`] when arguments are missing, unknown,
/// duplicated, malformed, or of the wrong type.
pub fn bind_workflow_command_arguments(
    manifest: &WorkflowCommandManifest,
    arguments: &[WorkflowInvocationArgumentDraft<'_>],
) -> Result<WorkflowInvocationBinding, WorkflowCommandError> {
    if arguments.len() > MAX_ARGUMENTS {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::InvalidArgument,
            ARGUMENT_NAME_FIELD,
            TOO_MANY_INPUTS_VALUE,
        ));
    }
    let provided = collect_arguments(arguments)?;
    for name in provided.keys() {
        if !manifest.input_schema().iter().any(|field| field.name() == name.as_str()) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::UnknownArgument,
                ARGUMENT_NAME_FIELD,
                name,
            ));
        }
    }
    let mut bound = Vec::with_capacity(provided.len());
    for field in manifest.input_schema() {
        match provided.get(field.name()) {
            Some(value) => {
                if !field.value_type().accepts(value) {
                    return Err(WorkflowCommandError::new(
                        WorkflowCommandErrorKind::ArgumentTypeMismatch,
                        field.name(),
                        value.value_type().as_str(),
                    ));
                }
                bound.push(WorkflowInvocationArgument {
                    name: String::from(field.name()),
                    value: value.clone(),
                });
            }
            None if field.required() => {
                return Err(WorkflowCommandError::new(
                    WorkflowCommandErrorKind::MissingArgument,
                    field.name(),
                    EMPTY_VALUE,
                ));
            }
            None => {}
        }
    }
    Ok(WorkflowInvocationBinding {
        command_name: String::from(manifest.name()),
        arguments: bound,
    })
}

/// Host policy for preparing a workflow command invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandPolicy {
    allowed_capabilities: Vec<WorkflowCommandCapability>,
    allowed_postures: Vec<WorkflowCommandSideEffectPosture>,
}

impl WorkflowCommandPolicy {
    /// Builds a workflow command policy from explicit allowlists.
    #[must_use]
    pub fn new(
        allowed_capabilities: Vec<WorkflowCommandCapability>,
        allowed_postures: Vec<WorkflowCommandSideEffectPosture>,
    ) -> Self {
        Self {
            allowed_capabilities,
            allowed_postures,
        }
    }

    /// Returns a policy that admits read-only command planning only.
    #[must_use]
    pub fn read_only() -> Self {
        Self::new(
            vec![
                WorkflowCommandCapability::ReadContext,
                WorkflowCommandCapability::CallModel,
                WorkflowCommandCapability::CallTool,
                WorkflowCommandCapability::ValidateOutput,
            ],
            vec![WorkflowCommandSideEffectPosture::ReadOnly],
        )
    }

    /// Returns allowed capability labels.
    #[must_use]
    pub fn allowed_capabilities(&self) -> &[WorkflowCommandCapability] {
        self.allowed_capabilities.as_slice()
    }

    /// Returns allowed side-effect postures.
    #[must_use]
    pub fn allowed_postures(&self) -> &[WorkflowCommandSideEffectPosture] {
        self.allowed_postures.as_slice()
    }
}

/// A policy-admitted invocation plan that is safe to hand to prompt rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowInvocationPlan {
    binding: WorkflowInvocationBinding,
    side_effect_posture: WorkflowCommandSideEffectPosture,
    required_capabilities: Vec<WorkflowCommandCapability>,
    output_contract: WorkflowCommandOutputContract,
}

impl WorkflowInvocationPlan {
    /// Returns bound invocation arguments.
    #[must_use]
    pub const fn binding(&self) -> &WorkflowInvocationBinding {
        &self.binding
    }

    /// Returns the admitted side-effect posture.
    #[must_use]
    pub const fn side_effect_posture(&self) -> WorkflowCommandSideEffectPosture {
        self.side_effect_posture
    }

    /// Returns the admitted required capabilities.
    #[must_use]
    pub fn required_capabilities(&self) -> &[WorkflowCommandCapability] {
        self.required_capabilities.as_slice()
    }

    /// Returns the output contract that must validate completion.
    #[must_use]
    pub const fn output_contract(&self) -> &WorkflowCommandOutputContract {
        &self.output_contract
    }
}

/// Binds and policy-checks a workflow command before any host side effect.
///
/// # Errors
///
/// Returns [`WorkflowCommandError`] when argument binding fails or policy denies
/// the command posture/capabilities.
pub fn prepare_workflow_command_invocation(
    manifest: &WorkflowCommandManifest,
    arguments: &[WorkflowInvocationArgumentDraft<'_>],
    policy: &WorkflowCommandPolicy,
) -> Result<WorkflowInvocationPlan, WorkflowCommandError> {
    let binding = bind_workflow_command_arguments(manifest, arguments)?;
    if !policy.allowed_postures().contains(&manifest.side_effect_posture()) {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::PolicyDenied,
            SIDE_EFFECT_POSTURE_FIELD,
            POLICY_POSTURE_DENIED_VALUE,
        ));
    }
    for capability in manifest.capabilities() {
        if !policy.allowed_capabilities().contains(capability) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::PolicyDenied,
                CAPABILITIES_FIELD,
                POLICY_CAPABILITY_DENIED_VALUE,
            ));
        }
    }
    Ok(WorkflowInvocationPlan {
        binding,
        side_effect_posture: manifest.side_effect_posture(),
        required_capabilities: manifest.capabilities().to_vec(),
        output_contract: manifest.output_contract().clone(),
    })
}

// r[impl agent_harness_runtime.claude_code_commands.outputs]
/// Validation status attached to validation-report command output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowValidationStatus {
    /// Validation succeeded.
    Passed,
    /// Validation failed.
    Failed,
    /// Validation could not run and needs follow-up.
    Blocked,
}

impl WorkflowValidationStatus {
    /// Returns the stable validation-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

/// Draft output emitted by a workflow command executor or model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkflowCommandOutputDraft<'a> {
    /// Actual output kind.
    pub kind: WorkflowCommandOutputKind,
    /// Output body. The reusable contract stores only size and BLAKE3 hash.
    pub body: &'a str,
    /// Structured schema label when the output is a structured DTO.
    pub structured_schema: Option<&'a str>,
    /// Validation status when the output is a validation report.
    pub validation_status: Option<WorkflowValidationStatus>,
}

/// Validated workflow command output evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandOutput {
    kind: WorkflowCommandOutputKind,
    body_chars: u64,
    body_hash: String,
    structured_schema: Option<String>,
    validation_status: Option<WorkflowValidationStatus>,
}

impl WorkflowCommandOutput {
    /// Validates output evidence against the command output contract.
    ///
    /// # Errors
    ///
    /// Returns [`WorkflowCommandError`] when output kind, schema, validation
    /// status, or body safety mismatches the contract.
    pub fn validate(
        contract: &WorkflowCommandOutputContract,
        draft: &WorkflowCommandOutputDraft<'_>,
    ) -> Result<Self, WorkflowCommandError> {
        validate_output_contract_match(contract, draft)?;
        let body = validate_safe_summary(FieldText {
            field: OUTPUT_BODY_FIELD,
            value: draft.body,
        })?;
        Ok(Self {
            kind: draft.kind,
            body_chars: count_chars_u64(body),
            body_hash: blake3_hex(body.as_bytes()),
            structured_schema: draft.structured_schema.map(String::from),
            validation_status: draft.validation_status,
        })
    }

    /// Returns the output kind.
    #[must_use]
    pub const fn kind(&self) -> WorkflowCommandOutputKind {
        self.kind
    }

    /// Returns the number of Unicode scalar values in the body.
    #[must_use]
    pub const fn body_chars(&self) -> u64 {
        self.body_chars
    }

    /// Returns the BLAKE3 hash of the output body.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the structured schema label when present.
    #[must_use]
    pub fn structured_schema(&self) -> Option<&str> {
        self.structured_schema.as_deref()
    }

    /// Returns the validation status when present.
    #[must_use]
    pub const fn validation_status(&self) -> Option<WorkflowValidationStatus> {
        self.validation_status
    }
}

/// Validates workflow command output evidence against a contract.
///
/// # Errors
///
/// Returns [`WorkflowCommandError`] when output evidence does not match the
/// declared contract.
pub fn validate_workflow_command_output(
    contract: &WorkflowCommandOutputContract,
    draft: &WorkflowCommandOutputDraft<'_>,
) -> Result<WorkflowCommandOutput, WorkflowCommandError> {
    WorkflowCommandOutput::validate(contract, draft)
}

/// Workflow command validation, binding, registry, or output error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowCommandError {
    kind: WorkflowCommandErrorKind,
    field: String,
    value: String,
}

impl WorkflowCommandError {
    fn new(kind: WorkflowCommandErrorKind, field: &str, value: &str) -> Self {
        Self {
            kind,
            field: String::from(field),
            value: String::from(value),
        }
    }

    /// Returns the stable error kind.
    #[must_use]
    pub const fn kind(&self) -> WorkflowCommandErrorKind {
        self.kind
    }

    /// Returns the field or argument name that failed validation.
    #[must_use]
    pub fn field(&self) -> &str {
        self.field.as_str()
    }

    /// Returns the rejected value or safe diagnostic label.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Stable workflow command error kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowCommandErrorKind {
    /// Command name was empty, unsafe, too long, or not in the command grammar.
    InvalidName,
    /// Bounded safe text was empty, overlong, or leak-looking.
    UnsafeText,
    /// Manifest omitted the input schema field.
    MissingInputSchema,
    /// Input schema had duplicates, unsafe names, or too many fields.
    InvalidInputSchema,
    /// Manifest omitted its side-effect posture.
    MissingSideEffectPosture,
    /// Capability set was empty, duplicated, or too large.
    InvalidCapabilities,
    /// Capability was not admitted by the declared side-effect posture.
    UndeclaredCapability,
    /// Manifest omitted validation gates.
    MissingValidationGate,
    /// Manifest omitted or malformed its output contract.
    MissingOutputContract,
    /// Source/provenance metadata was missing or unsafe.
    InvalidProvenance,
    /// Source was not trusted for advertisement or invocation.
    UntrustedSource,
    /// Invocation supplied a duplicate argument.
    DuplicateArgument,
    /// Invocation supplied an unknown argument.
    UnknownArgument,
    /// Invocation omitted a required argument.
    MissingArgument,
    /// Invocation supplied a malformed argument value.
    InvalidArgument,
    /// Invocation argument value type did not match the schema.
    ArgumentTypeMismatch,
    /// Host policy denied the command before expansion or dispatch.
    PolicyDenied,
    /// Output evidence did not match the declared output contract.
    OutputContractMismatch,
}

fn count_chars_u64(value: &str) -> u64 {
    value.chars().fold(0_u64, |count, _character| count.saturating_add(1))
}

fn validate_input_schema(
    schema: Option<&[WorkflowCommandInputFieldDraft<'_>]>,
) -> Result<Vec<WorkflowCommandInputField>, WorkflowCommandError> {
    let schema = schema.ok_or_else(|| {
        WorkflowCommandError::new(
            WorkflowCommandErrorKind::MissingInputSchema,
            INPUT_SCHEMA_FIELD,
            MISSING_INPUT_SCHEMA_VALUE,
        )
    })?;
    if schema.len() > MAX_INPUT_FIELDS {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::InvalidInputSchema,
            INPUT_SCHEMA_FIELD,
            TOO_MANY_INPUTS_VALUE,
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut fields = Vec::with_capacity(schema.len());
    for field in schema {
        let name = validate_safe_label(FieldText {
            field: INPUT_FIELD_NAME_FIELD,
            value: field.name,
        })?;
        if !seen.insert(String::from(name)) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::InvalidInputSchema,
                INPUT_FIELD_NAME_FIELD,
                name,
            ));
        }
        fields.push(WorkflowCommandInputField {
            name: String::from(name),
            value_type: field.value_type,
            required: field.required,
        });
    }
    Ok(fields)
}

fn validate_capabilities(
    side_effect_posture: WorkflowCommandSideEffectPosture,
    capabilities: &[WorkflowCommandCapability],
) -> Result<Vec<WorkflowCommandCapability>, WorkflowCommandError> {
    if capabilities.is_empty() {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::InvalidCapabilities,
            CAPABILITIES_FIELD,
            MISSING_CAPABILITIES_VALUE,
        ));
    }
    if capabilities.len() > MAX_CAPABILITIES {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::InvalidCapabilities,
            CAPABILITIES_FIELD,
            TOO_MANY_CAPABILITIES_VALUE,
        ));
    }
    let mut normalized = Vec::with_capacity(capabilities.len());
    for capability in capabilities {
        if !side_effect_posture.permits(*capability) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::UndeclaredCapability,
                CAPABILITIES_FIELD,
                capability.as_str(),
            ));
        }
        if normalized.contains(capability) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::InvalidCapabilities,
                CAPABILITIES_FIELD,
                capability.as_str(),
            ));
        }
        normalized.push(*capability);
    }
    normalized.sort();
    Ok(normalized)
}

fn validate_validation_gates(gates: &[&str]) -> Result<Vec<String>, WorkflowCommandError> {
    if gates.is_empty() {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::MissingValidationGate,
            VALIDATION_GATES_FIELD,
            MISSING_GATES_VALUE,
        ));
    }
    if gates.len() > MAX_VALIDATION_GATES {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::MissingValidationGate,
            VALIDATION_GATES_FIELD,
            TOO_MANY_GATES_VALUE,
        ));
    }
    let mut normalized = Vec::with_capacity(gates.len());
    for gate in gates {
        normalized.push(String::from(validate_safe_label(FieldText {
            field: VALIDATION_GATES_FIELD,
            value: gate,
        })?));
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn validate_provenance(
    draft: &WorkflowCommandProvenanceDraft<'_>,
) -> Result<WorkflowCommandProvenance, WorkflowCommandError> {
    let source_id = validate_safe_label(FieldText {
        field: SOURCE_ID_FIELD,
        value: draft.source_id,
    })?;
    let content_hash = validate_blake3_hex(FieldText {
        field: CONTENT_HASH_FIELD,
        value: draft.content_hash,
    })?;
    if draft.trust_status != WorkflowCommandTrustStatus::Trusted {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::UntrustedSource,
            TRUST_STATUS_FIELD,
            draft.trust_status.as_str(),
        ));
    }
    Ok(WorkflowCommandProvenance {
        source_id: String::from(source_id),
        source_scope: draft.source_scope,
        trust_status: draft.trust_status,
        content_hash: String::from(content_hash),
    })
}

fn collect_arguments(
    arguments: &[WorkflowInvocationArgumentDraft<'_>],
) -> Result<std::collections::BTreeMap<String, WorkflowCommandArgumentValue>, WorkflowCommandError> {
    let mut provided = std::collections::BTreeMap::new();
    for argument in arguments {
        let name = validate_safe_label(FieldText {
            field: ARGUMENT_NAME_FIELD,
            value: argument.name,
        })?;
        if provided.contains_key(name) {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::DuplicateArgument,
                ARGUMENT_NAME_FIELD,
                name,
            ));
        }
        provided.insert(String::from(name), argument.value.to_owned()?);
    }
    Ok(provided)
}

fn validate_output_contract_match(
    contract: &WorkflowCommandOutputContract,
    draft: &WorkflowCommandOutputDraft<'_>,
) -> Result<(), WorkflowCommandError> {
    if contract.kind() != draft.kind {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::OutputContractMismatch,
            OUTPUT_KIND_FIELD,
            draft.kind.as_str(),
        ));
    }
    match (contract.structured_schema(), draft.structured_schema) {
        (Some(expected), Some(actual)) if expected == actual => {}
        (Some(_), Some(actual)) => {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::OutputContractMismatch,
                STRUCTURED_SCHEMA_FIELD,
                actual,
            ));
        }
        (Some(_), None) => {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::OutputContractMismatch,
                STRUCTURED_SCHEMA_FIELD,
                MISSING_SCHEMA_LABEL_VALUE,
            ));
        }
        (None, Some(_)) => {
            return Err(WorkflowCommandError::new(
                WorkflowCommandErrorKind::OutputContractMismatch,
                STRUCTURED_SCHEMA_FIELD,
                UNEXPECTED_SCHEMA_LABEL_VALUE,
            ));
        }
        (None, None) => {}
    }
    if contract.kind() == WorkflowCommandOutputKind::ValidationReport && draft.validation_status.is_none() {
        return Err(WorkflowCommandError::new(
            WorkflowCommandErrorKind::OutputContractMismatch,
            OUTPUT_KIND_FIELD,
            VALIDATION_STATUS_REQUIRED_VALUE,
        ));
    }
    Ok(())
}

fn registry_diagnostic(
    name: &str,
    kind: WorkflowCommandRegistryDiagnosticKind,
    source_ids: Vec<String>,
    selected_source_id: Option<&str>,
) -> WorkflowCommandRegistryDiagnostic {
    WorkflowCommandRegistryDiagnostic {
        command_name: String::from(name),
        kind,
        source_ids,
        selected_source_id: selected_source_id.map(String::from),
    }
}

fn sorted_source_ids(manifests: &[&WorkflowCommandManifest]) -> Vec<String> {
    let mut source_ids: Vec<String> =
        manifests.iter().map(|manifest| String::from(manifest.provenance().source_id())).collect();
    source_ids.sort();
    source_ids
}

fn select_precedence_winner<'a>(manifests: &[&'a WorkflowCommandManifest]) -> Option<&'a WorkflowCommandManifest> {
    manifests.iter().copied().max_by(|left, right| {
        left.provenance()
            .source_scope()
            .precedence_rank()
            .cmp(&right.provenance().source_scope().precedence_rank())
            .then_with(|| right.provenance().source_id().cmp(left.provenance().source_id()))
    })
}

fn validate_command_name(value: &str) -> Result<&str, WorkflowCommandError> {
    if value.is_empty() || value.len() > MAX_COMMAND_NAME_CHARS || !is_command_name(value) {
        return Err(WorkflowCommandError::new(WorkflowCommandErrorKind::InvalidName, COMMAND_NAME_FIELD, value));
    }
    Ok(value)
}

fn is_command_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || matches!(character, '-' | '_')
        })
}

#[derive(Clone, Copy)]
struct FieldText<'a> {
    field: &'a str,
    value: &'a str,
}

#[derive(Clone, Copy)]
struct BoundedText<'a> {
    text: FieldText<'a>,
    max_chars: usize,
}

fn validate_safe_label(input: FieldText<'_>) -> Result<&str, WorkflowCommandError> {
    if input.value.is_empty() {
        return Err(unsafe_text_error(input));
    }
    if input.value.len() > MAX_SAFE_LABEL_CHARS {
        return Err(unsafe_text_error(input));
    }
    if !is_safe_label(input.value) {
        return Err(unsafe_text_error(input));
    }
    if leak_marker(input.value).is_some() {
        return Err(unsafe_text_error(input));
    }
    Ok(input.value)
}

fn validate_safe_summary(input: FieldText<'_>) -> Result<&str, WorkflowCommandError> {
    validate_bounded_text(BoundedText {
        text: input,
        max_chars: MAX_SAFE_SUMMARY_CHARS,
    })
}

fn validate_argument_text(input: FieldText<'_>) -> Result<&str, WorkflowCommandError> {
    validate_bounded_text(BoundedText {
        text: input,
        max_chars: MAX_ARGUMENT_TEXT_CHARS,
    })
}

fn validate_bounded_text(input: BoundedText<'_>) -> Result<&str, WorkflowCommandError> {
    if input.text.value.is_empty() {
        return Err(unsafe_text_error(input.text));
    }
    if input.text.value.chars().count() > input.max_chars {
        return Err(unsafe_text_error(input.text));
    }
    if leak_marker(input.text.value).is_some() {
        return Err(unsafe_text_error(input.text));
    }
    Ok(input.text.value)
}

fn validate_blake3_hex(input: FieldText<'_>) -> Result<&str, WorkflowCommandError> {
    if input.value.len() != BLAKE3_HEX_CHARS {
        return Err(WorkflowCommandError::new(WorkflowCommandErrorKind::InvalidProvenance, input.field, input.value));
    }
    if !input.value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(WorkflowCommandError::new(WorkflowCommandErrorKind::InvalidProvenance, input.field, input.value));
    }
    Ok(input.value)
}

fn is_safe_label(value: &str) -> bool {
    value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.'))
}

fn unsafe_text_error(input: FieldText<'_>) -> WorkflowCommandError {
    WorkflowCommandError::new(WorkflowCommandErrorKind::UnsafeText, input.field, input.value)
}

fn leak_marker(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    LEAK_MARKERS.iter().copied().find(|marker| lower.contains(marker))
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND_NAME: &str = "review_changes";
    const DUPLICATE_COMMAND_NAME: &str = "review_changes";
    const COMMAND_SUMMARY: &str = "Review the current patch and report safe findings.";
    const INPUT_FOCUS: &str = "focus";
    const INPUT_INCLUDE_TESTS: &str = "include_tests";
    const INPUT_LIMIT: &str = "limit";
    const RETRIEVAL_BUDGET: &str = "focused_review";
    const VALIDATION_GATE: &str = "cargo_test_runtime";
    const HOST_SOURCE_ID: &str = "host_builtin";
    const USER_SOURCE_ID: &str = "user_local";
    const PROJECT_SOURCE_ID: &str = "project_commands";
    const REVIEW_SCHEMA: &str = "animus.review.v1";
    const REVIEW_BODY: &str = "safe review body";
    const ARG_FOCUS: &str = "runtime";
    const INVALID_NAME: &str = "/review";
    const UNKNOWN_ARGUMENT: &str = "unknown";
    const RAW_PROMPT_LEAK: &str = "raw_prompt: include private text";
    const INVALID_HASH: &str = "not-a-blake3-hash";
    const INVALID_MANIFEST_CASES: usize = 6;
    const INVALID_BINDING_CASES: usize = 4;
    const OUTPUT_KIND_COUNT: usize = 6;
    const DUPLICATE_DIAGNOSTICS: usize = 2;

    static INPUTS: &[WorkflowCommandInputFieldDraft<'static>] = &[
        WorkflowCommandInputFieldDraft {
            name: INPUT_FOCUS,
            value_type: WorkflowCommandArgumentType::Text,
            required: true,
        },
        WorkflowCommandInputFieldDraft {
            name: INPUT_INCLUDE_TESTS,
            value_type: WorkflowCommandArgumentType::Boolean,
            required: false,
        },
        WorkflowCommandInputFieldDraft {
            name: INPUT_LIMIT,
            value_type: WorkflowCommandArgumentType::Integer,
            required: false,
        },
    ];
    static READ_ONLY_CAPABILITIES: &[WorkflowCommandCapability] = &[
        WorkflowCommandCapability::ReadContext,
        WorkflowCommandCapability::CallModel,
        WorkflowCommandCapability::ValidateOutput,
    ];
    static MUTATING_CAPABILITIES: &[WorkflowCommandCapability] = &[
        WorkflowCommandCapability::ReadContext,
        WorkflowCommandCapability::MutateWorkspace,
    ];
    static VALIDATION_GATES: &[&str] = &[VALIDATION_GATE];

    fn content_hash(label: &str) -> String {
        blake3_hex(label.as_bytes())
    }

    fn provenance(
        source_id: &'static str,
        source_scope: WorkflowCommandSourceScope,
        trust_status: WorkflowCommandTrustStatus,
    ) -> WorkflowCommandProvenanceDraft<'static> {
        WorkflowCommandProvenanceDraft {
            source_id,
            source_scope,
            trust_status,
            content_hash: Box::leak(content_hash(source_id).into_boxed_str()),
        }
    }

    fn output_contract(kind: WorkflowCommandOutputKind) -> WorkflowCommandOutputContractDraft<'static> {
        WorkflowCommandOutputContractDraft {
            kind,
            structured_schema: (kind == WorkflowCommandOutputKind::StructuredDto).then_some(REVIEW_SCHEMA),
        }
    }

    fn manifest_draft(
        source_id: &'static str,
        source_scope: WorkflowCommandSourceScope,
    ) -> WorkflowCommandManifestDraft<'static> {
        WorkflowCommandManifestDraft {
            name: COMMAND_NAME,
            summary: COMMAND_SUMMARY,
            input_schema: Some(INPUTS),
            capabilities: READ_ONLY_CAPABILITIES,
            side_effect_posture: Some(WorkflowCommandSideEffectPosture::ReadOnly),
            retrieval_budget: RETRIEVAL_BUDGET,
            validation_gates: VALIDATION_GATES,
            output_contract: Some(output_contract(WorkflowCommandOutputKind::Review)),
            provenance: provenance(source_id, source_scope, WorkflowCommandTrustStatus::Trusted),
        }
    }

    fn valid_manifest() -> WorkflowCommandManifest {
        WorkflowCommandManifest::validate(&manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host))
            .expect("valid workflow command manifest should validate")
    }

    #[test]
    fn workflow_command_manifest_validation_accepts_typed_safe_commands() {
        // r[verify agent_harness_runtime.claude_code_commands.manifest]
        let manifest = valid_manifest();

        assert_eq!(manifest.name(), COMMAND_NAME);
        assert_eq!(manifest.input_schema().len(), INPUTS.len());
        assert_eq!(manifest.capabilities(), READ_ONLY_CAPABILITIES);
        assert_eq!(manifest.validation_gates()[0], VALIDATION_GATE);
        assert_eq!(manifest.output_contract().kind(), WorkflowCommandOutputKind::Review);
        assert_eq!(manifest.provenance().source_id(), HOST_SOURCE_ID);
    }

    #[test]
    fn workflow_command_manifest_validation_rejects_unsafe_or_incomplete_commands() {
        // r[verify agent_harness_runtime.claude_code_commands.manifest.invalid]
        let invalid_name = WorkflowCommandManifestDraft {
            name: INVALID_NAME,
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let missing_schema = WorkflowCommandManifestDraft {
            input_schema: None,
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let missing_posture = WorkflowCommandManifestDraft {
            side_effect_posture: None,
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let undeclared_capability = WorkflowCommandManifestDraft {
            capabilities: MUTATING_CAPABILITIES,
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let missing_output = WorkflowCommandManifestDraft {
            output_contract: None,
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let untrusted_project = WorkflowCommandManifestDraft {
            provenance: provenance(
                PROJECT_SOURCE_ID,
                WorkflowCommandSourceScope::Project,
                WorkflowCommandTrustStatus::Untrusted,
            ),
            ..manifest_draft(PROJECT_SOURCE_ID, WorkflowCommandSourceScope::Project)
        };
        let cases = [
            (invalid_name, WorkflowCommandErrorKind::InvalidName),
            (missing_schema, WorkflowCommandErrorKind::MissingInputSchema),
            (missing_posture, WorkflowCommandErrorKind::MissingSideEffectPosture),
            (undeclared_capability, WorkflowCommandErrorKind::UndeclaredCapability),
            (missing_output, WorkflowCommandErrorKind::MissingOutputContract),
            (untrusted_project, WorkflowCommandErrorKind::UntrustedSource),
        ];

        assert_eq!(cases.len(), INVALID_MANIFEST_CASES);
        for (draft, expected_kind) in cases {
            let error = WorkflowCommandManifest::validate(&draft).expect_err("invalid manifest must not be advertised");
            assert_eq!(error.kind(), expected_kind);
            assert!(!error.field().is_empty());
        }
    }

    #[test]
    fn workflow_command_duplicate_registry_policy_is_deterministic_and_safe() {
        // r[verify agent_harness_runtime.claude_code_commands.registry]
        // r[verify agent_harness_runtime.claude_code_commands.registry.duplicate]
        let host = valid_manifest();
        let user = WorkflowCommandManifest::validate(&manifest_draft(USER_SOURCE_ID, WorkflowCommandSourceScope::User))
            .expect("user command should validate");
        let manifests = [host.clone(), user.clone()];

        let disabled =
            build_workflow_command_registry(manifests.as_slice(), WorkflowCommandRegistryPolicy::fail_closed());
        assert!(disabled.lookup(DUPLICATE_COMMAND_NAME).is_none());
        assert_eq!(disabled.diagnostics().len(), DUPLICATE_DIAGNOSTICS);

        let selected = build_workflow_command_registry(
            manifests.as_slice(),
            WorkflowCommandRegistryPolicy::prefer_source_precedence(),
        );
        let command = selected.lookup(DUPLICATE_COMMAND_NAME).expect("host source should win deterministic precedence");
        assert_eq!(command.provenance().source_id(), HOST_SOURCE_ID);
        assert!(
            selected
                .diagnostics()
                .iter()
                .flat_map(WorkflowCommandRegistryDiagnostic::source_ids)
                .all(|source| !source.contains('/'))
        );
    }

    #[test]
    fn workflow_command_invocation_binding_accepts_schema_ordered_arguments() {
        // r[verify agent_harness_runtime.claude_code_commands.binding]
        let manifest = valid_manifest();
        let binding = bind_workflow_command_arguments(&manifest, &[
            WorkflowInvocationArgumentDraft {
                name: INPUT_INCLUDE_TESTS,
                value: WorkflowCommandArgumentValueDraft::Boolean(true),
            },
            WorkflowInvocationArgumentDraft {
                name: INPUT_FOCUS,
                value: WorkflowCommandArgumentValueDraft::Text(ARG_FOCUS),
            },
        ])
        .expect("valid arguments should bind before expansion");

        assert_eq!(binding.command_name(), COMMAND_NAME);
        assert_eq!(binding.arguments()[0].name(), INPUT_FOCUS);
        assert_eq!(binding.arguments()[1].name(), INPUT_INCLUDE_TESTS);
    }

    #[test]
    fn workflow_command_malformed_arguments_stop_before_prompt_or_dispatch() {
        // r[verify agent_harness_runtime.claude_code_commands.binding.invalid]
        let manifest = valid_manifest();
        let missing_required: &[WorkflowInvocationArgumentDraft<'_>] = &[];
        let unknown = &[WorkflowInvocationArgumentDraft {
            name: UNKNOWN_ARGUMENT,
            value: WorkflowCommandArgumentValueDraft::Text(ARG_FOCUS),
        }];
        let malformed = &[WorkflowInvocationArgumentDraft {
            name: INPUT_FOCUS,
            value: WorkflowCommandArgumentValueDraft::Text(RAW_PROMPT_LEAK),
        }];
        let wrong_type = &[WorkflowInvocationArgumentDraft {
            name: INPUT_FOCUS,
            value: WorkflowCommandArgumentValueDraft::Boolean(true),
        }];
        let cases = [
            (missing_required, WorkflowCommandErrorKind::MissingArgument),
            (unknown, WorkflowCommandErrorKind::UnknownArgument),
            (malformed, WorkflowCommandErrorKind::UnsafeText),
            (wrong_type, WorkflowCommandErrorKind::ArgumentTypeMismatch),
        ];

        assert_eq!(cases.len(), INVALID_BINDING_CASES);
        for (arguments, expected_kind) in cases {
            let error = prepare_workflow_command_invocation(&manifest, arguments, &WorkflowCommandPolicy::read_only())
                .expect_err("bad arguments must stop before workflow side effects");
            assert_eq!(error.kind(), expected_kind);
        }
    }

    #[test]
    fn workflow_command_policy_denies_command_text_side_effect_bypass() {
        // r[verify agent_harness_runtime.claude_code_commands.scope]
        // r[verify agent_harness_runtime.claude_code_commands.scope.no_policy_bypass]
        let mutating = WorkflowCommandManifest::validate(&WorkflowCommandManifestDraft {
            capabilities: MUTATING_CAPABILITIES,
            side_effect_posture: Some(WorkflowCommandSideEffectPosture::WorkspaceMutation),
            output_contract: Some(output_contract(WorkflowCommandOutputKind::PatchSummary)),
            ..manifest_draft(PROJECT_SOURCE_ID, WorkflowCommandSourceScope::Project)
        })
        .expect("mutating command manifest is valid only with explicit posture");
        let error = prepare_workflow_command_invocation(
            &mutating,
            &[WorkflowInvocationArgumentDraft {
                name: INPUT_FOCUS,
                value: WorkflowCommandArgumentValueDraft::Text(ARG_FOCUS),
            }],
            &WorkflowCommandPolicy::read_only(),
        )
        .expect_err("read-only policy must block mutating workflow before prompt expansion");

        assert_eq!(error.kind(), WorkflowCommandErrorKind::PolicyDenied);
        assert_eq!(error.field(), SIDE_EFFECT_POSTURE_FIELD);
    }

    #[test]
    fn workflow_command_output_contract_validation_accepts_declared_shapes() {
        // r[verify agent_harness_runtime.claude_code_commands.outputs]
        let cases = [
            (WorkflowCommandOutputKind::Explanation, None, None),
            (WorkflowCommandOutputKind::Review, None, None),
            (WorkflowCommandOutputKind::PatchSummary, None, None),
            (WorkflowCommandOutputKind::ValidationReport, None, Some(WorkflowValidationStatus::Passed)),
            (WorkflowCommandOutputKind::CommitMessageProposal, None, None),
            (WorkflowCommandOutputKind::StructuredDto, Some(REVIEW_SCHEMA), None),
        ];

        assert_eq!(cases.len(), OUTPUT_KIND_COUNT);
        for (kind, schema, status) in cases {
            let contract = WorkflowCommandOutputContract::validate(&WorkflowCommandOutputContractDraft {
                kind,
                structured_schema: schema,
            })
            .expect("contract should validate");
            let output = validate_workflow_command_output(&contract, &WorkflowCommandOutputDraft {
                kind,
                body: REVIEW_BODY,
                structured_schema: schema,
                validation_status: status,
            })
            .expect("matching output should validate");
            assert_eq!(output.kind(), kind);
            assert!(!output.body_hash().is_empty());
        }
    }

    #[test]
    fn workflow_command_output_contract_mismatches_are_not_successful_completion() {
        // r[verify agent_harness_runtime.claude_code_commands.outputs.mismatch]
        let validation_contract =
            WorkflowCommandOutputContract::validate(&output_contract(WorkflowCommandOutputKind::ValidationReport))
                .expect("validation report contract should validate");
        let structured_contract =
            WorkflowCommandOutputContract::validate(&output_contract(WorkflowCommandOutputKind::StructuredDto))
                .expect("structured contract should validate");
        let wrong_kind = WorkflowCommandOutputDraft {
            kind: WorkflowCommandOutputKind::Explanation,
            body: REVIEW_BODY,
            structured_schema: None,
            validation_status: None,
        };
        let missing_status = WorkflowCommandOutputDraft {
            kind: WorkflowCommandOutputKind::ValidationReport,
            body: REVIEW_BODY,
            structured_schema: None,
            validation_status: None,
        };
        let wrong_schema = WorkflowCommandOutputDraft {
            kind: WorkflowCommandOutputKind::StructuredDto,
            body: REVIEW_BODY,
            structured_schema: Some("other.schema"),
            validation_status: None,
        };
        let leaking_body = WorkflowCommandOutputDraft {
            kind: WorkflowCommandOutputKind::ValidationReport,
            body: RAW_PROMPT_LEAK,
            structured_schema: None,
            validation_status: Some(WorkflowValidationStatus::Failed),
        };

        let errors = [
            validate_workflow_command_output(&validation_contract, &wrong_kind).expect_err("wrong kind must fail"),
            validate_workflow_command_output(&validation_contract, &missing_status)
                .expect_err("missing status must fail"),
            validate_workflow_command_output(&structured_contract, &wrong_schema).expect_err("wrong schema must fail"),
            validate_workflow_command_output(&validation_contract, &leaking_body)
                .expect_err("leaking output body must fail"),
        ];

        assert!(errors.iter().any(|error| { error.kind() == WorkflowCommandErrorKind::OutputContractMismatch }));
        assert!(errors.iter().any(|error| error.kind() == WorkflowCommandErrorKind::UnsafeText));
    }

    #[test]
    fn workflow_command_invalid_provenance_hash_fails_closed() {
        let draft = WorkflowCommandManifestDraft {
            provenance: WorkflowCommandProvenanceDraft {
                source_id: HOST_SOURCE_ID,
                source_scope: WorkflowCommandSourceScope::Host,
                trust_status: WorkflowCommandTrustStatus::Trusted,
                content_hash: INVALID_HASH,
            },
            ..manifest_draft(HOST_SOURCE_ID, WorkflowCommandSourceScope::Host)
        };
        let error = WorkflowCommandManifest::validate(&draft).expect_err("invalid content identity must fail");

        assert_eq!(error.kind(), WorkflowCommandErrorKind::InvalidProvenance);
        assert_eq!(error.field(), CONTENT_HASH_FIELD);
    }

    #[test]
    fn workflow_command_schema_label_is_stable() {
        assert_eq!(WORKFLOW_COMMAND_SCHEMA, "animus.workflow-command.v1");
        assert_eq!(SOURCE_IDS_FIELD, "source_ids");
    }
}
