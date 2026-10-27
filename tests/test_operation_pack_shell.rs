use std::collections::BTreeMap;

use lattice::operation_pack as pack;

const VALID_FIXTURE: &str = "tests/fixtures/operation-packs/valid";
const EXPECTED_RECORD_COUNT: usize = 2;
const SHORT_TIMEOUT_SECONDS: u64 = 1;
const TRUNCATED_OUTPUT_BYTES: usize = 64;
const TRUNCATION_PROGRAM_SOURCE: &str = r#"
use std::io::Write as _;

const OUTPUT_BYTES: usize = 1_024;

fn main() {
    // Settle stdin delivery before exiting so this fixture isolates output overflow.
    std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink()).expect("consume analyzer input");
    let output = [b'x'; OUTPUT_BYTES];
    let write_result = std::io::stdout().write_all(&output);
    if write_result.is_err() {
        return;
    }
}
"#;

fn fixture_root() -> std::path::PathBuf {
    std::fs::canonicalize(VALID_FIXTURE).expect("fixture root")
}

fn admitted() -> pack::AdmittedPack {
    let candidate = pack::shell::load(&fixture_root()).expect("candidate");
    let facts = pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::new(),
    };
    let lock = pack::create_lock(&candidate, &facts).expect("lock");
    pack::admit(&candidate, &lock, &facts).expect("admitted pack")
}

#[test]
fn install_is_create_only_and_preserves_exact_identity_after_relocation() {
    let admitted = admitted();
    let root = tempfile::tempdir().expect("installation root");
    let destination = root.path().join("installed-pack");
    let receipt = pack::shell::installation::install_pack_create_only(&admitted, &fixture_root(), &destination)
        .expect("pack installation");
    assert_eq!(receipt.pack_identity, admitted.candidate.pack_identity);
    let installed = pack::shell::load(&destination).expect("installed candidate");
    assert_eq!(installed.pack_identity, admitted.candidate.pack_identity);
    assert_ne!(installed.source_locator, admitted.candidate.source_locator);
    assert!(pack::validate_lock(&installed, &admitted.lock).is_ok());
    assert!(pack::shell::installation::install_pack_create_only(&admitted, &fixture_root(), &destination).is_err());
}

#[test]
fn skills_are_metadata_first_then_explicitly_disclosed_installed_and_removed() {
    let admitted = admitted();
    let metadata = pack::skill::metadata(&admitted.candidate).expect("skill metadata");
    assert_eq!(metadata.len(), 1);
    let gateway = &admitted.candidate.manifest.skills[0];
    // r[verify lattice.operation_packs.skills.valid]
    assert!(pack::skill::validate_routing_cases(gateway));
    assert!(pack::skill::route_query(gateway, "Cairn lifecycle inspection"));
    // r[verify lattice.operation_packs.skills.denied]
    assert!(!pack::skill::route_query(gateway, "Cairn lifecycle inspection as a generic shell task",));
    assert!(!pack::skill::route_query(gateway, "unavailable operation type"));

    let disclosed = pack::shell::disclose_skill(&fixture_root(), &admitted, "lattice-operation-gateway", true)
        .expect("skill disclosure");
    assert_eq!(disclosed.references.len(), 1);
    assert!(!disclosed.grants_authority);
    assert!(pack::shell::disclose_skill(&fixture_root(), &admitted, "missing", false).is_err());

    let destination = tempfile::tempdir().expect("skill root");
    let receipt = pack::shell::installation::install_skill_create_only(
        &admitted,
        &fixture_root(),
        "lattice-operation-gateway",
        "pi",
        destination.path(),
    )
    .expect("skill installation");
    assert!(!receipt.grants_authority);
    assert!(
        pack::shell::installation::install_skill_create_only(
            &admitted,
            &fixture_root(),
            "lattice-operation-gateway",
            "pi",
            destination.path(),
        )
        .is_err()
    );
    assert!(
        pack::shell::installation::remove_skill(
            destination.path(),
            "lattice-operation-gateway",
            &pack::content_identity(b"wrong skill"),
        )
        .is_err()
    );
    pack::shell::installation::remove_skill(destination.path(), "lattice-operation-gateway", &receipt.skill_identity)
        .expect("exact skill removal");
    assert!(!destination.path().join("lattice-operation-gateway").exists());
}

#[test]
fn exact_operation_and_required_analyzer_publish_metadata_only_operation_data() {
    let admitted = admitted();
    let root = tempfile::tempdir().expect("runtime root");
    let repository = root.path().join("repository");
    std::fs::create_dir(&repository).expect("repository root");
    let instances = root.path().join("instances.ncl");
    std::fs::write(&instances, instance_source(&repository)).expect("instances source");
    let data_root = root.path().join("data");
    let app_config = lattice::config::AppConfig {
        data_dir: data_root.clone(),
        ..lattice::config::AppConfig::default()
    };
    let report = pack::shell::execution::run(pack::shell::execution::Request {
        app_config: &app_config,
        pack_root: &fixture_root(),
        admitted: &admitted,
        instances_path: &instances,
        instance_id: "local-repository",
        method: "change_list",
        arguments_json: "{}",
        sequence: 1,
        prohibited_fragments: &[],
        cancellation: None,
    })
    .expect("packed operation execution");
    // r[verify lattice.operation_packs.runtime.executable]
    // r[verify lattice.operation_packs.registry.offline]
    // r[verify lattice.operation_packs.verification.positive]
    assert_eq!(report.status, pack::shell::execution::Status::Completed);
    assert_eq!(report.operation_state, lattice::operation::RunState::Completed);
    assert!(report.operation_data_version.is_some());
    assert_eq!(report.analyzer_outcomes.len(), 1);
    // r[verify lattice.operation_packs.analyzers.valid]
    assert_eq!(report.analyzer_outcomes[0].status, pack::analyzer::Status::Completed);
    assert!(report.output_content_omitted);
    assert!(report.credentials_omitted);
    assert!(report.private_paths_omitted);

    let store_path = lattice::config::store_path(&data_root);
    let plane = lattice::operation_data::Plane::open(&store_path, &data_root).expect("operation data plane");
    let records = plane.list_metadata().expect("operation data metadata");
    assert_eq!(records.len(), EXPECTED_RECORD_COUNT);
    assert!(records.iter().all(|record| record.payload.byte_length_bytes > 0));
    assert!(records.iter().any(|record| record.logical_key.contains("analyzer")));
}

#[test]
fn active_runtime_ceilings_deny_planning_before_store_creation() {
    let admitted = admitted();
    let root = tempfile::tempdir().expect("runtime ceiling root");
    let repository = root.path().join("repository");
    std::fs::create_dir(&repository).expect("repository");
    let instances = root.path().join("instances.ncl");
    std::fs::write(&instances, instance_source(&repository)).expect("instances");
    let data_root = root.path().join("data");
    let process_limits = lattice::process::core::LimitConfig {
        stdout_max_bytes: 1,
        ..lattice::process::core::LimitConfig::default()
    };
    let app_config = lattice::config::AppConfig {
        data_dir: data_root.clone(),
        process_limits,
        ..lattice::config::AppConfig::default()
    };
    let result = pack::shell::execution::run(pack::shell::execution::Request {
        app_config: &app_config,
        pack_root: &fixture_root(),
        admitted: &admitted,
        instances_path: &instances,
        instance_id: "local-repository",
        method: "change_list",
        arguments_json: "{}",
        sequence: 1,
        prohibited_fragments: &[],
        cancellation: None,
    });
    // r[verify lattice.operation_packs.verification.negative]
    assert!(result.is_err());
    assert!(!data_root.exists());
}

#[test]
fn runtime_bytes_are_rechecked_after_admission_and_before_dispatch() {
    let admitted = admitted();
    let root = tempfile::tempdir().expect("runtime recheck root");
    let pack_root = root.path().join("pack");
    copy_pack(&fixture_root(), &pack_root);
    std::fs::write(pack_root.join("analyzer-fixture"), "#!/bin/sh\nexit 0\n").expect("tampered runtime");
    assert!(pack::shell::content_path(&pack_root, &admitted.candidate, "runtime-analyzer").is_err());
}

#[test]
fn analyzer_timeout_and_truncation_are_explicit_and_block_required_parent_success() {
    let timeout_script = "#!/bin/sh\nwhile :; do\n  :\ndone\n";
    let timed_out =
        execute_with_analyzer(AnalyzerFixture::Script(timeout_script), SHORT_TIMEOUT_SECONDS, TRUNCATED_OUTPUT_BYTES);
    // r[verify lattice.operation_packs.analyzers.failure]
    assert_eq!(timed_out.status, pack::shell::execution::Status::Failed);
    assert_eq!(timed_out.analyzer_outcomes[0].status, pack::analyzer::Status::TimedOut);

    let truncated = execute_with_analyzer(
        AnalyzerFixture::Rust(TRUNCATION_PROGRAM_SOURCE),
        SHORT_TIMEOUT_SECONDS,
        TRUNCATED_OUTPUT_BYTES,
    );
    assert_eq!(truncated.status, pack::shell::execution::Status::Failed);
    assert_eq!(
        truncated.analyzer_outcomes[0].status,
        pack::analyzer::Status::Truncated,
        "unexpected analyzer outcome: {:?}",
        truncated.analyzer_outcomes[0]
    );
}

#[test]
fn declared_no_effect_probe_runs_without_credentials_mutation_or_network_authority() {
    let mut candidate = pack::shell::load(&fixture_root()).expect("candidate");
    candidate.manifest.probe = Some(pack::ProbeManifest {
        runtime_id: "analyzer-runtime".to_string(),
        launch_label: "analyze".to_string(),
        effectful: false,
        uses_production_credentials: false,
    });
    candidate.manifest = pack::validate_manifest(candidate.manifest).expect("probe manifest");
    candidate.manifest_identity = pack::manifest_identity(&candidate.manifest).expect("manifest identity");
    candidate.pack_identity = pack::pack_identity(&candidate.manifest).expect("pack identity");
    let facts = pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::new(),
    };
    let lock = pack::create_lock(&candidate, &facts).expect("probe lock");
    let admitted = pack::admit(&candidate, &lock, &facts).expect("probe admission");
    let outcome =
        pack::shell::probe::run(&fixture_root(), &admitted, &lattice::process::core::RuntimeConfig::default(), None)
            .expect("no-effect probe");
    assert_eq!(outcome.status, pack::shell::probe::Status::Succeeded);
    assert!(!outcome.production_credentials_used);
    assert!(!outcome.mutation_authority_granted);
    assert!(!outcome.network_authority_granted);
    assert!(outcome.output_content_omitted);

    let mut unsafe_probe = candidate.manifest;
    unsafe_probe.probe.as_mut().expect("probe").effectful = true;
    assert!(pack::validate_manifest(unsafe_probe).is_err());
}

#[test]
fn doctor_reports_distinct_cache_registry_platform_and_admission_states() {
    let admitted = admitted();
    let healthy = pack::doctor::evaluate(
        Ok(&admitted.candidate),
        Some(&admitted.lock),
        Some(&pack::AdmissionFacts {
            trusted_pack_identity: admitted.candidate.pack_identity.clone(),
            platform: pack::shell::platform_label(),
            dependency_identities: BTreeMap::new(),
            allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
            allowed_capabilities: admitted.candidate.manifest.admission.allowed_capabilities.clone(),
            evidence: BTreeMap::new(),
        }),
        pack::doctor::ShellFacts {
            cache_present: true,
            registry_configured: true,
            registry_available: true,
        },
    );
    // r[verify lattice.operation_packs.doctor.valid]
    assert!(healthy.healthy);
    assert!(!healthy.ordinary_effect_invoked);
    assert!(!healthy.credentials_resolved);

    let invalid = pack::Error::Invalid("fixture");
    let unhealthy = pack::doctor::evaluate(Err(&invalid), None, None, pack::doctor::ShellFacts {
        cache_present: false,
        registry_configured: true,
        registry_available: false,
    });
    // r[verify lattice.operation_packs.doctor.invalid]
    assert!(!unhealthy.healthy);
    assert!(unhealthy.findings.iter().any(|finding| finding.id == "manifest"));
    assert!(unhealthy.findings.iter().any(|finding| finding.id == "cache"));
    assert!(unhealthy.findings.iter().any(|finding| finding.id == "registry"));
}

enum AnalyzerFixture<'a> {
    Script(&'a str),
    Rust(&'a str),
}

fn execute_with_analyzer(
    analyzer_fixture: AnalyzerFixture<'_>,
    timeout_seconds: u64,
    output_bytes: usize,
) -> pack::shell::execution::Report {
    let root = tempfile::tempdir().expect("custom analyzer root");
    let pack_root = root.path().join("pack");
    copy_pack(&fixture_root(), &pack_root);
    let analyzer_path = pack_root.join("analyzer-fixture");
    match analyzer_fixture {
        AnalyzerFixture::Script(source) => {
            std::fs::write(&analyzer_path, source).expect("custom analyzer script");
        }
        AnalyzerFixture::Rust(source) => {
            let source_path = root.path().join("analyzer-fixture.rs");
            std::fs::write(&source_path, source).expect("custom analyzer Rust source");
            let status = std::process::Command::new("rustc")
                .arg("--edition=2024")
                .arg(&source_path)
                .arg("-o")
                .arg(&analyzer_path)
                .status()
                .expect("compile custom analyzer");
            assert!(status.success());
        }
    }
    let mut candidate = pack::shell::load(&fixture_root()).expect("candidate");
    let bytes = std::fs::read(&analyzer_path).expect("custom analyzer bytes");
    let content = candidate
        .manifest
        .contents
        .iter_mut()
        .find(|content| content.id == "runtime-analyzer")
        .expect("analyzer content");
    content.content_identity = pack::content_identity(&bytes);
    content.byte_length = u64::try_from(bytes.len()).expect("analyzer length");
    let measured = candidate
        .measured_contents
        .iter_mut()
        .find(|content| content.id == "runtime-analyzer")
        .expect("measured analyzer");
    measured.content_identity = content.content_identity.clone();
    measured.byte_length = content.byte_length;
    let runtime = candidate
        .manifest
        .runtimes
        .iter_mut()
        .find(|runtime| runtime.id == "analyzer-runtime")
        .expect("analyzer runtime");
    runtime.timeout_seconds = timeout_seconds;
    runtime.output_bytes = output_bytes;
    let analyzer = candidate.manifest.analyzers.first_mut().expect("analyzer manifest");
    analyzer.timeout_seconds = timeout_seconds;
    analyzer.output_bytes = output_bytes;
    analyzer.maximum_markdown_bytes = output_bytes;
    candidate.manifest = pack::validate_manifest(candidate.manifest).expect("custom analyzer manifest");
    candidate.manifest_identity = pack::manifest_identity(&candidate.manifest).expect("manifest identity");
    candidate.pack_identity = pack::pack_identity(&candidate.manifest).expect("pack identity");
    let facts = pack::AdmissionFacts {
        trusted_pack_identity: candidate.pack_identity.clone(),
        platform: pack::shell::platform_label(),
        dependency_identities: BTreeMap::new(),
        allowed_licenses: vec!["AGPL-3.0-or-later".to_string()],
        allowed_capabilities: candidate.manifest.admission.allowed_capabilities.clone(),
        evidence: BTreeMap::new(),
    };
    let lock = pack::create_lock(&candidate, &facts).expect("custom analyzer lock");
    let admitted = pack::admit(&candidate, &lock, &facts).expect("custom analyzer admission");
    let repository = root.path().join("repository");
    std::fs::create_dir(&repository).expect("repository");
    let instances = root.path().join("instances.ncl");
    std::fs::write(&instances, instance_source(&repository)).expect("instances");
    let app_config = lattice::config::AppConfig {
        data_dir: root.path().join("data"),
        ..lattice::config::AppConfig::default()
    };
    pack::shell::execution::run(pack::shell::execution::Request {
        app_config: &app_config,
        pack_root: &pack_root,
        admitted: &admitted,
        instances_path: &instances,
        instance_id: "local-repository",
        method: "change_list",
        arguments_json: "{}",
        sequence: 1,
        prohibited_fragments: &[],
        cancellation: None,
    })
    .expect("custom analyzer execution")
}

fn copy_pack(source: &std::path::Path, destination: &std::path::Path) {
    std::fs::create_dir(destination).expect("create pack copy");
    for entry in std::fs::read_dir(source).expect("source pack") {
        let entry = entry.expect("pack entry");
        let copied = destination.join(entry.file_name());
        std::fs::copy(entry.path(), &copied).expect("copy pack entry");
        std::fs::set_permissions(&copied, entry.metadata().expect("entry metadata").permissions())
            .expect("copy entry permissions");
    }
}

fn instance_source(repository: &std::path::Path) -> String {
    format!(
        r#"{{
  schema = "lattice.operation-instances.v1",
  instances = [{{
    id = "local-repository",
    display_alias = "Local repository",
    type_id = "cairn.repository",
    type_revision = "b3:35bb448c27e5def0f6204d8fc3f14fa40275b73a735b1971a1c5954afa30bd2d",
    target = {{ repository_root = "{}" }},
    tags = {{ environment = "test" }},
    method_defaults = {{}},
    credentials = {{}},
  }}],
}}
"#,
        repository.display()
    )
}
