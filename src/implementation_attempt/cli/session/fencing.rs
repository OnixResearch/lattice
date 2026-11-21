//! Per-work fence records and the per-work lock of one attempt state root.
//!
//! `works/<blake3(work_id)>/<fence>` names the one attempt admitted at that
//! fence of that work. Every write that can start or preclude a worker of
//! that work in this state root is made while this process holds the
//! exclusive lock on the `works/<blake3(work_id)>` directory: recording a
//! fence, admitting the attempt's agent revision session, creating an
//! attempt's execution intent, creating a repair step's intent, and the
//! closure marker that recovery or a refusal before the intent writes. A first
//! run reads the intent store under the lock before anything else, so it
//! never admits an attempt another request already holds. An execution
//! intent and every step intent are created only after a recheck under that
//! lock finds this attempt recorded at its fence and no higher fence
//! recorded. So once a higher fence or another attempt is recorded, or the
//! attempt-fence is closed, no attempt it supersedes can create an intent or
//! spawn a step in this state root, and the `dispatched` of a refusal, read
//! from the intent store under the same lock, stays true of every later run.
//! The agent revision store may be opened while holding this lock, but no
//! path takes this lock while holding the store open.
//!
//! The lock is `flock(2)` through [`std::fs::File::lock`]: waiters block, it
//! never expires, and the kernel releases it when its holder closes the
//! handle or exits. Kiln remains the fence authority; these records only stop
//! this state root from executing a superseded or crossed request. Recovery
//! takes the lock but never writes a fence record.

// r[impl lattice.implementation_attempt.consumer.scenario.refusal]
// r[impl lattice.implementation_attempt.consumer.scenario.fence_lock]

const DIRECTORY: &str = "works";
const FENCE_DIGITS: usize = 20;
const MAX_FENCES: usize = 4_096;

type Envelope = crate::implementation_attempt::ImplementationAttemptEnvelope;
type Reason = super::super::refusal::Reason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Decision {
    /// No attempt is recorded at this fence; record this one.
    Record,
    /// This exact attempt is already recorded at this fence.
    Replay,
}

/// Decide one run from the highest recorded fence and the attempt recorded at
/// this request's fence.
pub(super) fn decide(
    highest: Option<u64>,
    recorded_at_fence: Option<&str>,
    fence: u64,
    attempt_id: &str,
) -> Result<Decision, Reason> {
    if highest.is_some_and(|recorded| recorded > fence) {
        return Err(Reason::StaleFence);
    }
    match recorded_at_fence {
        None => Ok(Decision::Record),
        Some(recorded) if recorded == attempt_id => Ok(Decision::Replay),
        Some(_) => Err(Reason::CrossedAttempt),
    }
}

/// A fence refusal decided under the lock. `dispatched` is whether the
/// refused attempt's directory held an execution intent under that lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct Refused {
    pub(in super::super) reason: Reason,
    pub(in super::super) dispatched: bool,
}

/// Why a fenced step did not proceed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) enum Error {
    /// A typed refusal; nothing was written for the request.
    Refused(Refused),
    /// The fence records or the intent store failed; nothing was decided.
    State(String),
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::State(message)
    }
}

impl Error {
    /// Print a refusal record, then return the diagnostic.
    pub(in super::super) fn report(self, envelope: &Envelope) -> String {
        match self {
            Self::Refused(refused) => super::super::refusal::refuse_fence(envelope, refused.reason, refused.dispatched),
            Self::State(message) => message,
        }
    }
}

/// The exclusive lock on one work's fence records in one state root, held
/// until dropped.
#[derive(Debug)]
pub(in super::super) struct Guard {
    directory: std::path::PathBuf,
    _lock: std::fs::File,
}

impl Guard {
    /// Block until this process holds the lock of `work_id` in `state`.
    pub(in super::super) fn acquire(state: &std::path::Path, work_id: &str) -> Result<Self, String> {
        super::super::artifacts::private_directory(&state.join(DIRECTORY))?;
        let directory = work_directory(state, work_id);
        super::super::artifacts::private_directory(&directory)?;
        let lock = std::fs::File::open(&directory).map_err(|_| "fence records cannot be opened")?;
        lock.lock().map_err(|_| "fence records cannot be locked")?;
        Ok(Self { directory, _lock: lock })
    }

    fn decide(&self, envelope: &Envelope) -> Result<Result<Decision, Reason>, String> {
        let recorded = recorded_at(&self.directory, envelope.fence)?;
        Ok(decide(highest_fence(&self.directory)?, recorded.as_deref(), envelope.fence, &envelope.attempt_id))
    }

    /// The refusal of the attempt whose directory is `attempt`, with
    /// `dispatched` read from its intent store under this lock.
    fn refused(reason: Reason, attempt: &std::path::Path) -> Error {
        match super::intent::is_execution_retained(attempt) {
            Ok(dispatched) => Error::Refused(Refused { reason, dispatched }),
            Err(message) => Error::State(message),
        }
    }

    /// Record this attempt at its fence, or refuse a stale or crossed
    /// request. `attempt` is the request's attempt directory.
    pub(in super::super) fn record(&self, envelope: &Envelope, attempt: &std::path::Path) -> Result<(), Error> {
        let decision = self.decide(envelope)?.map_err(|reason| Self::refused(reason, attempt))?;
        let record = self.directory.join(fence_name(envelope.fence));
        if decision == Decision::Record
            && !super::super::artifacts::create_new(&record, envelope.attempt_id.as_bytes())?
        {
            return Err(Error::State("fence record appeared while locked".into()));
        }
        // A replay found this exact attempt at its fence; either way the record
        // now names this attempt.
        debug_assert!(record.is_file());
        Ok(())
    }

    /// Whether this attempt is recorded at its fence and no higher fence of
    /// its work is recorded.
    pub(in super::super) fn is_current(&self, envelope: &Envelope) -> Result<bool, String> {
        Ok(self.decide(envelope)? == Ok(Decision::Replay))
    }

    /// Recheck the fence, then create the execution intent `bytes` once in
    /// `attempt`; `false` when the identical intent already exists.
    pub(in super::super) fn create_intent(
        &self,
        envelope: &Envelope,
        attempt: &std::path::Path,
        bytes: &[u8],
    ) -> Result<bool, Error> {
        match self.decide(envelope)? {
            Ok(Decision::Replay) => {}
            Ok(Decision::Record) => {
                return Err(Error::State("execution intent has no fence record of its attempt".into()));
            }
            Err(reason) => return Err(Self::refused(reason, attempt)),
        }
        debug_assert!(self.directory.join(fence_name(envelope.fence)).is_file());
        Ok(super::super::artifacts::write_once(&attempt.join(super::intent::FILE), bytes)?)
    }

    /// Recovery: refuse when another attempt is recorded at this fence.
    pub(in super::super) fn check_crossed(&self, envelope: &Envelope, attempt: &std::path::Path) -> Result<(), Error> {
        let recorded = recorded_at(&self.directory, envelope.fence)?;
        if recorded.is_some_and(|recorded| recorded != envelope.attempt_id) {
            return Err(Self::refused(Reason::CrossedAttempt, attempt));
        }
        Ok(())
    }
}

/// Take the lock and record this attempt at its fence, or print a stale or
/// crossed refusal, as a run records its fence; tests record fences with it.
#[cfg(test)]
pub(in super::super) fn admit(
    state: &std::path::Path,
    envelope: &Envelope,
    attempt: &std::path::Path,
) -> Result<Guard, String> {
    let guard = Guard::acquire(state, &envelope.work_id)?;
    guard.record(envelope, attempt).map_err(|error| error.report(envelope))?;
    Ok(guard)
}

fn work_directory(state: &std::path::Path, work_id: &str) -> std::path::PathBuf {
    state.join(DIRECTORY).join(blake3::hash(work_id.as_bytes()).to_hex().as_str())
}

fn fence_name(fence: u64) -> String {
    format!("{fence:0FENCE_DIGITS$}")
}

fn recorded_at(directory: &std::path::Path, fence: u64) -> Result<Option<String>, String> {
    let bytes = super::super::artifacts::read_optional(
        &directory.join(fence_name(fence)),
        crate::implementation_attempt::MAX_IDENTIFIER_BYTES,
    )?;
    bytes
        .map(|value| String::from_utf8(value).map_err(|_| "retained fence record is malformed".to_string()))
        .transpose()
}

fn highest_fence(directory: &std::path::Path) -> Result<Option<u64>, String> {
    let entries = std::fs::read_dir(directory).map_err(|_| "retained fence records are unreadable")?;
    let mut highest = None;
    let mut count: usize = 0;
    for entry in entries {
        count = count.saturating_add(1);
        if count > MAX_FENCES {
            return Err("retained fence records exceed their bound".into());
        }
        let name = entry.map_err(|_| "retained fence records are unreadable")?.file_name();
        let name = name.to_str().ok_or("retained fence record name is malformed")?;
        if name.len() != FENCE_DIGITS || !name.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err("retained fence record name is malformed".into());
        }
        let fence = name.parse::<u64>().map_err(|_| "retained fence record name is malformed")?;
        highest = highest.max(Some(fence));
    }
    Ok(highest)
}

#[cfg(test)]
mod tests;
