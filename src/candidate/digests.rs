//! Produced receipts re-derived from file digests.
//!
//! A captured candidate and the base digests of its changed paths determine
//! the base snapshot's digests: every other base file equals the candidate's.
//! From them the receipt is derived with the same tree and diff identities as
//! [`super::produced_receipt_from_snapshots`], so a retained receipt can be
//! checked against a later capture of the candidate without the base bytes.

#[cfg(test)]
mod tests;

/// The length and BLAKE3 of one captured file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileDigest {
    pub len: u64,
    /// Lowercase hex BLAKE3 of the file bytes.
    pub blake3: String,
}

impl FileDigest {
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        let (len, hash) = super::snapshots::entry(bytes);
        Self {
            len,
            blake3: blake3::Hash::from_bytes(hash).to_hex().to_string(),
        }
    }

    fn entry(&self) -> Result<super::snapshots::Entry, super::CandidateReceiptError> {
        if !super::snapshots::is_lowercase_blake3(&self.blake3) {
            return Err(super::CandidateReceiptError::InvalidIdentity);
        }
        let hash = blake3::Hash::from_hex(&self.blake3).map_err(|_| super::CandidateReceiptError::InvalidIdentity)?;
        Ok((self.len, *hash.as_bytes()))
    }
}

type Entries<'a> = std::collections::BTreeMap<&'a str, super::snapshots::Entry>;

/// The produced receipt of `candidate` against the base that differs from it
/// exactly at `changes`: each changed path's base digest, `None` where the
/// base had no file. A listed path whose base equals the candidate is
/// refused, so the changed paths are re-derived, never taken from a claim.
pub fn produced_receipt_from_changes(
    bindings: super::Bindings<'_>,
    candidate: &std::collections::BTreeMap<String, FileDigest>,
    changes: &std::collections::BTreeMap<String, Option<FileDigest>>,
) -> Result<super::CandidateReceipt, super::CandidateReceiptError> {
    if candidate.len() > super::MAX_SNAPSHOT_FILES || changes.len() > super::MAX_CHANGED_PATHS {
        return Err(super::CandidateReceiptError::SnapshotTooLarge);
    }
    let candidate_entries = entries(candidate)?;
    let mut base_entries = candidate_entries.clone();
    for (path, digest) in changes {
        super::admission::path(path)?;
        match digest {
            Some(digest) => base_entries.insert(path.as_str(), digest.entry()?),
            None => base_entries.remove(path.as_str()),
        };
    }
    validate(&base_entries)?;
    let changed_paths: Vec<String> = changes
        .keys()
        .filter(|path| base_entries.get(path.as_str()) != candidate_entries.get(path.as_str()))
        .cloned()
        .collect();
    if changed_paths.len() != changes.len() {
        return Err(super::CandidateReceiptError::ReceiptMismatch);
    }
    if changed_paths.is_empty() {
        return Err(super::CandidateReceiptError::NoCandidateChanges);
    }
    debug_assert!(changed_paths.is_sorted(), "a map's keys are sorted");
    let tree = |entries: &Entries<'_>| {
        super::snapshots::tree_identity_of(entries.len(), entries.iter().map(|(path, entry)| (*path, *entry)))
    };
    let diff_identity = super::snapshots::diff_identity_of(
        &changed_paths,
        |path| base_entries.get(path).copied(),
        |path| candidate_entries.get(path).copied(),
    );
    let receipt = super::CandidateReceipt {
        schema: super::CANDIDATE_RECEIPT_SCHEMA.to_string(),
        work_id: bindings.work_id.to_string(),
        attempt_id: bindings.attempt_id.to_string(),
        fence: bindings.fence.to_string(),
        workspace_id: bindings.workspace_id.to_string(),
        base_identity: tree(&base_entries),
        candidate_identity: Some(tree(&candidate_entries)),
        diff_identity: Some(diff_identity),
        changed_paths,
        disposition: super::CandidateDisposition::Produced,
        blockers: Vec::new(),
    };
    receipt.validate()?;
    Ok(receipt)
}

fn entries(
    digests: &std::collections::BTreeMap<String, FileDigest>,
) -> Result<Entries<'_>, super::CandidateReceiptError> {
    let entries = digests
        .iter()
        .map(|(path, digest)| {
            super::admission::path(path)?;
            Ok((path.as_str(), digest.entry()?))
        })
        .collect::<Result<Entries<'_>, super::CandidateReceiptError>>()?;
    validate(&entries)?;
    Ok(entries)
}

/// The snapshot bounds of [`super::snapshots::validate_snapshot`].
fn validate(entries: &Entries<'_>) -> Result<(), super::CandidateReceiptError> {
    let total = entries.values().try_fold(0u64, |total, (len, _)| total.checked_add(*len));
    let is_bounded =
        total.is_some_and(|total| usize::try_from(total).is_ok_and(|total| total <= super::MAX_SNAPSHOT_BYTES));
    if entries.len() > super::MAX_SNAPSHOT_FILES || !is_bounded {
        return Err(super::CandidateReceiptError::SnapshotTooLarge);
    }
    Ok(())
}
