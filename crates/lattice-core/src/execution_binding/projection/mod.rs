//! Read-only profile, instance and usage projections of admitted bindings.
//!
//! Profile facts describe what a worker profile declares; instance
//! observations describe one retained attempt; usage provenance freezes the
//! binding a usage observation belongs to. None of them grants execution,
//! credential or compute authority, and none reports idle, running, healthy or
//! available.

pub mod accounting;
pub mod inventory;
pub mod retained;

/// Exactly one support level per fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportLevel {
    /// Stated by configuration or request; not qualified.
    Declared,
    /// Declared and bound to qualification evidence.
    Qualified,
    Unavailable,
    Unknown,
}

/// One fact. `value` is present only for declared or qualified support.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fact<T> {
    pub support: SupportLevel,
    pub value: Option<T>,
}

/// What a profile owner states about one fact before projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered<T> {
    Declared(T),
    Unavailable,
    Unknown,
}

impl<T> Offered<T> {
    /// Promote declared support to qualified only when evidence is bound.
    pub fn project<U>(self, is_qualified: bool, convert: impl FnOnce(T) -> U) -> Fact<U> {
        match self {
            Self::Declared(value) => Fact {
                support: if is_qualified {
                    SupportLevel::Qualified
                } else {
                    SupportLevel::Declared
                },
                value: Some(convert(value)),
            },
            Self::Unavailable => Fact {
                support: SupportLevel::Unavailable,
                value: None,
            },
            Self::Unknown => Fact {
                support: SupportLevel::Unknown,
                value: None,
            },
        }
    }
}

/// A capability whose owner has not published a consumer-ready contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedUnknown {
    pub support: SupportLevel,
    pub owner: String,
}

// r[impl lattice.execution_binding.deferred]
// r[impl lattice.execution_binding.deferred.scenario.serving]
/// Serving readiness, request admission and per-request usage.
pub const SERVING_OWNER: &str = "animus#17";
/// Live host, device and cross-host qualification.
pub const LIVE_HOST_OWNER: &str = "kiln#49";
/// Device utilization has no producer readback field.
pub const DEVICE_UTILIZATION_OWNER: &str = "aspen#42";

impl OwnedUnknown {
    #[must_use]
    pub fn unknown(owner: &str) -> Self {
        Self {
            support: SupportLevel::Unknown,
            owner: owner.to_owned(),
        }
    }
}
