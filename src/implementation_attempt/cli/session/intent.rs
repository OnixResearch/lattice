//! The one-shot execution intent of one attempt directory.
//!
//! Without a revision store the intent is the request identity's bytes, as it
//! was before admission modes were recorded. With one, it is a
//! `lattice.implementation-attempt-intent.v1` record of the request identity,
//! the session and the admission mode `seam::admit` decided, with the pin of a
//! managed session, written in the same exclusive write. Every reader takes
//! the mode back from here and never derives it from the current lifecycle.
//! An intent without an admission field was admitted unmanaged by
//! construction.

// r[impl lattice.implementation_attempt.consumer.scenario.admission_mode]

pub(in super::super) const FILE: &str = "intent.json";
pub(in super::super) const SCHEMA: &str = "lattice.implementation-attempt-intent.v1";
const MAX_INTENT_BYTES: usize = 4_096;

type Admission = crate::agent_revision::seam::Admission;

/// The pin `seam::admit` returned for a managed session.
pub(in super::super) type Pin = lattice_core::agent_revision::projection::RecordedPin;

/// The admission recorded with one intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct Recorded {
    pub admission: Admission,
    /// Present exactly for a managed session.
    pub pin: Option<Pin>,
}

impl Recorded {
    /// Unmanaged: no admission field, or recorded as `unmanaged`.
    pub(in super::super) const UNMANAGED: Self = Self {
        admission: Admission::Unmanaged,
        pin: None,
    };

    /// The recorded admission as inventory reads it.
    pub(in super::super) const fn inventory_admission(
        &self,
    ) -> lattice_core::execution_binding::projection::lineage::RecordedAdmission<'_> {
        match &self.pin {
            Some(pin) => lattice_core::execution_binding::projection::lineage::RecordedAdmission::Managed(pin),
            None => lattice_core::execution_binding::projection::lineage::RecordedAdmission::Unmanaged,
        }
    }
}

/// The request and session one intent is written or read for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct Subject<'a> {
    pub request_identity: &'a str,
    pub session: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    schema: String,
    request_identity: String,
    session: String,
    admission: Admission,
    pin: Option<Pin>,
}

/// What one attempt directory's intent says about this request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum Record {
    /// This exact request owns the execution intent.
    Owned(Recorded),
    /// Recovery closed the attempt before any execution.
    Closed,
    /// The intent belongs to different request bytes or is malformed.
    Foreign,
}

/// The intent bytes of `subject`: its bare request identity without a
/// recorded admission, otherwise the canonical record.
pub(in super::super) fn encode(subject: Subject<'_>, recorded: Option<&Recorded>) -> Result<Vec<u8>, String> {
    let Some(recorded) = recorded else {
        return Ok(subject.request_identity.as_bytes().to_vec());
    };
    debug_assert_eq!(recorded.admission == Admission::Managed, recorded.pin.is_some());
    let stored = Stored {
        schema: SCHEMA.to_owned(),
        request_identity: subject.request_identity.to_owned(),
        session: subject.session.to_owned(),
        admission: recorded.admission,
        pin: recorded.pin.clone(),
    };
    let bytes = serde_json::to_vec(&stored).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_INTENT_BYTES {
        return Err("execution intent exceeds its byte bound".into());
    }
    Ok(bytes)
}

/// Classify retained intent bytes for this request and session.
pub(in super::super) fn classify(bytes: &[u8], subject: Subject<'_>) -> Record {
    let marker = super::super::recovery::closed_marker(subject.request_identity);
    // The bare identity and the closure marker never collide, so neither
    // check below can shadow the other.
    debug_assert_ne!(marker.as_bytes(), subject.request_identity.as_bytes());
    if bytes == subject.request_identity.as_bytes() {
        return Record::Owned(Recorded::UNMANAGED);
    }
    if bytes == marker.as_bytes() {
        return Record::Closed;
    }
    let Ok(stored) = serde_json::from_slice::<Stored>(bytes) else {
        return Record::Foreign;
    };
    let recorded = Recorded {
        admission: stored.admission,
        pin: stored.pin,
    };
    let is_consistent = stored.schema == SCHEMA
        && stored.request_identity == subject.request_identity
        && stored.session == subject.session
        && (recorded.admission == Admission::Managed) == recorded.pin.is_some();
    let is_canonical = is_consistent && encode(subject, Some(&recorded)).is_ok_and(|canonical| canonical == bytes);
    if !is_canonical {
        return Record::Foreign;
    }
    // An owned intent carries a pin exactly when it was admitted managed.
    debug_assert_eq!(recorded.admission == Admission::Managed, recorded.pin.is_some());
    Record::Owned(recorded)
}

/// Read the retained intent; `None` when no intent exists.
pub(in super::super) fn read(directory: &std::path::Path, subject: Subject<'_>) -> Result<Option<Record>, String> {
    let bytes = super::super::artifacts::read_optional(&directory.join(FILE), MAX_INTENT_BYTES)?;
    Ok(bytes.map(|bytes| classify(&bytes, subject)))
}

/// Whether `directory` retains an execution intent: any intent other than a
/// recovery closure marker, including one of other request bytes.
pub(in super::super) fn is_execution_retained(directory: &std::path::Path) -> Result<bool, String> {
    let bytes = super::super::artifacts::read_optional(&directory.join(FILE), MAX_INTENT_BYTES)?;
    Ok(bytes.is_some_and(|bytes| !super::super::recovery::is_closed_marker(&bytes)))
}

#[cfg(test)]
mod tests {
    // r[verify lattice.implementation_attempt.consumer.scenario.admission_mode]
    use super::Admission;
    use super::Record;
    use super::Recorded;
    use super::Subject;

    const IDENTITY: &str = "1111111111111111111111111111111111111111111111111111111111111111";
    const SESSION: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const SUBJECT: Subject<'static> = Subject {
        request_identity: IDENTITY,
        session: SESSION,
    };

    fn managed() -> Recorded {
        Recorded {
            admission: Admission::Managed,
            pin: Some(super::Pin {
                revision: "3".repeat(64),
                qualification_ref: format!("blake3:{}", "4".repeat(64)),
                rollout_generation: 1,
                pin_identity: format!("blake3:{}", "5".repeat(64)),
            }),
        }
    }

    #[test]
    fn intents_record_and_read_back_their_admission_mode() {
        let legacy = super::encode(SUBJECT, None).expect("legacy");
        assert_eq!(legacy, IDENTITY.as_bytes(), "without a store the intent is unchanged");
        assert_eq!(super::classify(&legacy, SUBJECT), Record::Owned(Recorded::UNMANAGED));
        for recorded in [managed(), Recorded::UNMANAGED] {
            let bytes = super::encode(SUBJECT, Some(&recorded)).expect("record");
            assert_eq!(super::classify(&bytes, SUBJECT), Record::Owned(recorded));
        }
        let marker = super::super::super::recovery::closed_marker(IDENTITY);
        assert_eq!(super::classify(marker.as_bytes(), SUBJECT), Record::Closed);
    }

    #[test]
    fn crossed_inconsistent_or_non_canonical_intents_are_foreign() {
        let bytes = super::encode(SUBJECT, Some(&managed())).expect("record");
        let other = "9".repeat(64);
        let other_request = Subject {
            request_identity: &other,
            ..SUBJECT
        };
        assert_eq!(super::classify(&bytes, other_request), Record::Foreign, "other request");
        let other_session = Subject {
            session: &other,
            ..SUBJECT
        };
        assert_eq!(super::classify(&bytes, other_session), Record::Foreign, "other session");
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        value["pin"] = serde_json::Value::Null;
        let unpinned = serde_json::to_vec(&value).expect("bytes");
        assert_eq!(super::classify(&unpinned, SUBJECT), Record::Foreign, "managed without a pin");
        let pretty = serde_json::to_vec_pretty(&serde_json::from_slice::<serde_json::Value>(&bytes).expect("json"))
            .expect("pretty");
        assert_eq!(super::classify(&pretty, SUBJECT), Record::Foreign, "not canonical");
        let mut extra: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        extra["admission_hint"] = serde_json::Value::from("managed");
        let extra = serde_json::to_vec(&extra).expect("bytes");
        assert_eq!(super::classify(&extra, SUBJECT), Record::Foreign, "unknown fields");
        assert_eq!(super::classify(b"other", SUBJECT), Record::Foreign);
    }
}
