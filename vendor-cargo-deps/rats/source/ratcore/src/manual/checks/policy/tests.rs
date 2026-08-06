const FIXTURE: &str = "dialog.policy";
const PATH: &str = "snapshots/policy.html";
const BYTES: &[u8] = b"policy snapshot";

fn fixtures() -> std::collections::BTreeSet<crate::showcase_fixtures::FixtureId> {
    std::collections::BTreeSet::from([crate::showcase_fixtures::FixtureId::new(FIXTURE)])
}

#[test]
fn unknown_evidence_adapter_is_rejected() {
    let entry = super::super::super::SnapshotManifestEntry::for_evidence_adapter(
        FIXTURE,
        crate::targeting_core::EvidenceAdapterId::new("unknown-adapter"),
        PATH,
        BYTES,
        super::super::super::SnapshotSize::new(1, 1),
    );
    let artifact = super::super::super::SnapshotArtifact::new(PATH, BYTES);
    let outcome = super::super::validate_snapshot_manifest(&[entry], &fixtures(), &[artifact]);
    assert!(!outcome.is_valid());
    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| { issue.code == super::super::SnapshotIssueCode::UnknownEvidenceAdapter })
    );
}

#[test]
fn wrong_target_adapter_cannot_satisfy_family_evidence() {
    let mut registry = crate::targeting_core::built_in_renderer_registry();
    registry
        .evidence_adapters
        .retain(|adapter| adapter.id == crate::targeting_core::EvidenceAdapterId::new("meteorite-dom"));
    registry.evidence_adapters[0].target =
        crate::targeting_core::RendererTargetId::new(crate::targeting_core::TARGET_SUBWAYRAT_TERMINAL);
    let meteorite = crate::targeting_core::RendererFamilyId::new(crate::targeting_core::FAMILY_METEORITE);
    let subwayrat = crate::targeting_core::RendererFamilyId::new(crate::targeting_core::FAMILY_SUBWAYRAT);
    assert!(super::visual_adapter_for_family(&registry, &meteorite).is_none());
    let mapped = super::visual_adapter_for_family(&registry, &subwayrat);
    assert!(mapped.is_some());
    assert_eq!(mapped.map(|adapter| adapter.id.as_str()), Some("meteorite-dom"));
}

#[test]
fn semantic_projection_adapter_cannot_satisfy_visual_snapshot_scope() {
    let registry = crate::targeting_core::built_in_renderer_registry().with_evidence_adapter(
        crate::targeting_core::EvidenceAdapterDescriptor::new(
            "meteorite-semantic",
            crate::targeting_core::TARGET_METEORITE_DOM,
            crate::targeting_core::EvidenceRole::SemanticProjection,
            "Meteorite semantic summary",
            crate::targeting_core::RendererLifecycle::Maintained,
        ),
    );
    let scope = crate::targeting_core::RendererRequirementScope::new(
        "snapshot.semantic.invalid",
        crate::targeting_core::RendererSubjectKind::EvidenceAdapter,
        crate::targeting_core::RendererScopeCompletion::Maintained,
    )
    .with_member(crate::targeting_core::RendererSubject::EvidenceAdapter(
        crate::targeting_core::EvidenceAdapterId::new("meteorite-semantic"),
    ));
    let entry = super::super::super::SnapshotManifestEntry::for_evidence_adapter(
        FIXTURE,
        crate::targeting_core::EvidenceAdapterId::new("meteorite-semantic"),
        PATH,
        BYTES,
        super::super::super::SnapshotSize::new(1, 1),
    );
    let artifact = super::super::super::SnapshotArtifact::new(PATH, BYTES);
    let outcome =
        super::super::validate_snapshot_manifest_with_policy(&[entry], &fixtures(), &[artifact], &registry, &scope);
    assert!(!outcome.is_valid());
    assert!(outcome.issues.iter().any(|issue| issue.code == super::super::SnapshotIssueCode::WrongEvidenceRole));
}
