//! The write-ahead record of one agent revision admission before its intent.
//!
//! Under the per-work lock and before `seam::admit` can pin, an admission
//! with a revision store creates `admission-pending.json` in the attempt
//! directory: the BLAKE3 of its work, the request identity, the session and
//! the lifecycle subject it is about to pin. The record is removed once the
//! intent exists, once the admission is refused or its pin is abandoned, and
//! at once when the pin predates it. Every later run, recovery or closure of
//! the attempt-fence by the same work reconciles the record under the work's
//! lock before anything else: unless the record's own request owns the
//! execution intent, it closes exactly the recorded subject's session through
//! the seam, never a session pinned to another revision, and only then
//! removes the record. A request of another work leaves the record untouched.

// r[impl lattice.implementation_attempt.consumer.scenario.reservation]

pub(in super::super::super) const FILE: &str = "admission-pending.json";
const SCHEMA: &str = "lattice.implementation-attempt-pending-admission.v1";
const MAX_RECORD_BYTES: usize = 4_096;

type Request = super::Request;
type Envelope = crate::implementation_attempt::ImplementationAttemptEnvelope;
type Closure = crate::agent_revision::seam::Closure;
type Refusal = lattice_core::agent_revision::Refusal;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: String,
    /// The BLAKE3 of the admitted request's work id.
    work: String,
    request_identity: String,
    session: String,
    definition: String,
    revision: String,
}

/// What reconciling the attempt directory's record left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super::super) enum Reconciled {
    /// No record of this work remains.
    Clear,
    /// The record belongs to another work and was left untouched: this
    /// request is crossed.
    OtherWork,
}

fn work(envelope: &Envelope) -> String {
    blake3::hash(envelope.work_id.as_bytes()).to_hex().to_string()
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
        work: work(&request.envelope),
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

/// Settle a record an earlier admission of `envelope`'s work left, before
/// anything else of the attempt-fence and before any closure marker; the
/// caller holds that work's lock. A record of another work is left untouched
/// and answers [`Reconciled::OtherWork`]. Otherwise, unless the record's own
/// request owns the execution intent (which then owns the pin), the session
/// of exactly the recorded subject is closed first; the record is removed
/// only after that.
pub(in super::super::super) fn reconcile(
    store: super::Store<'_>,
    directory: &std::path::Path,
    envelope: &Envelope,
) -> Result<Reconciled, String> {
    let path = directory.join(FILE);
    let Some(bytes) = super::super::super::artifacts::read_optional(&path, MAX_RECORD_BYTES)? else {
        return Ok(Reconciled::Clear);
    };
    let record: Record =
        serde_json::from_slice(&bytes).map_err(|_| "pending agent revision admission is malformed".to_owned())?;
    let fields = [
        &record.work,
        &record.request_identity,
        &record.session,
        &record.definition,
        &record.revision,
    ];
    if record.schema != SCHEMA || fields.iter().any(|field| field.is_empty()) {
        return Err("pending agent revision admission is malformed".into());
    }
    if record.work != work(envelope) {
        return Ok(Reconciled::OtherWork);
    }
    let owner = super::super::intent::Subject {
        request_identity: &record.request_identity,
        session: &record.session,
    };
    // A closure marker, another request's intent or no intent leaves the pin
    // to this record alone.
    let is_owned =
        matches!(super::super::intent::read(directory, owner)?, Some(super::super::intent::Record::Owned(_)));
    if !is_owned {
        close(store, &record)?;
    }
    release(directory)?;
    debug_assert!(!path.exists());
    Ok(Reconciled::Clear)
}

/// Close the session of exactly the recorded subject; a session already
/// gone, closed or pinned to another revision is settled.
fn close(store: super::Store<'_>, record: &Record) -> Result<(), String> {
    let Some(store_path) = store.0 else {
        return Err("a pending agent revision admission requires its --revision-store".into());
    };
    // `reconcile` refused a record with an empty field as malformed.
    debug_assert_eq!(record.schema, SCHEMA);
    debug_assert!(!record.session.is_empty() && !record.definition.is_empty() && !record.revision.is_empty());
    let subject = crate::agent_revision::seam::Subject {
        definition: record.definition.clone(),
        revision: record.revision.clone(),
    };
    let opened = super::Store::open(store_path)?;
    match crate::agent_revision::seam::close_subject(&opened, &subject, &record.session, &[])? {
        Closure::Closed { .. }
        | Closure::Unmanaged
        | Closure::Refused {
            refusal: Refusal::UnknownSession | Refusal::SessionClosed | Refusal::SessionConflict,
        } => Ok(()),
        Closure::Refused { refusal } => {
            Err(format!("pending agent revision admission was not closed: {}", serde_json::json!(refusal)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Reconciled;

    fn record(work: &str, session: &str, schema: &str) -> Vec<u8> {
        let record = super::Record {
            schema: schema.to_owned(),
            work: work.to_owned(),
            request_identity: "1".repeat(64),
            session: session.to_owned(),
            definition: "3".repeat(64),
            revision: "4".repeat(64),
        };
        serde_json::to_vec(&record).expect("record bytes")
    }

    #[test]
    // r[verify lattice.implementation_attempt.consumer.scenario.reservation]
    fn a_record_is_read_only_by_its_own_work_and_only_when_complete() {
        let scratch = tempfile::tempdir().expect("scratch");
        let directory = scratch.path();
        let path = directory.join(super::FILE);
        let envelope = super::super::super::super::tests::host_local().envelope;
        let unmanaged = super::super::Store(None);
        let reconcile = || super::reconcile(unmanaged, directory, &envelope);
        assert_eq!(reconcile(), Ok(Reconciled::Clear), "no record");

        // Another work's record is left untouched, even without a store.
        let other = record(&"0".repeat(64), &"2".repeat(64), super::SCHEMA);
        std::fs::write(&path, &other).expect("record");
        assert_eq!(reconcile(), Ok(Reconciled::OtherWork));
        assert_eq!(std::fs::read(&path).expect("kept"), other);

        // This work's record needs its store, and is kept without it.
        let own = record(&super::work(&envelope), &"2".repeat(64), super::SCHEMA);
        std::fs::write(&path, &own).expect("record");
        assert!(reconcile().expect_err("no store").contains("--revision-store"));
        assert_eq!(std::fs::read(&path).expect("kept"), own);

        // An empty field or another schema is malformed and kept.
        for malformed in [
            record(&super::work(&envelope), "", super::SCHEMA),
            record("", &"2".repeat(64), super::SCHEMA),
            record(&super::work(&envelope), &"2".repeat(64), "another-schema"),
        ] {
            std::fs::write(&path, &malformed).expect("record");
            assert_eq!(reconcile(), Err("pending agent revision admission is malformed".to_owned()));
            assert_eq!(std::fs::read(&path).expect("kept"), malformed);
        }
    }
}
