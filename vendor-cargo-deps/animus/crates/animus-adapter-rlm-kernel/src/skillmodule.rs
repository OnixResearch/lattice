use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Exact schema for a reviewed Steel skill-module descriptor.
pub const STEEL_SKILL_MODULE_SCHEMA: &str = "animus.steel-skill-module.v1";
/// Exact language cohort accepted by the first module loader.
pub const STEEL_SKILL_MODULE_COHORT: &str = "steel-source-v1";
/// Domain for module identities.
pub const STEEL_SKILL_MODULE_IDENTITY_DOMAIN: &str = "animus.steel-skill-module-identity.v1";
/// Domain for metadata-only load receipts.
pub const STEEL_SKILL_MODULE_RECEIPT_DOMAIN: &str = "animus.steel-skill-module-receipt.v1";

const BLAKE3_HEX_CHARS: usize = 64;
const MAX_MODULE_NAME_CHARS: u32 = 128;
const MAX_VERSION_CHARS: u32 = 64;
const MAX_EXPORT_CHARS: u32 = 128;
const MAX_CAPABILITY_CHARS: u32 = 128;
const MAX_MODULES: usize = 64;
const MAX_EXPORTS_PER_MODULE: usize = 64;
const MAX_DEPENDENCIES_PER_MODULE: usize = 64;
const MAX_CAPABILITIES_PER_MODULE: usize = 32;
const MAX_TOTAL_SOURCE_CHARS: u64 = 1_048_576;
const MAX_TOTAL_EVALUATION_FUEL: u64 = 16_777_216;
const REDACTION_MARKER: &str = "steel-skill-module-private-content-omitted";
const NON_CLAIM: &str = "no-source-authenticity,no-skill-correctness,no-native-extension-safety,no-sandbox-claim,no-task-success,no-production-readiness";
const KNOWN_BRIDGE_CAPABILITIES: &[&str] = &[
    "context_read",
    "file_edit",
    "file_read",
    "kernel_clear",
    "shell",
    "subagent",
    "tool",
];

/// Trust state for one host-selected skill module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SteelSkillModuleTrust {
    /// The host reviewed and trusted this exact source identity.
    Trusted,
    /// The source is known but has no current trust decision.
    Unreviewed,
    /// The host explicitly denied this source.
    Denied,
}

impl SteelSkillModuleTrust {
    /// Returns the stable trust label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Unreviewed => "unreviewed",
            Self::Denied => "denied",
        }
    }
}

/// One exact module dependency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SteelSkillModuleDependency {
    name: String,
    version: String,
    module_identity: String,
}

/// Borrowed dependency fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SteelSkillModuleDependencyDraft<'a> {
    /// Exact module name.
    pub name: &'a str,
    /// Exact reviewed version.
    pub version: &'a str,
    /// Exact module identity.
    pub module_identity: &'a str,
}

impl SteelSkillModuleDependency {
    /// Validates one exact dependency.
    ///
    /// # Errors
    ///
    /// Returns [`SteelSkillModuleError`] for malformed names, versions, or identities.
    pub fn validate(
        draft: SteelSkillModuleDependencyDraft<'_>,
    ) -> Result<Self, SteelSkillModuleError> {
        Ok(Self {
            name: safe_label("dependency_name", draft.name, MAX_MODULE_NAME_CHARS)?,
            version: safe_label("dependency_version", draft.version, MAX_VERSION_CHARS)?,
            module_identity: blake3_identity("dependency_module_identity", draft.module_identity)?,
        })
    }

    /// Returns the dependency module name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the exact dependency version.
    #[must_use]
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Returns the exact dependency identity.
    #[must_use]
    pub fn module_identity(&self) -> &str {
        self.module_identity.as_str()
    }
}

/// Bounded evaluation limits for one Steel skill module.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SteelSkillModuleBounds {
    /// Maximum source characters.
    pub source_chars: u32,
    /// Maximum structural evaluation fuel.
    pub evaluation_fuel: u64,
}

impl SteelSkillModuleBounds {
    fn validate(self) -> Result<(), SteelSkillModuleError> {
        if self.source_chars == 0 || u64::from(self.source_chars) > MAX_TOTAL_SOURCE_CHARS {
            return Err(SteelSkillModuleError::BoundExceeded {
                field: "source_chars",
            });
        }
        if self.evaluation_fuel == 0 || self.evaluation_fuel > MAX_TOTAL_EVALUATION_FUEL {
            return Err(SteelSkillModuleError::BoundExceeded {
                field: "evaluation_fuel",
            });
        }
        Ok(())
    }
}

/// Validated reviewed Steel skill module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SteelSkillModuleDescriptor {
    schema: &'static str,
    name: String,
    version: String,
    manifest_identity: String,
    frozen_source_identity: String,
    source_identity: String,
    source: String,
    exports: Vec<String>,
    dependencies: Vec<SteelSkillModuleDependency>,
    capability_requests: Vec<String>,
    trust: SteelSkillModuleTrust,
    bounds: SteelSkillModuleBounds,
    module_identity: String,
}

/// Borrowed module fields before validation.
#[derive(Clone, Copy, Debug)]
pub struct SteelSkillModuleDescriptorDraft<'a> {
    /// Descriptor schema.
    pub schema: &'a str,
    /// Exact module name.
    pub name: &'a str,
    /// Exact reviewed version.
    pub version: &'a str,
    /// Existing skill-manifest identity.
    pub manifest_identity: &'a str,
    /// Existing frozen skill-source identity.
    pub frozen_source_identity: &'a str,
    /// Domain-separated BLAKE3 identity of `source`.
    pub source_identity: &'a str,
    /// Complete private Steel source unit.
    pub source: &'a str,
    /// Named exports defined by the source unit.
    pub exports: &'a [&'a str],
    /// Exact reviewed module dependencies.
    pub dependencies: &'a [SteelSkillModuleDependencyDraft<'a>],
    /// Requested existing bridge capability labels.
    pub capability_requests: &'a [&'a str],
    /// Host trust decision.
    pub trust: SteelSkillModuleTrust,
    /// Source and evaluation bounds.
    pub bounds: SteelSkillModuleBounds,
}

impl SteelSkillModuleDescriptor {
    /// Validates one complete private Steel source module without evaluating it.
    ///
    /// # Errors
    ///
    /// Returns [`SteelSkillModuleError`] for schema, identity, trust, source,
    /// export, dependency, capability, or bound failures.
    pub fn validate(
        draft: &SteelSkillModuleDescriptorDraft<'_>,
    ) -> Result<Self, SteelSkillModuleError> {
        // r[impl agent_harness_runtime.steel_skill_modules.descriptor]
        // r[impl agent_harness_runtime.steel_skill_modules.admission]
        if draft.schema != STEEL_SKILL_MODULE_SCHEMA {
            return Err(SteelSkillModuleError::UnsupportedSchema);
        }
        if draft.trust != SteelSkillModuleTrust::Trusted {
            return Err(SteelSkillModuleError::UntrustedSource);
        }
        draft.bounds.validate()?;
        let name = safe_label("module_name", draft.name, MAX_MODULE_NAME_CHARS)?;
        let version = safe_label("module_version", draft.version, MAX_VERSION_CHARS)?;
        let manifest_identity = blake3_identity("manifest_identity", draft.manifest_identity)?;
        let frozen_source_identity =
            blake3_identity("frozen_source_identity", draft.frozen_source_identity)?;
        let source_identity = blake3_identity("source_identity", draft.source_identity)?;
        if steel_skill_module_source_identity(draft.source) != source_identity {
            return Err(SteelSkillModuleError::SourceIdentityMismatch);
        }
        let exports = validated_labels(
            "exports",
            draft.exports,
            MAX_EXPORTS_PER_MODULE,
            MAX_EXPORT_CHARS,
        )?;
        let capability_requests = validated_optional_labels(
            "capability_requests",
            draft.capability_requests,
            MAX_CAPABILITIES_PER_MODULE,
            MAX_CAPABILITY_CHARS,
        )?;
        if capability_requests.iter().any(|capability| {
            KNOWN_BRIDGE_CAPABILITIES
                .binary_search(&capability.as_str())
                .is_err()
        }) {
            return Err(SteelSkillModuleError::UnknownCapability);
        }
        validate_source(
            draft.source,
            draft.bounds.source_chars,
            capability_requests.as_slice(),
        )?;
        let dependencies = draft
            .dependencies
            .iter()
            .copied()
            .map(SteelSkillModuleDependency::validate)
            .collect::<Result<Vec<_>, _>>()?;
        if dependencies.len() > MAX_DEPENDENCIES_PER_MODULE {
            return Err(SteelSkillModuleError::BoundExceeded {
                field: "dependencies",
            });
        }
        unique_dependency_names(dependencies.as_slice())?;
        let source_chars = draft.bounds.source_chars.to_string();
        let evaluation_fuel = draft.bounds.evaluation_fuel.to_string();
        let dependency_parts = dependencies
            .iter()
            .flat_map(|dependency| {
                [
                    dependency.name(),
                    dependency.version(),
                    dependency.module_identity(),
                ]
            })
            .collect::<Vec<_>>();
        let mut identity_parts = vec![
            STEEL_SKILL_MODULE_SCHEMA,
            STEEL_SKILL_MODULE_COHORT,
            name.as_str(),
            version.as_str(),
            manifest_identity.as_str(),
            frozen_source_identity.as_str(),
            source_identity.as_str(),
            source_chars.as_str(),
            evaluation_fuel.as_str(),
        ];
        identity_parts.extend(exports.iter().map(String::as_str));
        identity_parts.extend(dependency_parts);
        identity_parts.extend(capability_requests.iter().map(String::as_str));
        identity_parts.push(draft.trust.as_str());
        let module_identity = hash_parts(STEEL_SKILL_MODULE_IDENTITY_DOMAIN, &identity_parts);
        Ok(Self {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name,
            version,
            manifest_identity,
            frozen_source_identity,
            source_identity,
            source: String::from(draft.source),
            exports,
            dependencies,
            capability_requests,
            trust: draft.trust,
            bounds: draft.bounds,
            module_identity,
        })
    }

    /// Returns the exact module name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the exact reviewed version.
    #[must_use]
    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    /// Returns the existing manifest identity.
    #[must_use]
    pub fn manifest_identity(&self) -> &str {
        self.manifest_identity.as_str()
    }

    /// Returns the existing frozen-source identity.
    #[must_use]
    pub fn frozen_source_identity(&self) -> &str {
        self.frozen_source_identity.as_str()
    }

    /// Returns the exact source identity.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the private source unit for an admitted loader.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the named exports.
    #[must_use]
    pub fn exports(&self) -> &[String] {
        self.exports.as_slice()
    }

    /// Returns exact dependencies.
    #[must_use]
    pub fn dependencies(&self) -> &[SteelSkillModuleDependency] {
        self.dependencies.as_slice()
    }

    /// Returns requested existing bridge capability labels.
    #[must_use]
    pub fn capability_requests(&self) -> &[String] {
        self.capability_requests.as_slice()
    }

    /// Returns the reviewed trust decision.
    #[must_use]
    pub const fn trust(&self) -> SteelSkillModuleTrust {
        self.trust
    }

    /// Returns the bounded evaluation limits.
    #[must_use]
    pub const fn bounds(&self) -> SteelSkillModuleBounds {
        self.bounds
    }

    /// Returns the domain-separated module identity.
    #[must_use]
    pub fn module_identity(&self) -> &str {
        self.module_identity.as_str()
    }
}

/// Exact authority inputs for module resolution.
#[derive(Clone, Copy, Debug)]
pub struct SteelSkillModuleAuthority<'a> {
    /// Existing session capability labels.
    pub session: &'a [&'a str],
    /// Existing model-profile capability labels.
    pub profile: &'a [&'a str],
    /// Existing selected-skill capability labels.
    pub skill: &'a [&'a str],
    /// Current host capability labels.
    pub host: &'a [&'a str],
}

/// Pure deterministic selected-module plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SteelSkillModulePlan {
    ordered_modules: Vec<SteelSkillModuleDescriptor>,
    module_set_identity: String,
    total_source_chars: u64,
    total_evaluation_fuel: u64,
}

impl SteelSkillModulePlan {
    /// Returns modules in exact dependency-first order.
    #[must_use]
    pub fn ordered_modules(&self) -> &[SteelSkillModuleDescriptor] {
        self.ordered_modules.as_slice()
    }

    /// Returns the selected module-set identity.
    #[must_use]
    pub fn module_set_identity(&self) -> &str {
        self.module_set_identity.as_str()
    }

    /// Returns the cumulative source character count.
    #[must_use]
    pub const fn total_source_chars(&self) -> u64 {
        self.total_source_chars
    }

    /// Returns cumulative evaluation fuel.
    #[must_use]
    pub const fn total_evaluation_fuel(&self) -> u64 {
        self.total_evaluation_fuel
    }
}

/// Resolves an explicitly selected module set under exact capability intersection.
///
/// # Errors
///
/// Returns [`SteelSkillModuleError`] for duplicates, missing or stale
/// dependencies, cycles, export conflicts, trust failures, capability widening,
/// or cumulative bounds.
pub fn resolve_steel_skill_modules(
    modules: &[SteelSkillModuleDescriptor],
    authority: SteelSkillModuleAuthority<'_>,
) -> Result<SteelSkillModulePlan, SteelSkillModuleError> {
    // r[impl agent_harness_runtime.steel_skill_modules.dependencies]
    // r[impl agent_harness_runtime.steel_skill_modules.authority]
    if modules.is_empty() || modules.len() > MAX_MODULES {
        return Err(SteelSkillModuleError::BoundExceeded { field: "modules" });
    }
    let mut by_name = BTreeMap::new();
    for module in modules {
        if module.trust() != SteelSkillModuleTrust::Trusted {
            return Err(SteelSkillModuleError::UntrustedSource);
        }
        if by_name.insert(module.name(), module).is_some() {
            return Err(SteelSkillModuleError::DuplicateModule);
        }
        for capability in module.capability_requests() {
            if !capability_admitted(capability.as_str(), authority) {
                return Err(SteelSkillModuleError::CapabilityWidening);
            }
        }
    }
    let mut export_owner = BTreeMap::new();
    for module in modules {
        for export in module.exports() {
            if export_owner
                .insert(export.as_str(), module.name())
                .is_some()
            {
                return Err(SteelSkillModuleError::ExportConflict);
            }
        }
    }
    let mut traversal = DependencyTraversal {
        by_name: &by_name,
        visiting: BTreeSet::new(),
        visited: BTreeSet::new(),
        ordered: Vec::new(),
    };
    let mut names = by_name.keys().copied().collect::<Vec<_>>();
    names.sort_unstable();
    for name in names {
        traversal.visit(name)?;
    }
    let ordered_names = traversal.ordered;
    let ordered_modules = ordered_names
        .iter()
        .map(|name| by_name[*name].clone())
        .collect::<Vec<_>>();
    let (total_source_chars, total_evaluation_fuel) =
        ordered_modules
            .iter()
            .try_fold((0_u64, 0_u64), |totals, module| {
                let source = totals
                    .0
                    .checked_add(u64::from(module.bounds().source_chars))
                    .ok_or(SteelSkillModuleError::BoundExceeded {
                        field: "total_source_chars",
                    })?;
                let fuel = totals
                    .1
                    .checked_add(module.bounds().evaluation_fuel)
                    .ok_or(SteelSkillModuleError::BoundExceeded {
                        field: "total_evaluation_fuel",
                    })?;
                if source > MAX_TOTAL_SOURCE_CHARS {
                    return Err(SteelSkillModuleError::BoundExceeded {
                        field: "total_source_chars",
                    });
                }
                if fuel > MAX_TOTAL_EVALUATION_FUEL {
                    return Err(SteelSkillModuleError::BoundExceeded {
                        field: "total_evaluation_fuel",
                    });
                }
                Ok((source, fuel))
            })?;
    let module_identities = ordered_modules
        .iter()
        .map(SteelSkillModuleDescriptor::module_identity)
        .collect::<Vec<_>>();
    let module_set_identity = hash_parts(STEEL_SKILL_MODULE_IDENTITY_DOMAIN, &module_identities);
    Ok(SteelSkillModulePlan {
        ordered_modules,
        module_set_identity,
        total_source_chars,
        total_evaluation_fuel,
    })
}

/// Metadata-only module load receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SteelSkillModuleReceipt {
    module_set_identity: String,
    module_count: u32,
    export_count: u32,
    dependency_count: u32,
    capability_count: u32,
    redaction_marker: &'static str,
    non_claim: &'static str,
    receipt_identity: String,
}

impl SteelSkillModuleReceipt {
    /// Projects one metadata-only receipt from an admitted plan.
    #[must_use]
    pub fn from_plan(plan: &SteelSkillModulePlan) -> Self {
        let module_count = u32::try_from(plan.ordered_modules.len()).unwrap_or(u32::MAX);
        let export_count = count_items(plan, SteelSkillModuleDescriptor::exports);
        let dependency_count = count_items(plan, SteelSkillModuleDescriptor::dependencies);
        let capability_count = count_items(plan, SteelSkillModuleDescriptor::capability_requests);
        let counts = [
            module_count.to_string(),
            export_count.to_string(),
            dependency_count.to_string(),
            capability_count.to_string(),
        ];
        let receipt_identity = hash_parts(
            STEEL_SKILL_MODULE_RECEIPT_DOMAIN,
            &[
                plan.module_set_identity(),
                counts[0].as_str(),
                counts[1].as_str(),
                counts[2].as_str(),
                counts[3].as_str(),
                REDACTION_MARKER,
                NON_CLAIM,
            ],
        );
        Self {
            module_set_identity: plan.module_set_identity.clone(),
            module_count,
            export_count,
            dependency_count,
            capability_count,
            redaction_marker: REDACTION_MARKER,
            non_claim: NON_CLAIM,
            receipt_identity,
        }
    }

    /// Returns the exact module-set identity.
    #[must_use]
    pub fn module_set_identity(&self) -> &str {
        self.module_set_identity.as_str()
    }

    /// Returns the safe module count.
    #[must_use]
    pub const fn module_count(&self) -> u32 {
        self.module_count
    }

    /// Returns the metadata-only receipt identity.
    #[must_use]
    pub fn receipt_identity(&self) -> &str {
        self.receipt_identity.as_str()
    }

    /// Returns the required omission marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns bounded non-claims.
    #[must_use]
    pub const fn non_claim(&self) -> &'static str {
        self.non_claim
    }
}

/// Closed Steel skill-module failure classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SteelSkillModuleError {
    /// The descriptor schema was not exact.
    UnsupportedSchema,
    /// A stable label was malformed.
    InvalidLabel {
        /// Stable field name.
        field: &'static str,
    },
    /// A BLAKE3 identity was malformed.
    InvalidIdentity {
        /// Stable field name.
        field: &'static str,
    },
    /// Source bytes did not match the declared identity.
    SourceIdentityMismatch,
    /// Source trust was not current and explicit.
    UntrustedSource,
    /// Source syntax or imports were outside the reviewed cohort.
    SourceDenied,
    /// One named bound was invalid or exceeded.
    BoundExceeded {
        /// Stable bound field.
        field: &'static str,
    },
    /// A set contained duplicate labels.
    DuplicateLabel {
        /// Stable collection field.
        field: &'static str,
    },
    /// Selected modules contained duplicate names.
    DuplicateModule,
    /// A dependency was absent or did not match exact version and identity.
    DependencyMismatch,
    /// The selected dependency graph contained a cycle.
    DependencyCycle,
    /// Two selected modules exported one name.
    ExportConflict,
    /// A selected descriptor did not match an explicitly disclosed skill.
    UnselectedModule,
    /// Module source used a bridge operation that it did not declare.
    UndeclaredCapabilityUse,
    /// A requested capability did not name an existing bridge operation.
    UnknownCapability,
    /// A requested capability was outside the effective intersection.
    CapabilityWidening,
}

impl core::fmt::Display for SteelSkillModuleError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::UnsupportedSchema => "unsupported_schema",
            Self::InvalidLabel { .. } => "invalid_label",
            Self::InvalidIdentity { .. } => "invalid_identity",
            Self::SourceIdentityMismatch => "source_identity_mismatch",
            Self::UntrustedSource => "untrusted_source",
            Self::SourceDenied => "source_denied",
            Self::BoundExceeded { .. } => "bound_exceeded",
            Self::DuplicateLabel { .. } => "duplicate_label",
            Self::DuplicateModule => "duplicate_module",
            Self::DependencyMismatch => "dependency_mismatch",
            Self::DependencyCycle => "dependency_cycle",
            Self::ExportConflict => "export_conflict",
            Self::UnselectedModule => "unselected_module",
            Self::UndeclaredCapabilityUse => "undeclared_capability_use",
            Self::UnknownCapability => "unknown_capability",
            Self::CapabilityWidening => "capability_widening",
        };
        write!(formatter, "Steel skill module rejected: {label}")
    }
}

/// Returns the domain-separated BLAKE3 source identity.
#[must_use]
pub fn steel_skill_module_source_identity(source: &str) -> String {
    hash_parts("animus.steel-skill-module-source.v1", &[source])
}

struct DependencyTraversal<'a, 'b> {
    by_name: &'b BTreeMap<&'a str, &'a SteelSkillModuleDescriptor>,
    visiting: BTreeSet<&'a str>,
    visited: BTreeSet<&'a str>,
    ordered: Vec<&'a str>,
}

impl<'a> DependencyTraversal<'a, '_> {
    fn visit(&mut self, name: &'a str) -> Result<(), SteelSkillModuleError> {
        if self.visited.contains(name) {
            return Ok(());
        }
        if !self.visiting.insert(name) {
            return Err(SteelSkillModuleError::DependencyCycle);
        }
        let module = self
            .by_name
            .get(name)
            .ok_or(SteelSkillModuleError::DependencyMismatch)?;
        let mut dependencies = module.dependencies().iter().collect::<Vec<_>>();
        dependencies.sort_unstable_by(|left, right| left.name().cmp(right.name()));
        for dependency in dependencies {
            let selected = self
                .by_name
                .get(dependency.name())
                .ok_or(SteelSkillModuleError::DependencyMismatch)?;
            if selected.version() != dependency.version()
                || selected.module_identity() != dependency.module_identity()
            {
                return Err(SteelSkillModuleError::DependencyMismatch);
            }
            self.visit(dependency.name())?;
        }
        self.visiting.remove(name);
        self.visited.insert(name);
        self.ordered.push(name);
        Ok(())
    }
}

fn capability_admitted(capability: &str, authority: SteelSkillModuleAuthority<'_>) -> bool {
    [
        authority.session,
        authority.profile,
        authority.skill,
        authority.host,
    ]
    .iter()
    .all(|ceiling| ceiling.contains(&capability))
}

fn validate_source(
    source: &str,
    max_chars: u32,
    capabilities: &[String],
) -> Result<(), SteelSkillModuleError> {
    let chars = u32::try_from(source.chars().count()).unwrap_or(u32::MAX);
    if source.trim().is_empty() || chars > max_chars || source.contains('\0') {
        return Err(SteelSkillModuleError::BoundExceeded { field: "source" });
    }
    if source.contains("#|") {
        return Err(SteelSkillModuleError::SourceDenied);
    }
    let tokens = source_tokens(source);
    let denied_symbols = [
        "call-with-input-file",
        "call-with-output-file",
        "command-line",
        "dynamic-library",
        "environment-variables",
        "eval",
        "getenv",
        "http-get",
        "https-get",
        "include",
        "load",
        "open-input-file",
        "open-output-file",
        "package-manager",
        "require",
        "require-builtin",
        "string->symbol",
        "tcp-connect",
    ];
    if tokens.iter().any(|token| {
        denied_symbols.contains(&token.as_str())
            || token.starts_with("ffi/")
            || token.contains("/ffi/")
            || token == "animus-host-request"
            || token.starts_with("animus-bridge-")
            || token.starts_with("*animus-bridge-")
    }) {
        return Err(SteelSkillModuleError::SourceDenied);
    }
    let denied_references = ["../", "/home/", "/nix/store/", "http://", "https://"];
    if denied_references
        .iter()
        .any(|reference| source.contains(reference))
    {
        return Err(SteelSkillModuleError::SourceDenied);
    }
    let bridge_symbols = [
        ("animus-context-read", "context_read"),
        ("animus-host-edit", "file_edit"),
        ("animus-host-read", "file_read"),
        ("animus-host-shell", "shell"),
        ("animus-host-tool", "tool"),
        ("animus-kernel-clear", "kernel_clear"),
        ("rlm", "subagent"),
    ];
    for (symbol, capability) in bridge_symbols {
        if tokens.iter().any(|token| token == symbol)
            && capabilities
                .binary_search_by(|candidate| candidate.as_str().cmp(capability))
                .is_err()
        {
            return Err(SteelSkillModuleError::UndeclaredCapabilityUse);
        }
    }
    Ok(())
}

fn source_tokens(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut line_comment = false;
    for character in source.chars() {
        if line_comment {
            if character == '\n' {
                line_comment = false;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }
        match character {
            ';' => {
                push_source_token(&mut tokens, &mut token);
                line_comment = true;
            }
            '"' => {
                push_source_token(&mut tokens, &mut token);
                in_string = true;
            }
            value if value.is_whitespace() || "()[]{}'`,".contains(value) => {
                push_source_token(&mut tokens, &mut token);
            }
            value => token.push(value),
        }
    }
    push_source_token(&mut tokens, &mut token);
    tokens
}

fn push_source_token(tokens: &mut Vec<String>, token: &mut String) {
    if !token.is_empty() {
        tokens.push(core::mem::take(token));
    }
}

fn validated_labels(
    field: &'static str,
    values: &[&str],
    max_items: usize,
    max_chars: u32,
) -> Result<Vec<String>, SteelSkillModuleError> {
    if values.is_empty() || values.len() > max_items {
        return Err(SteelSkillModuleError::BoundExceeded { field });
    }
    let mut labels = values
        .iter()
        .map(|value| safe_label(field, value, max_chars))
        .collect::<Result<Vec<_>, _>>()?;
    labels.sort_unstable();
    if labels.windows(2).any(|window| window[0] == window[1]) {
        return Err(SteelSkillModuleError::DuplicateLabel { field });
    }
    Ok(labels)
}

fn validated_optional_labels(
    field: &'static str,
    values: &[&str],
    max_items: usize,
    max_chars: u32,
) -> Result<Vec<String>, SteelSkillModuleError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }
    validated_labels(field, values, max_items, max_chars)
}

fn unique_dependency_names(
    dependencies: &[SteelSkillModuleDependency],
) -> Result<(), SteelSkillModuleError> {
    let mut names = BTreeSet::new();
    for dependency in dependencies {
        if !names.insert(dependency.name()) {
            return Err(SteelSkillModuleError::DuplicateLabel {
                field: "dependencies",
            });
        }
    }
    Ok(())
}

fn safe_label(
    field: &'static str,
    value: &str,
    max_chars: u32,
) -> Result<String, SteelSkillModuleError> {
    let chars = u32::try_from(value.chars().count()).unwrap_or(u32::MAX);
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err(SteelSkillModuleError::InvalidLabel { field });
    };
    if chars > max_chars
        || !(first.is_ascii_lowercase() || first == b'_')
        || !bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'?')
        })
    {
        return Err(SteelSkillModuleError::InvalidLabel { field });
    }
    Ok(String::from(value))
}

fn blake3_identity(field: &'static str, value: &str) -> Result<String, SteelSkillModuleError> {
    if crate::is_lower_hex(value, BLAKE3_HEX_CHARS) {
        Ok(String::from(value))
    } else {
        Err(SteelSkillModuleError::InvalidIdentity { field })
    }
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        let length = u64::try_from(part.len()).unwrap_or(u64::MAX);
        hasher.update(&length.to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn count_items<T>(
    plan: &SteelSkillModulePlan,
    select: impl Fn(&SteelSkillModuleDescriptor) -> &[T],
) -> u32 {
    plan.ordered_modules.iter().fold(0_u32, |total, module| {
        total.saturating_add(u32::try_from(select(module).len()).unwrap_or(u32::MAX))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const FROZEN: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const CAPABILITY: &str = "tool";
    const SOURCE_A: &str = "(define (skill-a value) (+ value 1))";
    const SOURCE_B: &str = "(define (skill-b value) (skill-a value))";

    fn descriptor(
        name: &str,
        source: &str,
        exports: &[&str],
        dependencies: &[SteelSkillModuleDependencyDraft<'_>],
    ) -> SteelSkillModuleDescriptor {
        SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name,
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: steel_skill_module_source_identity(source).as_str(),
            source,
            exports,
            dependencies,
            capability_requests: &[CAPABILITY],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        })
        .expect("descriptor must validate")
    }

    fn authority<'a>() -> SteelSkillModuleAuthority<'a> {
        SteelSkillModuleAuthority {
            session: &[CAPABILITY],
            profile: &[CAPABILITY],
            skill: &[CAPABILITY],
            host: &[CAPABILITY],
        }
    }

    #[test]
    fn descriptor_and_dependency_plan_are_deterministic() {
        let first = descriptor("skill-a", SOURCE_A, &["skill-a"], &[]);
        let dependency = SteelSkillModuleDependencyDraft {
            name: first.name(),
            version: first.version(),
            module_identity: first.module_identity(),
        };
        let second = descriptor("skill-b", SOURCE_B, &["skill-b"], &[dependency]);
        let plan = resolve_steel_skill_modules(&[second, first], authority())
            .expect("dependency order must resolve");
        assert_eq!(plan.ordered_modules()[0].name(), "skill-a");
        assert_eq!(plan.ordered_modules()[1].name(), "skill-b");
        let receipt = SteelSkillModuleReceipt::from_plan(&plan);
        assert_eq!(receipt.module_count(), 2);
        assert!(
            receipt
                .redaction_marker()
                .contains("private-content-omitted")
        );
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "one adversarial source and authority boundary table"
    )]
    fn unsafe_source_cycles_conflicts_and_capability_widening_fail_closed() {
        let source_identity = steel_skill_module_source_identity("(require-builtin steel/process)");
        let denied = SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name: "denied",
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: source_identity.as_str(),
            source: "(require-builtin steel/process)",
            exports: &["denied"],
            dependencies: &[],
            capability_requests: &[CAPABILITY],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        });
        assert_eq!(denied, Err(SteelSkillModuleError::SourceDenied));

        let first = descriptor("skill-a", SOURCE_A, &["same-export"], &[]);
        let second = descriptor("skill-b", SOURCE_B, &["same-export"], &[]);
        assert_eq!(
            resolve_steel_skill_modules(&[first.clone(), second], authority()),
            Err(SteelSkillModuleError::ExportConflict)
        );

        let disguised_require = "( require-builtin steel/process)";
        let disguised_identity = steel_skill_module_source_identity(disguised_require);
        let disguised = SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name: "disguised",
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: disguised_identity.as_str(),
            source: disguised_require,
            exports: &["disguised"],
            dependencies: &[],
            capability_requests: &[],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        });
        assert_eq!(disguised, Err(SteelSkillModuleError::SourceDenied));

        let forged_request = "(define (forged) (list 'animus-host-request \"shell\" \"id\"))";
        let forged_identity = steel_skill_module_source_identity(forged_request);
        let forged = SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name: "forged",
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: forged_identity.as_str(),
            source: forged_request,
            exports: &["forged"],
            dependencies: &[],
            capability_requests: &["shell"],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        });
        assert_eq!(forged, Err(SteelSkillModuleError::SourceDenied));

        let undeclared_source = "(define (undeclared command) (animus-host-shell command))";
        let undeclared_identity = steel_skill_module_source_identity(undeclared_source);
        let undeclared = SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name: "undeclared",
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: undeclared_identity.as_str(),
            source: undeclared_source,
            exports: &["undeclared"],
            dependencies: &[],
            capability_requests: &[],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        });
        assert_eq!(
            undeclared,
            Err(SteelSkillModuleError::UndeclaredCapabilityUse)
        );

        let unknown_source = "(define unknown-capability 1)";
        let unknown_source_identity = steel_skill_module_source_identity(unknown_source);
        let unknown = SteelSkillModuleDescriptor::validate(&SteelSkillModuleDescriptorDraft {
            schema: STEEL_SKILL_MODULE_SCHEMA,
            name: "unknown-capability",
            version: "v1",
            manifest_identity: MANIFEST,
            frozen_source_identity: FROZEN,
            source_identity: unknown_source_identity.as_str(),
            source: unknown_source,
            exports: &["unknown-capability"],
            dependencies: &[],
            capability_requests: &["tool.unknown"],
            trust: SteelSkillModuleTrust::Trusted,
            bounds: SteelSkillModuleBounds {
                source_chars: 1_024,
                evaluation_fuel: 65_536,
            },
        });
        assert_eq!(unknown, Err(SteelSkillModuleError::UnknownCapability));

        let no_host_capability = SteelSkillModuleAuthority {
            host: &[],
            ..authority()
        };
        assert_eq!(
            resolve_steel_skill_modules(&[first], no_host_capability),
            Err(SteelSkillModuleError::CapabilityWidening)
        );
    }
}
