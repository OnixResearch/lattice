use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};

use super::*;

const TEST_INPUT_LIMIT: u64 = 65_536;
const TEST_EXTENSION_BYTE_LIMIT: usize = 65_536;
const TEST_COLLECTION_LIMIT: usize = 64;
const TEST_STRING_LIMIT: usize = 4_096;
const TEST_EXTENSION_LIMIT: usize = 1_024;
const TEST_DEPTH_LIMIT: usize = 8;
const SECOND_STEP_ID: u32 = 2;
const INVALID_TOTAL_STEPS: u64 = 3;
const EXPECTED_EXTERNAL_REFERENCES: u32 = 3;

fn policy() -> ValidatedAtifPolicy {
    validate_atif_policy(AtifPolicy {
        schema: POLICY_SCHEMA.into(),
        profile_id: "animus.harbor-atif.test.v1".into(),
        harbor_release: HARBOR_RELEASE.into(),
        harbor_revision: HARBOR_REVISION.into(),
        atif_schema: ATIF_SCHEMA.into(),
        codec_identity: CODEC_IDENTITY.into(),
        source_cohort_identity: SOURCE_COHORT_IDENTITY.into(),
        source_repository: "laude-institute/harbor".into(),
        source_license: "Apache-2.0".into(),
        source_paths: vec![
            "rfcs/0001-trajectory-format.md".into(),
            "src/harbor/models/trajectories".into(),
            "src/harbor/utils/trajectory_validator.py".into(),
            "tests/golden".into(),
        ],
        bounds: AtifBounds {
            max_input_bytes: TEST_INPUT_LIMIT,
            max_trajectories: TEST_COLLECTION_LIMIT,
            max_depth: TEST_DEPTH_LIMIT,
            max_steps: TEST_COLLECTION_LIMIT,
            max_aggregate_steps: TEST_EXTENSION_LIMIT,
            max_string_bytes: TEST_STRING_LIMIT,
            max_content_parts: TEST_COLLECTION_LIMIT,
            max_tool_definitions: TEST_COLLECTION_LIMIT,
            max_tool_calls: TEST_COLLECTION_LIMIT,
            max_observation_results: TEST_COLLECTION_LIMIT,
            max_subagent_refs: TEST_COLLECTION_LIMIT,
            max_token_items: TEST_EXTENSION_LIMIT,
            max_metrics_entries: TEST_COLLECTION_LIMIT,
            max_extension_nodes: TEST_EXTENSION_LIMIT,
            max_extension_depth: TEST_DEPTH_LIMIT,
            max_extension_bytes: TEST_EXTENSION_BYTE_LIMIT,
            max_loss_classes: TEST_COLLECTION_LIMIT,
        },
        projection: ProjectionPolicy {
            messages: FieldDisposition::Include,
            reasoning: FieldDisposition::Omit,
            tool_arguments: FieldDisposition::Omit,
            tool_results: FieldDisposition::Redact,
            images: FieldDisposition::Unresolved,
            token_ids: FieldDisposition::Omit,
            logprobs: FieldDisposition::Omit,
            notes: FieldDisposition::Omit,
            provider_model: FieldDisposition::Hash,
            extensions: FieldDisposition::Omit,
            copied_context_dataset: FieldDisposition::Reject,
        },
        external_references: ExternalReferencePolicy::Unresolved,
        extension_scalar_allowlist: vec!["trace_class".into()],
        receipt_fields: REQUIRED_ATIF_RECEIPT_FIELDS
            .iter()
            .map(|value| String::from(*value))
            .collect(),
        required_non_claims: REQUIRED_ATIF_NON_CLAIMS
            .iter()
            .map(|value| String::from(*value))
            .collect(),
        export_defaults: AtifExportDefaults {
            invent_timestamps: false,
            invent_metrics: false,
            invent_reasoning: false,
            invent_tool_outcomes: false,
            invent_external_resources: false,
            invent_copied_context: false,
            invent_llm_call_count: false,
            invent_terminal_state: false,
        },
        ambient_resolution: false,
        automatic_persistence: false,
        automatic_dataset_promotion: false,
        automatic_evaluation_promotion: false,
    })
    .expect("test policy must validate")
}

#[cfg(feature = "json-codec")]
const GRAPH_JSON: &[u8] = br#"{
  "schema_version":"ATIF-v1.7",
  "session_id":"shared-run",
  "trajectory_id":"parent",
  "agent":{"name":"parent-agent","version":"1"},
  "steps":[
    {"step_id":1,"source":"user","message":[
      {"type":"text","text":"inspect"},
      {"type":"image","source":{"media_type":"image/png","path":"artifact://image-a"}}
    ],"is_copied_context":true},
    {"step_id":2,"source":"agent","message":"delegate",
     "tool_calls":[{"tool_call_id":"dispatch-a","function_name":"delegate","arguments":{}}],
     "observation":{"results":[{"source_call_id":"dispatch-a","subagent_trajectory_ref":[
       {"trajectory_id":"child-a","session_id":"shared-run"},
       {"trajectory_path":"artifact://external-child"}
     ]}]},"llm_call_count":0,"extra":{"trace_class":"bounded"}}
  ],
  "continued_trajectory_ref":"artifact://continuation",
  "subagent_trajectories":[{
    "schema_version":"ATIF-v1.7",
    "session_id":"shared-run",
    "trajectory_id":"child-a",
    "agent":{"name":"child-agent","version":"1"},
    "steps":[{"step_id":1,"source":"user","message":"work"}]
  }]
}"#;

#[cfg(feature = "json-codec")]
const VALID_JSON: &[u8] = br#"{
  "schema_version":"ATIF-v1.7",
  "session_id":"run-a",
  "trajectory_id":"root",
  "agent":{"name":"animus","version":"1","model_name":"model-a"},
  "steps":[
    {"step_id":1,"timestamp":"2026-08-08T00:00:00Z","source":"user","message":"hello"},
    {"step_id":2,"source":"agent","model_name":"model-a","message":"done",
     "tool_calls":[{"tool_call_id":"call-a","function_name":"echo","arguments":{"command":"echo"}}],
     "observation":{"results":[{"source_call_id":"call-a","content":"ok"}]},
     "metrics":{"prompt_tokens":1,"completion_tokens":1,"cached_tokens":0,"cost_usd":0.1,
                "prompt_token_ids":[1],"completion_token_ids":[2],"logprobs":[-0.1]},
     "llm_call_count":1}
  ],
  "final_metrics":{"total_prompt_tokens":1,"total_completion_tokens":1,"total_cached_tokens":0,
                   "total_cost_usd":0.1,"total_steps":2}
}"#;

#[cfg(feature = "json-codec")]
#[test]
fn generated_nickel_policy_revalidates_in_rust() {
    let bytes = include_bytes!("../../../config/generated/harbor-atif-profile.json");
    let generated = decode_atif_policy_json(bytes).expect("generated policy must revalidate");
    assert_eq!(generated.value().harbor_revision, HARBOR_REVISION);
    assert_eq!(generated.value().atif_schema, ATIF_SCHEMA);
    assert!(generated.identity().starts_with("b3:"));
}

#[cfg(feature = "json-codec")]
#[test]
fn valid_json_round_trips_and_projects_with_explicit_losses() {
    let policy = policy();
    let decoded = decode_atif_json(VALID_JSON, &policy).expect("valid ATIF must decode");
    assert_eq!(decoded.validated.counts().steps, SECOND_STEP_ID);
    assert!(decoded.wire_identity.starts_with("b3:"));

    let projection = import_validated_trajectory(&decoded.validated, &policy)
        .expect("validated input must project");
    assert_eq!(
        projection.ledger.len(),
        usize::try_from(SECOND_STEP_ID).unwrap()
    );
    assert!(projection.losses.contains(&AtifLossClass::ToolArguments));
    assert!(projection.losses.contains(&AtifLossClass::ToolResults));
    assert!(projection.losses.contains(&AtifLossClass::TokenIds));
    assert!(projection.losses.contains(&AtifLossClass::Logprobs));
    assert_eq!(projection.status, AtifProjectionStatus::LossReported);

    let encoded = encode_atif_json(&decoded.validated, &policy).expect("valid value must encode");
    let round_trip = decode_atif_json(&encoded.bytes, &policy).expect("output must revalidate");
    assert_eq!(
        round_trip.validated.semantic_identity(),
        decoded.validated.semantic_identity()
    );

    let receipt = build_atif_receipt(
        AtifReceiptInput {
            operation: AtifOperation::Import,
            validated: &decoded.validated,
            projection_identity: Some(&projection.projection_identity),
            projection_status: Some(projection.status),
            losses: &projection.losses,
            dispositions: &projection.dispositions,
            wire_input_identity: Some(&decoded.wire_identity),
            wire_output_identity: None,
            claims: &[
                AtifBoundedClaim::ExactCohortMatched,
                AtifBoundedClaim::CallerBytesDecoded,
                AtifBoundedClaim::TypedSchemaValidated,
                AtifBoundedClaim::ProjectionLossReported,
            ],
        },
        &policy,
    )
    .expect("safe metadata must produce a receipt");
    assert_eq!(receipt.counts.steps, SECOND_STEP_ID);
    assert!(receipt.disposition_classes.contains(&"redaction"));
    assert!(receipt.disposition_classes.contains(&"hash_retention"));
    assert_eq!(receipt.non_claims.len(), REQUIRED_ATIF_NON_CLAIMS.len());
}

#[cfg(feature = "json-codec")]
#[test]
fn multimodal_no_llm_embedded_and_external_facts_remain_bounded() {
    let policy = policy();
    let decoded =
        decode_atif_json(GRAPH_JSON, &policy).expect("complete graph fixture must decode");
    assert_eq!(decoded.validated.counts().trajectories, SECOND_STEP_ID);
    assert_eq!(decoded.validated.counts().no_llm_steps, 1);
    assert_eq!(decoded.validated.counts().copied_context_steps, 1);
    assert_eq!(
        decoded.validated.counts().external_references,
        EXPECTED_EXTERNAL_REFERENCES
    );
    assert!(
        decoded
            .validated
            .references()
            .iter()
            .any(|fact| fact.disposition == ReferenceDisposition::Embedded)
    );

    let imported = import_validated_trajectory(&decoded.validated, &policy)
        .expect("validated graph must project");
    assert!(!imported.dataset[0].eligible);
    assert_eq!(imported.lineage.len(), 1);
    assert!(imported.losses.contains(&AtifLossClass::ImageReference));
    assert!(imported.losses.contains(&AtifLossClass::ExternalReference));

    let mut hash_extensions = policy.value().clone();
    hash_extensions.projection.extensions = FieldDisposition::Hash;
    let hash_extensions = validate_atif_policy(hash_extensions).expect("hash policy must validate");
    decode_atif_json(GRAPH_JSON, &hash_extensions)
        .expect("allowlisted scalar extension must admit hash classification");

    let mut reject_external = policy.value().clone();
    reject_external.external_references = ExternalReferencePolicy::Reject;
    let reject_external = validate_atif_policy(reject_external).expect("deny policy must validate");
    assert_eq!(
        decode_atif_json(GRAPH_JSON, &reject_external)
            .expect_err("external references must obey deny policy")
            .class,
        AtifIssueClass::Reference
    );
}

#[cfg(feature = "json-codec")]
#[test]
fn malformed_duplicate_unknown_and_oversized_json_fail_closed() {
    let policy = policy();
    let malformed = decode_atif_json(b"{", &policy).expect_err("malformed JSON must fail");
    assert_eq!(malformed.class, AtifIssueClass::Codec);

    let duplicate = br#"{"schema_version":"ATIF-v1.7","schema_version":"ATIF-v1.7"}"#;
    assert_eq!(
        decode_atif_json(duplicate, &policy)
            .expect_err("duplicate key must fail")
            .class,
        AtifIssueClass::Codec
    );

    let nested_duplicate = br#"{
      "schema_version":"ATIF-v1.7",
      "agent":{"name":"a","version":"1"},
      "steps":[{"step_id":1,"source":"agent","message":"x","tool_calls":[
        {"tool_call_id":"c","function_name":"f","arguments":{"x":1,"x":2}}
      ]}]
    }"#;
    assert_eq!(
        decode_atif_json(nested_duplicate, &policy)
            .expect_err("nested duplicate object key must fail")
            .class,
        AtifIssueClass::Codec
    );

    let unknown = br#"{"schema_version":"ATIF-v1.7","agent":{"name":"a","version":"1"},"steps":[],"mystery":true}"#;
    assert_eq!(
        decode_atif_json(unknown, &policy)
            .expect_err("unknown field must fail")
            .class,
        AtifIssueClass::Codec
    );

    let oversized = vec![b' '; usize::try_from(TEST_INPUT_LIMIT).unwrap() + 1];
    assert_eq!(
        decode_atif_json(&oversized, &policy)
            .expect_err("oversized input must fail")
            .class,
        AtifIssueClass::Bound
    );
}

#[cfg(feature = "json-codec")]
#[test]
fn semantic_validation_rejects_sequence_correlation_metrics_and_no_llm_conflicts() {
    let policy = policy();
    let decoded = decode_atif_json(VALID_JSON, &policy).expect("fixture must decode");

    let mut unknown_hashed_extension = decoded.validated.document().clone();
    let mut unknown_entries = BTreeMap::new();
    unknown_entries.insert("unreviewed".into(), AtifValue::String("value".into()));
    unknown_hashed_extension.extra = Some(AtifObject::new(unknown_entries));
    let mut hash_policy = policy.value().clone();
    hash_policy.projection.extensions = FieldDisposition::Hash;
    let hash_policy = validate_atif_policy(hash_policy).expect("hash policy must validate");
    assert_eq!(
        validate_trajectory(unknown_hashed_extension, &hash_policy)
            .expect_err("unreviewed hashed extension must fail")
            .class,
        AtifIssueClass::Extension
    );

    let mut unsafe_extension = decoded.validated.document().clone();
    let mut extension_entries = BTreeMap::new();
    extension_entries.insert("credential".into(), AtifValue::String("hidden".into()));
    unsafe_extension.extra = Some(AtifObject::new(extension_entries));
    assert_eq!(
        validate_trajectory(unsafe_extension, &policy)
            .expect_err("authority-bearing extension must fail")
            .class,
        AtifIssueClass::Extension
    );

    let mut bad_timestamp = decoded.validated.document().clone();
    bad_timestamp.steps[0].timestamp = Some("2026-02-30T00:00:00Z".into());
    assert_eq!(
        validate_trajectory(bad_timestamp, &policy)
            .expect_err("invalid calendar date must fail")
            .class,
        AtifIssueClass::Timestamp
    );

    let mut bad_sequence = decoded.validated.document().clone();
    bad_sequence.steps[1].step_id = 1;
    assert_eq!(
        validate_trajectory(bad_sequence, &policy)
            .expect_err("duplicate step ID must fail")
            .class,
        AtifIssueClass::StepSequence
    );

    let mut bad_correlation = decoded.validated.document().clone();
    bad_correlation.steps[1]
        .observation
        .as_mut()
        .expect("fixture observation")
        .results[0]
        .source_call_id = Some("missing".into());
    assert_eq!(
        validate_trajectory(bad_correlation, &policy)
            .expect_err("unknown call ID must fail")
            .class,
        AtifIssueClass::ToolCorrelation
    );

    let mut bad_metrics = decoded.validated.document().clone();
    bad_metrics.steps[1]
        .metrics
        .as_mut()
        .expect("fixture metrics")
        .cached_tokens = Some(SECOND_STEP_ID.into());
    assert_eq!(
        validate_trajectory(bad_metrics, &policy)
            .expect_err("cached tokens above prompt must fail")
            .class,
        AtifIssueClass::Metric
    );

    let mut bad_final_steps = decoded.validated.document().clone();
    bad_final_steps
        .final_metrics
        .as_mut()
        .expect("fixture final metrics")
        .total_steps = Some(INVALID_TOTAL_STEPS);
    assert_eq!(
        validate_trajectory(bad_final_steps, &policy)
            .expect_err("undocumented final step difference must fail")
            .class,
        AtifIssueClass::Metric
    );

    let mut bad_dispatch = decoded.validated.document().clone();
    bad_dispatch.steps[1].llm_call_count = Some(0);
    assert_eq!(
        validate_trajectory(bad_dispatch, &policy)
            .expect_err("no-LLM contradiction must fail")
            .class,
        AtifIssueClass::NoLlm
    );
}

#[test]
fn resolved_graph_accepts_dag_and_rejects_cycle_or_missing_node() {
    let policy = policy();
    let root_refs = ["child"];
    let empty_refs: [&str; 0] = [];
    let dag = [
        ResolvedTrajectoryNode {
            trajectory_id: "root",
            references: &root_refs,
        },
        ResolvedTrajectoryNode {
            trajectory_id: "child",
            references: &empty_refs,
        },
    ];
    validate_resolved_graph(&dag, &policy).expect("finite DAG must validate");

    let duplicate_refs = ["child", "child"];
    let duplicate_edge = [
        ResolvedTrajectoryNode {
            trajectory_id: "root",
            references: &duplicate_refs,
        },
        ResolvedTrajectoryNode {
            trajectory_id: "child",
            references: &empty_refs,
        },
    ];
    assert_eq!(
        validate_resolved_graph(&duplicate_edge, &policy)
            .expect_err("duplicate graph edge must fail")
            .class,
        AtifIssueClass::Reference
    );

    let child_cycle = ["root"];
    let cycle = [
        ResolvedTrajectoryNode {
            trajectory_id: "root",
            references: &root_refs,
        },
        ResolvedTrajectoryNode {
            trajectory_id: "child",
            references: &child_cycle,
        },
    ];
    assert_eq!(
        validate_resolved_graph(&cycle, &policy)
            .expect_err("cycle must fail")
            .class,
        AtifIssueClass::Cycle
    );

    let missing_refs = ["absent"];
    let missing = [ResolvedTrajectoryNode {
        trajectory_id: "root",
        references: &missing_refs,
    }];
    assert_eq!(
        validate_resolved_graph(&missing, &policy)
            .expect_err("unresolved graph target must fail")
            .class,
        AtifIssueClass::Reference
    );
}

#[test]
fn unsafe_policy_and_export_without_source_facts_fail_closed() {
    let mut unsafe_policy = policy().value().clone();
    unsafe_policy.ambient_resolution = true;
    assert_eq!(
        validate_atif_policy(unsafe_policy)
            .expect_err("ambient resolution must fail")
            .class,
        AtifIssueClass::Authority
    );

    let snapshot = AnimusTrajectorySnapshot {
        trajectory_identity: "export-root".into(),
        session_identity: Some("run-export".into()),
        agent_name: "animus".into(),
        agent_version: "1".into(),
        model_name: Some("model-a".into()),
        steps: vec![AnimusStepSnapshot {
            source: StepSource::Agent,
            message: Some("answer".into()),
            model_name: Some("model-a".into()),
            tool_calls: Vec::new(),
            observations: Vec::new(),
            metrics: None,
            copied_context: None,
            llm_call_count: Some(1),
        }],
    };
    let exported = export_trajectory_snapshot(&snapshot, &policy())
        .expect("admitted snapshot must export and revalidate");
    assert!(exported.validated.document().steps[0].timestamp.is_none());
    assert!(
        exported.validated.document().steps[0]
            .reasoning_content
            .is_none()
    );
    assert!(exported.losses.contains(&AtifLossClass::ProviderModel));
}
