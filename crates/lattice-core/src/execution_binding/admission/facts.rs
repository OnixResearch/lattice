//! Classification of time-, device-, freshness- and health-dependent facts.
//!
//! Missing provider logical time, absent offer facts, a stale readback and a
//! non-healthy host stay unknown or unavailable. None of them is read as zero,
//! unexpired, idle or healthy.

// r[impl lattice.execution_binding.allocation_observation.scenario.unknown]

/// Every time- and evidence-dependent fact of one validated observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Classification {
    pub expiry: Expiry,
    pub device_kind: DeviceKind,
    pub freshness: Freshness,
    pub health: HostHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Expiry {
    Unexpired,
    Expired,
    /// The port supplied no producer logical time.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum DeviceKind {
    /// Claimed `chips` quantity from offer pool facts; zero means CPU-only.
    Known { chips: u64 },
    /// No offer pool facts at the allocation's revision.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Freshness {
    Fresh,
    /// The readback is stale; current state is unknown.
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum HostHealth {
    Healthy,
    /// Degraded, unavailable or unknown host health.
    Unavailable {
        reported: super::super::model::Health,
    },
}

/// Classify a validated observation without consulting any clock.
#[must_use]
pub fn classify(observation: &super::super::model::ComputeAllocationObservation) -> Classification {
    Classification {
        expiry: expiry(observation),
        device_kind: observation
            .offer
            .as_ref()
            .and_then(|offer| claimed_chips(&observation.allocation, offer))
            .map_or(DeviceKind::Unknown, |chips| DeviceKind::Known { chips }),
        freshness: if observation.readback.is_fresh {
            Freshness::Fresh
        } else {
            Freshness::Unknown
        },
        health: match observation.readback.health {
            super::super::model::Health::Healthy => HostHealth::Healthy,
            reported @ (super::super::model::Health::Degraded
            | super::super::model::Health::Unavailable
            | super::super::model::Health::Unknown) => HostHealth::Unavailable { reported },
        },
    }
}

fn expiry(observation: &super::super::model::ComputeAllocationObservation) -> Expiry {
    match observation.port.provider_logical_now {
        None => Expiry::Unknown,
        Some(now) if now < observation.allocation.valid_until => Expiry::Unexpired,
        Some(_) => Expiry::Expired,
    }
}

/// Sum of claimed quantities on `chips` pools; `None` when a claim has no
/// pool fact or the sum overflows, which keeps the device kind unknown.
fn claimed_chips(
    allocation: &super::super::model::AllocationRecord,
    offer: &super::super::model::OfferPoolFacts,
) -> Option<u64> {
    allocation.claims.iter().try_fold(0_u64, |total, claim| {
        let pool = offer.claimed_pools.iter().find(|pool| pool.pool_id == claim.pool_id)?;
        if pool.kind == super::super::model::PoolKind::Chips {
            total.checked_add(claim.quantity)
        } else {
            Some(total)
        }
    })
}
