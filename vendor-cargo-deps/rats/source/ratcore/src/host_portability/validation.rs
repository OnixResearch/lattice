//! Deterministic validation for host-portability policy values.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Stable host-portability diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostPortabilityIssueCode {
    /// Required identity, path, owner, policy, or label is empty.
    EmptyField,
    /// Stable identity is duplicated.
    DuplicateIdentity,
    /// A package has no unique base production row.
    MissingPackageBase,
    /// Unknown ownership role was supplied.
    UnknownRole,
    /// A row references an unknown host family.
    UnknownHostFamily,
    /// A row references an unknown exact Rust target.
    UnknownRustTarget,
    /// A row references an unknown target scope.
    UnknownTargetScope,
    /// Exact/classification target scope is malformed.
    InvalidTargetScope,
    /// A row references an unknown capability.
    UnknownCapability,
    /// A migration blocker has no dependent change.
    MissingDependentChange,
    /// A conforming pure/reusable row directly acquires an ambient effect.
    ReusableAmbientEffect,
    /// Adapter/shell/evidence effect lacks diagnostic or fallback policy.
    MissingEffectPolicy,
    /// Effect and capability canonical owners disagree.
    EffectOwnerMismatch,
    /// Capability identity resembles a universal platform/service locator.
    UniversalPlatformCapability,
    /// Capability fallback infers another mechanism from an OS/host label.
    InferredHostFallback,
    /// Reusable authoritative state stores a callback, host handle, executor, or backend event.
    RetainedHostState,
    /// Caller/external authority attachment is incomplete.
    MissingAuthorityAttachment,
    /// Scanner found an unclassified coordinate.
    UnclassifiedScanCoordinate,
    /// Inventory scanner coordinate no longer exists.
    MissingScanCoordinate,
    /// Scanner match count changed.
    ScanCountMismatch,
    /// Evidence requirement/receipt combines an incompatible host and target.
    HostTargetMismatch,
}

/// Deterministic content-free policy diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HostPortabilityIssue {
    /// Stable diagnostic code.
    pub code: HostPortabilityIssueCode,
    /// Stable policy domain.
    pub domain: String,
    /// Stable affected identity.
    pub subject: String,
    /// Human-readable content-free message.
    pub message: String,
}

impl HostPortabilityIssue {
    fn new(
        code: HostPortabilityIssueCode,
        domain: &'static str,
        subject: impl Into<String>,
        message: &'static str,
    ) -> Self {
        Self {
            code,
            domain: domain.to_string(),
            subject: subject.into(),
            message: message.to_string(),
        }
    }
}

/// Ordered host-portability validation result.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostPortabilityReport {
    /// Stable sorted diagnostics.
    pub issues: Vec<HostPortabilityIssue>,
}

impl HostPortabilityReport {
    /// Returns true when classification/policy structure is valid.
    ///
    /// Valid classification can still contain explicitly non-passing migration
    /// blockers; callers must inspect dispositions before making support claims.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

/// One observed source scanner coordinate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScannedHostCoordinate {
    /// Workspace-relative source path.
    pub path: String,
    /// Scanner class.
    pub scan_class: super::EffectScanClass,
    /// Number of non-comment matches.
    pub match_count: usize,
}

impl ScannedHostCoordinate {
    /// Creates a scanner observation.
    #[must_use]
    pub fn new(path: impl Into<String>, scan_class: super::EffectScanClass, match_count: usize) -> Self {
        Self {
            path: path.into(),
            scan_class,
            match_count,
        }
    }
}

/// Borrowed values forming one complete host-portability policy.
#[derive(Debug, Clone, Copy)]
pub struct HostPortabilityPolicy<'a> {
    /// Host-family registry.
    pub families: &'a [super::HostFamilyDescriptor],
    /// Exact Rust target registry.
    pub targets: &'a [super::RustTargetDescriptor],
    /// Inventory target scopes.
    pub scopes: &'a [super::HostTargetScope],
    /// Package and source-slice inventory.
    pub packages: &'a [super::PackageSurface],
    /// Ambient effect and dependency inventory.
    pub effects: &'a [super::HostEffectCoordinate],
    /// Narrow capability contracts.
    pub capabilities: &'a [super::HostCapabilityContract],
}

/// Validates the complete built-in taxonomy, inventory, and capability policy.
#[must_use]
pub fn validate_built_in_host_portability_policy() -> HostPortabilityReport {
    validate_host_portability_policy(HostPortabilityPolicy {
        families: &super::built_in_host_families(),
        targets: &super::built_in_rust_targets(),
        scopes: &super::built_in_host_target_scopes(),
        packages: &super::built_in_package_surfaces(),
        effects: &super::built_in_host_effects(),
        capabilities: &super::built_in_host_capability_contracts(),
    })
}

/// Validates supplied taxonomy, package/effect inventories, and capabilities.
#[must_use]
pub fn validate_host_portability_policy(policy: HostPortabilityPolicy<'_>) -> HostPortabilityReport {
    let mut issues = Vec::new();
    validate_taxonomy(policy.families, policy.targets, policy.scopes, &mut issues);
    validate_packages(policy.scopes, policy.packages, &mut issues);
    validate_capabilities(policy.scopes, policy.capabilities, &mut issues);
    validate_effects(policy.scopes, policy.effects, policy.capabilities, &mut issues);
    finish(issues)
}

fn validate_taxonomy(
    families: &[super::HostFamilyDescriptor],
    targets: &[super::RustTargetDescriptor],
    scopes: &[super::HostTargetScope],
    issues: &mut Vec<HostPortabilityIssue>,
) {
    let mut family_ids = BTreeSet::new();
    for family in families {
        required(family.id.as_str(), "host-family", family.id.as_str(), issues);
        required(&family.label, "host-family", family.id.as_str(), issues);
        duplicate(family_ids.insert(family.id.clone()), "host-family", family.id.as_str(), issues);
    }
    let mut target_ids = BTreeSet::new();
    for target in targets {
        required(target.id.as_str(), "rust-target", target.id.as_str(), issues);
        required(&target.architecture, "rust-target", target.id.as_str(), issues);
        duplicate(target_ids.insert(target.id.clone()), "rust-target", target.id.as_str(), issues);
        if !family_ids.contains(&target.host_family) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownHostFamily,
                "rust-target",
                target.id.as_str(),
                "exact target references an unknown host family",
            ));
        }
    }
    let target_map = targets.iter().map(|target| (target.id.clone(), target)).collect::<BTreeMap<_, _>>();
    let mut scope_ids = BTreeSet::new();
    for scope in scopes {
        required(scope.id.as_str(), "target-scope", scope.id.as_str(), issues);
        duplicate(scope_ids.insert(scope.id.clone()), "target-scope", scope.id.as_str(), issues);
        if !family_ids.contains(&scope.host_family) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownHostFamily,
                "target-scope",
                scope.id.as_str(),
                "target scope references an unknown host family",
            ));
        }
        match (scope.kind, scope.rust_target.as_ref()) {
            (super::HostTargetScopeKind::ClassificationOnly, None) => {}
            (super::HostTargetScopeKind::ExactTarget, Some(target_id)) => {
                if let Some(target) = target_map.get(target_id) {
                    if target.host_family != scope.host_family {
                        issues.push(HostPortabilityIssue::new(
                            HostPortabilityIssueCode::InvalidTargetScope,
                            "target-scope",
                            scope.id.as_str(),
                            "exact target and host family do not match",
                        ));
                    }
                } else {
                    issues.push(HostPortabilityIssue::new(
                        HostPortabilityIssueCode::UnknownRustTarget,
                        "target-scope",
                        scope.id.as_str(),
                        "target scope references an unknown exact target",
                    ));
                }
            }
            _ => issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::InvalidTargetScope,
                "target-scope",
                scope.id.as_str(),
                "classification scope must omit and exact scope must name a Rust target",
            )),
        }
    }
}

fn validate_packages(
    scopes: &[super::HostTargetScope],
    packages: &[super::PackageSurface],
    issues: &mut Vec<HostPortabilityIssue>,
) {
    let scope_ids = scopes.iter().map(|scope| scope.id.clone()).collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    let mut base_counts = BTreeMap::<String, usize>::new();
    for row in packages {
        for value in [
            row.id.as_str(),
            row.package.as_str(),
            row.path.as_str(),
            row.owner.as_str(),
            row.default_features.as_str(),
        ] {
            required(value, "package-surface", row.id.as_str(), issues);
        }
        duplicate(ids.insert(row.id.clone()), "package-surface", row.id.as_str(), issues);
        if row.role == super::PortabilityRole::Unknown {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownRole,
                "package-surface",
                row.id.as_str(),
                "package surface uses an unknown ownership role",
            ));
        }
        if !scope_ids.contains(&row.target_scope) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownTargetScope,
                "package-surface",
                row.id.as_str(),
                "package surface references an unknown target scope",
            ));
        }
        blocker_has_change(
            row.disposition,
            row.dependent_change.as_deref(),
            "package-surface",
            row.id.as_str(),
            issues,
        );
        if row.id.as_str().starts_with("package.") && row.surface == super::SourceSurface::Production {
            *base_counts.entry(row.package.clone()).or_default() += 1;
        }
    }
    let all_packages = packages.iter().map(|row| row.package.as_str()).collect::<BTreeSet<_>>();
    for package in all_packages {
        if base_counts.get(package).copied() != Some(1) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::MissingPackageBase,
                "package",
                package,
                "package must have exactly one base production row",
            ));
        }
    }
}

fn validate_capabilities(
    scopes: &[super::HostTargetScope],
    capabilities: &[super::HostCapabilityContract],
    issues: &mut Vec<HostPortabilityIssue>,
) {
    let scope_ids = scopes.iter().map(|scope| scope.id.clone()).collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    for capability in capabilities {
        for value in [
            capability.id.as_str(),
            capability.owner.as_str(),
            capability.diagnostics.as_str(),
            capability.fallback.as_str(),
        ] {
            required(value, "capability", capability.id.as_str(), issues);
        }
        duplicate(ids.insert(capability.id.clone()), "capability", capability.id.as_str(), issues);
        if capability.role == super::PortabilityRole::Unknown {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownRole,
                "capability",
                capability.id.as_str(),
                "capability uses an unknown ownership role",
            ));
        }
        if !scope_ids.contains(&capability.target_scope) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownTargetScope,
                "capability",
                capability.id.as_str(),
                "capability references an unknown target scope",
            ));
        }
        let normalized = capability.id.as_str().to_ascii_lowercase();
        if normalized.contains("universal-platform")
            || normalized.contains("platform-service")
            || normalized.contains("service-locator")
            || normalized.contains("runtime-handle")
        {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UniversalPlatformCapability,
                "capability",
                capability.id.as_str(),
                "capability must be feature-specific rather than a platform service object",
            ));
        }
        let fallback = capability.fallback.to_ascii_lowercase();
        if [
            "target_os",
            "os label",
            "if linux",
            "if windows",
            "if macos",
            "wayland_display",
        ]
        .iter()
        .any(|marker| fallback.contains(marker))
        {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::InferredHostFallback,
                "capability",
                capability.id.as_str(),
                "fallback must not infer an alternate mechanism from a host label",
            ));
        }
        if capability.state_kind != super::CapabilityStateKind::PlainValues {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::RetainedHostState,
                "capability",
                capability.id.as_str(),
                "capability boundary may retain only plain values",
            ));
        }
        if matches!(capability.authority, super::CapabilityAuthority::Caller | super::CapabilityAuthority::External)
            && !capability.attachments.is_complete()
        {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::MissingAuthorityAttachment,
                "capability",
                capability.id.as_str(),
                "caller/external effects require current identity, generation, capabilities, and authorization",
            ));
        }
    }
}

fn validate_effects(
    scopes: &[super::HostTargetScope],
    effects: &[super::HostEffectCoordinate],
    capabilities: &[super::HostCapabilityContract],
    issues: &mut Vec<HostPortabilityIssue>,
) {
    let scope_ids = scopes.iter().map(|scope| scope.id.clone()).collect::<BTreeSet<_>>();
    let capability_map = capabilities
        .iter()
        .map(|capability| (capability.id.clone(), capability))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut scanner_keys = BTreeSet::new();
    for effect in effects {
        for value in [
            effect.id.as_str(),
            effect.package.as_str(),
            effect.path.as_str(),
            effect.owner.as_str(),
            effect.diagnostics.as_str(),
            effect.fallback.as_str(),
        ] {
            required(value, "host-effect", effect.id.as_str(), issues);
        }
        duplicate(ids.insert(effect.id.clone()), "host-effect", effect.id.as_str(), issues);
        let scanner_key = (effect.path.clone(), effect.scan_class);
        duplicate(scanner_keys.insert(scanner_key), "host-effect-scan", effect.id.as_str(), issues);
        validate_effect_role(effect, &scope_ids, issues);
        if let Some(capability) = capability_map.get(&effect.capability) {
            if capability.owner != effect.owner {
                issues.push(HostPortabilityIssue::new(
                    HostPortabilityIssueCode::EffectOwnerMismatch,
                    "host-effect",
                    effect.id.as_str(),
                    "effect owner differs from canonical capability owner",
                ));
            }
        } else {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnknownCapability,
                "host-effect",
                effect.id.as_str(),
                "effect references an unknown narrow capability",
            ));
        }
        if effect.scan_match_count == 0 {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::MissingScanCoordinate,
                "host-effect",
                effect.id.as_str(),
                "effect scanner match count must be non-zero",
            ));
        }
    }
}

fn validate_effect_role(
    effect: &super::HostEffectCoordinate,
    scope_ids: &BTreeSet<super::HostTargetScopeId>,
    issues: &mut Vec<HostPortabilityIssue>,
) {
    if effect.role == super::PortabilityRole::Unknown {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::UnknownRole,
            "host-effect",
            effect.id.as_str(),
            "effect uses an unknown ownership role",
        ));
    }
    if !scope_ids.contains(&effect.target_scope) {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::UnknownTargetScope,
            "host-effect",
            effect.id.as_str(),
            "effect references an unknown target scope",
        ));
    }
    blocker_has_change(
        effect.disposition,
        effect.dependent_change.as_deref(),
        "host-effect",
        effect.id.as_str(),
        issues,
    );
    if !effect.role.may_acquire_host_effects() && effect.disposition == super::PortabilityDisposition::Conforming {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::ReusableAmbientEffect,
            "host-effect",
            effect.id.as_str(),
            "pure/reusable surface cannot classify ambient acquisition as conforming",
        ));
    }
    if effect.role.may_acquire_host_effects()
        && (effect.diagnostics.trim().is_empty() || effect.fallback.trim().is_empty())
    {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::MissingEffectPolicy,
            "host-effect",
            effect.id.as_str(),
            "effect-owning edge requires diagnostics and fallback policy",
        ));
    }
}

/// Validates exact source-scan coverage against effect inventory rows.
#[must_use]
pub fn validate_host_scan_coverage(
    effects: &[super::HostEffectCoordinate],
    observed: &[ScannedHostCoordinate],
) -> HostPortabilityReport {
    let expected = effects
        .iter()
        .filter(|effect| effect.scan_class != super::EffectScanClass::Dependency)
        .map(|effect| ((effect.path.clone(), effect.scan_class), effect.scan_match_count))
        .collect::<BTreeMap<_, _>>();
    let actual = observed
        .iter()
        .map(|coordinate| ((coordinate.path.clone(), coordinate.scan_class), coordinate.match_count))
        .collect::<BTreeMap<_, _>>();
    let mut issues = Vec::new();
    for ((path, scan_class), expected_count) in &expected {
        let subject = format!("{}:{}", scan_class.as_str(), path);
        match actual.get(&(path.clone(), *scan_class)) {
            None => issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::MissingScanCoordinate,
                "source-scan",
                subject,
                "inventoried scanner coordinate is missing",
            )),
            Some(actual_count) if actual_count != expected_count => {
                issues.push(HostPortabilityIssue::new(
                    HostPortabilityIssueCode::ScanCountMismatch,
                    "source-scan",
                    subject,
                    "scanner match count differs from checked inventory",
                ));
            }
            Some(_) => {}
        }
    }
    for (path, scan_class) in actual.keys() {
        if !expected.contains_key(&(path.clone(), *scan_class)) {
            issues.push(HostPortabilityIssue::new(
                HostPortabilityIssueCode::UnclassifiedScanCoordinate,
                "source-scan",
                format!("{}:{}", scan_class.as_str(), path),
                "scanner found an unclassified host coordinate",
            ));
        }
    }
    finish(issues)
}

/// Validates that a requirement or receipt host family matches its exact target.
#[must_use]
pub fn validate_evidence_scopes(
    targets: &[super::RustTargetDescriptor],
    requirements: &[super::HostEvidenceRequirement],
    receipts: &[super::HostEvidenceReceipt],
) -> HostPortabilityReport {
    let target_map = targets
        .iter()
        .map(|target| (target.id.clone(), target.host_family.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut issues = Vec::new();
    for requirement in requirements {
        validate_evidence_scope(
            &target_map,
            &requirement.id,
            &requirement.host_family,
            &requirement.rust_target,
            &mut issues,
        );
    }
    for receipt in receipts {
        validate_evidence_scope(&target_map, &receipt.id, &receipt.host_family, &receipt.rust_target, &mut issues);
    }
    finish(issues)
}

fn validate_evidence_scope(
    target_map: &BTreeMap<super::RustTargetId, super::HostFamilyId>,
    id: &str,
    host_family: &super::HostFamilyId,
    rust_target: &super::RustTargetId,
    issues: &mut Vec<HostPortabilityIssue>,
) {
    if target_map.get(rust_target) != Some(host_family) {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::HostTargetMismatch,
            "evidence",
            id,
            "evidence host family does not match its exact Rust target",
        ));
    }
}

fn blocker_has_change(
    disposition: super::PortabilityDisposition,
    change: Option<&str>,
    domain: &'static str,
    subject: &str,
    issues: &mut Vec<HostPortabilityIssue>,
) {
    if disposition == super::PortabilityDisposition::MigrationBlocker && change.is_none_or(str::is_empty) {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::MissingDependentChange,
            domain,
            subject,
            "migration blocker must name its dependent change",
        ));
    }
}

fn required(value: &str, domain: &'static str, subject: &str, issues: &mut Vec<HostPortabilityIssue>) {
    if value.trim().is_empty() {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::EmptyField,
            domain,
            subject,
            "required field is empty",
        ));
    }
}

fn duplicate(inserted: bool, domain: &'static str, subject: &str, issues: &mut Vec<HostPortabilityIssue>) {
    if !inserted {
        issues.push(HostPortabilityIssue::new(
            HostPortabilityIssueCode::DuplicateIdentity,
            domain,
            subject,
            "stable identity or scanner coordinate is duplicated",
        ));
    }
}

fn finish(mut issues: Vec<HostPortabilityIssue>) -> HostPortabilityReport {
    issues.sort();
    issues.dedup();
    HostPortabilityReport { issues }
}
