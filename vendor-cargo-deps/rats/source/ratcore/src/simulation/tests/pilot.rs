use super::super::*;

#[test]
fn observer_presence_is_behavior_neutral() {
    let controls = HeadlessPilotControls::default();
    let without = run_headless_pilot(controls, None).expect("headless run");
    let mut no_op = NoopSimulationProbeObserver;
    let with_no_op = run_headless_pilot(controls, Some(&mut no_op)).expect("no-op run");
    let mut recording = RecordingSimulationProbeObserver::default();
    let with_recording = run_headless_pilot(controls, Some(&mut recording)).expect("recording run");

    assert_eq!(without.final_state, with_no_op.final_state);
    assert_eq!(without.final_state, with_recording.final_state);
    assert_eq!(without.behavior_identity, with_no_op.behavior_identity);
    assert_eq!(without.behavior_identity, with_recording.behavior_identity);
    assert_eq!(without.diagnostics, with_recording.diagnostics);
    assert!(!recording.records().is_empty());
}

#[test]
fn identical_controls_repeat_probe_and_state_bytes_exactly() {
    let controls = HeadlessPilotControls::default();
    let mut first_observer = RecordingSimulationProbeObserver::default();
    let first = run_headless_pilot(controls, Some(&mut first_observer)).expect("first run");
    let mut second_observer = RecordingSimulationProbeObserver::default();
    let second = run_headless_pilot(controls, Some(&mut second_observer)).expect("second run");

    assert_eq!(first, second);
    assert_eq!(first_observer.normalized_bytes(), second_observer.normalized_bytes());
    assert_eq!(first.final_projection.blake3_identity(), second.final_projection.blake3_identity());
}

#[test]
fn clock_choice_order_and_effect_perturbations_have_classified_divergence() {
    let baseline = run_headless_pilot(HeadlessPilotControls::default(), None).expect("baseline headless run");
    let clock = run_headless_pilot(
        HeadlessPilotControls {
            logical_tick: 11,
            ..HeadlessPilotControls::default()
        },
        None,
    )
    .expect("clock perturbation");
    let choice = run_headless_pilot(
        HeadlessPilotControls {
            selected_choice: 0,
            ..HeadlessPilotControls::default()
        },
        None,
    )
    .expect("choice perturbation");
    let order = run_headless_pilot(
        HeadlessPilotControls {
            event_order: HeadlessPilotEventOrder::ChoiceThenTick,
            ..HeadlessPilotControls::default()
        },
        None,
    )
    .expect("schedule perturbation");
    let effect = run_headless_pilot(
        HeadlessPilotControls {
            effect_result: SimulationEffectResult::Failed,
            ..HeadlessPilotControls::default()
        },
        None,
    )
    .expect("effect perturbation");

    assert_ne!(baseline.behavior_identity, clock.behavior_identity, "clock");
    assert_ne!(baseline.behavior_identity, choice.behavior_identity, "choice");
    assert_ne!(baseline.behavior_identity, order.behavior_identity, "order");
    assert_ne!(baseline.behavior_identity, effect.behavior_identity, "effect");
    assert_eq!(effect.diagnostics, vec![HeadlessPilotDiagnosticCode::PersistenceNotCompleted]);
}

#[test]
fn invalid_choice_fails_before_application_or_observer_runs() {
    let mut observer = RecordingSimulationProbeObserver::default();
    let error = run_headless_pilot(
        HeadlessPilotControls {
            selected_choice: 2,
            ..HeadlessPilotControls::default()
        },
        Some(&mut observer),
    )
    .expect_err("out-of-range controlled choice must fail");

    let HeadlessPilotError::Controls(issues) = error else {
        panic!("expected controlled-input failure");
    };
    assert!(issues.iter().any(|issue| { issue.code == SimulationControlDiagnosticCode::ChoiceOutOfRange }));
    assert!(observer.records().is_empty());
}

#[test]
fn pure_update_covers_accepted_rejected_ignored_resize_timer_and_shutdown() {
    let mut state = HeadlessPilotState::default();
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::LogicalTick(10)).kind,
        HeadlessPilotTransitionKind::Accepted
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::LogicalTick(10)).kind,
        HeadlessPilotTransitionKind::Ignored
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::LogicalTick(9)).kind,
        HeadlessPilotTransitionKind::Rejected
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::SelectPane(3)).kind,
        HeadlessPilotTransitionKind::Rejected
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::Resize { columns: 80, rows: 24 },).kind,
        HeadlessPilotTransitionKind::Accepted
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::Resize { columns: 80, rows: 24 },).kind,
        HeadlessPilotTransitionKind::Ignored
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::Resize { columns: 0, rows: 24 },).kind,
        HeadlessPilotTransitionKind::Rejected
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::Shutdown).kind,
        HeadlessPilotTransitionKind::Accepted
    );
    assert_eq!(
        update_headless_pilot(&mut state, HeadlessPilotMessage::Shutdown).kind,
        HeadlessPilotTransitionKind::Ignored
    );
}

#[test]
fn receipt_keeps_local_hook_evidence_separate_from_replay_claims() {
    let receipt = run_headless_pilot(HeadlessPilotControls::default(), None).expect("headless receipt").render();

    assert!(receipt.contains("evidence_class=provider-neutral-headless"));
    assert!(receipt.contains("claims=local-controlled-hooks-only"));
    assert!(receipt.contains("not_claimed=in-process-simulator,vm-replay"));
}
