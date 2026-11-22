//! A first run's pre-intent section under the per-work lock, and the
//! settling of every refusal decided before a request's own intent.

type Guard = super::super::session::fencing::Guard;
type Pending = super::super::refusal::deferred::Pending;
type Stop = super::super::refusal::deferred::Stop;
type Revisions<'a> = super::super::session::revision::Store<'a>;
type Reconciled = super::super::session::revision::reservation::Reconciled;

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
    pub(super) repair: super::super::repair::admission::Admitted<'a>,
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
    pub(super) fn superseded(&self, revisions: Revisions<'_>) -> Option<String> {
        if !self.is_superseded {
            return None;
        }
        let guard = match self.lock() {
            Ok(guard) => guard,
            Err(message) => return Some(message),
        };
        let pending = Pending::new(super::super::refusal::Reason::ProfileRevisionSuperseded);
        Some(self.settle(&guard, revisions, pending))
    }

    /// A first run's admission, in this order: `run`'s preflight (request
    /// validation, the workspace boundary and the bounded-step timeout), the
    /// per-work lock, the closing of a pin an earlier admission of this work
    /// left without its intent, the intent store, the lineage and repair-port
    /// gates, the fence, and the session behind its write-ahead record. Every
    /// refusal is settled before it returns, and nothing is pinned then.
    // r[impl lattice.implementation_attempt.consumer.scenario.reservation]
    pub(super) fn admit_first<'a>(
        &self,
        repair: Option<super::super::repair::Ports<'a>>,
        operator: Option<super::super::repair::operator::Standalone<'a>>,
        revisions: Revisions<'_>,
    ) -> Result<First<'a>, String> {
        let worker = super::super::session::launch::worker_request(&self.request)?;
        super::super::session::revision::require_named(revisions, &self.request)?;
        let guard = self.lock()?;
        if self.reconcile(revisions)? == Reconciled::OtherWork {
            return Err(self.crossed());
        }
        if self.directory.join(super::super::session::intent::FILE).exists() {
            return Ok(First::Retained(guard));
        }
        match self.gate(&guard, repair, operator, revisions) {
            Ok((repair, session)) => Ok(First::Admitted(Box::new(Ready {
                guard,
                worker,
                repair,
                session,
            }))),
            Err(stop) => Err(self.stop(&guard, revisions, stop)),
        }
    }

    /// Settle the write-ahead record an earlier admission of this work left;
    /// the caller holds the per-work lock.
    fn reconcile(&self, revisions: Revisions<'_>) -> Result<Reconciled, String> {
        super::super::session::revision::reservation::reconcile(revisions, &self.directory, &self.request.envelope)
    }

    /// The refusal of a request whose attempt directory holds another work's
    /// pending admission: no intent, marker or pin is written, and
    /// `dispatched` is whether that directory holds an execution intent.
    fn crossed(&self) -> String {
        super::super::refusal::refuse_crossed(&self.request.envelope, &self.directory).unwrap_or_else(|error| error)
    }

    /// Settle, under `_guard`, a refusal decided before this request's own
    /// intent exists: `dispatched` is whether the attempt directory retains an
    /// execution intent. When it does not, a pin an earlier admission of this
    /// work left is closed first, then the attempt-fence is closed with this
    /// request's closure marker (a closure marker of any request already
    /// closes it), so no later request for it can create an intent in this
    /// state root. Another work's pending admission makes the request
    /// `crossed_attempt` and creates no intent, marker or pin. When closing
    /// fails nothing is printed.
    // r[impl lattice.implementation_attempt.consumer.scenario.closure]
    pub(super) fn settle(&self, _guard: &Guard, revisions: Revisions<'_>, pending: Pending) -> String {
        match self.close(revisions) {
            Ok(Some(dispatched)) => pending.print(&self.request.envelope, dispatched),
            Ok(None) => self.crossed(),
            Err(message) => format!("{} could not close its attempt: {message}", pending.reason().message()),
        }
    }

    /// Whether an execution intent is retained, after reconciling this work's
    /// pending admission and closing the attempt-fence when no intent is;
    /// `None` when another work's pending admission is recorded there. The
    /// caller holds the per-work lock.
    fn close(&self, revisions: Revisions<'_>) -> Result<Option<bool>, String> {
        if self.reconcile(revisions)? == Reconciled::OtherWork {
            return Ok(None);
        }
        let intent = self.directory.join(super::super::session::intent::FILE);
        if super::super::session::intent::is_execution_retained(&self.directory)? {
            return Ok(Some(true));
        }
        let marker = super::super::recovery::closed_marker(&self.identity, &self.request.envelope);
        super::super::artifacts::create_new(&intent, marker.as_bytes())?;
        debug_assert!(matches!(super::super::session::intent::is_execution_retained(&self.directory), Ok(false)));
        debug_assert!(intent.is_file(), "the attempt-fence is closed");
        Ok(Some(false))
    }

    pub(super) fn stop(&self, guard: &Guard, revisions: Revisions<'_>, stop: Stop) -> String {
        match stop {
            Stop::Refused(pending) => self.settle(guard, revisions, pending),
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
        operator: Option<super::super::repair::operator::Standalone<'a>>,
        revisions: Revisions<'_>,
    ) -> Result<(super::super::repair::admission::Admitted<'a>, super::super::session::revision::Admitted), Stop> {
        let repair = super::super::repair::admission::admit(&self.request, repair, operator)?;
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
}

/// A first run lost right after its session was admitted, before its binding
/// and intent: the pin and its write-ahead record stay as a crash leaves them.
#[cfg(test)]
pub(in super::super) fn lost_after_pin(
    request_path: &std::path::Path,
    state: &std::path::Path,
    store: &std::path::Path,
) -> Result<(), String> {
    let context = super::Context::load(request_path, state)?;
    match context.admit_first(None, None, super::super::session::revision::Store(Some(store)))? {
        First::Retained(_) => Err("the attempt directory already holds an intent".into()),
        First::Admitted(_) => Ok(()),
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
        self.0.first_run(ports, None, revisions).await
    }
}
