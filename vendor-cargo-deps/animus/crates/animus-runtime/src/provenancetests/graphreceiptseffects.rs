#[test]
fn branch_queries_detect_dead_ends_and_active_ancestry() {
    // r[verify agent_harness_runtime.provenance_ledger.branch_queries.dead_ends]
    let entries = vec![
        ProvenanceGraphEntry {
            id: "root".to_owned(),
            parents: Collection::new(),
        },
        ProvenanceGraphEntry {
            id: "active".to_owned(),
            parents: vec!["root".to_owned()],
        },
        ProvenanceGraphEntry {
            id: "abandoned".to_owned(),
            parents: vec!["root".to_owned()],
        },
    ];
    let report = validate_provenance_graph(entries.as_slice(), "active")
        .expect("branch graph should validate");

    assert!(report.active_ancestry.contains(&"active".to_owned()));
    assert!(report.active_ancestry.contains(&"root".to_owned()));
    assert_eq!(report.dead_end_entry_ids, vec!["abandoned".to_owned()]);
}

#[test]
fn malformed_graphs_fail_without_panic() {
    // r[verify agent_harness_runtime.provenance_ledger.branch_queries.invalid]
    let missing_parent = vec![ProvenanceGraphEntry {
        id: "child".to_owned(),
        parents: vec!["missing".to_owned()],
    }];
    let duplicate = vec![
        ProvenanceGraphEntry {
            id: "same".to_owned(),
            parents: Collection::new(),
        },
        ProvenanceGraphEntry {
            id: "same".to_owned(),
            parents: Collection::new(),
        },
    ];
    let cycle = vec![
        ProvenanceGraphEntry {
            id: "a".to_owned(),
            parents: vec!["b".to_owned()],
        },
        ProvenanceGraphEntry {
            id: "b".to_owned(),
            parents: vec!["a".to_owned()],
        },
    ];
    let reversed_order = vec![
        ProvenanceGraphEntry {
            id: "child".to_owned(),
            parents: vec!["root".to_owned()],
        },
        ProvenanceGraphEntry {
            id: "root".to_owned(),
            parents: Collection::new(),
        },
    ];

    assert_eq!(
        validate_provenance_graph(missing_parent.as_slice(), "child")
            .expect_err("missing parent should fail")
            .kind(),
        ProvenanceErrorKind::MissingParent,
    );
    assert_eq!(
        validate_provenance_graph(duplicate.as_slice(), "same")
            .expect_err("duplicate should fail")
            .kind(),
        ProvenanceErrorKind::DuplicateEntry,
    );
    assert_eq!(
        validate_provenance_graph(cycle.as_slice(), "a")
            .expect_err("cycle should fail")
            .kind(),
        ProvenanceErrorKind::Cycle,
    );
    assert!(validate_provenance_graph(reversed_order.as_slice(), "child").is_ok());
}

#[test]
fn safe_receipts_link_hashes_without_sensitive_content() {
    // r[verify agent_harness_runtime.provenance_ledger.receipts.safe]
    let artifact = redacted_artifact(RAW_PROMPT_LEAK);
    let safe_receipt = receipt(artifact.artifact_hash.as_str());

    assert_eq!(safe_receipt.status, RECEIPT_STATUS);
    assert_eq!(safe_receipt.artifact_hashes, vec![artifact.artifact_hash]);
    assert!(!safe_receipt.receipt_hash.contains(RAW_PROMPT_LEAK));
    assert!(
        safe_receipt
            .redaction_markers
            .contains(&REDACTED_PAYLOAD_MARKER.to_owned())
    );
}

#[test]
fn receipts_reject_leak_fixtures() {
    // r[verify agent_harness_runtime.provenance_ledger.receipts.rejects_leaks]
    let artifact_hash = hash("receipt artifact");
    let leak_fixtures = [
        RAW_PROMPT_LEAK,
        PROVIDER_JSON_LEAK,
        CREDENTIAL_LEAK,
        FULL_PATH_LEAK,
        TOOL_INPUT_LEAK,
        TOOL_OUTPUT_LEAK,
        MODEL_OUTPUT_LEAK,
        UNBOUNDED_OUTPUT_LEAK,
        RAW_DIFF_LEAK,
    ];

    for leak_fixture in leak_fixtures {
        let error = ProvenanceReceipt::validate(&ProvenanceReceiptDraft {
            actor_label: leak_fixture,
            id: RECEIPT_ID,
            parents: &[],
            status: RECEIPT_STATUS,
            issue_code: None,
            session_label: SESSION_LABEL,
            trace_label: TRACE_LABEL,
            event_count: RECEIPT_EVENT_COUNT,
            artifact_hashes: &[artifact_hash.as_str()],
            mutation_kind: None,
            effect_kind: None,
            backend_label: None,
            interchange_label: None,
            redaction_markers: &[REDACTED_PAYLOAD_MARKER],
        })
        .expect_err("leak fixture should fail");
        assert_eq!(error.kind(), ProvenanceErrorKind::UnsafeMetadata);
    }

    let count_error = ProvenanceReceipt::validate(&ProvenanceReceiptDraft {
        id: RECEIPT_ID,
        parents: &[],
        status: RECEIPT_STATUS,
        issue_code: None,
        actor_label: ACTOR_LABEL,
        session_label: SESSION_LABEL,
        trace_label: TRACE_LABEL,
        event_count: OVERSIZED_COUNT,
        artifact_hashes: &[artifact_hash.as_str()],
        mutation_kind: None,
        effect_kind: None,
        backend_label: None,
        interchange_label: None,
        redaction_markers: &[REDACTED_PAYLOAD_MARKER],
    })
    .expect_err("unbounded count should fail");
    assert_eq!(count_error.kind(), ProvenanceErrorKind::FieldTooLong);
}

#[test]
fn missing_or_denied_services_do_not_claim_completed_effects() {
    // r[verify agent_harness_runtime.provenance_ledger.tool_effects.missing_service]
    let artifact_hash = hash("tool artifact");
    let unavailable = ProvenanceToolEffect::validate(&ProvenanceToolEffectDraft {
        tool_kind: TOOL_KIND,
        service_label: SERVICE_LABEL,
        effect_ability: EFFECT_ABILITY,
        effect_resource: EFFECT_RESOURCE,
        status: ProvenanceToolEffectStatus::ServiceUnavailable,
        original_output_bytes: None,
        retained_output_bytes: None,
    })
    .expect("effect should validate");
    let denied = ProvenanceToolEffect::validate(&ProvenanceToolEffectDraft {
        tool_kind: TOOL_KIND,
        service_label: SERVICE_LABEL,
        effect_ability: EFFECT_ABILITY,
        effect_resource: EFFECT_RESOURCE,
        status: ProvenanceToolEffectStatus::CapabilityDenied,
        original_output_bytes: None,
        retained_output_bytes: None,
    })
    .expect("effect should validate");

    assert_eq!(
        unavailable
            .completed_evidence("receipt-unavailable", artifact_hash.as_str())
            .expect_err("unavailable service should not claim completion")
            .kind(),
        ProvenanceErrorKind::IncompleteEffect,
    );
    assert_eq!(
        denied
            .completed_evidence("receipt-denied", artifact_hash.as_str())
            .expect_err("denied service should not claim completion")
            .kind(),
        ProvenanceErrorKind::IncompleteEffect,
    );
}

#[test]
fn truncation_records_counts_without_omitted_output() {
    // r[verify agent_harness_runtime.provenance_ledger.tool_effects.truncation]
    let artifact_hash = hash(TOOL_OUTPUT_LEAK);
    let truncated = ProvenanceToolEffect::validate(&ProvenanceToolEffectDraft {
        tool_kind: TOOL_KIND,
        service_label: SERVICE_LABEL,
        effect_ability: EFFECT_ABILITY,
        effect_resource: EFFECT_RESOURCE,
        status: ProvenanceToolEffectStatus::Truncated,
        original_output_bytes: Some(ORIGINAL_OUTPUT_BYTES),
        retained_output_bytes: Some(RETAINED_OUTPUT_BYTES),
    })
    .expect("truncated effect should validate");
    let (_fact, receipt) = truncated
        .completed_evidence("receipt-truncated", artifact_hash.as_str())
        .expect("truncated completion should produce safe evidence");

    assert_eq!(receipt.event_count, ORIGINAL_OUTPUT_BYTES);
    assert!(
        receipt
            .redaction_markers
            .contains(&TRUNCATED_STATUS.to_owned())
    );
    assert!(!receipt.receipt_hash.contains(TOOL_OUTPUT_LEAK));
}
