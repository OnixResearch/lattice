//! Shared observations for the retained future's exclusive store borrow.
// r[impl onix.wasmtime_runtime.instantiation_checkpoint]

/// Successful instantiation, before the selected export has started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstantiationCheckpoint {
    /// Exact remaining cumulative fuel after instantiation.
    pub fuel_remaining: u64,
}

pub(crate) struct Progress {
    is_instantiated: std::sync::atomic::AtomicBool,
    is_invoking: std::sync::atomic::AtomicBool,
    fuel_remaining: std::sync::atomic::AtomicU64,
}

impl Progress {
    pub(crate) const fn new() -> Self {
        Self {
            is_instantiated: std::sync::atomic::AtomicBool::new(false),
            is_invoking: std::sync::atomic::AtomicBool::new(false),
            fuel_remaining: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub(crate) fn checkpoint(&self) -> Option<InstantiationCheckpoint> {
        if self
            .is_instantiated
            .load(std::sync::atomic::Ordering::Acquire)
            && !self.is_invoking.load(std::sync::atomic::Ordering::Acquire)
        {
            Some(InstantiationCheckpoint {
                fuel_remaining: self
                    .fuel_remaining
                    .load(std::sync::atomic::Ordering::Relaxed),
            })
        } else {
            None
        }
    }

    pub(crate) fn phase(&self) -> hardened_wasmtime_runtime::outcomes::InvocationPhase {
        if self.is_invoking.load(std::sync::atomic::Ordering::Acquire) {
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
        } else {
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation
        }
    }

    pub(crate) async fn instantiated(&self, fuel_remaining: u64) {
        self.fuel_remaining
            .store(fuel_remaining, std::sync::atomic::Ordering::Relaxed);
        self.is_instantiated
            .store(true, std::sync::atomic::Ordering::Release);
        let mut is_first_poll = true;
        std::future::poll_fn(|context| {
            if is_first_poll {
                is_first_poll = false;
                context.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(())
            }
        })
        .await;
        self.is_invoking
            .store(true, std::sync::atomic::Ordering::Release);
    }
}
