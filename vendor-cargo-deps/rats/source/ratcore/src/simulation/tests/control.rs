use super::super::*;

fn controls_fixture() -> Vec<SimulationControlInput> {
    vec![
        SimulationControlInput::LogicalTick {
            source_id: "source.timer".to_string(),
            tick: 10,
        },
        SimulationControlInput::FiniteChoice {
            choice_point: SimulationChoicePointId::new("simulation.choice.initial-pane-v1"),
            bound: 2,
            selected: 1,
        },
        SimulationControlInput::EventSelection {
            ready_sources: vec!["source.effect".to_string(), "source.timer".to_string()],
            selected_source: "source.effect".to_string(),
        },
        SimulationControlInput::EffectResult {
            request_id: "request.test-1".to_string(),
            effect_class: SimulationEffectClassId::new("simulation.effect.persist-layout-v1"),
            result: SimulationEffectResult::Completed,
        },
    ]
}

#[test]
fn scripted_and_recording_controls_preserve_explicit_order() {
    let inputs = controls_fixture();
    assert!(validate_control_history(&super::registry(), &inputs).is_empty());
    let scripted = ScriptedSimulationControls::new(inputs.clone()).expect("bounded script");
    let mut recording = RecordingSimulationControls::new(scripted);

    for expected in &inputs {
        assert_eq!(recording.next_input().expect("script input"), *expected);
    }
    assert_eq!(recording.consumed(), inputs);
    assert_eq!(
        recording.next_input().expect_err("exhausted script must not use ambient fallback").code,
        SimulationControlDiagnosticCode::MissingScriptedInput
    );
}

#[test]
fn no_op_controller_fails_without_host_fallback() {
    let issue = NoopSimulationControls.next_input().expect_err("no-op controls have no ambient default");

    assert_eq!(issue.code, SimulationControlDiagnosticCode::MissingScriptedInput);
}

#[test]
fn choices_ticks_scheduler_and_duplicate_results_fail_classifiably() {
    let history = vec![
        SimulationControlInput::LogicalTick {
            source_id: "source.timer".to_string(),
            tick: 10,
        },
        SimulationControlInput::LogicalTick {
            source_id: "source.timer".to_string(),
            tick: 9,
        },
        SimulationControlInput::FiniteChoice {
            choice_point: SimulationChoicePointId::new("choice.unknown"),
            bound: 0,
            selected: 1,
        },
        SimulationControlInput::EventSelection {
            ready_sources: vec!["source.z".to_string(), "source.a".to_string()],
            selected_source: "source.missing".to_string(),
        },
        SimulationControlInput::EffectResult {
            request_id: "request.same".to_string(),
            effect_class: SimulationEffectClassId::new("simulation.effect.persist-layout-v1"),
            result: SimulationEffectResult::Failed,
        },
        SimulationControlInput::EffectResult {
            request_id: "request.same".to_string(),
            effect_class: SimulationEffectClassId::new("simulation.effect.persist-layout-v1"),
            result: SimulationEffectResult::Completed,
        },
    ];
    let codes: Vec<_> = validate_control_history(&super::registry(), &history).iter().map(|issue| issue.code).collect();

    assert!(codes.contains(&SimulationControlDiagnosticCode::RegressingLogicalTick));
    assert!(codes.contains(&SimulationControlDiagnosticCode::UnknownChoicePoint));
    assert!(codes.contains(&SimulationControlDiagnosticCode::EmptyChoiceSet));
    assert!(codes.contains(&SimulationControlDiagnosticCode::InvalidReadySources));
    assert!(codes.contains(&SimulationControlDiagnosticCode::SelectedSourceNotReady));
    assert!(codes.contains(&SimulationControlDiagnosticCode::DuplicateEffectResult));
}

#[test]
fn host_clock_random_and_external_io_are_explicit_failures() {
    let history = [
        SimulationControlInput::ForbiddenSource(ForbiddenSimulationSource::HostClock),
        SimulationControlInput::ForbiddenSource(ForbiddenSimulationSource::HostRandom),
        SimulationControlInput::ForbiddenSource(ForbiddenSimulationSource::DirectExternalIo),
    ];
    let codes: Vec<_> = validate_control_history(&super::registry(), &history).iter().map(|issue| issue.code).collect();

    assert_eq!(codes, vec![
        SimulationControlDiagnosticCode::HostClockForbidden,
        SimulationControlDiagnosticCode::HostRandomForbidden,
        SimulationControlDiagnosticCode::DirectExternalIoForbidden,
    ]);
}

#[test]
fn effect_tracker_covers_success_failure_cancellation_and_source_outcomes() {
    for (index, result) in [
        SimulationEffectResult::Completed,
        SimulationEffectResult::Failed,
        SimulationEffectResult::Cancelled,
        SimulationEffectResult::Replaced,
        SimulationEffectResult::TimedOut,
        SimulationEffectResult::SourceClosed,
        SimulationEffectResult::ExternallyControlledFailure,
    ]
    .into_iter()
    .enumerate()
    {
        let mut tracker = SimulationEffectTracker::default();
        let request = format!("request.lifecycle-{index}");
        let class = SimulationEffectClassId::new("simulation.effect.persist-layout-v1");
        tracker.observe(&request, &class, SimulationEffectLifecycle::Requested).expect("request accepted");
        tracker.observe(&request, &class, SimulationEffectLifecycle::Started).expect("start accepted");
        tracker
            .observe(&request, &class, SimulationEffectLifecycle::Result(result))
            .expect("terminal result accepted");
        assert_eq!(
            tracker
                .observe(&request, &class, SimulationEffectLifecycle::Result(result),)
                .expect_err("duplicate terminal result must fail")
                .code,
            SimulationEffectDiagnosticCode::DuplicateOrLateResult
        );
    }
}

#[test]
fn missing_adapter_unknown_request_and_replacement_class_mismatch_fail() {
    let mut tracker = SimulationEffectTracker::default();
    let class = SimulationEffectClassId::new("simulation.effect.persist-layout-v1");
    let other = SimulationEffectClassId::new("simulation.effect.other-v1");
    assert_eq!(
        tracker
            .observe("request.missing", &class, SimulationEffectLifecycle::Started)
            .expect_err("missing request")
            .code,
        SimulationEffectDiagnosticCode::UnknownRequest
    );
    tracker
        .observe("request.known", &class, SimulationEffectLifecycle::Requested)
        .expect("request accepted");
    assert_eq!(
        tracker
            .observe("request.known", &other, SimulationEffectLifecycle::Started)
            .expect_err("effect class cannot change")
            .code,
        SimulationEffectDiagnosticCode::EffectClassMismatch
    );
}
