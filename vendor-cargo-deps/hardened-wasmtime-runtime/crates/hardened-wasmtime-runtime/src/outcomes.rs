//! Stable outcome classes and phases for every bounded execution failure.
//!
//! Consumers compare typed classes and phases, never engine strings.
//! Implementation detail stays in optional diagnostics carried beside them.
// r[impl onix.wasmtime_runtime.normalized_outcomes]

/// Existing narrow outcome model from the profile-admission rung.
pub use crate::resource::admission::{
    DEADLINE_CLASS, INPUT_LIMIT_CLASS, MEMORY_LIMIT_CLASS, OUTPUT_LIMIT_CLASS,
};

/// Execution stage reached by one bounded invocation, independent of its class.
// r[impl onix.wasmtime_runtime.failure_phase]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvocationPhase {
    /// Input, concurrency and store admission, before consumer binding starts.
    Admission,
    /// Consumer import binding, before guest instantiation starts.
    Linking,
    /// Instantiation, including guest start functions and initializers.
    Instantiation,
    /// Export selection, execution and automatic guest post-return cleanup.
    Invocation,
}

/// Stable class for one completed or failed bounded invocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// The invocation finished within every bound.
    Completed,
    /// Profile admission denied the request before execution.
    ProfileDenied,
    /// Artifact shape or identity admission denied the request.
    ArtifactDenied,
    /// Compilation of the artifact failed.
    CompileFailed,
    /// Instantiation failed, including undeclared or WASI imports.
    InstantiateFailed,
    /// The guest consumed its admitted fuel.
    FuelExhausted,
    /// The invocation passed its epoch deadline.
    DeadlineExhausted,
    /// The guest exceeded a memory or table bound.
    MemoryLimitExceeded,
    /// The guest trapped.
    GuestTrapped,
    /// A host adapter failed; the guest is not at fault.
    HostFailed,
    /// The guest's input exceeded the admitted bound.
    InputLimitExceeded,
    /// The guest's output exceeded the admitted bound.
    OutputLimitExceeded,
    /// An invalid guest pointer, length, or overflow was rejected.
    GuestPointerRejected,
    /// The component returned its declared guest-error result.
    GuestDenied,
    /// The engine already has the admitted number of live invocations.
    ConcurrencyExhausted,
    /// The selected runtime cannot classify this failure more precisely.
    RuntimeFailed,
}

impl Class {
    /// Stable lowercase class name used in observations and comparisons.
    ///
    /// Keep this closed wire-name table in discriminant order when adding classes.
    #[must_use]
    pub const fn class_name(self) -> &'static str {
        const NAMES: [&str; 16] = [
            "completed",
            "profile-denied",
            "artifact-denied",
            "compile-failed",
            "instantiate-failed",
            "fuel-exhausted",
            "deadline-exhausted",
            "memory-limit-exceeded",
            "guest-trapped",
            "host-failed",
            "input-limit-exceeded",
            "output-limit-exceeded",
            "guest-pointer-rejected",
            "guest-denied",
            "concurrency-exhausted",
            "runtime-failed",
        ];
        NAMES[self as usize]
    }

    /// Whether the class represents a completed bounded execution.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }
}
