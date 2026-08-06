use super::super::*;

#[test]
fn tabs_declaration_and_reviewed_inventory_are_complete() {
    let declaration = tabs_contract_declaration();
    let inventory = built_in_headless_inventory();

    assert!(validate_headless_contract(&declaration).is_empty());
    assert!(validate_headless_inventory(&inventory).is_empty());
    assert_eq!(inventory.iter().filter(|row| row.component_id == "headless.tabs.v1").count(), 3);
}

#[test]
fn rejects_framework_leak_missing_ownership_hidden_diagnostics_and_recomputation() {
    let mut declaration = tabs_contract_declaration();
    declaration.ownership = None;
    declaration.shared_type_paths.push("dioxus::prelude::Element".to_string());
    declaration.adapter_recomputes_semantics = true;
    declaration.diagnostics_exposed = false;
    declaration.evidence.clear();
    declaration.facets.remove(&HeadlessContractFacet::CallerOwnedSlots);

    let codes = validate_headless_contract(&declaration)
        .into_iter()
        .map(|issue| issue.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(codes.contains(&HeadlessContractIssueCode::MissingOwnership));
    assert!(codes.contains(&HeadlessContractIssueCode::FrameworkTypeLeak));
    assert!(codes.contains(&HeadlessContractIssueCode::AdapterRecomputesSemantics));
    assert!(codes.contains(&HeadlessContractIssueCode::DiagnosticsHidden));
    assert!(codes.contains(&HeadlessContractIssueCode::MissingEvidence));
    assert!(codes.contains(&HeadlessContractIssueCode::MissingFacet));
}

#[test]
fn inventory_rejects_duplicate_unknown_and_unsupported_claims_without_evidence() {
    let mut rows = built_in_headless_inventory();
    let duplicate = rows[0].clone();
    rows.push(duplicate);
    rows.push(HeadlessComponentInventoryRow {
        component_id: "headless.unknown.v1".to_string(),
        owner: "renderer".to_string(),
        classification: "complete-ish".to_string(),
        reason: None,
        next_action: None,
        capabilities: Vec::new(),
    });
    rows.push(HeadlessComponentInventoryRow {
        component_id: "headless.unsupported.v1".to_string(),
        owner: "renderer".to_string(),
        classification: HeadlessComponentClassification::RendererOwned.as_str().to_string(),
        reason: None,
        next_action: None,
        capabilities: vec![HeadlessCapabilityEntry {
            capability: HeadlessComponentCapability::Keyboard,
            support: HeadlessCapabilitySupport::Unsupported,
            evidence: None,
            reason: None,
            next_action: None,
        }],
    });

    let codes = validate_headless_inventory(&rows)
        .into_iter()
        .map(|issue| issue.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(codes.contains(&HeadlessInventoryIssueCode::DuplicateRow));
    assert!(codes.contains(&HeadlessInventoryIssueCode::UnknownClassification));
    assert!(codes.contains(&HeadlessInventoryIssueCode::OwnershipReasonMissing));
    assert!(codes.contains(&HeadlessInventoryIssueCode::CapabilityReasonMissing));
    assert!(codes.contains(&HeadlessInventoryIssueCode::NextActionMissing));
}
