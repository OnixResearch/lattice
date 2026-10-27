//! Host-driven fuel suspension of synchronous component exports.
// r[impl onix.wasmtime_runtime.resumable_execution]
// r[impl onix.wasmtime_runtime.slice_limits]
// r[impl onix.wasmtime_runtime.instantiation_checkpoint]

/// Owned carrier and fuel interval for one retained invocation.
pub struct SliceOptions {
    /// Selected admitted root export.
    pub export: String,
    /// Bounded input, moved into the future without copying.
    pub input: Vec<u8>,
    /// Positive yield interval, no larger than the admitted total fuel budget.
    pub quantum_fuel: u64,
}

/// One observed suspension; not a per-poll fuel-consumption measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceBoundary {
    /// One-based ordinal of this invocation's pending poll.
    pub ordinal: u64,
    /// Configured Wasmtime yield interval; instruction checks may batch work.
    pub quantum_fuel: u64,
    /// Whether instantiation or the selected export/cleanup suspended.
    pub phase: hardened_wasmtime_runtime::outcomes::InvocationPhase,
}

/// Result of polling a retained invocation once.
#[derive(Debug, PartialEq, Eq)]
pub enum SliceStep {
    /// The retained future suspended; it has not published output.
    Yielded(SliceBoundary),
    /// Successful instantiation, before the selected export can execute.
    Instantiated(crate::InstantiationCheckpoint),
    /// The single terminal result, including cleanup and measured remaining fuel.
    Completed(crate::InvocationOutcome),
}

/// An invalid continuation operation, never another guest execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceError {
    /// The invocation already completed or was stopped by observation overflow.
    AlreadyCompleted,
    /// A consumer polled more suspension boundaries than a u64 can represent.
    OrdinalExhausted,
}

type InvocationFuture =
    std::pin::Pin<Box<dyn Future<Output = crate::InvocationOutcome> + Send + 'static>>;

/// One movable invocation retaining its store, limits and guest continuation.
/// Drop cancels the future and releases its store and concurrency ownership.
/// Cancellation does not roll back imports or promise guest post-return cleanup.
pub struct ResumableInvocation {
    future: Option<InvocationFuture>,
    ordinal: u64,
    quantum_fuel: u64,
    progress: std::sync::Arc<crate::milestone::Progress>,
}

impl ResumableInvocation {
    /// Poll the same future exactly once. The consumer owns scheduling between polls.
    /// Exact remaining fuel is observed at successful instantiation and completion;
    /// a suspended Wasmtime future otherwise holds its store's exclusive borrow.
    pub fn run_slice(&mut self) -> Result<SliceStep, SliceError> {
        let future = self.future.as_mut().ok_or(SliceError::AlreadyCompleted)?;
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Pending => self.pending(),
            std::task::Poll::Ready(outcome) => {
                self.future = None;
                Ok(SliceStep::Completed(outcome))
            }
        }
    }

    fn pending(&mut self) -> Result<SliceStep, SliceError> {
        if let Some(checkpoint) = self.progress.checkpoint() {
            return Ok(SliceStep::Instantiated(checkpoint));
        }
        self.yielded()
    }

    fn yielded(&mut self) -> Result<SliceStep, SliceError> {
        let Some(ordinal) = self.ordinal.checked_add(1) else {
            self.future = None;
            return Err(SliceError::OrdinalExhausted);
        };
        self.ordinal = ordinal;
        Ok(SliceStep::Yielded(SliceBoundary {
            ordinal,
            quantum_fuel: self.quantum_fuel,
            phase: self.progress.phase(),
        }))
    }
}

impl crate::ComponentInvocation {
    /// Admit owned input and quanta, retaining a single invocation for host polling.
    /// Binding and guest execution start on the first poll, not at construction.
    /// The consumer binder and synchronous imports must return without blocking.
    /// Host-driven yielding does not admit guest Component Model async features.
    pub fn start_sliced<T: Send + 'static, F>(
        self: std::sync::Arc<Self>,
        options: SliceOptions,
        host: T,
        bind: F,
    ) -> Result<ResumableInvocation, crate::InvocationOutcome>
    where
        F: FnMut(
                &hardened_wasmtime_runtime::restricted::imports::Descriptor,
                &mut wasmtime::component::Linker<crate::State<T>>,
            ) -> Result<(), crate::HostDenied>
            + Send
            + 'static,
    {
        if options.input.len() as u64 > self.prepared.runtime.bounds.max_input_bytes {
            return Err(crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded,
            ));
        }
        if options.quantum_fuel == 0 || options.quantum_fuel > self.prepared.runtime.bounds.fuel {
            return Err(crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::ProfileDenied,
            ));
        }
        let quantum_fuel = options.quantum_fuel;
        let Some(permit) = self.prepared.runtime.enter_owned() else {
            return Err(crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted,
            ));
        };
        let progress = std::sync::Arc::new(crate::milestone::Progress::new());
        let observed = std::sync::Arc::clone(&progress);
        Ok(ResumableInvocation {
            future: Some(Box::pin(async move {
                let _permit = permit;
                self.run_sliced(options, host, bind, &progress).await
            })),
            ordinal: 0,
            quantum_fuel,
            progress: observed,
        })
    }

    async fn run_sliced<T: Send + 'static, F>(
        &self,
        options: SliceOptions,
        host: T,
        mut bind: F,
        progress: &crate::milestone::Progress,
    ) -> crate::InvocationOutcome
    where
        F: FnMut(
                &hardened_wasmtime_runtime::restricted::imports::Descriptor,
                &mut wasmtime::component::Linker<crate::State<T>>,
            ) -> Result<(), crate::HostDenied>
            + Send,
    {
        let runtime = &self.prepared.runtime;
        let mut store = match crate::store::create(runtime, host, self.prepared.ticks) {
            Ok(store) => store,
            Err(class) => return crate::store::failure(class),
        };
        if store
            .fuel_async_yield_interval(Some(options.quantum_fuel))
            .is_err()
        {
            return crate::store::finish(
                &store,
                Err(hardened_wasmtime_runtime::outcomes::Class::HostFailed),
            );
        }
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Linking;
        let mut linker = wasmtime::component::Linker::new(&runtime.engine);
        for descriptor in &self.prepared.descriptors {
            if bind(descriptor, &mut linker).is_err() {
                return crate::store::finish(
                    &store,
                    Err(hardened_wasmtime_runtime::outcomes::Class::HostFailed),
                );
            }
        }
        let result = self
            .execute_sliced(&mut store, &linker, &options, progress)
            .await;
        crate::store::finish(&store, result)
    }

    // r[impl onix.wasmtime_runtime.failure_phase]
    async fn execute_sliced<T: Send + 'static>(
        &self,
        store: &mut wasmtime::Store<crate::State<T>>,
        linker: &wasmtime::component::Linker<crate::State<T>>,
        options: &SliceOptions,
        progress: &crate::milestone::Progress,
    ) -> Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class> {
        store.data_mut().phase =
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation;
        let instance = linker
            .instantiate_async(&mut *store, &self.component)
            .await
            .map_err(crate::store::instantiation_error)?;
        let fuel_remaining = store
            .get_fuel()
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::HostFailed)?;
        progress.instantiated(fuel_remaining).await;
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation;
        let function = instance
            .get_typed_func::<(&[u8],), (Result<Vec<u8>, wasmtime::component::WasmStr>,)>(
                &mut *store,
                &options.export,
            )
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let (result,) = function
            .call_async(&mut *store, (&options.input,))
            .await
            .map_err(|error| crate::store::classify(&error))?;
        result.map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestDenied)
    }
}
