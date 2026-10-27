const OUTPUT_BYTES: u64 = 2;
const SHORT_OUTPUT_LIMIT: u64 = 1;
const ONE_DEADLINE_TICK: u64 = 1;

#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
// r[verify onix.wasmtime_runtime.failure_phase]
fn action_and_cleanup_exhaust_one_cumulative_budget_without_refilling_between_slices() {
    for body in [
        super::support::Body {
            action: "(loop br 0)",
            cleanup: "",
        },
        super::support::Body {
            action: "",
            cleanup: "(loop br 0)",
        },
    ] {
        let bytes = super::support::component("invoke", body);
        let (prepared, _) = super::slicesupport::prepare(
            &bytes,
            super::slicesupport::bounds(),
            super::slicesupport::DEADLINE_TICKS,
        );
        let synchronous = prepared.run("invoke", b"", (), |_, _| {
            Err(hardened_wasmtime_shell::HostDenied)
        });
        let mut invocation = super::slicesupport::start(&prepared);
        let (outcome, yields) = super::slicesupport::finish(&mut invocation);
        assert!(
            yields >= super::slicesupport::MIN_YIELDS,
            "total fuel must span several finite quanta"
        );
        assert_eq!(
            outcome.class,
            hardened_wasmtime_runtime::outcomes::Class::FuelExhausted
        );
        assert_eq!(
            outcome.phase,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
        );
        assert_eq!(outcome.fuel_remaining, 0);
        assert!(outcome.output.is_empty());
        assert_eq!(outcome, synchronous);
        assert!(matches!(
            invocation.run_slice(),
            Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
        ));
        // A terminal invocation must release its permit even while its handle lives.
        let mut replacement = super::slicesupport::start(&prepared);
        super::slicesupport::checkpoint(&mut replacement)
            .0
            .expect("successful instantiation");
        assert!(matches!(
            replacement.run_slice(),
            Ok(hardened_wasmtime_shell::SliceStep::Yielded(_))
        ));
    }
}

#[test]
// r[verify onix.wasmtime_runtime.resumable_execution]
// r[verify onix.wasmtime_runtime.slice_limits]
// r[verify onix.wasmtime_runtime.failure_phase]
fn yielded_cleanup_traps_and_oversized_results_never_publish_success_bytes() {
    let trapping_cleanup = format!("{} unreachable", super::slicesupport::FINITE_WORK);
    let cases = [
        (
            super::support::Body {
                action: "",
                cleanup: &trapping_cleanup,
            },
            OUTPUT_BYTES,
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
        ),
        (
            super::support::Body {
                action: super::slicesupport::FINITE_WORK,
                cleanup: "",
            },
            SHORT_OUTPUT_LIMIT,
            hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded,
        ),
    ];
    for (body, output_limit, expected) in cases {
        let bytes = super::support::component("invoke", body);
        let mut resource_bounds = super::slicesupport::bounds();
        resource_bounds.max_output_bytes = output_limit;
        let (prepared, _) = super::slicesupport::prepare(
            &bytes,
            resource_bounds,
            super::slicesupport::DEADLINE_TICKS,
        );
        let synchronous = prepared.run("invoke", b"", (), |_, _| {
            Err(hardened_wasmtime_shell::HostDenied)
        });
        let mut invocation = super::slicesupport::start(&prepared);
        let (outcome, yields) = super::slicesupport::finish(&mut invocation);
        assert!(
            yields >= super::slicesupport::MIN_YIELDS,
            "failure must occur after resumable guest work"
        );
        assert_eq!(outcome.class, expected);
        assert_eq!(
            outcome.phase,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
        );
        assert!(outcome.output.is_empty());
        assert_eq!(outcome, synchronous);
        assert!(matches!(
            invocation.run_slice(),
            Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
        ));
    }
}

#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
// r[verify onix.wasmtime_runtime.failure_phase]
fn an_injected_epoch_expires_the_original_deadline_while_the_invocation_is_suspended() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "(loop br 0)",
            cleanup: "",
        },
    );
    let (prepared, service) =
        super::slicesupport::prepare(&bytes, super::slicesupport::bounds(), ONE_DEADLINE_TICK);
    let mut invocation = super::slicesupport::start(&prepared);
    super::slicesupport::checkpoint(&mut invocation)
        .0
        .expect("successful instantiation");
    assert_eq!(
        invocation.run_slice(),
        Ok(hardened_wasmtime_shell::SliceStep::Yielded(
            hardened_wasmtime_shell::SliceBoundary {
                ordinal: 1,
                quantum_fuel: super::slicesupport::QUANTUM_FUEL,
                phase: hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
            }
        ))
    );
    service.advance();
    let (outcome, _) = super::slicesupport::finish(&mut invocation);
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::DeadlineExhausted
    );
    assert_eq!(
        outcome.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert!(outcome.fuel_remaining > 0);
    assert!(outcome.output.is_empty());
    assert!(matches!(
        invocation.run_slice(),
        Err(hardened_wasmtime_shell::SliceError::AlreadyCompleted)
    ));
}

#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
// r[verify onix.wasmtime_runtime.failure_phase]
fn only_positive_quanta_within_the_total_budget_are_admitted() {
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
    for quantum in [
        0,
        super::slicesupport::TOTAL_FUEL
            .checked_add(1)
            .expect("fixture budget leaves room for refusal"),
    ] {
        let refused = std::sync::Arc::clone(&prepared)
            .start_sliced(super::slicesupport::options(quantum), (), |_, _| {
                Err(hardened_wasmtime_shell::HostDenied)
            })
            .err()
            .expect("invalid fuel quantum must fail admission");
        assert_eq!(
            refused.class,
            hardened_wasmtime_runtime::outcomes::Class::ProfileDenied
        );
        assert_eq!(
            refused.phase,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission
        );
        assert!(refused.output.is_empty());
    }
    let synchronous = prepared.run("invoke", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        synchronous.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(synchronous.output, b"OK");
    let mut invocation = std::sync::Arc::clone(&prepared)
        .start_sliced(
            super::slicesupport::options(super::slicesupport::TOTAL_FUEL),
            (),
            |_, _| Err(hardened_wasmtime_shell::HostDenied),
        )
        .expect("a quantum equal to the total budget is valid");
    super::slicesupport::checkpoint(&mut invocation)
        .0
        .expect("successful instantiation");
    assert_eq!(
        invocation.run_slice(),
        Ok(hardened_wasmtime_shell::SliceStep::Completed(synchronous))
    );
}
