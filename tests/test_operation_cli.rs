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
