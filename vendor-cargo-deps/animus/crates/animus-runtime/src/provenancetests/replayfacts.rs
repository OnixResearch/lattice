#[test]
fn replay_rebuilds_model_visible_context_and_counts() {
    // r[verify agent_harness_runtime.provenance_ledger.session_ledger.replay]
    let replay = replay_provenance_ledger(ledger_entries().as_slice())
        .expect("supported entries should replay");

    assert_eq!(replay.model_visible.len(), EXPECTED_REPLAY_MESSAGES);
    assert_eq!(replay.model_visible[0].role, ProvenanceReplayRole::User);
    assert_eq!(replay.model_visible[0].content, USER_CONTENT);
    assert_eq!(replay.metadata.user_messages, EXPECTED_USER_MESSAGES);
    assert_eq!(
        replay.metadata.assistant_messages,
        EXPECTED_ASSISTANT_MESSAGES
    );
    assert_eq!(replay.metadata.tool_messages, EXPECTED_TOOL_MESSAGES);
    assert_eq!(replay.metadata.summaries, EXPECTED_SUMMARIES);
    assert_eq!(
        replay.metadata.usage_observations,
        EXPECTED_USAGE_OBSERVATIONS
    );
    assert_eq!(replay.metadata.receipts, EXPECTED_RECEIPTS);
}

#[test]
fn replay_unsupported_entry_fails_without_raw_payload() {
    // r[verify agent_harness_runtime.provenance_ledger.session_ledger.unsupported]
    let mut entries = ledger_entries();
    entries.push(ProvenanceLedgerEntry::Unsupported(
        ProvenanceUnsupportedEntry {
            id: UNSUPPORTED_ID.to_owned(),
            parents: vec![RECEIPT_ID.to_owned()],
            adapter_label: "future-adapter".to_owned(),
            payload_hash: hash("omitted raw payload"),
        },
    ));
    let error = replay_provenance_ledger(entries.as_slice())
        .expect_err("unsupported entry should fail closed");

    assert_eq!(error.kind(), ProvenanceErrorKind::UnsupportedEntry);
    assert_eq!(error.subject(), UNSUPPORTED_ID);
    assert!(!error.reason().contains("omitted raw payload"));
}

#[test]
fn safe_fact_queries_match_without_raw_payloads() {
    // r[verify agent_harness_runtime.provenance_ledger.typed_facts.query]
    let artifact = public_artifact("fact payload");
    let facts = vec![
        ProvenanceFact::ModelActivity(ProvenanceFactModelActivity {
            fact_id: "fact-model".to_owned(),
            artifact_hash: artifact.artifact_hash.clone(),
            requirement_id: Some(REQUIREMENT_ID.to_owned()),
            request_shape: REQUEST_SHAPE.to_owned(),
            actor_label: ACTOR_LABEL.to_owned(),
        }),
        ProvenanceFact::ToolActivity(ProvenanceFactToolActivity {
            fact_id: "fact-tool".to_owned(),
            tool_kind: TOOL_KIND.to_owned(),
            artifact_hash: artifact.artifact_hash.clone(),
            effect_ability: EFFECT_ABILITY.to_owned(),
            effect_resource: EFFECT_RESOURCE.to_owned(),
        }),
        ProvenanceFact::Error(ProvenanceFactError {
            fact_id: "fact-error".to_owned(),
            error_class: ERROR_CLASS.to_owned(),
            requirement_id: Some(REQUIREMENT_ID.to_owned()),
        }),
        ProvenanceFact::Authorization(ProvenanceFactAuthorization {
            fact_id: "fact-auth".to_owned(),
            status: ProvenanceAuthorizationStatus::Denied,
            effect_ability: EFFECT_ABILITY.to_owned(),
            effect_resource: EFFECT_RESOURCE.to_owned(),
        }),
    ];

    assert_eq!(
        query_provenance_facts(
            facts.as_slice(),
            &ProvenanceFactQuery {
                artifact_hash: Some(artifact.artifact_hash.clone()),
                request_shape: Some(REQUEST_SHAPE.to_owned()),
                ..ProvenanceFactQuery::default()
            },
        )
        .len(),
        EXPECTED_USER_MESSAGES as usize,
    );
    assert_eq!(
        query_provenance_facts(
            facts.as_slice(),
            &ProvenanceFactQuery {
                tool_kind: Some(TOOL_KIND.to_owned()),
                effect_ability: Some(EFFECT_ABILITY.to_owned()),
                effect_resource: Some(EFFECT_RESOURCE.to_owned()),
                ..ProvenanceFactQuery::default()
            },
        )
        .len(),
        EXPECTED_USER_MESSAGES as usize,
    );
    assert_eq!(
        query_provenance_facts(
            facts.as_slice(),
            &ProvenanceFactQuery {
                error_class: Some(ERROR_CLASS.to_owned()),
                requirement_id: Some(REQUIREMENT_ID.to_owned()),
                ..ProvenanceFactQuery::default()
            },
        )
        .len(),
        EXPECTED_USER_MESSAGES as usize,
    );
    assert_eq!(
        query_provenance_facts(
            facts.as_slice(),
            &ProvenanceFactQuery {
                authorization_status: Some(ProvenanceAuthorizationStatus::Denied),
                effect_resource: Some(EFFECT_RESOURCE.to_owned()),
                ..ProvenanceFactQuery::default()
            },
        )
        .len(),
        EXPECTED_USER_MESSAGES as usize,
    );
}

#[test]
fn opaque_facts_keep_sanitized_metadata_or_fail_closed() {
    // r[verify agent_harness_runtime.provenance_ledger.typed_facts.opaque]
    let safe = ProvenanceOpaqueFact::new(
        "opaque-fact",
        "future-schema",
        vec![ProvenanceSafeField {
            key: "payload_hash".to_owned(),
            value: hash("raw omitted"),
        }],
    )
    .expect("safe opaque fact should validate");
    let error = ProvenanceOpaqueFact::new(
        "opaque-fact",
        "future-schema",
        vec![ProvenanceSafeField {
            key: "raw".to_owned(),
            value: RAW_PROMPT_LEAK.to_owned(),
        }],
    )
    .expect_err("raw opaque metadata should fail");

    assert_eq!(safe.metadata[0].key, "payload_hash");
    assert_eq!(error.kind(), ProvenanceErrorKind::UnsafeMetadata);
}

#[test]
fn artifact_hashes_sort_dependencies_and_round_trip() {
    // r[verify agent_harness_runtime.provenance_ledger.artifacts.deterministic]
    let first_dependency = hash("first dependency");
    let second_dependency = hash("second dependency");
    let draft_a = ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Runtime,
        redaction: ProvenanceRedactionClass::Public,
        normalized_payload: "payload",
        dependencies: &[first_dependency.as_str(), second_dependency.as_str()],
    };
    let draft_b = ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Runtime,
        redaction: ProvenanceRedactionClass::Public,
        normalized_payload: "payload",
        dependencies: &[second_dependency.as_str(), first_dependency.as_str()],
    };
    let artifact_a = ProvenanceArtifact::from_draft(&draft_a).expect("artifact a");
    let artifact_b = ProvenanceArtifact::from_draft(&draft_b).expect("artifact b");

    assert_eq!(artifact_a.artifact_hash, artifact_b.artifact_hash);
    assert_eq!(
        ProvenanceArtifact::parse_hash(artifact_a.artifact_hash.as_str())
            .expect("hash should parse"),
        artifact_a.artifact_hash,
    );
}

#[test]
fn redaction_classes_omit_non_public_payloads() {
    // r[verify agent_harness_runtime.provenance_ledger.artifacts.redaction]
    let public = public_artifact("public review body");
    let metadata_only = ProvenanceArtifact::from_draft(&ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Model,
        redaction: ProvenanceRedactionClass::MetadataOnly,
        normalized_payload: RAW_PROMPT_LEAK,
        dependencies: &[],
    })
    .expect("metadata-only artifact");
    let redacted = redacted_artifact(TOOL_OUTPUT_LEAK);
    let secret = ProvenanceArtifact::from_draft(&ProvenanceArtifactDraft {
        kind: ProvenanceArtifactKind::Tool,
        redaction: ProvenanceRedactionClass::Secret,
        normalized_payload: CREDENTIAL_LEAK,
        dependencies: &[],
    })
    .expect("secret artifact");

    assert_eq!(public.review_payload(), Some("public review body"));
    assert_eq!(metadata_only.review_payload(), None);
    assert_eq!(redacted.review_payload(), None);
    assert_eq!(secret.review_payload(), None);
    assert_eq!(secret.redaction_marker(), SECRET_REDACTION_MARKER);
}
