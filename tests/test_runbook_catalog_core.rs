//! Runbook-catalog manifest validation, per-bundle admission and the authority boundary.

mod runbook_catalog;

use std::path::Path;

use lattice::operation_pack::catalog::decision::Status;
use runbook_catalog::HELLO;
use runbook_catalog::STATUS;

const POLICY_VALID: &str = "tests/fixtures/runbook-catalog/manifest-policy-valid.ncl";
const POLICY_INVALID: &str = "tests/fixtures/runbook-catalog/manifest-policy-invalid.ncl";
const UNKNOWN_FEATURE: &str = "lattice.unknown-runbook-feature.v9";

fn install(pack: &Path, catalog: &Path) -> lattice::operation_pack::shell::catalog::InstallReport {
    let admitted = runbook_catalog::admit(pack);
    lattice::operation_pack::shell::catalog::install(&lattice::operation_pack::shell::catalog::InstallRequest {
        pack_root: pack,
        admitted: &admitted,
        catalog,
        confirm_create: true,
    })
    .expect("installation report")
}

fn outcome<'report>(
    report: &'report lattice::operation_pack::shell::catalog::InstallReport,
    id: &str,
) -> &'report lattice::operation_pack::catalog::decision::BundleOutcome {
    report.bundles.iter().find(|bundle| bundle.id == id).expect("bundle outcome")
}

fn edit(path: &Path, from: &str, to: &str) {
    let source = std::fs::read_to_string(path).expect("fixture text");
    assert!(source.contains(from), "{from} in {}", path.display());
    std::fs::write(path, source.replace(from, to)).expect("fixture edit");
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.manifest]
#[test]
fn seed_manifest_satisfies_the_contract_and_identity_covers_every_bundle_byte() {
    let valid = std::fs::canonicalize(POLICY_VALID).expect("valid policy fixture");
    assert!(lattice::operation_pack::shell::validate_manifest_contract(&valid).is_ok());
    let invalid = std::fs::canonicalize(POLICY_INVALID).expect("invalid policy fixture");
    assert!(lattice::operation_pack::shell::validate_manifest_contract(&invalid).is_err());

    let candidate = lattice::operation_pack::shell::load(&runbook_catalog::seed_pack()).expect("seed pack");
    assert_eq!(candidate.manifest.workflow_bundles.len(), 2);
    assert_eq!(candidate.measured_contents.len(), 5);
    let observed = runbook_catalog::loader_identities(&runbook_catalog::seed_pack());
    for declaration in &candidate.manifest.workflow_bundles {
        assert_eq!(observed.get(&declaration.id), Some(&declaration.bundle_identity), "seed pins are current");
    }
    for index in 0..candidate.manifest.contents.len() {
        let mut changed = candidate.manifest.clone();
        changed.contents[index].content_identity = runbook_catalog::placeholder();
        let changed_identity = lattice::operation_pack::pack_identity(&changed).expect("pack identity");
        assert_ne!(changed_identity, candidate.pack_identity, "content {index} is covered");
    }
    let mut repinned = candidate.manifest.clone();
    repinned.workflow_bundles[0].bundle_identity = runbook_catalog::placeholder();
    assert_ne!(lattice::operation_pack::pack_identity(&repinned).expect("pack identity"), candidate.pack_identity);
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.manifest]
#[test]
fn pack_manifests_reject_traversal_duplicates_misplacement_and_symlinks() {
    let root = tempfile::tempdir().expect("test root");
    let pack = runbook_catalog::copy_seed_pack(root.path());
    let manifest = pack.join("manifest.ncl");
    let seed = std::fs::read_to_string(&manifest).expect("seed manifest");
    let cases = [
        (
            "duplicate workflow bundle id",
            "id = \"status-runbook\",\n      bundle_identity",
            "id = \"hello-runbook\",\n      bundle_identity",
        ),
        (
            "content path",
            "path = \"workflow-templates/hello-runbook/workflow.ncl\"",
            "path = \"workflow-templates/../hello-runbook/workflow.ncl\"",
        ),
        (
            "workflow bundle content placement",
            "path = \"workflow-templates/hello-runbook/workflow.ncl\"",
            "path = \"workflow-templates/other-runbook/workflow.ncl\"",
        ),
        (
            "workflow bundle manifest content",
            "path = \"workflow-templates/hello-runbook/manifest.ncl\"",
            "path = \"workflow-templates/hello-runbook/manifest-copy.ncl\"",
        ),
        (
            "workflow bundle content placement",
            "role = \"workflow_bundle\",\n      path = \"workflow-templates/status-runbook/assets",
            "role = \"skill_reference\",\n      path = \"workflow-templates/status-runbook/assets",
        ),
        (
            "workflow bundle id",
            "id = \"hello-runbook\",\n      bundle_identity",
            "id = \"Hello_Runbook\",\n      bundle_identity",
        ),
    ];
    for (reason, from, to) in cases {
        assert!(seed.contains(from), "{reason}");
        std::fs::write(&manifest, seed.replace(from, to)).expect("mutated manifest");
        let error = lattice::operation_pack::shell::load(&pack).expect_err("mutated manifest must be refused");
        assert_eq!(error.to_string(), format!("operation pack is invalid: {reason}"));
    }
    std::fs::write(&manifest, &seed).expect("restore manifest");
    assert!(lattice::operation_pack::shell::load(&pack).is_ok());
    let link = pack.join("workflow-templates").join(HELLO).join("linked.ncl");
    std::os::unix::fs::symlink("workflow.ncl", &link).expect("symlink member");
    let error = lattice::operation_pack::shell::load(&pack).expect_err("symlink members are refused");
    assert_eq!(error.to_string(), "operation pack is invalid: pack symlink");
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.partial]
#[test]
fn one_bad_bundle_is_rejected_with_its_diagnostic_while_the_other_installs() {
    type Mutation = fn(&Path) -> Vec<(&'static str, Option<&'static str>)>;
    let wrong_identity: Mutation = |_| {
        vec![
            (HELLO, Some("b3:1111111111111111111111111111111111111111111111111111111111111111")),
            (STATUS, None),
        ]
    };
    let incompatible: Mutation = |pack| {
        let manifest = pack.join("workflow-templates").join(HELLO).join("manifest.ncl");
        edit(
            &manifest,
            "feature_ids = [\"lattice.workflow-template-bundle.v1\"]",
            &format!("feature_ids = [\"{UNKNOWN_FEATURE}\"]"),
        );
        vec![(HELLO, None), (STATUS, None)]
    };
    let stale_internal_pin: Mutation = |pack| {
        let workflow = pack.join("workflow-templates").join(HELLO).join("workflow.ncl");
        edit(&workflow, "Print one fixed runbook line", "Print one changed runbook line");
        vec![(HELLO, None), (STATUS, None)]
    };
    for (name, mutation, code) in [
        ("wrong identity", wrong_identity, "identity_mismatch"),
        ("compatibility", incompatible, "incompatible_content"),
        ("stale bundle pin", stale_internal_pin, "identity_mismatch"),
    ] {
        let root = tempfile::tempdir().expect("test root");
        let pack = runbook_catalog::copy_seed_pack(root.path());
        let declarations = mutation(&pack);
        runbook_catalog::regenerate_manifest(&pack, &declarations);
        let (_, catalog) = runbook_catalog::config_root(root.path());
        let report = install(&pack, &catalog);
        let rejected = outcome(&report, HELLO);
        assert_eq!(rejected.status, Status::Rejected, "{name}");
        assert_eq!(rejected.code.as_deref(), Some(code), "{name}: {rejected:?}");
        assert_eq!(outcome(&report, STATUS).status, Status::Installed, "{name}: the good bundle still installs");
        assert!(!report.is_complete());
        assert!(!catalog.join(HELLO).exists(), "{name}: nothing of the bad bundle is published");
        let loaded = lattice::workflow_template::load_catalog(&catalog).expect("catalog");
        assert_eq!(loaded.bundles.len(), 1);
        assert!(loaded.diagnostics.is_empty());
    }
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.authority]
#[tokio::test]
async fn installation_grants_no_effect_authority() {
    let root = tempfile::tempdir().expect("test root");
    let (_, catalog) = runbook_catalog::config_root(root.path());
    let report = install(&runbook_catalog::seed_pack(), &catalog);
    assert!(report.is_complete());
    assert!(!report.grants_authority);
    let loaded = lattice::workflow_template::load_catalog(&catalog).expect("installed catalog");
    let bundle = loaded.bundles.iter().find(|bundle| bundle.manifest().id == HELLO).expect("installed bundle");

    let inspection =
        lattice::engine::effect::standalone::inspect_workflow(bundle.workflow()).expect("authority requirements");
    assert!(!inspection.grants.is_empty(), "the installed workflow still needs a reviewed authority profile");
    let store = lattice::store::Store::new(root.path().join("state.redb")).expect("store");
    store.save_workflow(bundle.workflow()).expect("saved workflow");
    let run = store.next_run_id().expect("run id");
    let summary = lattice::headless::run_saved_workflow_with_authority_provider(
        &store,
        &bundle.workflow().meta.name,
        run,
        &lattice::engine::effect::execution::UnavailableRunContextSource,
    )
    .await
    .expect("terminal summary");
    assert_eq!(summary.status, lattice::headless::HeadlessRunStatus::Failed, "no authority, no effect");
}

#[test]
fn tree_listings_admit_only_regular_files_inside_the_selected_pack() {
    let object = "0123456789abcdef0123456789abcdef01234567";
    let record = |mode: &str, path: &str| format!("{mode} blob {object}\tpacks/seed/{path}\0").into_bytes();
    let parse = |listing: &[u8]| lattice::operation_pack::catalog::source::parse_tree(listing, Some("packs/seed"));
    let mut valid = record("100644", "manifest.ncl");
    valid.extend(record("100755", "workflow-templates/a/run.sh"));
    let entries = parse(&valid).expect("regular files");
    assert_eq!(entries.len(), 2);
    assert!(entries[1].is_executable);
    assert!(parse(&record("120000", "manifest.ncl")).is_err(), "symlink");
    assert!(parse(&record("160000", "vendor")).is_err(), "submodule");
    assert!(parse(format!("100644 blob {object}\tother/manifest.ncl\0").as_bytes()).is_err(), "outside");
    assert!(parse(&record("100644", "a/../manifest.ncl")).is_err(), "traversal");
    assert!(parse(b"").is_err(), "empty");
    for commit in ["HEAD", "main", "0123456", "0123456789ABCDEF0123456789ABCDEF01234567"] {
        assert!(lattice::operation_pack::catalog::source::validate_commit(commit).is_err(), "{commit}");
    }
    assert!(lattice::operation_pack::catalog::source::validate_commit(object).is_ok());
    for locator in [
        "ext::sh -c id",
        "/srv/catalog",
        "-uhelper",
        "git://example.invalid/catalog",
        "file://",
    ] {
        assert!(lattice::operation_pack::catalog::source::validate_repository_locator(locator).is_err(), "{locator}");
    }
    assert!(
        lattice::operation_pack::catalog::source::validate_repository_locator("https://example.invalid/catalog.git")
            .is_ok()
    );
}
