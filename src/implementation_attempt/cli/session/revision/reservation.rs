//! The write-ahead record of one agent revision admission before its intent.
//!
//! Under the per-work lock and before `seam::admit` can pin, an admission
//! with a revision store creates `admission-pending.json` in the attempt
//! directory: the request identity, the session and the lifecycle subject it
//! is about to pin. The record is removed once the intent exists, once the
//! admission is refused or its pin is abandoned, and at once when the pin
//! predates it. A later run or recovery of the attempt-fence that finds the
//! record without an intent closes exactly that subject's session through
//! the seam before anything else; a session pinned to another revision is
//! never touched. An intent supersedes the record.

// r[impl lattice.implementation_attempt.consumer.scenario.reservation]

pub(in super::super::super) const FILE: &str = "admission-pending.json";
const SCHEMA: &str = "lattice.implementation-attempt-pending-admission.v1";
const MAX_RECORD_BYTES: usize = 4_096;

type Request = super::Request;
type Closure = crate::agent_revision::seam::Closure;
type Refusal = lattice_core::agent_revision::Refusal;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: String,
    request_identity: String,
    session: String,
    definition: String,
    revision: String,
}

/// Create the record of `request`'s admission before it can pin. The caller
/// holds the per-work lock and has reconciled any earlier record.
pub(in super::super::super) fn reserve(
    directory: &std::path::Path,
    request: &Request,
    request_identity: &str,
) -> Result<(), String> {
    let subject = crate::agent_revision::seam::subject(request)?;
    let record = Record {
        schema: SCHEMA.to_owned(),
        request_identity: request_identity.to_owned(),
        session: super::identifier(request),
        definition: subject.definition,
        revision: subject.revision,
    };
    let bytes = serde_json::to_vec(&record).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_RECORD_BYTES {
        return Err("pending agent revision admission exceeds its byte bound".into());
    }
    let path = directory.join(FILE);
    if !super::super::super::artifacts::create_new(&path, &bytes)? {
        return Err("a pending agent revision admission is already recorded".into());
    }
    debug_assert!(path.is_file());
    Ok(())
}

/// Remove the record: its pin is owned by the intent, abandoned, refused or
/// not this admission's.
pub(in super::super::super) fn release(directory: &std::path::Path) -> Result<(), String> {
    match std::fs::remove_file(directory.join(FILE)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err("pending agent revision admission cannot be removed".into()),
    }
}

/// Settle a record an earlier admission left: without an intent, close the
/// session of exactly the recorded subject, then remove the record. The
/// caller holds the per-work lock, before anything else of the attempt-fence.
pub(in super::super::super) fn reconcile(store: super::Store<'_>, directory: &std::path::Path) -> Result<(), String> {
    let path = directory.join(FILE);
    let Some(bytes) = super::super::super::artifacts::read_optional(&path, MAX_RECORD_BYTES)? else {
        return Ok(());
    };
    let record: Record =
        serde_json::from_slice(&bytes).map_err(|_| "pending agent revision admission is malformed".to_owned())?;
    if record.schema != SCHEMA {
        return Err("pending agent revision admission is malformed".into());
    }
    if !directory.join(super::super::intent::FILE).exists() {
        let Some(store_path) = store.0 else {
            return Err("a pending agent revision admission requires its --revision-store".into());
        };
        let subject = crate::agent_revision::seam::Subject {
            definition: record.definition,
            revision: record.revision,
        };
        let opened = super::Store::open(store_path)?;
        match crate::agent_revision::seam::close_subject(&opened, &subject, &record.session, &[])? {
            Closure::Closed { .. }
            | Closure::Unmanaged
            | Closure::Refused {
                refusal: Refusal::UnknownSession | Refusal::SessionClosed | Refusal::SessionConflict,
            } => {}
            Closure::Refused { refusal } => {
                return Err(format!("pending agent revision admission was not closed: {}", serde_json::json!(refusal)));
            }
        }
    }
    release(directory)?;
    debug_assert!(!path.exists());
    Ok(())
}
