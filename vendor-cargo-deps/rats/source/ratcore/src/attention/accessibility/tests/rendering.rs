#[test]
fn renderer_summaries_agree_on_meaning_and_keep_limitations_visible() {
    let snapshot = super::super::menu_dialog_semantic_fixture();
    let meteorite =
        super::super::project_semantic_summary(&snapshot, super::super::SemanticRendererTarget::MeteoriteDom);
    let subwayrat =
        super::super::project_semantic_summary(&snapshot, super::super::SemanticRendererTarget::SubwayratTerminal);
    assert!(meteorite.issues.is_empty());
    assert!(subwayrat.issues.is_empty());
    assert_eq!(
        meteorite.nodes.iter().map(|node| &node.id).collect::<Vec<_>>(),
        subwayrat.nodes.iter().map(|node| &node.id).collect::<Vec<_>>()
    );
    assert_eq!(meteorite.nodes[0].role, subwayrat.nodes[0].role);
    assert!(meteorite.nodes.iter().flat_map(|node| &node.support).all(|row| matches!(
        (&row.feature, row.level),
        (super::super::SemanticFeature::Action(_), super::super::SemanticSupportLevel::Unverified)
            | (_, super::super::SemanticSupportLevel::Direct)
    )));
    assert!(
        meteorite
            .nodes
            .iter()
            .flat_map(|node| &node.support)
            .filter(|row| row.level != super::super::SemanticSupportLevel::Direct)
            .all(|row| row.reason.as_ref().is_some_and(|reason| !reason.is_empty()))
    );
    assert!(
        subwayrat
            .nodes
            .iter()
            .flat_map(|node| &node.support)
            .filter(|rows| rows.level != super::super::SemanticSupportLevel::Direct)
            .all(|rows| rows.reason.as_ref().is_some_and(|reason| !reason.is_empty()))
    );
}

#[test]
fn native_join_is_identity_only_and_runtime_support_stays_unverified() {
    let snapshot = super::super::tree_semantic_fixture();
    let native =
        super::super::project_semantic_summary(&snapshot, super::super::SemanticRendererTarget::NativeAccessibility);
    let join = super::super::NativeSemanticJoinKey {
        generation: snapshot.generation,
        node: snapshot.nodes[1].id.clone(),
    };
    assert_eq!(join.generation, super::super::SemanticGeneration::new(7));
    assert_eq!(join.node, "semantic.tree-source");
    assert!(
        native
            .nodes
            .iter()
            .flat_map(|node| &node.support)
            .all(|rows| rows.level == super::super::SemanticSupportLevel::Unverified)
    );
}

#[test]
fn fixture_hashes_are_deterministic_and_semantic_only() {
    let snapshot = super::super::menu_dialog_semantic_fixture();
    let first_hash = super::super::semantic_snapshot_hash(&snapshot);
    let second_hash = super::super::semantic_snapshot_hash(&snapshot);
    assert_eq!(first_hash, second_hash);
    assert_eq!(first_hash, "e169ff2fa51404a32e488b3a1f57c40c450f7d51e449800124d71d0d41aceff8");
    assert_eq!(
        super::super::semantic_snapshot_hash(&super::super::tree_semantic_fixture()),
        "8f910f79fffcf4203503c776078dbf4e3194059a329a31db27eb6cf94227f779"
    );

    let evidence = super::super::semantic_fixture_evidence("accessibility.menu-dialog", &snapshot);
    assert_eq!(evidence.kind, super::super::SemanticEvidenceKind::SemanticProjection);
    assert!(evidence.runtime_receipt.is_none());
    assert!(super::super::validate_semantic_evidence(&evidence, &snapshot).is_empty());
}

#[test]
fn stale_mislabeled_and_unreasoned_evidence_is_rejected() {
    let snapshot = super::super::tree_semantic_fixture();
    let mut evidence = super::super::semantic_fixture_evidence("accessibility.tree", &snapshot);
    evidence.generation = super::super::SemanticGeneration::new(6);
    evidence.kind = super::super::SemanticEvidenceKind::NativeRuntime;
    let rows = evidence
        .support
        .iter_mut()
        .find(|row| row.level != super::super::SemanticSupportLevel::Direct)
        .unwrap();
    rows.reason = None;

    let issues = super::super::validate_semantic_evidence(&evidence, &snapshot);
    let codes = super::issue_codes(&issues);
    assert!(codes.contains(&super::super::SemanticIssueCode::StaleEvidence));
    assert!(codes.contains(&super::super::SemanticIssueCode::MisclassifiedEvidence));
    assert!(codes.contains(&super::super::SemanticIssueCode::MissingSupportReason));
}

#[test]
fn malformed_snapshot_emits_diagnostics_instead_of_partial_projection() {
    let mut snapshot = super::super::tree_semantic_fixture();
    snapshot.nodes[1].collection = Some(super::super::SemanticCollection {
        level: 0,
        position: 3,
        set_size: 2,
    });
    let projection =
        super::super::project_semantic_summary(&snapshot, super::super::SemanticRendererTarget::MeteoriteDom);
    assert!(projection.nodes.is_empty());
    assert!(
        projection
            .issues
            .iter()
            .any(|issue| issue.code == super::super::SemanticIssueCode::InvalidCollection)
    );
}
