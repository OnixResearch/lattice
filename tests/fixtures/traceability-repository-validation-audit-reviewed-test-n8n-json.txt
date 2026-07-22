//! Tests for n8n JSON import/export interoperability.

use std::process::Command;

use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::file_node::FileOperationKind;
use lattice::n8n_json::N8nDiagnosticCode;
use lattice::n8n_json::export_workflow_to_n8n_json;
use lattice::n8n_json::import_n8n_json_str;
use lattice::n8n_json::report_to_pretty_json;
use lattice::store::Store;
use lattice::workflow_validation;
use pretty_assertions::assert_eq;

const SHELL_FIXTURE: &str = include_str!("fixtures/n8n/shell.json");
const HTTP_FIXTURE: &str = include_str!("fixtures/n8n/http.json");
const UNSUPPORTED_NODE_FIXTURE: &str = include_str!("fixtures/n8n/unsupported-node.json");
const PLAINTEXT_CREDENTIAL_FIXTURE: &str = include_str!("fixtures/n8n/plaintext-credentials.json");
const MALFORMED_FIXTURE: &str = include_str!("fixtures/n8n/malformed.json");
const MALFORMED_CONNECTION_FIXTURE: &str = include_str!("fixtures/n8n/malformed-connection.json");
const UNSUPPORTED_EXPRESSION_FIXTURE: &str = include_str!("fixtures/n8n/unsupported-expression.json");
const PARITY_README: &str = include_str!("fixtures/n8n/parity/README.md");
const PARITY_COMMAND_PIPELINE_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.json");
const PARITY_COMMAND_PIPELINE_REPORT: &str = include_str!("fixtures/n8n/parity/supported/command-pipeline.report.json");
const PARITY_HTTP_CREDENTIAL_CHAIN_FIXTURE: &str =
    include_str!("fixtures/n8n/parity/supported/http-credential-chain.json");
const PARITY_HTTP_CREDENTIAL_CHAIN_REPORT: &str =
    include_str!("fixtures/n8n/parity/supported/http-credential-chain.report.json");
const PARITY_DEFAULT_HTTP_METHOD_FIXTURE: &str = include_str!("fixtures/n8n/parity/supported/default-http-method.json");
const PARITY_WEBHOOK_TRIGGER_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.json");
const PARITY_WEBHOOK_TRIGGER_REPORT: &str = include_str!("fixtures/n8n/parity/unsupported/webhook-trigger.report.json");
const PARITY_SCHEDULE_TRIGGER_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/schedule-trigger.json");
const PARITY_EXPRESSION_URL_FIXTURE: &str = include_str!("fixtures/n8n/parity/unsupported/expression-url.json");
const PARITY_PLAINTEXT_CREDENTIAL_FIXTURE: &str =
    include_str!("fixtures/n8n/parity/unsupported/plaintext-node-credential.json");
const PARITY_MALFORMED_CONNECTION_FIXTURE: &str =
    include_str!("fixtures/n8n/parity/unsupported/malformed-connection.json");
const CONFIG_FILE_NAME: &str = "config.ncl";
const STORE_FILE_NAME: &str = "lattice.redb";
const REPORT_FILE_NAME: &str = "n8n-report.json";
const IMPORT_FILE_NAME: &str = "workflow.json";
const EXPORT_FILE_NAME: &str = "exported.json";
const DATA_DIR_NAME: &str = "data";
const IMPORTED_SHELL_WORKFLOW_NAME: &str = "n8n-shell-fixture";
const EXPECTED_SINGLE_NODE_COUNT: usize = 1;
const EXPECTED_ZERO_DIAGNOSTICS: usize = 0;
const DEFAULT_TIMEOUT_SECS: u64 = 60;
const MAX_PARALLELISM: usize = 4;
const EXPECTED_PIPELINE_NODE_COUNT: usize = 2;
const EXPECTED_PIPELINE_EDGE_COUNT: usize = 1;
const EXPECTED_EMPTY_EDGE_COUNT: usize = 0;
const EXPECTED_PARITY_DOC_FIXTURE_COUNT: usize = 8;
const PLAINTEXT_SECRET_SENTINEL: &str = "plaintext-secret";

fn write_test_config(temp_dir: &tempfile::TempDir) -> (std::path::PathBuf, std::path::PathBuf) {
    let data_dir = temp_dir.path().join(DATA_DIR_NAME);
    std::fs::create_dir_all(&data_dir).expect("data directory created");
    let config_path = temp_dir.path().join(CONFIG_FILE_NAME);
    let data_dir_string = data_dir.display().to_string();
    std::fs::write(
        &config_path,
        format!(
            "{{\n  data_dir = {:?},\n  default_timeout_secs = {DEFAULT_TIMEOUT_SECS},\n  shell = \"sh\",\n  max_parallelism = {MAX_PARALLELISM},\n}}\n",
            data_dir_string,
        ),
    )
    .expect("config written");
    (config_path, data_dir)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SupportedParityFixture {
    file_name: &'static str,
    text: &'static str,
    golden_report: Option<&'static str>,
    expected_nodes: usize,
    expected_edges: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedParityFixture {
    file_name: &'static str,
    text: &'static str,
    golden_report: Option<&'static str>,
    expected_code: N8nDiagnosticCode,
}

const SUPPORTED_PARITY_FIXTURES: &[SupportedParityFixture] = &[
    SupportedParityFixture {
        file_name: "command-pipeline.json",
        text: PARITY_COMMAND_PIPELINE_FIXTURE,
        golden_report: Some(PARITY_COMMAND_PIPELINE_REPORT),
        expected_nodes: EXPECTED_PIPELINE_NODE_COUNT,
        expected_edges: EXPECTED_PIPELINE_EDGE_COUNT,
    },
    SupportedParityFixture {
        file_name: "http-credential-chain.json",
        text: PARITY_HTTP_CREDENTIAL_CHAIN_FIXTURE,
        golden_report: Some(PARITY_HTTP_CREDENTIAL_CHAIN_REPORT),
        expected_nodes: EXPECTED_PIPELINE_NODE_COUNT,
        expected_edges: EXPECTED_PIPELINE_EDGE_COUNT,
    },
    SupportedParityFixture {
        file_name: "default-http-method.json",
        text: PARITY_DEFAULT_HTTP_METHOD_FIXTURE,
        golden_report: None,
        expected_nodes: EXPECTED_SINGLE_NODE_COUNT,
        expected_edges: EXPECTED_EMPTY_EDGE_COUNT,
    },
];

const UNSUPPORTED_PARITY_FIXTURES: &[UnsupportedParityFixture] = &[
    UnsupportedParityFixture {
        file_name: "webhook-trigger.json",
        text: PARITY_WEBHOOK_TRIGGER_FIXTURE,
        golden_report: Some(PARITY_WEBHOOK_TRIGGER_REPORT),
        expected_code: N8nDiagnosticCode::UnsupportedNodeKind,
    },
    UnsupportedParityFixture {
        file_name: "schedule-trigger.json",
        text: PARITY_SCHEDULE_TRIGGER_FIXTURE,
        golden_report: None,
        expected_code: N8nDiagnosticCode::UnsupportedNodeKind,
    },
    UnsupportedParityFixture {
        file_name: "expression-url.json",
        text: PARITY_EXPRESSION_URL_FIXTURE,
        golden_report: None,
        expected_code: N8nDiagnosticCode::UnsupportedExpression,
    },
    UnsupportedParityFixture {
        file_name: "plaintext-node-credential.json",
        text: PARITY_PLAINTEXT_CREDENTIAL_FIXTURE,
        golden_report: None,
        expected_code: N8nDiagnosticCode::PlaintextCredentialMaterial,
    },
    UnsupportedParityFixture {
        file_name: "malformed-connection.json",
        text: PARITY_MALFORMED_CONNECTION_FIXTURE,
        golden_report: None,
        expected_code: N8nDiagnosticCode::MalformedConnection,
    },
];

fn diagnostic_codes(result: &lattice::n8n_json::N8nImportResult) -> Vec<N8nDiagnosticCode> {
    result.report.diagnostics.iter().map(|diagnostic| diagnostic.code).collect()
}

fn assert_report_matches_golden(actual: &lattice::n8n_json::N8nMappingReport, golden: &str) {
    assert_eq!(report_to_pretty_json(actual), golden.trim_end());
}

// r[verify n8nclone.workflow_format.n8n_parity_fixture_pack.supported]
// r[verify n8nclone.workflow_format.n8n_parity_mapping_reports.supported]
// r[verify n8nclone.credentials.n8n_parity_secret_safety.placeholders]
#[test]
fn parity_supported_fixtures_import_validate_and_match_golden_reports() {
    for fixture in SUPPORTED_PARITY_FIXTURES {
        let result = import_n8n_json_str(fixture.text);
        let workflow = result.workflow.expect("supported parity fixture imports");
        let validation = workflow_validation::validate_workflow(&workflow);
        let report_json = report_to_pretty_json(&result.report);

        assert_eq!(workflow.graph.node_count(), fixture.expected_nodes, "{} node count", fixture.file_name);
        assert_eq!(workflow.graph.edge_count(), fixture.expected_edges, "{} edge count", fixture.file_name);
        assert!(validation.is_empty(), "{} validation diagnostics: {validation:?}", fixture.file_name);
        assert!(result.report.diagnostics.is_empty(), "{} import diagnostics", fixture.file_name);
        assert!(!report_json.contains(PLAINTEXT_SECRET_SENTINEL));
        if let Some(golden_report) = fixture.golden_report {
            assert_report_matches_golden(&result.report, golden_report);
        }
    }
}

// r[verify n8nclone.workflow_format.n8n_parity_fixture_pack.unsupported]
// r[verify n8nclone.workflow_validation.n8n_parity_diagnostics]
// r[verify n8nclone.workflow_format.n8n_parity_mapping_reports.unsupported]
// r[verify n8nclone.credentials.n8n_parity_secret_safety.plaintext_rejected]
#[test]
fn parity_unsupported_fixtures_fail_with_stable_safe_diagnostics() {
    for fixture in UNSUPPORTED_PARITY_FIXTURES {
        let result = import_n8n_json_str(fixture.text);
        let report_json = report_to_pretty_json(&result.report);

        assert!(result.workflow.is_none(), "{} must not partially import", fixture.file_name);
        assert!(diagnostic_codes(&result).contains(&fixture.expected_code), "{} diagnostic", fixture.file_name);
        assert!(!report_json.contains(PLAINTEXT_SECRET_SENTINEL));
        if let Some(golden_report) = fixture.golden_report {
            assert_report_matches_golden(&result.report, golden_report);
        }
    }
}

// r[verify n8nclone.workflow_format.n8n_parity_fixture_pack.docs]
#[test]
fn parity_fixture_documentation_lists_all_tested_fixture_files() {
    let documented_count =
        SUPPORTED_PARITY_FIXTURES.iter().filter(|fixture| PARITY_README.contains(fixture.file_name)).count()
            + UNSUPPORTED_PARITY_FIXTURES
                .iter()
                .filter(|fixture| PARITY_README.contains(fixture.file_name))
                .count();

    assert_eq!(documented_count, EXPECTED_PARITY_DOC_FIXTURE_COUNT);
}

// r[verify n8nclone.workflow_format.n8n_json.import]
// r[verify n8nclone.workflow_format.n8n_json.fixtures]
#[test]
fn supported_shell_fixture_imports_to_typed_workflow() {
    let result = import_n8n_json_str(SHELL_FIXTURE);
    let workflow = result.workflow.expect("supported shell fixture imports");

    assert_eq!(workflow.meta.name, IMPORTED_SHELL_WORKFLOW_NAME);
    assert_eq!(workflow.graph.node_count(), EXPECTED_SINGLE_NODE_COUNT);
    assert_eq!(result.report.diagnostics.len(), EXPECTED_ZERO_DIAGNOSTICS);
    let node = workflow.graph.nodes().next().expect("node imported");
    match workflow.node_config(node.id).expect("config imported") {
        NodeConfig::Shell { command, .. } => assert_eq!(command, "printf hello"),
        other => panic!("expected shell config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_format.n8n_json.import]
// r[verify n8nclone.credentials.n8n_json.placeholders]
#[test]
fn supported_http_fixture_imports_credential_placeholder() {
    let result = import_n8n_json_str(HTTP_FIXTURE);
    let workflow = result.workflow.expect("supported HTTP fixture imports");
    let node = workflow.graph.nodes().next().expect("node imported");
    let credentials = workflow.node_credentials(node.id).expect("credential refs imported");

    assert_eq!(workflow.graph.node_count(), EXPECTED_SINGLE_NODE_COUNT);
    assert_eq!(result.report.diagnostics.len(), EXPECTED_ZERO_DIAGNOSTICS);
    assert!(credentials.env.contains_key("N8N_CREDENTIAL_HTTPHEADERAUTH"));
    match workflow.node_config(node.id).expect("config imported") {
        NodeConfig::Http { method, url, body, .. } => {
            assert_eq!(method, "POST");
            assert_eq!(url, "https://example.com/api");
            assert_eq!(body.as_deref(), Some("payload"));
        }
        other => panic!("expected HTTP config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_format.n8n_json.export]
#[test]
fn supported_shell_workflow_exports_to_n8n_json() {
    let mut workflow = Workflow::new("exportable");
    let node_id = workflow.add_shell_node("Echo", "printf hello");
    let node = workflow.graph.node_mut(node_id).expect("export node exists");
    node.x = 10;
    node.y = 20;

    let result = export_workflow_to_n8n_json(&workflow);
    let json = result.json.expect("workflow exports");
    let nodes = json["nodes"].as_array().expect("nodes exported");

    assert_eq!(result.report.diagnostics.len(), EXPECTED_ZERO_DIAGNOSTICS);
    assert_eq!(nodes.len(), EXPECTED_SINGLE_NODE_COUNT);
    assert_eq!(nodes[0]["type"], "n8n-nodes-base.executeCommand");
    assert_eq!(nodes[0]["parameters"]["command"], "printf hello");
}

// r[verify n8nclone.workflow_validation.n8n_json.unsupported]
#[test]
fn unsupported_node_fixture_reports_fatal_diagnostic() {
    let result = import_n8n_json_str(UNSUPPORTED_NODE_FIXTURE);

    assert!(result.workflow.is_none());
    assert!(diagnostic_codes(&result).contains(&N8nDiagnosticCode::UnsupportedNodeKind));
}

// r[verify n8nclone.credentials.n8n_json.placeholders]
#[test]
fn plaintext_credentials_are_rejected() {
    let result = import_n8n_json_str(PLAINTEXT_CREDENTIAL_FIXTURE);

    assert!(result.workflow.is_none());
    assert!(diagnostic_codes(&result).contains(&N8nDiagnosticCode::PlaintextCredentialMaterial));
}

// r[verify n8nclone.workflow_validation.n8n_json.unsupported]
#[test]
fn malformed_json_is_rejected() {
    let result = import_n8n_json_str(MALFORMED_FIXTURE);

    assert!(result.workflow.is_none());
    assert!(diagnostic_codes(&result).contains(&N8nDiagnosticCode::MalformedJson));
}

// r[verify n8nclone.workflow_validation.n8n_json.unsupported]
#[test]
fn malformed_connections_are_rejected() {
    let result = import_n8n_json_str(MALFORMED_CONNECTION_FIXTURE);

    assert!(result.workflow.is_none());
    assert!(diagnostic_codes(&result).contains(&N8nDiagnosticCode::MalformedConnection));
}

// r[verify n8nclone.workflow_validation.n8n_json.unsupported]
#[test]
fn unsupported_expressions_are_rejected() {
    let result = import_n8n_json_str(UNSUPPORTED_EXPRESSION_FIXTURE);

    assert!(result.workflow.is_none());
    assert!(diagnostic_codes(&result).contains(&N8nDiagnosticCode::UnsupportedExpression));
}

// r[verify n8nclone.workflow_validation.n8n_json.export_limits]
#[test]
fn unsupported_export_node_reports_diagnostic() {
    let mut workflow = Workflow::new("unsupported-export");
    workflow.add_file_node("Read", FileOperationKind::Read, "input.txt");

    let result = export_workflow_to_n8n_json(&workflow);

    assert!(result.json.is_none());
    assert_eq!(result.report.diagnostics[0].code, N8nDiagnosticCode::UnsupportedExportNode);
}

// r[verify n8nclone.workflow_format.n8n_json.import_cli]
#[test]
fn import_n8n_cli_persists_workflow_and_report() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let import_path = temp_dir.path().join(IMPORT_FILE_NAME);
    let report_path = temp_dir.path().join(REPORT_FILE_NAME);
    std::fs::write(&import_path, SHELL_FIXTURE).expect("fixture written");

    let status = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("import-n8n")
        .arg(&import_path)
        .arg("--report")
        .arg(&report_path)
        .status()
        .expect("import command runs");
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let workflow = store
        .load_workflow(IMPORTED_SHELL_WORKFLOW_NAME)
        .expect("workflow load succeeds")
        .expect("workflow persisted");
    let report = std::fs::read_to_string(report_path).expect("report written");

    assert!(status.success());
    assert_eq!(workflow.meta.name, IMPORTED_SHELL_WORKFLOW_NAME);
    assert!(report.contains(IMPORTED_SHELL_WORKFLOW_NAME));
}

// r[verify n8nclone.workflow_format.n8n_json.export]
#[test]
fn export_n8n_cli_writes_workflow_and_report() {
    let temp_dir = tempfile::tempdir().expect("temp dir created");
    let (config_path, data_dir) = write_test_config(&temp_dir);
    let store = Store::new(data_dir.join(STORE_FILE_NAME)).expect("store opens");
    let mut workflow = Workflow::new("cli-export");
    workflow.add_shell_node("Echo", "printf hello");
    store.save_workflow(&workflow).expect("workflow saved");
    drop(store);
    let output_path = temp_dir.path().join(EXPORT_FILE_NAME);
    let report_path = temp_dir.path().join(REPORT_FILE_NAME);

    let status = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config_path)
        .arg("export-n8n")
        .arg("cli-export")
        .arg(&output_path)
        .arg("--report")
        .arg(&report_path)
        .status()
        .expect("export command runs");
    let output = std::fs::read_to_string(output_path).expect("export written");
    let report = std::fs::read_to_string(report_path).expect("report written");

    assert!(status.success());
    assert!(output.contains("n8n-nodes-base.executeCommand"));
    assert!(report.contains("cli-export"));
}
