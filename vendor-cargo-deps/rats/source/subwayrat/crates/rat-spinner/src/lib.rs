//! Reusable spinner presets and animation state.

const DOTS_INTERVAL_MS: u64 = 80;
const LINE_INTERVAL_MS: u64 = 120;
const PULSE_INTERVAL_MS: u64 = 100;
const ARROW_INTERVAL_MS: u64 = 90;
const BOUNCE_INTERVAL_MS: u64 = 110;
const DEFAULT_INTERVAL_MS: u64 = DOTS_INTERVAL_MS;
const DOTS_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const LINE_FRAMES: &[&str] = &["-", "\\", "|", "/"];
const PULSE_FRAMES: &[&str] = &["∙∙∙", "●∙∙", "∙●∙", "∙∙●"];
const ARROW_FRAMES: &[&str] = &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"];
const BOUNCE_FRAMES: &[&str] = &["⠁", "⠂", "⠄", "⠂"];

#[derive(Debug, Clone)]
pub struct SpinnerSpec<'a> {
    frames: SpinnerFrames<'a>,
    interval: core::time::Duration,
    reversed: bool,
}

#[derive(Debug, Clone)]
pub enum SpinnerFrames<'a> {
    Preset(SpinnerPreset),
    Custom(&'a [&'a str]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerPreset {
    Dots,
    Line,
    Pulse,
    Arrow,
    Bounce,
}

#[derive(Debug, Clone, Default)]
pub struct SpinnerState {
    frame: usize,
    elapsed_ms: u128,
}

impl<'a> SpinnerSpec<'a> {
    pub fn new(preset: SpinnerPreset) -> Self {
        Self {
            frames: SpinnerFrames::Preset(preset),
            interval: preset.default_interval(),
            reversed: false,
        }
    }

    pub fn dots() -> Self {
        Self::new(SpinnerPreset::Dots)
    }

    pub fn line() -> Self {
        Self::new(SpinnerPreset::Line)
    }

    pub fn pulse() -> Self {
        Self::new(SpinnerPreset::Pulse)
    }

    pub fn arrow() -> Self {
        Self::new(SpinnerPreset::Arrow)
    }

    pub fn bounce() -> Self {
        Self::new(SpinnerPreset::Bounce)
    }

    pub fn custom(frames: &'a [&'a str]) -> Self {
        Self {
            frames: SpinnerFrames::Custom(frames),
            interval: core::time::Duration::from_millis(DEFAULT_INTERVAL_MS),
            reversed: false,
        }
    }

    pub fn with_interval(mut self, interval: core::time::Duration) -> Self {
        self.interval = normalize_interval(interval);
        self
    }

    pub fn with_reversed(mut self, reversed: bool) -> Self {
        self.reversed = reversed;
        self
    }

    pub fn interval(&self) -> core::time::Duration {
        self.interval
    }

    pub fn is_reversed(&self) -> bool {
        self.reversed
    }

    pub fn label(&self) -> &'static str {
        match &self.frames {
            SpinnerFrames::Preset(preset) => preset.label(),
            SpinnerFrames::Custom(_) => "custom",
        }
    }

    pub fn frames(&self) -> &'a [&'a str] {
        self.frames.as_slice()
    }
}

impl<'a> SpinnerFrames<'a> {
    fn as_slice(&self) -> &'a [&'a str] {
        match self {
            Self::Preset(preset) => preset.frames(),
            Self::Custom(frames) => frames,
        }
    }
}

impl SpinnerPreset {
    pub fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Dots => DOTS_FRAMES,
            Self::Line => LINE_FRAMES,
            Self::Pulse => PULSE_FRAMES,
            Self::Arrow => ARROW_FRAMES,
            Self::Bounce => BOUNCE_FRAMES,
        }
    }

    pub fn default_interval(self) -> core::time::Duration {
        match self {
            Self::Dots => core::time::Duration::from_millis(DOTS_INTERVAL_MS),
            Self::Line => core::time::Duration::from_millis(LINE_INTERVAL_MS),
            Self::Pulse => core::time::Duration::from_millis(PULSE_INTERVAL_MS),
            Self::Arrow => core::time::Duration::from_millis(ARROW_INTERVAL_MS),
            Self::Bounce => core::time::Duration::from_millis(BOUNCE_INTERVAL_MS),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dots => "dots",
            Self::Line => "line",
            Self::Pulse => "pulse",
            Self::Arrow => "arrow",
            Self::Bounce => "bounce",
        }
    }
}

impl SpinnerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, spinner: &SpinnerSpec<'_>) {
        let len = spinner.frames().len();
        if len == 0 {
            return;
        }
        self.frame = next_frame_index(FrameIndex(self.frame), FrameCount(len));
    }

    pub fn advance(&mut self, spinner: &SpinnerSpec<'_>, delta: core::time::Duration) {
        let frame_count = spinner.frames().len();
        if frame_count == 0 {
            return;
        }

        let interval_ms = nonzero_interval_ms(spinner.interval());
        if interval_ms == 0 {
            return;
        }

        self.elapsed_ms = self.elapsed_ms.saturating_add(delta.as_millis());
        let elapsed = U128Division {
            value: self.elapsed_ms,
            divisor: interval_ms,
        };
        let steps = checked_quotient_u128(elapsed);
        self.elapsed_ms = checked_remainder_u128(elapsed);

        if steps > 0 {
            self.frame = advance_frame_index(FrameIndex(self.frame), steps, FrameCount(frame_count));
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.elapsed_ms = 0;
    }

    pub fn current_frame<'a>(&self, spinner: &'a SpinnerSpec<'a>) -> &'a str {
        let frames = spinner.frames();
        let frame_count = frames.len();
        if frame_count == 0 {
            return "";
        }

        let frame_position = checked_remainder_usize(UsizeDivision {
            value: self.frame,
            divisor: frame_count,
        });
        let idx = if spinner.is_reversed() {
            frame_count.saturating_sub(1).saturating_sub(frame_position)
        } else {
            frame_position
        };
        frames[idx]
    }
}

fn normalize_interval(interval: core::time::Duration) -> core::time::Duration {
    let bounded_ms = interval.as_millis().max(1).min(u128::from(u64::MAX));
    let millis = match u64::try_from(bounded_ms) {
        Ok(value) => value,
        Err(_) => u64::MAX,
    };
    core::time::Duration::from_millis(millis)
}

fn nonzero_interval_ms(interval: core::time::Duration) -> u128 {
    interval.as_millis().max(1)
}

#[derive(Clone, Copy)]
struct FrameIndex(usize);

#[derive(Clone, Copy)]
struct FrameCount(usize);

#[derive(Clone, Copy)]
struct U128Division {
    value: u128,
    divisor: u128,
}

#[derive(Clone, Copy)]
struct UsizeDivision {
    value: usize,
    divisor: usize,
}

fn checked_quotient_u128(input: U128Division) -> u128 {
    match input.value.checked_div(input.divisor) {
        Some(result) => result,
        None => 0,
    }
}

fn checked_remainder_u128(input: U128Division) -> u128 {
    match input.value.checked_rem(input.divisor) {
        Some(result) => result,
        None => 0,
    }
}

fn checked_remainder_usize(input: UsizeDivision) -> usize {
    match input.value.checked_rem(input.divisor) {
        Some(result) => result,
        None => 0,
    }
}

fn bounded_step_offset(steps: u128, frame_count: FrameCount) -> usize {
    let frame_count_u128 = match u128::try_from(frame_count.0) {
        Ok(value) => value,
        Err(_) => u128::MAX,
    };
    let bounded_position = checked_remainder_u128(U128Division {
        value: steps,
        divisor: frame_count_u128,
    });

    match usize::try_from(bounded_position) {
        Ok(value) => value,
        Err(_) => 0,
    }
}

fn next_frame_index(frame: FrameIndex, frame_count: FrameCount) -> usize {
    checked_remainder_usize(UsizeDivision {
        value: frame.0.saturating_add(1),
        divisor: frame_count.0,
    })
}

fn advance_frame_index(frame: FrameIndex, steps: u128, frame_count: FrameCount) -> usize {
    let advanced = frame.0.saturating_add(bounded_step_offset(steps, frame_count));
    checked_remainder_usize(UsizeDivision {
        value: advanced,
        divisor: frame_count.0,
    })
}

#[cfg(test)]
mod tests;
