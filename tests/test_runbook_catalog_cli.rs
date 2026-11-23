//! End-to-end `lattice pack catalog-*` behavior over a local `file://` catalog repository.

mod runbook_catalog;

use std::path::Path;
use std::process::Command;

use runbook_catalog::HELLO;
use runbook_catalog::LICENSE;
use runbook_catalog::PACK_SUBDIRECTORY;
use runbook_catalog::STATUS;

fn lattice(config: &Path, no_git: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
    // Commands other than `catalog-fetch` must work without any `git` executable on PATH.
    command.env("PATH", no_git).arg("--config").arg(config).arg("pack");
    command
}

fn stderr(output: &std::process::Output) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(&output.stderr)
}

fn json(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|_| panic!("JSON output; stderr: {}", stderr(output)))
}

fn lock(config: &Path, no_git: &Path, pack: &Path, destination: &Path) {
    let output = lattice(config, no_git)
        .args([
            "lock",
            "--confirm-trust-local",
            "--allow-license",
            LICENSE,
            "--confirm-create",
        ])
        .arg("--pack")
        .arg(pack)
        .arg("--destination")
        .arg(destination)
        .output()
        .expect("lock command");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

fn install(
    config: &Path,
    no_git: &Path,
    pack: &Path,
    lock: &Path,
    catalog: &Path,
    confirm: bool,
) -> std::process::Output {
    let mut command = lattice(config, no_git);
    command
        .args(["catalog-install", "--confirm-trust-local", "--allow-license", LICENSE])
        .arg("--pack")
        .arg(pack)
        .arg("--lock")
        .arg(lock)
        .arg("--catalog")
        .arg(catalog);
    if confirm {
        command.arg("--confirm-create");
    }
    command.output().expect("catalog-install command")
}

fn statuses(report: &serde_json::Value) -> Vec<(String, String, Option<String>)> {
    report["bundles"]
        .as_array()
        .expect("bundle outcomes")
        .iter()
        .map(|bundle| {
            (
                bundle["id"].as_str().expect("id").to_string(),
                bundle["status"].as_str().expect("status").to_string(),
                bundle["code"].as_str().map(str::to_string),
            )
        })
        .collect()
}

// r[verify lattice.operation_packs.runbook_catalog]
// r[verify lattice.operation_packs.runbook_catalog.scenario.install]
#[test]
fn fetch_lock_install_and_list_from_an_exact_commit_without_network_on_load() {
    let root = tempfile::tempdir().expect("test root");
    let no_git = root.path().join("empty-path");
    std::fs::create_dir(&no_git).expect("empty PATH directory");
    let (config, catalog) = runbook_catalog::config_root(root.path());
    let (repository, commit) = runbook_catalog::catalog_repository(root.path());
    let expected = lattice::operation_pack::shell::load(&runbook_catalog::seed_pack()).expect("seed pack");
    let fetched = root.path().join("fetched-pack");

    let preview = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config)
        .args([
            "pack",
            "catalog-fetch",
            "--repository",
            &runbook_catalog::file_url(&repository),
            "--commit",
            &commit,
        ])
        .args([
            "--subdirectory",
            PACK_SUBDIRECTORY,
            "--pack-identity",
            &expected.pack_identity,
        ])
        .arg("--destination")
        .arg(&fetched)
        .output()
        .expect("fetch preview");
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    assert_eq!(json(&preview)["network_accessed"], false);
    assert!(!fetched.exists(), "fetch preview must not contact the repository or write");

    let fetch = Command::new(env!("CARGO_BIN_EXE_lattice"))
        .arg("--config")
        .arg(&config)
        .args([
            "pack",
            "catalog-fetch",
            "--repository",
            &runbook_catalog::file_url(&repository),
            "--commit",
            &commit,
        ])
        .args([
            "--subdirectory",
            PACK_SUBDIRECTORY,
            "--pack-identity",
            &expected.pack_identity,
            "--confirm-fetch",
        ])
        .arg("--destination")
        .arg(&fetched)
        .output()
        .expect("fetch command");
    assert!(fetch.status.success(), "{}", String::from_utf8_lossy(&fetch.stderr));
    let receipt = json(&fetch);
    assert_eq!(receipt["commit"], commit.as_str());
    assert_eq!(receipt["pack_identity"], expected.pack_identity.as_str());
    assert_eq!(receipt["workflow_bundle_count"], 2);
    assert_eq!(receipt["grants_authority"], false);
    assert!(!root.path().join(".fetched-pack.fetch-staging").exists());

    // The source repository disappears: lock, install and list must use only local bytes.
    std::fs::remove_dir_all(&repository).expect("remove the catalog repository");
    let lock_path = root.path().join("seed.lock.json");
    lock(&config, &no_git, &fetched, &lock_path);

    let previewed = install(&config, &no_git, &fetched, &lock_path, &catalog, false);
    assert!(previewed.status.success(), "{}", String::from_utf8_lossy(&previewed.stderr));
    let previewed = json(&previewed);
    assert_eq!(previewed["required_flag"], "--confirm-create");
    assert!(statuses(&previewed).iter().all(|(_, status, _)| status == "admitted"));
    assert_eq!(std::fs::read_dir(&catalog).expect("catalog").count(), 0, "preview writes nothing");

    let installed = install(&config, &no_git, &fetched, &lock_path, &catalog, true);
    assert!(installed.status.success(), "{}", String::from_utf8_lossy(&installed.stderr));
    let report = json(&installed);
    assert_eq!(report["installed_count"], 2);
    assert_eq!(report["grants_authority"], false);
    assert_eq!(report["network_accessed"], false);
    let loaded = lattice::workflow_template::load_catalog(&catalog).expect("installed catalog");
    assert!(loaded.diagnostics.is_empty(), "{:?}", loaded.diagnostics);
    let ids = loaded.bundles.iter().map(|bundle| bundle.manifest().id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, [HELLO, STATUS]);
    let config_directory = catalog.parent().expect("config directory");
    assert!(
        std::fs::read_dir(config_directory)
            .expect("config directory")
            .all(|entry| { !entry.expect("entry").file_name().to_string_lossy().ends_with(".staging") })
    );
    assert!(!config_directory.join("data").exists(), "installation touches only the catalog");

    let listed = lattice(&config, &no_git).arg("catalog-list").arg("--catalog").arg(&catalog).output().expect("list");
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    assert_eq!(json(&listed)["bundles"].as_array().expect("listed bundles").len(), 2);

    // Create-only: a second installation reports a conflict and leaves operator bytes alone.
    let before = std::fs::read(catalog.join(HELLO).join("workflow.ncl")).expect("installed bytes");
    let again = install(&config, &no_git, &fetched, &lock_path, &catalog, true);
    assert!(!again.status.success(), "a second installation must be refused; stderr: {}", stderr(&again));
    assert!(
        statuses(&json(&again))
            .iter()
            .all(|(_, status, code)| status == "conflict" && code.as_deref() == Some("catalog_conflict")),
        "every bundle must conflict; stderr: {}",
        stderr(&again)
    );
    assert_eq!(
        std::fs::read(catalog.join(HELLO).join("workflow.ncl")).expect("installed bytes"),
        before,
        "stderr: {}",
        stderr(&again)
    );
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.fetch_denied]
#[test]
fn fetch_refuses_inexact_sources_wrong_identity_and_symlink_members() {
    let root = tempfile::tempdir().expect("test root");
    let (config, _) = runbook_catalog::config_root(root.path());
    let (repository, commit) = runbook_catalog::catalog_repository(root.path());
    let url = runbook_catalog::file_url(&repository);
    let fetch = |repository: &str, commit: &str, identity: Option<&str>, destination: &Path| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_lattice"));
        command
            .arg("--config")
            .arg(&config)
            .args([
                "pack",
                "catalog-fetch",
                "--repository",
                repository,
                "--commit",
                commit,
                "--confirm-fetch",
            ])
            .args(["--subdirectory", PACK_SUBDIRECTORY])
            .arg("--destination")
            .arg(destination);
        if let Some(identity) = identity {
            command.args(["--pack-identity", identity]);
        }
        command.output().expect("fetch command")
    };
    let destination = root.path().join("fetched");
    for (locator, revision) in [
        (url.as_str(), "HEAD"),
        (url.as_str(), &commit[..12]),
        ("ext::sh -c touch% /tmp/pwned", commit.as_str()),
        (repository.to_str().expect("UTF-8 path"), commit.as_str()),
    ] {
        let output = fetch(locator, revision, None, &destination);
        assert!(!output.status.success(), "{locator} {revision} must be refused; stderr: {}", stderr(&output));
        assert!(!destination.exists(), "{locator} {revision} wrote its destination; stderr: {}", stderr(&output));
    }

    let placeholder = runbook_catalog::placeholder();
    let wrong_identity = fetch(&url, &commit, Some(&placeholder), &destination);
    assert!(
        !wrong_identity.status.success(),
        "a wrong identity must be refused; stderr: {}",
        stderr(&wrong_identity)
    );
    assert!(
        stderr(&wrong_identity).contains("fetched pack identity differs"),
        "unexpected refusal; stderr: {}",
        stderr(&wrong_identity)
    );
    assert!(!destination.exists(), "a wrong identity wrote its destination; stderr: {}", stderr(&wrong_identity));
    assert!(
        !root.path().join(".fetched.fetch-staging").exists(),
        "a wrong identity left its staging; stderr: {}",
        stderr(&wrong_identity)
    );

    let link = repository.join(PACK_SUBDIRECTORY).join("workflow-templates").join(HELLO).join("linked.ncl");
    std::os::unix::fs::symlink("/etc/hostname", &link).expect("symlink member");
    runbook_catalog::git(&repository, &["add", "--all"]);
    runbook_catalog::git(&repository, &["commit", "--quiet", "--message", "Add a symlink member"]);
    let linked_commit = runbook_catalog::git(&repository, &["rev-parse", "HEAD"]);
    let symlinked = fetch(&url, &linked_commit, None, &destination);
    assert!(!symlinked.status.success(), "a symlink member must be refused; stderr: {}", stderr(&symlinked));
    assert!(
        stderr(&symlinked).contains("catalog symlink member"),
        "unexpected refusal; stderr: {}",
        stderr(&symlinked)
    );
    assert!(!destination.exists(), "a symlink member wrote its destination; stderr: {}", stderr(&symlinked));
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.drift]
#[test]
fn tampered_bytes_refuse_the_pack_and_name_each_drifted_bundle() {
    let root = tempfile::tempdir().expect("test root");
    let no_git = root.path().join("empty-path");
    std::fs::create_dir(&no_git).expect("empty PATH directory");
    let (config, catalog) = runbook_catalog::config_root(root.path());
    let pack = runbook_catalog::copy_seed_pack(root.path());
    let lock_path = root.path().join("seed.lock.json");
    lock(&config, &no_git, &pack, &lock_path);
    let workflow = pack.join("workflow-templates").join(HELLO).join("workflow.ncl");
    let tampered = std::fs::read_to_string(&workflow).expect("workflow").replace("hello from", "HELLO FROM");
    std::fs::write(&workflow, tampered).expect("tamper bundle bytes");

    let output = install(&config, &no_git, &pack, &lock_path, &catalog, true);
    assert!(!output.status.success(), "a tampered pack must be refused; stderr: {}", stderr(&output));
    let report = json(&output);
    assert_eq!(report["pack_refused"], true, "stderr: {}", stderr(&output));
    assert_eq!(
        statuses(&report),
        [
            (HELLO.to_string(), "rejected".to_string(), Some("pack_bytes_drift".to_string())),
            (STATUS.to_string(), "rejected".to_string(), Some("pack_refused".to_string())),
        ],
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(std::fs::read_dir(&catalog).expect("catalog").count(), 0, "stderr: {}", stderr(&output));
}
