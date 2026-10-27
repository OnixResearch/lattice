//! Shared store construction and stable outcome classification.

/// Explicit consumer host denial. Return this from an admitted binding to keep
/// denial distinct from guest traps, without unstable engine message matching.
#[derive(Debug)]
pub struct HostDenied;

impl std::fmt::Display for HostDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("consumer host adapter denied the request")
    }
}
impl std::error::Error for HostDenied {}

/// Consumer-owned state and private cumulative Wasmtime store limits.
pub struct State<T> {
    limits: crate::budget::Limits,
    pub(crate) phase: hardened_wasmtime_runtime::outcomes::InvocationPhase,
    /// Product state; the runtime neither interprets nor grants authority to it.
    pub host: T,
}

/// Stable bounded result, independent of Wasmtime's display text.
#[derive(Debug, PartialEq, Eq)]
pub struct InvocationOutcome {
    /// Completed or normalized failure class.
    pub class: hardened_wasmtime_runtime::outcomes::Class,
    /// Stage reached by the invocation; completed outcomes always use Invocation.
    /// Export selection and automatic post-return cleanup belong to Invocation.
    pub phase: hardened_wasmtime_runtime::outcomes::InvocationPhase,
    /// Bounded successful output; empty on failure.
    pub output: Vec<u8>,
    /// Fuel remaining after the entire invocation, including post-return.
    pub fuel_remaining: u64,
}

pub(crate) fn create<T>(
    runtime: &crate::BoundedRuntime,
    host: T,
    ticks: u64,
) -> Result<wasmtime::Store<State<T>>, hardened_wasmtime_runtime::outcomes::Class> {
    let mut store = wasmtime::Store::new(&runtime.engine, initial_state(runtime, host));
    store.limiter(|state| &mut state.limits);
    store
        .set_fuel(runtime.bounds.fuel)
        .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::HostFailed)?;
    store.set_epoch_deadline(ticks);
    store.set_hostcall_fuel(runtime.transfer_bytes);
    Ok(store)
}

fn initial_state<T>(runtime: &crate::BoundedRuntime, host: T) -> State<T> {
    State {
        limits: crate::budget::Limits {
            inner: runtime.store_limits.clone(),
            is_denied: false,
            memory_bytes_remaining: runtime.total_memory_bytes,
        },
        phase: hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission,
        host,
    }
}

pub(crate) fn failure(class: hardened_wasmtime_runtime::outcomes::Class) -> InvocationOutcome {
    InvocationOutcome {
        class,
        phase: hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission,
        output: Vec::new(),
        fuel_remaining: 0,
    }
}

// r[impl onix.wasmtime_runtime.failure_phase]
pub(crate) fn finish<T>(
    store: &wasmtime::Store<State<T>>,
    result: Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class>,
) -> InvocationOutcome {
    let result = if store.data().limits.is_denied {
        Err(hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded)
    } else {
        result
    };
    let fuel_remaining = match store.get_fuel() {
        Ok(value) => value,
        Err(_) => {
            return outcome(
                Err(hardened_wasmtime_runtime::outcomes::Class::HostFailed),
                0,
                store.data().phase,
            );
        }
    };
    outcome(result, fuel_remaining, store.data().phase)
}

fn outcome(
    result: Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class>,
    fuel_remaining: u64,
    phase: hardened_wasmtime_runtime::outcomes::InvocationPhase,
) -> InvocationOutcome {
    match result {
        Ok(output) => InvocationOutcome {
            class: hardened_wasmtime_runtime::outcomes::Class::Completed,
            phase: hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
            output,
            fuel_remaining,
        },
        Err(class) => InvocationOutcome {
            class,
            phase,
            output: Vec::new(),
            fuel_remaining,
        },
    }
}

pub(crate) fn classify(error: &wasmtime::Error) -> hardened_wasmtime_runtime::outcomes::Class {
    if error.downcast_ref::<HostDenied>().is_some() {
        return hardened_wasmtime_runtime::outcomes::Class::HostFailed;
    }
    if let Some(trap) = error.downcast_ref::<wasmtime::Trap>() {
        return classify_trap(trap);
    }
    if crate::transfer::is_exhausted(error) {
        return hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded;
    }
    hardened_wasmtime_runtime::outcomes::Class::RuntimeFailed
}

const fn classify_trap(trap: &wasmtime::Trap) -> hardened_wasmtime_runtime::outcomes::Class {
    match trap {
        wasmtime::Trap::OutOfFuel => hardened_wasmtime_runtime::outcomes::Class::FuelExhausted,
        wasmtime::Trap::Interrupt => hardened_wasmtime_runtime::outcomes::Class::DeadlineExhausted,
        wasmtime::Trap::MemoryOutOfBounds
        | wasmtime::Trap::TableOutOfBounds
        | wasmtime::Trap::AllocationTooLarge => {
            hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
        }
        _ => hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
    }
}

pub(crate) fn instantiation_error(
    error: wasmtime::Error,
) -> hardened_wasmtime_runtime::outcomes::Class {
    if error.downcast_ref::<wasmtime::Trap>().is_some()
        || error.downcast_ref::<crate::HostDenied>().is_some()
    {
        classify(&error)
    } else {
        hardened_wasmtime_runtime::outcomes::Class::InstantiateFailed
    }
}
