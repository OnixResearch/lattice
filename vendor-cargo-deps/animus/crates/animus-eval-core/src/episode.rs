//! Episode reduction over canonical ledger references.

use alloc::{format, string::String, vec::Vec};

use crate::{RecipePlan, hash_identity, validate_identity};

/// Terminal run classification supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalObservationClass {
    /// Run completed under its admitted policy.
    Completed,
    /// Run was cancelled.
    Cancelled,
    /// Run failed without a successful result.
    Failed,
    /// Run reached an admitted deadline.
    TimedOut,
}

/// Exact reference to a canonical Animus ledger leaf.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerTraceReference<'a> {
    /// Ledger session identity.
    pub session: &'a str,
    /// Ledger branch identity.
    pub branch: &'a str,
    /// Ledger leaf identity.
    pub leaf: &'a str,
    /// Immutable subject identity.
    pub subject: &'a str,
    /// Semantic role identity.
    pub role: &'a str,
    /// Evaluation run identity.
    pub run: &'a str,
    /// Ledger policy identity.
    pub policy: &'a str,
}

/// Bounded terminal observation for one planned run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EpisodeRunObservation<'a> {
    /// Exact run identity.
    pub run_identity: &'a str,
    /// Exact planned seat identity.
    pub seat_identity: &'a str,
    /// Canonical ledger reference.
    pub ledger: LedgerTraceReference<'a>,
    /// Terminal class.
    pub terminal: TerminalObservationClass,
    /// Optional identity of an admitted metadata-only safe summary.
    pub safe_summary_identity: Option<&'a str>,
}

/// One validated episode run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpisodeRun {
    /// Run identity.
    pub run_identity: String,
    /// Seat identity.
    pub seat_identity: String,
    /// Canonical ledger-reference identity.
    pub ledger_reference_identity: String,
    /// Terminal class.
    pub terminal: TerminalObservationClass,
    /// Stable plan order.
    pub order: u32,
}

/// Validated multi-agent episode metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationEpisode {
    /// Canonical episode identity.
    pub identity: String,
    /// Exact recipe plan identity.
    pub plan_identity: String,
    /// Ordered run references.
    pub runs: Vec<EpisodeRun>,
    /// Number of completed runs.
    pub completed_count: u32,
}

/// Episode reduction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EpisodeError {
    /// An identity or safe summary identity is malformed.
    Identity,
    /// An observation is missing, duplicated, or out of plan order.
    Sequence,
    /// A ledger reference does not bind its planned run.
    LedgerBinding,
    /// A run is not terminal.
    Incomplete,
    /// A count exceeds the supported range.
    Bound,
}

/// Reduce ordered terminal observations into an episode of ledger references.
///
/// This contract has no field for transcript text, reasoning, tool bodies, or
/// provider payloads. The referenced ledger remains the replay authority.
///
/// # Errors
/// Returns a deterministic identity, sequence, binding, incomplete, or bound class.
pub fn reduce_episode(
    plan: &RecipePlan,
    observations: &[EpisodeRunObservation<'_>],
) -> Result<EvaluationEpisode, EpisodeError> {
    if observations.len() != plan.seats.len() || observations.is_empty() {
        return Err(EpisodeError::Sequence);
    }
    let mut runs = Vec::with_capacity(observations.len());
    let mut completed_count = 0u32;
    for (seat, observation) in plan.seats.iter().zip(observations) {
        if observation.run_identity != seat.run_identity
            || observation.seat_identity != seat.seat_identity
        {
            return Err(EpisodeError::Sequence);
        }
        validate_observation(observation)?;
        if observation.ledger.run != observation.run_identity {
            return Err(EpisodeError::LedgerBinding);
        }
        if observation.terminal == TerminalObservationClass::Completed {
            completed_count = completed_count.checked_add(1).ok_or(EpisodeError::Bound)?;
        }
        let order = u32::try_from(runs.len()).map_err(|_| EpisodeError::Bound)?;
        let ledger_reference_identity = ledger_reference_identity(&observation.ledger);
        runs.push(EpisodeRun {
            run_identity: observation.run_identity.into(),
            seat_identity: observation.seat_identity.into(),
            ledger_reference_identity,
            terminal: observation.terminal,
            order,
        });
    }
    let run_material = runs
        .iter()
        .map(|run| run.ledger_reference_identity.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let completed_material = format!("{completed_count}");
    let identity = hash_identity(
        "animus.native-evaluation.episode.v1",
        &[&plan.identity, &run_material, &completed_material],
    );
    Ok(EvaluationEpisode {
        identity,
        plan_identity: plan.identity.clone(),
        runs,
        completed_count,
    })
}

fn validate_observation(observation: &EpisodeRunObservation<'_>) -> Result<(), EpisodeError> {
    for identity in [
        observation.run_identity,
        observation.seat_identity,
        observation.ledger.session,
        observation.ledger.branch,
        observation.ledger.leaf,
        observation.ledger.subject,
        observation.ledger.role,
        observation.ledger.run,
        observation.ledger.policy,
    ] {
        validate_identity(identity).map_err(|_| EpisodeError::Identity)?;
    }
    if let Some(summary) = observation.safe_summary_identity {
        validate_identity(summary).map_err(|_| EpisodeError::Identity)?;
    }
    Ok(())
}

fn ledger_reference_identity(reference: &LedgerTraceReference<'_>) -> String {
    hash_identity(
        "animus.native-evaluation.ledger-reference.v1",
        &[
            reference.session,
            reference.branch,
            reference.leaf,
            reference.subject,
            reference.role,
            reference.run,
            reference.policy,
        ],
    )
}
