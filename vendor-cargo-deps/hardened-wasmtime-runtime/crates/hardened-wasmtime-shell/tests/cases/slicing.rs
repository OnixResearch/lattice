#[test]
// r[verify onix.wasmtime_runtime.resumable_execution]
// r[verify onix.wasmtime_runtime.failure_phase]
fn one_store_survives_action_and_cleanup_yields_with_exact_sync_fuel_and_output() {
    let bytes = super::slicesupport::stateful_component();
    let prepared = super::slicesupport::marked_prepared(&bytes);
    let synchronous_marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let synchronous = prepared.run(
        "invoke",
        b"",
        std::sync::Arc::clone(&synchronous_marks),
        super::slicesupport::bind_marker,
    );
    assert_eq!(
        synchronous.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(
        synchronous.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert_eq!(synchronous.output, [super::slicesupport::LOOP_ITERATIONS]);
    assert_eq!(
        synchronous_marks.load(std::sync::atomic::Ordering::SeqCst),
        super::slicesupport::CLEANUP_MARK
    );

    let marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            std::sync::Arc::clone(&marks),
            super::slicesupport::bind_marker,
        )
        .expect("sliced component");
    let terminal = super::slicesupport::observe_marked(&mut invocation, &marks);
    assert_eq!(terminal, synchronous);
    assert!(matches!(
        invocation.run_slice(),
        Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
    ));
}

struct HostOwner(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Drop for HostOwner {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[test]
// r[verify onix.wasmtime_runtime.resumable_execution]
// r[verify onix.wasmtime_runtime.slice_limits]
// r[verify onix.wasmtime_runtime.failure_phase]
fn suspended_work_holds_its_permit_and_cancellation_drops_the_host_and_releases_it() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "(loop br 0)",
            cleanup: "",
        },
    );
    let (prepared, _) = super::slicesupport::prepare(
        &bytes,
        super::slicesupport::bounds(),
        super::slicesupport::DEADLINE_TICKS,
    );
    let drops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            HostOwner(std::sync::Arc::clone(&drops)),
            |_, _| Err(hardened_wasmtime_shell::HostDenied),
        )
        .expect("first invocation");
    super::slicesupport::checkpoint(&mut invocation)
        .0
        .expect("successful instantiation");
    for ordinal in 1..=super::slicesupport::MIN_YIELDS {
        assert_eq!(
            invocation.run_slice(),
            Ok(hardened_wasmtime_shell::SliceStep::Yielded(
                hardened_wasmtime_shell::SliceBoundary {
                    ordinal,
                    quantum_fuel: super::slicesupport::QUANTUM_FUEL,
                    phase: hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
                }
            ))
        );
        let refused = std::sync::Arc::clone(&prepared)
            .start_sliced(
                super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
                (),
                |_, _| Err(hardened_wasmtime_shell::HostDenied),
            )
            .err()
            .expect("suspended invocation retains the sole concurrency permit");
        assert_eq!(
            refused.class,
            hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted
        );
        assert_eq!(
            refused.phase,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission
        );
        assert!(refused.output.is_empty());
        assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    drop(invocation);
    assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 1);
    let mut replacement = super::slicesupport::start(&prepared);
    super::slicesupport::checkpoint(&mut replacement)
        .0
        .expect("successful instantiation");
    assert!(matches!(
        replacement.run_slice(),
        Ok(hardened_wasmtime_shell::SliceStep::Yielded(_))
    ));
}

#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
fn an_unpolled_invocation_reserves_and_releases_its_permit_and_host() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "",
        },
    );
    let (prepared, _) = super::slicesupport::prepare(
        &bytes,
        super::slicesupport::bounds(),
        super::slicesupport::DEADLINE_TICKS,
    );
    let drops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            HostOwner(std::sync::Arc::clone(&drops)),
            |_, _| Err(hardened_wasmtime_shell::HostDenied),
        )
        .expect("first invocation");
    let denied = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            (),
            |_, _| Err(hardened_wasmtime_shell::HostDenied),
        )
        .err()
        .expect("unpolled invocation retains its permit");
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted
    );
    assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 0);
    drop(invocation);
    assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 1);
    let mut replacement = super::slicesupport::start(&prepared);
    let (outcome, _) = super::slicesupport::finish(&mut replacement);
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(outcome.output, b"OK");
}
