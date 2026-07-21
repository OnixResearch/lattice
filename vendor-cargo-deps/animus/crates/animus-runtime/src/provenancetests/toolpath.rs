#[test]
fn toolpath_export_preserves_safe_dag_projection() {
    // r[verify agent_harness_runtime.provenance_ledger.toolpath_adapter.export]
    let projection = export_toolpath_projection(
        ledger_entries().as_slice(),
        RECEIPT_ID,
        ToolpathExportPolicy::RedactedDefault,
    )
    .expect("toolpath projection should validate");

    assert_eq!(projection.path_head, RECEIPT_ID);
    assert!(projection.steps.iter().any(|step| step.id == USER_ID));
    assert!(projection.steps.iter().any(|step| {
        step.parents.contains(&SUMMARY_ID.to_owned())
            && step
                .artifact_hashes
                .iter()
                .all(|value| value.len() == HASH_HEX_LENGTH)
    }));
    assert!(projection.dead_end_steps.is_empty());
}

#[test]
fn toolpath_default_export_redacts_raw_sensitive_material() {
    // r[verify agent_harness_runtime.provenance_ledger.toolpath_adapter.redacted_default]
    let entries = vec![ProvenanceLedgerEntry::Message(ProvenanceMessageEntry {
        id: USER_ID.to_owned(),
        parents: Collection::new(),
        role: ProvenanceReplayRole::User,
        actor: actor(),
        content: RAW_PROMPT_LEAK.to_owned(),
        content_hash: hash(RAW_PROMPT_LEAK),
    })];
    let projection = export_toolpath_projection(
        entries.as_slice(),
        USER_ID,
        ToolpathExportPolicy::RedactedDefault,
    )
    .expect("redacted projection should succeed");
    let step = &projection.steps[0];

    assert!(!format!("{step:?}").contains(RAW_PROMPT_LEAK));
    assert!(
        step.redaction_markers
            .contains(&TOOLPATH_REDACTED_PROMPT_MARKER.to_owned())
    );
}

#[test]
fn toolpath_projection_uses_owned_contracts_without_external_dependencies() {
    // r[verify agent_harness_runtime.provenance_ledger.toolpath_adapter.docs]
    // r[verify agent_harness_runtime.provenance_ledger.toolpath_adapter.isolated]
    let projection = ToolpathProjection {
        schema: TOOLPATH_PROJECTION_SCHEMA.to_owned(),
        path_head: USER_ID.to_owned(),
        steps: vec![ToolpathStep {
            id: USER_ID.to_owned(),
            parents: Collection::new(),
            actor: actor().toolpath_label(),
            change_kind: ProvenanceReplayRole::User.as_str().to_owned(),
            artifact_hashes: vec![hash(USER_CONTENT)],
            refs: vec![REQUIREMENT_ID.to_owned()],
            redaction_markers: vec![REDACTED_PAYLOAD_MARKER.to_owned()],
        }],
        dead_end_steps: Collection::new(),
    };

    assert_eq!(projection.schema, TOOLPATH_PROJECTION_SCHEMA);
    assert_eq!(projection.steps[0].actor, "human:operator");
}
