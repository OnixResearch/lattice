//! Exact-scope support evidence and fail-closed aggregation.

/// Evidence grade required by one portability cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostEvidenceGrade {
    /// Exact-target compilation.
    Compile,
    /// Tests executed on the named native host/target.
    NativeHostTest,
    /// Runtime smoke executed on the named native host/target.
    NativeRuntimeSmoke,
    /// Runtime evidence in a browser on the named WASM target.
    BrowserRuntime,
}

impl HostEvidenceGrade {
    /// Returns the stable generated-policy spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::NativeHostTest => "native-host-test",
            Self::NativeRuntimeSmoke => "native-runtime-smoke",
            Self::BrowserRuntime => "browser-runtime",
        }
    }
}

/// Method used to produce an evidence receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostEvidenceMethod {
    /// Direct compile/test/runtime evidence at the declared scope.
    Direct,
    /// Compilation for another target from the current host.
    CrossCompile,
    /// Simulated host behavior.
    Simulated,
    /// Renderer test backend rather than a real host runtime.
    TestBackend,
    /// Semantic projection without host execution.
    SemanticProjection,
    /// Screenshot/visual artifact without runtime behavior proof.
    Screenshot,
    /// Experimental receipt outside maintained evidence.
    Experimental,
}

/// Recorded result state for one receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostEvidenceState {
    /// Required command passed.
    Passed,
    /// Required command failed.
    Failed,
    /// Evidence could not run because a prerequisite is blocked.
    Blocked,
    /// Evidence was deliberately skipped.
    Skipped,
    /// Scope is explicitly unsupported.
    Unsupported,
}

/// Freshness state for one receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceFreshness {
    /// Receipt matches the current source/profile identity.
    Fresh,
    /// Receipt is known not to match current source/profile identity.
    Stale,
}

/// Exact target/host requirement for one support cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEvidenceRequirement {
    /// Stable cell identity.
    pub id: String,
    /// Host-family identity.
    pub host_family: super::HostFamilyId,
    /// Exact Rust target identity.
    pub rust_target: super::RustTargetId,
    /// Required evidence grade.
    pub grade: HostEvidenceGrade,
}

impl HostEvidenceRequirement {
    /// Creates an exact evidence requirement.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        host_family: impl Into<String>,
        rust_target: impl Into<String>,
        grade: HostEvidenceGrade,
    ) -> Self {
        Self {
            id: id.into(),
            host_family: super::HostFamilyId::new(host_family),
            rust_target: super::RustTargetId::new(rust_target),
            grade,
        }
    }
}

/// Construction fields for one exact-scope evidence receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEvidenceReceiptParts {
    /// Stable receipt identity.
    pub id: String,
    /// Host family where evidence was produced.
    pub host_family: super::HostFamilyId,
    /// Exact compiled/executed Rust target.
    pub rust_target: super::RustTargetId,
    /// Evidence grade actually produced.
    pub grade: HostEvidenceGrade,
    /// Evidence method.
    pub method: HostEvidenceMethod,
    /// Result state.
    pub state: HostEvidenceState,
    /// Receipt freshness.
    pub freshness: EvidenceFreshness,
}

/// One exact-scope evidence receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEvidenceReceipt {
    /// Stable receipt identity.
    pub id: String,
    /// Host family where evidence was produced.
    pub host_family: super::HostFamilyId,
    /// Exact compiled/executed Rust target.
    pub rust_target: super::RustTargetId,
    /// Evidence grade actually produced.
    pub grade: HostEvidenceGrade,
    /// Evidence method.
    pub method: HostEvidenceMethod,
    /// Result state.
    pub state: HostEvidenceState,
    /// Receipt freshness.
    pub freshness: EvidenceFreshness,
}

impl HostEvidenceReceipt {
    /// Creates a receipt from explicit construction fields.
    #[must_use]
    pub fn new(parts: HostEvidenceReceiptParts) -> Self {
        Self {
            id: parts.id,
            host_family: parts.host_family,
            rust_target: parts.rust_target,
            grade: parts.grade,
            method: parts.method,
            state: parts.state,
            freshness: parts.freshness,
        }
    }
}

/// Fail-closed support status for a required cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostSupportStatus {
    /// Direct fresh evidence satisfies this exact cell.
    Supported,
    /// Related/weaker/non-direct evidence exists but does not satisfy the cell.
    Partial,
    /// Scope is explicitly unsupported.
    Unsupported,
    /// Evidence is blocked or skipped.
    Blocked,
    /// No related evidence exists.
    Missing,
    /// Matching evidence failed.
    Failed,
    /// Matching evidence is stale.
    Stale,
}

/// Returns whether a receipt satisfies a requirement without scope promotion.
#[must_use]
pub fn receipt_satisfies(requirement: &HostEvidenceRequirement, receipt: &HostEvidenceReceipt) -> bool {
    receipt.host_family == requirement.host_family
        && receipt.rust_target == requirement.rust_target
        && receipt.method == HostEvidenceMethod::Direct
        && receipt.state == HostEvidenceState::Passed
        && receipt.freshness == EvidenceFreshness::Fresh
        && grade_covers(requirement, receipt.grade)
}

/// Evaluates one exact support cell from available receipts.
#[must_use]
pub fn evaluate_host_support(
    requirement: &HostEvidenceRequirement,
    receipts: &[HostEvidenceReceipt],
) -> HostSupportStatus {
    if receipts.iter().any(|receipt| receipt_satisfies(requirement, receipt)) {
        return HostSupportStatus::Supported;
    }

    let matching_scope = receipts.iter().filter(|receipt| {
        receipt.host_family == requirement.host_family && receipt.rust_target == requirement.rust_target
    });
    let scoped = matching_scope.collect::<Vec<_>>();
    if scoped.is_empty() {
        return HostSupportStatus::Missing;
    }
    if scoped
        .iter()
        .any(|receipt| receipt.freshness == EvidenceFreshness::Stale && receipt.grade == requirement.grade)
    {
        return HostSupportStatus::Stale;
    }
    if scoped
        .iter()
        .any(|receipt| receipt.state == HostEvidenceState::Failed && receipt.grade == requirement.grade)
    {
        return HostSupportStatus::Failed;
    }
    if scoped.iter().any(|receipt| receipt.state == HostEvidenceState::Unsupported) {
        return HostSupportStatus::Unsupported;
    }
    if scoped
        .iter()
        .any(|receipt| matches!(receipt.state, HostEvidenceState::Blocked | HostEvidenceState::Skipped))
    {
        return HostSupportStatus::Blocked;
    }
    HostSupportStatus::Partial
}

fn grade_covers(requirement: &HostEvidenceRequirement, actual: HostEvidenceGrade) -> bool {
    match requirement.grade {
        HostEvidenceGrade::Compile => match requirement.host_family.as_str() {
            super::HOST_FAMILY_BROWSER => {
                matches!(actual, HostEvidenceGrade::Compile | HostEvidenceGrade::BrowserRuntime)
            }
            _ => matches!(
                actual,
                HostEvidenceGrade::Compile | HostEvidenceGrade::NativeHostTest | HostEvidenceGrade::NativeRuntimeSmoke
            ),
        },
        HostEvidenceGrade::NativeHostTest => {
            matches!(actual, HostEvidenceGrade::NativeHostTest | HostEvidenceGrade::NativeRuntimeSmoke)
        }
        HostEvidenceGrade::NativeRuntimeSmoke => actual == HostEvidenceGrade::NativeRuntimeSmoke,
        HostEvidenceGrade::BrowserRuntime => actual == HostEvidenceGrade::BrowserRuntime,
    }
}
