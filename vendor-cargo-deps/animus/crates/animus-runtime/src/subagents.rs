//! Unified subagent child-run facade, coordination, lineage, UX, and conformance helpers.
//!
//! The module is deliberately a pure functional core. It validates already-loaded
//! DTOs, computes deterministic identities, reduces child outcomes, and describes
//! host actions without spawning workers, reading files, opening networks,
//! constructing provider clients, or importing child output by ambient authority.

#![allow(unknown_lints, excessive_file_length)]

const BLAKE3_ALGORITHM: &str = "BLAKE3";
const HASH_PART_SEPARATOR: &str = "\u{1f}";
const HASH_HEX_CHARS: usize = 64;
const MAX_LABEL_CHARS: usize = 128;
const MAX_TEXT_CHARS: usize = 4_096;
const MAX_PACKET_ITEMS: u16 = 32;
const MAX_PACKET_TOKENS: u32 = 8_192;
const MAX_EVIDENCE_REFS: usize = 16;
const MAX_CAPABILITY_CATALOG_ENTRIES: usize = 128;
const CAPABILITY_CEILING_COUNT: usize = 5;
const CONTEXT_PACKET_HASH_PREFIX_PARTS: usize = 2;
const MAX_FANOUT_CHILDREN: u16 = 16;
const MAX_FANOUT_CONCURRENCY: u16 = 8;
const MAX_DETAIL_IMPORT_CHARS: u32 = 16_384;
const MIN_POSITIVE_U16: u16 = 1;
const MIN_POSITIVE_U32: u32 = 1;
const SAFE_HASH_DOMAIN_CAPABILITY: &str = "animus.subagent.capability.v1";
const SAFE_HASH_DOMAIN_CONTEXT: &str = "animus.subagent.context-packet.v1";
const SAFE_HASH_DOMAIN_ADMISSION: &str = "animus.subagent.admission.v1";
const SAFE_HASH_DOMAIN_RECEIPT: &str = "animus.subagent.receipt.v1";
const SAFE_HASH_DOMAIN_LINEAGE: &str = "animus.subagent.lineage.v1";
const UNSUPPORTED_KIND_LABEL: &str = "unsupported";
const WILDCARD_STAR: &str = "*";
const WILDCARD_ALL: &str = "all";
const FIELD_CAPABILITY: &str = "capability";
const FIELD_CATALOG: &str = "capability_catalog";
const FIELD_CONTEXT: &str = "context_packet";
const FIELD_CHILD: &str = "child_run";
const FIELD_SERVICE: &str = "subagent_service";
const FIELD_REPORT: &str = "subagent_report";
const FIELD_PLAN: &str = "fanout_plan";
const FIELD_LINEAGE: &str = "lineage_record";
const FIELD_UX: &str = "subagent_ux";
const FIELD_EXTERNAL: &str = "external_fixture";
const UNSAFE_TEXT_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "RAW_CHILD_LOG:",
    "RAW_TRANSCRIPT:",
    "PROVIDER_PAYLOAD:",
    "UNBOUNDED_OUTPUT:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
];

#[derive(Clone, Copy)]
struct TextValidationInput<'a> {
    field: &'static str,
    value: &'a str,
}

macro_rules! validate_label {
    ($field:expr, $value:expr) => {
        validate_label_input(TextValidationInput {
            field: $field,
            value: $value,
        })
    };
}

macro_rules! normalize_label {
    ($field:expr, $value:expr) => {
        normalize_label_input(TextValidationInput {
            field: $field,
            value: $value,
        })
    };
}

macro_rules! validate_safe_text {
    ($field:expr, $value:expr) => {
        validate_safe_text_input(TextValidationInput {
            field: $field,
            value: $value,
        })
    };
}

macro_rules! reject_unsafe_text {
    ($field:expr, $value:expr) => {
        reject_unsafe_text_input(TextValidationInput {
            field: $field,
            value: $value,
        })
    };
}

/// High-level diagnostic emitted by pure subagent validation and reducers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentDiagnostic {
    /// Stable machine-readable failure class.
    pub kind: SubagentDiagnosticKind,
    /// Boundary field that caused or best explains the diagnostic.
    pub field: &'static str,
    /// Safe human-readable explanation without raw child output.
    pub message: String,
}

impl SubagentDiagnostic {
    /// Builds a safe diagnostic.
    #[must_use]
    pub fn new(
        kind: SubagentDiagnosticKind,
        field: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field,
            message: message.into(),
        }
    }
}

/// Machine-readable subagent diagnostic classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentDiagnosticKind {
    /// Required data is missing.
    MissingField,
    /// Text is empty after normalization.
    EmptyField,
    /// Text or collection exceeds configured bounds.
    TooLong,
    /// Text contains raw prompt, credential, provider payload, path, or unbounded output markers.
    UnsafeText,
    /// A label was unknown to the checked catalog.
    UnknownLabel,
    /// A normalized label was duplicated.
    DuplicateNormalizedLabel,
    /// A wildcard or all-authority label was requested.
    OverbroadAuthority,
    /// A catalog entry was malformed.
    MalformedCatalogEntry,
    /// A named tool was not authorized by the catalog or policy.
    UnauthorizedTool,
    /// A capability class is unsupported.
    UnsupportedCapabilityClass,
    /// Two labels collapse onto the same typed capability identity.
    LabelCollision,
    /// A capability was denied by a ceiling.
    DeniedByCeiling,
    /// A context packet exceeded a configured budget.
    OverBudget,
    /// Unsafe context was not admitted or failed leak validation.
    UnsafeContext,
    /// A kind-specific projection would lose source data.
    LossyProjection,
    /// The child-run kind has no typed validator.
    UnsupportedKind,
    /// A validation path tried to require execution authority.
    ExecutionAuthorityRequired,
    /// An injected service is missing or unavailable.
    ServiceUnavailable,
    /// A handle no longer matches the active child-run state.
    StaleHandle,
    /// A service denied the requested action.
    ServiceDenied,
    /// A service timed out.
    TimedOut,
    /// A service or child run was cancelled.
    Cancelled,
    /// A child report was malformed.
    MalformedReport,
    /// A child identifier is duplicated.
    DuplicateChild,
    /// A child outcome is missing.
    MissingChildOutcome,
    /// Mutation intents conflict.
    MutationConflict,
    /// A mutation attempted to bypass arbitration.
    MutationBypass,
    /// A parent-visible child import has no matching lineage.
    MissingLineage,
    /// Parent or child lineage identity does not match.
    LineageMismatch,
    /// A hash is not a BLAKE3 hex identity.
    InvalidHash,
    /// A UI command is not authorized by policy.
    UnauthorizedAction,
    /// A UI command descriptor is stale.
    StaleCommand,
    /// An external fixture expected failure class is missing.
    MissingExpectedFailure,
    /// External fixture protocol is malformed or unsupported.
    MalformedProtocol,
    /// External fixture requested ambient CLI discovery.
    AmbientDiscoveryDenied,
    /// External fixture requested inherited credentials.
    InheritedCredentialDenied,
    /// External fixture omitted an injected service.
    MissingService,
    /// External fixture requested a tool it did not declare.
    UndeclaredTool,
    /// External fixture requested unsafe output import.
    UnsafeOutputImport,
    /// External fixture observed result does not match expectation.
    FixtureExpectationMismatch,
}

/// Capability class used by typed subagent decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubagentCapabilityClass {
    /// Selected parent context can be read.
    ContextRead,
    /// Search or retrieval can be performed.
    Search,
    /// A named tool can be invoked.
    ToolUse,
    /// A named mutation intent can be proposed.
    MutationIntent,
    /// A workspace scope can be read or written according to its mode.
    WorkspaceAccess,
    /// A named model profile can be used.
    ModelUse,
    /// A host-admitted external service can be used.
    ExternalService,
    /// Unsupported class used only to prove fail-closed catalog validation.
    Unsupported,
}

/// Workspace capability access mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubagentWorkspaceAccessMode {
    /// Read-only workspace access.
    Read,
    /// Write intent over the workspace scope; later mutation policy still arbitrates effects.
    Write,
}

/// Typed subagent capability used by runtime decisions.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum TypedSubagentCapability {
    /// Read selected parent context.
    ContextRead {
        /// Stable public catalog label.
        stable_label: String,
        /// Safe context class label.
        context_label: String,
    },
    /// Perform safe search or retrieval.
    Search {
        /// Stable public catalog label.
        stable_label: String,
        /// Safe search index or source label.
        index_label: String,
    },
    /// Use a named tool.
    ToolUse {
        /// Stable public catalog label.
        stable_label: String,
        /// Tool label authorized by the catalog.
        tool_label: String,
    },
    /// Propose a named mutation intent.
    MutationIntent {
        /// Stable public catalog label.
        stable_label: String,
        /// Mutation kind label.
        mutation_kind: String,
    },
    /// Access a workspace scope.
    WorkspaceAccess {
        /// Stable public catalog label.
        stable_label: String,
        /// Safe workspace scope label.
        scope_label: String,
        /// Access mode requested by this capability.
        mode: SubagentWorkspaceAccessMode,
    },
    /// Use a named model profile.
    ModelUse {
        /// Stable public catalog label.
        stable_label: String,
        /// Model profile label.
        model_label: String,
    },
    /// Use a host-admitted external service.
    ExternalService {
        /// Stable public catalog label.
        stable_label: String,
        /// Injected service label.
        service_label: String,
    },
}

impl TypedSubagentCapability {
    /// Returns the capability class.
    #[must_use]
    pub const fn class(&self) -> SubagentCapabilityClass {
        match self {
            Self::ContextRead { .. } => SubagentCapabilityClass::ContextRead,
            Self::Search { .. } => SubagentCapabilityClass::Search,
            Self::ToolUse { .. } => SubagentCapabilityClass::ToolUse,
            Self::MutationIntent { .. } => SubagentCapabilityClass::MutationIntent,
            Self::WorkspaceAccess { .. } => SubagentCapabilityClass::WorkspaceAccess,
            Self::ModelUse { .. } => SubagentCapabilityClass::ModelUse,
            Self::ExternalService { .. } => SubagentCapabilityClass::ExternalService,
        }
    }

    /// Returns the stable public label preserved at adapter and receipt boundaries.
    #[must_use]
    pub fn stable_label(&self) -> &str {
        match self {
            Self::ContextRead { stable_label, .. }
            | Self::Search { stable_label, .. }
            | Self::ToolUse { stable_label, .. }
            | Self::MutationIntent { stable_label, .. }
            | Self::WorkspaceAccess { stable_label, .. }
            | Self::ModelUse { stable_label, .. }
            | Self::ExternalService { stable_label, .. } => stable_label.as_str(),
        }
    }

    fn identity_key(&self) -> String {
        match self {
            Self::ContextRead { context_label, .. } => format!("context:{context_label}"),
            Self::Search { index_label, .. } => format!("search:{index_label}"),
            Self::ToolUse { tool_label, .. } => format!("tool:{tool_label}"),
            Self::MutationIntent { mutation_kind, .. } => format!("mutation:{mutation_kind}"),
            Self::WorkspaceAccess {
                scope_label, mode, ..
            } => format!("workspace:{mode:?}:{scope_label}"),
            Self::ModelUse { model_label, .. } => format!("model:{model_label}"),
            Self::ExternalService { service_label, .. } => format!("external:{service_label}"),
        }
    }
}

/// Catalog entry that maps a stable adapter string to a typed capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityCatalogEntry {
    /// Stable boundary label.
    pub stable_label: String,
    /// Typed capability class.
    pub class: SubagentCapabilityClass,
    /// Class-specific safe target label.
    pub target_label: String,
    /// Workspace access mode when `class` is workspace access.
    pub workspace_mode: Option<SubagentWorkspaceAccessMode>,
    /// Whether the catalog authorizes the target for runtime use.
    pub authorized: bool,
}

/// Catalog conversion result for stable labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityConversion {
    /// Converted typed capabilities in request order.
    pub capabilities: Vec<TypedSubagentCapability>,
}

/// Denial source for typed capability intersection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentCapabilityCeilingSource {
    /// Parent policy did not admit the request.
    ParentPolicy,
    /// Recipe or profile ceiling did not admit the request.
    RecipeOrProfile,
    /// Semantic agent-role ceiling did not admit the request.
    SemanticRole,
    /// Workspace scope did not admit the request.
    WorkspaceScope,
    /// Host admission did not admit the request.
    HostAdmission,
}

/// Per-capability denial emitted by typed intersection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityDenial {
    /// Denied typed capability.
    pub capability: TypedSubagentCapability,
    /// Ceiling that denied the capability.
    pub source: SubagentCapabilityCeilingSource,
}

/// Typed capability intersection input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityResolutionInput {
    /// Capabilities allowed by parent policy.
    pub parent_policy: Vec<TypedSubagentCapability>,
    /// Capabilities allowed by recipe or profile ceilings.
    pub recipe_or_profile: Vec<TypedSubagentCapability>,
    /// Capabilities requested by the child run.
    pub request: Vec<TypedSubagentCapability>,
    /// Capabilities allowed by the independently admitted semantic role.
    pub semantic_role: Vec<TypedSubagentCapability>,
    /// Capabilities allowed by workspace scope.
    pub workspace_scope: Vec<TypedSubagentCapability>,
    /// Capabilities admitted by host policy.
    pub host_admission: Vec<TypedSubagentCapability>,
}

/// Effective typed subagent capability resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityResolution {
    /// Capabilities admitted by every ceiling.
    pub effective: Vec<TypedSubagentCapability>,
    /// Per-ceiling denial diagnostics.
    pub denied: Vec<SubagentCapabilityDenial>,
}

/// Converts stable capability labels through a checked catalog.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the catalog or requested labels are
/// unknown, duplicated, overbroad, unauthorized, unsupported, or colliding.
pub fn convert_subagent_capability_labels(
    catalog: &[SubagentCapabilityCatalogEntry],
    labels: &[&str],
) -> Result<SubagentCapabilityConversion, SubagentDiagnostic> {
    let catalog_by_label = validate_capability_catalog(catalog)?;
    ensure_non_empty(labels, FIELD_CAPABILITY)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut capabilities = Vec::with_capacity(labels.len());
    for label in labels {
        let normalized = normalize_label!(FIELD_CAPABILITY, label)?;
        reject_wildcard_label(normalized.as_str())?;
        if !seen.insert(normalized.clone()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                FIELD_CAPABILITY,
                "requested capability labels must be unique after normalization",
            ));
        }
        let entry = catalog_by_label.get(normalized.as_str()).ok_or_else(|| {
            SubagentDiagnostic::new(
                SubagentDiagnosticKind::UnknownLabel,
                FIELD_CAPABILITY,
                "capability label is not present in the checked catalog",
            )
        })?;
        capabilities.push(capability_from_entry(entry)?);
    }
    debug_assert_eq!(capabilities.len(), seen.len());
    debug_assert!(capabilities.len() <= labels.len());
    Ok(SubagentCapabilityConversion { capabilities })
}

/// Resolves effective capabilities by intersecting every typed ceiling.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when any input set is empty or contains
/// duplicate typed capability identities.
pub fn resolve_typed_subagent_capabilities(
    input: &SubagentCapabilityResolutionInput,
) -> Result<SubagentCapabilityResolution, SubagentDiagnostic> {
    ensure_unique_capabilities(&input.parent_policy, "parent_policy")?;
    ensure_unique_capabilities(&input.recipe_or_profile, "recipe_or_profile")?;
    ensure_unique_capabilities(&input.request, "request")?;
    ensure_unique_capabilities(&input.semantic_role, "semantic_role")?;
    ensure_unique_capabilities(&input.workspace_scope, "workspace_scope")?;
    ensure_unique_capabilities(&input.host_admission, "host_admission")?;
    let denial_slots = input.request.len().saturating_mul(CAPABILITY_CEILING_COUNT);
    let mut effective = Vec::with_capacity(input.request.len());
    let mut denied = Vec::with_capacity(denial_slots);
    for capability in &input.request {
        let mut is_denied_here = false;
        for source in missing_capability_sources(input, capability) {
            is_denied_here = true;
            denied.push(SubagentCapabilityDenial {
                capability: capability.clone(),
                source,
            });
        }
        if !is_denied_here {
            effective.push(capability.clone());
        }
    }
    debug_assert!(effective.len() <= input.request.len());
    debug_assert!(denied.len() <= denial_slots);
    Ok(SubagentCapabilityResolution { effective, denied })
}

/// Boundary-compatible receipt label for a typed capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCapabilityReceiptLabel {
    /// Stable public label.
    pub stable_label: String,
    /// Safe capability class.
    pub class: SubagentCapabilityClass,
    /// BLAKE3 identity of the typed capability.
    pub capability_hash: String,
}

/// Builds a safe receipt label for a typed capability.
#[must_use]
pub fn subagent_capability_receipt_label(
    capability: &TypedSubagentCapability,
) -> SubagentCapabilityReceiptLabel {
    SubagentCapabilityReceiptLabel {
        stable_label: capability.stable_label().to_owned(),
        class: capability.class(),
        capability_hash: hash_parts(SAFE_HASH_DOMAIN_CAPABILITY, &[&capability.identity_key()]),
    }
}

/// Returns the ordered, domain-separated BLAKE3 identity of a capability set.
#[must_use]
pub fn subagent_capability_set_identity(capabilities: &[TypedSubagentCapability]) -> String {
    let labels = capabilities
        .iter()
        .map(TypedSubagentCapability::stable_label)
        .collect::<Vec<_>>();
    hash_parts(SAFE_HASH_DOMAIN_CAPABILITY, labels.as_slice())
}

/// Shared subagent context item class.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubagentContextItemKind {
    /// Selected parent excerpt that was explicitly admitted.
    SelectedParentExcerpt,
    /// Safe summary.
    Summary,
    /// File observation summary or content hash.
    FileObservation,
    /// Memory descriptor without hidden body material.
    MemoryDescriptor,
    /// Bounded tool-result summary.
    ToolResultSummary,
    /// Admitted context-reference descriptor.
    AdmittedContextReference,
    /// Full parent transcript; unsafe by default.
    FullTranscript,
    /// Hidden memory body; unsafe by default.
    HiddenMemory,
    /// Credential-like material; unsafe by default.
    Credential,
    /// Provider-native payload; unsafe by default.
    ProviderPayload,
    /// Unbounded tool output; unsafe by default.
    UnboundedToolOutput,
}

impl SubagentContextItemKind {
    const fn is_safe_by_default(self) -> bool {
        matches!(
            self,
            Self::SelectedParentExcerpt
                | Self::Summary
                | Self::FileObservation
                | Self::MemoryDescriptor
                | Self::ToolResultSummary
                | Self::AdmittedContextReference
        )
    }
}

/// Context item redaction posture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentContextRedaction {
    /// Safe summary text is present.
    SafeSummary,
    /// Only a hash is present.
    HashOnly,
    /// Only an external safe reference label is present.
    ReferenceOnly,
    /// Raw material is present and must be admitted and leak-checked.
    Raw,
}

/// Context admission and budget policy for one child packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentContextAdmissionPolicy {
    /// Maximum item count.
    pub max_items: u16,
    /// Maximum estimated safe-token budget.
    pub max_tokens: u32,
    /// Whether unsafe context classes are admitted after leak validation.
    pub admit_unsafe_classes: bool,
}

impl Default for SubagentContextAdmissionPolicy {
    fn default() -> Self {
        Self {
            max_items: MAX_PACKET_ITEMS,
            max_tokens: MAX_PACKET_TOKENS,
            admit_unsafe_classes: false,
        }
    }
}

/// Draft context item supplied before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentContextItemDraft {
    /// Context class.
    pub kind: SubagentContextItemKind,
    /// Stable safe item label.
    pub label: String,
    /// Safe summary, hash, or reference text according to the redaction posture.
    pub safe_body: String,
    /// Estimated safe-token budget charged for this item.
    pub token_estimate: u32,
    /// Redaction posture.
    pub redaction: SubagentContextRedaction,
}

/// Draft shared context packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentContextPacketDraft {
    /// Safe packet id supplied by the parent runtime.
    pub packet_id: String,
    /// Child-run kind that will receive this packet.
    pub child_kind: SubagentChildRunKind,
    /// Admission and budget policy.
    pub admission: SubagentContextAdmissionPolicy,
    /// Context items.
    pub items: Vec<SubagentContextItemDraft>,
}

/// Budget summary safe for receipts and ledgers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentContextBudgetSummary {
    /// Count of admitted items.
    pub item_count: u16,
    /// Sum of estimated safe tokens.
    pub token_estimate: u32,
}

/// Validated shared context packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentContextPacket {
    /// Safe packet id.
    pub packet_id: String,
    /// Child-run kind that receives this packet.
    pub child_kind: SubagentChildRunKind,
    /// Deterministic BLAKE3 identity over safe packet fields.
    pub packet_hash: String,
    /// Safe budget summary.
    pub budget: SubagentContextBudgetSummary,
    /// Validated items.
    pub items: Vec<SubagentContextItemDraft>,
}

/// Validates a shared minimized subagent context packet.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the packet is missing an identity, empty,
/// over budget, duplicated, lossy, or contains unadmitted unsafe material.
pub fn validate_subagent_context_packet(
    draft: &SubagentContextPacketDraft,
) -> Result<SubagentContextPacket, SubagentDiagnostic> {
    let packet_id = validate_label!(FIELD_CONTEXT, draft.packet_id.as_str())?;
    ensure_non_empty(draft.items.as_slice(), FIELD_CONTEXT)?;
    if draft.items.len() > usize::from(draft.admission.max_items) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_CONTEXT,
            "context packet item count exceeds policy",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut token_estimate = 0_u32;
    let safe_part_slots = draft
        .items
        .len()
        .saturating_add(CONTEXT_PACKET_HASH_PREFIX_PARTS);
    let mut safe_parts = Vec::with_capacity(safe_part_slots);
    safe_parts.push(packet_id.as_str());
    safe_parts.push(child_kind_label(draft.child_kind));
    for item in &draft.items {
        let label = validate_label!(FIELD_CONTEXT, item.label.as_str())?;
        if !seen.insert(label.clone()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                FIELD_CONTEXT,
                "context item labels must be unique within a packet",
            ));
        }
        validate_context_item(draft.admission, item)?;
        token_estimate = token_estimate
            .checked_add(item.token_estimate)
            .ok_or_else(|| over_budget("context token estimate overflowed policy budget"))?;
        if token_estimate > draft.admission.max_tokens {
            return Err(over_budget("context packet exceeds token budget"));
        }
        safe_parts.push(item.safe_body.as_str());
    }
    let item_count = u16::try_from(draft.items.len()).map_err(|_| {
        SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_CONTEXT,
            "context packet item count exceeds representable budget summary",
        )
    })?;
    let packet_hash = hash_parts(SAFE_HASH_DOMAIN_CONTEXT, &safe_parts);
    debug_assert_eq!(usize::from(item_count), draft.items.len());
    debug_assert!(token_estimate <= draft.admission.max_tokens);
    Ok(SubagentContextPacket {
        packet_id,
        child_kind: draft.child_kind,
        packet_hash,
        budget: SubagentContextBudgetSummary {
            item_count,
            token_estimate,
        },
        items: draft.items.clone(),
    })
}

/// Projects a validated packet for a child-run kind without losing packet identity.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the requested kind does not match the
/// validated packet kind or the requested kind is unsupported.
pub fn project_context_packet_for_child_kind(
    packet: &SubagentContextPacket,
    kind: SubagentChildRunKind,
) -> Result<SubagentContextPacket, SubagentDiagnostic> {
    if matches!(kind, SubagentChildRunKind::Unsupported) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsupportedKind,
            FIELD_CONTEXT,
            "unsupported child-run kind cannot receive a context projection",
        ));
    }
    if packet.child_kind != kind {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::LossyProjection,
            FIELD_CONTEXT,
            "context projection must preserve the source child-run kind",
        ));
    }
    Ok(packet.clone())
}

/// Supported child-run kinds for the unified facade.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubagentChildRunKind {
    /// Recipe-backed child run.
    RecipeBacked,
    /// Fresh-context investigation.
    FreshContextInvestigation,
    /// Read-only side question.
    ReadOnlySideQuestion,
    /// Worker-team task.
    WorkerTeamTask,
    /// Unsupported placeholder used to prove fail-closed validation.
    Unsupported,
}

/// Handoff mode used for parent-visible child outputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentHandoffMode {
    /// Parent receives only the safe summary.
    SummaryOnly,
    /// Parent receives status and safe summary.
    StatusPlusSummary,
    /// Parent may request bounded detail after policy admits it.
    FullDetailBounded,
}

/// Recipe-backed child-run fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecipeSubagentDraft {
    /// Recipe identity label.
    pub recipe_id: String,
    /// Trust receipt proving the recipe was admitted.
    pub trust_receipt_label: String,
    /// Parent handoff mode.
    pub handoff_mode: SubagentHandoffMode,
}

/// Fresh-context or side-question fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FreshContextSubagentDraft {
    /// Safe child objective.
    pub objective: String,
    /// Injected scheduler or service label.
    pub scheduler_label: String,
    /// Whether this child kind is restricted to read-only work.
    pub read_only: bool,
}

/// Worker-team child-run fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerTeamSubagentDraft {
    /// Injected worker service label.
    pub service_label: String,
    /// Expected output schema label.
    pub output_schema_label: String,
}

/// Unified child-run draft supplied before facade validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentChildRunDraft {
    /// Safe child identity.
    pub child_id: String,
    /// Safe parent run identity.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Shared context packet.
    pub context_packet: Option<SubagentContextPacketDraft>,
    /// Effective capabilities requested by the child.
    pub capabilities: Vec<TypedSubagentCapability>,
    /// Recipe-backed kind-specific fields.
    pub recipe: Option<RecipeSubagentDraft>,
    /// Fresh-context or side-question kind-specific fields.
    pub fresh_context: Option<FreshContextSubagentDraft>,
    /// Worker-team kind-specific fields.
    pub worker_team: Option<WorkerTeamSubagentDraft>,
    /// Whether the caller tried to execute while validating.
    pub execution_requested: bool,
}

/// Validated child-run facade object.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentChildRun {
    /// Safe child identity.
    pub child_id: String,
    /// Safe parent identity.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Validated shared context packet.
    pub context_packet: SubagentContextPacket,
    /// Validated capabilities.
    pub capabilities: Vec<TypedSubagentCapability>,
    /// Kind-specific safe metadata label.
    pub kind_metadata_label: String,
}

/// Validated admission used by coordinator and fan-out plans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentAdmission {
    /// Safe child identity.
    pub child_id: String,
    /// Safe parent identity.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// BLAKE3 admission identity.
    pub admission_identity: String,
    /// BLAKE3 identity of effective capability labels.
    pub capability_identity: String,
    /// BLAKE3 context packet identity.
    pub context_packet_identity: String,
    /// Safe context budget summary.
    pub budget: SubagentContextBudgetSummary,
    /// Safe receipt hash for the admission.
    pub receipt_hash: String,
}

/// Validates a child-run through the unified facade without execution effects.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] for unsupported kinds, missing kind-specific
/// fields, lossy projection, invalid lower-level data, or execution attempts.
pub fn validate_subagent_child_run(
    draft: &SubagentChildRunDraft,
) -> Result<SubagentChildRun, SubagentDiagnostic> {
    if draft.execution_requested {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::ExecutionAuthorityRequired,
            FIELD_CHILD,
            "facade validation must not execute child runs",
        ));
    }
    let child_id = validate_label!(FIELD_CHILD, draft.child_id.as_str())?;
    let parent_id = validate_label!(FIELD_CHILD, draft.parent_id.as_str())?;
    if matches!(draft.kind, SubagentChildRunKind::Unsupported) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsupportedKind,
            FIELD_CHILD,
            "unsupported child-run kind has no registered typed validator",
        ));
    }
    ensure_unique_capabilities(&draft.capabilities, FIELD_CAPABILITY)?;
    let packet_draft = draft.context_packet.as_ref().ok_or_else(|| {
        SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingField,
            FIELD_CONTEXT,
            "child-run facade requires a shared context packet",
        )
    })?;
    let packet = validate_subagent_context_packet(packet_draft)?;
    let packet = project_context_packet_for_child_kind(&packet, draft.kind)?;
    let kind_metadata_label = validate_kind_fields(draft)?;
    debug_assert_eq!(packet.child_kind, draft.kind);
    debug_assert!(!kind_metadata_label.is_empty());
    Ok(SubagentChildRun {
        child_id,
        parent_id,
        kind: draft.kind,
        context_packet: packet,
        capabilities: draft.capabilities.clone(),
        kind_metadata_label,
    })
}

/// Converts a validated child run into an admitted coordinator DTO.
#[must_use]
pub fn admit_subagent_child_run(child: &SubagentChildRun) -> SubagentAdmission {
    let capability_identity = subagent_capability_set_identity(child.capabilities.as_slice());
    let admission_parts = [
        child.parent_id.as_str(),
        child.child_id.as_str(),
        child_kind_label(child.kind),
        child.context_packet.packet_hash.as_str(),
        capability_identity.as_str(),
        child.kind_metadata_label.as_str(),
    ];
    let admission_identity = hash_parts(SAFE_HASH_DOMAIN_ADMISSION, &admission_parts);
    let receipt_hash = hash_parts(
        SAFE_HASH_DOMAIN_RECEIPT,
        &[
            admission_identity.as_str(),
            child.context_packet.packet_hash.as_str(),
        ],
    );
    debug_assert!(!child.context_packet.packet_hash.is_empty());
    SubagentAdmission {
        child_id: child.child_id.clone(),
        parent_id: child.parent_id.clone(),
        kind: child.kind,
        admission_identity,
        capability_identity,
        context_packet_identity: child.context_packet.packet_hash.clone(),
        budget: child.context_packet.budget,
        receipt_hash,
    }
}

/// Runtime coordinator state class for a child run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentRunStatus {
    /// Validated but not enqueued.
    Admitted,
    /// Enqueued with an injected service.
    Queued,
    /// Running according to injected service status.
    Running,
    /// Cancellation accepted and in progress.
    Cancelling,
    /// Completed successfully.
    Completed,
    /// Failed with a typed failure class.
    Failed,
    /// Timed out.
    TimedOut,
    /// Cancelled.
    Cancelled,
    /// Terminal report collected.
    Collected,
    /// Blocked by policy or missing data.
    Blocked,
    /// Awaiting parent arbitration or acknowledgement.
    AwaitingParent,
}

impl SubagentRunStatus {
    const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Queued | Self::Running | Self::Cancelling | Self::AwaitingParent
        )
    }

    const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::TimedOut | Self::Cancelled | Self::Collected
        )
    }
}

/// Generation-bound values for one concrete subagent handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentHandleBinding<'a> {
    /// Safe parent identity.
    pub parent_id: &'a str,
    /// Safe child identity.
    pub child_id: &'a str,
    /// Positive registry generation.
    pub generation: u64,
    /// Current parent-authority identity.
    pub authority_identity: &'a str,
}

/// Opaque service handle for a child run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentHandle {
    /// Safe host-issued handle label.
    pub handle_id: String,
    parent_id: Option<String>,
    child_id: Option<String>,
    generation: Option<u64>,
    authority_identity: Option<String>,
}

impl SubagentHandle {
    /// Validates and builds a legacy unbound subagent handle.
    ///
    /// Concrete child-session hosts reject this compatibility shape because it
    /// has no parent, child, generation, or authority fence.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentDiagnostic`] when the handle is empty, unsafe, or too long.
    pub fn new(handle_id: impl AsRef<str>) -> Result<Self, SubagentDiagnostic> {
        Ok(Self {
            handle_id: validate_label!(FIELD_SERVICE, handle_id.as_ref())?,
            parent_id: None,
            child_id: None,
            generation: None,
            authority_identity: None,
        })
    }

    /// Builds a parent-, child-, generation-, and authority-bound handle.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentDiagnostic`] for unsafe labels, a zero generation, or
    /// a malformed authority identity.
    pub fn generation_bound(
        handle_id: impl AsRef<str>,
        binding: SubagentHandleBinding<'_>,
    ) -> Result<Self, SubagentDiagnostic> {
        if binding.generation == 0 {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::StaleHandle,
                FIELD_SERVICE,
                "generation-bound handles require a positive generation",
            ));
        }
        validate_hash(binding.authority_identity)?;
        Ok(Self {
            handle_id: validate_label!(FIELD_SERVICE, handle_id.as_ref())?,
            parent_id: Some(validate_label!(FIELD_SERVICE, binding.parent_id)?),
            child_id: Some(validate_label!(FIELD_SERVICE, binding.child_id)?),
            generation: Some(binding.generation),
            authority_identity: Some(binding.authority_identity.to_owned()),
        })
    }

    /// Returns the bound parent identity when present.
    #[must_use]
    pub fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    /// Returns the bound child identity when present.
    #[must_use]
    pub fn child_id(&self) -> Option<&str> {
        self.child_id.as_deref()
    }

    /// Returns the bound registry generation when present.
    #[must_use]
    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }

    /// Returns the bound current-authority identity when present.
    #[must_use]
    pub fn authority_identity(&self) -> Option<&str> {
        self.authority_identity.as_deref()
    }
}

/// Safe child mutation-intent summary.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SubagentMutationIntentSummary {
    /// Safe mutation target label.
    pub target: String,
    /// Safe mutation kind label.
    pub mutation_kind: String,
    /// Evidence reference label.
    pub evidence_ref: String,
}

/// Safe child report collected from an injected service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentReport {
    /// Child id the report belongs to.
    pub child_id: String,
    /// Terminal status reported by the service.
    pub status: SubagentRunStatus,
    /// BLAKE3 hash of the safe child output summary.
    pub output_hash: Option<String>,
    /// Safe evidence references.
    pub evidence_refs: Vec<String>,
    /// Pending mutation intents; aggregation never applies them.
    pub mutation_intents: Vec<SubagentMutationIntentSummary>,
}

/// Failure emitted by an injected subagent service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentServiceFailure {
    /// Failure kind.
    pub kind: SubagentServiceFailureKind,
    /// Safe diagnostic.
    pub diagnostic: SubagentDiagnostic,
}

/// Injected service failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentServiceFailureKind {
    /// Service unavailable.
    Unavailable,
    /// Service denied the action.
    Denied,
    /// Service timed out.
    TimedOut,
    /// Service cancelled.
    Cancelled,
    /// Service returned a malformed report.
    MalformedReport,
    /// Handle is stale or unknown.
    StaleHandle,
}

/// Host-injected child-run service.
pub trait SubagentService {
    /// Enqueues a validated child-run admission.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentServiceFailure`] when the service is unavailable, denied,
    /// timed out, cancelled, or cannot provide a safe handle.
    fn enqueue(
        &mut self,
        admission: &SubagentAdmission,
    ) -> Result<SubagentHandle, SubagentServiceFailure>;

    /// Reads child-run status.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentServiceFailure`] when status lookup fails safely.
    fn status(
        &mut self,
        handle: &SubagentHandle,
    ) -> Result<SubagentRunStatus, SubagentServiceFailure>;

    /// Requests child-run cancellation.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentServiceFailure`] when cancellation fails safely.
    fn cancel(
        &mut self,
        handle: &SubagentHandle,
    ) -> Result<SubagentRunStatus, SubagentServiceFailure>;

    /// Collects a terminal child report.
    ///
    /// # Errors
    ///
    /// Returns [`SubagentServiceFailure`] when collection fails safely.
    fn collect(
        &mut self,
        handle: &SubagentHandle,
    ) -> Result<SubagentReport, SubagentServiceFailure>;
}

/// Coordinator state for one child run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentRunState {
    /// Validated admission.
    pub admission: SubagentAdmission,
    /// Current status.
    pub status: SubagentRunStatus,
    /// Optional injected service handle.
    pub handle: Option<SubagentHandle>,
    /// Optional collected terminal report.
    pub report: Option<SubagentReport>,
    /// Last safe failure class.
    pub failure_class: Option<SubagentServiceFailureKind>,
}

impl SubagentRunState {
    /// Creates an admitted state.
    #[must_use]
    pub const fn admitted(admission: SubagentAdmission) -> Self {
        Self {
            admission,
            status: SubagentRunStatus::Admitted,
            handle: None,
            report: None,
            failure_class: None,
        }
    }
}

/// Pure coordinator state transition event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubagentCoordinatorEvent {
    /// Service returned an enqueue handle.
    Enqueued(SubagentHandle),
    /// Service returned a status class.
    Status(SubagentRunStatus),
    /// Service accepted or completed cancellation.
    Cancel(SubagentRunStatus),
    /// Service returned a terminal report.
    Collect(SubagentReport),
    /// Service failed safely.
    Fail(SubagentServiceFailureKind),
}

/// Pure subagent coordinator state transition.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the transition is stale, malformed, or
/// would claim success without a terminal report.
pub fn transition_subagent_state(
    state: &SubagentRunState,
    event: SubagentCoordinatorEvent,
) -> Result<SubagentRunState, SubagentDiagnostic> {
    let mut next = state.clone();
    match event {
        SubagentCoordinatorEvent::Enqueued(handle) => {
            require_status(state.status, SubagentRunStatus::Admitted)?;
            next.status = SubagentRunStatus::Queued;
            next.handle = Some(handle);
        }
        SubagentCoordinatorEvent::Status(status) => {
            if state.handle.is_none() {
                return Err(stale_handle("status lookup requires an active handle"));
            }
            if status.is_terminal()
                && !matches!(
                    status,
                    SubagentRunStatus::Cancelled | SubagentRunStatus::TimedOut
                )
            {
                return Err(SubagentDiagnostic::new(
                    SubagentDiagnosticKind::MalformedReport,
                    FIELD_REPORT,
                    "terminal success requires collect with a validated report",
                ));
            }
            next.status = status;
        }
        SubagentCoordinatorEvent::Cancel(status) => {
            if state.handle.is_none() || !state.status.is_active() {
                return Err(stale_handle("cancellation requires an active handle"));
            }
            next.status = status;
        }
        SubagentCoordinatorEvent::Collect(report) => {
            if state.handle.is_none() {
                return Err(stale_handle("collect requires an active handle"));
            }
            validate_subagent_report(&state.admission, &report)?;
            next.status = SubagentRunStatus::Collected;
            next.report = Some(report);
        }
        SubagentCoordinatorEvent::Fail(kind) => {
            next.status = status_for_failure(kind);
            next.failure_class = Some(kind);
        }
    }
    debug_assert_eq!(next.admission.child_id, state.admission.child_id);
    debug_assert!(next.status.is_terminal() || next.report.is_none());
    Ok(next)
}

/// Enqueues a child run through an injected service.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the service is missing or enqueueing fails.
pub fn subagent_enqueue(
    service: Option<&mut dyn SubagentService>,
    state: &SubagentRunState,
) -> Result<SubagentRunState, SubagentDiagnostic> {
    let service = injected_subagent_service(service)?;
    let handle = service
        .enqueue(&state.admission)
        .map_err(service_failure_to_diagnostic)?;
    transition_subagent_state(state, SubagentCoordinatorEvent::Enqueued(handle))
}

/// Requests status through an injected service.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the service is missing, the handle is
/// stale, or status lookup fails.
pub fn subagent_status(
    service: Option<&mut dyn SubagentService>,
    state: &SubagentRunState,
) -> Result<SubagentRunState, SubagentDiagnostic> {
    let handle = state
        .handle
        .as_ref()
        .ok_or_else(|| stale_handle("status lookup requires an active handle"))?;
    let service = injected_subagent_service(service)?;
    let status = service
        .status(handle)
        .map_err(service_failure_to_diagnostic)?;
    transition_subagent_state(state, SubagentCoordinatorEvent::Status(status))
}

/// Requests cancellation through an injected service.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the service is missing, stale, denied, or fails safely.
pub fn subagent_cancel(
    service: Option<&mut dyn SubagentService>,
    state: &SubagentRunState,
) -> Result<SubagentRunState, SubagentDiagnostic> {
    let handle = state
        .handle
        .as_ref()
        .ok_or_else(|| stale_handle("cancel requires an active handle"))?;
    let service = injected_subagent_service(service)?;
    let status = service
        .cancel(handle)
        .map_err(service_failure_to_diagnostic)?;
    transition_subagent_state(state, SubagentCoordinatorEvent::Cancel(status))
}

/// Collects a child report through an injected service.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the service is missing, stale, denied, or returns malformed data.
pub fn subagent_collect(
    service: Option<&mut dyn SubagentService>,
    state: &SubagentRunState,
) -> Result<SubagentRunState, SubagentDiagnostic> {
    let handle = state
        .handle
        .as_ref()
        .ok_or_else(|| stale_handle("collect requires an active handle"))?;
    let service = injected_subagent_service(service)?;
    let collected_child = service
        .collect(handle)
        .map_err(service_failure_to_diagnostic)?;
    transition_subagent_state(state, SubagentCoordinatorEvent::Collect(collected_child))
}

/// Safe lifecycle event projection for child-run states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentLifecycleEvent {
    /// Child id.
    pub child_id: String,
    /// Parent id.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Status class.
    pub status: SubagentRunStatus,
    /// Failure class when present.
    pub failure_class: Option<SubagentServiceFailureKind>,
    /// Admission identity.
    pub admission_identity: String,
    /// Capability identity.
    pub capability_identity: String,
    /// Context packet identity.
    pub context_packet_identity: String,
    /// Budget summary.
    pub budget: SubagentContextBudgetSummary,
    /// Safe evidence references.
    pub evidence_refs: Vec<String>,
}

/// Projects a coordinator state into a safe lifecycle event.
#[must_use]
pub fn project_subagent_lifecycle_event(state: &SubagentRunState) -> SubagentLifecycleEvent {
    SubagentLifecycleEvent {
        child_id: state.admission.child_id.clone(),
        parent_id: state.admission.parent_id.clone(),
        kind: state.admission.kind,
        status: state.status,
        failure_class: state.failure_class,
        admission_identity: state.admission.admission_identity.clone(),
        capability_identity: state.admission.capability_identity.clone(),
        context_packet_identity: state.admission.context_packet_identity.clone(),
        budget: state.admission.budget,
        evidence_refs: state
            .report
            .as_ref()
            .map_or_else(Vec::new, |report| report.evidence_refs.clone()),
    }
}

/// Shared budget policy for fan-out plans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentFanOutBudget {
    /// Maximum children allowed in the plan.
    pub max_children: u16,
    /// Shared token or context budget.
    pub max_tokens: u32,
}

/// Fan-out cancellation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentCancellationPolicy {
    /// Cancel remaining children after the first terminal failure.
    CancelRemainingOnFailure,
    /// Continue independent read-only children after nonfatal failures.
    ContinueReadOnly,
}

/// Mutation handling policy for fan-in aggregation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentMutationPolicy {
    /// Deny all child mutation intents.
    DenyAll,
    /// Preserve mutation intents as unresolved parent work.
    CollectUnresolved,
    /// Allow non-conflicting intents to remain pending for later policy.
    AllowNonConflicting,
}

/// Fan-out plan draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentFanOutPlanDraft {
    /// Safe fan-out plan id.
    pub plan_id: String,
    /// Child admissions to coordinate.
    pub children: Vec<SubagentAdmission>,
    /// Maximum concurrent children.
    pub max_concurrency: u16,
    /// Shared budget.
    pub budget: SubagentFanOutBudget,
    /// Cancellation policy.
    pub cancellation_policy: SubagentCancellationPolicy,
    /// Aggregation schema label.
    pub aggregation_schema: String,
    /// Mutation arbitration policy.
    pub mutation_policy: SubagentMutationPolicy,
}

/// Validated fan-out plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentFanOutPlan {
    /// Safe fan-out plan id.
    pub plan_id: String,
    /// Child admissions to coordinate.
    pub children: Vec<SubagentAdmission>,
    /// Maximum concurrent children.
    pub max_concurrency: u16,
    /// Shared budget.
    pub budget: SubagentFanOutBudget,
    /// Cancellation policy.
    pub cancellation_policy: SubagentCancellationPolicy,
    /// Aggregation schema label.
    pub aggregation_schema: String,
    /// Mutation arbitration policy.
    pub mutation_policy: SubagentMutationPolicy,
}

/// Validates a fan-out plan without executing children.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when admissions are missing, duplicated, unsafe,
/// over budget, or concurrency is not bounded.
pub fn validate_subagent_fanout_plan(
    draft: &SubagentFanOutPlanDraft,
) -> Result<SubagentFanOutPlan, SubagentDiagnostic> {
    let plan_id = validate_label!(FIELD_PLAN, draft.plan_id.as_str())?;
    let aggregation_schema = validate_label!(FIELD_PLAN, draft.aggregation_schema.as_str())?;
    ensure_non_empty(draft.children.as_slice(), FIELD_PLAN)?;
    if draft.children.len() > usize::from(draft.budget.max_children)
        || draft.children.len() > usize::from(MAX_FANOUT_CHILDREN)
    {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_PLAN,
            "fan-out plan exceeds child budget",
        ));
    }
    if draft.max_concurrency < MIN_POSITIVE_U16
        || draft.max_concurrency > MAX_FANOUT_CONCURRENCY
        || usize::from(draft.max_concurrency) > draft.children.len()
    {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_PLAN,
            "fan-out concurrency must be positive, bounded, and no larger than child count",
        ));
    }
    if draft.budget.max_tokens < MIN_POSITIVE_U32 {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_PLAN,
            "fan-out shared budget must be positive",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for child in &draft.children {
        if !seen.insert(child.child_id.clone()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateChild,
                FIELD_PLAN,
                "fan-out child ids must be unique",
            ));
        }
    }
    debug_assert_eq!(seen.len(), draft.children.len());
    debug_assert!(draft.max_concurrency <= MAX_FANOUT_CONCURRENCY);
    Ok(SubagentFanOutPlan {
        plan_id,
        children: draft.children.clone(),
        max_concurrency: draft.max_concurrency,
        budget: draft.budget,
        cancellation_policy: draft.cancellation_policy,
        aggregation_schema,
        mutation_policy: draft.mutation_policy,
    })
}

/// Child outcome supplied to fan-in aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentFanInOutcome {
    /// Child id.
    pub child_id: String,
    /// Terminal status.
    pub status: SubagentRunStatus,
    /// Optional validated report.
    pub report: Option<SubagentReport>,
    /// Budget used by the child.
    pub budget_used: u32,
    /// Mutation intents supplied by the child.
    pub mutation_intents: Vec<SubagentMutationIntentSummary>,
}

/// Aggregate fan-in status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentFanInStatus {
    /// Every child completed successfully and no conflicts remain.
    Success,
    /// At least one child succeeded and at least one failed, cancelled, or timed out.
    PartialSuccess,
    /// No child succeeded and at least one failed.
    Failed,
    /// Timeout is the dominant terminal class.
    TimedOut,
    /// Cancellation is the dominant terminal class.
    Cancelled,
    /// Shared budget exhaustion stopped the fan-out.
    BudgetExhausted,
    /// A malformed child report was present.
    Malformed,
    /// A child capability was denied.
    DeniedCapability,
    /// Mutation intents conflict and require parent arbitration.
    WriteConflict,
}

/// Mutation arbitration summary produced by fan-in aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentMutationArbitration {
    /// Mutation intents denied by policy.
    pub denied: Vec<SubagentMutationIntentSummary>,
    /// Mutation intents left unresolved for parent policy.
    pub unresolved: Vec<SubagentMutationIntentSummary>,
    /// Conflicting mutation intents.
    pub conflicts: Vec<SubagentMutationIntentSummary>,
}

/// Deterministic fan-in aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentFanInAggregate {
    /// Aggregate status.
    pub status: SubagentFanInStatus,
    /// Successful child ids.
    pub successful_children: Vec<String>,
    /// Non-success child ids.
    pub non_success_children: Vec<String>,
    /// Mutation arbitration summary.
    pub mutation_arbitration: SubagentMutationArbitration,
}

/// Aggregates child outcomes deterministically without applying effects.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] for missing, duplicate, malformed, over-budget,
/// or mutation-bypass outcomes.
pub fn aggregate_subagent_fan_in(
    plan: &SubagentFanOutPlan,
    outcomes: &[SubagentFanInOutcome],
) -> Result<SubagentFanInAggregate, SubagentDiagnostic> {
    let outcomes_by_child = indexed_outcomes(outcomes)?;
    let mutation_intent_slots = outcomes.iter().fold(0_usize, |slots, outcome| {
        slots.saturating_add(outcome.mutation_intents.len())
    });
    let mut successful_children = Vec::with_capacity(plan.children.len());
    let mut non_success_children = Vec::with_capacity(plan.children.len());
    let mut statuses = Vec::with_capacity(plan.children.len());
    let mut mutation_intents = Vec::with_capacity(mutation_intent_slots);
    let mut total_budget = 0_u32;
    for child in &plan.children {
        let outcome = outcomes_by_child
            .get(child.child_id.as_str())
            .ok_or_else(|| {
                SubagentDiagnostic::new(
                    SubagentDiagnosticKind::MissingChildOutcome,
                    FIELD_PLAN,
                    "fan-in aggregation requires one outcome per admitted child",
                )
            })?;
        total_budget = total_budget
            .checked_add(outcome.budget_used)
            .ok_or_else(|| over_budget("fan-in budget usage overflowed shared budget"))?;
        if total_budget > plan.budget.max_tokens {
            statuses.push(SubagentFanInStatus::BudgetExhausted);
        }
        validate_outcome_report(child, outcome)?;
        statuses.push(status_to_aggregate(outcome.status));
        if matches!(
            outcome.status,
            SubagentRunStatus::Completed | SubagentRunStatus::Collected
        ) {
            successful_children.push(child.child_id.clone());
        } else {
            non_success_children.push(child.child_id.clone());
        }
        mutation_intents.extend(outcome.mutation_intents.iter().cloned());
    }
    let mutation_arbitration = arbitrate_mutations(plan.mutation_policy, mutation_intents)?;
    let status = aggregate_status(
        &statuses,
        &mutation_arbitration,
        !successful_children.is_empty(),
    );
    debug_assert_eq!(
        successful_children
            .len()
            .saturating_add(non_success_children.len()),
        plan.children.len()
    );
    debug_assert!(statuses.len() >= plan.children.len());
    Ok(SubagentFanInAggregate {
        status,
        successful_children,
        non_success_children,
        mutation_arbitration,
    })
}

/// Draft safe subagent lineage record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentLineageRecordDraft {
    /// Parent run id.
    pub parent_id: String,
    /// Child run id.
    pub child_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Delegation source label.
    pub delegation_source: String,
    /// Admission identity hash.
    pub admission_identity: String,
    /// Effective capability identity hash.
    pub capability_identity: String,
    /// Context packet identity hash.
    pub context_packet_identity: String,
    /// Output summary hash.
    pub output_hash: String,
    /// Handoff mode.
    pub handoff_mode: SubagentHandoffMode,
    /// Terminal status.
    pub terminal_status: SubagentRunStatus,
    /// Failure class label when present.
    pub failure_class: Option<String>,
    /// Safe evidence references.
    pub evidence_refs: Vec<String>,
    /// Safe summary reference or bounded summary.
    pub safe_summary: String,
}

/// Validated safe subagent lineage record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentLineageRecord {
    /// Parent run id.
    pub parent_id: String,
    /// Child run id.
    pub child_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Delegation source label.
    pub delegation_source: String,
    /// Admission identity hash.
    pub admission_identity: String,
    /// Effective capability identity hash.
    pub capability_identity: String,
    /// Context packet identity hash.
    pub context_packet_identity: String,
    /// Output summary hash.
    pub output_hash: String,
    /// Handoff mode.
    pub handoff_mode: SubagentHandoffMode,
    /// Terminal status.
    pub terminal_status: SubagentRunStatus,
    /// Failure class label when present.
    pub failure_class: Option<String>,
    /// Safe evidence references.
    pub evidence_refs: Vec<String>,
    /// Safe summary reference or bounded summary.
    pub safe_summary: String,
    /// BLAKE3 lineage record hash.
    pub record_hash: String,
}

/// Validates a safe subagent lineage record.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when required identities are missing, hashes are
/// malformed, evidence is unbounded, or unsafe raw material is present.
pub fn validate_subagent_lineage_record(
    draft: &SubagentLineageRecordDraft,
) -> Result<SubagentLineageRecord, SubagentDiagnostic> {
    let parent_id = validate_label!(FIELD_LINEAGE, draft.parent_id.as_str())?;
    let child_id = validate_label!(FIELD_LINEAGE, draft.child_id.as_str())?;
    let delegation_source = validate_label!(FIELD_LINEAGE, draft.delegation_source.as_str())?;
    validate_hash(draft.admission_identity.as_str())?;
    validate_hash(draft.capability_identity.as_str())?;
    validate_hash(draft.context_packet_identity.as_str())?;
    validate_hash(draft.output_hash.as_str())?;
    if !draft.terminal_status.is_terminal() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MalformedReport,
            FIELD_LINEAGE,
            "lineage records require terminal child status",
        ));
    }
    validate_evidence_refs(draft.evidence_refs.as_slice(), FIELD_LINEAGE)?;
    let safe_summary = validate_safe_text!(FIELD_LINEAGE, draft.safe_summary.as_str())?;
    let failure_class = draft
        .failure_class
        .as_ref()
        .map(|value| validate_label!(FIELD_LINEAGE, value.as_str()))
        .transpose()?;
    let record_parts = [
        parent_id.as_str(),
        child_id.as_str(),
        child_kind_label(draft.kind),
        delegation_source.as_str(),
        draft.admission_identity.as_str(),
        draft.capability_identity.as_str(),
        draft.context_packet_identity.as_str(),
        draft.output_hash.as_str(),
        safe_summary.as_str(),
    ];
    let record_hash = hash_parts(SAFE_HASH_DOMAIN_LINEAGE, &record_parts);
    debug_assert!(draft.terminal_status.is_terminal());
    debug_assert_eq!(record_hash.len(), HASH_HEX_CHARS);
    Ok(SubagentLineageRecord {
        parent_id,
        child_id,
        kind: draft.kind,
        delegation_source,
        admission_identity: draft.admission_identity.clone(),
        capability_identity: draft.capability_identity.clone(),
        context_packet_identity: draft.context_packet_identity.clone(),
        output_hash: draft.output_hash.clone(),
        handoff_mode: draft.handoff_mode,
        terminal_status: draft.terminal_status,
        failure_class,
        evidence_refs: draft.evidence_refs.clone(),
        safe_summary,
        record_hash,
    })
}

/// Input for projecting a lifecycle event into a lineage draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubagentLineageProjection<'a> {
    /// Safe lifecycle event to project.
    pub event: &'a SubagentLifecycleEvent,
    /// Delegation source label.
    pub delegation_source: &'a str,
    /// Output summary hash.
    pub output_hash: &'a str,
    /// Accepted handoff mode.
    pub handoff_mode: SubagentHandoffMode,
    /// Safe summary reference or bounded summary.
    pub safe_summary: &'a str,
}

/// Builds a lineage draft from a safe lifecycle event.
#[must_use]
pub fn lineage_draft_from_lifecycle(
    input: SubagentLineageProjection<'_>,
) -> SubagentLineageRecordDraft {
    SubagentLineageRecordDraft {
        parent_id: input.event.parent_id.clone(),
        child_id: input.event.child_id.clone(),
        kind: input.event.kind,
        delegation_source: input.delegation_source.to_owned(),
        admission_identity: input.event.admission_identity.clone(),
        capability_identity: input.event.capability_identity.clone(),
        context_packet_identity: input.event.context_packet_identity.clone(),
        output_hash: input.output_hash.to_owned(),
        handoff_mode: input.handoff_mode,
        terminal_status: input.event.status,
        failure_class: input
            .event
            .failure_class
            .map(|failure| format!("{failure:?}")),
        evidence_refs: input.event.evidence_refs.clone(),
        safe_summary: input.safe_summary.to_owned(),
    }
}

/// Parent-visible child influence import for replay checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentInfluenceImport {
    /// Parent run id.
    pub parent_id: String,
    /// Child run id.
    pub child_id: String,
    /// Output hash imported by the parent.
    pub output_hash: String,
    /// Handoff mode accepted by the parent.
    pub handoff_mode: SubagentHandoffMode,
}

/// Validates parent-visible child influence against lineage records.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when an import lacks lineage or mismatches
/// parent, child, output hash, or handoff mode.
pub fn validate_subagent_replay_influence(
    records: &[SubagentLineageRecord],
    imports: &[SubagentInfluenceImport],
) -> Result<(), SubagentDiagnostic> {
    for import in imports {
        validate_hash(import.output_hash.as_str())?;
        let record = records
            .iter()
            .find(|record| {
                record.child_id == import.child_id && record.parent_id == import.parent_id
            })
            .ok_or_else(|| {
                SubagentDiagnostic::new(
                    SubagentDiagnosticKind::MissingLineage,
                    FIELD_LINEAGE,
                    "parent-visible child import has no matching lineage record",
                )
            })?;
        if record.output_hash != import.output_hash || record.handoff_mode != import.handoff_mode {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::LineageMismatch,
                FIELD_LINEAGE,
                "lineage record does not match imported child influence",
            ));
        }
    }
    debug_assert!(imports.iter().all(|import| records.iter().any(|record| {
        record.child_id == import.child_id && record.parent_id == import.parent_id
    })));
    debug_assert!(
        imports
            .iter()
            .all(|import| import.output_hash.len() == HASH_HEX_CHARS)
    );
    Ok(())
}

/// UI-neutral subagent status projection draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentStatusProjectionDraft {
    /// Child id.
    pub child_id: String,
    /// Parent id.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Status class.
    pub status: SubagentRunStatus,
    /// Budget usage summary.
    pub budget: SubagentContextBudgetSummary,
    /// Optional safe denial reason.
    pub denial_reason: Option<String>,
    /// Optional handoff summary reference.
    pub handoff_summary_ref: Option<String>,
    /// Optional evidence reference.
    pub evidence_ref: Option<String>,
    /// Pending mutation-intent summaries.
    pub pending_mutations: Vec<SubagentMutationIntentSummary>,
    /// Raw child log field; must be absent.
    pub raw_log: Option<String>,
}

/// Validated UI-neutral subagent status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentStatusProjection {
    /// Child id.
    pub child_id: String,
    /// Parent id.
    pub parent_id: String,
    /// Child-run kind.
    pub kind: SubagentChildRunKind,
    /// Status class.
    pub status: SubagentRunStatus,
    /// Budget usage summary.
    pub budget: SubagentContextBudgetSummary,
    /// Optional safe denial reason.
    pub denial_reason: Option<String>,
    /// Optional handoff summary reference.
    pub handoff_summary_ref: Option<String>,
    /// Optional evidence reference.
    pub evidence_ref: Option<String>,
    /// Pending mutation-intent summaries.
    pub pending_mutations: Vec<SubagentMutationIntentSummary>,
}

/// Validates a UI-neutral status projection.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when projection fields are unsafe, lossy, or raw logs are present.
pub fn validate_subagent_status_projection(
    draft: &SubagentStatusProjectionDraft,
) -> Result<SubagentStatusProjection, SubagentDiagnostic> {
    if draft.raw_log.is_some() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsafeText,
            FIELD_UX,
            "status projections must not include raw child logs",
        ));
    }
    let child_id = validate_label!(FIELD_UX, draft.child_id.as_str())?;
    let parent_id = validate_label!(FIELD_UX, draft.parent_id.as_str())?;
    let denial_reason = draft
        .denial_reason
        .as_ref()
        .map(|value| validate_safe_text!(FIELD_UX, value.as_str()))
        .transpose()?;
    let handoff_summary_ref = draft
        .handoff_summary_ref
        .as_ref()
        .map(|value| validate_label!(FIELD_UX, value.as_str()))
        .transpose()?;
    let evidence_ref = draft
        .evidence_ref
        .as_ref()
        .map(|value| validate_label!(FIELD_UX, value.as_str()))
        .transpose()?;
    for mutation in &draft.pending_mutations {
        validate_mutation_intent(mutation)?;
    }
    debug_assert!(draft.raw_log.is_none());
    debug_assert!(!child_id.is_empty());
    Ok(SubagentStatusProjection {
        child_id,
        parent_id,
        kind: draft.kind,
        status: draft.status,
        budget: draft.budget,
        denial_reason,
        handoff_summary_ref,
        evidence_ref,
        pending_mutations: draft.pending_mutations.clone(),
    })
}

/// UI command descriptor kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubagentCommandKind {
    /// Request child-run cancellation.
    Cancel,
    /// Request bounded detail import after policy checks.
    DetailImportRequest,
    /// Acknowledge a child terminal state.
    Acknowledge,
    /// Request mutation-intent arbitration.
    MutationArbitrationRequest,
}

/// UI-neutral command descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCommandDescriptor {
    /// Command id.
    pub command_id: String,
    /// Target child id.
    pub child_id: String,
    /// Command kind.
    pub kind: SubagentCommandKind,
    /// Optional requested detail bound in characters.
    pub requested_detail_bound: Option<u32>,
    /// Optional handoff mode.
    pub handoff_mode: Option<SubagentHandoffMode>,
    /// Safe command reason.
    pub reason: String,
    /// Whether caller authority was admitted by policy.
    pub authorized: bool,
    /// Whether the handle binding is fresh.
    pub handle_fresh: bool,
    /// Optional mutation target for arbitration.
    pub mutation_target: Option<String>,
}

/// Routed command decision without direct effects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubagentCommandDecision {
    /// Whether the descriptor may be handed to the relevant policy or coordinator.
    pub accepted: bool,
    /// Whether normal policy routing is still required before effects.
    pub routed_to_policy: bool,
    /// Safe diagnostic when denied.
    pub diagnostic: Option<SubagentDiagnostic>,
}

/// Routes a UI command descriptor through pure state and policy gates.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the descriptor is malformed or unsafe.
pub fn route_subagent_ux_command(
    state: &SubagentRunState,
    descriptor: &SubagentCommandDescriptor,
) -> Result<SubagentCommandDecision, SubagentDiagnostic> {
    let command_id = validate_label!(FIELD_UX, descriptor.command_id.as_str())?;
    let child_id = validate_label!(FIELD_UX, descriptor.child_id.as_str())?;
    validate_safe_text!(FIELD_UX, descriptor.reason.as_str())?;
    debug_assert!(!command_id.is_empty());
    debug_assert!(!child_id.is_empty());
    if let Some(decision) = command_binding_denial(state, descriptor, child_id.as_str()) {
        return Ok(decision);
    }
    if let Some(decision) = command_kind_denial(state, descriptor)? {
        return Ok(decision);
    }
    Ok(SubagentCommandDecision {
        accepted: true,
        routed_to_policy: true,
        diagnostic: None,
    })
}

fn command_binding_denial(
    state: &SubagentRunState,
    descriptor: &SubagentCommandDescriptor,
    child_id: &str,
) -> Option<SubagentCommandDecision> {
    debug_assert!(!child_id.is_empty());
    debug_assert!(!state.admission.child_id.is_empty());
    if child_id != state.admission.child_id {
        return Some(denied_command(stale_handle(
            "command target does not match active child",
        )));
    }
    if !descriptor.handle_fresh {
        return Some(denied_command(SubagentDiagnostic::new(
            SubagentDiagnosticKind::StaleCommand,
            FIELD_UX,
            "UX command handle binding is stale",
        )));
    }
    if !descriptor.authorized {
        return Some(denied_command(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnauthorizedAction,
            FIELD_UX,
            "UX command requires normal authority admission",
        )));
    }
    None
}

fn command_kind_denial(
    state: &SubagentRunState,
    descriptor: &SubagentCommandDescriptor,
) -> Result<Option<SubagentCommandDecision>, SubagentDiagnostic> {
    debug_assert!(descriptor.handle_fresh);
    debug_assert!(descriptor.authorized);
    match descriptor.kind {
        SubagentCommandKind::Cancel if !state.status.is_active() => Ok(Some(denied_command(
            stale_handle("cancel requires an active child state"),
        ))),
        SubagentCommandKind::DetailImportRequest => detail_import_denial(descriptor),
        SubagentCommandKind::Acknowledge if !state.status.is_terminal() => {
            Ok(Some(denied_command(SubagentDiagnostic::new(
                SubagentDiagnosticKind::StaleCommand,
                FIELD_UX,
                "acknowledgement requires a terminal child state",
            ))))
        }
        SubagentCommandKind::MutationArbitrationRequest => mutation_arbitration_denial(descriptor),
        SubagentCommandKind::Cancel | SubagentCommandKind::Acknowledge => Ok(None),
    }
}

fn detail_import_denial(
    descriptor: &SubagentCommandDescriptor,
) -> Result<Option<SubagentCommandDecision>, SubagentDiagnostic> {
    let bound = descriptor.requested_detail_bound.ok_or_else(|| {
        SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingField,
            FIELD_UX,
            "detail import requests require an explicit bound",
        )
    })?;
    debug_assert!(descriptor.handle_fresh);
    debug_assert!(descriptor.authorized);
    if bound > MAX_DETAIL_IMPORT_CHARS || descriptor.handoff_mode.is_none() {
        return Ok(Some(denied_command(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverBudget,
            FIELD_UX,
            "detail import request must be bounded and tied to a handoff mode",
        ))));
    }
    Ok(None)
}

fn mutation_arbitration_denial(
    descriptor: &SubagentCommandDescriptor,
) -> Result<Option<SubagentCommandDecision>, SubagentDiagnostic> {
    let target = descriptor.mutation_target.as_ref().ok_or_else(|| {
        SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingField,
            FIELD_UX,
            "mutation arbitration requires a mutation target",
        )
    })?;
    let target = validate_label!(FIELD_UX, target.as_str())?;
    debug_assert!(!target.is_empty());
    debug_assert!(descriptor.authorized);
    Ok(None)
}

/// External subagent protocol fixture kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalSubagentProtocolKind {
    /// Agent Client Protocol descriptor.
    Acp,
    /// Model Context Protocol descriptor.
    Mcp,
    /// Unsupported protocol used by negative fixtures.
    Unsupported,
}

/// Expected conformance status for an external fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalSubagentFixtureStatus {
    /// Fixture should be accepted.
    Accepted,
    /// Fixture should be denied with a specific class.
    Denied,
}

/// External subagent conformance fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSubagentFixture {
    /// Fixture id.
    pub fixture_id: String,
    /// External protocol kind.
    pub protocol: ExternalSubagentProtocolKind,
    /// External identity label.
    pub identity: String,
    /// Injected service label.
    pub service_label: String,
    /// Declared tool labels.
    pub declared_tools: Vec<String>,
    /// Requested tool labels.
    pub requested_tools: Vec<String>,
    /// Optional credential reference label.
    pub credential_label: Option<String>,
    /// Whether fixture asks for ambient CLI discovery.
    pub ambient_cli: bool,
    /// Whether fixture asks to inherit parent credentials.
    pub inherited_credentials: bool,
    /// Whether fixture asks to import unsafe output.
    pub unsafe_output_import: bool,
    /// Capability claims.
    pub capability_claims: Vec<String>,
    /// Expected status.
    pub expected_status: ExternalSubagentFixtureStatus,
    /// Expected failure class for denied fixtures.
    pub expected_failure: Option<SubagentDiagnosticKind>,
    /// Requirement ids covered by this fixture.
    pub covered_requirements: Vec<String>,
}

/// External fixture classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSubagentFixtureResult {
    /// Fixture id.
    pub fixture_id: String,
    /// Observed status.
    pub status: ExternalSubagentFixtureStatus,
    /// Observed failure class when denied.
    pub failure: Option<SubagentDiagnosticKind>,
    /// Whether validation was pure/offline.
    pub offline_only: bool,
}

/// External conformance index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSubagentConformanceIndex {
    /// Indexed fixtures.
    pub fixtures: Vec<ExternalSubagentFixture>,
}

/// External conformance report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSubagentConformanceReport {
    /// Fixture results in index order.
    pub results: Vec<ExternalSubagentFixtureResult>,
    /// Covered requirement ids.
    pub covered_requirements: Vec<String>,
}

/// Validates one external subagent conformance fixture without live dependencies.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when the fixture expectation is malformed or
/// observed validation does not match the expected status/failure class.
pub fn validate_external_subagent_fixture(
    fixture: &ExternalSubagentFixture,
) -> Result<ExternalSubagentFixtureResult, SubagentDiagnostic> {
    let fixture_id = validate_label!(FIELD_EXTERNAL, fixture.fixture_id.as_str())?;
    if matches!(
        fixture.expected_status,
        ExternalSubagentFixtureStatus::Denied
    ) && fixture.expected_failure.is_none()
    {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingExpectedFailure,
            FIELD_EXTERNAL,
            "negative external fixture requires an expected failure class",
        ));
    }
    if fixture.covered_requirements.is_empty() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingField,
            FIELD_EXTERNAL,
            "external fixture must name covered requirement ids",
        ));
    }
    let observed_failure = classify_external_fixture_failure(fixture)?;
    let observed_status = if observed_failure.is_some() {
        ExternalSubagentFixtureStatus::Denied
    } else {
        ExternalSubagentFixtureStatus::Accepted
    };
    if observed_status != fixture.expected_status || observed_failure != fixture.expected_failure {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::FixtureExpectationMismatch,
            FIELD_EXTERNAL,
            "external fixture observed status did not match expected conformance index",
        ));
    }
    debug_assert_eq!(observed_status, fixture.expected_status);
    debug_assert_eq!(observed_failure, fixture.expected_failure);
    Ok(ExternalSubagentFixtureResult {
        fixture_id,
        status: observed_status,
        failure: observed_failure,
        offline_only: true,
    })
}

/// Validates a machine-checkable external conformance index.
///
/// # Errors
///
/// Returns [`SubagentDiagnostic`] when fixtures are missing, duplicated, malformed,
/// or do not match their expected pure/offline classification.
pub fn validate_external_subagent_conformance_index(
    index: &ExternalSubagentConformanceIndex,
) -> Result<ExternalSubagentConformanceReport, SubagentDiagnostic> {
    ensure_non_empty(index.fixtures.as_slice(), FIELD_EXTERNAL)?;
    let mut seen = std::collections::BTreeSet::new();
    let mut results = Vec::with_capacity(index.fixtures.len());
    let mut covered = std::collections::BTreeSet::new();
    for fixture in &index.fixtures {
        let fixture_id = validate_label!(FIELD_EXTERNAL, fixture.fixture_id.as_str())?;
        if !seen.insert(fixture_id) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                FIELD_EXTERNAL,
                "external fixture ids must be unique",
            ));
        }
        for requirement in &fixture.covered_requirements {
            covered.insert(validate_safe_text!(FIELD_EXTERNAL, requirement.as_str())?);
        }
        results.push(validate_external_subagent_fixture(fixture)?);
    }
    debug_assert_eq!(seen.len(), index.fixtures.len());
    debug_assert_eq!(results.len(), index.fixtures.len());
    Ok(ExternalSubagentConformanceReport {
        results,
        covered_requirements: covered.into_iter().collect(),
    })
}

fn validate_capability_catalog(
    catalog: &[SubagentCapabilityCatalogEntry],
) -> Result<std::collections::BTreeMap<String, SubagentCapabilityCatalogEntry>, SubagentDiagnostic>
{
    ensure_non_empty(catalog, FIELD_CATALOG)?;
    if catalog.len() > MAX_CAPABILITY_CATALOG_ENTRIES {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::TooLong,
            FIELD_CATALOG,
            "capability catalog exceeds safe entry bound",
        ));
    }
    let mut seen_labels = std::collections::BTreeSet::new();
    let mut seen_identities = std::collections::BTreeSet::new();
    let mut normalized_entries = Vec::with_capacity(catalog.len());
    for entry in catalog.iter().take(MAX_CAPABILITY_CATALOG_ENTRIES) {
        let label = normalize_label!(FIELD_CATALOG, entry.stable_label.as_str())?;
        reject_wildcard_label(label.as_str())?;
        let target = normalize_label!(FIELD_CATALOG, entry.target_label.as_str())?;
        reject_wildcard_label(target.as_str())?;
        if matches!(entry.class, SubagentCapabilityClass::Unsupported) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::UnsupportedCapabilityClass,
                FIELD_CATALOG,
                "capability catalog contains an unsupported class",
            ));
        }
        if matches!(entry.class, SubagentCapabilityClass::ToolUse) && !entry.authorized {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::UnauthorizedTool,
                FIELD_CATALOG,
                "tool capability catalog entry is not authorized",
            ));
        }
        if matches!(entry.class, SubagentCapabilityClass::WorkspaceAccess)
            && entry.workspace_mode.is_none()
        {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::MalformedCatalogEntry,
                FIELD_CATALOG,
                "workspace capability entries require an access mode",
            ));
        }
        if !seen_labels.insert(label.clone()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                FIELD_CATALOG,
                "capability catalog labels must be unique after normalization",
            ));
        }
        let mut normalized_entry = entry.clone();
        normalized_entry.stable_label.clone_from(&label);
        normalized_entry.target_label = target;
        let capability = capability_from_entry(&normalized_entry)?;
        if !seen_identities.insert(capability.identity_key()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::LabelCollision,
                FIELD_CATALOG,
                "multiple labels collapse to the same typed capability",
            ));
        }
        normalized_entries.push((label, normalized_entry));
    }
    debug_assert_eq!(normalized_entries.len(), catalog.len());
    debug_assert!(seen_identities.len() <= normalized_entries.len());
    Ok(normalized_entries.into_iter().collect())
}

fn capability_from_entry(
    entry: &SubagentCapabilityCatalogEntry,
) -> Result<TypedSubagentCapability, SubagentDiagnostic> {
    let stable_label = entry.stable_label.clone();
    let target = entry.target_label.clone();
    let capability = match entry.class {
        SubagentCapabilityClass::ContextRead => TypedSubagentCapability::ContextRead {
            stable_label,
            context_label: target,
        },
        SubagentCapabilityClass::Search => TypedSubagentCapability::Search {
            stable_label,
            index_label: target,
        },
        SubagentCapabilityClass::ToolUse => TypedSubagentCapability::ToolUse {
            stable_label,
            tool_label: target,
        },
        SubagentCapabilityClass::MutationIntent => TypedSubagentCapability::MutationIntent {
            stable_label,
            mutation_kind: target,
        },
        SubagentCapabilityClass::WorkspaceAccess => TypedSubagentCapability::WorkspaceAccess {
            stable_label,
            scope_label: target,
            mode: entry.workspace_mode.ok_or_else(|| {
                SubagentDiagnostic::new(
                    SubagentDiagnosticKind::MalformedCatalogEntry,
                    FIELD_CATALOG,
                    "workspace capability entries require an access mode",
                )
            })?,
        },
        SubagentCapabilityClass::ModelUse => TypedSubagentCapability::ModelUse {
            stable_label,
            model_label: target,
        },
        SubagentCapabilityClass::ExternalService => TypedSubagentCapability::ExternalService {
            stable_label,
            service_label: target,
        },
        SubagentCapabilityClass::Unsupported => {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::UnsupportedCapabilityClass,
                FIELD_CATALOG,
                "unsupported capability class cannot be converted",
            ));
        }
    };
    debug_assert_eq!(capability.stable_label(), entry.stable_label.as_str());
    debug_assert_eq!(capability.class(), entry.class);
    Ok(capability)
}

fn missing_capability_sources(
    input: &SubagentCapabilityResolutionInput,
    capability: &TypedSubagentCapability,
) -> Vec<SubagentCapabilityCeilingSource> {
    let mut sources = Vec::with_capacity(CAPABILITY_CEILING_COUNT);
    if !input.parent_policy.contains(capability) {
        sources.push(SubagentCapabilityCeilingSource::ParentPolicy);
    }
    if !input.recipe_or_profile.contains(capability) {
        sources.push(SubagentCapabilityCeilingSource::RecipeOrProfile);
    }
    if !input.semantic_role.contains(capability) {
        sources.push(SubagentCapabilityCeilingSource::SemanticRole);
    }
    if !input.workspace_scope.contains(capability) {
        sources.push(SubagentCapabilityCeilingSource::WorkspaceScope);
    }
    if !input.host_admission.contains(capability) {
        sources.push(SubagentCapabilityCeilingSource::HostAdmission);
    }
    sources
}

fn ensure_unique_capabilities(
    capabilities: &[TypedSubagentCapability],
    field: &'static str,
) -> Result<(), SubagentDiagnostic> {
    ensure_non_empty(capabilities, field)?;
    let mut seen = std::collections::BTreeSet::new();
    for capability in capabilities {
        if !seen.insert(capability.identity_key()) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                field,
                "typed capability identities must be unique",
            ));
        }
    }
    Ok(())
}

fn validate_context_item(
    policy: SubagentContextAdmissionPolicy,
    item: &SubagentContextItemDraft,
) -> Result<(), SubagentDiagnostic> {
    validate_label!(FIELD_CONTEXT, item.label.as_str())?;
    validate_safe_text!(FIELD_CONTEXT, item.safe_body.as_str())?;
    if item.token_estimate < MIN_POSITIVE_U32 {
        return Err(over_budget("context item token estimate must be positive"));
    }
    if !item.kind.is_safe_by_default() && !policy.admit_unsafe_classes {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsafeContext,
            FIELD_CONTEXT,
            "unsafe context class requires explicit admission",
        ));
    }
    if matches!(item.redaction, SubagentContextRedaction::Raw) && !item.kind.is_safe_by_default() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsafeContext,
            FIELD_CONTEXT,
            "raw unsafe context cannot enter shared packets",
        ));
    }
    debug_assert!(item.token_estimate >= MIN_POSITIVE_U32);
    debug_assert!(item.kind.is_safe_by_default() || policy.admit_unsafe_classes);
    Ok(())
}

fn validate_kind_fields(draft: &SubagentChildRunDraft) -> Result<String, SubagentDiagnostic> {
    match draft.kind {
        SubagentChildRunKind::RecipeBacked => {
            let recipe = draft.recipe.as_ref().ok_or_else(missing_kind_field)?;
            let recipe_id = validate_label!(FIELD_CHILD, recipe.recipe_id.as_str())?;
            validate_label!(FIELD_CHILD, recipe.trust_receipt_label.as_str())?;
            Ok(recipe_id)
        }
        SubagentChildRunKind::FreshContextInvestigation => {
            let fresh = draft
                .fresh_context
                .as_ref()
                .ok_or_else(missing_kind_field)?;
            validate_safe_text!(FIELD_CHILD, fresh.objective.as_str())?;
            validate_label!(FIELD_CHILD, fresh.scheduler_label.as_str())
        }
        SubagentChildRunKind::ReadOnlySideQuestion => {
            let fresh = draft
                .fresh_context
                .as_ref()
                .ok_or_else(missing_kind_field)?;
            if !fresh.read_only {
                return Err(SubagentDiagnostic::new(
                    SubagentDiagnosticKind::LossyProjection,
                    FIELD_CHILD,
                    "side-question child runs must preserve read-only posture",
                ));
            }
            validate_safe_text!(FIELD_CHILD, fresh.objective.as_str())?;
            validate_label!(FIELD_CHILD, fresh.scheduler_label.as_str())
        }
        SubagentChildRunKind::WorkerTeamTask => {
            let worker = draft.worker_team.as_ref().ok_or_else(missing_kind_field)?;
            validate_label!(FIELD_CHILD, worker.output_schema_label.as_str())?;
            validate_label!(FIELD_CHILD, worker.service_label.as_str())
        }
        SubagentChildRunKind::Unsupported => Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsupportedKind,
            FIELD_CHILD,
            "unsupported child-run kind has no registered typed validator",
        )),
    }
}

fn missing_kind_field() -> SubagentDiagnostic {
    SubagentDiagnostic::new(
        SubagentDiagnosticKind::MissingField,
        FIELD_CHILD,
        "child-run kind-specific fields are required",
    )
}

fn validate_subagent_report(
    admission: &SubagentAdmission,
    report: &SubagentReport,
) -> Result<(), SubagentDiagnostic> {
    let child_id = validate_label!(FIELD_REPORT, report.child_id.as_str())?;
    if child_id != admission.child_id {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MalformedReport,
            FIELD_REPORT,
            "child report id must match admission",
        ));
    }
    if !report.status.is_terminal() || matches!(report.status, SubagentRunStatus::Collected) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MalformedReport,
            FIELD_REPORT,
            "collected child report must carry a terminal pre-collection status",
        ));
    }
    if matches!(report.status, SubagentRunStatus::Completed) && report.output_hash.is_none() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MalformedReport,
            FIELD_REPORT,
            "completed child reports require an output hash",
        ));
    }
    if let Some(output_hash) = &report.output_hash {
        validate_hash(output_hash.as_str())?;
    }
    validate_evidence_refs(report.evidence_refs.as_slice(), FIELD_REPORT)?;
    for mutation in &report.mutation_intents {
        validate_mutation_intent(mutation)?;
    }
    debug_assert_eq!(child_id, admission.child_id);
    debug_assert!(report.status.is_terminal());
    Ok(())
}

fn indexed_outcomes(
    outcomes: &[SubagentFanInOutcome],
) -> Result<std::collections::BTreeMap<String, &SubagentFanInOutcome>, SubagentDiagnostic> {
    ensure_non_empty(outcomes, FIELD_PLAN)?;
    let mut by_child = std::collections::BTreeMap::new();
    for outcome in outcomes {
        let child_id = validate_label!(FIELD_PLAN, outcome.child_id.as_str())?;
        if by_child.insert(child_id, outcome).is_some() {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateChild,
                FIELD_PLAN,
                "fan-in outcomes must contain each child id only once",
            ));
        }
    }
    Ok(by_child)
}

fn validate_outcome_report(
    child: &SubagentAdmission,
    outcome: &SubagentFanInOutcome,
) -> Result<(), SubagentDiagnostic> {
    if let Some(report) = &outcome.report {
        validate_subagent_report(child, report)?;
    }
    if matches!(outcome.status, SubagentRunStatus::Completed) && outcome.report.is_none() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MalformedReport,
            FIELD_PLAN,
            "successful fan-in outcomes require validated child reports",
        ));
    }
    for mutation in &outcome.mutation_intents {
        validate_mutation_intent(mutation)?;
    }
    Ok(())
}

fn status_to_aggregate(status: SubagentRunStatus) -> SubagentFanInStatus {
    match status {
        SubagentRunStatus::Completed | SubagentRunStatus::Collected => SubagentFanInStatus::Success,
        SubagentRunStatus::TimedOut => SubagentFanInStatus::TimedOut,
        SubagentRunStatus::Cancelled | SubagentRunStatus::Cancelling => {
            SubagentFanInStatus::Cancelled
        }
        SubagentRunStatus::Failed => SubagentFanInStatus::Failed,
        SubagentRunStatus::Blocked => SubagentFanInStatus::DeniedCapability,
        SubagentRunStatus::Admitted
        | SubagentRunStatus::Queued
        | SubagentRunStatus::Running
        | SubagentRunStatus::AwaitingParent => SubagentFanInStatus::Malformed,
    }
}

fn arbitrate_mutations(
    policy: SubagentMutationPolicy,
    intents: Vec<SubagentMutationIntentSummary>,
) -> Result<SubagentMutationArbitration, SubagentDiagnostic> {
    let intent_bound = intents.len();
    let mut denied = Vec::with_capacity(intent_bound);
    let mut unresolved = Vec::with_capacity(intent_bound);
    let mut conflicts = Vec::with_capacity(intent_bound);
    match policy {
        SubagentMutationPolicy::DenyAll => {
            if !intents.is_empty() {
                return Err(SubagentDiagnostic::new(
                    SubagentDiagnosticKind::MutationBypass,
                    FIELD_PLAN,
                    "mutation intents cannot bypass deny-all mutation policy",
                ));
            }
        }
        SubagentMutationPolicy::CollectUnresolved => unresolved = intents,
        SubagentMutationPolicy::AllowNonConflicting => {
            let mut by_target = std::collections::BTreeMap::new();
            for intent in intents {
                if let Some(existing_kind) =
                    by_target.insert(intent.target.clone(), intent.mutation_kind.clone())
                {
                    if existing_kind == intent.mutation_kind {
                        unresolved.push(intent);
                    } else {
                        conflicts.push(intent);
                    }
                } else {
                    unresolved.push(intent);
                }
            }
        }
    }
    denied.sort();
    unresolved.sort();
    conflicts.sort();
    debug_assert!(denied.len() <= intent_bound);
    debug_assert!(unresolved.len().saturating_add(conflicts.len()) <= intent_bound);
    Ok(SubagentMutationArbitration {
        denied,
        unresolved,
        conflicts,
    })
}

fn aggregate_status(
    statuses: &[SubagentFanInStatus],
    arbitration: &SubagentMutationArbitration,
    has_success: bool,
) -> SubagentFanInStatus {
    debug_assert!(!statuses.is_empty());
    debug_assert!(!has_success || statuses.contains(&SubagentFanInStatus::Success));
    if !arbitration.conflicts.is_empty() {
        return SubagentFanInStatus::WriteConflict;
    }
    if statuses.contains(&SubagentFanInStatus::BudgetExhausted) {
        return SubagentFanInStatus::BudgetExhausted;
    }
    if statuses
        .iter()
        .all(|status| matches!(status, SubagentFanInStatus::Success))
    {
        return SubagentFanInStatus::Success;
    }
    if has_success {
        return SubagentFanInStatus::PartialSuccess;
    }
    if statuses.contains(&SubagentFanInStatus::Malformed) {
        return SubagentFanInStatus::Malformed;
    }
    if statuses.contains(&SubagentFanInStatus::DeniedCapability) {
        return SubagentFanInStatus::DeniedCapability;
    }
    if statuses.contains(&SubagentFanInStatus::TimedOut) {
        return SubagentFanInStatus::TimedOut;
    }
    if statuses.contains(&SubagentFanInStatus::Cancelled) {
        return SubagentFanInStatus::Cancelled;
    }
    SubagentFanInStatus::Failed
}

fn classify_external_fixture_failure(
    fixture: &ExternalSubagentFixture,
) -> Result<Option<SubagentDiagnosticKind>, SubagentDiagnostic> {
    validate_label!(FIELD_EXTERNAL, fixture.identity.as_str())?;
    if matches!(fixture.protocol, ExternalSubagentProtocolKind::Unsupported) {
        return Ok(Some(SubagentDiagnosticKind::MalformedProtocol));
    }
    if fixture.ambient_cli {
        return Ok(Some(SubagentDiagnosticKind::AmbientDiscoveryDenied));
    }
    if fixture.inherited_credentials {
        return Ok(Some(SubagentDiagnosticKind::InheritedCredentialDenied));
    }
    if fixture.service_label.trim().is_empty() {
        return Ok(Some(SubagentDiagnosticKind::MissingService));
    }
    validate_label!(FIELD_EXTERNAL, fixture.service_label.as_str())?;
    let declared = labels_set(&fixture.declared_tools)?;
    if declared.is_empty() {
        return Ok(Some(SubagentDiagnosticKind::MissingField));
    }
    for tool in &fixture.requested_tools {
        let normalized = normalize_label!(FIELD_EXTERNAL, tool.as_str())?;
        if !declared.contains(normalized.as_str()) {
            return Ok(Some(SubagentDiagnosticKind::UndeclaredTool));
        }
    }
    if fixture.unsafe_output_import {
        return Ok(Some(SubagentDiagnosticKind::UnsafeOutputImport));
    }
    for claim in &fixture.capability_claims {
        let normalized = normalize_label!(FIELD_EXTERNAL, claim.as_str())?;
        if matches!(normalized.as_str(), WILDCARD_STAR | WILDCARD_ALL) {
            return Ok(Some(SubagentDiagnosticKind::OverbroadAuthority));
        }
    }
    if let Some(credential_label) = &fixture.credential_label {
        validate_label!(FIELD_EXTERNAL, credential_label.as_str())?;
    }
    debug_assert!(!fixture.identity.trim().is_empty());
    debug_assert!(!fixture.service_label.trim().is_empty());
    Ok(None)
}

fn labels_set(labels: &[String]) -> Result<std::collections::BTreeSet<String>, SubagentDiagnostic> {
    let mut set = std::collections::BTreeSet::new();
    for label in labels {
        let normalized = normalize_label!(FIELD_EXTERNAL, label.as_str())?;
        if !set.insert(normalized) {
            return Err(SubagentDiagnostic::new(
                SubagentDiagnosticKind::DuplicateNormalizedLabel,
                FIELD_EXTERNAL,
                "labels must be unique after normalization",
            ));
        }
    }
    Ok(set)
}

fn injected_subagent_service(
    service: Option<&mut dyn SubagentService>,
) -> Result<&mut dyn SubagentService, SubagentDiagnostic> {
    service.ok_or_else(|| {
        SubagentDiagnostic::new(
            SubagentDiagnosticKind::ServiceUnavailable,
            FIELD_SERVICE,
            "subagent coordination requires an injected host service",
        )
    })
}

fn service_failure_to_diagnostic(failure: SubagentServiceFailure) -> SubagentDiagnostic {
    failure.diagnostic
}

fn require_status(
    actual: SubagentRunStatus,
    expected: SubagentRunStatus,
) -> Result<(), SubagentDiagnostic> {
    if actual == expected {
        Ok(())
    } else {
        Err(stale_handle(
            "coordinator transition is not valid for current state",
        ))
    }
}

fn status_for_failure(kind: SubagentServiceFailureKind) -> SubagentRunStatus {
    match kind {
        SubagentServiceFailureKind::Unavailable
        | SubagentServiceFailureKind::Denied
        | SubagentServiceFailureKind::MalformedReport
        | SubagentServiceFailureKind::StaleHandle => SubagentRunStatus::Failed,
        SubagentServiceFailureKind::TimedOut => SubagentRunStatus::TimedOut,
        SubagentServiceFailureKind::Cancelled => SubagentRunStatus::Cancelled,
    }
}

fn validate_mutation_intent(
    intent: &SubagentMutationIntentSummary,
) -> Result<(), SubagentDiagnostic> {
    validate_label!(FIELD_REPORT, intent.target.as_str())?;
    validate_label!(FIELD_REPORT, intent.mutation_kind.as_str())?;
    validate_label!(FIELD_REPORT, intent.evidence_ref.as_str())?;
    Ok(())
}

fn validate_evidence_refs(refs: &[String], field: &'static str) -> Result<(), SubagentDiagnostic> {
    if refs.len() > MAX_EVIDENCE_REFS {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::TooLong,
            field,
            "evidence reference list exceeds safe bound",
        ));
    }
    for evidence_ref in refs {
        validate_label!(field, evidence_ref.as_str())?;
    }
    Ok(())
}

fn validate_hash(value: &str) -> Result<(), SubagentDiagnostic> {
    if value.len() != HASH_HEX_CHARS || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::InvalidHash,
            FIELD_LINEAGE,
            format!("expected {BLAKE3_ALGORITHM} hex identity"),
        ));
    }
    Ok(())
}

fn validate_label_input(input: TextValidationInput<'_>) -> Result<String, SubagentDiagnostic> {
    let normalized = normalize_label!(input.field, input.value)?;
    reject_unsafe_text!(input.field, normalized.as_str())?;
    Ok(normalized)
}

fn normalize_label_input(input: TextValidationInput<'_>) -> Result<String, SubagentDiagnostic> {
    let trimmed = input.value.trim();
    if trimmed.is_empty() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::EmptyField,
            input.field,
            "label must be non-empty",
        ));
    }
    if trimmed.len() > MAX_LABEL_CHARS {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::TooLong,
            input.field,
            "label exceeds safe length bound",
        ));
    }
    Ok(trimmed.to_ascii_lowercase())
}

fn validate_safe_text_input(input: TextValidationInput<'_>) -> Result<String, SubagentDiagnostic> {
    let trimmed = input.value.trim();
    if trimmed.is_empty() {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::EmptyField,
            input.field,
            "safe text must be non-empty",
        ));
    }
    if trimmed.len() > MAX_TEXT_CHARS {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::TooLong,
            input.field,
            "safe text exceeds length bound",
        ));
    }
    reject_unsafe_text!(input.field, trimmed)?;
    Ok(trimmed.to_owned())
}

fn reject_unsafe_text_input(input: TextValidationInput<'_>) -> Result<(), SubagentDiagnostic> {
    if UNSAFE_TEXT_MARKERS
        .iter()
        .any(|marker| input.value.contains(marker))
    {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::UnsafeText,
            input.field,
            "text contains raw, credential-like, provider, path, or unbounded output marker",
        ));
    }
    Ok(())
}

fn reject_wildcard_label(value: &str) -> Result<(), SubagentDiagnostic> {
    if matches!(value, WILDCARD_STAR | WILDCARD_ALL) {
        return Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::OverbroadAuthority,
            FIELD_CAPABILITY,
            "wildcard capability authority is not allowed",
        ));
    }
    Ok(())
}

fn ensure_non_empty<T>(values: &[T], field: &'static str) -> Result<(), SubagentDiagnostic> {
    if values.is_empty() {
        Err(SubagentDiagnostic::new(
            SubagentDiagnosticKind::MissingField,
            field,
            "required collection must be non-empty",
        ))
    } else {
        Ok(())
    }
}

fn over_budget(message: impl Into<String>) -> SubagentDiagnostic {
    SubagentDiagnostic::new(SubagentDiagnosticKind::OverBudget, FIELD_CONTEXT, message)
}

fn stale_handle(message: impl Into<String>) -> SubagentDiagnostic {
    SubagentDiagnostic::new(SubagentDiagnosticKind::StaleHandle, FIELD_SERVICE, message)
}

fn denied_command(diagnostic: SubagentDiagnostic) -> SubagentCommandDecision {
    SubagentCommandDecision {
        accepted: false,
        routed_to_policy: false,
        diagnostic: Some(diagnostic),
    }
}

fn child_kind_label(kind: SubagentChildRunKind) -> &'static str {
    match kind {
        SubagentChildRunKind::RecipeBacked => "recipe_backed",
        SubagentChildRunKind::FreshContextInvestigation => "fresh_context_investigation",
        SubagentChildRunKind::ReadOnlySideQuestion => "read_only_side_question",
        SubagentChildRunKind::WorkerTeamTask => "worker_team_task",
        SubagentChildRunKind::Unsupported => UNSUPPORTED_KIND_LABEL,
    }
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(HASH_PART_SEPARATOR.as_bytes());
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(HASH_PART_SEPARATOR.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHILD_ID: &str = "child-review";
    const PARENT_ID: &str = "parent-run";
    const PACKET_ID: &str = "packet-review";
    const SUMMARY_LABEL: &str = "selected-summary";
    const SUMMARY_BODY: &str = "Safe summary of parent context";
    const TOOL_CAPABILITY: &str = "tool:read-docs";
    const SEARCH_CAPABILITY: &str = "search:docs";
    const MUTATION_CAPABILITY: &str = "mutation:patch-proposal";
    const WORKSPACE_CAPABILITY: &str = "workspace:repo-write";
    const SERVICE_CAPABILITY: &str = "external:mcp-reviewer";
    const TOOL_LABEL: &str = "read-docs";
    const SEARCH_LABEL: &str = "docs-index";
    const MUTATION_LABEL: &str = "patch-proposal";
    const WORKSPACE_LABEL: &str = "repo";
    const SERVICE_LABEL: &str = "mcp-reviewer";
    const RECIPE_ID: &str = "review-recipe";
    const TRUST_RECEIPT: &str = "trusted-recipe";
    const SCHEDULER_LABEL: &str = "fresh-service";
    const OUTPUT_SCHEMA: &str = "review-report";
    const HANDLE_ID: &str = "handle-child-review";
    const EVIDENCE_REF: &str = "ev-summary";
    const MUTATION_TARGET: &str = "docs-runtime";
    const PLAN_ID: &str = "plan-review";
    const DELEGATION_SOURCE: &str = "user";
    const SAFE_SUMMARY_REF: &str = "child summary ref";
    const COMMAND_ID: &str = "cmd-cancel";
    const VALID_TOKENS: u32 = 12;
    const TWO_CHILDREN: u16 = 2;
    const TWO_OUTCOMES: usize = 2;
    const EXPECTED_ONE: usize = 1;

    #[test]
    fn typed_capabilities_convert_intersect_and_keep_receipt_labels() {
        // r[verify agent_harness_runtime.subagent_typed_capabilities.types]
        // r[verify agent_harness_runtime.subagent_typed_capabilities.intersection]
        // r[verify agent_harness_runtime.subagent_typed_capabilities.compatibility]
        // r[verify agent_harness_runtime.subagent_typed_capabilities.verification]
        let conversion = convert_subagent_capability_labels(
            &catalog(),
            &[TOOL_CAPABILITY, SEARCH_CAPABILITY, MUTATION_CAPABILITY],
        )
        .expect("catalog labels should convert");
        let request = conversion.capabilities.clone();
        let denied_workspace = request
            .iter()
            .filter(|capability| {
                !matches!(capability, TypedSubagentCapability::MutationIntent { .. })
            })
            .cloned()
            .collect::<Vec<_>>();
        let resolution = resolve_typed_subagent_capabilities(&SubagentCapabilityResolutionInput {
            parent_policy: request.clone(),
            recipe_or_profile: request.clone(),
            semantic_role: request.clone(),
            request,
            workspace_scope: denied_workspace,
            host_admission: conversion.capabilities.clone(),
        })
        .expect("typed resolution should run");

        assert_eq!(resolution.effective.len(), usize::from(TWO_CHILDREN));
        assert!(resolution.denied.iter().any(|denial| {
            denial.source == SubagentCapabilityCeilingSource::WorkspaceScope
                && denial.capability.stable_label() == MUTATION_CAPABILITY
        }));
        let receipt = subagent_capability_receipt_label(&conversion.capabilities[0]);
        assert_eq!(receipt.stable_label, TOOL_CAPABILITY);
        assert_eq!(receipt.class, SubagentCapabilityClass::ToolUse);
        assert_eq!(receipt.capability_hash.len(), HASH_HEX_CHARS);
    }

    #[test]
    fn typed_capabilities_fail_closed_for_bad_labels_and_catalogs() {
        // r[verify agent_harness_runtime.subagent_typed_capabilities.fail_closed]
        assert_eq!(
            convert_subagent_capability_labels(&catalog(), &["*"])
                .expect_err("wildcard labels must fail")
                .kind,
            SubagentDiagnosticKind::OverbroadAuthority
        );
        assert_eq!(
            convert_subagent_capability_labels(&catalog(), &["unknown"])
                .expect_err("unknown labels must fail")
                .kind,
            SubagentDiagnosticKind::UnknownLabel
        );
        let duplicate_catalog = vec![catalog()[0].clone(), catalog()[0].clone()];
        assert_eq!(
            convert_subagent_capability_labels(&duplicate_catalog, &[TOOL_CAPABILITY])
                .expect_err("duplicate normalized catalog labels must fail")
                .kind,
            SubagentDiagnosticKind::DuplicateNormalizedLabel
        );
        let unauthorized = vec![SubagentCapabilityCatalogEntry {
            authorized: false,
            ..catalog()[0].clone()
        }];
        assert_eq!(
            convert_subagent_capability_labels(&unauthorized, &[TOOL_CAPABILITY])
                .expect_err("unauthorized tools must fail")
                .kind,
            SubagentDiagnosticKind::UnauthorizedTool
        );
    }

    #[test]
    fn context_packets_validate_identity_projection_and_leaks() {
        // r[verify agent_harness_runtime.subagent_context_packets.shared_packet]
        // r[verify agent_harness_runtime.subagent_context_packets.unsafe_admission]
        // r[verify agent_harness_runtime.subagent_context_packets.identity]
        // r[verify agent_harness_runtime.subagent_context_packets.kind_projection]
        // r[verify agent_harness_runtime.subagent_context_packets.fail_closed]
        // r[verify agent_harness_runtime.subagent_context_packets.verification]
        let packet = validate_subagent_context_packet(&context_packet_draft(
            SubagentChildRunKind::RecipeBacked,
        ))
        .expect("minimized context packet should validate");
        assert_eq!(packet.budget.item_count, MIN_POSITIVE_U16);
        assert_eq!(packet.packet_hash.len(), HASH_HEX_CHARS);
        assert!(
            project_context_packet_for_child_kind(&packet, SubagentChildRunKind::RecipeBacked)
                .is_ok()
        );
        assert_eq!(
            project_context_packet_for_child_kind(&packet, SubagentChildRunKind::WorkerTeamTask)
                .expect_err("lossy kind projection must fail")
                .kind,
            SubagentDiagnosticKind::LossyProjection
        );

        let mut unsafe_packet = context_packet_draft(SubagentChildRunKind::RecipeBacked);
        unsafe_packet.items[0].kind = SubagentContextItemKind::Credential;
        assert_eq!(
            validate_subagent_context_packet(&unsafe_packet)
                .expect_err("credential context must fail")
                .kind,
            SubagentDiagnosticKind::UnsafeContext
        );
        let mut over_budget = context_packet_draft(SubagentChildRunKind::RecipeBacked);
        over_budget.admission.max_tokens = MIN_POSITIVE_U32;
        assert_eq!(
            validate_subagent_context_packet(&over_budget)
                .expect_err("over-budget packets must fail")
                .kind,
            SubagentDiagnosticKind::OverBudget
        );
    }

    #[test]
    fn facade_validates_all_kinds_without_execution_and_fails_closed() {
        // r[verify agent_harness_runtime.subagent_facade.boundary]
        // r[verify agent_harness_runtime.subagent_facade.lossless_projection]
        // r[verify agent_harness_runtime.subagent_facade.all_kinds]
        // r[verify agent_harness_runtime.subagent_facade.fail_closed]
        // r[verify agent_harness_runtime.subagent_facade.verification]
        for kind in [
            SubagentChildRunKind::RecipeBacked,
            SubagentChildRunKind::FreshContextInvestigation,
            SubagentChildRunKind::ReadOnlySideQuestion,
            SubagentChildRunKind::WorkerTeamTask,
        ] {
            let child =
                validate_subagent_child_run(&child_draft(kind)).expect("kind should validate");
            let admission = admit_subagent_child_run(&child);
            assert_eq!(admission.child_id, CHILD_ID);
            assert_eq!(admission.kind, kind);
            assert_eq!(admission.receipt_hash.len(), HASH_HEX_CHARS);
        }
        assert_eq!(
            validate_subagent_child_run(&child_draft(SubagentChildRunKind::Unsupported))
                .expect_err("unsupported kind must fail")
                .kind,
            SubagentDiagnosticKind::UnsupportedKind
        );
        let mut executing = child_draft(SubagentChildRunKind::RecipeBacked);
        executing.execution_requested = true;
        assert_eq!(
            validate_subagent_child_run(&executing)
                .expect_err("facade validation must not execute")
                .kind,
            SubagentDiagnosticKind::ExecutionAuthorityRequired
        );
    }

    #[test]
    fn coordinator_uses_injected_service_and_safe_lifecycle_events() {
        // r[verify agent_harness_runtime.subagent_coordinator.service]
        // r[verify agent_harness_runtime.subagent_coordinator.state_machine]
        // r[verify agent_harness_runtime.subagent_coordinator.lifecycle]
        // r[verify agent_harness_runtime.subagent_coordinator.verification]
        let admission = valid_admission(SubagentChildRunKind::WorkerTeamTask);
        let state = SubagentRunState::admitted(admission);
        assert_eq!(
            subagent_enqueue(None, &state)
                .expect_err("missing service must fail closed")
                .kind,
            SubagentDiagnosticKind::ServiceUnavailable
        );
        let mut service = RecordingSubagentService::default();
        let queued = subagent_enqueue(Some(&mut service), &state).expect("enqueue through service");
        let running = subagent_status(Some(&mut service), &queued).expect("status through service");
        assert_eq!(running.status, SubagentRunStatus::Running);
        let collected =
            subagent_collect(Some(&mut service), &running).expect("collect through service");
        let event = project_subagent_lifecycle_event(&collected);
        assert_eq!(event.status, SubagentRunStatus::Collected);
        assert_eq!(event.evidence_refs, vec![String::from(EVIDENCE_REF)]);
        assert_eq!(service.enqueues, EXPECTED_ONE);
    }

    #[test]
    fn coordinator_failures_block_stale_malformed_and_ambient_fallbacks() {
        // r[verify agent_harness_runtime.subagent_coordinator.fail_closed]
        let admission = valid_admission(SubagentChildRunKind::WorkerTeamTask);
        let state = SubagentRunState::admitted(admission.clone());
        assert_eq!(
            subagent_status(None, &state)
                .expect_err("stale handle must fail")
                .kind,
            SubagentDiagnosticKind::StaleHandle
        );
        let malformed_child_result = SubagentReport {
            child_id: String::from("other-child"),
            status: SubagentRunStatus::Completed,
            output_hash: Some(hash_parts(SAFE_HASH_DOMAIN_RECEIPT, &["output"])),
            evidence_refs: vec![String::from(EVIDENCE_REF)],
            mutation_intents: Vec::new(),
        };
        let state_with_handle = SubagentRunState {
            handle: Some(SubagentHandle::new(HANDLE_ID).expect("valid handle")),
            ..SubagentRunState::admitted(admission)
        };
        assert_eq!(
            transition_subagent_state(
                &state_with_handle,
                SubagentCoordinatorEvent::Collect(malformed_child_result)
            )
            .expect_err("mismatched reports must fail")
            .kind,
            SubagentDiagnosticKind::MalformedReport
        );
    }

    #[test]
    fn fanout_plan_and_fanin_aggregation_are_deterministic() {
        // r[verify agent_harness_runtime.subagent_fanout_fanin.plan]
        // r[verify agent_harness_runtime.subagent_fanout_fanin.aggregation]
        // r[verify agent_harness_runtime.subagent_fanout_fanin.cancellation_budget]
        // r[verify agent_harness_runtime.subagent_fanout_fanin.mutation_arbitration]
        // r[verify agent_harness_runtime.subagent_fanout_fanin.verification]
        let plan = validate_subagent_fanout_plan(&fanout_plan_draft())
            .expect("fan-out plan should validate without execution");
        let outcomes = vec![
            successful_outcome(plan.children[0].child_id.as_str(), Vec::new()),
            SubagentFanInOutcome {
                child_id: plan.children[1].child_id.clone(),
                status: SubagentRunStatus::TimedOut,
                report: None,
                budget_used: VALID_TOKENS,
                mutation_intents: Vec::new(),
            },
        ];
        let aggregate =
            aggregate_subagent_fan_in(&plan, &outcomes).expect("fanin should aggregate");
        assert_eq!(outcomes.len(), TWO_OUTCOMES);
        assert_eq!(aggregate.status, SubagentFanInStatus::PartialSuccess);
        assert_eq!(
            aggregate.successful_children,
            vec![plan.children[0].child_id.clone()]
        );
    }

    #[test]
    fn fanout_failures_reject_duplicates_conflicts_budgets_and_bypass() {
        // r[verify agent_harness_runtime.subagent_fanout_fanin.fail_closed]
        let mut duplicate = fanout_plan_draft();
        duplicate.children[1].child_id = duplicate.children[0].child_id.clone();
        assert_eq!(
            validate_subagent_fanout_plan(&duplicate)
                .expect_err("duplicate children must fail")
                .kind,
            SubagentDiagnosticKind::DuplicateChild
        );
        let mut deny_plan_draft = fanout_plan_draft();
        deny_plan_draft.mutation_policy = SubagentMutationPolicy::DenyAll;
        let deny_plan =
            validate_subagent_fanout_plan(&deny_plan_draft).expect("deny-all plan validates");
        let mutation = valid_mutation();
        let outcomes = deny_plan
            .children
            .iter()
            .map(|child| successful_outcome(child.child_id.as_str(), vec![mutation.clone()]))
            .collect::<Vec<_>>();
        assert_eq!(
            aggregate_subagent_fan_in(&deny_plan, &outcomes)
                .expect_err("mutation bypass must fail")
                .kind,
            SubagentDiagnosticKind::MutationBypass
        );
        let mut conflict_plan_draft = fanout_plan_draft();
        conflict_plan_draft.mutation_policy = SubagentMutationPolicy::AllowNonConflicting;
        let conflict_plan =
            validate_subagent_fanout_plan(&conflict_plan_draft).expect("conflict plan validates");
        let outcomes = vec![
            successful_outcome(
                conflict_plan.children[0].child_id.as_str(),
                vec![valid_mutation()],
            ),
            successful_outcome(
                conflict_plan.children[1].child_id.as_str(),
                vec![SubagentMutationIntentSummary {
                    mutation_kind: String::from("different-patch"),
                    ..valid_mutation()
                }],
            ),
        ];
        assert_eq!(
            aggregate_subagent_fan_in(&conflict_plan, &outcomes)
                .expect("write conflict should aggregate safely")
                .status,
            SubagentFanInStatus::WriteConflict
        );
    }

    #[test]
    fn lineage_records_and_replay_validate_child_influence() {
        // r[verify agent_harness_runtime.subagent_lineage_ledger.record]
        // r[verify agent_harness_runtime.subagent_lineage_ledger.lifecycle_projection]
        // r[verify agent_harness_runtime.subagent_lineage_ledger.replay]
        // r[verify agent_harness_runtime.subagent_lineage_ledger.verification]
        let event = project_subagent_lifecycle_event(&collected_state());
        let output_hash = hash_parts(SAFE_HASH_DOMAIN_RECEIPT, &["child-output"]);
        let draft = lineage_draft_from_lifecycle(SubagentLineageProjection {
            event: &event,
            delegation_source: DELEGATION_SOURCE,
            output_hash: output_hash.as_str(),
            handoff_mode: SubagentHandoffMode::StatusPlusSummary,
            safe_summary: SAFE_SUMMARY_REF,
        });
        let record = validate_subagent_lineage_record(&draft).expect("lineage should validate");
        assert_eq!(record.parent_id, event.parent_id);
        assert_eq!(record.child_id, event.child_id);
        validate_subagent_replay_influence(
            &[record],
            &[SubagentInfluenceImport {
                parent_id: event.parent_id,
                child_id: event.child_id,
                output_hash,
                handoff_mode: SubagentHandoffMode::StatusPlusSummary,
            }],
        )
        .expect("replay influence should match lineage");
    }

    #[test]
    fn lineage_failures_reject_missing_mismatch_hash_and_raw_transcripts() {
        // r[verify agent_harness_runtime.subagent_lineage_ledger.fail_closed]
        let event = project_subagent_lifecycle_event(&collected_state());
        let mut draft = lineage_draft_from_lifecycle(SubagentLineageProjection {
            event: &event,
            delegation_source: DELEGATION_SOURCE,
            output_hash: "not-a-hash",
            handoff_mode: SubagentHandoffMode::StatusPlusSummary,
            safe_summary: SAFE_SUMMARY_REF,
        });
        assert_eq!(
            validate_subagent_lineage_record(&draft)
                .expect_err("bad hashes must fail")
                .kind,
            SubagentDiagnosticKind::InvalidHash
        );
        draft.output_hash = hash_parts(SAFE_HASH_DOMAIN_RECEIPT, &["child-output"]);
        draft.safe_summary = String::from("RAW_TRANSCRIPT: child log");
        assert_eq!(
            validate_subagent_lineage_record(&draft)
                .expect_err("raw transcript marker must fail")
                .kind,
            SubagentDiagnosticKind::UnsafeText
        );
        assert_eq!(
            validate_subagent_replay_influence(
                &[],
                &[SubagentInfluenceImport {
                    parent_id: String::from(PARENT_ID),
                    child_id: String::from(CHILD_ID),
                    output_hash: hash_parts(SAFE_HASH_DOMAIN_RECEIPT, &["child-output"]),
                    handoff_mode: SubagentHandoffMode::SummaryOnly,
                }],
            )
            .expect_err("missing lineage must fail")
            .kind,
            SubagentDiagnosticKind::MissingLineage
        );
    }

    #[test]
    fn ux_status_and_commands_are_policy_routed_descriptors() {
        // r[verify agent_harness_runtime.subagent_ux_hooks.status_projection]
        // r[verify agent_harness_runtime.subagent_ux_hooks.safe_metadata]
        // r[verify agent_harness_runtime.subagent_ux_hooks.command_descriptors]
        // r[verify agent_harness_runtime.subagent_ux_hooks.policy_routing]
        // r[verify agent_harness_runtime.subagent_ux_hooks.verification]
        let projection = validate_subagent_status_projection(&status_projection_draft())
            .expect("safe projection should validate");
        assert_eq!(projection.status, SubagentRunStatus::Running);
        let state = running_state();
        let decision =
            route_subagent_ux_command(&state, &command_descriptor(SubagentCommandKind::Cancel))
                .expect("cancel descriptor should route");
        assert!(decision.accepted);
        assert!(decision.routed_to_policy);
    }

    #[test]
    fn ux_hooks_fail_closed_for_raw_logs_unbounded_detail_and_unauthorized_commands() {
        // r[verify agent_harness_runtime.subagent_ux_hooks.fail_closed]
        let mut raw = status_projection_draft();
        raw.raw_log = Some(String::from("RAW_CHILD_LOG: details"));
        assert_eq!(
            validate_subagent_status_projection(&raw)
                .expect_err("raw logs must fail")
                .kind,
            SubagentDiagnosticKind::UnsafeText
        );
        let state = running_state();
        let mut unbounded = command_descriptor(SubagentCommandKind::DetailImportRequest);
        unbounded.requested_detail_bound =
            Some(MAX_DETAIL_IMPORT_CHARS.saturating_add(MIN_POSITIVE_U32));
        assert_eq!(
            route_subagent_ux_command(&state, &unbounded)
                .expect("descriptor validates")
                .diagnostic
                .expect("denied command should explain")
                .kind,
            SubagentDiagnosticKind::OverBudget
        );
        let mut unauthorized = command_descriptor(SubagentCommandKind::Cancel);
        unauthorized.authorized = false;
        assert_eq!(
            route_subagent_ux_command(&state, &unauthorized)
                .expect("descriptor validates")
                .diagnostic
                .expect("denied command should explain")
                .kind,
            SubagentDiagnosticKind::UnauthorizedAction
        );
    }

    #[test]
    fn external_subagent_conformance_fixtures_validate_offline() {
        // r[verify agent_harness_runtime.subagent_external_conformance.positive_fixtures]
        // r[verify agent_harness_runtime.subagent_external_conformance.negative_fixtures]
        // r[verify agent_harness_runtime.subagent_external_conformance.index]
        // r[verify agent_harness_runtime.subagent_external_conformance.no_live_dependency]
        let conformance_result =
            validate_external_subagent_conformance_index(&ExternalSubagentConformanceIndex {
                fixtures: vec![valid_external_fixture(), ambient_external_fixture()],
            })
            .expect("external conformance index should validate offline");
        assert_eq!(conformance_result.results.len(), usize::from(TWO_CHILDREN));
        assert!(
            conformance_result
                .results
                .iter()
                .all(|result| result.offline_only)
        );
        assert!(
            conformance_result.results.iter().any(
                |result| result.failure == Some(SubagentDiagnosticKind::AmbientDiscoveryDenied)
            )
        );
    }

    #[test]
    fn external_subagent_conformance_rejects_bad_negative_index() {
        // r[verify agent_harness_runtime.subagent_external_conformance.verification]
        let mut fixture = ambient_external_fixture();
        fixture.expected_failure = None;
        assert_eq!(
            validate_external_subagent_fixture(&fixture)
                .expect_err("negative fixtures require expected failure")
                .kind,
            SubagentDiagnosticKind::MissingExpectedFailure
        );
        let mut undeclared = valid_external_fixture();
        undeclared
            .requested_tools
            .push(String::from("undeclared-tool"));
        undeclared.expected_status = ExternalSubagentFixtureStatus::Denied;
        undeclared.expected_failure = Some(SubagentDiagnosticKind::UndeclaredTool);
        assert_eq!(
            validate_external_subagent_fixture(&undeclared)
                .expect("undeclared tool fixture should classify")
                .failure,
            Some(SubagentDiagnosticKind::UndeclaredTool)
        );
    }

    fn catalog() -> Vec<SubagentCapabilityCatalogEntry> {
        vec![
            SubagentCapabilityCatalogEntry {
                stable_label: String::from(TOOL_CAPABILITY),
                class: SubagentCapabilityClass::ToolUse,
                target_label: String::from(TOOL_LABEL),
                workspace_mode: None,
                authorized: true,
            },
            SubagentCapabilityCatalogEntry {
                stable_label: String::from(SEARCH_CAPABILITY),
                class: SubagentCapabilityClass::Search,
                target_label: String::from(SEARCH_LABEL),
                workspace_mode: None,
                authorized: true,
            },
            SubagentCapabilityCatalogEntry {
                stable_label: String::from(MUTATION_CAPABILITY),
                class: SubagentCapabilityClass::MutationIntent,
                target_label: String::from(MUTATION_LABEL),
                workspace_mode: None,
                authorized: true,
            },
            SubagentCapabilityCatalogEntry {
                stable_label: String::from(WORKSPACE_CAPABILITY),
                class: SubagentCapabilityClass::WorkspaceAccess,
                target_label: String::from(WORKSPACE_LABEL),
                workspace_mode: Some(SubagentWorkspaceAccessMode::Write),
                authorized: true,
            },
            SubagentCapabilityCatalogEntry {
                stable_label: String::from(SERVICE_CAPABILITY),
                class: SubagentCapabilityClass::ExternalService,
                target_label: String::from(SERVICE_LABEL),
                workspace_mode: None,
                authorized: true,
            },
        ]
    }

    fn capabilities() -> Vec<TypedSubagentCapability> {
        convert_subagent_capability_labels(&catalog(), &[TOOL_CAPABILITY, SEARCH_CAPABILITY])
            .expect("capabilities convert")
            .capabilities
    }

    fn context_packet_draft(kind: SubagentChildRunKind) -> SubagentContextPacketDraft {
        SubagentContextPacketDraft {
            packet_id: String::from(PACKET_ID),
            child_kind: kind,
            admission: SubagentContextAdmissionPolicy::default(),
            items: vec![SubagentContextItemDraft {
                kind: SubagentContextItemKind::Summary,
                label: String::from(SUMMARY_LABEL),
                safe_body: String::from(SUMMARY_BODY),
                token_estimate: VALID_TOKENS,
                redaction: SubagentContextRedaction::SafeSummary,
            }],
        }
    }

    fn child_draft(kind: SubagentChildRunKind) -> SubagentChildRunDraft {
        SubagentChildRunDraft {
            child_id: String::from(CHILD_ID),
            parent_id: String::from(PARENT_ID),
            kind,
            context_packet: Some(context_packet_draft(kind)),
            capabilities: capabilities(),
            recipe: Some(RecipeSubagentDraft {
                recipe_id: String::from(RECIPE_ID),
                trust_receipt_label: String::from(TRUST_RECEIPT),
                handoff_mode: SubagentHandoffMode::StatusPlusSummary,
            }),
            fresh_context: Some(FreshContextSubagentDraft {
                objective: String::from("Review selected context"),
                scheduler_label: String::from(SCHEDULER_LABEL),
                read_only: true,
            }),
            worker_team: Some(WorkerTeamSubagentDraft {
                service_label: String::from(SERVICE_LABEL),
                output_schema_label: String::from(OUTPUT_SCHEMA),
            }),
            execution_requested: false,
        }
    }

    fn valid_admission(kind: SubagentChildRunKind) -> SubagentAdmission {
        let child = validate_subagent_child_run(&child_draft(kind)).expect("child run validates");
        admit_subagent_child_run(&child)
    }

    fn completed_report(child_id: &str) -> SubagentReport {
        SubagentReport {
            child_id: String::from(child_id),
            status: SubagentRunStatus::Completed,
            output_hash: Some(hash_parts(SAFE_HASH_DOMAIN_RECEIPT, &["child-output"])),
            evidence_refs: vec![String::from(EVIDENCE_REF)],
            mutation_intents: Vec::new(),
        }
    }

    #[derive(Default)]
    struct RecordingSubagentService {
        enqueues: usize,
    }

    impl SubagentService for RecordingSubagentService {
        fn enqueue(
            &mut self,
            _admission: &SubagentAdmission,
        ) -> Result<SubagentHandle, SubagentServiceFailure> {
            self.enqueues += EXPECTED_ONE;
            SubagentHandle::new(HANDLE_ID).map_err(|diagnostic| SubagentServiceFailure {
                kind: SubagentServiceFailureKind::MalformedReport,
                diagnostic,
            })
        }

        fn status(
            &mut self,
            _handle: &SubagentHandle,
        ) -> Result<SubagentRunStatus, SubagentServiceFailure> {
            Ok(SubagentRunStatus::Running)
        }

        fn cancel(
            &mut self,
            _handle: &SubagentHandle,
        ) -> Result<SubagentRunStatus, SubagentServiceFailure> {
            Ok(SubagentRunStatus::Cancelling)
        }

        fn collect(
            &mut self,
            _handle: &SubagentHandle,
        ) -> Result<SubagentReport, SubagentServiceFailure> {
            Ok(completed_report(CHILD_ID))
        }
    }

    fn running_state() -> SubagentRunState {
        SubagentRunState {
            admission: valid_admission(SubagentChildRunKind::WorkerTeamTask),
            status: SubagentRunStatus::Running,
            handle: Some(SubagentHandle::new(HANDLE_ID).expect("valid handle")),
            report: None,
            failure_class: None,
        }
    }

    fn collected_state() -> SubagentRunState {
        SubagentRunState {
            report: Some(completed_report(CHILD_ID)),
            status: SubagentRunStatus::Collected,
            ..running_state()
        }
    }

    fn fanout_plan_draft() -> SubagentFanOutPlanDraft {
        let first = valid_admission(SubagentChildRunKind::RecipeBacked);
        let mut second = valid_admission(SubagentChildRunKind::WorkerTeamTask);
        second.child_id = String::from("child-worker");
        SubagentFanOutPlanDraft {
            plan_id: String::from(PLAN_ID),
            children: vec![first, second],
            max_concurrency: TWO_CHILDREN,
            budget: SubagentFanOutBudget {
                max_children: TWO_CHILDREN,
                max_tokens: MAX_PACKET_TOKENS,
            },
            cancellation_policy: SubagentCancellationPolicy::CancelRemainingOnFailure,
            aggregation_schema: String::from(OUTPUT_SCHEMA),
            mutation_policy: SubagentMutationPolicy::CollectUnresolved,
        }
    }

    fn successful_outcome(
        child_id: &str,
        mutation_intents: Vec<SubagentMutationIntentSummary>,
    ) -> SubagentFanInOutcome {
        SubagentFanInOutcome {
            child_id: String::from(child_id),
            status: SubagentRunStatus::Completed,
            report: Some(SubagentReport {
                mutation_intents: mutation_intents.clone(),
                ..completed_report(child_id)
            }),
            budget_used: VALID_TOKENS,
            mutation_intents,
        }
    }

    fn valid_mutation() -> SubagentMutationIntentSummary {
        SubagentMutationIntentSummary {
            target: String::from(MUTATION_TARGET),
            mutation_kind: String::from(MUTATION_LABEL),
            evidence_ref: String::from(EVIDENCE_REF),
        }
    }

    fn status_projection_draft() -> SubagentStatusProjectionDraft {
        SubagentStatusProjectionDraft {
            child_id: String::from(CHILD_ID),
            parent_id: String::from(PARENT_ID),
            kind: SubagentChildRunKind::WorkerTeamTask,
            status: SubagentRunStatus::Running,
            budget: SubagentContextBudgetSummary {
                item_count: MIN_POSITIVE_U16,
                token_estimate: VALID_TOKENS,
            },
            denial_reason: Some(String::from("capability policy pending")),
            handoff_summary_ref: Some(String::from(EVIDENCE_REF)),
            evidence_ref: Some(String::from(EVIDENCE_REF)),
            pending_mutations: vec![valid_mutation()],
            raw_log: None,
        }
    }

    fn command_descriptor(kind: SubagentCommandKind) -> SubagentCommandDescriptor {
        SubagentCommandDescriptor {
            command_id: String::from(COMMAND_ID),
            child_id: String::from(CHILD_ID),
            kind,
            requested_detail_bound: Some(MAX_DETAIL_IMPORT_CHARS),
            handoff_mode: Some(SubagentHandoffMode::FullDetailBounded),
            reason: String::from("operator requested bounded detail"),
            authorized: true,
            handle_fresh: true,
            mutation_target: Some(String::from(MUTATION_TARGET)),
        }
    }

    fn valid_external_fixture() -> ExternalSubagentFixture {
        ExternalSubagentFixture {
            fixture_id: String::from("mcp-positive"),
            protocol: ExternalSubagentProtocolKind::Mcp,
            identity: String::from("mcp-reviewer"),
            service_label: String::from(SERVICE_LABEL),
            declared_tools: vec![String::from(TOOL_LABEL)],
            requested_tools: vec![String::from(TOOL_LABEL)],
            credential_label: Some(String::from("cred-ref")),
            ambient_cli: false,
            inherited_credentials: false,
            unsafe_output_import: false,
            capability_claims: vec![String::from(TOOL_CAPABILITY)],
            expected_status: ExternalSubagentFixtureStatus::Accepted,
            expected_failure: None,
            covered_requirements: vec![String::from(
                "agent_harness_runtime.subagent_external_conformance.positive_fixtures",
            )],
        }
    }

    fn ambient_external_fixture() -> ExternalSubagentFixture {
        ExternalSubagentFixture {
            fixture_id: String::from("acp-ambient-denied"),
            protocol: ExternalSubagentProtocolKind::Acp,
            identity: String::from("acp-reviewer"),
            service_label: String::from(SERVICE_LABEL),
            declared_tools: vec![String::from(TOOL_LABEL)],
            requested_tools: vec![String::from(TOOL_LABEL)],
            credential_label: None,
            ambient_cli: true,
            inherited_credentials: false,
            unsafe_output_import: false,
            capability_claims: vec![String::from(TOOL_CAPABILITY)],
            expected_status: ExternalSubagentFixtureStatus::Denied,
            expected_failure: Some(SubagentDiagnosticKind::AmbientDiscoveryDenied),
            covered_requirements: vec![String::from(
                "agent_harness_runtime.subagent_external_conformance.negative_fixtures",
            )],
        }
    }
}
