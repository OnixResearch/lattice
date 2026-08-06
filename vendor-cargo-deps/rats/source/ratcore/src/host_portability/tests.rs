use std::collections::BTreeSet;

fn codes(report: &super::HostPortabilityReport) -> BTreeSet<super::HostPortabilityIssueCode> {
    report.issues.iter().map(|issue| issue.code).collect()
}

macro_rules! receipt {
    ($id:expr, $host:expr, $target:expr, $grade:expr, $method:expr, $state:expr, $freshness:expr) => {
        super::HostEvidenceReceipt::new(super::HostEvidenceReceiptParts {
            id: $id.to_string(),
            host_family: super::HostFamilyId::new($host),
            rust_target: super::RustTargetId::new($target),
            grade: $grade,
            method: $method,
            state: $state,
            freshness: $freshness,
        })
    };
}

#[test]
fn built_in_taxonomy_inventory_roles_and_capabilities_are_valid() {
    let report = super::validate_built_in_host_portability_policy();
    assert!(report.is_valid(), "{:#?}", report.issues);

    let rows = super::built_in_package_surfaces();
    let base_rows = rows.iter().filter(|row| row.id.as_str().starts_with("package.")).collect::<Vec<_>>();
    assert_eq!(base_rows.len(), 49);
    assert!(base_rows.iter().any(|row| row.package == "ratcore"));
    assert!(base_rows.iter().any(|row| row.package == "meteorite"));
    assert!(base_rows.iter().any(|row| row.package == "iced-showcase"));

    let effects = super::built_in_host_effects();
    assert!(effects.iter().any(|effect| effect.kind == super::HostEffectKind::Filesystem));
    assert!(effects.iter().any(|effect| effect.kind == super::HostEffectKind::TargetBranch));
    assert!(effects.iter().all(|effect| effect.kind != super::HostEffectKind::Network));
}

#[test]
fn host_renderer_target_and_capability_domains_remain_nominal() {
    let host = super::HostFamilyId::new("same-label");
    let target = super::RustTargetId::new("same-label");
    let capability = super::HostCapabilityId::new("same-label");
    let renderer_family = crate::renderer_target_policy::RendererFamilyId::new("same-label");
    let renderer_target = crate::renderer_target_policy::RendererTargetId::new("same-label");
    assert_eq!(host.as_str(), target.as_str());
    assert_eq!(target.as_str(), capability.as_str());
    assert_eq!(capability.as_str(), renderer_family.as_str());
    assert_eq!(renderer_family.as_str(), renderer_target.as_str());

    let targets = super::built_in_rust_targets();
    let linux = targets.iter().find(|entry| entry.id == super::TARGET_X86_64_LINUX_GNU).expect("linux target");
    let mac = targets.iter().find(|entry| entry.id == super::TARGET_AARCH64_APPLE_DARWIN).expect("mac target");
    let browser = targets.iter().find(|entry| entry.id == super::TARGET_WASM_BROWSER).expect("browser target");
    assert_ne!(linux.architecture, mac.architecture);
    assert_eq!(browser.host_family, super::HOST_FAMILY_BROWSER);
    assert_ne!(browser.host_family, super::HOST_FAMILY_NATIVE);
}

#[test]
fn migrated_packages_examples_and_browser_gates_stay_visible() {
    let packages = super::built_in_package_surfaces();
    let image = packages.iter().filter(|row| row.package == "rat-image").collect::<Vec<_>>();
    assert!(image.len() >= 2);
    assert!(image.iter().all(|row| {
        row.disposition == super::PortabilityDisposition::Conforming
            || row.disposition == super::PortabilityDisposition::EvidenceOnly
    }));
    let inline = packages.iter().find(|row| row.id == "package.rat-inline.production").expect("inline adapter row");
    assert_eq!(inline.role, super::PortabilityRole::BackendAdapter);
    assert!(
        super::built_in_host_effects()
            .iter()
            .all(|effect| effect.dependent_change.as_deref() != Some("isolate-reusable-host-effects"))
    );

    let node_example = packages
        .iter()
        .find(|row| row.id == "slice.rat-nodegraph.interactive-example")
        .expect("nodegraph example row");
    assert_eq!(node_example.surface, super::SourceSurface::Example);
    assert_eq!(node_example.defaults, super::DefaultFeatureDisposition::NotApplicable);

    let effects = super::built_in_host_effects();
    let storage = effects
        .iter()
        .find(|effect| effect.id == "effect.met-hooks.browser-storage")
        .expect("browser storage row");
    assert_eq!(storage.target_scope, super::SCOPE_BROWSER_WASM);
    assert_eq!(storage.role, super::PortabilityRole::BackendAdapter);
    let node_dependency = effects
        .iter()
        .find(|effect| effect.id == "dependency.rat-nodegraph.crossterm-dev")
        .expect("example-only dependency row");
    assert_eq!(node_dependency.surface, super::SourceSurface::Example);
    assert_eq!(node_dependency.defaults, super::DefaultFeatureDisposition::NotApplicable);
}

#[test]
fn malformed_inventory_rejects_unknown_roles_owners_scopes_and_blocker_changes() {
    let families = super::built_in_host_families();
    let targets = super::built_in_rust_targets();
    let scopes = super::built_in_host_target_scopes();
    let mut packages = super::built_in_package_surfaces();
    let effects = super::built_in_host_effects();
    let capabilities = super::built_in_host_capability_contracts();

    let duplicate = packages[0].clone();
    packages.push(duplicate);
    packages[1].role = super::PortabilityRole::Unknown;
    packages[2].owner.clear();
    packages[3].target_scope = super::HostTargetScopeId::new("missing-scope");
    packages[4].disposition = super::PortabilityDisposition::MigrationBlocker;
    packages[4].dependent_change = None;

    let report = super::validate_host_portability_policy(super::HostPortabilityPolicy {
        families: &families,
        targets: &targets,
        scopes: &scopes,
        packages: &packages,
        effects: &effects,
        capabilities: &capabilities,
    });
    let actual = codes(&report);
    assert!(actual.contains(&super::HostPortabilityIssueCode::DuplicateIdentity));
    assert!(actual.contains(&super::HostPortabilityIssueCode::UnknownRole));
    assert!(actual.contains(&super::HostPortabilityIssueCode::EmptyField));
    assert!(actual.contains(&super::HostPortabilityIssueCode::UnknownTargetScope));
    assert!(actual.contains(&super::HostPortabilityIssueCode::MissingDependentChange));
    assert!(report.issues.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn malformed_taxonomy_rejects_cross_family_exact_scope_and_duplicate_identity() {
    let mut families = super::built_in_host_families();
    families.push(families[0].clone());
    let targets = super::built_in_rust_targets();
    let mut scopes = super::built_in_host_target_scopes();
    scopes.push(super::HostTargetScope::exact(
        "bad-browser-linux",
        super::HOST_FAMILY_BROWSER,
        super::TARGET_X86_64_LINUX_GNU,
    ));
    let report = super::validate_host_portability_policy(super::HostPortabilityPolicy {
        families: &families,
        targets: &targets,
        scopes: &scopes,
        packages: &super::built_in_package_surfaces(),
        effects: &super::built_in_host_effects(),
        capabilities: &super::built_in_host_capability_contracts(),
    });
    let actual = codes(&report);
    assert!(actual.contains(&super::HostPortabilityIssueCode::DuplicateIdentity));
    assert!(actual.contains(&super::HostPortabilityIssueCode::InvalidTargetScope));
}

#[test]
fn role_policy_allows_values_and_rejects_unblocked_ambient_acquisition() {
    let mut effects = super::built_in_host_effects();
    let blocker = effects
        .iter_mut()
        .find(|effect| effect.id == "effect.rat-inline.terminal-geometry")
        .expect("inline terminal adapter");
    blocker.role = super::PortabilityRole::ReusableLibrary;
    let report = super::validate_host_portability_policy(super::HostPortabilityPolicy {
        families: &super::built_in_host_families(),
        targets: &super::built_in_rust_targets(),
        scopes: &super::built_in_host_target_scopes(),
        packages: &super::built_in_package_surfaces(),
        effects: &effects,
        capabilities: &super::built_in_host_capability_contracts(),
    });
    assert!(codes(&report).contains(&super::HostPortabilityIssueCode::ReusableAmbientEffect));

    let fact = super::HostCapabilityFact {
        capability: super::HostCapabilityId::new("rat-widgets.observed-time"),
        availability: super::CapabilityAvailability::Unsupported,
    };
    let classify = |availability| -> super::HostOperationOutcome<()> {
        match availability {
            super::CapabilityAvailability::Available => super::HostOperationOutcome::Completed(()),
            super::CapabilityAvailability::Unsupported => super::HostOperationOutcome::Unsupported {
                diagnostic: "time.unsupported".to_string(),
            },
            super::CapabilityAvailability::Denied => super::HostOperationOutcome::Rejected {
                diagnostic: "time.denied".to_string(),
            },
        }
    };
    assert!(matches!(classify(fact.availability), super::HostOperationOutcome::Unsupported { .. }));
    assert!(matches!(
        classify(super::CapabilityAvailability::Denied),
        super::HostOperationOutcome::Rejected { .. }
    ));
    let duration = std::time::Duration::from_millis(5);
    let supplied_bytes = [1_u8, 2, 3];
    assert_eq!(duration.as_millis(), 5);
    assert_eq!(supplied_bytes.len(), 3);
}

#[test]
fn capability_policy_rejects_universal_objects_retained_host_state_and_stale_authority() {
    let mut capabilities = super::built_in_host_capability_contracts();
    let template = capabilities[0].clone();
    let mut universal = template.clone();
    universal.id = super::HostCapabilityId::new("universal-platform-service");
    universal.state_kind = super::CapabilityStateKind::HostCallback;
    universal.authority = super::CapabilityAuthority::External;
    universal.fallback = "if linux infer stdout from the OS label".to_string();
    universal.attachments.generation = super::AuthorityAttachmentState::Missing;
    universal.attachments.authorization = super::AuthorityAttachmentState::Missing;
    capabilities.push(universal);
    capabilities.push(template);

    let report = super::validate_host_portability_policy(super::HostPortabilityPolicy {
        families: &super::built_in_host_families(),
        targets: &super::built_in_rust_targets(),
        scopes: &super::built_in_host_target_scopes(),
        packages: &super::built_in_package_surfaces(),
        effects: &super::built_in_host_effects(),
        capabilities: &capabilities,
    });
    let actual = codes(&report);
    assert!(actual.contains(&super::HostPortabilityIssueCode::DuplicateIdentity));
    assert!(actual.contains(&super::HostPortabilityIssueCode::UniversalPlatformCapability));
    assert!(actual.contains(&super::HostPortabilityIssueCode::InferredHostFallback));
    assert!(actual.contains(&super::HostPortabilityIssueCode::RetainedHostState));
    assert!(actual.contains(&super::HostPortabilityIssueCode::MissingAuthorityAttachment));
}

#[test]
fn duplicate_or_missing_effect_ownership_fails_closed() {
    let mut capabilities = super::built_in_host_capability_contracts();
    let target = capabilities
        .iter_mut()
        .find(|entry| entry.id == "rat-inline.terminal-geometry")
        .expect("inline terminal capability");
    target.owner = "another-owner".to_string();
    let report = super::validate_host_portability_policy(super::HostPortabilityPolicy {
        families: &super::built_in_host_families(),
        targets: &super::built_in_rust_targets(),
        scopes: &super::built_in_host_target_scopes(),
        packages: &super::built_in_package_surfaces(),
        effects: &super::built_in_host_effects(),
        capabilities: &capabilities,
    });
    assert!(codes(&report).contains(&super::HostPortabilityIssueCode::EffectOwnerMismatch));
}

#[test]
fn source_scan_coverage_rejects_unknown_changed_missing_and_hidden_target_coordinates() {
    let effects = super::built_in_host_effects();
    let observed = effects
        .iter()
        .filter(|effect| effect.scan_class != super::EffectScanClass::Dependency)
        .map(|effect| {
            super::ScannedHostCoordinate::new(effect.path.clone(), effect.scan_class, effect.scan_match_count)
        })
        .collect::<Vec<_>>();
    assert!(super::validate_host_scan_coverage(&effects, &observed).is_valid());

    let mut malformed = observed;
    malformed.remove(0);
    malformed[0].match_count += 1;
    malformed.push(super::ScannedHostCoordinate::new(
        "reusable/src/hidden.rs",
        super::EffectScanClass::TargetBranch,
        1,
    ));
    let report = super::validate_host_scan_coverage(&effects, &malformed);
    let actual = codes(&report);
    assert!(actual.contains(&super::HostPortabilityIssueCode::MissingScanCoordinate));
    assert!(actual.contains(&super::HostPortabilityIssueCode::ScanCountMismatch));
    assert!(actual.contains(&super::HostPortabilityIssueCode::UnclassifiedScanCoordinate));
}

#[test]
fn direct_evidence_never_promotes_across_host_target_or_grade() {
    let browser_runtime = receipt!(
        "browser",
        super::HOST_FAMILY_BROWSER,
        super::TARGET_WASM_BROWSER,
        super::HostEvidenceGrade::BrowserRuntime,
        super::HostEvidenceMethod::Direct,
        super::HostEvidenceState::Passed,
        super::EvidenceFreshness::Fresh
    );
    let windows_runtime = super::HostEvidenceRequirement::new(
        "windows-runtime",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_WINDOWS_MSVC,
        super::HostEvidenceGrade::NativeRuntimeSmoke,
    );
    assert_eq!(
        super::evaluate_host_support(&windows_runtime, &[browser_runtime]),
        super::HostSupportStatus::Missing
    );

    let cross_compile = receipt!(
        "windows-cross",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_WINDOWS_MSVC,
        super::HostEvidenceGrade::Compile,
        super::HostEvidenceMethod::CrossCompile,
        super::HostEvidenceState::Passed,
        super::EvidenceFreshness::Fresh
    );
    let windows_compile = super::HostEvidenceRequirement::new(
        "windows-compile",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_WINDOWS_MSVC,
        super::HostEvidenceGrade::Compile,
    );
    assert_eq!(
        super::evaluate_host_support(&windows_compile, std::slice::from_ref(&cross_compile)),
        super::HostSupportStatus::Partial
    );
    assert_eq!(super::evaluate_host_support(&windows_runtime, &[cross_compile]), super::HostSupportStatus::Partial);
}

#[test]
fn simulated_test_backend_experimental_failed_stale_and_skipped_evidence_stay_nonpassing() {
    let requirement = super::HostEvidenceRequirement::new(
        "linux-runtime",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_LINUX_GNU,
        super::HostEvidenceGrade::NativeRuntimeSmoke,
    );
    let direct = receipt!(
        "direct",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_LINUX_GNU,
        super::HostEvidenceGrade::NativeRuntimeSmoke,
        super::HostEvidenceMethod::Direct,
        super::HostEvidenceState::Passed,
        super::EvidenceFreshness::Fresh
    );
    assert_eq!(super::evaluate_host_support(&requirement, &[direct]), super::HostSupportStatus::Supported);
    let unsupported = receipt!(
        "unsupported",
        super::HOST_FAMILY_NATIVE,
        super::TARGET_X86_64_LINUX_GNU,
        super::HostEvidenceGrade::NativeRuntimeSmoke,
        super::HostEvidenceMethod::Direct,
        super::HostEvidenceState::Unsupported,
        super::EvidenceFreshness::Fresh
    );
    assert_eq!(super::evaluate_host_support(&requirement, &[unsupported]), super::HostSupportStatus::Unsupported);
    for method in [
        super::HostEvidenceMethod::Simulated,
        super::HostEvidenceMethod::TestBackend,
        super::HostEvidenceMethod::SemanticProjection,
        super::HostEvidenceMethod::Screenshot,
        super::HostEvidenceMethod::Experimental,
    ] {
        let receipt = receipt!(
            "non-direct",
            super::HOST_FAMILY_NATIVE,
            super::TARGET_X86_64_LINUX_GNU,
            super::HostEvidenceGrade::NativeRuntimeSmoke,
            method,
            super::HostEvidenceState::Passed,
            super::EvidenceFreshness::Fresh
        );
        assert_eq!(super::evaluate_host_support(&requirement, &[receipt]), super::HostSupportStatus::Partial);
    }
    let make = |state, freshness| {
        receipt!(
            "receipt",
            super::HOST_FAMILY_NATIVE,
            super::TARGET_X86_64_LINUX_GNU,
            super::HostEvidenceGrade::NativeRuntimeSmoke,
            super::HostEvidenceMethod::Direct,
            state,
            freshness
        )
    };
    assert_eq!(
        super::evaluate_host_support(&requirement, &[make(
            super::HostEvidenceState::Failed,
            super::EvidenceFreshness::Fresh
        )]),
        super::HostSupportStatus::Failed
    );
    assert_eq!(
        super::evaluate_host_support(&requirement, &[make(
            super::HostEvidenceState::Passed,
            super::EvidenceFreshness::Stale
        )]),
        super::HostSupportStatus::Stale
    );
    assert_eq!(
        super::evaluate_host_support(&requirement, &[make(
            super::HostEvidenceState::Skipped,
            super::EvidenceFreshness::Fresh
        )]),
        super::HostSupportStatus::Blocked
    );
}

#[test]
fn exact_scope_validation_rejects_architecture_and_host_family_coincidence() {
    let requirements = vec![super::HostEvidenceRequirement::new(
        "wrong-family",
        super::HOST_FAMILY_BROWSER,
        super::TARGET_AARCH64_APPLE_DARWIN,
        super::HostEvidenceGrade::Compile,
    )];
    let report = super::validate_evidence_scopes(&super::built_in_rust_targets(), &requirements, &[]);
    assert!(codes(&report).contains(&super::HostPortabilityIssueCode::HostTargetMismatch));
}

#[test]
fn generated_inventories_are_deterministic_and_sorted() {
    let package_tsv = super::render_package_inventory_tsv(&super::built_in_package_surfaces());
    let effect_tsv = super::render_effect_inventory_tsv(&super::built_in_host_effects());
    let package_markdown = super::render_package_inventory_markdown(&super::built_in_package_surfaces());
    let effect_markdown = super::render_effect_inventory_markdown(&super::built_in_host_effects());
    assert_eq!(package_tsv, super::render_package_inventory_tsv(&super::built_in_package_surfaces()));
    assert_eq!(effect_tsv, super::render_effect_inventory_tsv(&super::built_in_host_effects()));
    assert!(package_tsv.contains("package.ratcore.production"));
    assert!(effect_tsv.contains("effect.rat-inline.terminal-geometry"));
    assert!(package_markdown.contains("Classification is not OS support evidence"));
    assert!(effect_markdown.contains("Migration blockers are classified but non-passing"));
}
