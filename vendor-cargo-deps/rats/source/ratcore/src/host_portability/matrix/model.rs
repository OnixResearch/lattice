//! Plain nominal portability-matrix values.

semantic_string_id!(
    /// Stable package-profile identity.
    pub struct PortabilityProfileId;
);
semantic_string_id!(
    /// Stable direct-host command identity.
    pub struct PortabilityCommandId;
);
semantic_string_id!(
    /// Stable runner-host scope, distinct from compiled target identity.
    pub struct RunnerHostScopeId;
);

/// Checked receipt schema.
pub const MATRIX_RECEIPT_SCHEMA: &str = "rats.host-portability-receipt.v1";
/// Checked matrix schema.
pub const MATRIX_SCHEMA: &str = "rats.host-portability-matrix.v1";
/// Maximum bounded diagnostic code length.
pub const MAX_MATRIX_DIAGNOSTIC_BYTES: usize = 96;
/// Fixed portability receipt non-claims.
pub const MATRIX_NON_CLAIMS: &str =
    "not-all-os;not-other-architectures;not-release;not-downstream;not-security;not-accessibility";

/// Role of one generated package profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortabilityProfileRole {
    /// Ratcore pure logic.
    PureCore,
    /// Reusable Subwayrat package set.
    ReusableSubwayrat,
    /// Native terminal adapter/application shell.
    NativeTerminal,
    /// Ratzilla WASM/browser shell.
    RatzillaWeb,
    /// Meteorite WASM/browser runtime slice.
    MeteoriteWeb,
    /// Reusable Meteorite component-library WASM package set.
    MeteoriteComponents,
    /// Mounted production Meteorite showcase application.
    MeteoriteShowcaseWeb,
    /// Experimental packages that cannot satisfy maintained cells.
    Experimental,
    /// Evidence-only packages and slices.
    EvidenceOnly,
}

impl PortabilityProfileRole {
    /// Stable generated spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PureCore => "pure-core",
            Self::ReusableSubwayrat => "reusable-subwayrat",
            Self::NativeTerminal => "native-terminal",
            Self::RatzillaWeb => "ratzilla-web",
            Self::MeteoriteWeb => "meteorite-web",
            Self::MeteoriteComponents => "meteorite-components-wasm",
            Self::MeteoriteShowcaseWeb => "meteorite-showcase-web",
            Self::Experimental => "experimental",
            Self::EvidenceOnly => "evidence-only",
        }
    }
}

/// One ordered profile member derived from package inventory.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortabilityProfilePackage {
    /// Cargo package name.
    pub package: String,
    /// Sorted default feature identity.
    pub default_features: String,
    /// Inventory role.
    pub role: super::super::PortabilityRole,
    /// Inventory target scope.
    pub target_scope: super::super::HostTargetScopeId,
}

/// One generated package profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityPackageProfile {
    /// Stable profile identity.
    pub id: PortabilityProfileId,
    /// Profile role.
    pub role: PortabilityProfileRole,
    /// Sorted deduplicated members.
    pub packages: Vec<PortabilityProfilePackage>,
    /// BLAKE3 over role, members, defaults, roles, and scopes.
    pub package_set_hash: String,
}

/// One required evidence grade for an exact profile/target cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityMatrixRequirement {
    /// Stable requirement identity.
    pub id: String,
    /// Package profile.
    pub profile: PortabilityProfileId,
    /// Exact Rust target.
    pub target: super::super::RustTargetId,
    /// Required evidence grade.
    pub grade: super::super::HostEvidenceGrade,
    /// Stable repository-owned command identity.
    pub command: PortabilityCommandId,
    /// BLAKE3 over the normalized command implementation.
    pub command_hash: String,
    /// Whether cross compile may provide partial compile evidence.
    pub allows_cross_compile: bool,
}

/// Construction values for one normalized receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityReceiptParts {
    /// Stable receipt identity.
    pub id: String,
    /// Profile identity.
    pub profile: PortabilityProfileId,
    /// Exact compiled/executed target.
    pub target: super::super::RustTargetId,
    /// Actual runner scope, separate from browser target.
    pub runner_host: RunnerHostScopeId,
    /// Evidence grade.
    pub grade: super::super::HostEvidenceGrade,
    /// Direct, cross, simulated, test-backend, semantic, visual, or experimental method.
    pub method: super::super::HostEvidenceMethod,
    /// Passed/failed/blocked/skipped/unsupported result.
    pub state: super::super::HostEvidenceState,
    /// Source revision token.
    pub source_revision: String,
    /// Hash of bounded source/profile freshness inputs.
    pub source_hash: String,
    /// Expected package-profile hash.
    pub package_set_hash: String,
    /// Repository command identity.
    pub command: PortabilityCommandId,
    /// Hash of the repository command implementation.
    pub command_hash: String,
    /// Normalized toolchain identity.
    pub toolchain: String,
    /// Recorded freshness at receipt generation.
    pub freshness: super::super::EvidenceFreshness,
    /// Content-free bounded diagnostic code or `-`.
    pub diagnostic: String,
    /// Required fixed non-claims.
    pub non_claims: String,
}

/// One bounded checked portability receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityReceipt {
    /// Receipt schema.
    pub schema: String,
    /// Stable receipt identity.
    pub id: String,
    /// Profile identity.
    pub profile: PortabilityProfileId,
    /// Exact compiled/executed target.
    pub target: super::super::RustTargetId,
    /// Actual runner scope.
    pub runner_host: RunnerHostScopeId,
    /// Evidence grade.
    pub grade: super::super::HostEvidenceGrade,
    /// Evidence method.
    pub method: super::super::HostEvidenceMethod,
    /// Result state.
    pub state: super::super::HostEvidenceState,
    /// Source revision token.
    pub source_revision: String,
    /// Current source input hash.
    pub source_hash: String,
    /// Package-profile hash.
    pub package_set_hash: String,
    /// Repository command identity.
    pub command: PortabilityCommandId,
    /// Repository command hash.
    pub command_hash: String,
    /// Normalized toolchain identity.
    pub toolchain: String,
    /// Recorded freshness at receipt generation.
    pub freshness: super::super::EvidenceFreshness,
    /// Content-free diagnostic code.
    pub diagnostic: String,
    /// Fixed non-claims.
    pub non_claims: String,
    /// Nominal binding hash preventing relabel/copy promotion.
    pub binding_hash: String,
}

impl PortabilityReceipt {
    /// Creates a receipt and computes its nominal binding hash.
    #[must_use]
    pub fn new(parts: PortabilityReceiptParts) -> Self {
        let mut receipt = Self {
            schema: MATRIX_RECEIPT_SCHEMA.to_string(),
            id: parts.id,
            profile: parts.profile,
            target: parts.target,
            runner_host: parts.runner_host,
            grade: parts.grade,
            method: parts.method,
            state: parts.state,
            source_revision: parts.source_revision,
            source_hash: parts.source_hash,
            package_set_hash: parts.package_set_hash,
            command: parts.command,
            command_hash: parts.command_hash,
            toolchain: parts.toolchain,
            freshness: parts.freshness,
            diagnostic: parts.diagnostic,
            non_claims: parts.non_claims,
            binding_hash: String::new(),
        };
        receipt.binding_hash = super::receipt_binding_hash(&receipt);
        receipt
    }
}

/// Fail-closed aggregate status for one profile/target cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortabilityMatrixStatus {
    /// Every role-required grade has direct fresh passing evidence.
    Supported,
    /// Some valid evidence exists but required grades are incomplete.
    Partial,
    /// Scope is explicitly unsupported.
    Unsupported,
    /// Required execution is blocked or skipped.
    Blocked,
    /// Required receipt is absent.
    Missing,
    /// Matching direct evidence failed.
    Failed,
    /// Matching evidence is stale.
    Stale,
    /// Only cross-compiled evidence exists.
    CrossOnly,
    /// Cell belongs only to an experimental package profile.
    Experimental,
    /// Profile/target pairing is not meaningful.
    Irrelevant,
}

impl PortabilityMatrixStatus {
    /// Stable matrix spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Partial => "partial",
            Self::Unsupported => "unsupported",
            Self::Blocked => "blocked",
            Self::Missing => "missing",
            Self::Failed => "failed",
            Self::Stale => "stale",
            Self::CrossOnly => "cross-only",
            Self::Experimental => "experimental",
            Self::Irrelevant => "irrelevant",
        }
    }

    /// Whether this status can satisfy a maintained required cell.
    #[must_use]
    pub const fn is_passing(self) -> bool {
        matches!(self, Self::Supported | Self::Irrelevant)
    }
}

/// One aggregate profile/target matrix cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortabilityMatrixCell {
    /// Profile identity.
    pub profile: PortabilityProfileId,
    /// Exact target identity.
    pub target: super::super::RustTargetId,
    /// Aggregate status.
    pub status: PortabilityMatrixStatus,
    /// Required evidence grades.
    pub required_grades: Vec<super::super::HostEvidenceGrade>,
    /// Stable bounded reason code.
    pub reason: String,
}

/// Stable matrix/receipt issue code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PortabilityMatrixIssueCode {
    /// Required value is empty.
    EmptyField,
    /// Identity appears more than once.
    DuplicateIdentity,
    /// Referenced profile is unknown.
    UnknownProfile,
    /// Referenced target is unknown.
    UnknownTarget,
    /// Package membership is stale or malformed.
    PackageSetMismatch,
    /// Receipt source revision/hash is stale.
    StaleSource,
    /// Command identity/hash is stale or mismatched.
    CommandMismatch,
    /// Runner host cannot satisfy the exact direct native target.
    WrongRunnerHost,
    /// Receipt binding hash does not match nominal fields.
    BindingHashMismatch,
    /// Receipt contains raw, path-shaped, secret-shaped, or oversized diagnostics.
    UnsafeDiagnostic,
    /// Receipt schema or non-claims are malformed.
    MalformedReceipt,
    /// Duplicate receipt competes for one exact grade.
    DuplicateReceipt,
}

/// Bounded deterministic matrix diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PortabilityMatrixIssue {
    /// Stable issue code.
    pub code: PortabilityMatrixIssueCode,
    /// Stable subject identity.
    pub subject: String,
    /// Content-free message.
    pub message: String,
}

/// Receipt validation context supplied by a host-owned gate.
#[derive(Debug, Clone, Copy)]
pub struct PortabilityReceiptContext<'a> {
    /// Current gate revision; validated as provenance but not used instead of content freshness.
    pub source_revision: &'a str,
    /// Current bounded source hash; this is authoritative for receipt freshness.
    pub source_hash: &'a str,
    /// Generated profiles.
    pub profiles: &'a [PortabilityPackageProfile],
    /// Generated requirements.
    pub requirements: &'a [PortabilityMatrixRequirement],
    /// Known exact targets.
    pub targets: &'a [super::super::RustTargetDescriptor],
}
