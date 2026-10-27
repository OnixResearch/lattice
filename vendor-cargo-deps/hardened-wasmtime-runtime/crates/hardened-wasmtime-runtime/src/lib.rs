//! Pure deterministic core for execution-profile admission, normalized
//! outcomes, and strategy identity.
//!
//! The core receives explicit values and performs no file, process, network,
//! clock, or engine effect. Wasmtime execution lives in the test-side
//! strategy-parity rail; the product shell (restricted linking and
//! instantiation) arrives with the extraction package.
#![cfg_attr(not(test), deny(clippy::expect_used, clippy::unwrap_used))]

// r[impl hardened_wasmtime.profile_admission]
// r[impl hardened_wasmtime.outcome_model]

/// Artifact identity and shape admission.
pub mod artifact {
    /// Exact shape checks against declared materialization facts.
    pub mod shape;
}

/// Epoch-deadline budgets and service requirements.
pub mod epoch {
    /// Admission of epoch-deadline budgets.
    pub mod deadline;
}

/// Explicit Wasmtime feature-policy admission.
pub mod feature {
    /// Complete feature-policy admission.
    pub mod policy;
}

pub mod outcomes;

/// Molten profile document admission.
pub mod profile {
    /// Checked profile-document admission.
    pub mod document;
    /// Deserialized profile-document shape.
    pub mod types;
}

/// Cumulative runtime resource-limit admission.
pub mod resource {
    /// Named limit admission and stable resource outcome classes.
    pub mod admission;
}

/// Deny-by-default restricted import admission.
pub mod restricted {
    /// Exact import-descriptor admission.
    pub mod imports;
}

#[cfg(test)]
mod parity;

/// Minimum admissible fuel budget for any executed case.
pub const MINIMUM_FUEL: u64 = 1;
/// Minimum admissible linear-memory size in WebAssembly pages (one page is
/// 64 KiB); a guest that may not allocate memory cannot run.
pub const MINIMUM_MEMORY_PAGES: u32 = 1;

/// Independent compilation/execution strategy for one admitted profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[doc = "hardened_wasmtime_sealed"]
pub enum Strategy {
    /// Wasmtime Cranelift optimizing JIT backend.
    Cranelift,
    /// Wasmtime Winch baseline compiler backend.
    Winch,
}

impl Strategy {
    /// Stable lowercase implementation name used in observation admission.
    #[must_use]
    pub const fn implementation_name(self) -> &'static str {
        match self {
            Strategy::Cranelift => "wasmtime-cranelift",
            Strategy::Winch => "wasmtime-winch",
        }
    }
}

/// Explicit resource limits for one admitted execution profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    /// Metered fuel budget for the whole execution.
    pub fuel: u64,
    /// Maximum guest linear memory in 64 KiB pages.
    pub maximum_memory_pages: u32,
}

/// A limits draft whose bounds have not yet been checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LimitsDraft {
    /// Metered fuel budget candidate.
    pub fuel: u64,
    /// Maximum memory-pages candidate.
    pub maximum_memory_pages: u32,
}

/// Limits admission failure.
#[doc = "hardened_wasmtime_sealed"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitsAdmissionError {
    /// Fuel bound was below the minimum.
    ZeroFuel,
    /// Memory-page bound was below the minimum.
    ZeroMemoryPages,
}

/// Admits explicit resource limits or names the violated field.
pub fn admit_limits(draft: LimitsDraft) -> Result<ResourceLimits, LimitsAdmissionError> {
    if draft.fuel < MINIMUM_FUEL {
        return Err(LimitsAdmissionError::ZeroFuel);
    }
    if draft.maximum_memory_pages < MINIMUM_MEMORY_PAGES {
        return Err(LimitsAdmissionError::ZeroMemoryPages);
    }
    Ok(ResourceLimits {
        fuel: draft.fuel,
        maximum_memory_pages: draft.maximum_memory_pages,
    })
}

/// Stable trap classes produced by outcome normalization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[doc = "hardened_wasmtime_sealed"]
pub enum TrapClass {
    /// The guest executed `unreachable`.
    Unreachable,
    /// Any other trap; carries no engine-specific detail.
    Other,
}

/// One normalized execution outcome, independent of strategy.
#[derive(Clone, Debug, PartialEq, Eq)]
#[doc = "hardened_wasmtime_sealed"]
pub enum NormalizedOutcome {
    /// Execution finished; carries the stable class of the completed run.
    Completed,
    /// Execution trapped; carries the stable trap class.
    Trapped(TrapClass),
    /// Execution ran out of metered fuel.
    FuelExhausted,
}

impl NormalizedOutcome {
    /// Stable lowercase class name used in observation admission.
    #[must_use]
    pub fn class_name(&self) -> &str {
        match self {
            NormalizedOutcome::Completed => "completed",
            NormalizedOutcome::Trapped(TrapClass::Unreachable) => "trap-unreachable",
            NormalizedOutcome::Trapped(TrapClass::Other) => "trap-other",
            NormalizedOutcome::FuelExhausted => "fuel-exhausted",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FUEL: u64 = 1_024;
    const SAMPLE_PAGES: u32 = 4;

    #[test]
    // r[verify hardened_wasmtime.profile_admission]
    fn positive_limits_round_trip_exactly() {
        let admitted = admit_limits(LimitsDraft {
            fuel: SAMPLE_FUEL,
            maximum_memory_pages: SAMPLE_PAGES,
        })
        .expect("positive limits must be admitted");

        assert_eq!(
            admitted,
            ResourceLimits {
                fuel: SAMPLE_FUEL,
                maximum_memory_pages: SAMPLE_PAGES,
            }
        );
    }

    #[test]
    // r[verify hardened_wasmtime.profile_admission]
    fn zero_bounds_fail_closed_naming_the_field() {
        let zero_fuel = admit_limits(LimitsDraft {
            fuel: 0,
            maximum_memory_pages: SAMPLE_PAGES,
        })
        .expect_err("zero fuel must fail");
        assert_eq!(zero_fuel, LimitsAdmissionError::ZeroFuel);

        let zero_pages = admit_limits(LimitsDraft {
            fuel: SAMPLE_FUEL,
            maximum_memory_pages: 0,
        })
        .expect_err("zero pages must fail");
        assert_eq!(zero_pages, LimitsAdmissionError::ZeroMemoryPages);
    }

    #[test]
    // r[verify hardened_wasmtime.outcome_model]
    fn outcome_classes_are_distinct_and_stable() {
        let classes = [
            NormalizedOutcome::Completed,
            NormalizedOutcome::Trapped(TrapClass::Unreachable),
            NormalizedOutcome::Trapped(TrapClass::Other),
            NormalizedOutcome::FuelExhausted,
        ];
        for (index, outcome) in classes.iter().enumerate() {
            let next = index.saturating_add(1);
            for other in classes.iter().skip(next) {
                assert_ne!(outcome.class_name(), other.class_name());
            }
        }
    }

    #[test]
    fn strategy_names_are_stable() {
        assert_eq!(
            Strategy::Cranelift.implementation_name(),
            "wasmtime-cranelift"
        );
        assert_eq!(Strategy::Winch.implementation_name(), "wasmtime-winch");
    }
}
