//! Read-only profile, instance and usage projections of admitted bindings.
//!
//! Profile facts describe what a worker profile declares; instance
//! observations describe one retained attempt; usage provenance freezes the
//! binding a usage observation belongs to. None of them grants execution,
//! credential or compute authority, and none reports idle, running, healthy or
//! available.

pub mod accounting;
pub mod inventory;
pub mod lineage;
pub mod retained;
pub mod wait;

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
/// Owner of model-request usage that no served route observed; serving
/// readiness is reported through [`ServingReadiness`] instead.
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

/// Serving readiness of the served routes, with its actual cause when nothing
/// was observed. It is never inferred from compute or from a profile.
// r[impl lattice.serving_binding.readiness]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServingReadiness {
    /// No serving-owner port was injected, so no snapshot was observed.
    NoServingPort {},
    /// The binding declares no served route.
    NoServedRoute {},
    /// The injected port's latest snapshot for each served route.
    Observed { routes: Vec<RouteReadiness> },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteReadiness {
    pub route_name: String,
    pub service_id: String,
    pub snapshot: SnapshotReading,
}

/// One port read. Readiness and freshness are producer labels; a service the
/// snapshot does not list reads `not-listed`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "reading", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotReading {
    Observed {
        snapshot_identity: String,
        source_generation: u64,
        sequence: u64,
        readiness: String,
        freshness: String,
    },
    Unavailable {},
    PortIssue {},
}

impl ServingReadiness {
    /// At most one reading per declared route, each with token identities.
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        let Self::Observed { routes } = self else {
            return true;
        };
        let is_named =
            routes.iter().all(|route| super::is_token(&route.route_name) && super::is_token(&route.service_id));
        routes.len() <= super::MAX_ROUTES && is_named
    }
}
