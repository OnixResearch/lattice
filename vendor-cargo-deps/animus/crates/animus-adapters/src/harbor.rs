//! Implementation-free ports for the optional Harbor-compatible evaluation lane.

/// Exact reviewed Harbor release.
pub const HARBOR_RELEASE: &str = "v0.18.0";
/// Exact reviewed Harbor commit.
pub const HARBOR_REVISION: &str = "527d50deb63a5d279e8c20593c18a2cbc7f61f9e";
/// Exact reviewed task schema.
pub const HARBOR_TASK_SCHEMA: &str = "1.3";
/// Exact reviewed trajectory schema.
pub const ATIF_SCHEMA: &str = "ATIF-v1.7";

/// One admitted evaluation trial request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrialRequest {
    /// Stable trial identity.
    pub trial_identity: alloc::string::String,
    /// Exact task identity.
    pub task_identity: alloc::string::String,
    /// Exact agent identity.
    pub agent_identity: alloc::string::String,
    /// Exact model-route identity.
    pub model_route_identity: alloc::string::String,
    /// Exact environment identity.
    pub environment_identity: alloc::string::String,
    /// Exact verifier identity.
    pub verifier_identity: alloc::string::String,
    /// Bounded artifact identities.
    pub artifacts: alloc::vec::Vec<alloc::string::String>,
}

/// Bounded observed outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeClass {
    /// Service completed and supplied typed observations.
    Completed,
    /// Service was unavailable.
    Unavailable,
    /// Policy denied execution.
    Denied,
    /// Cancellation was observed.
    Cancelled,
    /// Deadline was observed.
    TimedOut,
    /// Output was malformed.
    Malformed,
    /// Evidence was stale or mismatched.
    Stale,
}

/// Normalized trial observation without raw payloads.
#[derive(Clone, Debug, PartialEq)]
pub struct TrialObservation {
    /// Stable trial identity.
    pub trial_identity: alloc::string::String,
    /// Outcome class.
    pub outcome: OutcomeClass,
    /// ATIF projection identity.
    pub trajectory_identity: Option<alloc::string::String>,
    /// Finite attributed rewards.
    pub rewards: alloc::vec::Vec<(alloc::string::String, f64)>,
    /// Bounded artifact identities.
    pub artifacts: alloc::vec::Vec<alloc::string::String>,
}

/// Environment preparation and trial execution port.
pub trait EvaluationEnvironmentService {
    /// Execute one admitted trial without ambient fallback.
    ///
    /// # Errors
    ///
    /// Returns an explicit outcome class for every unavailable or failed path.
    fn execute(&mut self, request: &TrialRequest) -> Result<TrialObservation, OutcomeClass>;
}

/// Independently controlled verifier port.
pub trait VerifierService {
    /// Verify one admitted subject through an explicit artifact manifest.
    ///
    /// # Errors
    ///
    /// Returns an explicit outcome class without replacing the selected verifier.
    fn verify(&mut self, request: &TrialRequest) -> Result<TrialObservation, OutcomeClass>;
}
