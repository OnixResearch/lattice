//! Epoch-deadline admission.
//!
//! The shell owns the host epoch service; the core admits the deadline
//! budget bound to an invocation and the outcome class it produces.
//! An invocation that requires epoch interruption without a current host
//! epoch service is denied before execution.
// r[impl onix.wasmtime_runtime.epoch_deadline]

/// Outcome class produced when an invocation passes its epoch deadline.
pub const DEADLINE_EXHAUSTED_CLASS: &str = "deadline-exhausted";

/// An epoch-deadline budget bound to one invocation. The shell converts
/// this into an injected epoch deadline; the core never reads clocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    /// Number of Wasmtime epoch ticks the invocation may consume.
    pub budget_ticks: u64,
}

/// Epoch-deadline admission failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// The budget must be positive; a zero budget cannot make progress.
    ZeroBudget,
}

/// Admit an epoch-deadline budget.
pub fn admit(budget: Budget) -> Result<Budget, Error> {
    if budget.budget_ticks == 0 {
        return Err(Error::ZeroBudget);
    }
    Ok(budget)
}

/// The host epoch service the shell requires when a profile uses epoch
/// interruption. The core only ever sees the evidence that the service
/// exists; construction stays in the shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServicePresent;

/// Deny an invocation whose profile requires epoch interruption but whose
/// execution environment provides no epoch service.
pub fn require(service: Option<ServicePresent>) -> Result<(), Error> {
    if service.is_some() {
        Ok(())
    } else {
        Err(Error::ZeroBudget)
    }
}
