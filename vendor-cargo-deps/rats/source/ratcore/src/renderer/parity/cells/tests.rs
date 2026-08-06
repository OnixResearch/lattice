fn complete_row() -> super::super::Row {
    super::super::Row::new("renderer.scope.pilot", "renderer", "ratcore", "keep pilot isolated")
        .with_renderer(super::super::Support::new("subwayrat", "complete").with_evidence("terminal evidence"))
        .with_renderer(super::super::Support::new("meteorite", "complete").with_evidence("DOM evidence"))
}

#[test]
fn experimental_family_is_required_only_by_explicit_pilot_scope() {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let pilot = crate::targeting_core::RendererRequirementScope::new(
        "renderer.pilot",
        crate::targeting_core::RendererSubjectKind::Family,
        crate::targeting_core::RendererScopeCompletion::Experimental,
    )
    .with_member(crate::targeting_core::RendererSubject::Family(crate::targeting_core::RendererFamilyId::new(
        "subwayrat",
    )))
    .with_member(crate::targeting_core::RendererSubject::Family(crate::targeting_core::RendererFamilyId::new(
        "meteorite",
    )))
    .with_member(crate::targeting_core::RendererSubject::Family(crate::targeting_core::RendererFamilyId::new("iced")));
    let row = complete_row().with_renderer_cell(
        super::super::FamilySupport::new("iced", "unsupported").with_reason("native pilot has no runtime evidence"),
    );
    let pilot_outcome = super::super::validate_with_renderer_policy(&[row.clone()], &registry, &pilot);
    assert!(pilot_outcome.is_valid(), "issues: {:?}", pilot_outcome.issues);

    let default_scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::GeneralParityFamilies,
    );
    let maintained = super::super::validate_with_renderer_policy(&[row], &registry, &default_scope);
    assert!(!maintained.is_valid());
    assert!(maintained.issues.iter().any(|issue| issue.code == super::super::IssueCode::OutOfScopeRenderer));
}

#[test]
fn experimental_target_does_not_expand_embedded_ratzilla_columns() {
    let row = embedded_row();
    assert!(super::super::validate_embedded_ratzilla_surface_rows(&[row.clone()]).is_valid());
    let extra = row
        .with_renderer(super::super::Support::new("iced-native", "unsupported").with_reason("unrelated native target"));
    let outcome = super::super::validate_embedded_ratzilla_surface_rows(&[extra]);
    assert!(!outcome.is_valid());
    assert!(outcome.issues.iter().any(|issue| issue.code == super::super::IssueCode::OutOfScopeRenderer));
}

#[test]
fn renderer_rows_reject_missing_duplicate_and_unknown_families() {
    let registry = crate::targeting_core::built_in_renderer_registry();
    let scope = crate::targeting_core::built_in_renderer_scope(
        crate::targeting_core::BuiltInRendererScope::GeneralParityFamilies,
    );
    let row = super::super::Row::new("renderer.scope.invalid", "renderer", "ratcore", "repair")
        .with_renderer(super::super::Support::new("subwayrat", "complete").with_evidence("first"))
        .with_renderer(super::super::Support::new("subwayrat", "complete").with_evidence("duplicate"))
        .with_renderer_cell(super::super::FamilySupport::new("unknown", "unsupported").with_reason("unknown"));
    let outcome = super::super::validate_with_renderer_policy(&[row], &registry, &scope);
    assert!(!outcome.is_valid());
    assert!(outcome.issues.iter().any(|issue| issue.code == super::super::IssueCode::DuplicateRenderer));
    assert!(outcome.issues.iter().any(|issue| issue.code == super::super::IssueCode::MissingRendererLabel));
    assert!(outcome.issues.iter().any(|issue| issue.code == super::super::IssueCode::UnknownRenderer));
}

fn embedded_row() -> super::super::EmbeddedRatzillaSurfaceRow {
    super::super::EmbeddedRatzillaSurfaceRow::new(
        "embedded.scope.fixed",
        "embedding",
        "ratcore; rat-ratzilla",
        "keep native DOM evidence separate",
    )
    .with_renderer(super::super::Support::new("subwayrat-terminal", "complete").with_evidence("terminal evidence"))
    .with_renderer(super::super::Support::new("subwayrat-ratzilla", "complete").with_evidence("Ratzilla evidence"))
    .with_renderer(
        super::super::Support::new("meteorite-embedded-ratzilla", "complete").with_evidence("embedded host evidence"),
    )
    .with_renderer(
        super::super::Support::new("meteorite-native-dom", "partial").with_evidence("native DOM gap evidence"),
    )
    .with_diagnostics_coverage("missing host and invalid claim diagnostics")
}
