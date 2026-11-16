use super::*;

// r[verify lattice.candidate_worker.file_capture]
#[test]
fn rejects_a_workspace_hard_link_to_an_unadmitted_host_file() {
    let directory = tempfile::tempdir().expect("workspace");
    let outside = directory.path().join("publisher-token");
    let workspace = directory.path().join("workspace");
    std::fs::create_dir(&workspace).expect("source root");
    std::fs::write(&outside, b"host secret").expect("host file");
    std::fs::hard_link(&outside, workspace.join("alias")).expect("hard link");
    assert!(matches!(super::workspace(&workspace), Err(Error::HardLinkRejected)));
    assert_eq!(std::fs::read(outside).expect("host file retained"), b"host secret");
}
