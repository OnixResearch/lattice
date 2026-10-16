#[path = "support/cairn.rs"]
mod cairn;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

use lattice_repository_validation_fixture as vendor_fixture;
use serde_json::Value;

const README_SOURCE: &str = include_str!("../README.md");
const FLAKE_SOURCE: &str = include_str!("../flake.nix");
const FLAKE_LOCK_SOURCE: &str = include_str!("../flake.lock");
const CARGO_SOURCE: &str = include_str!("../Cargo.toml");
const CARGO_LOCK_SOURCE: &str = include_str!("../Cargo.lock");
const GITIGNORE_SOURCE: &str = include_str!("../.gitignore");
const JSON_FIXTURE_SOURCE: &str = include_str!("fixtures/n8n/shell.json");
const VENDOR_MANIFEST_SOURCE: &str = include_str!("fixtures/repository-validation/vendor-dependency/Cargo.toml");
const VENDOR_LIB_SOURCE: &str = include_str!("fixtures/repository-validation/vendor-dependency/src/lib.rs");
const VENDOR_RESULT_SOURCE: &str =
    include_str!("fixtures/repository-validation/vendor-dependency/src/coverage/result.rs");
const BOUNDED_EXEC_SOURCE_ADMISSION: &str = include_str!("fixtures/radicle/bounded-exec-source-admission-v1.json");

const CHECK_COMMAND: &str = "nix flake check -L --option secret-key-files ''";
const TEST_FILTER: &str = "pkgs.lib.hasPrefix \"${sourceRoot}/tests/\" (toString path)";
const PATH_DEPENDENCY: &str =
    "lattice-repository-validation-fixture = { path = \"tests/fixtures/repository-validation/vendor-dependency\" }";
const FORMAT_SCOPE: &str = "cargoExtraArgs = \"--package lattice\";";
const CLIPPY_SCOPE: &str = "cargoClippyExtraArgs = \"--package lattice --all-targets --no-deps -- -D warnings\";";
const ROOT_COVERAGE_IGNORE: &str = "/coverage/";
const ROOT_RESULT_IGNORE: &str = "/result*";
const UNANCHORED_COVERAGE_IGNORE: &str = "coverage/";
const UNANCHORED_RESULT_IGNORE: &str = "result*";
const NESTED_VENDOR_PATH: &str = "tests/fixtures/repository-validation/vendor-dependency/src/coverage/result.rs";
const VENDOR_SENTINEL: &str = "nested-output-name-admitted";
const VENDOR_SENTINEL_DECLARATION: &str = "pub const SENTINEL:&str=";
const GIT_PROGRAM: &str = "git";
const RUSTFMT_PROGRAM: &str = "rustfmt";
const GIT_NOT_IGNORED_STATUS: i32 = 1;
const BOUNDED_EXEC_REVISION: &str = "29dac88ecded94457572db3fdfaaaab95fa91525";
const BOUNDED_EXEC_RID: &str = "rad:z2CpqLFpdP36fZXYUK5ZNWxMibpCo";
const BOUNDED_EXEC_HTTPS_URL: &str = "https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git";
const BOUNDED_EXEC_SOURCE_ARCHIVE_BLAKE3: &str = "4fbbf8f0749262469f00748e04c775180488dba800303f139172656d25931927";
const BOUNDED_EXEC_PUBLICATION_BLAKE3: &str = "6eab0250b4bfefb6f6e91ec75d9786a76ded20bc052e2e624b44c6bc5b1cc9f7";
const BOUNDED_EXEC_NAR_HASH: &str = "sha256-BVmqyUYyoNpY6LfOABxwPO3DY88ZtXeKNg3TPoGCcL0=";
const BOUNDED_EXEC_SEED_COUNT: u64 = 2;
const BOUNDED_EXEC_CARGO_PIN: &str = "bounded-exec = { git = \"https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git\", rev = \"29dac88ecded94457572db3fdfaaaab95fa91525\" }";
const BOUNDED_EXEC_FLAKE_PIN: &str = "url = \"git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525\";";
const BOUNDED_EXEC_CARGO_LOCK_SOURCE: &str = "git+https://git.onix.computer/z2CpqLFpdP36fZXYUK5ZNWxMibpCo.git?rev=29dac88ecded94457572db3fdfaaaab95fa91525#29dac88ecded94457572db3fdfaaaab95fa91525";
const BOUNDED_EXEC_NIX_VENDOR_SOURCE: &str = "src = bounded-exec;";
const BOUNDED_EXEC_GITHUB_SOURCE: &str = "github.com/OnixResearch/bounded-exec";

const ISSUE_SIGNING_COMMAND: &str = "signing.command_scope";
const ISSUE_SIGNING_FLAKE_OVERRIDE: &str = "signing.flake_override";
const ISSUE_TEST_FILTER: &str = "source.tests_filter";
const ISSUE_PATH_DEPENDENCY: &str = "source.path_dependency";
const ISSUE_NESTED_RESULT: &str = "source.nested_result";
const ISSUE_JSON_FIXTURE: &str = "source.json_fixture";
const ISSUE_IGNORE_SCOPE: &str = "source.ignore_scope";
const ISSUE_FORMAT_SCOPE: &str = "format.package_scope";
const ISSUE_FORMAT_PRECONDITION: &str = "format.vendor_precondition";
const ISSUE_CLIPPY_SCOPE: &str = "clippy.package_scope";
const ISSUE_CLIPPY_PRECONDITION: &str = "clippy.vendor_precondition";
const ISSUE_BOUNDED_EXEC_SOURCE: &str = "source.bounded_exec_identity";
const ISSUE_BOUNDED_EXEC_EVIDENCE: &str = "source.bounded_exec_producer_evidence";
const ISSUE_BOUNDED_EXEC_CRATES: &str = "source.bounded_exec_crate_set";

#[derive(Debug, Clone, Copy)]
struct ContractInputs<'a> {
    readme: &'a str,
    flake: &'a str,
    flake_lock: &'a str,
    cargo: &'a str,
    cargo_lock: &'a str,
    gitignore: &'a str,
    json_fixture: &'a str,
    vendor_manifest: &'a str,
    vendor_lib: &'a str,
    vendor_result: &'a str,
    bounded_exec_source_admission: &'a str,
    bounded_exec_crates_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IgnoreAdmission {
    Admitted,
    Ignored,
    ToolFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatAdmission {
    Conforming,
    Nonconforming,
    ToolFailure,
}

fn current_inputs() -> ContractInputs<'static> {
    ContractInputs {
        readme: README_SOURCE,
        flake: FLAKE_SOURCE,
        flake_lock: FLAKE_LOCK_SOURCE,
        cargo: CARGO_SOURCE,
        cargo_lock: CARGO_LOCK_SOURCE,
        gitignore: GITIGNORE_SOURCE,
        json_fixture: JSON_FIXTURE_SOURCE,
        vendor_manifest: VENDOR_MANIFEST_SOURCE,
        vendor_lib: VENDOR_LIB_SOURCE,
        vendor_result: VENDOR_RESULT_SOURCE,
        bounded_exec_source_admission: BOUNDED_EXEC_SOURCE_ADMISSION,
        bounded_exec_crates_present: true,
    }
}

fn has_exact_line(source: &str, expected: &str) -> bool {
    source.lines().any(|line| line.trim() == expected)
}

fn validates_json(source: &str) -> bool {
    serde_json::from_str::<Value>(source).is_ok()
}

fn bounded_exec_lock_matches(source: &str) -> bool {
    let Ok(lock) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    let Some(node) = lock.pointer("/nodes/bounded-exec") else {
        return false;
    };
    node.pointer("/locked/rev").and_then(Value::as_str) == Some(BOUNDED_EXEC_REVISION)
        && node.pointer("/locked/url").and_then(Value::as_str) == Some(BOUNDED_EXEC_HTTPS_URL)
        && node.pointer("/locked/narHash").and_then(Value::as_str) == Some(BOUNDED_EXEC_NAR_HASH)
        && node.pointer("/locked/type").and_then(Value::as_str) == Some("git")
        && node.pointer("/original/rev").and_then(Value::as_str) == Some(BOUNDED_EXEC_REVISION)
        && node.pointer("/original/url").and_then(Value::as_str) == Some(BOUNDED_EXEC_HTTPS_URL)
        && node.pointer("/original/type").and_then(Value::as_str) == Some("git")
}

fn bounded_exec_source_admission_matches(source: &str) -> bool {
    let Ok(admission) = serde_json::from_str::<Value>(source) else {
        return false;
    };
    admission.pointer("/schema").and_then(Value::as_str) == Some("lattice.radicle-source-admission.v1")
        && admission.pointer("/status").and_then(Value::as_str) == Some("accepted")
        && admission.pointer("/repository/rid").and_then(Value::as_str) == Some(BOUNDED_EXEC_RID)
        && admission.pointer("/repository/reviewed_commit").and_then(Value::as_str) == Some(BOUNDED_EXEC_REVISION)
        && admission.pointer("/repository/source_archive_blake3").and_then(Value::as_str)
            == Some(BOUNDED_EXEC_SOURCE_ARCHIVE_BLAKE3)
        && admission.pointer("/publication/receipt_blake3").and_then(Value::as_str)
            == Some(BOUNDED_EXEC_PUBLICATION_BLAKE3)
        && admission.pointer("/onixos/publication_acceptance_revision").and_then(Value::as_str) == Some("b5ba554")
        && admission.pointer("/sources/cargo_git").and_then(Value::as_str) == Some(BOUNDED_EXEC_HTTPS_URL)
        && admission.pointer("/availability/seed_count").and_then(Value::as_u64) == Some(BOUNDED_EXEC_SEED_COUNT)
        && admission.pointer("/availability/primary_only").and_then(Value::as_str) == Some("verified")
        && admission.pointer("/availability/replica_only").and_then(Value::as_str) == Some("verified")
        && admission.pointer("/availability/github_fallback_configured").and_then(Value::as_bool) == Some(false)
        && !source.contains(BOUNDED_EXEC_GITHUB_SOURCE)
}

fn bounded_exec_source_matches(inputs: ContractInputs<'_>) -> bool {
    inputs.cargo.contains(BOUNDED_EXEC_CARGO_PIN)
        && !inputs.cargo.contains("bounded-exec = { path =")
        && !inputs.cargo.contains(BOUNDED_EXEC_GITHUB_SOURCE)
        && inputs.flake.contains(BOUNDED_EXEC_FLAKE_PIN)
        && inputs.flake.contains(BOUNDED_EXEC_NIX_VENDOR_SOURCE)
        && !inputs.flake.contains("../bounded-exec")
        && !inputs.flake.contains(BOUNDED_EXEC_GITHUB_SOURCE)
        && inputs.cargo_lock.contains(BOUNDED_EXEC_CARGO_LOCK_SOURCE)
        && !inputs.cargo_lock.contains(BOUNDED_EXEC_GITHUB_SOURCE)
        && bounded_exec_lock_matches(inputs.flake_lock)
        && !inputs.flake_lock.contains(BOUNDED_EXEC_GITHUB_SOURCE)
}

fn validate_contract(inputs: ContractInputs<'_>) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();

    if !inputs.readme.contains(CHECK_COMMAND) {
        issues.insert(ISSUE_SIGNING_COMMAND);
    }
    if inputs.flake.contains("nixConfig") || inputs.flake.contains("secret-key-files") {
        issues.insert(ISSUE_SIGNING_FLAKE_OVERRIDE);
    }
    if !inputs.flake.contains(TEST_FILTER) {
        issues.insert(ISSUE_TEST_FILTER);
    }
    if !inputs.cargo.contains(PATH_DEPENDENCY)
        || !inputs.vendor_manifest.contains("name = \"lattice-repository-validation-fixture\"")
    {
        issues.insert(ISSUE_PATH_DEPENDENCY);
    }
    if !inputs.vendor_result.contains(VENDOR_SENTINEL) || !inputs.vendor_result.contains(VENDOR_SENTINEL_DECLARATION) {
        issues.insert(ISSUE_NESTED_RESULT);
    }
    if !validates_json(inputs.json_fixture) {
        issues.insert(ISSUE_JSON_FIXTURE);
    }
    if !has_exact_line(inputs.gitignore, ROOT_COVERAGE_IGNORE)
        || !has_exact_line(inputs.gitignore, ROOT_RESULT_IGNORE)
        || has_exact_line(inputs.gitignore, UNANCHORED_COVERAGE_IGNORE)
        || has_exact_line(inputs.gitignore, UNANCHORED_RESULT_IGNORE)
    {
        issues.insert(ISSUE_IGNORE_SCOPE);
    }
    if !inputs.flake.contains(FORMAT_SCOPE) {
        issues.insert(ISSUE_FORMAT_SCOPE);
    }
    if !inputs.vendor_result.contains(VENDOR_SENTINEL_DECLARATION) {
        issues.insert(ISSUE_FORMAT_PRECONDITION);
    }
    if !inputs.flake.contains(CLIPPY_SCOPE) {
        issues.insert(ISSUE_CLIPPY_SCOPE);
    }
    if !bounded_exec_source_matches(inputs) {
        issues.insert(ISSUE_BOUNDED_EXEC_SOURCE);
    }
    if !bounded_exec_source_admission_matches(inputs.bounded_exec_source_admission) {
        issues.insert(ISSUE_BOUNDED_EXEC_EVIDENCE);
    }
    if !inputs.bounded_exec_crates_present {
        issues.insert(ISSUE_BOUNDED_EXEC_CRATES);
    }
    if !inputs.vendor_lib.contains("#![deny(clippy::ptr_arg)]") || !inputs.vendor_lib.contains("&Vec<String>") {
        issues.insert(ISSUE_CLIPPY_PRECONDITION);
    }

    issues
}

fn classify_git_status(status: ExitStatus) -> IgnoreAdmission {
    if status.success() {
        return IgnoreAdmission::Ignored;
    }
    if status.code() == Some(GIT_NOT_IGNORED_STATUS) {
        return IgnoreAdmission::Admitted;
    }
    IgnoreAdmission::ToolFailure
}

fn git_ignore_admission(ignore_source: &str, relative_path: &str) -> IgnoreAdmission {
    let directory = tempfile::tempdir().expect("temporary Git fixture directory");
    fs::write(directory.path().join(".gitignore"), ignore_source).expect("write Git ignore fixture");
    let candidate = directory.path().join(relative_path);
    fs::create_dir_all(candidate.parent().expect("candidate parent")).expect("create candidate parent");
    fs::write(&candidate, VENDOR_RESULT_SOURCE).expect("write candidate source");

    let init = Command::new(GIT_PROGRAM)
        .args(["init", "--quiet"])
        .current_dir(directory.path())
        .status()
        .expect("Git must be available to repository checks");
    assert!(init.success(), "Git fixture initialization failed");

    let status = Command::new(GIT_PROGRAM)
        .args(["check-ignore", "--quiet", "--no-index", relative_path])
        .current_dir(directory.path())
        .status()
        .expect("Git must classify the fixture path");
    classify_git_status(status)
}

fn classify_rustfmt_status(status: ExitStatus) -> FormatAdmission {
    if status.success() {
        return FormatAdmission::Conforming;
    }
    if status.code().is_some() {
        return FormatAdmission::Nonconforming;
    }
    FormatAdmission::ToolFailure
}

fn rustfmt_admission(source: &str) -> FormatAdmission {
    let directory = tempfile::tempdir().expect("temporary rustfmt fixture directory");
    let source_path = directory.path().join("fixture.rs");
    fs::write(&source_path, source).expect("write rustfmt fixture");
    let status = Command::new(RUSTFMT_PROGRAM)
        .args(["--check", "--edition", "2024"])
        .arg(Path::new(&source_path))
        .status()
        .expect("rustfmt must be available to repository checks");
    classify_rustfmt_status(status)
}

// r[verify n8nclone.repository_validation.flake_check]
// r[verify n8nclone.repository_validation.flake_check.scenario.missing-signing-key]
// r[verify n8nclone.repository_validation.flake_check.scenario.normal-signing]
#[test]
fn check_scoped_signing_override_leaves_flake_signing_external() {
    let issues = validate_contract(current_inputs());
    assert!(!issues.contains(ISSUE_SIGNING_COMMAND));
    assert!(!issues.contains(ISSUE_SIGNING_FLAKE_OVERRIDE));
    assert!(README_SOURCE.contains(CHECK_COMMAND));
    assert!(!FLAKE_SOURCE.contains("nixConfig"));
    assert!(!FLAKE_SOURCE.contains("secret-key-files"));
}

// r[verify n8nclone.repository_validation.vendor_snapshot]
// r[verify n8nclone.repository_validation.vendor_snapshot.scenario.nested-output-name]
// r[verify n8nclone.repository_validation.test_source]
// r[verify n8nclone.repository_validation.test_source.scenario.json-fixture]
#[test]
fn filtered_nix_source_compiles_nested_vendor_and_json_fixtures() {
    let issues = validate_contract(current_inputs());
    let parsed: Value = serde_json::from_str(JSON_FIXTURE_SOURCE).expect("tracked JSON fixture must parse");
    let values = Vec::new();

    assert!(!issues.contains(ISSUE_TEST_FILTER));
    assert!(!issues.contains(ISSUE_PATH_DEPENDENCY));
    assert!(!issues.contains(ISSUE_NESTED_RESULT));
    assert!(!issues.contains(ISSUE_JSON_FIXTURE));
    assert!(!issues.contains(ISSUE_IGNORE_SCOPE));
    assert_eq!(vendor_fixture::coverage::result::SENTINEL, VENDOR_SENTINEL);
    assert_eq!(vendor_fixture::lint_boundary(&values), values.len());
    assert!(parsed.get("nodes").is_some());
    assert_eq!(git_ignore_admission(GITIGNORE_SOURCE, NESTED_VENDOR_PATH), IgnoreAdmission::Admitted);
}

// r[verify n8nclone.repository_validation.format_scope]
// r[verify n8nclone.repository_validation.format_scope.scenario.vendor-format]
#[test]
fn format_check_is_scoped_away_from_nonconforming_vendor_fixture() {
    let issues = validate_contract(current_inputs());
    assert!(!issues.contains(ISSUE_FORMAT_SCOPE));
    assert!(!issues.contains(ISSUE_FORMAT_PRECONDITION));
    assert_eq!(rustfmt_admission(VENDOR_RESULT_SOURCE), FormatAdmission::Nonconforming);
}

// r[verify n8nclone.repository_validation.clippy_scope]
// r[verify n8nclone.repository_validation.clippy_scope.scenario.vendor-lints]
#[test]
fn clippy_denies_owned_warnings_without_admitting_vendor_lints() {
    let issues = validate_contract(current_inputs());
    assert!(!issues.contains(ISSUE_CLIPPY_SCOPE));
    assert!(!issues.contains(ISSUE_CLIPPY_PRECONDITION));
    assert!(FLAKE_SOURCE.contains(CLIPPY_SCOPE));
    assert!(VENDOR_LIB_SOURCE.contains("#![deny(clippy::ptr_arg)]"));
    assert!(VENDOR_LIB_SOURCE.contains("&Vec<String>"));
}

// r[verify lattice.repository_validation.bounded_exec_source]
// r[verify lattice.repository_validation.bounded_exec_source.scenario.published]
// r[verify lattice.repository_validation.bounded_exec_source.scenario.invalid]
#[test]
fn cargo_and_nix_resolve_the_same_immutable_bounded_exec_source() {
    let inputs = current_inputs();
    let issues = validate_contract(inputs);
    // r[verify lattice.repository_validation.radicle_bounded_exec_locks.scenario.aligned]
    assert!(bounded_exec_source_matches(inputs));
    assert!(bounded_exec_source_admission_matches(inputs.bounded_exec_source_admission));
    assert!(!issues.contains(ISSUE_BOUNDED_EXEC_SOURCE));
    assert!(!issues.contains(ISSUE_BOUNDED_EXEC_EVIDENCE));

    let mut path_substitution = inputs;
    path_substitution.cargo = "bounded-exec = { path = \"../bounded-exec\" }";
    path_substitution.flake = "bounded-exec = ../bounded-exec;";
    assert!(!bounded_exec_source_matches(path_substitution));

    let mut endpoint_mismatch = inputs;
    endpoint_mismatch.cargo = "bounded-exec = { git = \"https://example.invalid/bounded-exec.git\", rev = \"29dac88ecded94457572db3fdfaaaab95fa91525\" }";
    assert!(!bounded_exec_source_matches(endpoint_mismatch));

    let mut stale_github = inputs;
    stale_github.cargo = "bounded-exec = { git = \"ssh://git@github.com/OnixResearch/bounded-exec.git\", rev = \"29dac88ecded94457572db3fdfaaaab95fa91525\" }";
    // r[verify lattice.repository_validation.radicle_bounded_exec_source.scenario.github-rejected]
    assert!(!bounded_exec_source_matches(stale_github));

    let revision_mismatch_source =
        inputs.flake.replace(BOUNDED_EXEC_REVISION, "1111111111111111111111111111111111111111");
    let mut revision_mismatch = inputs;
    revision_mismatch.flake = &revision_mismatch_source;
    // r[verify lattice.repository_validation.radicle_bounded_exec_locks.scenario.drift]
    assert!(!bounded_exec_source_matches(revision_mismatch));

    let altered_lock_source = inputs
        .flake_lock
        .replace(BOUNDED_EXEC_NAR_HASH, "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=");
    let mut altered_lock = inputs;
    altered_lock.flake_lock = &altered_lock_source;
    // r[verify lattice.repository_validation.radicle_bounded_exec_locks.scenario.drift]
    assert!(!bounded_exec_source_matches(altered_lock));

    let mut unresolved_lock = inputs;
    unresolved_lock.cargo_lock = "";
    unresolved_lock.flake_lock = "{}";
    assert!(!bounded_exec_source_matches(unresolved_lock));

    let mut missing_crates = inputs;
    missing_crates.bounded_exec_crates_present = false;
    assert!(validate_contract(missing_crates).contains(ISSUE_BOUNDED_EXEC_CRATES));

    let mut malformed_evidence = inputs;
    malformed_evidence.bounded_exec_source_admission = "{";
    assert!(!bounded_exec_source_admission_matches(malformed_evidence.bounded_exec_source_admission));

    let altered_evidence_source = inputs
        .bounded_exec_source_admission
        .replace(BOUNDED_EXEC_SOURCE_ARCHIVE_BLAKE3, BOUNDED_EXEC_PUBLICATION_BLAKE3);
    let mut altered_evidence = inputs;
    altered_evidence.bounded_exec_source_admission = &altered_evidence_source;
    assert!(!bounded_exec_source_admission_matches(altered_evidence.bounded_exec_source_admission));
}

// r[verify lattice.architecture.core_crate.validation]
#[test]
fn repository_check_contract_mutations_fail_closed() {
    let mut inputs = current_inputs();
    inputs.readme = "nix flake check -L";
    inputs.flake = "nixConfig.secret-key-files = [ \"/missing\" ];";
    inputs.cargo = "[dev-dependencies]";
    inputs.gitignore = "coverage/\nresult*\n";
    inputs.json_fixture = "{";
    inputs.vendor_manifest = "[package]";
    inputs.vendor_lib = "pub fn lint_boundary(values: &[String]) -> usize { values.len() }";
    inputs.vendor_result = "pub const OTHER: &str = \"formatted\";";
    inputs.bounded_exec_source_admission = "{}";
    inputs.bounded_exec_crates_present = false;
    let issues = validate_contract(inputs);

    for code in [
        ISSUE_SIGNING_COMMAND,
        ISSUE_SIGNING_FLAKE_OVERRIDE,
        ISSUE_TEST_FILTER,
        ISSUE_PATH_DEPENDENCY,
        ISSUE_NESTED_RESULT,
        ISSUE_JSON_FIXTURE,
        ISSUE_IGNORE_SCOPE,
        ISSUE_FORMAT_SCOPE,
        ISSUE_FORMAT_PRECONDITION,
        ISSUE_CLIPPY_SCOPE,
        ISSUE_CLIPPY_PRECONDITION,
        ISSUE_BOUNDED_EXEC_SOURCE,
        ISSUE_BOUNDED_EXEC_EVIDENCE,
        ISSUE_BOUNDED_EXEC_CRATES,
    ] {
        assert!(issues.contains(code));
    }

    let unsafe_ignore = GITIGNORE_SOURCE.replace(ROOT_COVERAGE_IGNORE, UNANCHORED_COVERAGE_IGNORE);
    assert_eq!(git_ignore_admission(&unsafe_ignore, NESTED_VENDOR_PATH), IgnoreAdmission::Ignored);
    assert_eq!(rustfmt_admission("pub const SENTINEL: &str = \"formatted\";\n"), FormatAdmission::Conforming);
}

// r[verify lattice.repository_validation.native_lifecycle]
// r[verify lattice.repository_validation.native_lifecycle.scenario.parity]
// r[verify lattice.repository_validation.native_lifecycle.scenario.legacy_rejected]
// r[verify lattice.repository_validation.native_lifecycle.scenario.active_paths]
#[test]
fn reviewed_legacy_migration_preserves_hashes_and_current_admission_reads_only_native_paths() -> cairn::TestResult {
    let fixture = cairn::Fixture::new()?;
    let native = fixture.root.path().join(".cairn");
    if native.exists() {
        fs::remove_dir_all(&native)?;
    }
    let spec = "# Native lifecycle fixture\n\n## Purpose\n\nExercises native lifecycle selection.\n\n## Requirements\n\n### Requirement: Native selection\n\nr[fixture.native_selection]\n\nThe fixture MUST select native lifecycle content.\n\n#### Scenario: Current path\n\nr[fixture.native_selection.current]\n- GIVEN native content exists\n- WHEN status reads the fixture\n- THEN it MUST report the native requirement\n";
    let archived = b"reviewed archive bytes\n";
    fixture.write("cairn/specs/native-selection/spec.md", spec)?;
    fixture.write("cairn/archive/reviewed/receipt.txt", archived)?;
    let rejected = fixture
        .command()
        .args([
            "status",
            "--root",
            ".",
            "--policy",
            concat!(env!("CARGO_MANIFEST_DIR"), "/cairn-policy/generated/cairn-policy.json"),
        ])
        .output()?;
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("cairn.lifecycle_directory.migration_required"));

    let (success, preview) = fixture.run(&["migrate", "cairn"])?;
    assert!(success, "migration preview: {preview}");
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!native.exists());
    let actions = preview["actions"].as_array().ok_or("migration actions")?;
    assert_eq!(actions.len(), 2);
    for action in actions {
        let source = fixture.root.path().join(action["source"].as_str().ok_or("source")?);
        let bytes = fs::read(source)?;
        assert_eq!(action["source_hash"], blake3::hash(&bytes).to_hex().to_string());
    }
    let plan_hash = preview["plan_hash"].as_str().ok_or("plan hash")?;
    let (success, executed) = fixture.run(&["migrate", "cairn", "--execute", "--plan-hash", plan_hash])?;
    assert!(success, "migration execution: {executed}");
    assert_eq!(executed["plan_hash"], preview["plan_hash"]);
    assert_eq!(executed["mutated"], true);
    for action in actions {
        let destination = fixture.root.path().join(action["destination"].as_str().ok_or("destination")?);
        let bytes = fs::read(destination)?;
        assert_eq!(action["source_hash"], blake3::hash(&bytes).to_hex().to_string());
    }
    assert_eq!(fs::read(native.join("specs/native-selection/spec.md"))?, spec.as_bytes());
    assert_eq!(fs::read(native.join("archive/reviewed/receipt.txt"))?, archived);

    // Leave contradictory legacy content behind: the current reader must not
    // merge or fall back to it after the exact reviewed copy has completed.
    fixture.write("cairn/specs/legacy-only/spec.md", "# Legacy-only content\n")?;
    let (success, status) = fixture.run(&["status"])?;
    assert!(success, "native status: {status}");
    assert_eq!(status["layout"], "cairn");
    let specs = status["specs"].as_array().ok_or("native specs")?;
    assert_eq!(specs.len(), 1);
    assert_eq!(fs::read(native.join("specs/native-selection/spec.md"))?, spec.as_bytes());
    Ok(())
}
