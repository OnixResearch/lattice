use super::FileDigest;

fn tree(files: &[(&str, &str)]) -> std::collections::BTreeMap<String, Vec<u8>> {
    files.iter().map(|(path, bytes)| ((*path).to_owned(), bytes.as_bytes().to_vec())).collect()
}

fn digests(snapshot: &std::collections::BTreeMap<String, Vec<u8>>) -> std::collections::BTreeMap<String, FileDigest> {
    snapshot.iter().map(|(path, bytes)| (path.clone(), FileDigest::of(bytes))).collect()
}

const BINDINGS: super::super::Bindings<'static> = super::super::Bindings {
    work_id: "work-1",
    attempt_id: "attempt-1",
    fence: "7",
    workspace_id: "workspace-1",
};

#[test]
fn changed_base_digests_re_derive_the_snapshot_receipt() {
    let base = tree(&[("a", "1"), ("b", "2"), ("c", "3")]);
    let candidate = tree(&[("a", "1"), ("b", "20"), ("d", "4")]);
    let expected = super::super::produced_receipt_from_snapshots(BINDINGS, &base, &candidate).expect("receipt");
    let changes = std::collections::BTreeMap::from([
        ("b".to_owned(), Some(FileDigest::of(b"2"))),
        ("c".to_owned(), Some(FileDigest::of(b"3"))),
        ("d".to_owned(), None),
    ]);
    let derived = super::produced_receipt_from_changes(BINDINGS, &digests(&candidate), &changes).expect("derived");
    assert_eq!(derived, expected);
    assert_eq!(derived.base_identity, super::super::snapshot_identity(&base).expect("base"));
}

#[test]
fn claimed_changes_that_do_not_re_derive_are_refused() {
    let candidate = digests(&tree(&[("a", "1"), ("b", "20")]));
    let base_b = ("b".to_owned(), Some(FileDigest::of(b"2")));
    let unchanged = std::collections::BTreeMap::from([base_b.clone(), ("a".to_owned(), Some(FileDigest::of(b"1")))]);
    assert_eq!(
        super::produced_receipt_from_changes(BINDINGS, &candidate, &unchanged),
        Err(super::super::CandidateReceiptError::ReceiptMismatch),
        "a listed path equal to the candidate"
    );
    assert_eq!(
        super::produced_receipt_from_changes(BINDINGS, &candidate, &std::collections::BTreeMap::new()),
        Err(super::super::CandidateReceiptError::NoCandidateChanges)
    );
    let malformed = std::collections::BTreeMap::from([(
        "b".to_owned(),
        Some(FileDigest {
            len: 1,
            blake3: "Z".repeat(64),
        }),
    )]);
    assert_eq!(
        super::produced_receipt_from_changes(BINDINGS, &candidate, &malformed),
        Err(super::super::CandidateReceiptError::InvalidIdentity)
    );
    // Another base for the same path yields another base and diff identity.
    let honest = std::collections::BTreeMap::from([base_b]);
    let other = std::collections::BTreeMap::from([("b".to_owned(), Some(FileDigest::of(b"other")))]);
    let honest = super::produced_receipt_from_changes(BINDINGS, &candidate, &honest).expect("honest");
    let other = super::produced_receipt_from_changes(BINDINGS, &candidate, &other).expect("other");
    assert_ne!(honest.base_identity, other.base_identity);
    assert_ne!(honest.diff_identity, other.diff_identity);
    assert_eq!(honest.candidate_identity, other.candidate_identity);
}
