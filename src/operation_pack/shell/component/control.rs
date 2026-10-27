//! Consumer-owned cooperative epoch driver; the shared runtime owns no clocks.
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use super::observation::Class;

const OBSERVER_INTERVAL: Duration = Duration::from_millis(1);
const RUNNING: u8 = 0;
const CANCELLED: u8 = 1;
const EXPIRED: u8 = 2;

pub(super) struct Observation {
    pub invocation: Option<hardened_wasmtime_shell::InvocationOutcome>,
    pub interruption: Option<Class>,
    pub observer_joined: bool,
}

pub(super) fn invoke(
    prepared: &super::verification::Prepared,
    input: &[u8],
    deadline: Instant,
    cancellation: Option<&AtomicBool>,
) -> Observation {
    if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
        return Observation {
            invocation: None,
            interruption: Some(Class::Cancelled),
            observer_joined: true,
        };
    }
    if Instant::now() >= deadline {
        return Observation {
            invocation: None,
            interruption: Some(Class::DeadlineExceeded),
            observer_joined: true,
        };
    }
    let finished = AtomicBool::new(false);
    let interruption = AtomicU8::new(RUNNING);
    std::thread::scope(|scope| {
        let observer =
            std::thread::Builder::new().name("lattice-component-epoch".to_string()).spawn_scoped(scope, || {
                while !finished.load(Ordering::Acquire) {
                    let reason = if cancellation.is_some_and(|flag| flag.load(Ordering::Acquire)) {
                        CANCELLED
                    } else if Instant::now() >= deadline {
                        EXPIRED
                    } else {
                        RUNNING
                    };
                    if reason != RUNNING {
                        let _ = interruption.compare_exchange(RUNNING, reason, Ordering::AcqRel, Ordering::Acquire);
                        // Store construction captures the engine's current epoch.
                        // Keep advancing until return so an interrupt requested
                        // just before that capture cannot be lost.
                        prepared.epoch_service.advance();
                    }
                    std::thread::park_timeout(OBSERVER_INTERVAL);
                }
            });
        let Ok(observer) = observer else {
            return Observation {
                invocation: None,
                interruption: Some(Class::Unknown),
                observer_joined: false,
            };
        };
        let invocation = prepared.component.run(&prepared.dispatch.guest_method, input);
        finished.store(true, Ordering::Release);
        observer.thread().unpark();
        let observer_joined = observer.join().is_ok();
        let interruption = match interruption.load(Ordering::Acquire) {
            CANCELLED => Some(Class::Cancelled),
            EXPIRED => Some(Class::DeadlineExceeded),
            _ => None,
        };
        Observation {
            invocation: Some(invocation),
            interruption,
            observer_joined,
        }
    })
}
