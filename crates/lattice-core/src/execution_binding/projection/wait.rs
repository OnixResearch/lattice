//! Closed-vocabulary wait reasons for profiles and instances.
//!
//! A reason names a code, a closed subject and the owner that can resolve it.
//! A subject is a fact field or a route by a name the same document already
//! publishes. Reasons carry no free text, so they cannot disclose a path,
//! endpoint, prompt, command, credential, grant reference or backend handle.
//! A reason is an explanation, never a reservation, claim or dispatch decision.

// r[impl lattice.worker_inventory.wait_reasons]
// r[impl lattice.worker_inventory.wait_reasons.scenario.causes]
// r[impl lattice.worker_inventory.wait_reasons.scenario.disclosure]

/// Reasons one profile or instance may carry.
pub const MAX_WAIT_REASONS: usize = 32;
/// The profile's own worker configuration declares the missing fact.
pub const CONFIGURATION_OWNER: &str = "configuration";
/// Qualification evidence that promotes declared support.
pub const QUALIFICATION_OWNER: &str = "animus#18";
/// The operator's revision lifecycle: desire, promotion, rollback and pins.
pub const REVISION_OWNER: &str = "lattice#25";
/// Global claims, attempts and dispatch.
pub const DISPATCH_OWNER: &str = "kiln#45";
/// One admitted attempt's recovery and cancellation observation.
pub const ATTEMPT_OWNER: &str = "lattice#6";

/// Codes in their published sort order.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    CapabilityUndeclared,
    QualificationUnbound,
    /// A lifecycle-managed definition admits no new session of this revision:
    /// another revision or none is selected, or it or its qualification is
    /// revoked.
    RevisionNotAdmitted,
    ServingPortAbsent,
    ServingSnapshotUnavailable,
    ServingPortIssue,
    ServingSnapshotNotCurrent,
    ServingNotReady,
    AwaitingDispatch,
    AdmissionExpired,
    DispatchOutcomeUnknown,
    CancellationUnobserved,
    LivenessUnobserved,
    LivenessStale,
    HostReachabilityUnknown,
    AllocationLivenessUnknown,
}

impl Code {
    /// The owner that can resolve the reason; fixed per code.
    #[must_use]
    pub const fn owner(self) -> &'static str {
        match self {
            Self::CapabilityUndeclared => CONFIGURATION_OWNER,
            Self::QualificationUnbound => QUALIFICATION_OWNER,
            Self::RevisionNotAdmitted => REVISION_OWNER,
            Self::ServingSnapshotUnavailable
            | Self::ServingPortIssue
            | Self::ServingSnapshotNotCurrent
            | Self::ServingNotReady => super::SERVING_OWNER,
            Self::AwaitingDispatch | Self::AdmissionExpired => DISPATCH_OWNER,
            Self::DispatchOutcomeUnknown | Self::CancellationUnobserved => ATTEMPT_OWNER,
            Self::ServingPortAbsent
            | Self::LivenessUnobserved
            | Self::LivenessStale
            | Self::HostReachabilityUnknown
            | Self::AllocationLivenessUnknown => super::LIVE_HOST_OWNER,
        }
    }
}

/// Published fact fields a reason may name.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum FactField {
    TaskFamilies,
    Tools,
    CapabilityQualifications,
    Revisions,
    ServingReadiness,
    Lifecycle,
    Cancellation,
    Liveness,
    HostReachability,
    AllocationLiveness,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Subject {
    Fact {
        field: FactField,
    },
    /// A route by the name the same document publishes in `model_routes`.
    Route {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reason {
    pub code: Code,
    pub subject: Subject,
    pub owner: String,
}

impl Reason {
    #[must_use]
    pub fn fact(code: Code, field: FactField) -> Self {
        Self {
            code,
            subject: Subject::Fact { field },
            owner: code.owner().to_owned(),
        }
    }

    #[must_use]
    pub fn route(code: Code, name: &str) -> Self {
        Self {
            code,
            subject: Subject::Route { name: name.to_owned() },
            owner: code.owner().to_owned(),
        }
    }
}

/// More reasons than [`MAX_WAIT_REASONS`]; never truncated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unbounded;

/// At most one reason per served route, or the missing port. A snapshot that
/// is not current never yields a readiness claim.
#[must_use]
pub fn serving(readiness: &super::ServingReadiness) -> Vec<Reason> {
    let routes = match readiness {
        super::ServingReadiness::NoServingPort {} => {
            return vec![Reason::fact(Code::ServingPortAbsent, FactField::ServingReadiness)];
        }
        super::ServingReadiness::NoServedRoute {} => return Vec::new(),
        super::ServingReadiness::Observed { routes } => routes,
    };
    let reasons = routes
        .iter()
        .filter_map(|route| route_code(&route.snapshot).map(|code| Reason::route(code, &route.route_name)))
        .collect::<Vec<_>>();
    debug_assert!(reasons.len() <= routes.len(), "at most one reason per route");
    debug_assert!(reasons.iter().all(|reason| matches!(reason.subject, Subject::Route { .. })));
    reasons
}

/// The producer's freshness label of a usable snapshot.
const CURRENT: &str = "current";
/// The producer's readiness label of a ready service.
const READY: &str = "ready";

fn route_code(snapshot: &super::SnapshotReading) -> Option<Code> {
    match snapshot {
        super::SnapshotReading::Unavailable {} => Some(Code::ServingSnapshotUnavailable),
        super::SnapshotReading::PortIssue {} => Some(Code::ServingPortIssue),
        super::SnapshotReading::Observed { freshness, .. } if freshness != CURRENT => {
            Some(Code::ServingSnapshotNotCurrent)
        }
        super::SnapshotReading::Observed { readiness, .. } if readiness != READY => Some(Code::ServingNotReady),
        super::SnapshotReading::Observed { .. } => None,
    }
}

/// Sort, deduplicate and bound reasons deterministically.
pub fn finish(mut reasons: Vec<Reason>) -> Result<Vec<Reason>, Unbounded> {
    reasons.sort();
    reasons.dedup();
    if reasons.len() > MAX_WAIT_REASONS {
        return Err(Unbounded);
    }
    debug_assert!(reasons.is_sorted_by(|left, right| left < right), "sorted and distinct");
    debug_assert!(reasons.iter().all(|reason| reason.owner == reason.code.owner()));
    Ok(reasons)
}
