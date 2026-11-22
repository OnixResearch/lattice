//! Pure decisions of the host model gateway over the C3/C4 records in
//! `core`: upstream validation, local prechecks, admission acceptance, the
//! settlement outcome and the guest reply. No I/O.

const MAX_REQUESTS: u32 = 256;
const MAX_PLACEMENT_HOST_BYTES: usize = 256;

pub fn validate_upstream(upstream: &super::core::Upstream) -> Result<(), String> {
    if upstream.schema != super::core::UPSTREAM_SCHEMA || upstream.max_requests == 0 {
        return Err("model gateway upstream schema or request ceiling is unsupported".into());
    }
    if upstream.max_requests > MAX_REQUESTS {
        return Err("model gateway upstream schema or request ceiling is unsupported".into());
    }
    crate::candidate_worker::broker::admission::validate_model(&upstream.model)?;
    crate::candidate_worker::broker::admission::validate_endpoint(&upstream.endpoint)?;
    let reference = upstream.credential.as_ref();
    if reference.is_some_and(|value| value.provider != "env" || value.field.is_some()) {
        return Err("model gateway requires an explicit env credential reference".into());
    }
    Ok(())
}

/// What the gateway knows of its one binding, fixed at start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    /// `b3:` identity of the C2 binding.
    pub binding: String,
    /// The binding's original admitted deadline; never extended.
    pub deadline_unix_ms: u64,
    pub model: String,
    pub max_requests: u32,
}

/// The gateway's own view of one binding, across requests of one incarnation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub bound: Bound,
    /// Requests this incarnation admitted through the authority.
    pub forwarded_requests: u32,
    /// The highest request index this incarnation sent to the authority, if any.
    pub last_index: Option<u32>,
}

impl Session {
    #[must_use]
    pub const fn new(bound: Bound) -> Self {
        Self {
            bound,
            forwarded_requests: 0,
            last_index: None,
        }
    }
}

/// The placement and budget an authority admitted one request under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grant {
    pub placement_host: String,
    pub placement_generation: u64,
    pub consumed_requests: u32,
    pub remaining_requests: u32,
}

/// Local checks before any authority call: binding, host deadline, index
/// order and the local ceiling. The body is checked by [`canonical_body`].
// r[impl lattice.vm_profile.gateway.scenario.deadline]
pub fn precheck(
    session: &Session,
    request: &super::core::ForwardedRequest,
    now_ms: u64,
) -> Result<String, super::core::ForwardedRefusal> {
    let bound = &session.bound;
    if request.binding != bound.binding {
        return Err(super::core::ForwardedRefusal::UnknownBinding);
    }
    if now_ms >= bound.deadline_unix_ms {
        return Err(super::core::ForwardedRefusal::Deadline);
    }
    // A replayed or reordered index is never sent upstream again.
    if session.last_index.is_some_and(|last| request.request_index <= last) {
        return Err(super::core::ForwardedRefusal::NotCurrent);
    }
    if session.forwarded_requests >= bound.max_requests {
        return Err(super::core::ForwardedRefusal::Budget);
    }
    debug_assert!(now_ms < bound.deadline_unix_ms);
    debug_assert!(session.forwarded_requests < bound.max_requests);
    canonical_body(&request.body, bound)
}

/// The re-encoded body of a bounded JSON request for exactly the bound
/// model, so duplicate keys cannot be read differently upstream.
fn canonical_body(body: &str, bound: &Bound) -> Result<String, super::core::ForwardedRefusal> {
    if body.len() > crate::http::MAX_REQUEST_BODY_BYTES {
        return Err(super::core::ForwardedRefusal::Malformed);
    }
    let value: serde_json::Value = serde_json::from_str(body).map_err(|_| super::core::ForwardedRefusal::Malformed)?;
    if value.get("model").and_then(serde_json::Value::as_str) != Some(bound.model.as_str()) {
        return Err(super::core::ForwardedRefusal::Malformed);
    }
    serde_json::to_string(&value).map_err(|_| super::core::ForwardedRefusal::Malformed)
}

/// Check an authority's reply to `Admit` against the binding: the deadline is
/// never moved, and the carried budget stays within the local ceiling.
// r[impl lattice.vm_profile.gateway.scenario.budget_continuity]
pub fn accept_grant(bound: &Bound, reply: super::core::AuthorityReply) -> Result<Grant, super::core::ForwardedRefusal> {
    let admission = match reply {
        super::core::AuthorityReply::Admitted {
            placement_host,
            placement_generation,
            consumed_requests,
            remaining_requests,
            deadline_unix_ms,
        } if deadline_unix_ms == bound.deadline_unix_ms => Grant {
            placement_host,
            placement_generation,
            consumed_requests,
            remaining_requests,
        },
        super::core::AuthorityReply::Admitted { .. } => return Err(super::core::ForwardedRefusal::Deadline),
        super::core::AuthorityReply::Refused { code } => return Err(code.into()),
        super::core::AuthorityReply::Recorded { .. } => return Err(super::core::ForwardedRefusal::NotCurrent),
    };
    let host = admission.placement_host.as_str();
    if host.is_empty() || host.len() > MAX_PLACEMENT_HOST_BYTES || host.chars().any(char::is_control) {
        return Err(super::core::ForwardedRefusal::NotCurrent);
    }
    let total_requests = admission
        .consumed_requests
        .checked_add(admission.remaining_requests)
        .ok_or(super::core::ForwardedRefusal::Budget)?;
    if admission.consumed_requests == 0 || total_requests > bound.max_requests {
        return Err(super::core::ForwardedRefusal::Budget);
    }
    debug_assert!(admission.consumed_requests <= bound.max_requests);
    debug_assert!(!admission.placement_host.is_empty());
    Ok(admission)
}

/// What the upstream call produced, as the gateway observed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observed {
    /// A complete response within bounds that did not echo the credential.
    Response {
        status: u16,
    },
    /// A response withheld because it echoed the host credential or exceeded a bound.
    Withheld,
    /// Refused locally before any byte left the gateway.
    NotSent,
    TimedOut,
    Cancelled,
    /// The transport failed after the request may have been sent.
    TransportFailed,
}

/// The C3 settlement outcome of one observed call.
#[must_use]
pub const fn settle_outcome(observed: Observed) -> super::core::SettleOutcome {
    match observed {
        Observed::Response { status } if status >= 200 && status < 300 => super::core::SettleOutcome::Completed,
        Observed::Response { .. } | Observed::Withheld | Observed::NotSent => super::core::SettleOutcome::Failed,
        Observed::TimedOut => super::core::SettleOutcome::TimedOut,
        Observed::Cancelled => super::core::SettleOutcome::Cancelled,
        Observed::TransportFailed => super::core::SettleOutcome::Uncertain,
    }
}

/// The C4 reply after settlement. Only a recorded settlement of a delivered
/// response returns it; a lost or refused settlement leaves the request
/// uncertain and the response is withheld.
// r[impl lattice.vm_profile.gateway.scenario.lost_settle]
#[must_use]
pub fn reply(
    request_index: u32,
    observed: Observed,
    body: String,
    is_recorded: bool,
) -> super::core::ForwardedResponse {
    match observed {
        Observed::Response { status } if is_recorded => {
            super::core::ForwardedResponse::Responded(super::core::Responded {
                request_index,
                status,
                body,
            })
        }
        Observed::NotSent if is_recorded => super::core::ForwardedResponse::Refused(super::core::Refused {
            request_index,
            refused: super::core::ForwardedRefusal::Malformed,
        }),
        Observed::Response { .. }
        | Observed::NotSent
        | Observed::Withheld
        | Observed::TimedOut
        | Observed::Cancelled
        | Observed::TransportFailed => super::core::ForwardedResponse::Uncertain(super::core::Uncertain {
            request_index,
            uncertain: true,
        }),
    }
}

/// Record a decided index; later frames must use a higher one.
pub fn decided(session: &mut Session, request_index: u32) {
    debug_assert!(session.last_index.is_none_or(|last| request_index > last));
    session.last_index = Some(request_index);
}
