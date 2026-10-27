//! Production-memory policy descriptors and safe receipt validation.
//!
//! The module describes memory surfaces and evidence without owning cache,
//! database, vector, graph, archive, clock, exporter, billing, or backup effects.

// Cairn adaptation compatibility module keeps DTOs, validators, and fixtures compact while the change is drained.
#![allow(missing_docs)]
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
    non_trait_imports,
    path_segment_repetition,
    underscore_in_module_filename,
    dead_code
)]
#![allow(numeric_units, raw_arithmetic_overflow, unbounded_collection_growth)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::struct_excessive_bools
)]

const HASH_DOMAIN: &str = "animus.production-memory-policy.v1";
const HASH_SEPARATOR: &str = "|";
const MAX_LABEL_CHARS: usize = 96;
const HASH_HEX_CHARS: usize = 64;
const EMPTY_COUNT: u32 = 0;
const MIN_COUNT: u32 = 1;
const UNSAFE_MARKERS: &[&str] = &[
    "RAW_BODY:",
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "sk-",
    "/home/",
    "UNBOUNDED_LOG:",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryTier {
    HotCache,
    WarmSearch,
    ColdArchive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPosture {
    Ephemeral,
    SessionBound,
    ReviewRetained,
    ArchiveRetained,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TtlPosture {
    None,
    HostManaged,
    Expiring,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackupPosture {
    None,
    HostManaged,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaintenanceStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IsolationScope {
    User,
    Project,
    Thread,
    Tenant,
    Host,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemorySurfaceDescriptor {
    pub surface_label: String,
    pub tier: MemoryTier,
    pub retention: RetentionPosture,
    pub ttl: TtlPosture,
    pub promotion_reason: String,
    pub demotion_reason: String,
    pub backup: BackupPosture,
    pub maintenance: MaintenanceStatus,
    pub multi_tenant: bool,
    pub required_scope: Option<IsolationScope>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryProductionPolicy {
    pub policy_label: String,
    pub surfaces: Vec<MemorySurfaceDescriptor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErasureOutcome {
    Complete,
    Partial,
    Denied,
    Stale,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SurfaceErasureReceipt {
    pub surface_label: String,
    pub outcome: ErasureOutcome,
    pub evidence_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryErasureReceipt {
    pub request_hash: String,
    pub surfaces: Vec<SurfaceErasureReceipt>,
    pub claims_complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryBudgetDescriptor {
    pub budget_label: String,
    pub cost_label: String,
    pub latency_label: String,
    pub quota_label: String,
    pub redaction_class: String,
    pub operation_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryOperationScope {
    pub surface_label: String,
    pub tenant_scope: Option<String>,
    pub user_scope: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryPolicyIssue {
    EmptyField,
    UnsafeLabel,
    DuplicateSurface,
    MissingSurface,
    TenantIsolationMissing,
    PartialErasureClaim,
    StaleReceipt,
    UnavailableSurface,
    UnsupportedSurface,
    InvalidHash,
    InvalidBudget,
}

impl MemoryPolicyIssue {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyField => "empty-field",
            Self::UnsafeLabel => "unsafe-label",
            Self::DuplicateSurface => "duplicate-surface",
            Self::MissingSurface => "missing-surface",
            Self::TenantIsolationMissing => "tenant-isolation-missing",
            Self::PartialErasureClaim => "partial-erasure-claim",
            Self::StaleReceipt => "stale-receipt",
            Self::UnavailableSurface => "unavailable-surface",
            Self::UnsupportedSurface => "unsupported-surface",
            Self::InvalidHash => "invalid-hash",
            Self::InvalidBudget => "invalid-budget",
        }
    }
}

pub type MemoryPolicyResult<T> = Result<T, Vec<MemoryPolicyIssue>>;

pub fn validate_memory_production_policy(
    policy: &MemoryProductionPolicy,
) -> MemoryPolicyResult<()> {
    let mut issues = Vec::new();
    validate_label(policy.policy_label.as_str(), &mut issues);
    if policy.surfaces.is_empty() {
        issues.push(MemoryPolicyIssue::MissingSurface);
    }
    let mut seen = Vec::new();
    for surface in &policy.surfaces {
        validate_surface(surface, &mut issues);
        if seen.iter().any(|label| label == &surface.surface_label) {
            issues.push(MemoryPolicyIssue::DuplicateSurface);
        }
        seen.push(surface.surface_label.clone());
    }
    result_from_issues(issues, ())
}

pub fn validate_memory_operation_scope(
    policy: &MemoryProductionPolicy,
    scope: &MemoryOperationScope,
) -> MemoryPolicyResult<()> {
    let mut issues = Vec::new();
    validate_label(scope.surface_label.as_str(), &mut issues);
    if let Some(tenant) = &scope.tenant_scope {
        validate_label(tenant.as_str(), &mut issues);
    }
    if let Some(user) = &scope.user_scope {
        validate_label(user.as_str(), &mut issues);
    }
    let Some(surface) = policy
        .surfaces
        .iter()
        .find(|surface| surface.surface_label == scope.surface_label)
    else {
        issues.push(MemoryPolicyIssue::MissingSurface);
        return result_from_issues(issues, ());
    };
    if surface.multi_tenant && scope.tenant_scope.is_none() && scope.user_scope.is_none() {
        issues.push(MemoryPolicyIssue::TenantIsolationMissing);
    }
    result_from_issues(issues, ())
}

pub fn validate_memory_erasure_receipt(
    policy: &MemoryProductionPolicy,
    receipt: &MemoryErasureReceipt,
) -> MemoryPolicyResult<()> {
    let mut issues = Vec::new();
    validate_hash(receipt.request_hash.as_str(), &mut issues);
    if receipt.surfaces.is_empty() {
        issues.push(MemoryPolicyIssue::MissingSurface);
    }
    for surface in &policy.surfaces {
        if !receipt
            .surfaces
            .iter()
            .any(|entry| entry.surface_label == surface.surface_label)
        {
            issues.push(MemoryPolicyIssue::MissingSurface);
        }
    }
    for entry in &receipt.surfaces {
        validate_label(entry.surface_label.as_str(), &mut issues);
        validate_hash(entry.evidence_hash.as_str(), &mut issues);
        match entry.outcome {
            ErasureOutcome::Complete => {}
            ErasureOutcome::Partial => issues.push(MemoryPolicyIssue::PartialErasureClaim),
            ErasureOutcome::Denied | ErasureOutcome::Stale => {
                issues.push(MemoryPolicyIssue::StaleReceipt);
            }
            ErasureOutcome::Unavailable => issues.push(MemoryPolicyIssue::UnavailableSurface),
            ErasureOutcome::Unsupported => issues.push(MemoryPolicyIssue::UnsupportedSurface),
        }
    }
    if receipt.claims_complete
        && receipt
            .surfaces
            .iter()
            .any(|entry| entry.outcome != ErasureOutcome::Complete)
    {
        issues.push(MemoryPolicyIssue::PartialErasureClaim);
    }
    result_from_issues(issues, ())
}

pub fn validate_memory_budget_descriptor(
    descriptor: &MemoryBudgetDescriptor,
) -> MemoryPolicyResult<()> {
    let mut issues = Vec::new();
    validate_label(descriptor.budget_label.as_str(), &mut issues);
    validate_label(descriptor.cost_label.as_str(), &mut issues);
    validate_label(descriptor.latency_label.as_str(), &mut issues);
    validate_label(descriptor.quota_label.as_str(), &mut issues);
    validate_label(descriptor.redaction_class.as_str(), &mut issues);
    if descriptor.operation_count < MIN_COUNT {
        issues.push(MemoryPolicyIssue::InvalidBudget);
    }
    result_from_issues(issues, ())
}

pub fn memory_policy_hash(label: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(label.as_bytes());
    hasher.update(HASH_SEPARATOR.as_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn validate_surface(surface: &MemorySurfaceDescriptor, issues: &mut Vec<MemoryPolicyIssue>) {
    validate_label(surface.surface_label.as_str(), issues);
    validate_label(surface.promotion_reason.as_str(), issues);
    validate_label(surface.demotion_reason.as_str(), issues);
    if surface.multi_tenant && surface.required_scope.is_none() {
        issues.push(MemoryPolicyIssue::TenantIsolationMissing);
    }
    if surface.maintenance == MaintenanceStatus::Unavailable {
        issues.push(MemoryPolicyIssue::UnavailableSurface);
    }
}

fn validate_label(value: &str, issues: &mut Vec<MemoryPolicyIssue>) {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_LABEL_CHARS {
        issues.push(MemoryPolicyIssue::EmptyField);
    }
    if UNSAFE_MARKERS.iter().any(|marker| trimmed.contains(marker)) {
        issues.push(MemoryPolicyIssue::UnsafeLabel);
    }
}

fn validate_hash(value: &str, issues: &mut Vec<MemoryPolicyIssue>) {
    if value.len() != HASH_HEX_CHARS || !value.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        issues.push(MemoryPolicyIssue::InvalidHash);
    }
}

fn result_from_issues<T>(mut issues: Vec<MemoryPolicyIssue>, value: T) -> MemoryPolicyResult<T> {
    issues.sort_by_key(|issue| issue.code());
    issues.dedup();
    if issues.is_empty() {
        Ok(value)
    } else {
        Err(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_LABEL: &str = "production-memory-policy";
    const HOT_SURFACE: &str = "hot-cache";
    const WARM_SURFACE: &str = "warm-search";
    const COLD_SURFACE: &str = "cold-archive";
    const PROMOTION_REASON: &str = "reviewed promotion";
    const DEMOTION_REASON: &str = "stale demotion";
    const REQUEST_BYTES: &[u8] = b"erasure request";
    const EVIDENCE_BYTES: &[u8] = b"surface evidence";
    const OPERATION_COUNT: u32 = 4;

    #[test]
    fn tier_budget_and_tenant_descriptors_validate_without_backend_authority() {
        // r[verify agent_harness_runtime.memory_production_policy.tiers]
        // r[verify agent_harness_runtime.memory_production_policy.budgets]
        // r[verify agent_harness_runtime.memory_production_policy.tenant_isolation]
        let policy = policy();
        let budget = MemoryBudgetDescriptor {
            budget_label: "retrieval-budget".to_owned(),
            cost_label: "cost-class-low".to_owned(),
            latency_label: "latency-class-local".to_owned(),
            quota_label: "quota-within-bound".to_owned(),
            redaction_class: "safe-counts-only".to_owned(),
            operation_count: OPERATION_COUNT,
        };
        let scope = MemoryOperationScope {
            surface_label: WARM_SURFACE.to_owned(),
            tenant_scope: Some("tenant-a".to_owned()),
            user_scope: None,
        };

        assert!(validate_memory_production_policy(&policy).is_ok());
        assert!(validate_memory_budget_descriptor(&budget).is_ok());
        assert!(validate_memory_operation_scope(&policy, &scope).is_ok());
    }

    #[test]
    fn erasure_receipts_do_not_claim_complete_when_any_surface_is_partial() {
        // r[verify agent_harness_runtime.memory_production_policy.erasure]
        let policy = policy();
        let receipt = MemoryErasureReceipt {
            request_hash: memory_policy_hash("request", REQUEST_BYTES),
            surfaces: vec![
                erasure(HOT_SURFACE, ErasureOutcome::Complete),
                erasure(WARM_SURFACE, ErasureOutcome::Unavailable),
                erasure(COLD_SURFACE, ErasureOutcome::Complete),
            ],
            claims_complete: true,
        };

        let issues = validate_memory_erasure_receipt(&policy, &receipt).expect_err("partial");

        assert!(issues.contains(&MemoryPolicyIssue::UnavailableSurface));
        assert!(issues.contains(&MemoryPolicyIssue::PartialErasureClaim));
    }

    #[test]
    fn invalid_policy_budget_scope_and_receipts_fail_closed() {
        // r[verify agent_harness_runtime.memory_production_policy.verification]
        let mut unsafe_policy = policy();
        unsafe_policy.surfaces[0].surface_label = "RAW_BODY:private".to_owned();
        assert!(
            validate_memory_production_policy(&unsafe_policy)
                .expect_err("unsafe")
                .contains(&MemoryPolicyIssue::UnsafeLabel)
        );

        let missing_scope = MemoryOperationScope {
            surface_label: WARM_SURFACE.to_owned(),
            tenant_scope: None,
            user_scope: None,
        };
        assert!(
            validate_memory_operation_scope(&policy(), &missing_scope)
                .expect_err("tenant")
                .contains(&MemoryPolicyIssue::TenantIsolationMissing)
        );

        let stale = MemoryErasureReceipt {
            request_hash: memory_policy_hash("request", REQUEST_BYTES),
            surfaces: vec![
                erasure(HOT_SURFACE, ErasureOutcome::Stale),
                erasure(WARM_SURFACE, ErasureOutcome::Complete),
                erasure(COLD_SURFACE, ErasureOutcome::Unsupported),
            ],
            claims_complete: false,
        };
        let issues = validate_memory_erasure_receipt(&policy(), &stale).expect_err("stale");
        assert!(issues.contains(&MemoryPolicyIssue::StaleReceipt));
        assert!(issues.contains(&MemoryPolicyIssue::UnsupportedSurface));

        let invalid_budget = MemoryBudgetDescriptor {
            budget_label: "budget".to_owned(),
            cost_label: "API_KEY=secret".to_owned(),
            latency_label: "latency".to_owned(),
            quota_label: "quota".to_owned(),
            redaction_class: "redacted".to_owned(),
            operation_count: EMPTY_COUNT,
        };
        let issues = validate_memory_budget_descriptor(&invalid_budget).expect_err("budget");
        assert!(issues.contains(&MemoryPolicyIssue::UnsafeLabel));
        assert!(issues.contains(&MemoryPolicyIssue::InvalidBudget));
    }

    fn policy() -> MemoryProductionPolicy {
        MemoryProductionPolicy {
            policy_label: POLICY_LABEL.to_owned(),
            surfaces: vec![
                surface(HOT_SURFACE, MemoryTier::HotCache, false),
                surface(WARM_SURFACE, MemoryTier::WarmSearch, true),
                surface(COLD_SURFACE, MemoryTier::ColdArchive, false),
            ],
        }
    }

    fn surface(label: &str, tier: MemoryTier, multi_tenant: bool) -> MemorySurfaceDescriptor {
        MemorySurfaceDescriptor {
            surface_label: label.to_owned(),
            tier,
            retention: RetentionPosture::ReviewRetained,
            ttl: TtlPosture::HostManaged,
            promotion_reason: PROMOTION_REASON.to_owned(),
            demotion_reason: DEMOTION_REASON.to_owned(),
            backup: BackupPosture::HostManaged,
            maintenance: MaintenanceStatus::Ready,
            multi_tenant,
            required_scope: multi_tenant.then_some(IsolationScope::Tenant),
        }
    }

    fn erasure(surface_label: &str, outcome: ErasureOutcome) -> SurfaceErasureReceipt {
        SurfaceErasureReceipt {
            surface_label: surface_label.to_owned(),
            outcome,
            evidence_hash: memory_policy_hash(surface_label, EVIDENCE_BYTES),
        }
    }
}
