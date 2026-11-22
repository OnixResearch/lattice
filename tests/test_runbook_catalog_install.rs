//! Installation keeps the loader's catalog limit and refuses the pack on mid-install drift.

mod runbook_catalog;

use std::path::Path;

use lattice::operation_pack::catalog::decision::Status;
use runbook_catalog::HELLO;
use runbook_catalog::STATUS;

fn install(
    pack: &Path,
    admitted: &lattice::operation_pack::AdmittedPack,
    catalog: &Path,
    confirm_create: bool,
) -> lattice::operation_pack::shell::catalog::InstallReport {
    lattice::operation_pack::shell::catalog::install(&lattice::operation_pack::shell::catalog::InstallRequest {
        pack_root: pack,
        admitted,
        catalog,
        confirm_create,
    })
    .expect("installation report")
}

fn statuses(report: &lattice::operation_pack::shell::catalog::InstallReport) -> Vec<(&str, Status, Option<&str>)> {
    report
        .bundles
        .iter()
        .map(|bundle| (bundle.id.as_str(), bundle.status, bundle.code.as_deref()))
        .collect()
}

fn fill(catalog: &Path, count: usize) {
    for index in 0..count {
        std::fs::create_dir(catalog.join(format!("filler-{index:02}"))).expect("filler directory");
    }
}

fn directory_names(directory: &Path) -> Vec<String> {
    let mut names = std::fs::read_dir(directory)
        .expect("directory")
        .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    names
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.capacity]
#[test]
fn installation_never_exceeds_the_loader_catalog_limit() {
    let maximum = lattice::workflow_template::MAX_CATALOG_BUNDLES;
    let root = tempfile::tempdir().expect("root");
    let (_, catalog) = runbook_catalog::config_root(root.path());
    fill(&catalog, maximum - 1);
    let pack = runbook_catalog::seed_pack();
    let admitted = runbook_catalog::admit(&pack);

    let preview = install(&pack, &admitted, &catalog, false);
    assert_eq!(statuses(&preview), [
        (HELLO, Status::Admitted, None),
        (STATUS, Status::Rejected, Some("catalog_capacity")),
    ]);
    let report = install(&pack, &admitted, &catalog, true);
    assert_eq!(statuses(&report), [
        (HELLO, Status::Installed, None),
        (STATUS, Status::Rejected, Some("catalog_capacity")),
    ]);
    assert!(!report.is_complete());
    assert_eq!(directory_names(&catalog).len(), maximum);
    let loaded = lattice::workflow_template::load_catalog(&catalog).expect("catalog");
    assert!(loaded.diagnostics.iter().all(|diagnostic| diagnostic.code.as_str() != "limit_exceeded"));
    assert!(
        loaded.bundles.iter().any(|bundle| bundle.manifest().id == HELLO),
        "existing and new bundles stay visible"
    );

    // A full catalog takes nothing and is left exactly as it was.
    let full = tempfile::tempdir().expect("root");
    let (_, catalog) = runbook_catalog::config_root(full.path());
    fill(&catalog, maximum);
    let before = directory_names(&catalog);
    let report = install(&pack, &admitted, &catalog, true);
    assert!(report.bundles.iter().all(|bundle| bundle.code.as_deref() == Some("catalog_capacity")));
    assert_eq!(directory_names(&catalog), before);
    assert!(directory_names(full.path().join("config").as_path()).iter().all(|name| !name.ends_with(".staging")));
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.drift]
#[test]
fn bytes_that_drift_after_admission_refuse_the_whole_pack() {
    let root = tempfile::tempdir().expect("root");
    let pack = runbook_catalog::copy_seed_pack(root.path());
    let (_, catalog) = runbook_catalog::config_root(root.path());
    let admitted = runbook_catalog::admit(&pack);
    let notes = pack.join("workflow-templates").join(STATUS).join("assets").join("notes.txt");
    let mut drifted = std::fs::read(&notes).expect("notes");
    drifted[0] ^= 1;
    std::fs::write(&notes, drifted).expect("drift after admission");

    let report = install(&pack, &admitted, &catalog, true);
    assert!(report.pack_refused);
    assert_eq!(statuses(&report), [
        (HELLO, Status::Rejected, Some("pack_refused")),
        (STATUS, Status::Rejected, Some("pack_bytes_drift")),
    ]);
    assert!(directory_names(&catalog).is_empty(), "no bundle is published");
    assert!(directory_names(root.path().join("config").as_path()).iter().all(|name| !name.ends_with(".staging")));

    let clean = tempfile::tempdir().expect("root");
    let clean_pack = runbook_catalog::copy_seed_pack(clean.path());
    let (_, clean_catalog) = runbook_catalog::config_root(clean.path());
    let report = install(&clean_pack, &runbook_catalog::admit(&clean_pack), &clean_catalog, true);
    assert!(report.is_complete());
    assert_eq!(directory_names(&clean_catalog), [HELLO, STATUS]);
}
