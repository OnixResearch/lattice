const MAX_SILENT_MS: u64 = 5_000;
const OVER_SILENT_MS: u64 = 5_001;
const OUTPUT_BOUND_BYTES: u64 = 1_024;
const SUSPENDED_MS: u64 = 6_000;
const MAX_ITEMS: u32 = 2;
const MAX_CALLS: u32 = 2;
const COMMITTED_CALLS: u32 = 2;

fn handle(token: &str) -> super::super::Handle {
    super::super::Handle::new(super::super::StoreKind::Result, token).expect("handle")
}

fn prepared_cycle() -> super::super::Cycle {
    let cycle = super::super::Cycle::new("cycle-1", MAX_CALLS).expect("cycle");
    super::super::prepare_cycle(&cycle, &["call-1", "call-2"]).expect("prepared")
}

fn observations(status: super::super::RunStatus) -> alloc::vec::Vec<super::super::RunObservation> {
    alloc::vec![
        super::super::RunObservation {
            call_id: alloc::string::String::from("call-1"),
            status,
            telemetry: None,
        },
        super::super::RunObservation {
            call_id: alloc::string::String::from("call-2"),
            status,
            telemetry: None,
        },
    ]
}

#[test]
fn over_bound_silence_detaches_with_handle_and_follow_command() {
    let exact = handle("res_waiting");
    let disposition = super::super::plan_silence(
        super::super::SilencePolicy {
            max_silent_ms: MAX_SILENT_MS,
            max_output_bytes: OUTPUT_BOUND_BYTES,
        },
        super::super::SilenceObservation {
            silent_ms: OVER_SILENT_MS,
            output_bytes: 0,
            terminal: false,
        },
        &exact,
    )
    .expect("detach");

    assert_ne!(disposition, super::super::SilenceDisposition::Continue);
    assert!(matches!(
        disposition,
        super::super::SilenceDisposition::Detach {
            ref handle,
            ref follow_command,
        } if handle == &exact && follow_command.contains(exact.token())
    ));
}

#[test]
fn in_bound_and_terminal_calls_never_detach() {
    let exact = handle("res_complete");
    let policy = super::super::SilencePolicy {
        max_silent_ms: MAX_SILENT_MS,
        max_output_bytes: OUTPUT_BOUND_BYTES,
    };
    let within = super::super::plan_silence(
        policy,
        super::super::SilenceObservation {
            silent_ms: MAX_SILENT_MS,
            output_bytes: OUTPUT_BOUND_BYTES,
            terminal: false,
        },
        &exact,
    );
    let terminal = super::super::plan_silence(
        policy,
        super::super::SilenceObservation {
            silent_ms: OVER_SILENT_MS,
            output_bytes: OUTPUT_BOUND_BYTES,
            terminal: true,
        },
        &exact,
    );

    assert_eq!(within, Ok(super::super::SilenceDisposition::Continue));
    assert_eq!(terminal, Ok(super::super::SilenceDisposition::Complete));
}

#[test]
fn suspension_dump_names_every_item_and_duration() {
    let items = [
        super::super::Suspended {
            call_id: alloc::string::String::from("call-1"),
            handle: handle("res_one"),
            duration_ms: SUSPENDED_MS,
        },
        super::super::Suspended {
            call_id: alloc::string::String::from("call-2"),
            handle: handle("res_two"),
            duration_ms: SUSPENDED_MS,
        },
    ];
    let dump = super::super::project_suspension_dump(&items, MAX_ITEMS).expect("dump");

    assert_eq!(dump.items, items);
    assert_eq!(dump.items[0].call_id, "call-1");
    assert_eq!(dump.items[1].call_id, "call-2");
    assert_eq!(dump.items[0].duration_ms, SUSPENDED_MS);
    assert_eq!(dump.items[1].duration_ms, SUSPENDED_MS);
}

#[test]
fn parked_cycle_is_not_finished() {
    let cycle = prepared_cycle();
    let parked = super::super::observe_batch(
        &cycle,
        &observations(super::super::RunStatus::NeedsHuman),
        true,
    )
    .expect("parked");

    assert_eq!(parked.phase(), super::super::CyclePhase::Parked);
    assert_ne!(parked.phase(), super::super::CyclePhase::Finished);
    assert!(
        super::super::commit_cycle(
            &parked,
            super::super::GuardObservation {
                guard_released: true,
                side_effects_inside_guard: false,
                committed_call_count: COMMITTED_CALLS,
            }
        )
        .is_err()
    );
}

#[test]
fn commit_requires_post_guard_exact_completion() {
    let prepared = prepared_cycle();
    let ready = super::super::observe_batch(
        &prepared,
        &observations(super::super::RunStatus::Completed),
        true,
    )
    .expect("ready");
    let inside_guard = super::super::commit_cycle(
        &ready,
        super::super::GuardObservation {
            guard_released: false,
            side_effects_inside_guard: true,
            committed_call_count: COMMITTED_CALLS,
        },
    );
    let finished = super::super::commit_cycle(
        &ready,
        super::super::GuardObservation {
            guard_released: true,
            side_effects_inside_guard: false,
            committed_call_count: COMMITTED_CALLS,
        },
    )
    .expect("finished");

    assert_eq!(ready.phase(), super::super::CyclePhase::ReadyToCommit);
    assert_eq!(inside_guard, Err(super::super::Error::InvalidSupervision));
    assert_eq!(finished.phase(), super::super::CyclePhase::Finished);
}

#[test]
fn duplicate_observation_and_false_parallel_claim_fail() {
    let prepared = prepared_cycle();
    let duplicate = alloc::vec![
        super::super::RunObservation {
            call_id: alloc::string::String::from("call-1"),
            status: super::super::RunStatus::Completed,
            telemetry: None,
        },
        super::super::RunObservation {
            call_id: alloc::string::String::from("call-1"),
            status: super::super::RunStatus::Completed,
            telemetry: None,
        },
    ];

    assert!(super::super::observe_batch(&prepared, &duplicate, true).is_err());
    assert!(
        super::super::observe_batch(
            &prepared,
            &observations(super::super::RunStatus::Completed),
            false
        )
        .is_err()
    );
}
