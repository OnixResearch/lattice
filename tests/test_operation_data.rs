use std::collections::BTreeMap;
use std::collections::BTreeSet;

use lattice::operation_data::*;

fn identity(label: &str) -> String {
    lattice::operation::identity::hash_bytes("test.operation-data.v1", label.as_bytes())
}

fn non_claims() -> Vec<String> {
    REQUIRED_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect()
}

fn producer(invocation: &str, run_id: u64) -> Producer {
    Producer {
        operation_type: "cairn.repository".to_string(),
        type_revision: identity("type"),
        instance: "local-repository".to_string(),
        method: "validate".to_string(),
        invocation: identity(invocation),
        run_id,
        node: "operation-0".to_string(),
        workflow_revision: identity("workflow"),
    }
}

fn resource_draft(invocation: &str, sequence: u64) -> RecordDraft {
    RecordDraft {
        logical_key: "validation-report".to_string(),
        sequence,
        producer: producer(invocation, sequence),
        output_name: "report".to_string(),
        output_contract_identity: identity("output-contract"),
        tags: BTreeMap::from([("environment".to_string(), "local".to_string())]),
        sensitivity: SensitivityClass::Internal,
        lifetime: Lifetime::Retained,
        retention: RetentionPolicy {
            identity: identity("retention"),
            maximum_versions: Some(2),
        },
        outcome: "completed".to_string(),
        payload: PayloadDraft::Resource {
            schema_identity: identity("schema"),
            selectable_paths: vec![vec!["counts".to_string(), "issues".to_string()]],
            value: serde_json::json!({"valid": true, "counts": {"issues": 0}}),
        },
        required_non_claims: non_claims(),
    }
}

fn policy<'a>(fragments: &'a [String]) -> AdmissionPolicy<'a> {
    AdmissionPolicy {
        allow_sensitive: false,
        prohibited_fragments: fragments,
    }
}

fn query() -> Query {
    Query {
        schema: QUERY_SCHEMA.to_string(),
        version_identity: None,
        record_identity: None,
        record_class: None,
        operation_type: Some("cairn.repository".to_string()),
        method: Some("validate".to_string()),
        instance: None,
        output_name: Some("report".to_string()),
        schema_identity: None,
        tag: None,
        producer_run_id: None,
        workflow_revision: None,
        outcome: Some("completed".to_string()),
        field_path: None,
        order: QueryOrder::VersionDescending,
        maximum_result_count: MAX_QUERY_RESULTS,
    }
}

#[test]
fn canonical_resources_are_order_independent_and_lineage_remains_distinct() {
    let fragments = Vec::new();
    let mut first = resource_draft("invocation-a", 1);
    let mut second = resource_draft("invocation-a", 1);
    first.payload = PayloadDraft::Resource {
        schema_identity: identity("schema"),
        selectable_paths: vec![vec!["a".to_string()]],
        value: serde_json::json!({"b": 2, "a": 1}),
    };
    second.payload = PayloadDraft::Resource {
        schema_identity: identity("schema"),
        selectable_paths: vec![vec!["a".to_string()]],
        value: serde_json::from_str(r#"{"a":1,"b":2}"#).expect("valid fixture"),
    };
    let (first_record, first_bytes) = admit(&first, &policy(&fragments)).expect("first record");
    let (second_record, second_bytes) = admit(&second, &policy(&fragments)).expect("second record");
    // r[verify lattice.operation_data.records.resource]
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first_record, second_record);

    let distinct = resource_draft("invocation-b", 2);
    let (distinct_record, distinct_bytes) = admit(&distinct, &policy(&fragments)).expect("distinct record");
    let (original_record, original_bytes) =
        admit(&resource_draft("invocation-a", 1), &policy(&fragments)).expect("original record");
    assert_eq!(original_bytes, distinct_bytes);
    assert_eq!(original_record.record_identity, distinct_record.record_identity);
    // r[verify lattice.operation_data.versions.append]
    assert_ne!(original_record.version_identity, distinct_record.version_identity);
}

#[test]
fn record_admission_rejects_secret_missing_non_claim_and_oversize_data() {
    let fragments = vec!["known-secret-value".to_string()];
    let mut secret = resource_draft("secret", 1);
    secret.payload = PayloadDraft::Resource {
        schema_identity: identity("schema"),
        selectable_paths: Vec::new(),
        value: serde_json::json!({"api_token": "known-secret-value"}),
    };
    // r[verify lattice.operation_data.confidentiality.rejected]
    assert_eq!(admit(&secret, &policy(&fragments)).expect_err("secret denied"), Error::Secret);

    let mut missing = resource_draft("missing", 1);
    missing.required_non_claims.pop();
    // r[verify lattice.operation_data.records.invalid]
    assert!(matches!(admit(&missing, &policy(&[])), Err(Error::Invalid(_))));

    let mut oversized = resource_draft("oversized", 1);
    oversized.payload = PayloadDraft::File {
        media_class: "application/octet-stream".to_string(),
        bytes: vec![0_u8; MAX_RECORD_BYTES + 1],
    };
    assert_eq!(admit(&oversized, &policy(&[])).expect_err("oversize denied"), Error::Limit("file bytes"));
}

#[test]
fn closed_query_is_bounded_stable_and_schema_confined() {
    let first = admit(&resource_draft("one", 1), &policy(&[])).expect("first");
    let second = admit(&resource_draft("two", 2), &policy(&[])).expect("second");
    let first_value = serde_json::from_slice(&first.1).expect("resource JSON");
    let second_value = serde_json::from_slice(&second.1).expect("resource JSON");
    let candidates = vec![(first.0, Some(first_value)), (second.0.clone(), Some(second_value))];
    let results = reduce_query(&candidates, &query(), QueryAuthority { allow_sensitive: false }).expect("query");
    // r[verify lattice.operation_data.query.valid]
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].metadata.sequence, 2);

    let mut tag_query = query();
    tag_query.tag = Some(TagSelector {
        key: "environment".to_string(),
        value: "local".to_string(),
    });
    assert_eq!(
        reduce_query(&candidates, &tag_query, QueryAuthority { allow_sensitive: false })
            .expect("tag query")
            .len(),
        2
    );

    let mut field_query = query();
    field_query.maximum_result_count = 1;
    field_query.field_path = Some(vec!["counts".to_string(), "issues".to_string()]);
    let field = reduce_query(&candidates, &field_query, QueryAuthority { allow_sensitive: false }).expect("field");
    assert_eq!(field[0].selected_value, Some(serde_json::json!(0)));

    let mut invalid = query();
    invalid.maximum_result_count = MAX_QUERY_RESULTS + 1;
    // r[verify lattice.operation_data.query.denied]
    assert_eq!(
        reduce_query(&candidates, &invalid, QueryAuthority { allow_sensitive: false }).expect_err("limit"),
        Error::Limit("query result limit")
    );
    let oversized_candidates = vec![(second.0, None); MAX_QUERY_CANDIDATES.saturating_add(1)];
    assert_eq!(
        reduce_query(&oversized_candidates, &query(), QueryAuthority { allow_sensitive: false })
            .expect_err("candidate bound"),
        Error::Limit("query candidates")
    );
}

#[test]
fn exact_selection_rejects_empty_ambiguous_and_stale_results() {
    assert_eq!(resolve_exact(&[]).expect_err("empty"), Error::Empty);
    let (metadata, _) = admit(&resource_draft("one", 1), &policy(&[])).expect("record");
    let result = QueryResult {
        metadata: metadata.clone(),
        selected_value: None,
    };
    // r[verify lattice.operation_data.query.bound]
    assert_eq!(resolve_exact(&[result.clone(), result.clone()]).expect_err("ambiguous"), Error::Ambiguous);
    let expected = resolve_exact(&[result]).expect("exact");
    let changed = ResolvedInput {
        version_identity: identity("changed-version"),
        content_identity: expected.content_identity.clone(),
    };
    assert_eq!(recheck(&expected, &changed).expect_err("stale"), Error::Stale);
}

#[test]
fn retention_never_acts_as_implicit_orphan_pruning() {
    let mut draft = resource_draft("retained", 1);
    draft.producer.operation_type = "missing.type".to_string();
    let (retained, _) = admit(&draft, &policy(&[])).expect("retained");
    let retention = retention_plan(std::slice::from_ref(&retained), &RetentionFacts {
        now_epoch_seconds: 10,
        terminal_run_ids: Vec::new(),
    })
    .expect("retention plan");
    // r[verify lattice.operation_data.retention.orphan_preview]
    assert!(retention.selected_versions.is_empty());

    let orphan = orphan_prune_plan(&[retained], &BTreeSet::new(), &BTreeSet::new()).expect("orphan preview");
    assert_eq!(orphan.selected_versions.len(), 1);
    assert_eq!(orphan.reason, "orphan-preview");
}

#[test]
fn recorded_only_projection_omits_payload_and_grants_no_authority() {
    let (metadata, _) = admit(&resource_draft("projection", 1), &policy(&[])).expect("record");
    let projection = project_recorded_only(&metadata).expect("projection");
    // r[verify lattice.operation_data.valence_projection.valid]
    assert_eq!(projection.verification_role, "RecordedOnly");
    assert!(projection.non_claims.iter().any(|value| value == PROJECTION_NON_CLAIM));
    let encoded = serde_json::to_string(&projection).expect("projection JSON");
    assert!(!encoded.contains("counts"));
    assert!(!encoded.contains("issues"));
}

#[test]
fn shell_appends_versions_idempotently_and_detects_missing_payloads() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store_path = directory.path().join("lattice.redb");
    let plane = Plane::open(&store_path, directory.path()).expect("data plane");
    let draft = resource_draft("shell", 1);
    let (metadata, first) = plane.publish(&draft, &policy(&[])).expect("first publication");
    assert_eq!(first.outcome, PublicationOutcome::Appended);
    let (_, second) = plane.publish(&draft, &policy(&[])).expect("idempotent publication");
    // r[verify lattice.operation_data.versions.conflict]
    assert_eq!(second.outcome, PublicationOutcome::Idempotent);
    assert_eq!(plane.list_metadata().expect("metadata").len(), 1);
    assert!(!plane.read_payload(&metadata).expect("payload").is_empty());
    let mut exact_query = query();
    exact_query.version_identity = Some(metadata.version_identity.clone());
    exact_query.operation_type = None;
    exact_query.method = None;
    exact_query.output_name = None;
    exact_query.outcome = None;
    exact_query.maximum_result_count = 1;
    let resolved = plane
        .resolve_exact_query(&exact_query, QueryAuthority { allow_sensitive: false })
        .expect("exact query input");
    plane
        .recheck_resolved_input(&resolved, QueryAuthority { allow_sensitive: false })
        .expect("exact input recheck");

    std::fs::remove_file(directory.path().join(&metadata.payload.safe_locator)).expect("remove fixture payload");
    assert!(plane.recheck_resolved_input(&resolved, QueryAuthority { allow_sensitive: false }).is_err());
    // r[verify lattice.operation_data.versions.uncertain]
    assert_eq!(plane.recover_publications().expect("recovery"), vec![(
        metadata.version_identity,
        PublicationOutcome::Incomplete
    )]);
}

#[test]
fn concurrent_writers_append_distinct_versions_without_losing_lineage() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane =
        std::sync::Arc::new(Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane"));
    let first_plane = std::sync::Arc::clone(&plane);
    let second_plane = std::sync::Arc::clone(&plane);
    let first = std::thread::spawn(move || first_plane.publish(&resource_draft("writer-one", 1), &policy(&[])));
    let second = std::thread::spawn(move || second_plane.publish(&resource_draft("writer-two", 2), &policy(&[])));
    let first_record = first.join().expect("first writer").expect("first publication").0;
    let second_record = second.join().expect("second writer").expect("second publication").0;
    // r[verify lattice.operation_data.versions.append]
    assert_ne!(first_record.version_identity, second_record.version_identity);
    assert_eq!(plane.list_metadata().expect("metadata").len(), 2);
}

#[test]
fn active_or_uncertain_mutation_owner_blocks_effects_and_is_recoverable_evidence() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let lock = directory.path().join(".operation-data-mutation.lock");
    std::fs::write(&lock, b"active\n").expect("active owner fixture");
    let error = plane
        .publish(&resource_draft("blocked", 1), &policy(&[]))
        .expect_err("active owner blocks publication");
    assert!(error.to_string().contains("active or uncertain"));
    assert_eq!(plane.recover_publications().expect("owner recovery evidence").len(), 1);
    std::fs::remove_file(lock).expect("clear simulated owner");
    plane
        .publish(&resource_draft("recovered", 1), &policy(&[]))
        .expect("publication after explicit repair");
}

#[test]
fn recovery_reports_payload_written_before_metadata_as_unknown() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let (metadata, bytes) = admit(&resource_draft("crash", 1), &policy(&[])).expect("admission");
    let payload_path = directory.path().join(&metadata.payload.safe_locator);
    std::fs::write(payload_path, bytes).expect("simulate post-payload crash");
    let outcomes = plane.recover_publications().expect("recovery");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].1, PublicationOutcome::Unknown);
    assert!(plane.list_metadata().expect("metadata").is_empty());
}

#[test]
fn deletion_is_idempotent_and_keeps_shared_content_until_last_version() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let mut first_draft = resource_draft("first", 1);
    first_draft.lifetime = Lifetime::Ephemeral;
    let mut second_draft = resource_draft("second", 2);
    second_draft.lifetime = Lifetime::Ephemeral;
    let (first, _) = plane.publish(&first_draft, &policy(&[])).expect("first");
    let (second, _) = plane.publish(&second_draft, &policy(&[])).expect("second");
    assert_eq!(first.payload.content_identity, second.payload.content_identity);
    let shared_path = directory.path().join(&first.payload.safe_locator);

    let first_plan = retention_plan(std::slice::from_ref(&first), &RetentionFacts {
        now_epoch_seconds: 0,
        terminal_run_ids: Vec::new(),
    })
    .expect("first deletion plan");
    plane.execute_deletion(&first_plan).expect("delete first");
    assert!(shared_path.exists());

    let second_plan = retention_plan(std::slice::from_ref(&second), &RetentionFacts {
        now_epoch_seconds: 0,
        terminal_run_ids: Vec::new(),
    })
    .expect("second deletion plan");
    plane.execute_deletion(&second_plan).expect("delete second");
    assert!(!shared_path.exists());
    let repeated = plane.execute_deletion(&second_plan).expect("repeat delete");
    // r[verify lattice.operation_data.retention.gc]
    assert_eq!(repeated.missing_versions, vec![second.version_identity]);
}

#[test]
fn metadata_query_deletion_and_projection_identities_fail_closed_after_tampering() {
    let (metadata, bytes) = admit(&resource_draft("identity", 1), &policy(&[])).expect("record");

    let mut changed_tag = metadata.clone();
    changed_tag.tags.insert("environment".to_string(), "changed".to_string());
    assert!(matches!(validate_metadata(&changed_tag), Err(Error::Invalid("version identity"))));

    let mut changed_locator = metadata.clone();
    changed_locator.payload.safe_locator_identity = identity("wrong-locator");
    assert!(matches!(validate_metadata(&changed_locator), Err(Error::Invalid("payload locator identity"))));

    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("resource payload");
    let mut malformed_query = query();
    malformed_query.version_identity = Some("b3:not-an-identity".to_string());
    assert!(matches!(
        reduce_query(&[(metadata.clone(), Some(value.clone()))], &malformed_query, QueryAuthority {
            allow_sensitive: false
        }),
        Err(Error::Invalid("BLAKE3 identity"))
    ));

    let mut undeclared = query();
    undeclared.field_path = Some(vec!["valid".to_string()]);
    assert!(matches!(
        reduce_query(&[(metadata.clone(), Some(value))], &undeclared, QueryAuthority { allow_sensitive: false }),
        Err(Error::Invalid("undeclared structured field path"))
    ));

    let mut ephemeral = resource_draft("delete", 2);
    ephemeral.lifetime = Lifetime::Ephemeral;
    let (ephemeral, _) = admit(&ephemeral, &policy(&[])).expect("ephemeral record");
    let mut deletion = retention_plan(&[ephemeral], &RetentionFacts {
        now_epoch_seconds: 0,
        terminal_run_ids: Vec::new(),
    })
    .expect("deletion plan");
    deletion.identity = identity("tampered-deletion");
    assert!(matches!(validate_deletion_plan(&deletion), Err(Error::Invalid("deletion plan identity"))));

    let mut projection = project_recorded_only(&metadata).expect("projection");
    projection.verification_role = "Property".to_string();
    // r[verify lattice.operation_data.valence_projection.denied]
    assert!(matches!(
        validate_recorded_only_projection(&projection),
        Err(Error::Invalid("recorded-only projection policy"))
    ));
    let mut missing_non_claim = project_recorded_only(&metadata).expect("projection");
    missing_non_claim.non_claims.retain(|value| value != PROJECTION_NON_CLAIM);
    assert!(matches!(
        validate_recorded_only_projection(&missing_non_claim),
        Err(Error::Invalid("projection non-claim"))
    ));
}

#[test]
fn latest_selection_rejects_ties_and_shell_reads_only_selected_resource_payloads() {
    let (first, first_bytes) = admit(&resource_draft("latest-a", 1), &policy(&[])).expect("first");
    let (second, second_bytes) = admit(&resource_draft("latest-b", 1), &policy(&[])).expect("second");
    let tied = vec![
        QueryResult {
            metadata: first,
            selected_value: Some(serde_json::from_slice(&first_bytes).expect("first value")),
        },
        QueryResult {
            metadata: second,
            selected_value: Some(serde_json::from_slice(&second_bytes).expect("second value")),
        },
    ];
    assert_eq!(resolve_latest(&tied).expect_err("tie denied"), Error::Ambiguous);

    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let (selected, _) = plane.publish(&resource_draft("selected", 2), &policy(&[])).expect("selected record");
    let mut unrelated_draft = resource_draft("unrelated", 3);
    unrelated_draft.output_name = "other".to_string();
    unrelated_draft.payload = PayloadDraft::Resource {
        schema_identity: identity("schema"),
        selectable_paths: vec![vec!["counts".to_string(), "issues".to_string()]],
        value: serde_json::json!({"valid": false, "counts": {"issues": 1}}),
    };
    let (unrelated, _) = plane.publish(&unrelated_draft, &policy(&[])).expect("unrelated record");
    std::fs::remove_file(directory.path().join(&unrelated.payload.safe_locator)).expect("remove unrelated payload");

    let mut field_query = query();
    field_query.field_path = Some(vec!["counts".to_string(), "issues".to_string()]);
    let results = plane
        .query(&field_query, QueryAuthority { allow_sensitive: false })
        .expect("selected query ignores unrelated missing payload");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].metadata.version_identity, selected.version_identity);
    assert_eq!(results[0].selected_value, Some(serde_json::json!(0)));
}

#[test]
fn confidentiality_covers_metadata_file_bytes_and_sensitive_projection() {
    let fragments = vec!["derived-secret".to_string()];
    let mut metadata_secret = resource_draft("metadata-secret", 1);
    metadata_secret.producer.node = "derived-secret".to_string();
    assert_eq!(admit(&metadata_secret, &policy(&fragments)).expect_err("metadata secret denied"), Error::Secret);

    let mut private_path = resource_draft("private-path", 2);
    private_path.tags.insert("source".to_string(), "/home/operator/data".to_string());
    assert_eq!(admit(&private_path, &policy(&[])).expect_err("private path denied"), Error::Secret);

    let mut file_secret = resource_draft("file-secret", 3);
    file_secret.payload = PayloadDraft::File {
        media_class: "application/octet-stream".to_string(),
        bytes: b"prefix-derived-secret-suffix".to_vec(),
    };
    assert_eq!(admit(&file_secret, &policy(&fragments)).expect_err("file secret denied"), Error::Secret);

    let mut sensitive = resource_draft("sensitive-projection", 4);
    sensitive.sensitivity = SensitivityClass::Sensitive;
    let (sensitive, _) = admit(&sensitive, &AdmissionPolicy {
        allow_sensitive: true,
        prohibited_fragments: &[],
    })
    .expect("sensitive admission");
    assert_eq!(project_recorded_only(&sensitive).expect_err("sensitive projection denied"), Error::Secret);
}

#[test]
fn retention_applies_explicit_lifetimes_and_version_limits() {
    let mut ephemeral = resource_draft("ephemeral", 1);
    ephemeral.lifetime = Lifetime::Ephemeral;
    let mut run = resource_draft("run", 2);
    run.lifetime = Lifetime::Run { run_id: 2 };
    let mut duration = resource_draft("duration", 3);
    duration.lifetime = Lifetime::Duration {
        expires_epoch_seconds: 10,
    };
    let retained = resource_draft("retained", 4);
    let records = [ephemeral, run, duration, retained]
        .iter()
        .map(|draft| admit(draft, &policy(&[])).expect("record").0)
        .collect::<Vec<_>>();
    let plan = retention_plan(&records, &RetentionFacts {
        now_epoch_seconds: 10,
        terminal_run_ids: vec![2],
    })
    .expect("retention plan");
    // r[verify lattice.operation_data.retention.gc]
    assert!(plan.selected_versions.contains(&records[0].version_identity));
    assert!(plan.selected_versions.contains(&records[1].version_identity));
    assert!(plan.selected_versions.contains(&records[2].version_identity));
    assert!(!plan.selected_versions.contains(&records[3].version_identity));
}

#[test]
fn deletion_failure_is_explicit_and_secure_erasure_overclaims_do_not_mutate() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let mut draft = resource_draft("deletion-failure", 1);
    draft.lifetime = Lifetime::Ephemeral;
    let (metadata, _) = plane.publish(&draft, &policy(&[])).expect("publication");
    let mut plan = retention_plan(std::slice::from_ref(&metadata), &RetentionFacts {
        now_epoch_seconds: 0,
        terminal_run_ids: Vec::new(),
    })
    .expect("deletion plan");
    plan.secure_erasure_claimed = true;
    assert!(plane.execute_deletion(&plan).is_err());
    assert!(plane.load_metadata(&metadata.version_identity).expect("metadata").is_some());

    plan.secure_erasure_claimed = false;
    let payload = directory.path().join(&metadata.payload.safe_locator);
    std::fs::remove_file(&payload).expect("remove payload fixture");
    std::fs::create_dir(&payload).expect("create deletion failure fixture");
    assert!(plane.execute_deletion(&plan).is_err());
    assert!(plane.load_metadata(&metadata.version_identity).expect("metadata").is_none());
    assert!(
        plane
            .recover_publications()
            .expect("recovery evidence")
            .iter()
            .any(|(_, outcome)| *outcome == PublicationOutcome::Unknown)
    );
}

#[test]
fn file_payload_is_opaque_to_field_queries_and_sensitive_reads_need_authority() {
    let mut file = resource_draft("file", 1);
    file.payload = PayloadDraft::File {
        media_class: "application/octet-stream".to_string(),
        bytes: b"opaque".to_vec(),
    };
    let (file_metadata, _) = admit(&file, &policy(&[])).expect("file");
    let mut field_query = query();
    field_query.field_path = Some(vec!["field".to_string()]);
    let candidates = vec![(file_metadata, None)];
    // r[verify lattice.operation_data.records.file]
    assert!(matches!(
        reduce_query(&candidates, &field_query, QueryAuthority { allow_sensitive: false }),
        Err(Error::Invalid("file payload is opaque"))
    ));

    let mut sensitive = resource_draft("sensitive", 2);
    sensitive.sensitivity = SensitivityClass::Sensitive;
    let sensitive_policy = AdmissionPolicy {
        allow_sensitive: true,
        prohibited_fragments: &[],
    };
    let (metadata, _) = admit(&sensitive, &sensitive_policy).expect("sensitive record");
    // r[verify lattice.operation_data.confidentiality.safe]
    assert_eq!(
        reduce_query(&[(metadata, None)], &query(), QueryAuthority { allow_sensitive: false })
            .expect_err("sensitive query denied"),
        Error::Secret
    );
}

#[test]
fn conflicting_payload_publication_preserves_existing_evidence() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let plane = Plane::open(&directory.path().join("lattice.redb"), directory.path()).expect("data plane");
    let draft = resource_draft("conflicting-payload", 1);
    let (metadata, _) = plane.publish(&draft, &policy(&[])).expect("original publication");
    let payload_path = directory.path().join(&metadata.payload.safe_locator);
    let conflicting_bytes = b"conflicting-payload-bytes";
    std::fs::write(&payload_path, conflicting_bytes).expect("conflicting evidence fixture");

    // r[verify lattice.operation_data.versions.conflict]
    assert!(matches!(
        plane.publish(&draft, &policy(&[])),
        Err(lattice::operation_data::shell::Error::Conflict(_))
    ));
    assert_eq!(std::fs::read(&payload_path).expect("existing payload evidence"), conflicting_bytes);
    assert_eq!(plane.load_metadata(&metadata.version_identity).expect("exact metadata"), Some(metadata.clone()));
    assert_eq!(plane.list_metadata().expect("metadata index"), vec![metadata]);
}
