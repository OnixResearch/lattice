pub(super) fn validate_snapshot(
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<(), super::CandidateReceiptError> {
    if snapshot.len() > super::MAX_SNAPSHOT_FILES {
        return Err(super::CandidateReceiptError::SnapshotTooLarge);
    }
    let mut total = 0usize;
    for (path, bytes) in snapshot {
        super::admission::changed_paths(std::slice::from_ref(path))?;
        total = total.checked_add(bytes.len()).ok_or(super::CandidateReceiptError::SnapshotTooLarge)?;
    }
    if total > super::MAX_SNAPSHOT_BYTES {
        return Err(super::CandidateReceiptError::SnapshotTooLarge);
    }
    Ok(())
}

/// One file as the tree and diff identities hash it: its length and the
/// BLAKE3 of its bytes.
pub(super) type Entry = (u64, [u8; 32]);

pub(super) fn entry(bytes: &[u8]) -> Entry {
    (bytes.len() as u64, *blake3::hash(bytes).as_bytes())
}

pub(super) fn tree_identity(snapshot: &std::collections::BTreeMap<String, Vec<u8>>) -> String {
    tree_identity_of(snapshot.len(), snapshot.iter().map(|(path, bytes)| (path.as_str(), entry(bytes))))
}

/// [`tree_identity`] over `count` path-ordered entries.
pub(super) fn tree_identity_of<'a>(count: usize, entries: impl Iterator<Item = (&'a str, Entry)>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-tree-v1");
    update_len(&mut hasher, count);
    for (path, (len, hash)) in entries {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        hasher.update(&len.to_le_bytes());
        hasher.update(&hash);
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn diff_identity(
    base: &std::collections::BTreeMap<String, Vec<u8>>,
    candidate: &std::collections::BTreeMap<String, Vec<u8>>,
    paths: &[String],
) -> String {
    diff_identity_of(
        paths,
        |path| base.get(path).map(|bytes| entry(bytes)),
        |path| candidate.get(path).map(|bytes| entry(bytes)),
    )
}

/// [`diff_identity`] over the entries `base` and `candidate` give each path.
pub(super) fn diff_identity_of(
    paths: &[String],
    base: impl Fn(&str) -> Option<Entry>,
    candidate: impl Fn(&str) -> Option<Entry>,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-diff-v1");
    update_len(&mut hasher, paths.len());
    for path in paths {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        update_optional(&mut hasher, base(path));
        update_optional(&mut hasher, candidate(path));
    }
    hasher.finalize().to_hex().to_string()
}

fn update_optional(hasher: &mut blake3::Hasher, entry: Option<Entry>) {
    match entry {
        Some((len, hash)) => {
            hasher.update(b"present");
            hasher.update(&len.to_le_bytes());
            hasher.update(&hash);
        }
        None => {
            hasher.update(b"absent");
        }
    }
}

fn update_len(hasher: &mut blake3::Hasher, len: usize) {
    hasher.update(&(len as u64).to_le_bytes());
}

pub(super) fn is_lowercase_blake3(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
