//! `KilnAuthorityGate`: the host model gateway's `ModelRequestGate` over
//! Kiln's per-host C3 Unix socket. One request per connection; `Admit` before
//! any upstream contact, `Settle` after it. A lost `Admit` reply refuses the
//! request (the authority may count it consumed; nothing reaches the
//! upstream). A lost `Settle` leaves the request `uncertain` on both sides
//! and is never retried.

// r[impl lattice.vm_profile.gateway]
// r[impl lattice.vm_profile.gateway.scenario.authority]

use std::io::Write;

const AUTHORITY_DEADLINE: std::time::Duration = std::time::Duration::from_secs(5);

/// One exchange with the authority socket.
pub trait AuthorityPort: Send {
    fn exchange(&mut self, request: &super::core::AuthorityRequest) -> Result<super::core::AuthorityReply, String>;
}

/// Kiln's per-host authority socket (mode 0600).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnixSocket {
    pub socket: std::path::PathBuf,
}

impl AuthorityPort for UnixSocket {
    #[cfg(unix)]
    fn exchange(&mut self, request: &super::core::AuthorityRequest) -> Result<super::core::AuthorityReply, String> {
        let mut stream = std::os::unix::net::UnixStream::connect(&self.socket)
            .map_err(|_| "effect authority socket is unavailable")?;
        stream
            .set_read_timeout(Some(AUTHORITY_DEADLINE))
            .map_err(|_| "authority read deadline is unavailable")?;
        stream
            .set_write_timeout(Some(AUTHORITY_DEADLINE))
            .map_err(|_| "authority write deadline is unavailable")?;
        let frame = lattice_workflow_transport::encode_frame(request, super::core::MAX_AUTHORITY_FRAME_BYTES)
            .map_err(|_| "authority request cannot be framed")?;
        lattice_workflow_transport::write_frame(&mut stream, &frame)
            .map_err(|_| "authority request was not written")?;
        stream.flush().map_err(|_| "authority request was not written")?;
        let reply = lattice_workflow_transport::read_frame(&mut stream, super::core::MAX_AUTHORITY_FRAME_BYTES)
            .map_err(|_| "authority reply was lost")?;
        serde_json::from_slice(&reply).map_err(|_| "authority reply is malformed".into())
    }

    #[cfg(not(unix))]
    fn exchange(&mut self, _request: &super::core::AuthorityRequest) -> Result<super::core::AuthorityReply, String> {
        Err("effect authority requires Unix sockets".into())
    }
}

/// The gateway's per-request gate. `prepare` names the request index of the
/// next `admit`; the refusal of a denied admission is kept for the reply.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "the relocation contract names this gate `KilnAuthorityGate`"
)]
pub struct KilnAuthorityGate<P> {
    pub port: P,
    pub bound: super::admission::Bound,
    pending: Option<u32>,
    admitted: Option<(u32, super::admission::Grant)>,
    refusal: Option<super::core::ForwardedRefusal>,
}

impl<P: AuthorityPort> KilnAuthorityGate<P> {
    #[must_use]
    pub const fn new(port: P, bound: super::admission::Bound) -> Self {
        Self {
            port,
            bound,
            pending: None,
            admitted: None,
            refusal: None,
        }
    }

    /// Name the request index the next `admit` asks the authority for.
    pub fn prepare(&mut self, request_index: u32) {
        self.pending = Some(request_index);
        self.refusal = None;
    }

    /// Why the last `admit` returned `None`.
    #[must_use]
    pub const fn refusal(&self) -> Option<super::core::ForwardedRefusal> {
        self.refusal
    }

    /// The placement and budget of the request admitted last, until settled.
    #[must_use]
    pub fn admission(&self) -> Option<&super::admission::Grant> {
        self.admitted.as_ref().map(|(_, admission)| admission)
    }

    fn admit_pending(&mut self) -> Result<u32, super::core::ForwardedRefusal> {
        let request_index = self.pending.take().ok_or(super::core::ForwardedRefusal::NotCurrent)?;
        let request = super::core::AuthorityRequest::Admit {
            binding: self.bound.binding.clone(),
            request_index,
            model: self.bound.model.clone(),
            route: super::core::ROUTE.to_owned(),
        };
        let reply = self.port.exchange(&request).map_err(|_| super::core::ForwardedRefusal::NotCurrent)?;
        let admission = super::admission::accept_grant(&self.bound, reply)?;
        self.admitted = Some((request_index, admission));
        Ok(request_index)
    }

    /// Settle the admitted request with its exact outcome. `Ok` carries the
    /// authority's recorded consumption; any other reply or a lost reply is
    /// an error and the request stays `uncertain`.
    pub fn settle_outcome(
        &mut self,
        _ticket: crate::candidate_worker::broker::GateTicket,
        outcome: super::core::SettleOutcome,
        usage: Option<super::core::Usage>,
    ) -> Result<u32, String> {
        let (request_index, _) = self.admitted.take().ok_or("model gateway settled without an admission")?;
        let request = super::core::AuthorityRequest::Settle {
            binding: self.bound.binding.clone(),
            request_index,
            outcome,
            usage,
        };
        match self.port.exchange(&request)? {
            super::core::AuthorityReply::Recorded { consumed_requests } => Ok(consumed_requests),
            super::core::AuthorityReply::Refused { code } => Err(format!("authority refused settlement: {code:?}")),
            super::core::AuthorityReply::Admitted { .. } => {
                Err("authority answered settlement with an admission".into())
            }
        }
    }
}

/// Provider-reported usage of a response body, when it reports any.
#[must_use]
pub fn usage(facts: &crate::execution_binding::serving::gate::ResponseFacts) -> Option<super::core::Usage> {
    let has_usage = facts.input_tokens.is_some() || facts.output_tokens.is_some();
    has_usage.then_some(super::core::Usage {
        input_tokens: facts.input_tokens,
        output_tokens: facts.output_tokens,
    })
}

impl<P: AuthorityPort> crate::candidate_worker::broker::ModelRequestGate for KilnAuthorityGate<P> {
    /// The broker's own checks first; then C3 `Admit` for the prepared index,
    /// which `prepare` received with the forwarded frame.
    fn admit(
        &mut self,
        checks: crate::execution_binding::serving::gate::Checks,
        _: u32,
    ) -> Option<crate::candidate_worker::broker::GateTicket> {
        let local = if !checks.is_current {
            Some(super::core::ForwardedRefusal::NotCurrent)
        } else if !checks.is_within_budget {
            Some(super::core::ForwardedRefusal::Budget)
        } else if !checks.is_exact_model {
            Some(super::core::ForwardedRefusal::Malformed)
        } else {
            None
        };
        if let Some(refusal) = local {
            self.pending = None;
            self.refusal = Some(refusal);
            return None;
        }
        match self.admit_pending() {
            Ok(_) => Some(crate::candidate_worker::broker::GateTicket::unserved()),
            Err(refusal) => {
                self.refusal = Some(refusal);
                None
            }
        }
    }

    /// Settle through C3; a response is released only after `Recorded`.
    fn settle(
        &mut self,
        ticket: crate::candidate_worker::broker::GateTicket,
        upstream: crate::execution_binding::serving::gate::Upstream,
    ) -> Result<bool, String> {
        let (outcome, usage) = match &upstream {
            crate::execution_binding::serving::gate::Upstream::Responded(facts) => {
                (super::core::SettleOutcome::Completed, usage(facts))
            }
            crate::execution_binding::serving::gate::Upstream::TimedOut => (super::core::SettleOutcome::TimedOut, None),
            crate::execution_binding::serving::gate::Upstream::Cancelled => {
                (super::core::SettleOutcome::Cancelled, None)
            }
            crate::execution_binding::serving::gate::Upstream::TransportFailed => {
                (super::core::SettleOutcome::Uncertain, None)
            }
        };
        self.settle_outcome(ticket, outcome, usage).map(|_| true)
    }

    /// A local budget refusal makes no authority call.
    fn exhausted(&mut self) -> Result<(), String> {
        Ok(())
    }
}
