use super::*;

#[test]
fn rejects_an_oversized_sparse_file_before_loading_its_bytes() {
    let directory = tempfile::tempdir().expect("workspace");
    let file = std::fs::File::create(directory.path().join("large-source")).expect("source file");
    let limit = u64::try_from(MAX_SNAPSHOT_BYTES).expect("snapshot limit fits u64");
    file.set_len(limit.checked_add(1).expect("oversized test bound")).expect("sparse source length");
    assert!(matches!(workspace(directory.path()), Err(Error::TooLarge)));
}

#[test]
fn bounded_read_admits_exact_remaining_bytes_and_rejects_excess() {
    let directory = tempfile::tempdir().expect("workspace");
    let path = directory.path().join("source");
    std::fs::write(&path, b"abc").expect("source bytes");
    let metadata = std::fs::metadata(&path).expect("source metadata");
    let exact_total = MAX_SNAPSHOT_BYTES.checked_sub(3).expect("exact remaining bound");
    assert_eq!(read_file(&path, &metadata, exact_total).expect("exact bound"), b"abc");
    let excessive_total = MAX_SNAPSHOT_BYTES.checked_sub(2).expect("insufficient remaining bound");
    assert!(matches!(read_file(&path, &metadata, excessive_total), Err(Error::TooLarge)));
    let invalid_total = MAX_SNAPSHOT_BYTES.checked_add(1).expect("invalid accumulated total");
    assert!(matches!(read_file(&path, &metadata, invalid_total), Err(Error::TooLarge)));
}
