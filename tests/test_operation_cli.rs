use std::collections::BTreeMap;
use std::process::Command;

const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAXIMUM_PARALLELISM: usize = 1;
const STORE_FILE_NAME: &str = "lattice.redb";

struct CliFixture {
    _root: tempfile::TempDir,
    config: std::path::PathBuf,
    catalog: std::path::PathBuf,
    instances: std::path::PathBuf,
    executable: std::path::PathBuf,
    data_dir: std::path::PathBuf,
}

fn fixture() -> CliFixture {
    let root = tempfile::tempdir().expect("CLI fixture root");
    let data_dir = root.path().join("data");
    let config = root.path().join("config.ncl");
    std::fs::write(
        &config,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = {DEFAULT_TIMEOUT_SECONDS}, shell = \"sh\", max_parallelism = {MAXIMUM_PARALLELISM} }}",
            data_dir.to_string_lossy()
        ),
    )
    .expect("config source");
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog = manifest.join("builtins/operations/catalog.ncl");
    let executable =
        std::fs::canonicalize(manifest.join("builtins/operations/cairn-read-only-fixture")).expect("fixture executor");
    let instances = root.path().join("instances.ncl");
    let revision = lattice::operation::format::load_catalog(&catalog).expect("catalog").types["cairn.repository"]
        .definition
        .revision
        .clone();
    std::fs::write(
        &instances,
        format!(
            r#"{{
  schema = "lattice.operation-instances.v1",
  instances = [{{
    id = "cli-repository",
    display_alias = "CLI repository",
    type_id = "cairn.repository",
    type_revision = "{revision}",
    target = {{ repository_root = {:?} }},
    tags = {{}},
    method_defaults = {{}},
    credentials = {{}},
  }}],
}}"#,
            root.path().to_string_lossy()
        ),
    )
    .expect("instance source");
    CliFixture {
        _root: root,
        config,
        catalog,
        instances,
        executable,
        data_dir,
    }
}

fn lattice_command(fixture: &CliFixture) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config").arg(&fixture.config);
    command
}

fn command(fixture: &CliFixture) -> Command {
    let mut command = lattice_command(fixture);
    command.arg("operation");
    command
}

fn operation_data_command(fixture: &CliFixture) -> Command {
    let mut command = lattice_command(fixture);
    command.arg("operation-data");
    command
}

fn operation_data_identity(label: &str) -> String {
    lattice::operation::identity::hash_bytes("test.operation-data-cli.v1", label.as_bytes())
}

fn operation_data_draft() -> lattice::operation_data::RecordDraft {
    lattice::operation_data::RecordDraft {
        logical_key: "cli-report".to_string(),
        sequence: 1,
        producer: lattice::operation_data::Producer {
            operation_type: "cairn.repository".to_string(),
            type_revision: operation_data_identity("type"),
            instance: "cli-repository".to_string(),
            method: "validate".to_string(),
            invocation: operation_data_identity("invocation"),
            run_id: 1,
            node: "operation-0".to_string(),
            workflow_revision: operation_data_identity("workflow"),
        },
        output_name: "report".to_string(),
        output_contract_identity: operation_data_identity("output-contract"),
        tags: BTreeMap::from([("environment".to_string(), "local".to_string())]),
        sensitivity: lattice::operation_data::SensitivityClass::Internal,
        lifetime: lattice::operation_data::Lifetime::Ephemeral,
        retention: lattice::operation_data::RetentionPolicy {
            identity: operation_data_identity("retention"),
            maximum_versions: Some(1),
        },
        outcome: "completed".to_string(),
        payload: lattice::operation_data::PayloadDraft::Resource {
            schema_identity: operation_data_identity("schema"),
            selectable_paths: vec![vec!["counts".to_string(), "issues".to_string()]],
            value: serde_json::json!({"valid": true, "counts": {"issues": 0}}),
        },
        required_non_claims: lattice::operation_data::REQUIRED_NON_CLAIMS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
    }
}

fn plan_args(command: &mut Command, fixture: &CliFixture) {
    command
        .arg("--catalog")
        .arg(&fixture.catalog)
        .arg("--instances")
        .arg(&fixture.instances)
        .arg("--instance")
        .arg("cli-repository")
        .arg("--method")
        .arg("change_list")
        .arg("--executable")
        .arg(&fixture.executable);
}

#[test]
fn catalog_discovery_is_safe_and_does_not_open_the_store() {
    let fixture = fixture();
    let output = command(&fixture)
        .arg("catalog")
        .arg("--catalog")
        .arg(&fixture.catalog)
        .arg("--instances")
        .arg(&fixture.instances)
        .output()
        .expect("catalog command");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(stdout.contains("cli-repository"));
    assert!(!stdout.contains(fixture._root.path().to_string_lossy().as_ref()));
    assert!(!fixture.data_dir.exists());
}

#[test]
fn run_is_preview_first_then_persists_exact_terminal_evidence() {
    let fixture = fixture();
    let mut preview = command(&fixture);
    preview.arg("run");
    plan_args(&mut preview, &fixture);
    let preview_output = preview.output().expect("run preview");
    assert!(preview_output.status.success());
    assert!(String::from_utf8_lossy(&preview_output.stdout).contains("execution_authority_granted"));
    assert!(!fixture.data_dir.exists());

    let mut execute = command(&fixture);
    execute.arg("run");
    plan_args(&mut execute, &fixture);
    execute.arg("--confirm-read-only");
    let executed = execute.output().expect("run execute");
    assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
    assert!(String::from_utf8_lossy(&executed.stdout).contains("completed"));
    assert!(fixture.data_dir.join(STORE_FILE_NAME).exists());

    let mut duplicate = command(&fixture);
    duplicate.arg("run");
    plan_args(&mut duplicate, &fixture);
    duplicate.arg("--confirm-read-only");
    let duplicate_output = duplicate.output().expect("duplicate run");
    assert!(!duplicate_output.status.success());
}

#[test]
fn doctor_checks_exact_operation_readiness_without_resolving_secrets() {
    let fixture = fixture();
    std::fs::create_dir(&fixture.data_dir).expect("doctor data directory");
    lattice::store::Store::new(fixture.data_dir.join(STORE_FILE_NAME)).expect("doctor store");
    let output = lattice_command(&fixture)
        .arg("doctor")
        .arg("--output")
        .arg("json")
        .arg("--operation-catalog")
        .arg(&fixture.catalog)
        .arg("--operation-instances")
        .arg(&fixture.instances)
        .arg("--operation-executable")
        .arg(&fixture.executable)
        .output()
        .expect("doctor operation readiness");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("doctor UTF-8");
    assert!(stdout.contains("operation_contracts"));
    assert!(!stdout.contains("repository_root"));
    assert!(!stdout.contains(fixture.executable.to_string_lossy().as_ref()));

    let partial = lattice_command(&fixture)
        .arg("doctor")
        .arg("--operation-catalog")
        .arg(&fixture.catalog)
        .output()
        .expect("partial doctor readiness");
    assert!(!partial.status.success());
    assert!(String::from_utf8_lossy(&partial.stdout).contains("needs a catalog, instances, and executable"));
}

#[test]
fn operation_data_cli_is_preview_first_create_only_and_metadata_safe() {
    let fixture = fixture();
    let payload_root = fixture.data_dir.join("operation-data");
    std::fs::create_dir_all(&payload_root).expect("operation data root");
    let plane = lattice::operation_data::Plane::open(&fixture.data_dir.join(STORE_FILE_NAME), &payload_root)
        .expect("operation data plane");
    let (metadata, _) = plane
        .publish(&operation_data_draft(), &lattice::operation_data::AdmissionPolicy {
            allow_sensitive: false,
            prohibited_fragments: &[],
        })
        .expect("operation data publication");
    drop(plane);

    let listed = operation_data_command(&fixture).arg("list").output().expect("list command");
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    let listed_text = String::from_utf8(listed.stdout).expect("list UTF-8");
    assert!(listed_text.contains(&metadata.version_identity));
    assert!(!listed_text.contains("\"valid\""));

    let selector = fixture._root.path().join("selector.ncl");
    std::fs::write(
        &selector,
        format!(
            r#"{{
  schema = "lattice.operation-data-query.v1",
  version_identity = {:?},
  field_path = ["counts", "issues"],
  order = "version_ascending",
  maximum_result_count = 1,
}}"#,
            metadata.version_identity
        ),
    )
    .expect("selector Nickel");
    let queried = operation_data_command(&fixture)
        .arg("query")
        .arg("--selector")
        .arg(&selector)
        .output()
        .expect("query command");
    assert!(queried.status.success(), "{}", String::from_utf8_lossy(&queried.stderr));
    assert!(String::from_utf8_lossy(&queried.stdout).contains("selected_value"));

    let projection = fixture._root.path().join("projection.json");
    let preview = operation_data_command(&fixture)
        .arg("project")
        .arg(&metadata.version_identity)
        .arg(&projection)
        .output()
        .expect("projection preview");
    assert!(preview.status.success());
    assert!(!projection.exists());
    let created = operation_data_command(&fixture)
        .arg("project")
        .arg(&metadata.version_identity)
        .arg(&projection)
        .arg("--confirm-create")
        .output()
        .expect("projection create");
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));
    assert!(projection.is_file());
    assert!(
        !operation_data_command(&fixture)
            .arg("project")
            .arg(&metadata.version_identity)
            .arg(&projection)
            .arg("--confirm-create")
            .output()
            .expect("duplicate projection")
            .status
            .success()
    );

    let preview_gc = operation_data_command(&fixture)
        .arg("garbage-collect")
        .arg("--now-epoch-seconds")
        .arg("0")
        .output()
        .expect("garbage collection preview");
    assert!(preview_gc.status.success());
    {
        let plane = lattice::operation_data::Plane::open(&fixture.data_dir.join(STORE_FILE_NAME), &payload_root)
            .expect("preview inspection plane");
        assert!(plane.load_metadata(&metadata.version_identity).expect("metadata read").is_some());
    }
    let execute_gc = operation_data_command(&fixture)
        .arg("garbage-collect")
        .arg("--now-epoch-seconds")
        .arg("0")
        .arg("--confirm-delete")
        .output()
        .expect("garbage collection execute");
    assert!(execute_gc.status.success(), "{}", String::from_utf8_lossy(&execute_gc.stderr));
    let plane = lattice::operation_data::Plane::open(&fixture.data_dir.join(STORE_FILE_NAME), &payload_root)
        .expect("execution inspection plane");
    assert!(plane.load_metadata(&metadata.version_identity).expect("metadata read").is_none());
}

#[test]
fn promotion_preview_is_inert_and_confirmed_creation_is_create_only() {
    let fixture = fixture();
    let destination = fixture._root.path().join("promoted.ncl");
    let mut preview = command(&fixture);
    preview.arg("promote");
    plan_args(&mut preview, &fixture);
    preview.arg("--workflow-name").arg("cli-promoted-operation").arg("--destination").arg(&destination);
    let preview_output = preview.output().expect("promotion preview");
    assert!(preview_output.status.success());
    assert!(!destination.exists());
    assert!(!fixture.data_dir.exists());

    let mut create = command(&fixture);
    create.arg("promote");
    plan_args(&mut create, &fixture);
    create
        .arg("--workflow-name")
        .arg("cli-promoted-operation")
        .arg("--destination")
        .arg(&destination)
        .arg("--confirm-create");
    let created = create.output().expect("promotion create");
    assert!(created.status.success(), "{}", String::from_utf8_lossy(&created.stderr));
    assert!(destination.exists());

    let mut duplicate_preview = command(&fixture);
    duplicate_preview.arg("promote");
    plan_args(&mut duplicate_preview, &fixture);
    duplicate_preview
        .arg("--workflow-name")
        .arg("cli-promoted-operation")
        .arg("--destination")
        .arg(&destination);
    assert!(!duplicate_preview.output().expect("existing destination preview").status.success());

    let mut duplicate = command(&fixture);
    duplicate.arg("promote");
    plan_args(&mut duplicate, &fixture);
    duplicate
        .arg("--workflow-name")
        .arg("cli-promoted-operation")
        .arg("--destination")
        .arg(&destination)
        .arg("--confirm-create");
    assert!(!duplicate.output().expect("duplicate create").status.success());
}
