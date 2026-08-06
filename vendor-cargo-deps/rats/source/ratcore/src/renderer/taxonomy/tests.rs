fn issue_codes(outcome: &super::RendererPolicyReport) -> std::collections::BTreeSet<super::RendererPolicyIssueCode> {
    outcome.issues.iter().map(|issue| issue.code).collect()
}

#[test]
fn built_in_registry_and_named_scopes_are_valid_and_ordered() {
    let registry = super::built_in_renderer_registry();
    assert!(super::validate_renderer_registry(&registry).is_valid());
    for scope_name in [
        super::BuiltInRendererScope::GeneralParityFamilies,
        super::BuiltInRendererScope::FixtureParityFamilies,
        super::BuiltInRendererScope::SnapshotVisualAdapters,
        super::BuiltInRendererScope::FullMouseTargets,
        super::BuiltInRendererScope::BrowserTargets,
        super::BuiltInRendererScope::EmbeddedRatzillaTargets,
        super::BuiltInRendererScope::IcedPilotTarget,
        super::BuiltInRendererScope::IcedPilotEvidenceAdapters,
    ] {
        let scope = super::built_in_renderer_scope(scope_name);
        assert!(super::validate_renderer_scope(&registry, &scope).is_valid());
    }
    let general = super::built_in_renderer_scope(super::BuiltInRendererScope::GeneralParityFamilies);
    assert_eq!(general.members[0].as_str(), super::FAMILY_SUBWAYRAT);
    assert_eq!(general.members[1].as_str(), super::FAMILY_METEORITE);
    let embedded = super::built_in_renderer_scope(super::BuiltInRendererScope::EmbeddedRatzillaTargets);
    assert_eq!(embedded.members.len(), 4);
    assert!(!embedded.members.iter().any(|member| member.as_str() == "iced-native"));
}

#[test]
fn equal_text_target_and_adapter_keep_nominal_identity() {
    let registry = super::built_in_renderer_registry();
    let target = super::RendererSubject::Target(super::RendererTargetId::new("meteorite-dom"));
    let adapter = super::RendererSubject::EvidenceAdapter(super::EvidenceAdapterId::new("meteorite-dom"));
    assert_eq!(target.as_str(), adapter.as_str());
    assert_ne!(target.kind(), adapter.kind());
    assert!(registry.target(&super::RendererTargetId::new("meteorite-dom")).is_some());
    assert!(registry.evidence_adapter(&super::EvidenceAdapterId::new("meteorite-dom")).is_some());
}

#[test]
fn malformed_registry_reports_identity_parent_mapping_and_lifecycle_failures() {
    let registry = super::RendererRegistry::new()
        .with_family(super::RendererFamilyDescriptor::new("", "", super::RendererLifecycle::Retired))
        .with_family(super::RendererFamilyDescriptor::new("duplicate", "First", super::RendererLifecycle::Experimental))
        .with_family(super::RendererFamilyDescriptor::new("duplicate", "Second", super::RendererLifecycle::Maintained))
        .with_target(super::RendererTargetDescriptor::new(
            "orphan",
            "missing-family",
            "Orphan",
            super::RendererLifecycle::Maintained,
        ))
        .with_target(super::RendererTargetDescriptor::new(
            "orphan",
            "duplicate",
            "Duplicate target",
            super::RendererLifecycle::Maintained,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            "visual-a",
            "orphan",
            super::EvidenceRole::VisualSnapshot,
            "Visual A",
            super::RendererLifecycle::Maintained,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            "visual-b",
            "orphan",
            super::EvidenceRole::VisualSnapshot,
            "Visual B",
            super::RendererLifecycle::Maintained,
        ))
        .with_evidence_adapter(super::EvidenceAdapterDescriptor::new(
            "missing-target",
            "unknown-target",
            super::EvidenceRole::RouteSummary,
            "Missing target",
            super::RendererLifecycle::Maintained,
        ));
    let outcome = super::validate_renderer_registry(&registry);
    let codes = issue_codes(&outcome);
    assert!(codes.contains(&super::RendererPolicyIssueCode::EmptyIdentity));
    assert!(codes.contains(&super::RendererPolicyIssueCode::RetiredWithoutReason));
    assert!(codes.contains(&super::RendererPolicyIssueCode::DuplicateFamily));
    assert!(codes.contains(&super::RendererPolicyIssueCode::DuplicateTarget));
    assert!(codes.contains(&super::RendererPolicyIssueCode::UnknownFamily));
    assert!(codes.contains(&super::RendererPolicyIssueCode::UnknownTarget));
    assert!(codes.contains(&super::RendererPolicyIssueCode::DuplicateEvidenceMapping));
    assert!(codes.contains(&super::RendererPolicyIssueCode::LifecycleMismatch));
    assert!(outcome.issues.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn malformed_scope_rejects_empty_duplicate_unknown_wrong_kind_and_retired_members() {
    let registry = super::built_in_renderer_registry().with_family(
        super::RendererFamilyDescriptor::new("retired", "Retired", super::RendererLifecycle::Retired)
            .with_reason("superseded"),
    );
    let scope = super::RendererRequirementScope::new(
        "scope.invalid",
        super::RendererSubjectKind::Family,
        super::RendererScopeCompletion::Maintained,
    )
    .with_member(super::RendererSubject::Family(super::RendererFamilyId::new("retired")))
    .with_member(super::RendererSubject::Family(super::RendererFamilyId::new("missing")))
    .with_member(super::RendererSubject::Target(super::RendererTargetId::new(super::TARGET_METEORITE_DOM)))
    .with_member(super::RendererSubject::Family(super::RendererFamilyId::new("missing")));
    let outcome = super::validate_renderer_scope(&registry, &scope);
    let codes = issue_codes(&outcome);
    assert!(codes.contains(&super::RendererPolicyIssueCode::RetiredScopeMember));
    assert!(codes.contains(&super::RendererPolicyIssueCode::UnknownScopeMember));
    assert!(codes.contains(&super::RendererPolicyIssueCode::WrongScopeMemberKind));
    assert!(codes.contains(&super::RendererPolicyIssueCode::DuplicateScopeMember));

    let empty = super::RendererRequirementScope::new(
        "",
        super::RendererSubjectKind::Target,
        super::RendererScopeCompletion::Maintained,
    );
    assert!(
        issue_codes(&super::validate_renderer_scope(&registry, &empty))
            .contains(&super::RendererPolicyIssueCode::EmptyScope)
    );
}

#[test]
fn scoped_cells_reject_missing_duplicate_out_of_scope_and_incomplete_evidence() {
    let registry = super::built_in_renderer_registry();
    let scope = super::built_in_renderer_scope(super::BuiltInRendererScope::GeneralParityFamilies);
    let subwayrat = super::RendererSubject::Family(super::RendererFamilyId::new(super::FAMILY_SUBWAYRAT));
    let rows = vec![
        super::ScopedRendererCell::new(subwayrat.clone(), super::RendererCellStatus::Complete),
        super::ScopedRendererCell::new(subwayrat, super::RendererCellStatus::Unsupported),
        super::ScopedRendererCell::new(
            super::RendererSubject::Family(super::RendererFamilyId::new("iced")),
            super::RendererCellStatus::Missing,
        ),
    ];
    let outcome = super::validate_scoped_renderer_cells(&registry, &scope, &rows);
    let codes = issue_codes(&outcome);
    assert!(codes.contains(&super::RendererPolicyIssueCode::DuplicateCell));
    assert!(codes.contains(&super::RendererPolicyIssueCode::MissingRequiredCell));
    assert!(codes.contains(&super::RendererPolicyIssueCode::OutOfScopeCell));
    assert!(codes.contains(&super::RendererPolicyIssueCode::MissingEvidence));
    assert!(codes.contains(&super::RendererPolicyIssueCode::MissingUnsupportedReason));
    assert!(codes.contains(&super::RendererPolicyIssueCode::MissingNextAction));
    assert!(codes.contains(&super::RendererPolicyIssueCode::CellOrderMismatch));
}

#[test]
fn experimental_target_isolated_to_explicit_pilot_scope() {
    let registry = super::built_in_renderer_registry();
    let member = super::RendererSubject::Target(super::RendererTargetId::new(super::TARGET_ICED_NATIVE));
    let pilot = super::built_in_renderer_scope(super::BuiltInRendererScope::IcedPilotTarget);
    let maintained = super::RendererRequirementScope::new(
        "global.invalid",
        super::RendererSubjectKind::Target,
        super::RendererScopeCompletion::Maintained,
    )
    .with_member(member);
    assert!(super::validate_renderer_scope(&registry, &pilot).is_valid());
    assert!(
        issue_codes(&super::validate_renderer_scope(&registry, &maintained))
            .contains(&super::RendererPolicyIssueCode::ExperimentalInMaintainedScope)
    );
    let embedded = super::built_in_renderer_scope(super::BuiltInRendererScope::EmbeddedRatzillaTargets);
    assert_eq!(embedded.members.len(), 4);
    let evidence = super::built_in_renderer_scope(super::BuiltInRendererScope::IcedPilotEvidenceAdapters);
    assert_eq!(evidence.members.len(), 4);
    assert!(super::validate_renderer_scope(&registry, &evidence).is_valid());
    assert!(
        registry
            .evidence_adapter(&super::EvidenceAdapterId::new(super::ADAPTER_ICED_NATIVE_SMOKE,))
            .is_some()
    );
}
