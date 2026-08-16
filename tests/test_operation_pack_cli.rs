use std::process::Command;

const VALID_FIXTURE: &str = "tests/fixtures/operation-packs/valid";
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
const MAXIMUM_PARALLELISM: usize = 1;
const VALID_REGISTRY: &str = "tests/fixtures/operation-packs/registry-valid.ncl";

struct Fixture {
    _root: tempfile::TempDir,
    config: std::path::PathBuf,
    pack: std::path::PathBuf,
    instances: std::path::PathBuf,
    data: std::path::PathBuf,
}

fn fixture() -> Fixture {
    let root = tempfile::tempdir().expect("CLI root");
    let data = root.path().join("data");
    let config = root.path().join("config.ncl");
    std::fs::write(
        &config,
        format!(
            "{{ data_dir = {:?}, default_timeout_secs = {DEFAULT_TIMEOUT_SECONDS}, shell = \"sh\", max_parallelism = {MAXIMUM_PARALLELISM} }}",
            data.to_string_lossy()
        ),
    )
    .expect("config source");
    let repository = root.path().join("repository");
    std::fs::create_dir(&repository).expect("repository root");
    let instances = root.path().join("instances.ncl");
    std::fs::write(
        &instances,
        format!(
            r#"{{
  schema = "lattice.operation-instances.v1",
  instances = [{{
    id = "cli-repository",
    display_alias = "CLI repository",
    type_id = "cairn.repository",
    type_revision = "b3:04f4f2b5e880435887fd6e8d124b80b9e94c9037ff310aacca88f065d6bf6cc9",
    target = {{ repository_root = {:?} }},
    tags = {{}},
    method_defaults = {{}},
    credentials = {{}},
  }}],
}}
"#,
            repository.to_string_lossy()
        ),
    )
    .expect("instances source");
    Fixture {
        _root: root,
        config,
        pack: std::fs::canonicalize(VALID_FIXTURE).expect("pack root"),
        instances,
        data,
    }
}

fn pack_candidate(fixture: &Fixture) -> lattice::operation_pack::Candidate {
    lattice::operation_pack::shell::load(&fixture.pack).expect("pack candidate")
}

fn command(fixture: &Fixture) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    command.arg("--config").arg(&fixture.config).arg("pack");
    command
}

fn trust(command: &mut Command) {
    command.arg("--confirm-trust-local").arg("--allow-license").arg("MIT");
}

#[test]
fn inspect_and_preview_are_inert_and_omit_full_skill_body() {
    let fixture = fixture();
    let output = command(&fixture).arg("inspect").arg("--pack").arg(&fixture.pack).output().expect("inspect command");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("inspect output");
    assert!(stdout.contains("pack_identity"));
    assert!(!stdout.contains("Use this passive skill only"));
    assert!(!fixture.data.exists());

    let lock = fixture._root.path().join("preview.lock.json");
    let mut preview = command(&fixture);
    preview.arg("lock").arg("--pack").arg(&fixture.pack);
    trust(&mut preview);
    let output = preview.arg("--destination").arg(&lock).output().expect("lock preview");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!lock.exists());
    assert!(!fixture.data.exists());
}

#[test]
fn run_requires_confirmation_then_executes_the_exact_locked_operation() {
    let fixture = fixture();
    let lock = fixture._root.path().join("pack.lock.json");
    let mut create = command(&fixture);
    create.arg("lock").arg("--pack").arg(&fixture.pack);
    trust(&mut create);
    let output = create.arg("--destination").arg(&lock).arg("--confirm-create").output().expect("lock create");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    let mut preview = command(&fixture);
    preview.arg("run").arg("--pack").arg(&fixture.pack);
    trust(&mut preview);
    let output = preview
        .arg("--lock")
        .arg(&lock)
        .arg("--instances")
        .arg(&fixture.instances)
        .arg("--instance")
        .arg("cli-repository")
        .arg("--method")
        .arg("change_list")
        .output()
        .expect("run preview");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!fixture.data.exists());

    let mut execute = command(&fixture);
    execute.arg("run").arg("--pack").arg(&fixture.pack);
    trust(&mut execute);
    let output = execute
        .arg("--lock")
        .arg(&lock)
        .arg("--instances")
        .arg(&fixture.instances)
        .arg("--instance")
        .arg("cli-repository")
        .arg("--method")
        .arg("change_list")
        .arg("--confirm-read-only")
        .output()
        .expect("run command");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8(output.stdout).expect("run output");
    assert!(output.status.success(), "stderr={stderr} stdout={stdout}");
    assert!(stdout.contains("\"status\": \"completed\""));
    assert!(stdout.contains("\"output_content_omitted\": true"));
    assert!(!stdout.contains("\"changes\": []"));
    assert!(fixture.data.join("lattice.redb").exists());
}

#[test]
fn registry_info_and_restore_plan_do_not_resolve_content_or_install() {
    let fixture = fixture();
    let registry = std::fs::canonicalize(VALID_REGISTRY).expect("registry");
    let candidate = pack_candidate(&fixture);
    let output = command(&fixture)
        .arg("registry-info")
        .arg("--catalog")
        .arg(&registry)
        .arg("--pack-identity")
        .arg(&candidate.pack_identity)
        .output()
        .expect("registry info");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"content_resolved\": false"));

    let destination = fixture._root.path().join("restored-pack");
    let output = command(&fixture)
        .arg("registry-restore")
        .arg("--catalog")
        .arg(&registry)
        .arg("--pack-identity")
        .arg(&candidate.pack_identity)
        .arg("--destination")
        .arg(&destination)
        .output()
        .expect("registry restore plan");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"destination_omitted\": true"));
    assert!(stdout.contains("\"network_contacted\": false"));
    assert!(!stdout.contains(destination.to_string_lossy().as_ref()));
    assert!(!destination.exists());
    assert!(!fixture.data.exists());
}

#[test]
fn skill_disclosure_and_installation_are_separate_explicit_actions() {
    let fixture = fixture();
    let lock = fixture._root.path().join("pack.lock.json");
    let mut create = command(&fixture);
    create.arg("lock").arg("--pack").arg(&fixture.pack);
    trust(&mut create);
    assert!(
        create
            .arg("--destination")
            .arg(&lock)
            .arg("--confirm-create")
            .status()
            .expect("lock create")
            .success()
    );

    let mut show = command(&fixture);
    show.arg("skill-show").arg("--pack").arg(&fixture.pack);
    trust(&mut show);
    let output = show
        .arg("--lock")
        .arg(&lock)
        .arg("--name")
        .arg("lattice-operation-gateway")
        .output()
        .expect("skill show");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Use this passive skill only"));

    let skills = fixture._root.path().join("skills");
    std::fs::create_dir(&skills).expect("skills root");
    let mut install = command(&fixture);
    install.arg("skill-install").arg("--pack").arg(&fixture.pack);
    trust(&mut install);
    let output = install
        .arg("--lock")
        .arg(&lock)
        .arg("--name")
        .arg("lattice-operation-gateway")
        .arg("--host")
        .arg("pi")
        .arg("--destination-root")
        .arg(&skills)
        .output()
        .expect("skill install preview");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!skills.join("lattice-operation-gateway").exists());
}
