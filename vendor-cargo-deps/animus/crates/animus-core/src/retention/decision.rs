//! Deterministic bounded-retention policy.

const PERCENT_MAX: u8 = 100;

/// Explicit bounds and promotion policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    capacity: u32,
    low_watermark: u32,
    high_watermark: u32,
    promotion_threshold_percent: u8,
}

impl Policy {
    /// Validates an explicit policy.
    ///
    /// An explicit zero capacity is valid and produces pass-through behavior.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError`] for invalid watermarks or percentage.
    pub const fn new(
        capacity: u32,
        low_watermark: u32,
        high_watermark: u32,
        promotion_threshold_percent: u8,
    ) -> Result<Self, DecisionError> {
        if low_watermark > high_watermark {
            return Err(DecisionError::ReversedWatermarks);
        }
        if high_watermark > capacity {
            return Err(DecisionError::WatermarkExceedsCapacity);
        }
        if promotion_threshold_percent > PERCENT_MAX {
            return Err(DecisionError::InvalidPercentage);
        }
        Ok(Self {
            capacity,
            low_watermark,
            high_watermark,
            promotion_threshold_percent,
        })
    }

    /// Returns the exact entry capacity.
    #[must_use]
    pub const fn capacity(self) -> u32 {
        self.capacity
    }

    /// Returns the low watermark.
    #[must_use]
    pub const fn low_watermark(self) -> u32 {
        self.low_watermark
    }

    /// Returns the high watermark.
    #[must_use]
    pub const fn high_watermark(self) -> u32 {
        self.high_watermark
    }

    /// Returns the lazy-promotion threshold percentage.
    #[must_use]
    pub const fn promotion_threshold_percent(self) -> u8 {
        self.promotion_threshold_percent
    }
}

/// Supplied retention facts for one occupied slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entry {
    /// Store-owned slot index.
    pub slot: u32,
    /// Supplied generation of the latest promotion.
    pub promotion_generation: u64,
}

/// Supplied age used for a deterministic promotion decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PromotionFacts {
    /// Relative age from 0 through 100 percent.
    pub age_percent: u8,
}

/// One deterministic insertion disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InsertDisposition {
    /// Explicit zero capacity retains no entry.
    DoNotRetain,
    /// Use the named free slot.
    UseFreeSlot(u32),
    /// Replace the named least-recently-promoted slot.
    EvictSlot(u32),
}

/// Entries that the shell must release after leaving its guard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleasePlan {
    slots: alloc::vec::Vec<u32>,
}

impl ReleasePlan {
    /// Returns the slots whose prior values must be released.
    #[must_use]
    pub fn slots(&self) -> &[u32] {
        &self.slots
    }
}

/// Deterministic insertion and deferred-release decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsertPlan {
    /// Slot selection or explicit no-retention disposition.
    pub disposition: InsertDisposition,
    /// Values to release after the shell drops its guard.
    pub release: ReleasePlan,
}

/// Deterministic watermark trim decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TrimPlan {
    /// Number of oldest entries to release.
    pub release_count: u32,
}

/// Invalid supplied facts for retention decisions.
#[cfg_attr(
    animus_octet,
    allow(
        tigerstyle::path_segment_repetition,
        reason = "the error name distinguishes policy rejection from shell execution failure"
    )
)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionError {
    /// Tool identity was blank.
    EmptyToolIdentity,
    /// A present capability context was blank.
    EmptyCapabilityContext,
    /// A canonical field length did not fit the fixed framing type.
    ProjectionTooLarge,
    /// Low watermark exceeded high watermark.
    ReversedWatermarks,
    /// A watermark exceeded the explicit capacity.
    WatermarkExceedsCapacity,
    /// A percentage exceeded 100.
    InvalidPercentage,
    /// Active count exceeded the explicit capacity.
    ActiveCountExceedsCapacity,
    /// A free slot was required but none was supplied.
    MissingFreeSlot,
    /// An eviction candidate was required but none was supplied.
    MissingEvictionCandidate,
    /// A supplied slot was outside the explicit capacity.
    SlotOutsideCapacity,
}

impl core::fmt::Display for DecisionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::EmptyToolIdentity => "tool identity must not be blank",
            Self::EmptyCapabilityContext => "present capability context must not be blank",
            Self::ProjectionTooLarge => "cache key projection field is too large",
            Self::ReversedWatermarks => "low watermark must not exceed high watermark",
            Self::WatermarkExceedsCapacity => "cache watermark must not exceed capacity",
            Self::InvalidPercentage => "cache percentage must be at most 100",
            Self::ActiveCountExceedsCapacity => "active cache count exceeds capacity",
            Self::MissingFreeSlot => "cache insertion requires a supplied free slot",
            Self::MissingEvictionCandidate => "cache insertion requires an eviction candidate",
            Self::SlotOutsideCapacity => "cache slot is outside the explicit capacity",
        };
        formatter.write_str(message)
    }
}

/// Decides whether one hit promotes to most-recently-used position.
///
/// Threshold 0 is strict LRU and always promotes. Threshold 100 is FIFO and
/// never promotes. Intermediate thresholds promote at or above the threshold.
///
/// # Errors
///
/// Returns [`DecisionError::InvalidPercentage`] for an invalid supplied age.
pub const fn decide_promotion(
    policy: Policy,
    facts: PromotionFacts,
) -> Result<bool, DecisionError> {
    if facts.age_percent > PERCENT_MAX {
        return Err(DecisionError::InvalidPercentage);
    }
    let threshold = policy.promotion_threshold_percent();
    if threshold == 0 {
        return Ok(true);
    }
    if threshold == PERCENT_MAX {
        return Ok(false);
    }
    Ok(facts.age_percent >= threshold)
}

/// Plans one insertion from supplied count, free-list, and generation facts.
///
/// # Errors
///
/// Returns [`DecisionError`] when supplied state contradicts capacity or omits
/// a required slot.
pub fn plan_insert(
    policy: Policy,
    active_count: u32,
    free_slots: &[u32],
    entries: &[Entry],
) -> Result<InsertPlan, DecisionError> {
    validate_state(policy, active_count, free_slots, entries)?;
    debug_assert!(active_count <= policy.capacity());
    debug_assert!(
        free_slots.iter().all(|slot| *slot < policy.capacity())
            && entries.iter().all(|entry| entry.slot < policy.capacity())
    );
    if policy.capacity() == 0 {
        return Ok(InsertPlan {
            disposition: InsertDisposition::DoNotRetain,
            release: ReleasePlan {
                slots: alloc::vec::Vec::new(),
            },
        });
    }
    if active_count < policy.capacity() {
        return plan_free_slot(free_slots);
    }
    let slot = select_oldest(entries).ok_or(DecisionError::MissingEvictionCandidate)?;
    Ok(InsertPlan {
        disposition: InsertDisposition::EvictSlot(slot),
        release: ReleasePlan {
            slots: alloc::vec![slot],
        },
    })
}

/// Plans a high-to-low watermark trim from an explicit active count.
///
/// # Errors
///
/// Returns [`DecisionError::ActiveCountExceedsCapacity`] for an invalid count.
pub const fn plan_trim(policy: Policy, active_count: u32) -> Result<TrimPlan, DecisionError> {
    if active_count > policy.capacity() {
        return Err(DecisionError::ActiveCountExceedsCapacity);
    }
    let release_count = if active_count > policy.high_watermark() {
        active_count.saturating_sub(policy.low_watermark())
    } else {
        0
    };
    Ok(TrimPlan { release_count })
}

/// Selects the oldest supplied entry. Ties use the smallest slot.
#[must_use]
pub fn select_oldest(entries: &[Entry]) -> Option<u32> {
    entries
        .iter()
        .min_by_key(|entry| (entry.promotion_generation, entry.slot))
        .map(|entry| entry.slot)
}

fn plan_free_slot(free_slots: &[u32]) -> Result<InsertPlan, DecisionError> {
    let slot = free_slots
        .iter()
        .copied()
        .min()
        .ok_or(DecisionError::MissingFreeSlot)?;
    Ok(InsertPlan {
        disposition: InsertDisposition::UseFreeSlot(slot),
        release: ReleasePlan {
            slots: alloc::vec::Vec::new(),
        },
    })
}

fn validate_state(
    policy: Policy,
    active_count: u32,
    free_slots: &[u32],
    entries: &[Entry],
) -> Result<(), DecisionError> {
    if active_count > policy.capacity() {
        return Err(DecisionError::ActiveCountExceedsCapacity);
    }
    if free_slots.iter().any(|slot| *slot >= policy.capacity())
        || entries.iter().any(|entry| entry.slot >= policy.capacity())
    {
        return Err(DecisionError::SlotOutsideCapacity);
    }
    Ok(())
}
