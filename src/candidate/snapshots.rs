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

pub(super) fn tree_identity(snapshot: &std::collections::BTreeMap<String, Vec<u8>>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-tree-v1");
    update_len(&mut hasher, snapshot.len());
    for (path, bytes) in snapshot {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        update_len(&mut hasher, bytes.len());
        hasher.update(blake3::hash(bytes).as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

pub(super) fn diff_identity(
    base: &std::collections::BTreeMap<String, Vec<u8>>,
    candidate: &std::collections::BTreeMap<String, Vec<u8>>,
    paths: &[String],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"lattice-candidate-diff-v1");
    update_len(&mut hasher, paths.len());
    for path in paths {
        update_len(&mut hasher, path.len());
        hasher.update(path.as_bytes());
        update_optional_bytes(&mut hasher, base.get(path).map(Vec::as_slice));
        update_optional_bytes(&mut hasher, candidate.get(path).map(Vec::as_slice));
    }
    hasher.finalize().to_hex().to_string()
}

fn update_optional_bytes(hasher: &mut blake3::Hasher, bytes: Option<&[u8]>) {
    match bytes {
        Some(bytes) => {
            hasher.update(b"present");
            update_len(hasher, bytes.len());
            hasher.update(blake3::hash(bytes).as_bytes());
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
