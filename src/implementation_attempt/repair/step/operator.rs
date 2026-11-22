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

/// The prompt of one step: the admitted prompt for the first step; for every
/// repair, the admitted prompt with the parent revision and what the loop
/// recorded of the previous step only (`no_candidate` as "produced no new
/// candidate", other admitted classes as failed check classes), never a
/// transcript or check output.
// r[impl lattice.implementation_attempt.operator_repair.scenario.repaired]
#[must_use]
pub fn prompt(admitted: &str, step: &super::Request) -> String {
    if step.phase != super::Phase::Repair {
        return admitted.to_owned();
    }
    debug_assert!(!step.failures.is_empty(), "a repair names its admitted failure classes");
    let mut text = format!("{admitted}\n\nRepair step {} of the same attempt.", step.iteration);
    if let Some(parent) = step.parent.as_deref() {
        text.push_str(&format!(" The workspace holds candidate revision {parent}."));
    }
    if step.failures.iter().any(|class| class == super::super::NO_CANDIDATE) {
        text.push_str(" The previous step produced no new candidate.");
    }
    let classes: Vec<&str> = step
        .failures
        .iter()
        .map(String::as_str)
        .filter(|class| *class != super::super::NO_CANDIDATE)
        .collect();
    if !classes.is_empty() {
        text.push_str(&format!(" That candidate failed the admitted check classes {}.", classes.join(", ")));
    }
    text.push_str(" Repair it in place.");
    text
}

/// The worker deadline in whole seconds: the time left before the elapsed
/// bound, rounded up, within the admission's own bound `admitted_secs`.
/// When the elapsed bound is the limit, a worker that runs to its deadline
/// returns at or past it, and the loop classifies the step `budget_exhausted`
/// (`elapsed`) without checking it. When the admission's bound is smaller,
/// the deadline is capped ([`is_capped`]) and such a stop is uncertain.
#[must_use]
pub fn timeout_secs(deadline: std::time::Duration, admitted_secs: u64) -> u64 {
    rounded_secs(deadline).min(admitted_secs)
}

/// Whether the admission's bound capped the worker below the rounded-up
/// elapsed deadline.
#[must_use]
pub fn is_capped(deadline: std::time::Duration, admitted_secs: u64) -> bool {
    admitted_secs < rounded_secs(deadline)
}

fn rounded_secs(deadline: std::time::Duration) -> u64 {
    let partial_secs = u64::from(deadline.subsec_nanos() > 0);
    deadline.as_secs().saturating_add(partial_secs)
}

/// The loop's worker outcome of one launcher observation. Only an observed
/// stop is `exited` or `cancelled`; a launch the launcher blocked, an
/// interrupted or unknown teardown, an incomplete OMP terminal stream, a
/// write outside the boundary, a stop without an exit status and, when the
/// admission capped the deadline (`is_capped`), a stop at that deadline are
/// `unknown`, so the loop ends `uncertain` and never retries or checks them.
/// A signal death inside the sandbox is an observed exit with bubblewrap's
/// `128 + N` status.
#[must_use]
pub fn outcome(observed: &crate::candidate_worker::Outcome, is_capped: bool) -> super::WorkerOutcome {
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
        (_, Some(crate::candidate_worker::omp::stop::Termination::Deadline)) if is_capped => {
            super::WorkerOutcome::Unknown
        }
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
