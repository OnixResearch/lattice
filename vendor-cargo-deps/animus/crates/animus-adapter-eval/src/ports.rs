//! Separate least-authority service ports.

use crate::{BindingError, ServiceKind, ServiceObservation, ServiceRequest};

macro_rules! request_type {
    ($name:ident, $kind:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name<'a> {
            service: ServiceRequest<'a>,
        }

        impl<'a> $name<'a> {
            /// Construct the typed request only for its exact service kind.
            ///
            /// # Errors
            /// Returns mismatch when a request for another service is supplied.
            pub fn new(service: ServiceRequest<'a>) -> Result<Self, BindingError> {
                if service.service() != $kind {
                    return Err(BindingError::Mismatch);
                }
                Ok(Self { service })
            }

            /// Return the exact generic service request.
            #[must_use]
            pub const fn service(&self) -> &ServiceRequest<'a> {
                &self.service
            }

            /// Consume the wrapper and return the exact generic request.
            #[must_use]
            pub fn into_service(self) -> ServiceRequest<'a> {
                self.service
            }
        }
    };
}

request_type!(
    TaskSourceRequest,
    ServiceKind::TaskSource,
    "Typed request for bounded task materialization."
);
request_type!(
    AgentExecutionRequest,
    ServiceKind::AgentExecution,
    "Typed request for one admitted agent run."
);
request_type!(
    VerifierExecutionRequest,
    ServiceKind::VerifierExecution,
    "Typed request for one independently identified verifier run."
);
request_type!(
    ArtifactCollectRequest,
    ServiceKind::ArtifactCollect,
    "Typed request for bounded artifact collection."
);
request_type!(
    ArtifactRestoreRequest,
    ServiceKind::ArtifactRestore,
    "Typed request for bounded artifact restoration."
);
request_type!(
    ModelCallCaptureRequest,
    ServiceKind::ModelCallCapture,
    "Typed request for bounded model-call metadata capture."
);
request_type!(
    ResultAppendRequest,
    ServiceKind::ResultAppend,
    "Typed request for append-only result storage."
);
request_type!(
    ResultReplaceRequest,
    ServiceKind::ResultReplace,
    "Typed request for exact result replacement."
);
request_type!(
    ResultReopenRequest,
    ServiceKind::ResultReopen,
    "Typed request for reopening an existing result."
);

/// Host-owned task-source port.
pub trait TaskSourcePort {
    /// Materialize bounded task facts without ambient discovery.
    fn materialize(&mut self, request: &TaskSourceRequest<'_>) -> ServiceObservation;
}

/// Host-owned agent-execution port.
pub trait AgentExecutionPort {
    /// Execute one admitted agent run.
    fn execute_agent(&mut self, request: &AgentExecutionRequest<'_>) -> ServiceObservation;
}

/// Host-owned verifier-execution port.
pub trait VerifierExecutionPort {
    /// Execute one independently bound verifier run.
    fn execute_verifier(&mut self, request: &VerifierExecutionRequest<'_>) -> ServiceObservation;
}

/// Host-owned artifact-collection port.
pub trait ArtifactCollectPort {
    /// Collect only the admitted artifact manifest.
    fn collect_artifacts(&mut self, request: &ArtifactCollectRequest<'_>) -> ServiceObservation;
}

/// Host-owned artifact-restore port.
pub trait ArtifactRestorePort {
    /// Restore only the admitted artifact manifest.
    fn restore_artifacts(&mut self, request: &ArtifactRestoreRequest<'_>) -> ServiceObservation;
}

/// Host-owned model-call-capture port.
pub trait ModelCallCapturePort {
    /// Capture bounded metadata for one exact model call.
    fn capture_model_call(&mut self, request: &ModelCallCaptureRequest<'_>) -> ServiceObservation;
}

/// Host-owned append-only result port.
pub trait ResultAppendPort {
    /// Append one exact result with write evidence.
    fn append_result(&mut self, request: &ResultAppendRequest<'_>) -> ServiceObservation;
}

/// Host-owned exact result replacement port.
pub trait ResultReplacePort {
    /// Replace one exact prior result with matching write evidence.
    fn replace_result(&mut self, request: &ResultReplaceRequest<'_>) -> ServiceObservation;
}

/// Host-owned result-reopen port.
pub trait ResultReopenPort {
    /// Reopen one exact existing result with matching evidence.
    fn reopen_result(&mut self, request: &ResultReopenRequest<'_>) -> ServiceObservation;
}
