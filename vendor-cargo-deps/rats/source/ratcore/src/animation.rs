//! Renderer-neutral animation intent, adapter projection, and parity evidence.
//!
//! This module is the functional core for animation parity. It accepts only
//! in-memory data, explicit elapsed-time inputs, and renderer labels; it never
//! reads clocks, files, terminal handles, browser APIs, or environment state.

use std::time::Duration;

const EASING_LINEAR: &str = "linear";
const EASING_QUAD_IN: &str = "quad-in";
const EASING_QUAD_OUT: &str = "quad-out";
const EFFECT_FADE: &str = "fade";
const EFFECT_HIGHLIGHT: &str = "highlight";
const EFFECT_SLIDE: &str = "slide";
const PLAYBACK_ONCE: &str = "once";
const REDUCED_FULL_MOTION: &str = "full-motion";
const REDUCED_NOOP: &str = "noop";
const REDUCED_SHORTEN: &str = "shorten";
const REDUCED_SIMPLIFY: &str = "simplify";
const REDUCED_SNAP: &str = "snap";
const RENDERER_METEORITE_CSS: &str = "meteorite-css";
const RENDERER_MISSING: &str = "missing-adapter";
const RENDERER_SUBWAYRAT_TACHYONFX: &str = "subwayrat-tachyonfx";
const SNAP_FAMILY: &str = "reduced-motion-snap";
const SIMPLIFIED_SUFFIX: &str = ":simplified";
const NOOP_DIAGNOSTIC_SUBJECT: &str = "reduced_motion";
const HASH_SEPARATOR: &str = "|";
const ZERO_PROGRESS: NormalizedAnimationProgress = NormalizedAnimationProgress::ZERO;
const FULL_PROGRESS: NormalizedAnimationProgress = NormalizedAnimationProgress::FULL;
const FULL_PROGRESS_U64: u64 = 10_000;
const REDUCED_MOTION_SHORTEN_DIVISOR: u32 = 2;
const EMPTY_ISSUE_BOUND: usize = 0;
const SINGLE_ISSUE_BOUND: usize = 1;
const INTENT_ISSUE_BOUND: usize = 5;
const PROJECTION_ISSUE_BOUND: usize = 2;
const KNOWN_EFFECTS: &[&str] = &[EFFECT_FADE, EFFECT_HIGHLIGHT, EFFECT_SLIDE];
const KNOWN_EASINGS: &[&str] = &[EASING_LINEAR, EASING_QUAD_IN, EASING_QUAD_OUT];
const KNOWN_PLAYBACK_POLICIES: &[&str] = &[PLAYBACK_ONCE];
const KNOWN_REDUCED_MOTION: &[&str] = &[
    REDUCED_FULL_MOTION,
    REDUCED_NOOP,
    REDUCED_SHORTEN,
    REDUCED_SIMPLIFY,
    REDUCED_SNAP,
];

/// Stable diagnostic code for animation validation and projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AnimationDiagnosticCode {
    /// Required animation ID, effect family, or target scope is missing.
    MissingRequiredField,
    /// Duration is zero or otherwise unusable.
    InvalidTiming,
    /// Easing name is not in the accepted vocabulary.
    UnknownEasing,
    /// Effect family is not in the shared semantic vocabulary.
    UnsupportedEffectFamily,
    /// Playback policy is not supported by the shared reducer.
    UnsupportedPlaybackPolicy,
    /// Animation ID appears more than once in the same validation set.
    DuplicateAnimationId,
    /// The renderer adapter cannot project the shared request.
    MissingAdapterSupport,
    /// Reduced motion intentionally produced a no-op projection.
    ReducedMotionNoop,
}

/// Deterministic animation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationDiagnostic {
    /// Stable diagnostic code.
    pub code: AnimationDiagnosticCode,
    /// Animation ID or fallback subject.
    pub animation_id: String,
    /// Field or renderer that triggered the diagnostic.
    pub subject: String,
    /// Human-readable diagnostic text.
    pub message: String,
}

impl AnimationDiagnostic {
    fn new(
        code: AnimationDiagnosticCode,
        animation_id: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            animation_id: animation_id.into(),
            subject: subject.into(),
            message: message.into(),
        }
    }
}

/// Normalized animation progress in inclusive ten-thousandths `0..=10_000`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedAnimationProgress(u32);

impl NormalizedAnimationProgress {
    /// No progress.
    pub const ZERO: Self = Self(0);
    /// Complete progress.
    pub const FULL: Self = Self(10_000);

    /// Validates a ten-thousandth progress value.
    pub const fn try_new(value: u32) -> Result<Self, AnimationProgressError> {
        if value <= Self::FULL.0 {
            Ok(Self(value))
        } else {
            Err(AnimationProgressError { value })
        }
    }

    /// Returns the exact ten-thousandth scalar.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for NormalizedAnimationProgress {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Rejected normalized animation progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationProgressError {
    /// Rejected ten-thousandth value.
    pub value: u32,
}

impl std::fmt::Display for AnimationProgressError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "animation progress {} exceeds 10_000", self.value)
    }
}

impl std::error::Error for AnimationProgressError {}

/// Renderer-neutral animation intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationIntent {
    /// Stable animation ID.
    pub animation_id: String,
    /// Semantic effect family, such as `fade`, `highlight`, or `slide`.
    pub effect_family: String,
    /// Renderer-neutral target scope ID.
    pub target_scope: String,
    /// Positive animation duration.
    duration: Duration,
    /// Nonnegative animation delay.
    delay: Duration,
    /// Easing vocabulary value.
    pub easing: String,
    /// Playback policy vocabulary value.
    pub playback_policy: String,
    /// Reduced-motion strategy vocabulary value.
    pub reduced_motion: String,
}

impl AnimationIntent {
    /// Creates an animation intent with deterministic defaults.
    #[must_use]
    pub fn new(
        animation_id: impl Into<String>,
        effect_family: impl Into<String>,
        target_scope: impl Into<String>,
        duration: Duration,
    ) -> Result<Self, AnimationTimingError> {
        if duration.is_zero() {
            return Err(AnimationTimingError::ZeroDuration);
        }
        Ok(Self {
            animation_id: animation_id.into(),
            effect_family: effect_family.into(),
            target_scope: target_scope.into(),
            duration,
            delay: Duration::ZERO,
            easing: EASING_LINEAR.to_string(),
            playback_policy: PLAYBACK_ONCE.to_string(),
            reduced_motion: REDUCED_FULL_MOTION.to_string(),
        })
    }

    /// Creates an intent from a compatibility duration in milliseconds.
    #[must_use]
    pub fn new_ms(
        animation_id: impl Into<String>,
        effect_family: impl Into<String>,
        target_scope: impl Into<String>,
        duration_ms: u32,
    ) -> Result<Self, AnimationTimingError> {
        Self::new(animation_id, effect_family, target_scope, Duration::from_millis(u64::from(duration_ms)))
    }

    /// Returns the positive animation duration.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the nonnegative animation delay.
    #[must_use]
    pub const fn delay(&self) -> Duration {
        self.delay
    }

    /// Sets a nonnegative delay.
    #[must_use]
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Sets a compatibility delay in explicit milliseconds.
    #[must_use]
    pub fn with_delay_ms(self, delay_ms: u32) -> Self {
        self.with_delay(Duration::from_millis(u64::from(delay_ms)))
    }

    /// Sets the easing vocabulary value.
    #[must_use]
    pub fn with_easing(mut self, easing: impl Into<String>) -> Self {
        self.easing = easing.into();
        self
    }

    /// Sets the playback policy vocabulary value.
    #[must_use]
    pub fn with_playback_policy(mut self, playback_policy: impl Into<String>) -> Self {
        self.playback_policy = playback_policy.into();
        self
    }

    /// Sets the reduced-motion strategy vocabulary value.
    #[must_use]
    pub fn with_reduced_motion(mut self, reduced_motion: impl Into<String>) -> Self {
        self.reduced_motion = reduced_motion.into();
        self
    }
}

/// Rejected animation timing input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationTimingError {
    /// Duration was zero.
    ZeroDuration,
}

impl std::fmt::Display for AnimationTimingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("animation duration must be greater than zero")
    }
}

impl std::error::Error for AnimationTimingError {}

/// Raw millisecond timing DTO retained at compatibility boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawAnimationTiming {
    /// Duration encoded in milliseconds.
    pub duration_ms: u32,
    /// Delay encoded in milliseconds.
    pub delay_ms: u32,
}

impl RawAnimationTiming {
    /// Atomically validates the complete raw timing record.
    pub fn validate(self) -> Result<(Duration, Duration), AnimationTimingError> {
        let duration = Duration::from_millis(u64::from(self.duration_ms));
        if duration.is_zero() {
            return Err(AnimationTimingError::ZeroDuration);
        }
        Ok((duration, Duration::from_millis(u64::from(self.delay_ms))))
    }
}

/// One shared effect request emitted by the reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationEffectRequest {
    /// Stable animation ID.
    pub animation_id: String,
    /// Semantic effect family.
    pub effect_family: String,
    /// Renderer-neutral target scope.
    pub target_scope: String,
    /// Normalized progress in ten-thousandths, from `0` to `10_000`.
    pub progress: NormalizedAnimationProgress,
}

/// Shared animation progress after one explicit reducer step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationProgress {
    /// Stable animation ID.
    pub animation_id: String,
    /// Normalized progress in ten-thousandths, from `0` to `10_000`.
    pub progress: NormalizedAnimationProgress,
    /// Whether the shared timeline reached completion.
    pub completed: bool,
    /// Effect requests available to renderer adapters.
    pub emitted_effects: Vec<AnimationEffectRequest>,
    /// Deterministic reducer diagnostics.
    pub diagnostics: Vec<AnimationDiagnostic>,
}

impl AnimationProgress {
    /// Returns true when no diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Validation report for a set of shared animation intents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationValidationReport {
    /// Deterministic validation diagnostics.
    pub diagnostics: Vec<AnimationDiagnostic>,
    /// Intents sorted by stable animation ID.
    pub intents: Vec<AnimationIntent>,
}

impl AnimationValidationReport {
    /// Returns true when all intents are accepted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Renderer adapter target for projecting shared animation requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationRendererAdapter {
    /// Subwayrat terminal projection through a TachyonFX-owned adapter.
    SubwayratTachyonFx,
    /// Meteorite DOM projection through Dioxus state and CSS classes/variables.
    MeteoriteCss,
    /// Deliberately unavailable adapter used for fail-closed diagnostics.
    Missing,
}

impl AnimationRendererAdapter {
    /// Stable renderer label used in parity evidence.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SubwayratTachyonFx => RENDERER_SUBWAYRAT_TACHYONFX,
            Self::MeteoriteCss => RENDERER_METEORITE_CSS,
            Self::Missing => RENDERER_MISSING,
        }
    }
}

/// Renderer-owned animation projection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererAnimationRequest {
    /// Stable animation ID.
    pub animation_id: String,
    /// Renderer label.
    pub renderer: String,
    /// Renderer-owned implementation hint.
    pub implementation: String,
    /// Target scope after adapter mapping.
    pub target_scope: String,
    /// Normalized progress in ten-thousandths.
    pub progress: NormalizedAnimationProgress,
}

/// Projection result for one renderer adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RendererAnimationProjection {
    /// Renderer label.
    pub renderer: String,
    /// Projected renderer requests.
    pub requests: Vec<RendererAnimationRequest>,
    /// Projection diagnostics.
    pub diagnostics: Vec<AnimationDiagnostic>,
}

impl RendererAnimationProjection {
    /// Returns true when no projection diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Deterministic animation parity evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationProjectionEvidence {
    /// Fixture ID associated with the projection.
    pub fixture_id: String,
    /// Renderer label.
    pub renderer: String,
    /// BLAKE3 hash of the shared logical state.
    pub shared_state_hash: String,
    /// BLAKE3 hash of the renderer projection snapshot.
    pub logical_snapshot_hash: String,
    /// Renderer artifact path or generated fixture path.
    pub artifact_path: String,
    /// BLAKE3 hash of the renderer artifact material.
    pub artifact_hash: String,
    /// Deterministic diagnostics summary.
    pub diagnostics: Vec<String>,
    /// Documented divergence reason when visuals differ.
    pub divergence_reason: String,
}

/// Validates shared animation intents and returns deterministic diagnostics.
#[must_use]
pub fn validate_animation_intents(intents: &[AnimationIntent]) -> AnimationValidationReport {
    let mut diagnostics = Vec::with_capacity(intents.len());
    let mut seen_ids = std::collections::BTreeSet::new();
    for intent in intents {
        if !seen_ids.insert(intent.animation_id.clone()) {
            diagnostics.push(AnimationDiagnostic::new(
                AnimationDiagnosticCode::DuplicateAnimationId,
                intent.animation_id.clone(),
                "animation_id",
                "duplicate animation ID",
            ));
        }
        diagnostics.extend(intent_diagnostics(intent));
    }
    let mut sorted = intents.to_vec();
    sorted.sort_by(|left, right| left.animation_id.cmp(&right.animation_id));
    AnimationValidationReport {
        diagnostics,
        intents: sorted,
    }
}

/// Advances one shared animation from explicit elapsed-time input.
#[must_use]
pub fn advance_animation(
    intent: &AnimationIntent,
    elapsed: Duration,
    reduced_motion_enabled: bool,
) -> AnimationProgress {
    let diagnostics = intent_diagnostics(intent);
    if !diagnostics.is_empty() {
        return AnimationProgress {
            animation_id: intent.animation_id.clone(),
            progress: ZERO_PROGRESS,
            completed: false,
            emitted_effects: Vec::new(),
            diagnostics,
        };
    }

    let strategy = if reduced_motion_enabled {
        intent.reduced_motion.as_str()
    } else {
        REDUCED_FULL_MOTION
    };
    if strategy == REDUCED_NOOP {
        return reduced_noop_progress(intent);
    }
    if strategy == REDUCED_SNAP {
        return reduced_snap_progress(intent);
    }

    let effective_duration = effective_duration(intent.duration, strategy);
    let active_elapsed = elapsed.checked_sub(intent.delay).unwrap_or(Duration::ZERO);
    let linear_progress = normalize_progress(active_elapsed, effective_duration);
    let eased_progress = apply_easing(linear_progress, intent.easing.as_str());
    let completed = eased_progress >= FULL_PROGRESS;
    let family = projected_family(intent.effect_family.as_str(), strategy);
    AnimationProgress {
        animation_id: intent.animation_id.clone(),
        progress: eased_progress,
        completed,
        emitted_effects: vec![AnimationEffectRequest {
            animation_id: intent.animation_id.clone(),
            effect_family: family,
            target_scope: intent.target_scope.clone(),
            progress: eased_progress,
        }],
        diagnostics: Vec::new(),
    }
}

/// Advances an animation from a compatibility elapsed-millisecond value.
#[must_use]
pub fn advance_animation_ms(
    intent: &AnimationIntent,
    elapsed_ms: u32,
    reduced_motion_enabled: bool,
) -> AnimationProgress {
    advance_animation(intent, Duration::from_millis(u64::from(elapsed_ms)), reduced_motion_enabled)
}

/// Projects shared animation requests into a renderer-owned adapter contract.
#[must_use]
pub fn project_animation(
    progress: &AnimationProgress,
    adapter: AnimationRendererAdapter,
) -> RendererAnimationProjection {
    let mut projection = RendererAnimationProjection {
        renderer: adapter.label().to_string(),
        requests: Vec::with_capacity(progress.emitted_effects.len()),
        diagnostics: Vec::with_capacity(PROJECTION_ISSUE_BOUND),
    };
    if adapter == AnimationRendererAdapter::Missing {
        projection.diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::MissingAdapterSupport,
            progress.animation_id.clone(),
            adapter.label(),
            "animation renderer adapter is unavailable",
        ));
        return projection;
    }
    projection.diagnostics.extend(progress.diagnostics.iter().cloned());
    for request in &progress.emitted_effects {
        match implementation_for(adapter, request.effect_family.as_str()) {
            Some(implementation) => projection.requests.push(RendererAnimationRequest {
                animation_id: request.animation_id.clone(),
                renderer: adapter.label().to_string(),
                implementation,
                target_scope: request.target_scope.clone(),
                progress: request.progress,
            }),
            None => projection.diagnostics.push(AnimationDiagnostic::new(
                AnimationDiagnosticCode::MissingAdapterSupport,
                request.animation_id.clone(),
                adapter.label(),
                format!("renderer adapter `{}` does not support `{}`", adapter.label(), request.effect_family),
            )),
        }
    }
    projection
}

/// Builds deterministic projection evidence and hashes it with BLAKE3.
#[must_use]
pub fn animation_projection_evidence(
    fixture_id: impl Into<String>,
    projection: &RendererAnimationProjection,
    artifact_path: impl Into<String>,
    divergence_reason: impl Into<String>,
) -> AnimationProjectionEvidence {
    let fixture_id = fixture_id.into();
    let artifact_path = artifact_path.into();
    let divergence_reason = divergence_reason.into();
    let shared_state_material = shared_state_material(projection);
    let snapshot_material = snapshot_material(projection, &artifact_path, &divergence_reason);
    let artifact_material =
        format!("{}{HASH_SEPARATOR}{}{HASH_SEPARATOR}{}", projection.renderer, artifact_path, snapshot_material);
    AnimationProjectionEvidence {
        fixture_id,
        renderer: projection.renderer.clone(),
        shared_state_hash: blake3_hash(&shared_state_material),
        logical_snapshot_hash: blake3_hash(&snapshot_material),
        artifact_path,
        artifact_hash: blake3_hash(&artifact_material),
        diagnostics: projection
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{:?}:{}", diagnostic.code, diagnostic.subject))
            .collect(),
        divergence_reason,
    }
}

fn intent_diagnostics(intent: &AnimationIntent) -> Vec<AnimationDiagnostic> {
    let mut diagnostics = Vec::with_capacity(INTENT_ISSUE_BOUND);
    if intent.animation_id.trim().is_empty()
        || intent.effect_family.trim().is_empty()
        || intent.target_scope.trim().is_empty()
    {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::MissingRequiredField,
            intent.animation_id.clone(),
            "intent",
            "animation ID, effect family, and target scope are required",
        ));
    }
    if intent.duration.is_zero() {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::InvalidTiming,
            intent.animation_id.clone(),
            "duration",
            "duration must be greater than zero",
        ));
    }
    if !KNOWN_EFFECTS.contains(&intent.effect_family.as_str()) {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::UnsupportedEffectFamily,
            intent.animation_id.clone(),
            "effect_family",
            format!("unsupported animation effect family `{}`", intent.effect_family),
        ));
    }
    if !KNOWN_EASINGS.contains(&intent.easing.as_str()) {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::UnknownEasing,
            intent.animation_id.clone(),
            "easing",
            format!("unknown animation easing `{}`", intent.easing),
        ));
    }
    if !KNOWN_PLAYBACK_POLICIES.contains(&intent.playback_policy.as_str()) {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::UnsupportedPlaybackPolicy,
            intent.animation_id.clone(),
            "playback_policy",
            format!("unsupported animation playback policy `{}`", intent.playback_policy),
        ));
    }
    if !KNOWN_REDUCED_MOTION.contains(&intent.reduced_motion.as_str()) {
        diagnostics.push(AnimationDiagnostic::new(
            AnimationDiagnosticCode::UnsupportedPlaybackPolicy,
            intent.animation_id.clone(),
            "reduced_motion",
            format!("unsupported reduced-motion strategy `{}`", intent.reduced_motion),
        ));
    }
    diagnostics
}

fn reduced_noop_progress(intent: &AnimationIntent) -> AnimationProgress {
    AnimationProgress {
        animation_id: intent.animation_id.clone(),
        progress: FULL_PROGRESS,
        completed: true,
        emitted_effects: Vec::with_capacity(EMPTY_ISSUE_BOUND),
        diagnostics: vec![AnimationDiagnostic::new(
            AnimationDiagnosticCode::ReducedMotionNoop,
            intent.animation_id.clone(),
            NOOP_DIAGNOSTIC_SUBJECT,
            "reduced motion produced a deterministic no-op",
        )],
    }
}

fn reduced_snap_progress(intent: &AnimationIntent) -> AnimationProgress {
    AnimationProgress {
        animation_id: intent.animation_id.clone(),
        progress: FULL_PROGRESS,
        completed: true,
        emitted_effects: vec![AnimationEffectRequest {
            animation_id: intent.animation_id.clone(),
            effect_family: SNAP_FAMILY.to_string(),
            target_scope: intent.target_scope.clone(),
            progress: FULL_PROGRESS,
        }],
        diagnostics: Vec::with_capacity(SINGLE_ISSUE_BOUND),
    }
}

fn effective_duration(duration: Duration, strategy: &str) -> Duration {
    if strategy == REDUCED_SHORTEN {
        return duration
            .checked_div(REDUCED_MOTION_SHORTEN_DIVISOR)
            .filter(|shortened| !shortened.is_zero())
            .unwrap_or(duration);
    }
    duration
}

fn projected_family(effect_family: &str, strategy: &str) -> String {
    if strategy == REDUCED_SIMPLIFY {
        return format!("{effect_family}{SIMPLIFIED_SUFFIX}");
    }
    effect_family.to_string()
}

fn normalize_progress(active_elapsed: Duration, duration: Duration) -> NormalizedAnimationProgress {
    if active_elapsed >= duration {
        return FULL_PROGRESS;
    }
    let numerator = active_elapsed.as_nanos().saturating_mul(u128::from(FULL_PROGRESS.get()));
    let raw = numerator / duration.as_nanos();
    let bounded = u32::try_from(raw.min(u128::from(FULL_PROGRESS.get()))).unwrap_or(FULL_PROGRESS.get());
    NormalizedAnimationProgress::try_new(bounded).unwrap_or(FULL_PROGRESS)
}

fn apply_easing(progress: NormalizedAnimationProgress, easing: &str) -> NormalizedAnimationProgress {
    match easing {
        EASING_LINEAR => progress,
        EASING_QUAD_IN => quadratic_in(progress),
        EASING_QUAD_OUT => quadratic_out(progress),
        _ => progress,
    }
}

fn quadratic_in(progress: NormalizedAnimationProgress) -> NormalizedAnimationProgress {
    let scalar = u64::from(progress.get());
    let squared = scalar * scalar / FULL_PROGRESS_U64;
    let bounded = u32::try_from(squared.min(FULL_PROGRESS_U64)).unwrap_or(FULL_PROGRESS.get());
    NormalizedAnimationProgress::try_new(bounded).unwrap_or(FULL_PROGRESS)
}

fn quadratic_out(progress: NormalizedAnimationProgress) -> NormalizedAnimationProgress {
    let inverse = FULL_PROGRESS.get().saturating_sub(progress.get());
    let eased_inverse =
        quadratic_in(NormalizedAnimationProgress::try_new(inverse).unwrap_or(NormalizedAnimationProgress::ZERO));
    NormalizedAnimationProgress::try_new(FULL_PROGRESS.get().saturating_sub(eased_inverse.get()))
        .unwrap_or(FULL_PROGRESS)
}

fn implementation_for(adapter: AnimationRendererAdapter, effect_family: &str) -> Option<String> {
    match adapter {
        AnimationRendererAdapter::SubwayratTachyonFx => subwayrat_tachyonfx_implementation(effect_family),
        AnimationRendererAdapter::MeteoriteCss => meteorite_css_implementation(effect_family),
        AnimationRendererAdapter::Missing => None,
    }
}

fn subwayrat_tachyonfx_implementation(effect_family: &str) -> Option<String> {
    match effect_family {
        EFFECT_FADE => Some("tachyonfx::fx::fade".to_string()),
        EFFECT_HIGHLIGHT => Some("tachyonfx::fx::highlight".to_string()),
        SNAP_FAMILY => Some("tachyonfx::snap".to_string()),
        family if family.ends_with(SIMPLIFIED_SUFFIX) => Some("tachyonfx::reduced-motion".to_string()),
        _ => None,
    }
}

fn meteorite_css_implementation(effect_family: &str) -> Option<String> {
    match effect_family {
        EFFECT_FADE => Some("css:opacity-transition".to_string()),
        EFFECT_HIGHLIGHT => Some("css:highlight-token".to_string()),
        EFFECT_SLIDE => Some("css:transform-transition".to_string()),
        SNAP_FAMILY => Some("css:reduced-motion-snap".to_string()),
        family if family.ends_with(SIMPLIFIED_SUFFIX) => Some("css:reduced-motion-simple".to_string()),
        _ => None,
    }
}

fn shared_state_material(projection: &RendererAnimationProjection) -> String {
    let mut material = String::new();
    for request in &projection.requests {
        material.push_str(&format!(
            "{}{}{}{}{}{}{}\n",
            request.animation_id,
            HASH_SEPARATOR,
            request.target_scope,
            HASH_SEPARATOR,
            request.progress,
            HASH_SEPARATOR,
            request.implementation
        ));
    }
    for diagnostic in &projection.diagnostics {
        material.push_str(&format!(
            "{:?}{}{}{}{}\n",
            diagnostic.code, HASH_SEPARATOR, diagnostic.animation_id, HASH_SEPARATOR, diagnostic.subject
        ));
    }
    material
}

fn snapshot_material(projection: &RendererAnimationProjection, artifact_path: &str, divergence_reason: &str) -> String {
    format!(
        "{}{}{}{}{}{}{}",
        projection.renderer,
        HASH_SEPARATOR,
        artifact_path,
        HASH_SEPARATOR,
        divergence_reason,
        HASH_SEPARATOR,
        shared_state_material(projection)
    )
}

fn blake3_hash(material: &str) -> String {
    blake3::hash(material.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
#[path = "animation/tests.rs"]
mod tests;
