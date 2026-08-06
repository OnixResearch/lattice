const LEGACY_RATIO_SCALE: u32 = 1_000_000;

/// Canonical non-collapsing split and resize ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitRatio {
    numerator: u32,
    denominator: u32,
}

impl SplitRatio {
    /// Creates an exact validated split ratio.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError::InvalidRatio`] when the denominator is
    /// zero or the numerator would collapse either side.
    pub fn new(numerator: impl Into<u32>, denominator: impl Into<u32>) -> Result<Self, super::DockingError> {
        let numerator = numerator.into();
        let denominator = denominator.into();
        if denominator == 0 || numerator == 0 || numerator >= denominator {
            return Err(super::DockingError::InvalidRatio);
        }
        let divisor = greatest_common_divisor([numerator, denominator]);
        debug_assert!(divisor > 0);
        debug_assert!(numerator < denominator);
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }

    /// Converts a supported finite legacy `f64` ratio explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError::InvalidRatio`] for NaN, infinity, edge,
    /// negative, overfull, or unrepresentable values.
    pub fn try_from_legacy_f64(value: f64) -> Result<Self, super::DockingError> {
        if !value.is_finite() || value <= 0.0 || value >= 1.0 {
            return Err(super::DockingError::InvalidRatio);
        }
        let scaled = (value * f64::from(LEGACY_RATIO_SCALE)).round();
        let Ok(numerator) = format!("{scaled:.0}").parse::<u32>() else {
            return Err(super::DockingError::InvalidRatio);
        };
        Self::new(numerator, LEGACY_RATIO_SCALE)
    }

    /// Converts a supported finite legacy `f32` ratio explicitly.
    ///
    /// # Errors
    ///
    /// Returns [`super::DockingError::InvalidRatio`] for NaN, infinity, edge,
    /// negative, overfull, or unrepresentable values.
    pub fn try_from_legacy_f32(value: f32) -> Result<Self, super::DockingError> {
        Self::try_from_legacy_f64(f64::from(value))
    }

    /// Returns the complementary non-collapsing ratio.
    #[must_use]
    pub const fn complement(self) -> Self {
        Self {
            numerator: self.denominator.saturating_sub(self.numerator),
            denominator: self.denominator,
        }
    }

    /// Returns the reduced exact numerator.
    #[must_use]
    pub const fn numerator(self) -> u32 {
        self.numerator
    }

    /// Returns the reduced exact denominator.
    #[must_use]
    pub const fn denominator(self) -> u32 {
        self.denominator
    }

    /// Projects the ratio as a renderer-owned legacy `f32` value.
    #[must_use]
    pub fn as_legacy_f32(self) -> f32 {
        let numerator = self.numerator.to_string().parse::<f32>().unwrap_or(0.0);
        let denominator = self.denominator.to_string().parse::<f32>().unwrap_or(1.0);
        debug_assert!(denominator > 0.0);
        numerator / denominator
    }

    pub(super) fn first_len(self, total: u16) -> u16 {
        let numerator = u64::from(self.numerator);
        let denominator = u64::from(self.denominator);
        let total_len = u64::from(total);
        let scaled_len = total_len.saturating_mul(numerator);
        let Some(first_len) = scaled_len.checked_div(denominator) else {
            return 0;
        };
        let capped_len = first_len.min(total_len);
        u16::try_from(capped_len).unwrap_or(total)
    }
}

fn greatest_common_divisor(values: [u32; 2]) -> u32 {
    let [mut left, mut right] = values;
    while right != 0 {
        let remainder = left.checked_rem(right).unwrap_or(0);
        left = right;
        right = remainder;
    }
    left.max(1)
}
