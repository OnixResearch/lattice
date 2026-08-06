use std::collections::BTreeSet;

const SOURCE_REVISION: &str = "0123456789012345678901234567890123456789";
const SOURCE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn policy() -> (
    Vec<super::PortabilityPackageProfile>,
    Vec<super::PortabilityMatrixRequirement>,
    Vec<super::super::RustTargetDescriptor>,
) {
    (
        super::built_in_portability_profiles(),
        super::built_in_portability_matrix_requirements(),
        super::super::built_in_rust_targets(),
    )
}

fn receipt_for(
    requirement: &super::PortabilityMatrixRequirement,
    profile: &super::PortabilityPackageProfile,
    method: super::super::HostEvidenceMethod,
    state: super::super::HostEvidenceState,
) -> super::PortabilityReceipt {
    let runner = if requirement.target == super::super::TARGET_WASM_BROWSER {
        super::super::TARGET_X86_64_LINUX_GNU
    } else {
        requirement.target.as_str()
    };
    super::PortabilityReceipt::new(super::PortabilityReceiptParts {
        id: format!("receipt.{}", requirement.id),
        profile: requirement.profile.clone(),
        target: requirement.target.clone(),
        runner_host: super::RunnerHostScopeId::new(runner),
        grade: requirement.grade,
        method,
        state,
        source_revision: SOURCE_REVISION.to_string(),
        source_hash: SOURCE_HASH.to_string(),
        package_set_hash: profile.package_set_hash.clone(),
        command: requirement.command.clone(),
        command_hash: requirement.command_hash.clone(),
        toolchain: "rustc-1.89.0".to_string(),
        freshness: super::super::EvidenceFreshness::Fresh,
        diagnostic: "-".to_string(),
        non_claims: super::MATRIX_NON_CLAIMS.to_string(),
    })
}

fn context<'a>(
    profiles: &'a [super::PortabilityPackageProfile],
    requirements: &'a [super::PortabilityMatrixRequirement],
    targets: &'a [super::super::RustTargetDescriptor],
) -> super::PortabilityReceiptContext<'a> {
    super::PortabilityReceiptContext {
        source_revision: SOURCE_REVISION,
        source_hash: SOURCE_HASH,
        profiles,
        requirements,
        targets,
    }
}

#[test]
fn profiles_are_generated_from_inventory_without_workspace_or_experimental_expansion() {
    let (profiles, requirements, targets) = policy();
    assert!(super::validate_portability_profiles(&profiles, &requirements, &targets).is_empty());
    assert_eq!(profiles.len(), 9);
    assert_eq!(requirements.len(), 30);

    let pure = profiles.iter().find(|profile| profile.id == "pure-core").expect("pure profile");
    assert_eq!(pure.packages.iter().map(|package| package.package.as_str()).collect::<Vec<_>>(), vec!["ratcore"]);
    let reusable = profiles.iter().find(|profile| profile.id == "reusable-subwayrat").expect("reusable profile");
    assert!(reusable.packages.iter().any(|package| package.package == "rat-keymap"));
    assert!(reusable.packages.iter().all(|package| package.package != "rat-showcase"));
    assert!(reusable.packages.iter().all(|package| package.package != "iced-showcase"));
    let meteorite = profiles.iter().find(|profile| profile.id == "meteorite-web").expect("Meteorite web profile");
    assert_eq!(meteorite.packages.iter().map(|package| package.package.as_str()).collect::<Vec<_>>(), vec![
        "met-core",
        "met-widgets"
    ]);
    let meteorite_components = profiles
        .iter()
        .find(|profile| profile.id == "meteorite-components-wasm")
        .expect("Meteorite component profile");
    assert!(meteorite_components.packages.iter().any(|package| package.package == "met-docking"));
    assert!(meteorite_components.packages.iter().any(|package| package.package == "meteorite"));
    assert!(meteorite_components.packages.iter().all(|package| package.package != "met-showcase"));
    assert!(requirements.iter().any(|requirement| {
        requirement.profile == "reusable-subwayrat"
            && requirement.target == super::super::TARGET_WASM_BROWSER
            && requirement.grade == super::super::HostEvidenceGrade::Compile
    }));
    assert!(requirements.iter().any(|requirement| {
        requirement.profile == "meteorite-components-wasm"
            && requirement.target == super::super::TARGET_WASM_BROWSER
            && requirement.grade == super::super::HostEvidenceGrade::Compile
    }));
    let showcase = profiles
        .iter()
        .find(|profile| profile.id == "meteorite-showcase-web")
        .expect("Meteorite showcase web profile");
    assert_eq!(showcase.packages.iter().map(|package| package.package.as_str()).collect::<Vec<_>>(), vec![
        "met-showcase"
    ]);
    assert!(profiles.iter().all(|profile| profile.id != "workspace"));
}

#[test]
fn malformed_library_profile_drift_fails_closed() {
    let (profiles, requirements, targets) = policy();
    let reusable_index =
        profiles.iter().position(|profile| profile.id == "reusable-subwayrat").expect("reusable profile");

    let mut omitted = profiles.clone();
    omitted[reusable_index].packages.pop();
    let omitted_issues = super::validate_portability_profiles(&omitted, &requirements, &targets);
    assert!(
        omitted_issues
            .iter()
            .any(|issue| issue.code == super::PortabilityMatrixIssueCode::PackageSetMismatch)
    );

    let mut duplicated = profiles.clone();
    let duplicate = duplicated[reusable_index].packages[0].clone();
    duplicated[reusable_index].packages.insert(0, duplicate);
    let duplicate_issues = super::validate_portability_profiles(&duplicated, &requirements, &targets);
    assert!(
        duplicate_issues
            .iter()
            .any(|issue| issue.code == super::PortabilityMatrixIssueCode::PackageSetMismatch)
    );

    let mut reordered = profiles.clone();
    reordered[reusable_index].packages.swap(0, 1);
    let reordered_issues = super::validate_portability_profiles(&reordered, &requirements, &targets);
    assert!(
        reordered_issues
            .iter()
            .any(|issue| issue.code == super::PortabilityMatrixIssueCode::PackageSetMismatch)
    );

    let mut native_scoped = profiles.clone();
    native_scoped[reusable_index].packages[0].target_scope =
        super::super::HostTargetScopeId::new(super::super::SCOPE_NATIVE_UNCLAIMED);
    native_scoped[reusable_index].package_set_hash = super::package_profile_hash(
        native_scoped[reusable_index].id.as_str(),
        native_scoped[reusable_index].role,
        &native_scoped[reusable_index].packages,
    );
    let native_issues = super::validate_portability_profiles(&native_scoped, &requirements, &targets);
    assert!(
        native_issues
            .iter()
            .any(|issue| issue.code == super::PortabilityMatrixIssueCode::PackageSetMismatch)
    );
}

#[test]
fn empty_receipts_keep_required_cells_missing_and_other_cells_explicit() {
    let (profiles, requirements, targets) = policy();
    let build = super::build_portability_matrix(&[], context(&profiles, &requirements, &targets));
    assert!(build.issues.is_empty());
    assert!(!build.all_required_supported());
    assert!(build.cells.iter().any(|cell| {
        cell.profile == "pure-core"
            && cell.target == super::super::TARGET_AARCH64_APPLE_DARWIN
            && cell.status == super::PortabilityMatrixStatus::Missing
    }));
    assert!(build.cells.iter().any(|cell| {
        cell.profile == "experimental"
            && cell.target == super::super::TARGET_X86_64_LINUX_GNU
            && cell.status == super::PortabilityMatrixStatus::Experimental
    }));
    assert!(build.cells.iter().any(|cell| {
        cell.profile == "native-terminal"
            && cell.target == super::super::TARGET_WASM_BROWSER
            && cell.status == super::PortabilityMatrixStatus::Irrelevant
    }));
}

#[test]
fn direct_exact_receipts_support_only_their_profile_target_and_required_grades() {
    let (profiles, requirements, targets) = policy();
    let profile = profiles.iter().find(|profile| profile.id == "pure-core").expect("pure profile");
    let selected = requirements
        .iter()
        .filter(|requirement| {
            requirement.profile == "pure-core" && requirement.target == super::super::TARGET_X86_64_LINUX_GNU
        })
        .map(|requirement| {
            let mut receipt = receipt_for(
                requirement,
                profile,
                super::super::HostEvidenceMethod::Direct,
                super::super::HostEvidenceState::Passed,
            );
            receipt.source_revision = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
            receipt.binding_hash = super::receipt_binding_hash(&receipt);
            receipt
        })
        .collect::<Vec<_>>();
    let build = super::build_portability_matrix(&selected, context(&profiles, &requirements, &targets));

    assert!(build.cells.iter().any(|cell| {
        cell.profile == "pure-core"
            && cell.target == super::super::TARGET_X86_64_LINUX_GNU
            && cell.status == super::PortabilityMatrixStatus::Supported
    }));
    assert!(build.cells.iter().any(|cell| {
        cell.profile == "pure-core"
            && cell.target == super::super::TARGET_X86_64_WINDOWS_MSVC
            && cell.status == super::PortabilityMatrixStatus::Missing
    }));
}

#[test]
fn cross_compile_simulation_and_experimental_evidence_never_promote_native_support() {
    let (profiles, requirements, targets) = policy();
    let profile = profiles.iter().find(|profile| profile.id == "pure-core").expect("pure profile");
    let selected = requirements
        .iter()
        .filter(|requirement| {
            requirement.profile == "pure-core" && requirement.target == super::super::TARGET_X86_64_WINDOWS_MSVC
        })
        .map(|requirement| {
            let method = if requirement.grade == super::super::HostEvidenceGrade::Compile {
                super::super::HostEvidenceMethod::CrossCompile
            } else {
                super::super::HostEvidenceMethod::Direct
            };
            receipt_for(requirement, profile, method, super::super::HostEvidenceState::Passed)
        })
        .collect::<Vec<_>>();
    let build = super::build_portability_matrix(&selected, context(&profiles, &requirements, &targets));
    let windows = build
        .cells
        .iter()
        .find(|cell| cell.profile == "pure-core" && cell.target == super::super::TARGET_X86_64_WINDOWS_MSVC)
        .expect("Windows cell");
    assert_eq!(windows.status, super::PortabilityMatrixStatus::CrossOnly);
    assert!(!build.all_required_supported());

    let methods = [
        super::super::HostEvidenceMethod::Simulated,
        super::super::HostEvidenceMethod::TestBackend,
        super::super::HostEvidenceMethod::SemanticProjection,
        super::super::HostEvidenceMethod::Screenshot,
        super::super::HostEvidenceMethod::Experimental,
    ];
    for method in methods {
        let compile = requirements
            .iter()
            .find(|requirement| {
                requirement.profile == "pure-core"
                    && requirement.target == super::super::TARGET_X86_64_LINUX_GNU
                    && requirement.grade == super::super::HostEvidenceGrade::Compile
            })
            .expect("compile requirement");
        let receipt = receipt_for(compile, profile, method, super::super::HostEvidenceState::Passed);
        let build = super::build_portability_matrix(&[receipt], context(&profiles, &requirements, &targets));
        assert!(build.cells.iter().any(|cell| {
            cell.profile == "pure-core"
                && cell.target == super::super::TARGET_X86_64_LINUX_GNU
                && cell.status != super::PortabilityMatrixStatus::Supported
        }));
    }
}

#[test]
fn stale_copied_changed_unsafe_and_duplicate_receipts_are_rejected() {
    let (profiles, requirements, targets) = policy();
    let profile = profiles.iter().find(|profile| profile.id == "pure-core").expect("pure profile");
    let requirement = requirements
        .iter()
        .find(|requirement| {
            requirement.profile == "pure-core"
                && requirement.target == super::super::TARGET_X86_64_LINUX_GNU
                && requirement.grade == super::super::HostEvidenceGrade::Compile
        })
        .expect("compile requirement");
    let valid = receipt_for(
        requirement,
        profile,
        super::super::HostEvidenceMethod::Direct,
        super::super::HostEvidenceState::Passed,
    );
    let mut stale = valid.clone();
    stale.id = "receipt.stale".to_string();
    stale.source_hash = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into();
    stale.binding_hash = super::receipt_binding_hash(&stale);
    let mut copied = valid.clone();
    copied.id = "receipt.copied".to_string();
    copied.target = super::super::RustTargetId::new(super::super::TARGET_X86_64_WINDOWS_MSVC);
    let mut changed = valid.clone();
    changed.id = "receipt.changed".to_string();
    changed.package_set_hash = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".into();
    changed.binding_hash = super::receipt_binding_hash(&changed);
    let mut wrong_command = valid.clone();
    wrong_command.id = "receipt.wrong-command".to_string();
    wrong_command.command = super::PortabilityCommandId::new("portability.other.compile.v1");
    wrong_command.command_hash = super::portability_command_hash(wrong_command.command.as_str());
    wrong_command.binding_hash = super::receipt_binding_hash(&wrong_command);
    let mut unsafe_receipt = valid.clone();
    unsafe_receipt.id = "receipt.unsafe".to_string();
    unsafe_receipt.diagnostic = "/home/runner/raw\nlog".to_string();
    unsafe_receipt.binding_hash = super::receipt_binding_hash(&unsafe_receipt);
    let mut duplicate = valid.clone();
    duplicate.id = "receipt.duplicate".to_string();
    duplicate.binding_hash = super::receipt_binding_hash(&duplicate);
    let mut malformed_revision = valid.clone();
    malformed_revision.id = "receipt.malformed-revision".to_string();
    malformed_revision.source_revision = "main".to_string();
    malformed_revision.binding_hash = super::receipt_binding_hash(&malformed_revision);

    let issues = super::validate_portability_receipts(
        &[
            valid,
            stale,
            copied,
            changed,
            wrong_command,
            unsafe_receipt,
            duplicate,
            malformed_revision,
        ],
        context(&profiles, &requirements, &targets),
    );
    let codes = issues.iter().map(|issue| issue.code).collect::<BTreeSet<_>>();
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::StaleSource));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::BindingHashMismatch));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::CommandMismatch));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::PackageSetMismatch));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::UnsafeDiagnostic));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::DuplicateReceipt));
    assert!(codes.contains(&super::PortabilityMatrixIssueCode::MalformedReceipt));
}

#[test]
fn receipt_parser_round_trips_and_rejects_raw_or_oversized_shapes() {
    let (profiles, requirements, _) = policy();
    let requirement = &requirements[0];
    let profile = profiles.iter().find(|profile| profile.id == requirement.profile).expect("profile");
    let receipt = receipt_for(
        requirement,
        profile,
        super::super::HostEvidenceMethod::Direct,
        super::super::HostEvidenceState::Blocked,
    );
    let encoded = super::render_portability_receipts_tsv(&[receipt.clone()]);
    let parsed = super::parse_portability_receipts_tsv(&encoded);
    assert!(parsed.issues.is_empty());
    assert_eq!(parsed.receipts, vec![receipt]);

    let malformed = format!("{}\nraw\tlog\n", encoded.lines().next().expect("header"));
    let parsed = super::parse_portability_receipts_tsv(&malformed);
    assert!(parsed.receipts.is_empty());
    assert_eq!(parsed.issues[0].code, super::PortabilityMatrixIssueCode::MalformedReceipt);
}

#[test]
fn generated_profiles_requirements_and_empty_matrix_are_deterministic() {
    let (profiles, requirements, targets) = policy();
    let build = super::build_portability_matrix(&[], context(&profiles, &requirements, &targets));
    assert_eq!(super::render_portability_profiles_tsv(&profiles), super::render_portability_profiles_tsv(&profiles));
    assert_eq!(
        super::render_portability_requirements_tsv(&requirements),
        super::render_portability_requirements_tsv(&requirements)
    );
    assert_eq!(super::render_portability_matrix_tsv(&build), super::render_portability_matrix_tsv(&build));
}
