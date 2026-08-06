use super::*;

const ANIMATION_FADE_IN: &str = "animation.fade-in";
const ANIMATION_HIGHLIGHT: &str = "animation.highlight";
const ANIMATION_SLIDE: &str = "animation.slide";
const TARGET_COMMAND_ROW: &str = "command-row";
const TARGET_STATUS_BADGE: &str = "status-badge";
const DURATION_MS: u32 = 500;
const HALF_ELAPSED_MS: u32 = 250;
const PAST_END_ELAPSED_MS: u32 = 900;
const ZERO_DURATION_MS: u32 = 0;
const EXPECTED_HALF_PROGRESS: u32 = 5_000;
const EXPECTED_QUAD_IN_HALF_PROGRESS: u32 = 2_500;
const EXPECTED_SHORTENED_FULL_PROGRESS: u32 = 10_000;
const EMPTY_ANIMATION_ID: &str = "";
const EMPTY_TARGET: &str = "";
const UNKNOWN_EASING: &str = "spring";
const UNKNOWN_EFFECT: &str = "sparkle";
const UNSUPPORTED_PLAYBACK: &str = "infinite";
const FIXTURE_ANIMATION: &str = "animation.parity.fade";
const SUBWAYRAT_ARTIFACT: &str = "docs/generated/animation/fade.subwayrat.txt";
const METEORITE_ARTIFACT: &str = "docs/generated/animation/fade.meteorite.html";
const DIVERGENCE_REASON: &str = "terminal cells and CSS transitions differ visually but share logical state";
const HASH_HEX_LENGTH: usize = 64;

fn intent(id: &str, effect: &str, target: &str, duration_ms: u32) -> AnimationIntent {
    AnimationIntent::new(id, effect, target, Duration::from_millis(u64::from(duration_ms)))
        .expect("test intent duration is positive")
}

fn advance(model: &AnimationIntent, elapsed_ms: u32, reduced: bool) -> AnimationProgress {
    advance_animation(model, Duration::from_millis(u64::from(elapsed_ms)), reduced)
}

#[test]
fn duration_and_progress_boundaries_are_typed() {
    let model =
        intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS).with_delay(Duration::from_millis(10));
    assert_eq!(model.duration(), Duration::from_millis(500));
    assert_eq!(model.delay(), Duration::from_millis(10));
    assert_eq!(NormalizedAnimationProgress::try_new(10_001), Err(AnimationProgressError { value: 10_001 }));
}

#[test]
fn reducer_advances_timeline_from_explicit_elapsed_input() {
    let model = intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS);
    let progress = advance(&model, HALF_ELAPSED_MS, false);

    assert!(progress.is_valid(), "diagnostics: {:?}", progress.diagnostics);
    assert_eq!(progress.animation_id, ANIMATION_FADE_IN);
    assert_eq!(progress.progress.get(), EXPECTED_HALF_PROGRESS);
    assert!(!progress.completed);
    assert_eq!(progress.emitted_effects.len(), 1);
    assert_eq!(progress.emitted_effects[0].effect_family, EFFECT_FADE);
    assert_eq!(progress.emitted_effects[0].target_scope, TARGET_COMMAND_ROW);
}

#[test]
fn reducer_applies_named_easing_without_clocks() {
    let model =
        intent(ANIMATION_HIGHLIGHT, EFFECT_HIGHLIGHT, TARGET_STATUS_BADGE, DURATION_MS).with_easing(EASING_QUAD_IN);
    let progress = advance(&model, HALF_ELAPSED_MS, false);

    assert!(progress.is_valid(), "diagnostics: {:?}", progress.diagnostics);
    assert_eq!(progress.progress.get(), EXPECTED_QUAD_IN_HALF_PROGRESS);
}

#[test]
fn validation_rejects_malformed_animation_inputs() {
    let duplicate = intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS);
    assert_eq!(
        RawAnimationTiming {
            duration_ms: ZERO_DURATION_MS,
            delay_ms: 10,
        }
        .validate(),
        Err(AnimationTimingError::ZeroDuration)
    );
    let missing_target = intent(EMPTY_ANIMATION_ID, EFFECT_FADE, EMPTY_TARGET, DURATION_MS);
    let unknown_easing =
        intent("animation.bad-easing", EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS).with_easing(UNKNOWN_EASING);
    let unsupported_effect = intent("animation.unsupported-effect", UNKNOWN_EFFECT, TARGET_COMMAND_ROW, DURATION_MS);
    let unsupported_playback = intent("animation.unsupported-playback", EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS)
        .with_playback_policy(UNSUPPORTED_PLAYBACK);

    let report = validate_animation_intents(&[
        duplicate.clone(),
        duplicate,
        missing_target,
        unknown_easing,
        unsupported_effect,
        unsupported_playback,
    ]);

    assert!(!report.is_valid());
    assert_diagnostic(&report.diagnostics, AnimationDiagnosticCode::DuplicateAnimationId);
    assert_diagnostic(&report.diagnostics, AnimationDiagnosticCode::MissingRequiredField);
    assert_diagnostic(&report.diagnostics, AnimationDiagnosticCode::UnknownEasing);
    assert_diagnostic(&report.diagnostics, AnimationDiagnosticCode::UnsupportedEffectFamily);
    assert_diagnostic(&report.diagnostics, AnimationDiagnosticCode::UnsupportedPlaybackPolicy);
}

#[test]
fn renderer_adapters_preserve_shared_animation_identity() {
    let model = intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS);
    let progress = advance(&model, HALF_ELAPSED_MS, false);
    let subwayrat = project_animation(&progress, AnimationRendererAdapter::SubwayratTachyonFx);
    let meteorite = project_animation(&progress, AnimationRendererAdapter::MeteoriteCss);

    assert!(subwayrat.is_valid(), "diagnostics: {:?}", subwayrat.diagnostics);
    assert!(meteorite.is_valid(), "diagnostics: {:?}", meteorite.diagnostics);
    assert_eq!(subwayrat.requests[0].animation_id, ANIMATION_FADE_IN);
    assert_eq!(meteorite.requests[0].animation_id, ANIMATION_FADE_IN);
    assert_eq!(subwayrat.requests[0].progress, meteorite.requests[0].progress);
    assert!(subwayrat.requests[0].implementation.contains("tachyonfx"));
    assert!(meteorite.requests[0].implementation.contains("css"));
}

#[test]
fn renderer_adapters_fail_closed_for_missing_support() {
    let slide = intent(ANIMATION_SLIDE, EFFECT_SLIDE, TARGET_COMMAND_ROW, DURATION_MS);
    let progress = advance(&slide, HALF_ELAPSED_MS, false);
    let subwayrat = project_animation(&progress, AnimationRendererAdapter::SubwayratTachyonFx);
    let missing = project_animation(&progress, AnimationRendererAdapter::Missing);

    assert!(!subwayrat.is_valid());
    assert!(subwayrat.requests.is_empty());
    assert_diagnostic(&subwayrat.diagnostics, AnimationDiagnosticCode::MissingAdapterSupport);
    assert!(!missing.is_valid());
    assert_diagnostic(&missing.diagnostics, AnimationDiagnosticCode::MissingAdapterSupport);
}

#[test]
fn reduced_motion_snap_and_noop_are_deterministic() {
    let snap =
        intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS).with_reduced_motion(REDUCED_SNAP);
    let noop = intent(ANIMATION_HIGHLIGHT, EFFECT_HIGHLIGHT, TARGET_STATUS_BADGE, DURATION_MS)
        .with_reduced_motion(REDUCED_NOOP);

    let snapped = advance(&snap, HALF_ELAPSED_MS, true);
    let nooped = advance(&noop, HALF_ELAPSED_MS, true);

    assert_eq!(snapped.progress, FULL_PROGRESS);
    assert!(snapped.completed);
    assert_eq!(snapped.emitted_effects[0].effect_family, SNAP_FAMILY);
    assert_eq!(nooped.progress, FULL_PROGRESS);
    assert!(nooped.completed);
    assert!(nooped.emitted_effects.is_empty());
    assert_diagnostic(&nooped.diagnostics, AnimationDiagnosticCode::ReducedMotionNoop);
}

#[test]
fn reduced_motion_shorten_completes_without_renderer_specific_state() {
    let model =
        intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS).with_reduced_motion(REDUCED_SHORTEN);
    let progress = advance(&model, HALF_ELAPSED_MS, true);

    assert_eq!(progress.progress.get(), EXPECTED_SHORTENED_FULL_PROGRESS);
    assert!(progress.completed);
}

#[test]
fn projection_evidence_hashes_shared_and_renderer_artifacts() {
    let model = intent(ANIMATION_FADE_IN, EFFECT_FADE, TARGET_COMMAND_ROW, DURATION_MS);
    let progress = advance(&model, PAST_END_ELAPSED_MS, false);
    let subwayrat = project_animation(&progress, AnimationRendererAdapter::SubwayratTachyonFx);
    let meteorite = project_animation(&progress, AnimationRendererAdapter::MeteoriteCss);

    let terminal_evidence =
        animation_projection_evidence(FIXTURE_ANIMATION, &subwayrat, SUBWAYRAT_ARTIFACT, DIVERGENCE_REASON);
    let dom_evidence =
        animation_projection_evidence(FIXTURE_ANIMATION, &meteorite, METEORITE_ARTIFACT, DIVERGENCE_REASON);

    assert_eq!(terminal_evidence.shared_state_hash.len(), HASH_HEX_LENGTH);
    assert_eq!(dom_evidence.logical_snapshot_hash.len(), HASH_HEX_LENGTH);
    assert_eq!(terminal_evidence.fixture_id, FIXTURE_ANIMATION);
    assert_eq!(dom_evidence.fixture_id, FIXTURE_ANIMATION);
    assert_ne!(terminal_evidence.artifact_hash, dom_evidence.artifact_hash);
}

fn assert_diagnostic(diagnostics: &[AnimationDiagnostic], code: AnimationDiagnosticCode) {
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == code));
}
