//! Pure decisions of the standalone CLI's operator-authorized repair: the
//! exact effects an operator profile must grant, the prompt and deadline of
//! one OMP step, and how the pinned OMP launcher's observation maps to the
//! loop's worker outcome.
//!
//! This module is pure: the shell evaluates the profile, launches the
//! worker and the checks and supplies every observation.

// r[impl lattice.implementation_attempt.operator_repair]
// r[impl lattice.implementation_attempt.operator_repair.scenario.outcomes]

/// The node of the Agent effect a profile grants the OMP repair worker.
pub const WORKER_NODE: u64 = 1;
/// The node of the process effect a profile grants the catalog checks.
pub const CHECK_NODE: u64 = 2;
const MACHINE_DOMAIN: &[u8] = b"lattice.implementation-attempt-operator-repair-worker.v1";
/// The host stops a worker at its deadline or output limit with `SIGKILL`;
/// such a step is reported with the shell's `128 + 9` status.
pub const HOST_STOP_STATUS: i32 = 137;

/// The workflow-revision identity a granting profile names: the request's
/// worker profile revision, which frames the pinned OMP executable, model,
/// harness flags and repair policy.
#[must_use]
pub fn workflow_revision(worker_profile: &str) -> String {
    debug_assert!(crate::implementation_attempt::is_blake3(worker_profile), "an admitted profile revision");
    format!("blake3:{worker_profile}")
}

/// The machine identity a granting profile names: the operator repair
/// worker of the bounded same-attempt repair loop.
#[must_use]
pub fn machine_identity() -> String {
    crate::engine::effect::authority::blake3_reference(MACHINE_DOMAIN, &[])
}

/// The exact Agent grant tuple of the OMP worker; no process, network,
/// file or other node's grant substitutes for it.
#[must_use]
pub fn worker_grant() -> crate::engine::effect::standalone::core::Grant {
    crate::engine::effect::standalone::core::Grant {
        node_id: WORKER_NODE,
        operation: "lattice/agent".to_owned(),
        resource: format!("lattice://agent/{WORKER_NODE}"),
        ability: "agent/execute".to_owned(),
    }
}

/// The exact process grant tuple of the catalog checks.
#[must_use]
pub fn check_grant() -> crate::engine::effect::standalone::core::Grant {
    crate::engine::effect::standalone::core::Grant {
        node_id: CHECK_NODE,
        operation: "lattice/shell".to_owned(),
        resource: format!("lattice://process/{CHECK_NODE}"),
        ability: "process/execute".to_owned(),
    }
}

/// The profile requirements an operator reviews for a request whose
/// worker profile revision is `worker_profile`; not a grant.
#[must_use]
pub fn requirements(worker_profile: &str) -> crate::engine::effect::standalone::ProfileInspection {
    crate::engine::effect::standalone::ProfileInspection {
        schema: crate::engine::effect::standalone::core::PROFILE_SCHEMA.to_owned(),
        workflow_revision: workflow_revision(worker_profile),
        machine_identity: machine_identity(),
        policy_ref: crate::engine::effect::authority::CHOREGRAPH_EFFECT_AUTHORITY_POLICY_REF.to_owned(),
        grants: vec![worker_grant(), check_grant()],
    }
}

/// The prompt of one step: the admitted prompt for the first step; for a
/// repair, the admitted prompt with the parent revision and the admitted
/// failure classes only, never a transcript or check output.
#[must_use]
pub fn prompt(admitted: &str, step: &super::Request) -> String {
    let Some(parent) = step.parent.as_deref().filter(|_| step.phase == super::Phase::Repair) else {
        return admitted.to_owned();
    };
    debug_assert!(!step.failures.is_empty(), "a repair names its admitted failure classes");
    format!(
        "{admitted}\n\nRepair step {iteration}: the candidate revision {parent} in this workspace failed the \
         admitted check classes {classes}. Repair it in place.",
        iteration = step.iteration,
        classes = step.failures.join(", "),
    )
}

/// The worker deadline in whole seconds: the time left before the elapsed
/// bound, rounded up, within the admission's own bound `admitted_secs`.
/// Rounding up lets a worker that runs to its deadline return at or past
/// the elapsed bound, which the loop then classifies as `budget_exhausted`
/// (`elapsed`) without checking the step.
#[must_use]
pub fn timeout_secs(deadline: std::time::Duration, admitted_secs: u64) -> u64 {
    let partial_secs = u64::from(deadline.subsec_nanos() > 0);
    deadline.as_secs().saturating_add(partial_secs).min(admitted_secs)
}

/// The loop's worker outcome of one launcher observation. Only an observed
/// stop is `exited` or `cancelled`; a launch the launcher blocked, an
/// interrupted or unknown teardown, an incomplete OMP terminal stream, a
/// write outside the boundary and a stop without an exit status are
/// `unknown`, so the loop ends `uncertain` and never retries them.
#[must_use]
pub fn outcome(observed: &crate::candidate_worker::Outcome) -> super::WorkerOutcome {
    let stop = crate::candidate_worker::omp::stop::of(observed);
    let is_failed = matches!(observed, crate::candidate_worker::Outcome::Failed { .. });
    debug_assert!(stop.is_none() || is_failed, "only a failed outcome has a stop");
    let outcome = match (observed, stop) {
        (crate::candidate_worker::Outcome::Produced { .. } | crate::candidate_worker::Outcome::Unchanged { .. }, _) => {
            super::WorkerOutcome::Exited { code: 0 }
        }
        (
            _,
            Some(crate::candidate_worker::omp::stop::Termination::Exited {
                status_code: Some(code),
            }),
        ) => super::WorkerOutcome::Exited { code },
        (
            _,
            Some(
                crate::candidate_worker::omp::stop::Termination::Deadline
                | crate::candidate_worker::omp::stop::Termination::OutputLimit,
            ),
        ) => super::WorkerOutcome::Exited { code: HOST_STOP_STATUS },
        (crate::candidate_worker::Outcome::Cancelled { .. }, _) => super::WorkerOutcome::Cancelled,
        _ => super::WorkerOutcome::Unknown,
    };
    debug_assert!(
        !matches!(outcome, super::WorkerOutcome::Exited { .. })
            || !matches!(observed, crate::candidate_worker::Outcome::Unknown { .. }),
        "an unknown launch is never an exit"
    );
    outcome
}

/// The retained label and blocker of one launcher observation.
#[must_use]
pub fn describe(observed: &crate::candidate_worker::Outcome) -> (&'static str, Option<&str>) {
    match observed {
        crate::candidate_worker::Outcome::Produced { .. } => ("produced", None),
        crate::candidate_worker::Outcome::Unchanged { .. } => ("unchanged", None),
        crate::candidate_worker::Outcome::Cancelled { .. } => ("cancelled", None),
        crate::candidate_worker::Outcome::Interrupted { .. } => ("interrupted", None),
        crate::candidate_worker::Outcome::Failed { blocker, .. } => ("failed", Some(blocker)),
        crate::candidate_worker::Outcome::Blocked { blocker } => ("blocked", Some(blocker)),
        crate::candidate_worker::Outcome::Unknown { blocker, .. } => ("unknown", Some(blocker)),
    }
}

#[cfg(test)]
mod tests;
