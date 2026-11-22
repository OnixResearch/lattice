//! Export of a repair attempt's passing candidate revision.
//!
//! A repair attempt has no native attempt result. After the same admission as
//! a single-shot export (the request, its intent, the attempt-fence directory
//! and the revision seam), its export reads the complete, request-bound
//! retained records (`cli::repair::evidence`) and replays them in the pure
//! core (`repair::exportable`). The exported tree is the workspace, which must
//! still capture to the named revision's candidate with the execute bits its
//! checks' copies were sealed with, and from which, with the retained base
//! digests of the changed paths, the whole receipt must re-derive.
//! `host-receipt.json` is that receipt and `native-observation.json` the
//! retained disposition. A missing disposition stays an unknown outcome; any
//! other unexportable record set is a typed refusal.

// r[impl lattice.implementation_attempt.repair_export]
// r[impl lattice.implementation_attempt.repair_export.scenario.passing]
// r[impl lattice.implementation_attempt.repair_export.scenario.refused]

type Reason = super::super::refusal::Reason;
type Record = crate::implementation_attempt::repair::revision::Record;
type Unexportable = crate::implementation_attempt::repair::exportable::Unexportable;
type Unretained = super::super::repair::Unretained;

pub(super) fn load(inputs: &super::Admitted) -> Result<(super::content::Payload, serde_json::Value), String> {
    let request = &inputs.request;
    let directory = &inputs.directory;
    let admitted = super::super::binding::load(directory, request)?
        .ok_or("retained attempt has no persisted execution binding")?;
    if super::super::binding::refusal(directory)?.is_some()
        || super::super::session::revision::retained_block(directory)?.is_some()
    {
        return Err("repair attempt was not dispatched; it has no candidate".into());
    }
    let refuse = |reason| super::super::refusal::refuse(&request.envelope, reason);
    let evidence =
        super::super::repair::evidence(request, directory, &inputs.identity, &admitted).map_err(|unretained| {
            match unretained {
                Unretained::Missing => refuse(Reason::RepairOutcomeUnknown),
                Unretained::Legacy | Unretained::Malformed | Unretained::Crossed => {
                    refuse(Reason::RepairDispositionInvalid)
                }
            }
        })?;
    let exported = super::super::repair::with_frame(request, |frame| {
        let record = crate::implementation_attempt::repair::exportable::candidate(frame, evidence.view()).map_err(
            |unexportable| match unexportable {
                Unexportable::NotCandidate => refuse(Reason::RepairNotCandidate),
                Unexportable::Contradicted => refuse(Reason::RepairDispositionInvalid),
            },
        )?;
        debug_assert!(record.receipt.is_usable_candidate(), "only a produced receipt is exported");
        debug_assert_eq!(Some(record), evidence.observation.revisions.last(), "the final revision");
        payload(inputs, frame, record)
    });
    let payload = exported.ok_or("repair export requires an admitted repair policy")??;
    Ok((payload, evidence.retained))
}

/// The workspace as the exact checked candidate of `record`: the same bytes,
/// the same execute bits, and a receipt that re-derives entirely from them
/// and the retained base digests of the changed paths.
fn payload(
    inputs: &super::Admitted,
    frame: &crate::implementation_attempt::repair::exportable::Frame<'_>,
    record: &Record,
) -> Result<super::content::Payload, String> {
    let snapshot = crate::candidate_capture::workspace(&inputs.workspace).map_err(|error| error.to_string())?;
    let modes =
        crate::candidate_capture::snapshot_modes(&inputs.workspace, &snapshot).map_err(|error| error.to_string())?;
    if crate::implementation_attempt::repair::revision::executables(&modes) != record.executables {
        return Err("workspace file modes differ from the checked candidate revision".into());
    }
    let digests = snapshot
        .iter()
        .map(|(path, bytes)| (path.clone(), crate::candidate::digests::FileDigest::of(bytes)))
        .collect();
    let derived = crate::candidate::digests::produced_receipt_from_changes(frame.receipt, &digests, &record.base_files);
    if derived.as_ref().ok() != Some(&record.receipt) || record.receipt.base_identity != frame.base_identity {
        return Err("workspace differs from the exact retained candidate revision".into());
    }
    debug_assert_eq!(snapshot.len(), modes.len(), "every captured file has its mode");
    debug_assert!(record.receipt.changed_paths.iter().all(|path| record.base_files.contains_key(path)));
    let receipt = serde_json::to_vec_pretty(&record.receipt).map_err(|error| error.to_string())?;
    Ok(super::content::Payload {
        snapshot,
        modes,
        receipt,
    })
}
