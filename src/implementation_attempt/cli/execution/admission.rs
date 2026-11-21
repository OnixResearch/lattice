//! A first run's pre-intent section under the per-work lock, and the
//! settling of every refusal decided before this request's own intent.

type Guard = super::super::session::fencing::Guard;
type Pending = super::super::refusal::deferred::Pending;
type Stop = super::super::refusal::deferred::Stop;

impl super::Context {
    /// The per-work lock of this request's work, after creating the state
    /// root and the attempt directory.
    pub(super) fn lock(&self) -> Result<Guard, String> {
        self.create_directories()?;
        Guard::acquire(&self.state, &self.request.envelope.work_id)
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
        let marker = super::super::recovery::closed_marker(&self.identity);
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
    pub(super) fn gate<'a>(
        &self,
        guard: &Guard,
        repair: Option<super::super::repair::Ports<'a>>,
        revisions: super::super::session::revision::Store<'_>,
    ) -> Result<(Option<super::super::repair::Ports<'a>>, super::super::session::revision::Admitted), Stop> {
        let repair = super::super::repair::admit(&self.request, repair)?;
        guard.record(&self.request.envelope, &self.directory).map_err(fence_stop)?;
        let session = super::super::session::revision::admit(revisions, &self.request)?;
        Ok((repair, session))
    }

    /// After the session: the persisted binding, the candidate store and,
    /// with the fence rechecked under the same lock, the intent.
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
        Ok((admitted, store))
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
