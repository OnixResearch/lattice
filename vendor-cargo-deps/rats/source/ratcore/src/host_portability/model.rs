//! Nominal identities and plain host-portability policy values.

semantic_string_id!(
    /// Stable host-family identity.
    pub struct HostFamilyId;
);
semantic_string_id!(
    /// Exact Rust target-triple identity.
    pub struct RustTargetId;
);
semantic_string_id!(
    /// Stable target-scope identity used by inventory rows.
    pub struct HostTargetScopeId;
);
semantic_string_id!(
    /// Stable narrow capability identity.
    pub struct HostCapabilityId;
);
semantic_string_id!(
    /// Stable package/source-surface identity.
    pub struct PortabilitySurfaceId;
);
semantic_string_id!(
    /// Stable ambient-effect coordinate identity.
    pub struct HostEffectId;
);

/// Host-independent value scope.
pub const HOST_FAMILY_INDEPENDENT: &str = "host-independent";
/// Native operating-system host family.
pub const HOST_FAMILY_NATIVE: &str = "native-os";
/// Browser host family.
pub const HOST_FAMILY_BROWSER: &str = "browser";
/// Linux GNU target used as one exact native target identity.
pub const TARGET_X86_64_LINUX_GNU: &str = "x86_64-unknown-linux-gnu";
/// Windows MSVC target used as one exact native target identity.
pub const TARGET_X86_64_WINDOWS_MSVC: &str = "x86_64-pc-windows-msvc";
/// Apple Silicon macOS target used as one exact native target identity.
pub const TARGET_AARCH64_APPLE_DARWIN: &str = "aarch64-apple-darwin";
/// Browser WASM target used as one exact browser target identity.
pub const TARGET_WASM_BROWSER: &str = "wasm32-unknown-unknown";
/// Classification-only scope for host-independent values.
pub const SCOPE_HOST_INDEPENDENT: &str = "scope.host-independent-values";
/// Classification-only scope where no native triple is claimed.
pub const SCOPE_NATIVE_UNCLAIMED: &str = "scope.native-unclaimed";
/// Classification-only scope delegated to a renderer host.
pub const SCOPE_RENDERER_MANAGED: &str = "scope.renderer-managed-unclaimed";
/// Exact browser-WASM implementation scope, without a support claim.
pub const SCOPE_BROWSER_WASM: &str = "scope.browser-wasm-implementation";
/// Exact Linux implementation/evidence scope, without cross-OS promotion.
pub const SCOPE_LINUX_X86_64: &str = "scope.linux-x86_64-implementation";

/// One host-family descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFamilyDescriptor {
    /// Stable nominal identity.
    pub id: HostFamilyId,
    /// Human-readable label.
    pub label: String,
}

impl HostFamilyDescriptor {
    /// Creates a host-family descriptor.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: HostFamilyId::new(id),
            label: label.into(),
        }
    }
}

/// One exact Rust target and its host family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTargetDescriptor {
    /// Exact target triple.
    pub id: RustTargetId,
    /// Architecture token; receipts never widen across this boundary.
    pub architecture: String,
    /// Host-family identity.
    pub host_family: HostFamilyId,
}

impl RustTargetDescriptor {
    /// Creates an exact-target descriptor.
    #[must_use]
    pub fn new(id: impl Into<String>, architecture: impl Into<String>, host_family: impl Into<String>) -> Self {
        Self {
            id: RustTargetId::new(id),
            architecture: architecture.into(),
            host_family: HostFamilyId::new(host_family),
        }
    }
}

/// Kind of scope attached to an inventory row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostTargetScopeKind {
    /// Classification only; no compile or runtime support is claimed.
    ClassificationOnly,
    /// One exact Rust target, still requiring separate evidence.
    ExactTarget,
}

impl HostTargetScopeKind {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClassificationOnly => "classification-only",
            Self::ExactTarget => "exact-target",
        }
    }
}

/// Target scope attached to package, capability, and effect rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostTargetScope {
    /// Stable scope identity.
    pub id: HostTargetScopeId,
    /// Host family classified by the scope.
    pub host_family: HostFamilyId,
    /// Exact target when this is an exact-target scope.
    pub rust_target: Option<RustTargetId>,
    /// Whether the scope is exact or classification-only.
    pub kind: HostTargetScopeKind,
}

impl HostTargetScope {
    /// Creates a classification-only scope.
    #[must_use]
    pub fn classification(id: impl Into<String>, host_family: impl Into<String>) -> Self {
        Self {
            id: HostTargetScopeId::new(id),
            host_family: HostFamilyId::new(host_family),
            rust_target: None,
            kind: HostTargetScopeKind::ClassificationOnly,
        }
    }

    /// Creates one exact-target scope.
    #[must_use]
    pub fn exact(id: impl Into<String>, host_family: impl Into<String>, rust_target: impl Into<String>) -> Self {
        Self {
            id: HostTargetScopeId::new(id),
            host_family: HostFamilyId::new(host_family),
            rust_target: Some(RustTargetId::new(rust_target)),
            kind: HostTargetScopeKind::ExactTarget,
        }
    }
}

/// Ownership role for one source surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortabilityRole {
    /// Pure renderer- and host-independent logic.
    PureCore,
    /// Reusable renderer/component library.
    ReusableLibrary,
    /// Narrow concrete backend or host adapter.
    BackendAdapter,
    /// Application-owned process/runtime shell.
    ApplicationShell,
    /// Tests, examples, build scripts, and evidence capture.
    EvidenceHarness,
    /// Malformed seed value; never valid in checked inventory.
    Unknown,
}

impl PortabilityRole {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PureCore => "pure-core",
            Self::ReusableLibrary => "reusable-library",
            Self::BackendAdapter => "backend-adapter",
            Self::ApplicationShell => "application-shell",
            Self::EvidenceHarness => "evidence-harness",
            Self::Unknown => "unknown",
        }
    }

    /// Returns whether this role may directly acquire named host effects.
    #[must_use]
    pub const fn may_acquire_host_effects(self) -> bool {
        matches!(self, Self::BackendAdapter | Self::ApplicationShell | Self::EvidenceHarness)
    }
}

/// Source/build surface represented by an inventory row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SourceSurface {
    /// Ordinary production source.
    Production,
    /// Source enabled by a named Cargo feature.
    OptionalFeature,
    /// Cargo example target.
    Example,
    /// Test or test-only source.
    Test,
    /// Cargo build script.
    BuildScript,
    /// Private receipt, capture, or evidence tool.
    Evidence,
}

impl SourceSurface {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::OptionalFeature => "optional-feature",
            Self::Example => "example",
            Self::Test => "test",
            Self::BuildScript => "build-script",
            Self::Evidence => "evidence",
        }
    }
}

/// Relationship between a source slice and package default features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DefaultFeatureDisposition {
    /// Included in the package's ordinary default build.
    Included,
    /// Enabled by a feature that is itself in the default set.
    DefaultEnabledFeature,
    /// Optional and excluded from package defaults.
    Excluded,
    /// Test/example/build/evidence surface rather than a default feature.
    NotApplicable,
}

impl DefaultFeatureDisposition {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::DefaultEnabledFeature => "default-enabled-feature",
            Self::Excluded => "excluded",
            Self::NotApplicable => "n/a",
        }
    }
}

/// Migration/conformance state of one classified row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortabilityDisposition {
    /// Conforms to the role boundary; this is not itself support evidence.
    Conforming,
    /// Fully classified but non-passing until its named migration lands.
    MigrationBlocker,
    /// Deliberately excluded evidence or experiment.
    EvidenceOnly,
}

impl PortabilityDisposition {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Conforming => "conforming",
            Self::MigrationBlocker => "migration-blocker",
            Self::EvidenceOnly => "evidence-only",
        }
    }
}

/// One package or finer source-slice classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSurface {
    /// Stable row identity.
    pub id: PortabilitySurfaceId,
    /// Cargo package name.
    pub package: String,
    /// Workspace-relative manifest or source path.
    pub path: String,
    /// Canonical maintenance/effect owner.
    pub owner: String,
    /// Ownership role.
    pub role: PortabilityRole,
    /// Source/build surface.
    pub surface: SourceSurface,
    /// Target-scope identity.
    pub target_scope: HostTargetScopeId,
    /// Default-feature relationship.
    pub defaults: DefaultFeatureDisposition,
    /// Sorted package default feature names (`-` when empty).
    pub default_features: String,
    /// Current conformance/migration state.
    pub disposition: PortabilityDisposition,
    /// Dependent Cairn for a migration blocker.
    pub dependent_change: Option<String>,
}

/// Ambient host/effect class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostEffectKind {
    /// Filesystem read or write.
    Filesystem,
    /// Process environment or argument acquisition.
    Environment,
    /// Child-process launch or process termination.
    Process,
    /// Network acquisition.
    Network,
    /// Clipboard mechanism.
    Clipboard,
    /// Terminal device, process stream, lifecycle, or geometry acquisition.
    TerminalDevice,
    /// Browser global or Web API acquisition.
    BrowserGlobal,
    /// Ambient clock read.
    Clock,
    /// Thread or executor acquisition.
    ThreadExecutor,
    /// Concrete backend event identity.
    BackendEvent,
    /// Host path syntax or opaque path-value semantics.
    PathSemantics,
    /// Target-specific dependency or backend feature.
    RuntimeDependency,
    /// A target-specific conditional branch.
    TargetBranch,
}

impl HostEffectKind {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Environment => "environment",
            Self::Process => "process",
            Self::Network => "network",
            Self::Clipboard => "clipboard",
            Self::TerminalDevice => "terminal-device",
            Self::BrowserGlobal => "browser-global",
            Self::Clock => "clock",
            Self::ThreadExecutor => "thread-executor",
            Self::BackendEvent => "backend-event",
            Self::PathSemantics => "path-semantic",
            Self::RuntimeDependency => "runtime-dependency",
            Self::TargetBranch => "target-branch",
        }
    }
}

/// Direction of a named effect coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectDirection {
    /// Read/observe data from the host.
    Read,
    /// Write/emit data to the host.
    Write,
    /// Both read and write.
    ReadWrite,
    /// Consume a concrete host/backend event value.
    Input,
    /// Acquire a dependency or runtime feature at build time.
    Dependency,
}

impl EffectDirection {
    /// Returns the stable generated-inventory spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::ReadWrite => "read-write",
            Self::Input => "input",
            Self::Dependency => "dependency",
        }
    }
}

/// Source scanner class for one checked coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectScanClass {
    /// Filesystem scanner.
    Filesystem,
    /// Environment scanner.
    Environment,
    /// Process scanner.
    Process,
    /// Network scanner.
    Network,
    /// Clipboard scanner.
    Clipboard,
    /// Terminal scanner.
    TerminalDevice,
    /// Browser-global scanner.
    BrowserGlobal,
    /// Clock-read scanner.
    Clock,
    /// Thread/executor scanner.
    ThreadExecutor,
    /// Backend-event scanner.
    BackendEvent,
    /// Path-semantics scanner.
    PathSemantics,
    /// Target-conditional branch scanner.
    TargetBranch,
    /// Manifest/dependency checker rather than source scanner.
    Dependency,
}

impl EffectScanClass {
    /// Returns the stable scanner spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Environment => "environment",
            Self::Process => "process",
            Self::Network => "network",
            Self::Clipboard => "clipboard",
            Self::TerminalDevice => "terminal-device",
            Self::BrowserGlobal => "browser-global",
            Self::Clock => "clock",
            Self::ThreadExecutor => "thread-executor",
            Self::BackendEvent => "backend-event",
            Self::PathSemantics => "path-semantic",
            Self::TargetBranch => "target-branch",
            Self::Dependency => "dependency",
        }
    }
}

/// One classified ambient-effect or dependency coordinate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEffectCoordinate {
    /// Stable coordinate identity.
    pub id: HostEffectId,
    /// Cargo package name.
    pub package: String,
    /// Workspace-relative source or manifest path.
    pub path: String,
    /// Source/build surface.
    pub surface: SourceSurface,
    /// Effect class.
    pub kind: HostEffectKind,
    /// Effect direction.
    pub direction: EffectDirection,
    /// Current canonical owner.
    pub owner: String,
    /// Current role at this coordinate.
    pub role: PortabilityRole,
    /// Target scope.
    pub target_scope: HostTargetScopeId,
    /// Default-feature relationship.
    pub defaults: DefaultFeatureDisposition,
    /// Narrow capability identity.
    pub capability: HostCapabilityId,
    /// Stable diagnostics policy.
    pub diagnostics: String,
    /// Explicit unsupported/failure fallback policy.
    pub fallback: String,
    /// Current conformance/migration state.
    pub disposition: PortabilityDisposition,
    /// Dependent Cairn for a migration blocker.
    pub dependent_change: Option<String>,
    /// Scanner class used by the source rail.
    pub scan_class: EffectScanClass,
    /// Number of non-comment scanner matches in this path.
    pub scan_match_count: usize,
}

/// Kind of state retained by a capability boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityStateKind {
    /// Plain snapshots, facts, requests, diagnostics, and outcomes only.
    PlainValues,
    /// A retained callback; forbidden in reusable authoritative state.
    HostCallback,
    /// A retained host/runtime handle; forbidden in reusable state.
    HostHandle,
    /// A retained executor/runtime; forbidden in reusable state.
    Executor,
    /// A retained concrete backend event; forbidden in reusable state.
    BackendEvent,
}

/// Authority class attached to a capability boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityAuthority {
    /// Rats/application is the canonical owner.
    Rats,
    /// Calling application owns current state and policy.
    Caller,
    /// An external service/daemon owns current state and policy.
    External,
}

/// Presence of one current authority fact at the final effect edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityAttachmentState {
    /// Current fact is attached.
    Attached,
    /// Current fact is missing.
    Missing,
}

/// Current authority context attached before an effect is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorityAttachments {
    /// Current external identity.
    pub identity: AuthorityAttachmentState,
    /// Current generation or revision.
    pub generation: AuthorityAttachmentState,
    /// Current capability facts.
    pub capabilities: AuthorityAttachmentState,
    /// Current policy and authorization.
    pub authorization: AuthorityAttachmentState,
}

impl AuthorityAttachments {
    /// Complete current authority context.
    pub const COMPLETE: Self = Self {
        identity: AuthorityAttachmentState::Attached,
        generation: AuthorityAttachmentState::Attached,
        capabilities: AuthorityAttachmentState::Attached,
        authorization: AuthorityAttachmentState::Attached,
    };

    /// Returns whether all required current facts are attached.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self.identity, AuthorityAttachmentState::Attached)
            && matches!(self.generation, AuthorityAttachmentState::Attached)
            && matches!(self.capabilities, AuthorityAttachmentState::Attached)
            && matches!(self.authorization, AuthorityAttachmentState::Attached)
    }
}

/// One narrow capability ownership contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCapabilityContract {
    /// Stable feature-specific identity.
    pub id: HostCapabilityId,
    /// Canonical effect/final-validation owner.
    pub owner: String,
    /// Owning source role.
    pub role: PortabilityRole,
    /// Target scope.
    pub target_scope: HostTargetScopeId,
    /// Unsupported/denied diagnostic policy.
    pub diagnostics: String,
    /// Explicit fallback that performs no inferred alternate mechanism.
    pub fallback: String,
    /// Kind of state retained at the reusable boundary.
    pub state_kind: CapabilityStateKind,
    /// Canonical authority class.
    pub authority: CapabilityAuthority,
    /// Current identity/generation/capability/authorization attachments.
    pub attachments: AuthorityAttachments,
    /// Current conformance/migration state.
    pub disposition: PortabilityDisposition,
}

/// Availability fact supplied to reusable logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CapabilityAvailability {
    /// Shell can evaluate the named request.
    Available,
    /// Host/shell does not implement it.
    Unsupported,
    /// Current policy or permission denies it.
    Denied,
}

/// Feature-specific capability fact supplied by a caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostCapabilityFact {
    /// Narrow capability identity.
    pub capability: HostCapabilityId,
    /// Current availability.
    pub availability: CapabilityAvailability,
}

/// Typed host-operation result returned by the owning adapter/shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostOperationOutcome<T> {
    /// Operation completed with a typed value.
    Completed(T),
    /// Capability is unsupported; no alternate mechanism was inferred.
    Unsupported {
        /// Stable diagnostic code/text.
        diagnostic: String,
    },
    /// Capability was rejected or denied; no effect was executed.
    Rejected {
        /// Stable diagnostic code/text.
        diagnostic: String,
    },
    /// Named mechanism failed after explicit selection.
    Failed {
        /// Stable content-free diagnostic code/text.
        diagnostic: String,
    },
}
