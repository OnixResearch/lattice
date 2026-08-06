//! Deterministic package-profile and exact-cell generation from accepted inventory.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;

const PROFILE_PURE_CORE: &str = "pure-core";
const PROFILE_REUSABLE_SUBWAYRAT: &str = "reusable-subwayrat";
const PROFILE_NATIVE_TERMINAL: &str = "native-terminal";
const PROFILE_RATZILLA_WEB: &str = "ratzilla-web";
const PROFILE_METEORITE_WEB: &str = "meteorite-web";
const PROFILE_METEORITE_COMPONENTS: &str = "meteorite-components-wasm";
const PROFILE_METEORITE_SHOWCASE_WEB: &str = "meteorite-showcase-web";
const PROFILE_EXPERIMENTAL: &str = "experimental";
const PROFILE_EVIDENCE_ONLY: &str = "evidence-only";

/// Returns generated ordered package profiles from the accepted inventory.
#[must_use]
pub fn built_in_portability_profiles() -> Vec<super::PortabilityPackageProfile> {
    let rows = super::super::built_in_package_surfaces();
    let base = rows.iter().filter(|row| row.id.as_str().starts_with("package.")).collect::<Vec<_>>();
    let mut profiles = vec![
        profile(
            PROFILE_PURE_CORE,
            super::PortabilityProfileRole::PureCore,
            base.iter().copied().filter(|row| row.role == super::super::PortabilityRole::PureCore),
        ),
        profile(
            PROFILE_REUSABLE_SUBWAYRAT,
            super::PortabilityProfileRole::ReusableSubwayrat,
            base.iter().copied().filter(|row| {
                row.owner == "subwayrat-maintainers"
                    && row.role == super::super::PortabilityRole::ReusableLibrary
                    && row.disposition == super::super::PortabilityDisposition::Conforming
            }),
        ),
        profile(
            PROFILE_NATIVE_TERMINAL,
            super::PortabilityProfileRole::NativeTerminal,
            base.iter().copied().filter(|row| {
                row.owner == "subwayrat-maintainers"
                    && row.target_scope == super::super::SCOPE_NATIVE_UNCLAIMED
                    && matches!(
                        row.role,
                        super::super::PortabilityRole::BackendAdapter | super::super::PortabilityRole::ApplicationShell
                    )
            }),
        ),
        profile(
            PROFILE_RATZILLA_WEB,
            super::PortabilityProfileRole::RatzillaWeb,
            base.iter().copied().filter(|row| {
                row.owner == "subwayrat-maintainers"
                    && row.target_scope == super::super::SCOPE_BROWSER_WASM
                    && row.disposition == super::super::PortabilityDisposition::Conforming
            }),
        ),
        profile(
            PROFILE_METEORITE_WEB,
            super::PortabilityProfileRole::MeteoriteWeb,
            base.iter().copied().filter(|row| {
                row.owner == "meteorite-maintainers"
                    && matches!(row.package.as_str(), "met-core" | "met-widgets")
                    && row.disposition == super::super::PortabilityDisposition::Conforming
            }),
        ),
        profile(
            PROFILE_METEORITE_COMPONENTS,
            super::PortabilityProfileRole::MeteoriteComponents,
            base.iter().copied().filter(|row| {
                row.owner == "meteorite-maintainers"
                    && row.role == super::super::PortabilityRole::ReusableLibrary
                    && row.disposition == super::super::PortabilityDisposition::Conforming
            }),
        ),
        profile(
            PROFILE_METEORITE_SHOWCASE_WEB,
            super::PortabilityProfileRole::MeteoriteShowcaseWeb,
            base.iter().copied().filter(|row| {
                row.package == "met-showcase"
                    && row.role == super::super::PortabilityRole::ApplicationShell
                    && row.target_scope == super::super::SCOPE_BROWSER_WASM
                    && row.disposition == super::super::PortabilityDisposition::Conforming
            }),
        ),
        profile(
            PROFILE_EXPERIMENTAL,
            super::PortabilityProfileRole::Experimental,
            base.iter().copied().filter(|row| row.owner == "iced-pilot-maintainers"),
        ),
        profile(
            PROFILE_EVIDENCE_ONLY,
            super::PortabilityProfileRole::EvidenceOnly,
            base.iter().copied().filter(|row| {
                row.role == super::super::PortabilityRole::EvidenceHarness
                    || row.disposition == super::super::PortabilityDisposition::EvidenceOnly
            }),
        ),
    ];
    profiles.sort_by(|left, right| left.id.cmp(&right.id));
    profiles
}

fn profile<'a>(
    id: &'static str,
    role: super::PortabilityProfileRole,
    rows: impl Iterator<Item = &'a super::super::PackageSurface>,
) -> super::PortabilityPackageProfile {
    let mut packages = BTreeMap::new();
    for row in rows {
        packages.entry(row.package.clone()).or_insert_with(|| super::PortabilityProfilePackage {
            package: row.package.clone(),
            default_features: row.default_features.clone(),
            role: row.role,
            target_scope: row.target_scope.clone(),
        });
    }
    let packages = packages.into_values().collect::<Vec<_>>();
    let package_set_hash = package_profile_hash(id, role, &packages);
    super::PortabilityPackageProfile {
        id: super::PortabilityProfileId::new(id),
        role,
        packages,
        package_set_hash,
    }
}

/// Computes the deterministic package-profile identity hash.
#[must_use]
pub fn package_profile_hash(
    id: &str,
    role: super::PortabilityProfileRole,
    packages: &[super::PortabilityProfilePackage],
) -> String {
    let mut input = format!("profile\t{id}\t{}\n", role.as_str());
    for package in packages {
        writeln!(
            input,
            "{}\t{}\t{}\t{}",
            package.package,
            package.default_features,
            package.role.as_str(),
            package.target_scope
        )
        .expect("writing to String cannot fail");
    }
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

/// Returns all exact evidence requirements for maintained profile/target cells.
#[must_use]
pub fn built_in_portability_matrix_requirements() -> Vec<super::PortabilityMatrixRequirement> {
    let native_targets = [
        super::super::TARGET_X86_64_LINUX_GNU,
        super::super::TARGET_X86_64_WINDOWS_MSVC,
        super::super::TARGET_AARCH64_APPLE_DARWIN,
    ];
    let mut requirements = Vec::new();
    for target in native_targets {
        for profile in [PROFILE_PURE_CORE, PROFILE_REUSABLE_SUBWAYRAT] {
            requirements.push(requirement(profile, target, super::super::HostEvidenceGrade::Compile, true));
            requirements.push(requirement(profile, target, super::super::HostEvidenceGrade::NativeHostTest, false));
        }
        for grade in [
            super::super::HostEvidenceGrade::Compile,
            super::super::HostEvidenceGrade::NativeHostTest,
            super::super::HostEvidenceGrade::NativeRuntimeSmoke,
        ] {
            requirements.push(requirement(
                PROFILE_NATIVE_TERMINAL,
                target,
                grade,
                grade == super::super::HostEvidenceGrade::Compile,
            ));
        }
    }

    for profile in [
        PROFILE_PURE_CORE,
        PROFILE_REUSABLE_SUBWAYRAT,
        PROFILE_METEORITE_COMPONENTS,
    ] {
        requirements.push(requirement(
            profile,
            super::super::TARGET_WASM_BROWSER,
            super::super::HostEvidenceGrade::Compile,
            true,
        ));
    }
    for profile in [
        PROFILE_RATZILLA_WEB,
        PROFILE_METEORITE_WEB,
        PROFILE_METEORITE_SHOWCASE_WEB,
    ] {
        requirements.push(requirement(
            profile,
            super::super::TARGET_WASM_BROWSER,
            super::super::HostEvidenceGrade::Compile,
            true,
        ));
        requirements.push(requirement(
            profile,
            super::super::TARGET_WASM_BROWSER,
            super::super::HostEvidenceGrade::BrowserRuntime,
            false,
        ));
    }
    requirements.sort_by(|left, right| left.id.cmp(&right.id));
    requirements
}

fn requirement(
    profile: &str,
    target: &str,
    grade: super::super::HostEvidenceGrade,
    allows_cross_compile: bool,
) -> super::PortabilityMatrixRequirement {
    let command = format!("portability.{profile}.{}.v1", grade.as_str());
    let id = format!("cell.{profile}.{target}.{}", grade.as_str());
    super::PortabilityMatrixRequirement {
        id,
        profile: super::PortabilityProfileId::new(profile),
        target: super::super::RustTargetId::new(target),
        grade,
        command: super::PortabilityCommandId::new(command.clone()),
        command_hash: portability_command_hash(&command),
        allows_cross_compile,
    }
}

/// Hashes one normalized repository-owned command identity.
#[must_use]
pub fn portability_command_hash(command: &str) -> String {
    blake3::hash(format!("rats-portability-command\t{command}\n").as_bytes()).to_hex().to_string()
}

/// Validates profile freshness and requirement references.
#[must_use]
pub fn validate_portability_profiles(
    profiles: &[super::PortabilityPackageProfile],
    requirements: &[super::PortabilityMatrixRequirement],
    targets: &[super::super::RustTargetDescriptor],
) -> Vec<super::PortabilityMatrixIssue> {
    let mut issues = Vec::new();
    let mut profile_ids = BTreeSet::new();
    for profile in profiles {
        if !profile_ids.insert(profile.id.clone()) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::DuplicateIdentity,
                profile.id.as_str(),
                "duplicate profile identity",
            ));
        }
        if profile.packages.is_empty() {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::EmptyField,
                profile.id.as_str(),
                "profile package set is empty",
            ));
        }
        if profile.package_set_hash != package_profile_hash(profile.id.as_str(), profile.role, &profile.packages) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::PackageSetMismatch,
                profile.id.as_str(),
                "profile package hash is stale",
            ));
        }
        if !profile.packages.windows(2).all(|pair| pair[0] < pair[1]) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::PackageSetMismatch,
                profile.id.as_str(),
                "profile packages are not strictly ordered",
            ));
        }
        for package in &profile.packages {
            let role_matches = match profile.role {
                super::PortabilityProfileRole::PureCore => package.role == super::super::PortabilityRole::PureCore,
                super::PortabilityProfileRole::ReusableSubwayrat
                | super::PortabilityProfileRole::MeteoriteComponents => {
                    package.role == super::super::PortabilityRole::ReusableLibrary
                        && package.target_scope != super::super::SCOPE_NATIVE_UNCLAIMED
                }
                super::PortabilityProfileRole::NativeTerminal => matches!(
                    package.role,
                    super::super::PortabilityRole::BackendAdapter | super::super::PortabilityRole::ApplicationShell
                ),
                super::PortabilityProfileRole::RatzillaWeb => package.target_scope == super::super::SCOPE_BROWSER_WASM,
                super::PortabilityProfileRole::MeteoriteWeb => {
                    package.role == super::super::PortabilityRole::ReusableLibrary
                }
                super::PortabilityProfileRole::MeteoriteShowcaseWeb => {
                    package.role == super::super::PortabilityRole::ApplicationShell
                        && package.target_scope == super::super::SCOPE_BROWSER_WASM
                }
                super::PortabilityProfileRole::Experimental | super::PortabilityProfileRole::EvidenceOnly => true,
            };
            if !role_matches {
                issues.push(issue(
                    super::PortabilityMatrixIssueCode::PackageSetMismatch,
                    format!("{}:{}", profile.id, package.package),
                    "profile package role or target scope is incompatible",
                ));
            }
        }
    }
    let target_ids = targets.iter().map(|target| target.id.clone()).collect::<BTreeSet<_>>();
    let mut requirement_ids = BTreeSet::new();
    for requirement in requirements {
        if !requirement_ids.insert(requirement.id.clone()) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::DuplicateIdentity,
                &requirement.id,
                "duplicate requirement identity",
            ));
        }
        if !profile_ids.contains(&requirement.profile) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::UnknownProfile,
                &requirement.id,
                "requirement references unknown profile",
            ));
        }
        if !target_ids.contains(&requirement.target) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::UnknownTarget,
                &requirement.id,
                "requirement references unknown target",
            ));
        }
        if requirement.command_hash != portability_command_hash(requirement.command.as_str()) {
            issues.push(issue(
                super::PortabilityMatrixIssueCode::CommandMismatch,
                &requirement.id,
                "requirement command hash is stale",
            ));
        }
    }
    issues.sort();
    issues
}

fn issue(
    code: super::PortabilityMatrixIssueCode,
    subject: impl Into<String>,
    message: &'static str,
) -> super::PortabilityMatrixIssue {
    super::PortabilityMatrixIssue {
        code,
        subject: subject.into(),
        message: message.to_string(),
    }
}
