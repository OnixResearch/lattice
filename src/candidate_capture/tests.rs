use super::*;

#[test]
fn captures_sorted_files_and_excludes_only_root_vcs_metadata() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src/nested")).expect("source directories");
    std::fs::create_dir_all(root.join(".git")).expect("git directory");
    std::fs::create_dir_all(root.join("target/debug")).expect("target directory");
    std::fs::write(root.join("src/main.rs"), b"main").expect("main source");
    std::fs::write(root.join("src/nested/lib.rs"), b"lib").expect("nested source");
    std::fs::write(root.join("README.md"), b"readme").expect("readme");
    std::fs::write(root.join(".git/config"), b"git").expect("git metadata");
    std::fs::write(root.join("target/debug/out"), b"build").expect("build artifact");
    std::fs::write(root.join(".DS_Store"), b"host").expect("host metadata");

    let snapshot = workspace(root).expect("workspace captures");

    assert_eq!(snapshot.keys().collect::<Vec<_>>(), vec![
        ".DS_Store",
        "README.md",
        "src/main.rs",
        "src/nested/lib.rs",
        "target/debug/out"
    ]);
    assert_eq!(snapshot["src/main.rs"], b"main");
}

#[test]
fn captures_agent_runtime_metadata_rather_than_silently_omitting_files() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join(".omp/agent")).expect("OMP metadata directory");
    std::fs::write(root.join(".omp/agent/models.yml"), b"models").expect("OMP model metadata");
    std::fs::write(root.join("src.rs"), b"source").expect("source file");

    let snapshot = workspace(root).expect("workspace captures");

    assert_eq!(snapshot.keys().collect::<Vec<_>>(), vec![".omp/agent/models.yml", "src.rs"]);
}

#[cfg(unix)]
#[test]
fn rejects_symlinks() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::write(root.join("real"), b"real").expect("real file");
    std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("symlink");

    assert!(matches!(workspace(root), Err(Error::SymlinkRejected)));
}

#[test]
fn admits_read_and_write_paths_within_the_root() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src/nested")).expect("source directory");
    std::fs::write(root.join("src/main.rs"), b"main").expect("source file");

    let boundary = admit_workspace_boundary(
        root,
        &["src".to_string(), "src/nested".to_string()],
        &["src".to_string(), "src/new.rs".to_string()],
        false,
        false,
    )
    .expect("boundary admits");
    assert_eq!(boundary.read_paths, vec!["src", "src/nested"]);
    assert_eq!(boundary.write_paths, vec!["src", "src/new.rs"]);
    assert!(!boundary.network);
    assert!(!boundary.credentials);
}

#[test]
fn rejects_vcs_metadata_boundaries() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join(".git")).expect("git metadata");
    std::fs::create_dir_all(root.join("src")).expect("source directory");

    assert!(matches!(
        admit_workspace_boundary(root, &[".git".to_string()], &[], false, false),
        Err(Error::VcsMetadataRejected)
    ));
    assert!(matches!(
        admit_workspace_boundary(root, &["src".to_string()], &["src/.git/config".to_string()], false, false),
        Err(Error::VcsMetadataRejected)
    ));
}

#[test]
fn rejects_escaping_outside_and_uncovered_write_paths() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    std::fs::create_dir_all(root.join("src")).expect("source directory");

    assert!(matches!(
        admit_workspace_boundary(root, &["../outside".to_string()], &[], false, false),
        Err(Error::InvalidPath)
    ));
    assert!(matches!(
        admit_workspace_boundary(root, &["src".to_string()], &["docs".to_string()], false, false),
        Err(Error::WriteOutsideRead)
    ));
    assert!(matches!(
        admit_workspace_boundary(root, &["src".to_string(), "src".to_string()], &[], false, false),
        Err(Error::DuplicateBoundaryPath)
    ));
}

#[cfg(unix)]
#[test]
fn rejects_symlink_escape_from_the_boundary() {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let root = directory.path();
    let outside = tempfile::tempdir().expect("outside directory");
    std::os::unix::fs::symlink(outside.path(), root.join("escape")).expect("symlink");

    assert!(matches!(
        admit_workspace_boundary(root, &["escape".to_string()], &[], false, false),
        Err(Error::BoundaryEscape)
    ));
}
