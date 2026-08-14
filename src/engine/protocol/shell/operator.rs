//! Typed application requests for protocol operator surfaces.
//!
//! r[impl lattice.role_protocol.diagnostics]
//! r[impl lattice.role_protocol.evidence]
#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the typed operator shell uses its adjacent protocol DTO and runtime vocabulary"
)]

use super::super::model::ActionRequest;
use super::super::model::Error;
use super::super::model::Identity;
use super::super::model::Issue;
use super::super::model::SessionPlan;
use super::Runtime;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Create {
        plan: Box<SessionPlan>,
    },
    Inspect {
        session: Identity,
    },
    Dispatch {
        session: Identity,
        request: ActionRequest,
    },
    Recover {
        session: Identity,
        attempt: Identity,
    },
    Retry {
        session: Identity,
        attempt: Identity,
        acknowledge_duplicate_risk: bool,
    },
    Abandon {
        session: Identity,
        attempt: Identity,
        acknowledge_unknown: bool,
    },
    Replay {
        session: Identity,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Output {
    Created {
        report: super::super::evidence::SessionInspection,
    },
    Inspected {
        report: super::super::evidence::SessionInspection,
    },
    Dispatched {
        report: super::super::evidence::SessionInspection,
        attempt: Identity,
        dispatched_messages: usize,
    },
    Settled {
        report: super::super::evidence::SessionInspection,
    },
    RetryPlanned {
        report: super::super::evidence::SessionInspection,
        attempt: Identity,
    },
    Abandoned {
        report: super::super::evidence::SessionInspection,
    },
    Replayed {
        report: super::super::evidence::SessionInspection,
    },
}

#[allow(
    function_length,
    reason = "one exhaustive command dispatcher keeps every operator action visible and fail closed"
)]
pub fn execute(shell: &Runtime<'_>, command: Command) -> Result<Output, Error> {
    match command {
        Command::Create { plan } => {
            let session = shell.create(*plan)?;
            Ok(Output::Created {
                report: super::super::evidence::inspect(&session, &[])?,
            })
        }
        Command::Inspect { session } => {
            let session = load(shell, &session)?;
            Ok(Output::Inspected {
                report: super::super::evidence::inspect(&session, &[])?,
            })
        }
        Command::Dispatch { session, request } => {
            let session = load(shell, &session)?;
            let dispatched = shell.dispatch(&session, request)?;
            Ok(Output::Dispatched {
                report: super::super::evidence::inspect(&dispatched.session, &[])?,
                attempt: dispatched.attempt,
                dispatched_messages: dispatched.dispatched_messages,
            })
        }
        Command::Recover { session, attempt } => {
            let session = load(shell, &session)?;
            let settled = shell.settle(&session, &attempt)?;
            Ok(Output::Settled {
                report: super::super::evidence::inspect(&settled, &[])?,
            })
        }
        Command::Retry {
            session,
            attempt,
            acknowledge_duplicate_risk,
        } => {
            let session = load(shell, &session)?;
            let recovery = super::super::core::retry_unknown(&session, &attempt, acknowledge_duplicate_risk)?;
            let event = recovery.recovered.events.last().ok_or(Issue::PersistenceFailed)?;
            shell
                .repository()
                .commit(session.version, &recovery.recovered, event)
                .map_err(|_| Issue::PersistenceFailed)?;
            let prepared_event = recovery.prepared.events.last().ok_or(Issue::PersistenceFailed)?;
            shell
                .repository()
                .commit(recovery.recovered.version, &recovery.prepared, prepared_event)
                .map_err(|_| Issue::PersistenceFailed)?;
            let attempt = recovery
                .prepared
                .attempts
                .last()
                .map(|attempt| attempt.identity.clone())
                .ok_or(Issue::PersistenceFailed)?;
            Ok(Output::RetryPlanned {
                report: super::super::evidence::inspect(&recovery.prepared, &[])?,
                attempt,
            })
        }
        Command::Abandon {
            session,
            attempt,
            acknowledge_unknown,
        } => {
            let session = load(shell, &session)?;
            let abandoned = shell.abandon(&session, &attempt, acknowledge_unknown)?;
            Ok(Output::Abandoned {
                report: super::super::evidence::inspect(&abandoned, &[])?,
            })
        }
        Command::Replay { session } => {
            let replayed = shell.replay(&session)?;
            Ok(Output::Replayed {
                report: super::super::evidence::inspect(&replayed, &[])?,
            })
        }
    }
}

fn load(shell: &Runtime<'_>, identity: &Identity) -> Result<super::super::model::SessionRecord, Error> {
    shell
        .repository()
        .load(identity)
        .map_err(|error| match error {
            super::StoreError::InvalidRecord => Issue::IdentityMismatch,
            super::StoreError::Conflict | super::StoreError::Unavailable => Issue::PersistenceFailed,
        })?
        .ok_or_else(|| Issue::MissingRole.into())
}
