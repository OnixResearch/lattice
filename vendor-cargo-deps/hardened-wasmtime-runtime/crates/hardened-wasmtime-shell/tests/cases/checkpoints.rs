//! Actual start work, callback fencing and cancellation at the public checkpoint.

fn component(start_work: &str) -> Vec<u8> {
    let source = String::from_utf8(super::slicesupport::stateful_component()).expect("fixture WAT");
    source
        .replacen(
            "(global $progress",
            &format!("(func $start {start_work}) (start $start) (global $progress"),
            1,
        )
        .into_bytes()
}

#[test]
// r[verify onix.wasmtime_runtime.instantiation_checkpoint]
fn start_work_finishes_before_the_callback_and_preserves_same_store_fuel() {
    let bytes = component(super::slicesupport::FINITE_WORK);
    let prepared = super::slicesupport::marked_prepared(&bytes);
    let sync_marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let synchronous = prepared.run("invoke", b"", sync_marks, super::slicesupport::bind_marker);
    let marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            std::sync::Arc::clone(&marks),
            super::slicesupport::bind_marker,
        )
        .expect("admitted component");
    let (observation, yields) = super::slicesupport::checkpoint(&mut invocation);
    let observation = observation.expect("successful instantiation");
    assert!(yields >= super::slicesupport::MIN_YIELDS);
    assert!(observation.fuel_remaining < super::slicesupport::TOTAL_FUEL);
    assert_eq!(marks.load(std::sync::atomic::Ordering::SeqCst), 0);
    let next = invocation.run_slice().expect("live callback");
    assert!(
        matches!(next, hardened_wasmtime_shell::SliceStep::Yielded(boundary)
        if boundary.phase == hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation)
    );
    let (completed, _) = super::slicesupport::finish(&mut invocation);
    assert_eq!(completed, synchronous);
    assert_eq!(
        marks.load(std::sync::atomic::Ordering::SeqCst),
        super::slicesupport::CLEANUP_MARK
    );
    assert!(matches!(
        invocation.run_slice(),
        Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
    ));
}

#[test]
// r[verify onix.wasmtime_runtime.instantiation_checkpoint]
// r[verify onix.wasmtime_runtime.slice_limits]
fn cancellation_at_instantiation_never_calls_the_export_and_releases_the_permit() {
    let bytes = component(super::slicesupport::FINITE_WORK);
    let prepared = super::slicesupport::marked_prepared(&bytes);
    let marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            std::sync::Arc::clone(&marks),
            super::slicesupport::bind_marker,
        )
        .expect("admitted component");
    super::slicesupport::checkpoint(&mut invocation)
        .0
        .expect("successful instantiation");
    let denied = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            (),
            |_, _| Err(hardened_wasmtime_shell::HostDenied),
        )
        .err()
        .expect("checkpoint retains the sole permit");
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted
    );
    drop(invocation);
    assert_eq!(marks.load(std::sync::atomic::Ordering::SeqCst), 0);
    let mut replacement = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
            std::sync::Arc::clone(&marks),
            super::slicesupport::bind_marker,
        )
        .expect("cancelled checkpoint released its permit");
    let (completed, _) = super::slicesupport::finish(&mut replacement);
    assert_eq!(
        completed.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(
        marks.load(std::sync::atomic::Ordering::SeqCst),
        super::slicesupport::CLEANUP_MARK
    );
}

#[test]
// r[verify onix.wasmtime_runtime.instantiation_checkpoint]
fn failed_start_never_reports_successful_instantiation() {
    for (work, class) in [
        (
            "unreachable",
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
        ),
        (
            "(loop br 0)",
            hardened_wasmtime_runtime::outcomes::Class::FuelExhausted,
        ),
    ] {
        let bytes = component(work);
        let prepared = super::slicesupport::marked_prepared(&bytes);
        let marks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut invocation = std::sync::Arc::clone(&prepared)
            .start_sliced(
                super::slicesupport::options(super::slicesupport::QUANTUM_FUEL),
                std::sync::Arc::clone(&marks),
                super::slicesupport::bind_marker,
            )
            .expect("admitted component");
        let outcome = super::slicesupport::checkpoint(&mut invocation)
            .0
            .expect_err("failed instantiation");
        assert_eq!(outcome.class, class);
        assert_eq!(
            outcome.phase,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation
        );
        assert!(outcome.output.is_empty());
        assert_eq!(marks.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(matches!(
            invocation.run_slice(),
            Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
        ));
    }
}
