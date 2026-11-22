//! Bounded catalog fetching: filtered transfer, blob-size checks and SHA-256 repositories.

mod runbook_catalog;

use std::path::Path;

use lattice::operation_pack::catalog::source::TreeEntry;
use lattice::operation_pack::catalog::transfer;
use runbook_catalog::PACK_SUBDIRECTORY;
use runbook_catalog::Repository;

const MEBIBYTE: usize = 1_048_576;
const LARGE_BLOB_BYTES: usize = 4 * MEBIBYTE;
const TRANSFER_LIMIT_BYTES: u64 = MEBIBYTE as u64;

fn request(
    repository: &Path,
    commit: &str,
    destination: &Path,
) -> lattice::operation_pack::shell::catalog::fetch::Request {
    lattice::operation_pack::shell::catalog::fetch::Request {
        repository: runbook_catalog::file_url(repository),
        commit: commit.to_string(),
        subdirectory: Some(PACK_SUBDIRECTORY.to_string()),
        expected_pack_identity: None,
        destination: destination.to_path_buf(),
        transfer_limit_bytes: TRANSFER_LIMIT_BYTES,
        process: lattice::process::core::RuntimeConfig::default(),
    }
}

fn assert_nothing_published(root: &Path, destination: &Path) {
    assert!(!destination.exists(), "destination is never published");
    let leftovers = std::fs::read_dir(root)
        .expect("root")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".fetch-staging"))
        .count();
    assert_eq!(leftovers, 0, "staging is removed");
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.fetch_bounds]
#[test]
fn a_large_blob_outside_the_pack_is_never_downloaded() {
    let root = tempfile::tempdir().expect("root");
    let (repository, commit) = runbook_catalog::catalog_repository_with(root.path(), &Repository {
        allow_filter: true,
        large_files: &[("datasets/large.bin", LARGE_BLOB_BYTES)],
        ..Repository::default()
    });
    let destination = root.path().join("fetched");
    let receipt = lattice::operation_pack::shell::catalog::fetch::run(&request(&repository, &commit, &destination))
        .expect("filtered fetch within budget");
    assert!(receipt.transferred_bytes <= TRANSFER_LIMIT_BYTES);
    assert!(receipt.transferred_bytes < LARGE_BLOB_BYTES as u64 / 4, "{} bytes", receipt.transferred_bytes);
    assert_eq!(receipt.object_format, "sha1");
    assert_eq!(receipt.workflow_bundle_count, 2);
    assert!(lattice::operation_pack::shell::load(&destination).is_ok());
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.fetch_bounds]
#[test]
fn a_transfer_beyond_the_budget_is_refused_before_publication() {
    // A server that ignores the filter sends the large blob; the file-size cap stops it.
    let ignoring = tempfile::tempdir().expect("root");
    let (repository, commit) = runbook_catalog::catalog_repository_with(ignoring.path(), &Repository {
        large_files: &[("datasets/large.bin", LARGE_BLOB_BYTES)],
        ..Repository::default()
    });
    let destination = ignoring.path().join("fetched");
    let error = lattice::operation_pack::shell::catalog::fetch::run(&request(&repository, &commit, &destination))
        .expect_err("unfiltered transfer exceeds the budget");
    assert_eq!(error.to_string(), "operation pack is invalid: catalog fetch transfer bound");
    assert_nothing_published(ignoring.path(), &destination);

    // A large blob inside the selected pack is fetched explicitly and still capped.
    let inside = tempfile::tempdir().expect("root");
    let large_member = format!("{PACK_SUBDIRECTORY}/workflow-templates/hello-runbook/large.bin");
    let (repository, commit) = runbook_catalog::catalog_repository_with(inside.path(), &Repository {
        allow_filter: true,
        large_files: &[(large_member.as_str(), LARGE_BLOB_BYTES)],
        ..Repository::default()
    });
    let destination = inside.path().join("fetched");
    let error = lattice::operation_pack::shell::catalog::fetch::run(&request(&repository, &commit, &destination))
        .expect_err("selected blob exceeds the budget");
    assert_eq!(error.to_string(), "operation pack is invalid: catalog fetch transfer bound");
    assert_nothing_published(inside.path(), &destination);

    let mut too_small = request(&repository, &commit, &destination);
    too_small.transfer_limit_bytes = transfer::MINIMUM_TRANSFER_LIMIT_BYTES - 1;
    assert!(lattice::operation_pack::shell::catalog::fetch::run(&too_small).is_err());
}

// r[verify lattice.operation_packs.runbook_catalog.scenario.fetch_bounds]
#[test]
fn sha256_repositories_fetch_into_sha256_staging() {
    let root = tempfile::tempdir().expect("root");
    let (repository, commit) = runbook_catalog::catalog_repository_with(root.path(), &Repository {
        object_format: Some("sha256"),
        allow_filter: true,
        ..Repository::default()
    });
    assert_eq!(commit.len(), 64);
    let destination = root.path().join("fetched");
    let receipt = lattice::operation_pack::shell::catalog::fetch::run(&request(&repository, &commit, &destination))
        .expect("SHA-256 fetch");
    assert_eq!(receipt.object_format, "sha256");
    let expected = lattice::operation_pack::shell::load(&runbook_catalog::seed_pack()).expect("seed pack");
    assert_eq!(receipt.pack_identity, expected.pack_identity);

    // A SHA-256 name never resolves in a SHA-1 repository.
    let sha1_root = tempfile::tempdir().expect("root");
    let (sha1_repository, _) = runbook_catalog::catalog_repository(sha1_root.path());
    let destination = sha1_root.path().join("fetched");
    assert!(
        lattice::operation_pack::shell::catalog::fetch::run(&request(&sha1_repository, &commit, &destination)).is_err()
    );
    assert_nothing_published(sha1_root.path(), &destination);
}

#[test]
fn object_sizes_are_checked_before_any_blob_is_read() {
    let object = "0123456789abcdef0123456789abcdef01234567";
    let entry = |path: &str| TreeEntry {
        path: path.to_string(),
        object: object.to_string(),
        is_executable: false,
    };
    let entries = [
        entry("manifest.ncl"),
        entry("workflow-templates/a/workflow.ncl"),
        entry("workflow-templates/a/assets/data.bin"),
    ];
    let line = |size: u64| format!("{object} blob {size}\n");
    let valid = format!("{}{}{}", line(100), line(200), line(300));
    assert_eq!(transfer::validate_object_sizes(valid.as_bytes(), &entries).expect("sizes"), [100, 200, 300]);

    let manifest_limit = lattice::operation_pack::model::MAXIMUM_MANIFEST_BYTES;
    let pack_limit = lattice::operation_pack::model::MAXIMUM_PACK_BYTES;
    let other = "fedcba9876543210fedcba9876543210fedcba98 blob 1\n";
    for (name, output) in [
        ("missing", format!("{object} missing\n{}{}", line(1), line(1))),
        ("tree", format!("{object} tree 10\n{}{}", line(1), line(1))),
        ("oversized manifest", format!("{}{}{}", line(manifest_limit + 1), line(1), line(1))),
        ("oversized file", format!("{}{}{}", line(1), line(pack_limit + 1), line(1))),
        ("oversized total", format!("{}{}{}", line(1), line(pack_limit), line(manifest_limit))),
        ("short", format!("{}{}", line(1), line(1))),
        ("other object", format!("{}{}{other}", line(1), line(1))),
    ] {
        assert!(transfer::validate_object_sizes(output.as_bytes(), &entries).is_err(), "{name}");
    }
    assert_eq!(transfer::object_format(object), "sha1");
    assert_eq!(transfer::object_format(&"a".repeat(64)), "sha256");
    assert_eq!(transfer::file_size_blocks(1_024), 2);
    assert!(transfer::check_budget(transfer::MINIMUM_TRANSFER_LIMIT_BYTES).is_ok());
    assert!(transfer::check_budget(transfer::MINIMUM_TRANSFER_LIMIT_BYTES - 1).is_err());
}
