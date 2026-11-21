//! Bounded same-attempt repair: the admitted policy and its pure decisions.
//!
//! A repair policy is part of the frozen request and of the worker profile
//! revision. It names the check failure classes a repair may address, the
//! number of repair iterations, an elapsed-time bound and aggregate model and
//! tool budgets over every step of the attempt. Each iteration's candidate is
//! a new host-derived revision bound to the same attempt and fence, and every
//! declared check is rebound to it. Only the injected host check port's
//! observations decide whether a candidate passed; the worker's exit status,
//! its own claims and skipped or reconfigured checks never do. Independent
//! acceptance stays with Kiln #46, and changed work or a new semantic attempt
//! stays with Kiln.
//!
//! This module is pure: the shell supplies every observation.

// r[impl lattice.implementation_attempt.repair]
// r[impl lattice.implementation_attempt.repair.scenario.policy]

pub mod checks;
pub mod revision;
pub mod state;
pub mod step;

/// Wire schema of an admitted repair policy.
pub const POLICY_SCHEMA: &str = "lattice.implementation-attempt-repair-policy.v1";
/// Upper bound on repair iterations after the first candidate.
pub const MAX_REPAIR_ITERATIONS: u32 = 8;
/// Upper bound on declared checks.
pub const MAX_CHECKS: usize = 16;
/// Upper bound on admitted failure classes.
pub const MAX_FAILURE_CLASSES: usize = 16;
const MAX_NAME_BYTES: usize = 64;
const POLICY_DOMAIN: &str = "lattice.implementation-attempt-repair-policy.v1 identity";
const PROFILE_DOMAIN: &str = "lattice.worker-profile-revision-repair.v1 identity";

/// One required check and the configuration identity it must run with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub name: String,
    /// BLAKE3 of the check configuration as admitted.
    pub configuration: String,
}

/// The admitted repair policy of one attempt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema: String,
    /// Check failure classes a repair may address, strictly sorted. The
    /// reserved class `no_candidate` admits repairing a step that produced
    /// no new candidate.
    pub failure_classes: Vec<String>,
    /// Repair iterations after the first candidate; each step is one turn
    /// of the envelope budget.
    pub max_iterations: u32,
    /// Elapsed bound over every step and check of the attempt.
    pub max_elapsed_ms: u64,
    /// Aggregate model requests over every step.
    pub max_model_requests: u64,
    /// Aggregate tool invocations over every step and check execution.
    pub max_tool_invocations: u64,
    /// Tool invocations reserved for one worker step; charged when the
    /// step's own count is unknown or lower.
    pub step_tool_invocations: u64,
    /// BLAKE3 identity of the admitted host check executor.
    pub check_executor: String,
    /// Required checks, strictly sorted by name.
    pub checks: Vec<Check>,
}

/// The reserved failure class of a step without a new candidate.
pub const NO_CANDIDATE: &str = "no_candidate";

/// Why a repair policy is not admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    #[error("repair policy schema is not supported")]
    Schema,
    #[error("repair policy failure classes are empty, unsorted, duplicated or malformed")]
    FailureClasses,
    #[error("repair policy checks are empty, unsorted, duplicated or malformed")]
    Checks,
    #[error("repair policy iterations exceed the bound or the envelope's turns")]
    Iterations,
    #[error("repair policy elapsed bound is zero or exceeds the envelope's seconds")]
    Elapsed,
    #[error("repair policy budgets cannot cover one step and its checks")]
    Budget,
    #[error("repair policy check executor is malformed")]
    Executor,
    #[error("repair policy cannot be combined with an admitted unchanged proposal")]
    Unchanged,
}

/// The frozen per-step facts a policy is admitted against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame<'a> {
    pub budget: &'a super::AttemptBudget,
    /// The broker's request ceiling of one worker step.
    pub step_model_requests: u64,
    pub is_unchanged_allowed: bool,
}

impl Policy {
    /// Admit the policy against the envelope budget and the broker ceiling.
    pub fn validate(&self, frame: Frame<'_>) -> Result<(), PolicyError> {
        if self.schema != POLICY_SCHEMA {
            return Err(PolicyError::Schema);
        }
        if frame.is_unchanged_allowed {
            return Err(PolicyError::Unchanged);
        }
        let is_class_list_valid = self.failure_classes.len() <= MAX_FAILURE_CLASSES
            && is_strictly_sorted(self.failure_classes.iter().map(String::as_str));
        if !is_class_list_valid || !self.failure_classes.iter().all(|class| is_name(class)) {
            return Err(PolicyError::FailureClasses);
        }
        let is_check_list_valid =
            self.checks.len() <= MAX_CHECKS && is_strictly_sorted(self.checks.iter().map(|check| check.name.as_str()));
        if !is_check_list_valid || !self.checks.iter().all(Check::is_well_formed) {
            return Err(PolicyError::Checks);
        }
        if !super::is_blake3(&self.check_executor) {
            return Err(PolicyError::Executor);
        }
        let steps = self.max_iterations.checked_add(1).ok_or(PolicyError::Iterations)?;
        if self.max_iterations == 0 || self.max_iterations > MAX_REPAIR_ITERATIONS || steps > frame.budget.max_turns {
            return Err(PolicyError::Iterations);
        }
        let envelope_ms = frame.budget.max_seconds.checked_mul(1000).ok_or(PolicyError::Elapsed)?;
        if self.max_elapsed_ms == 0 || self.max_elapsed_ms > envelope_ms {
            return Err(PolicyError::Elapsed);
        }
        let first = self.reservation(frame.step_model_requests)?;
        if self.max_model_requests < first.model_requests || self.max_tool_invocations < first.tool_invocations {
            return Err(PolicyError::Budget);
        }
        Ok(())
    }

    /// The conservative reservation of one worker step and its checks.
    pub fn reservation(&self, step_model_requests: u64) -> Result<super::lineage::Draws, PolicyError> {
        let check_count = u64::try_from(self.checks.len()).map_err(|_| PolicyError::Checks)?;
        let tool_invocations = self.step_tool_invocations.checked_add(check_count).ok_or(PolicyError::Budget)?;
        if step_model_requests == 0 || self.step_tool_invocations == 0 {
            return Err(PolicyError::Budget);
        }
        Ok(super::lineage::Draws {
            model_requests: step_model_requests,
            tool_invocations,
        })
    }

    /// Domain-separated BLAKE3 identity of the complete policy.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut hasher = blake3::Hasher::new_derive_key(POLICY_DOMAIN);
        super::lineage::frame(&mut hasher, self.schema.as_bytes());
        super::lineage::frame_count(&mut hasher, self.failure_classes.len());
        for class in &self.failure_classes {
            super::lineage::frame(&mut hasher, class.as_bytes());
        }
        hasher.update(&self.max_iterations.to_be_bytes());
        for amount in [
            self.max_elapsed_ms,
            self.max_model_requests,
            self.max_tool_invocations,
            self.step_tool_invocations,
        ] {
            hasher.update(&amount.to_be_bytes());
        }
        super::lineage::frame(&mut hasher, self.check_executor.as_bytes());
        super::lineage::frame_count(&mut hasher, self.checks.len());
        for check in &self.checks {
            super::lineage::frame(&mut hasher, check.name.as_bytes());
            super::lineage::frame(&mut hasher, check.configuration.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    /// Whether a repair may address every one of `classes`.
    #[must_use]
    pub fn admits(&self, classes: &[String]) -> bool {
        !classes.is_empty() && classes.iter().all(|class| self.failure_classes.binary_search(class).is_ok())
    }
}

impl Check {
    fn is_well_formed(&self) -> bool {
        is_name(&self.name) && super::is_blake3(&self.configuration)
    }
}

/// The profile revision of a repair-capable profile: the single-shot
/// revision bound to the policy identity. A request without a policy keeps
/// its single-shot revision unchanged.
#[must_use]
pub fn profile_revision(single_shot: &str, policy: &Policy) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(PROFILE_DOMAIN);
    super::lineage::frame(&mut hasher, single_shot.as_bytes());
    super::lineage::frame(&mut hasher, policy.identity().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// The frozen attempt facts every step and candidate revision binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepBinding {
    pub envelope: String,
    pub work_id: String,
    pub work_version: u64,
    pub task_spec_digest: String,
    pub attempt_id: String,
    pub fence: u64,
}

impl StepBinding {
    #[must_use]
    pub fn of(envelope: &super::ImplementationAttemptEnvelope) -> Self {
        Self {
            envelope: envelope.identity(),
            work_id: envelope.work_id.clone(),
            work_version: envelope.work_version,
            task_spec_digest: envelope.task_spec_digest.clone(),
            attempt_id: envelope.attempt_id.clone(),
            fence: envelope.fence,
        }
    }

    /// Refuse a step that names other work or another semantic attempt.
    pub fn admit_step(&self, requested: &Self) -> Result<(), KilnOwned> {
        let is_same_work = self.work_id == requested.work_id
            && self.work_version == requested.work_version
            && self.task_spec_digest == requested.task_spec_digest;
        if !is_same_work {
            return Err(KilnOwned::ChangedWork);
        }
        let is_same_attempt = self.attempt_id == requested.attempt_id
            && self.fence == requested.fence
            && self.envelope == requested.envelope;
        if !is_same_attempt {
            return Err(KilnOwned::NewSemanticAttempt);
        }
        Ok(())
    }
}

/// A continuation only Kiln may admit; a repair loop refuses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KilnOwned {
    ChangedWork,
    NewSemanticAttempt,
}

fn is_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_NAME_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-'))
}

fn is_strictly_sorted<'a>(mut values: impl Iterator<Item = &'a str>) -> bool {
    let Some(mut previous) = values.next() else {
        return false;
    };
    for value in values {
        if value <= previous {
            return false;
        }
        previous = value;
    }
    true
}

#[cfg(test)]
mod tests;
