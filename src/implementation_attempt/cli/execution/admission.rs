//! A first run's pre-intent section under the per-work lock, shared with an
//! attempt-bound `agent-revision admit-session`, and the settling of every
//! refusal decided before a request's own intent.

type Guard = super::super::session::fencing::Guard;
type Pending = super::super::refusal::deferred::Pending;
type Stop = super::super::refusal::deferred::Stop;
type Revisions<'a> = super::super::session::revision::Store<'a>;

/// What a first admission found or did; the per-work lock is still held.
pub(super) enum First<'a> {
    /// The attempt directory holds an intent or closure marker: nothing was
    /// admitted.
    Retained(Guard),
    /// The session is admitted; `run` creates its intent under this lock.
    Admitted(Box<Ready<'a>>),
}

/// An admitted first run, still holding the per-work lock.
pub(super) struct Ready<'a> {
    pub(super) guard: Guard,
    pub(super) worker: crate::candidate_worker::Request,
    pub(super) repair: Option<super::super::repair::Ports<'a>>,
    pub(super) session: super::super::session::revision::Admitted,
}

impl super::Context {
    /// The per-work lock of this request's work, after creating the state
    /// root and the attempt directory.
    pub(super) fn lock(&self) -> Result<Guard, String> {
        self.create_directories()?;
        Guard::acquire(&self.state, &self.request.envelope.work_id)
    }

    /// The refusal of a request retained under an earlier profile framing,
    /// settled under the lock; `None` for a current request.
    pub(super) fn superseded(&self) -> Option<String> {
        if !self.is_superseded {
            return None;
        }
        let guard = match self.lock() {
            Ok(guard) => guard,
            Err(message) => return Some(message),
        };
        Some(self.settle(&guard, Pending::new(super::super::refusal::Reason::ProfileRevisionSuperseded)))
    }

    /// The first-run admission that `run` and attempt-bound `admit-session`
    /// share, in this order: `run`'s preflight (request validation, the
    /// workspace boundary and the bounded-step timeout), the per-work lock,
    /// the closing of a pin an earlier admission left without an intent, the
    /// intent store, the lineage and repair-port gates, the fence, and the
    /// session behind its write-ahead record. Every refusal is settled before
    /// it returns, and nothing is pinned then.
    // r[impl lattice.implementation_attempt.consumer.scenario.reservation]
    pub(super) fn admit_first<'a>(
        &self,
        repair: Option<super::super::repair::Ports<'a>>,
        revisions: Revisions<'_>,
    ) -> Result<First<'a>, String> {
        let worker = super::worker_request(&self.request)?;
        super::super::session::revision::require_named(revisions, &self.request)?;
        let guard = self.lock()?;
        super::super::session::revision::reservation::reconcile(revisions, &self.directory)?;
        if self.directory.join(super::super::session::intent::FILE).exists() {
            return Ok(First::Retained(guard));
        }
        match self.gate(&guard, repair, revisions) {
            Ok((repair, session)) => Ok(First::Admitted(Box::new(Ready {
                guard,
                worker,
                repair,
                session,
            }))),
            Err(stop) => Err(self.stop(&guard, stop)),
        }
    }

    /// Settle, under `_guard`, a refusal decided before this request's own
    /// intent exists: `dispatched` is whether the attempt directory retains an
    /// execution intent. When it does not, the attempt-fence is first closed
    /// with this request's closure marker (a closure marker of any request
    /// already closes it), so no later request for it can create an intent in
    /// this state root. When closing fails nothing is printed.
    // r[impl lattice.implementation_attempt.consumer.scenario.closure]
    pub(super) fn settle(&self, _guard: &Guard, pending: Pending) -> String {
        match self.close() {
            Ok(dispatched) => pending.print(&self.request.envelope, dispatched),
            Err(message) => format!("{} could not close its attempt: {message}", pending.reason().message()),
        }
    }

    /// Whether an execution intent is retained, after closing the
    /// attempt-fence when none is. The caller holds the per-work lock.
    fn close(&self) -> Result<bool, String> {
        let intent = self.directory.join(super::super::session::intent::FILE);
        if super::super::session::intent::is_execution_retained(&self.directory)? {
            return Ok(true);
        }
        let marker = super::super::recovery::closed_marker(&self.identity, &self.request.envelope);
        super::super::artifacts::create_new(&intent, marker.as_bytes())?;
        debug_assert!(matches!(super::super::session::intent::is_execution_retained(&self.directory), Ok(false)));
        debug_assert!(intent.is_file(), "the attempt-fence is closed");
        Ok(false)
    }

    pub(super) fn stop(&self, guard: &Guard, stop: Stop) -> String {
        match stop {
            Stop::Refused(pending) => self.settle(guard, pending),
            Stop::Error(message) => message,
        }
    }

    /// Before any pin: the lineage and repair-port gates, which write
    /// nothing, then the fence check and record; then the revision session.
    // r[impl lattice.implementation_attempt.consumer.scenario.refusal]
    // r[impl lattice.implementation_attempt.lineage.scenario.exhausted]
    fn gate<'a>(
        &self,
        guard: &Guard,
        repair: Option<super::super::repair::Ports<'a>>,
        revisions: Revisions<'_>,
    ) -> Result<(Option<super::super::repair::Ports<'a>>, super::super::session::revision::Admitted), Stop> {
        let repair = super::super::repair::admit(&self.request, repair)?;
        guard.record(&self.request.envelope, &self.directory).map_err(fence_stop)?;
        let session =
            super::super::session::revision::admit(revisions, &self.request, &self.directory, &self.identity)?;
        Ok((repair, session))
    }

    /// After the session: the persisted binding, the candidate store and,
    /// with the fence rechecked under the same lock, the intent, which
    /// supersedes the session's write-ahead record.
    pub(super) fn prepare(
        &self,
        guard: &Guard,
        compute: &mut dyn crate::execution_binding::ComputeAllocationPort,
        session: &super::super::session::revision::Admitted,
    ) -> Result<(lattice_core::execution_binding::admission::bound::Admitted, crate::store::Store), Stop> {
        let admitted = super::super::binding::admit_and_persist(&self.request, &self.directory, compute)?;
        let store =
            crate::store::Store::new(self.directory.join("candidate.redb")).map_err(|error| error.to_string())?;
        let identifier = super::super::session::revision::identifier(&self.request);
        let subject = super::super::session::intent::Subject {
            request_identity: &self.identity,
            session: &identifier,
        };
        let bytes = super::super::session::intent::encode(subject, session.recorded.as_ref())?;
        // The intent store held no intent under this lock, so no other run
        // can have created one since.
        if !guard.create_intent(&self.request.envelope, &self.directory, &bytes).map_err(fence_stop)? {
            return Err(Stop::Error("execution intent appeared under the fence lock".into()));
        }
        debug_assert!(self.directory.join(super::super::session::intent::FILE).is_file());
        // The intent owns the pin from here on; a record that cannot be
        // removed now is superseded and removed by the next reconciliation.
        let _superseded = super::super::session::revision::reservation::release(&self.directory);
        Ok((admitted, store))
    }

    /// `admit-session`'s refusal when the attempt directory already holds an
    /// intent or closure marker: like `run`'s reconciliation, but never
    /// recovering or pinning.
    fn refuse_retained(&self, guard: &Guard) -> String {
        let session = super::super::session::revision::identifier(&self.request);
        let subject = super::super::session::intent::Subject {
            request_identity: &self.identity,
            session: &session,
        };
        let retained = super::super::session::intent::read(&self.directory, subject);
        match retained {
            Ok(Some(super::super::session::intent::Record::Owned(_))) => {
                "the attempt already holds its execution intent; nothing was pinned".into()
            }
            Ok(Some(super::super::session::intent::Record::Closed)) => {
                super::super::refusal::refuse(&self.request.envelope, super::super::refusal::Reason::AttemptClosed)
            }
            Ok(Some(super::super::session::intent::Record::Foreign) | None) => {
                super::super::recovery::refuse_retained(guard, &self.state, &self.request, &self.directory)
                    .unwrap_or_else(|message| message)
            }
            Err(message) => message,
        }
    }
}

/// `agent-revision admit-session --state`: `run`'s first-run admission of the
/// request's attempt session, stopping once the session is admitted. A
/// refusal is settled exactly as `run` settles it and pins nothing. The pin
/// and its write-ahead record stay until a run of the attempt-fence creates
/// its intent or a run or recovery closes the pin.
pub(crate) fn admit_session(
    request_path: &std::path::Path,
    state: &std::path::Path,
    store: &std::path::Path,
) -> Result<crate::agent_revision::seam::Decision, String> {
    let context = super::Context::load(request_path, state)?;
    if let Some(refused) = context.superseded() {
        return Err(refused);
    }
    match context.admit_first(None, super::super::session::revision::Store(Some(store)))? {
        First::Retained(guard) => Err(context.refuse_retained(&guard)),
        First::Admitted(ready) => Ok(ready.session.decision),
    }
}

/// A fence refusal before the intent is settled like every other refusal.
fn fence_stop(error: super::super::session::fencing::Error) -> Stop {
    match error {
        super::super::session::fencing::Error::Refused(refused) => Stop::Refused(Pending::new(refused.reason)),
        super::super::session::fencing::Error::State(message) => Stop::Error(message),
    }
}

/// A request loaded by a first run that has passed its no-intent check, as a
/// run paused there holds it.
#[cfg(test)]
pub(in super::super) struct Paused(super::Context);

#[cfg(test)]
impl Paused {
    pub(in super::super) fn load(request_path: &std::path::Path, state: &std::path::Path) -> Result<Self, String> {
        let context = super::Context::load(request_path, state)?;
        assert!(!context.is_superseded && !context.directory.join(super::super::session::intent::FILE).exists());
        Ok(Self(context))
    }

    pub(in super::super) async fn resume(
        self,
        ports: super::super::Ports<'_>,
        revisions: super::super::session::revision::Store<'_>,
    ) -> Result<(), String> {
        self.0.first_run(ports, revisions).await
    }
}
