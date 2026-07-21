// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
/// Context file class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerClass {
    /// Agent guidance such as AGENTS-style instructions.
    AgentGuidance,
    /// Architecture guidance.
    Architecture,
    /// Project policy or notes.
    ProjectNote,
    /// Host-provided context.
    HostNote,
    /// Unknown class fixture.
    Unknown,
}

/// Source identity class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerSourceKind {
    /// Global host context.
    Global,
    /// Ancestor-discovered project context.
    Ancestor,
    /// Current project context.
    Project,
    /// Host-injected context.
    Host,
}

/// Descriptor trust state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerTrust {
    /// Trusted for prompt influence.
    Trusted,
    /// Not trusted for prompt influence.
    Untrusted,
    /// Unknown trust state.
    Unknown,
}

/// Freshness state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerFreshness {
    /// Fresh context.
    Fresh,
    /// Stale context.
    Stale,
    /// Unknown freshness.
    Unknown,
}

/// Redaction state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerRedaction {
    /// Safe body.
    Safe,
    /// Body already redacted.
    Redacted,
    /// Unsafe body.
    Unsafe,
}

/// Prompt inclusion policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerInclusionPolicy {
    /// Descriptor may be included after admission.
    Enabled,
    /// Descriptor is disabled.
    Disabled,
}

/// In-memory context descriptor supplied by a host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerDescriptor {
    /// Stable descriptor ID.
    pub descriptor_id: String,
    /// Context class.
    pub class: ContextLayerClass,
    /// Safe source identity label or hash.
    pub source_identity: String,
    /// Source kind.
    pub source_kind: ContextLayerSourceKind,
    /// Trust state.
    pub trust: ContextLayerTrust,
    /// Freshness state.
    pub freshness: ContextLayerFreshness,
    /// Declared body size in characters.
    pub size_chars: usize,
    /// Redaction state.
    pub redaction: ContextLayerRedaction,
    /// Inclusion policy.
    pub inclusion: ContextLayerInclusionPolicy,
    /// Higher values win within the same prompt section.
    pub precedence: u32,
    /// Prompt section key.
    pub section: String,
    /// Already-loaded body supplied by the host.
    pub body: String,
}

/// Policy switch for admission options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerPolicySwitch {
    /// Option is disabled.
    Disabled,
    /// Option is enabled.
    Enabled,
}

/// Boundary effect state for no-I/O proof fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerBoundaryEffect {
    /// Effect was not performed.
    NotPerformed,
    /// Effect was performed.
    Performed,
}

/// Admission policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerAdmissionPolicy {
    /// Discovery host policy.
    pub discovery: ContextLayerPolicySwitch,
    /// Maximum model-visible context body characters.
    pub max_body_chars: usize,
    /// Stale-context inclusion policy.
    pub stale_context: ContextLayerPolicySwitch,
    /// Unknown-freshness inclusion policy.
    pub unknown_freshness: ContextLayerPolicySwitch,
    /// Untrusted-context inclusion policy.
    pub untrusted_context: ContextLayerPolicySwitch,
    /// Conflict diagnostic policy.
    pub conflict_diagnostics: ContextLayerPolicySwitch,
}

/// Proof boundary that prompt assembly performed no ambient discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContextLayerDiscoveryBoundary {
    /// Ancestor walk effect state.
    pub ancestor_walk: ContextLayerBoundaryEffect,
    /// File read effect state.
    pub file_read: ContextLayerBoundaryEffect,
    /// Environment read effect state.
    pub environment_read: ContextLayerBoundaryEffect,
    /// Path canonicalization effect state.
    pub path_canonicalization: ContextLayerBoundaryEffect,
}

/// Receipt entry for included or omitted descriptors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerReceiptEntry {
    /// Descriptor ID.
    pub descriptor_id: String,
    /// Context class.
    pub class: ContextLayerClass,
    /// Source kind.
    pub source_kind: ContextLayerSourceKind,
    /// Safe source identity.
    pub source_identity: String,
    /// BLAKE3 body hash.
    pub body_hash: String,
    /// Omission or inclusion reason.
    pub reason: String,
}

/// Context layering receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerReceipt {
    /// Included descriptors.
    pub included: Vec<ContextLayerReceiptEntry>,
    /// Omitted descriptors.
    pub omitted: Vec<ContextLayerReceiptEntry>,
}

/// Context layering plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerPlan {
    /// Model-visible prompt sections.
    pub prompt_sections: Vec<ContextLayerReceiptEntry>,
    /// Safe receipt.
    pub receipt: ContextLayerReceipt,
    /// Discovery boundary proof.
    pub discovery_boundary: ContextLayerDiscoveryBoundary,
    /// Diagnostics.
    pub diagnostics: Vec<ContextLayerDiagnostic>,
}

/// Context layering diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextLayerDiagnosticKind {
    /// Unknown class.
    UnknownClass,
    /// Missing source.
    MissingSource,
    /// Discovery disabled.
    DiscoveryDisabled,
    /// Context is untrusted.
    Untrusted,
    /// Context is stale.
    Stale,
    /// Context is oversized.
    Oversized,
    /// Context has unsafe redaction/body markers.
    Unsafe,
    /// Lower precedence conflict.
    Conflict,
}

/// Context layering diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextLayerDiagnostic {
    /// Diagnostic class.
    pub kind: ContextLayerDiagnosticKind,
    /// Stable field label.
    pub field: String,
    /// Safe detail.
    pub detail: String,
}

impl ContextLayerDiagnostic {
    /// Builds a context layering diagnostic.
    #[must_use]
    pub fn new(
        kind: ContextLayerDiagnosticKind,
        field: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Context descriptor validation result.
pub type ContextLayerValidationResult = Result<(), Vec<ContextLayerDiagnostic>>;
