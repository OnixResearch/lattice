mod failure;
mod metadata;

pub use failure::AdapterCallFailureClass;

// r[impl agent_harness_runtime.boundary_versions.contract_identities]
// r[impl agent_harness_runtime.adapter_outcomes.typed_envelope]
/// Stable contract identity for typed adapter-call outcome envelopes.
pub const ADAPTER_CALL_OUTCOME_CONTRACT_IDENTITY: &str = "animus.adapter-call-outcome.v1";

/// Borrowed safe metadata for a non-completed adapter-call outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterCallFailureMetadataDraft<'a> {
    /// Bounded safe message or reason class.
    pub message: &'a str,
    /// Optional safe request or call correlation identifier.
    pub correlation_id: Option<&'a str>,
    /// Optional safe receipt/reference metadata, such as a receipt hash label.
    pub receipt_reference: Option<&'a str>,
}

/// Validated safe metadata for a non-completed adapter-call outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterCallFailureMetadata {
    message: alloc::string::String,
    correlation_id: Option<alloc::string::String>,
    receipt_reference: Option<alloc::string::String>,
}

impl AdapterCallFailureMetadata {
    /// Validates safe adapter-call failure metadata.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when metadata is empty, too long,
    /// contains a NUL byte, or carries known raw prompt, provider, credential,
    /// path, model-output, or tool-output leak markers.
    pub fn new(
        draft: AdapterCallFailureMetadataDraft<'_>,
    ) -> Result<Self, crate::AdapterContractError> {
        let message = metadata::required_message(draft.message)?;
        let correlation_id = metadata::optional_correlation_id(draft.correlation_id)?;
        let receipt_reference = metadata::optional_receipt_reference(draft.receipt_reference)?;
        Ok(Self {
            message,
            correlation_id,
            receipt_reference,
        })
    }

    /// Returns the bounded safe failure message.
    #[must_use]
    pub fn message(&self) -> &str {
        self.message.as_str()
    }

    /// Returns the optional safe correlation identifier.
    #[must_use]
    pub fn correlation_id(&self) -> Option<&str> {
        self.correlation_id.as_deref()
    }

    /// Returns the optional safe receipt/reference metadata.
    #[must_use]
    pub fn receipt_reference(&self) -> Option<&str> {
        self.receipt_reference.as_deref()
    }

    fn fallback() -> Self {
        Self {
            message: alloc::string::String::from(
                AdapterCallFailureClass::ExecutionFailure.as_str(),
            ),
            correlation_id: None,
            receipt_reference: None,
        }
    }
}

/// Typed adapter-call envelope preserving completed values and fail-closed classes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterCallOutcome<Response> {
    /// Adapter call completed and returned a validated response DTO.
    Completed {
        /// Validated response.
        response: Response,
    },
    /// Adapter call failed closed before a completed response existed.
    Failed {
        /// Stable fail-closed class.
        class: AdapterCallFailureClass,
        /// Bounded safe failure metadata.
        metadata: AdapterCallFailureMetadata,
    },
}

impl<Response> AdapterCallOutcome<Response> {
    /// Builds a completed adapter-call envelope.
    #[must_use]
    pub const fn completed(response: Response) -> Self {
        Self::Completed { response }
    }

    /// Builds a non-completed adapter-call envelope after metadata validation.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when failure metadata is unsafe.
    pub fn failed(
        class: AdapterCallFailureClass,
        metadata: AdapterCallFailureMetadataDraft<'_>,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self::Failed {
            class,
            metadata: AdapterCallFailureMetadata::new(metadata)?,
        })
    }

    /// Builds a fallback fail-closed envelope with a fixed safe failure message.
    #[must_use]
    pub fn fallback_failed(class: AdapterCallFailureClass) -> Self {
        Self::Failed {
            class,
            metadata: AdapterCallFailureMetadata::fallback(),
        }
    }

    /// Returns the stable outcome contract identity.
    #[must_use]
    pub const fn contract_identity() -> &'static str {
        ADAPTER_CALL_OUTCOME_CONTRACT_IDENTITY
    }

    /// Returns true only when the adapter call completed.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }

    /// Returns the fail-closed class for non-completed outcomes.
    #[must_use]
    pub const fn failure_class(&self) -> Option<AdapterCallFailureClass> {
        match self {
            Self::Completed { .. } => None,
            Self::Failed { class, .. } => Some(*class),
        }
    }
}

// r[impl agent_harness_runtime.adapter_outcomes.compatibility]
/// Opt-in adapter port that returns a typed outcome envelope instead of prose-only errors.
pub trait ComponentOutcomeAdapter<Request, Response>:
    crate::ComponentAdapter<Request, Response>
{
    /// Calls the component and returns a typed completed or fail-closed outcome.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`] for transport-level failures before a typed
    /// adapter outcome can be produced.
    fn call_with_outcome(
        &mut self,
        request: &Request,
    ) -> Result<AdapterCallOutcome<Response>, Self::Error>;
}

/// Opt-in model adapter contract with typed adapter-call outcomes.
pub trait ModelOutcomeAdapter:
    ComponentOutcomeAdapter<crate::ModelRequest, crate::ModelResponse>
{
}

impl<T> ModelOutcomeAdapter for T where
    T: ComponentOutcomeAdapter<crate::ModelRequest, crate::ModelResponse>
{
}

/// Opt-in tool adapter contract with typed adapter-call outcomes.
pub trait ToolOutcomeAdapter:
    ComponentOutcomeAdapter<crate::ToolRequest, crate::ToolResponse>
{
}

impl<T> ToolOutcomeAdapter for T where
    T: ComponentOutcomeAdapter<crate::ToolRequest, crate::ToolResponse>
{
}

#[cfg(test)]
mod tests;
