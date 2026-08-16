use lattice::operation_pack as pack;

const VALID_FIXTURE: &str = "tests/fixtures/operation-packs/valid";
const FIXTURE_RUN_ID: u64 = 7;
const ANALYZER_TIMEOUT_SECONDS: u64 = 10;
const ANALYZER_OUTPUT_BYTES: usize = 65_536;

fn analyzer() -> pack::AnalyzerManifest {
    let candidate = pack::shell::load(&std::fs::canonicalize(VALID_FIXTURE).expect("fixture root")).expect("candidate");
    candidate.manifest.analyzers[0].clone()
}

fn valid_output() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "schema": pack::model::ANALYZER_OUTPUT_SCHEMA,
        "structured": {"summary": "bounded"},
        "markdown": "# Bounded",
        "non_claims": ["analyzer output does not strengthen the original operation observation"]
    }))
    .expect("output bytes")
}

fn outcome(required: bool, status: pack::analyzer::Status) -> pack::analyzer::Outcome {
    pack::analyzer::Outcome {
        analyzer_id: "fixture".to_string(),
        analyzer_identity: pack::content_identity(b"fixture analyzer"),
        required,
        status,
        published_version_identity: None,
        diagnostic: "fixture".to_string(),
        content_omitted: true,
    }
}

#[test]
fn bounded_clear_environment_executes_the_declared_analyzer_artifact() {
    let root = std::fs::canonicalize(VALID_FIXTURE).expect("fixture root");
    let request = lattice::agent_node::ProcessRequest {
        program: root.join("analyzer-fixture").to_string_lossy().into_owned(),
        args: vec!["analyze".to_string(), "summary".to_string()],
        cwd: Some(root.to_string_lossy().into_owned()),
        env: std::collections::BTreeMap::new(),
        stdin: b"{}".to_vec(),
        timeout_secs: ANALYZER_TIMEOUT_SECONDS,
        output_limit_bytes: ANALYZER_OUTPUT_BYTES,
    };
    let output = lattice::process::shell::run_bounded_clear_environment(
        &request,
        &lattice::process::core::RuntimeConfig::default(),
    )
    .expect("bounded analyzer process");
    assert_eq!(output.status_code, Some(0), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn typed_analyzer_output_accepts_bounded_structured_and_markdown_views() {
    let output = pack::analyzer::classify_output(&analyzer(), &valid_output(), &[]).expect("valid output");
    assert_eq!(output.structured["summary"], "bounded");
    assert_eq!(output.markdown.as_deref(), Some("# Bounded"));
}

#[test]
fn analyzer_output_rejects_schema_non_claim_authority_secret_and_markdown_drift() {
    let manifest = analyzer();
    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["schema"] = serde_json::Value::String("unknown".to_string());
    assert!(
        pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("schema bytes"), &[]).is_err()
    );

    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["non_claims"] = serde_json::json!([]);
    assert!(
        pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("claim bytes"), &[]).is_err()
    );

    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["structured"] = serde_json::json!({"authority": true});
    assert!(
        pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("authority bytes"), &[]).is_err()
    );

    let secret = "fixture-secret".to_string();
    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["structured"] = serde_json::json!({"summary": secret});
    assert!(
        pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("secret bytes"), &[
            "fixture-secret".to_string()
        ],)
        .is_err()
    );

    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["markdown"] = serde_json::Value::String("/home/private/analysis".to_string());
    assert!(pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("path bytes"), &[]).is_err());

    let mut value: serde_json::Value = serde_json::from_slice(&valid_output()).expect("output value");
    value["markdown"] = serde_json::Value::String("x".repeat(manifest.maximum_markdown_bytes.saturating_add(1)));
    assert!(
        pack::analyzer::classify_output(&manifest, &serde_json::to_vec(&value).expect("oversized bytes"), &[]).is_err()
    );

    let mut no_markdown = manifest;
    no_markdown.markdown_allowed = false;
    assert!(pack::analyzer::classify_output(&no_markdown, &valid_output(), &[]).is_err());
}

#[test]
fn required_failures_block_parent_while_optional_partial_failures_remain_explicit() {
    let completed = outcome(true, pack::analyzer::Status::Completed);
    let required_timeout = outcome(true, pack::analyzer::Status::TimedOut);
    let required_truncation = outcome(true, pack::analyzer::Status::Truncated);
    let optional_failure = outcome(false, pack::analyzer::Status::Failed);

    assert!(pack::analyzer::parent_succeeded(true, std::slice::from_ref(&completed)));
    assert!(!pack::analyzer::parent_succeeded(true, &[completed.clone(), required_timeout]));
    assert!(!pack::analyzer::parent_succeeded(true, &[completed.clone(), required_truncation]));
    assert!(pack::analyzer::parent_succeeded(true, &[completed, optional_failure]));
    assert!(!pack::analyzer::parent_succeeded(false, &[]));
}

#[test]
fn direct_selection_excludes_workflow_scope_analyzers() {
    let method = analyzer();
    let mut workflow = method.clone();
    workflow.id = "workflow-summary".to_string();
    workflow.scope = pack::AnalyzerScope::Workflow;
    let analyzers = [method, workflow];
    let selected = pack::analyzer::select_for_direct(&analyzers);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].scope, pack::AnalyzerScope::Method);
}

#[test]
fn analyzer_inputs_expose_only_declared_metadata_and_exact_data_references() {
    let manifest = analyzer();
    let reference = pack::analyzer::DataReference {
        version_identity: pack::content_identity(b"version"),
        content_identity: pack::content_identity(b"content"),
    };
    let input = pack::analyzer::plan_input(
        &pack::content_identity(b"pack"),
        &manifest,
        FIXTURE_RUN_ID,
        "completed",
        std::slice::from_ref(&reference),
    )
    .expect("analyzer input");
    assert_eq!(input.run_id, Some(FIXTURE_RUN_ID));
    assert_eq!(input.data_references, vec![reference]);

    let mut metadata_only = manifest;
    metadata_only.data_view = vec![pack::AnalyzerDataView::RunMetadata];
    let input =
        pack::analyzer::plan_input(&pack::content_identity(b"pack"), &metadata_only, FIXTURE_RUN_ID, "completed", &[])
            .expect("metadata-only input");
    assert!(input.data_references.is_empty());
}
