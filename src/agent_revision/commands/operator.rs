//! `agent-revision admit-session`: the operator's admission of one session.
//!
//! An implementation attempt's own session, its envelope identity, is
//! attempt-bound: only `lattice implementation-attempt run` admits it, under
//! the attempt's fence lock and behind its write-ahead record, so an operator
//! pin can never outlive, race or pre-empt the attempt's own admission. The
//! operator command refuses it before any state change. Any other session is
//! pinned after `run`'s request validation. The session check and the
//! admission use one read of the request.

// r[impl lattice.agent_revisions.workflow.attempt.scenario.operator]

type Request = crate::implementation_attempt::cli::Request;
type Decision = super::super::seam::Decision;

/// Why `admit-session` admits nothing, decided before any state change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AdmitRefusal {
    /// The session is the request's own attempt session.
    #[error(
        "attempt_session: an implementation attempt's own session (its envelope identity) is admitted only by \
         `lattice implementation-attempt run`"
    )]
    AttemptSession,
}

/// Whether the operator may admit `session` for `request`: never the
/// request's own attempt session.
pub fn check(request: &Request, session: &str) -> Result<(), AdmitRefusal> {
    if session == request.envelope.identity() {
        return Err(AdmitRefusal::AttemptSession);
    }
    Ok(())
}

/// Admit `session` for one already-read request: the session check, `run`'s
/// request validation and `seam::admit` all read this one parse. A refused
/// attempt session opens no store.
pub(crate) fn admit(request: &Request, store: &std::path::Path, session: &str) -> Result<Decision, String> {
    check(request, session).map_err(|refusal| refusal.to_string())?;
    request.validate(crate::implementation_attempt::cli::now_ms()?)?;
    let decision = super::super::seam::admit(&super::open(store)?, request, session)?;
    debug_assert_ne!(session, request.envelope.identity());
    Ok(decision)
}

/// The `admit-session` command: read the request once, admit, and print the
/// outcome document under the subject's definition.
pub(super) fn admit_command(
    request_path: &std::path::Path,
    store: &std::path::Path,
    session: &str,
) -> Result<serde_json::Value, String> {
    let request = super::super::bundle::load(request_path)?;
    let decision = admit(&request, store, session)?;
    let definition = super::super::seam::subject(&request)?.definition;
    let result = match decision {
        Decision::Refused { refusal } => Err(refusal),
        admitted @ (Decision::Unmanaged | Decision::Admitted { .. }) => Ok(admitted),
    };
    let header = super::Header {
        command: "admit-session",
        definition: &definition,
    };
    Ok(super::document(header, result))
}
