use super::*;

#[test]
fn preset_defaults_match_expected_intervals() {
    let dots = SpinnerSpec::dots();
    let pulse = SpinnerSpec::pulse();

    assert_eq!(dots.interval(), core::time::Duration::from_millis(DOTS_INTERVAL_MS));
    assert_eq!(pulse.interval(), core::time::Duration::from_millis(PULSE_INTERVAL_MS));
    assert_eq!(dots.frames()[0], "⠋");
    assert_eq!(pulse.frames()[0], "∙∙∙");
}

#[test]
fn tick_cycles_custom_frames() {
    let frames = &["a", "b", "c"];
    let spec = SpinnerSpec::custom(frames);
    let mut state = SpinnerState::new();

    assert_eq!(state.current_frame(&spec), "a");
    state.tick(&spec);
    assert_eq!(state.current_frame(&spec), "b");
    state.tick(&spec);
    assert_eq!(state.current_frame(&spec), "c");
    state.tick(&spec);
    assert_eq!(state.current_frame(&spec), "a");
}

#[test]
fn reverse_reads_frames_from_end() {
    let frames = &["a", "b", "c"];
    let spec = SpinnerSpec::custom(frames).with_reversed(true);
    let mut state = SpinnerState::new();

    assert_eq!(state.current_frame(&spec), "c");
    state.tick(&spec);
    assert_eq!(state.current_frame(&spec), "b");
    state.tick(&spec);
    assert_eq!(state.current_frame(&spec), "a");
}

#[test]
fn advance_respects_interval() {
    let spec = SpinnerSpec::line().with_interval(core::time::Duration::from_millis(PULSE_INTERVAL_MS));
    let mut state = SpinnerState::new();

    state.advance(&spec, core::time::Duration::from_millis(99));
    assert_eq!(state.current_frame(&spec), "-");
    state.advance(&spec, core::time::Duration::from_millis(1));
    assert_eq!(state.current_frame(&spec), "\\");
    state.advance(&spec, core::time::Duration::from_millis(250));
    assert_eq!(state.current_frame(&spec), "/");
}

#[test]
fn empty_custom_spinner_is_safe() {
    let spec = SpinnerSpec::custom(&[]);
    let mut state = SpinnerState::new();

    state.tick(&spec);
    state.advance(&spec, core::time::Duration::from_millis(10));
    assert_eq!(state.current_frame(&spec), "");
}
